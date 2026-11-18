"""Exercise 2K retained precision, rational delivery, and a seam-crossing tile pack."""
from pathlib import Path
import argparse, datetime, hashlib, json, struct, subprocess, uuid, zlib
ROOT=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser()
parser.add_argument('--synth',type=Path,default=ROOT/'target-prod/debug/synth.exe')
EXE=parser.parse_args().synth
run=ROOT/'production'/('precision-'+datetime.datetime.now().strftime('%Y%m%d-%H%M%S')+'-'+uuid.uuid4().hex[:8]);run.mkdir(parents=True)
def chunk(tag,data):return struct.pack('>I',len(data))+tag+data+struct.pack('>I',zlib.crc32(tag+data)&0xffffffff)
def png(width,height,bits,rows):
    return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,height,bits,6,0,0,0))+chunk(b'sRGB',b'\0')+chunk(b'IDAT',zlib.compress(b''.join(b'\0'+row for row in rows)))+chunk(b'IEND',b'')
row=b''.join(struct.pack('>4H',x*31+1,x*31+2,x*31+3,65535) for x in range(2048))
(run/'gradient.png').write_bytes(png(2048,2048,16,[row]*2048))
for tile in range(2):
    tile_row=b''.join(bytes(((x+tile*512)//4%256,80,(x+tile*512)%251,255)) for x in range(512))
    (run/f'tile{tile}.png').write_bytes(png(512,64,8,[tile_row]*64))
source='''requires [capability("capability.invoke")];
let image=capability.invoke("hypermedia.radiance.import_rgba_png",{input:"gradient.png",profile:"16-bit"});
let original=capability.invoke("hypermedia.radiance.encode_rgba_png",{image:image,profile:"16-bit"});
let graded=capability.invoke("hypermedia.radiance.grade_color",{image:image,gain:[0.95,0.98,1.0]});
let sequence=capability.invoke("hypermedia.chronicle.capture_deterministic_movie",{frames:[graded,graded],fps:{numerator:24000,denominator:1001},profile:"png-sequence"});
let left=capability.invoke("hypermedia.radiance.import_rgba_png",{input:"tile0.png"});
let right=capability.invoke("hypermedia.radiance.import_rgba_png",{input:"tile1.png"});
let tiles=capability.invoke("hypermedia.radiance.pack_sprite_sheet",{tiles:[left,right],columns:2,padding:0});
let asset={files:[{key:"retained.png",resource:original},{key:"sequence.json",resource:sequence.manifest_resource},{key:"tiles.png",resource:tiles.png}]};
'''
(run/'precision.vibe').write_text(source,encoding='utf-8')
command=[str(EXE),'produce','--quality-profile','production16','--source',str(run/'precision.vibe'),'--out',str(run/'precision.hmc'),'--export-dir',str(run/'export')]
for name in ['gradient.png','tile0.png','tile1.png']:command+=['--input',name,'png',str(run/name)]
result=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=600)
(run/'produce.log').write_text(result.stdout+'\n'+result.stderr,encoding='utf-8')
checks={'production':result.returncode==0,'command':command}
def first_row(path):
    data=path.read_bytes();offset=8;payload=b'';metadata={}
    while offset<len(data):
        length=struct.unpack_from('>I',data,offset)[0];tag=data[offset+4:offset+8];body=data[offset+8:offset+8+length];offset+=length+12
        if tag==b'IHDR':w,h,bits,color,_,_,interlace=struct.unpack('>IIBBBBB',body);assert color==6 and interlace==0
        if tag==b'IDAT':payload+=body
        if tag in [b'sRGB',b'gAMA']:metadata[tag.decode()]=body.hex()
    raw=zlib.decompress(payload);kind=raw[0];stride=bits//8*4;row=bytearray(raw[1:1+w*stride])
    # First scanline has a zero previous row: Paeth chooses its left sample.
    for i in range(len(row)):
        left=row[i-stride] if i>=stride else 0
        if kind in [1,4]:row[i]=(row[i]+left)&255
        elif kind==3:row[i]=(row[i]+left//2)&255
        else:assert kind in [0,2]
    return w,h,bits,bytes(row),metadata
if result.returncode==0:
    w,h,bits,retained,metadata=first_row(run/'export/retained.png')
    checks.update(dimensions=[w,h],bit_depth=bits,lossless_first_row=retained==row,retained_transfer=metadata)
    manifest=json.loads((run/'export/sequence.json').read_text())
    checks['rational_timing']=manifest['frame_rate']=={'numerator':24000,'denominator':1001} and manifest['frames'][1]['start']=={'numerator':1001,'denominator':24000}
    frame=run/'export'/manifest['frames'][0]['path'];_,_,_,pixels,_=first_row(frame)
    distinct=len({struct.unpack_from('>H',pixels,i)[0] for i in range(0,len(pixels),8)})
    checks['graded_distinct_red_values']=distinct;checks['beyond_8bit']=distinct>256
    checks['frame_hashes']=all(hashlib.sha256((run/'export'/f['path']).read_bytes()).hexdigest()==f['sha256'] for f in manifest['frames'])
    tw,th,tb,tile_pixels,_=first_row(run/'export/tiles.png')
    expected=b''.join(bytes((x//4%256,80,x%251,255)) for x in range(1024))
    checks['tile_continuity']=tw==1024 and th==64 and tb==8 and tile_pixels==expected
    checks['passed']=all(checks.get(key) is True for key in ['production','lossless_first_row','rational_timing','beyond_8bit','frame_hashes','tile_continuity'])
else:checks['passed']=False;checks['error']=result.stderr[-2500:]
(run/'results.json').write_text(json.dumps(checks,indent=2),encoding='utf-8')
print(json.dumps(checks,indent=2));print(run)
raise SystemExit(0 if checks['passed'] else 1)
