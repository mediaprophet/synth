"""Exercise the same retained fixture through local CLI and MCP adapters.

Run only after the implementation gate. Native images require separate review.
No model provider, network service, live MCP process or user config is changed.
"""
from pathlib import Path
import argparse, datetime, hashlib, json, os, struct, subprocess, uuid, zlib

ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser()
p.add_argument('--synth',type=Path,default=ROOT/'target-gap-review/debug/synth.exe')
args=p.parse_args()
run=ROOT/'production'/('client-parity-'+datetime.datetime.now().strftime('%Y%m%d-%H%M%S')+'-'+uuid.uuid4().hex[:8])
run.mkdir(parents=True)
env=dict(os.environ,SYNTH_BIN=str(args.synth),SYNTH_INPUT_ROOT=str(run),SYNTH_OUTPUT_ROOT=str(run))
results=[]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def save(): (run/'results.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
def check(name,passed,**details):
    results.append(dict(name=name,passed=bool(passed),**details));save();print(name,passed,flush=True)
def execute(argv,name,**kwargs):
    result=subprocess.run(list(map(str,argv)),cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=600,**kwargs)
    (run/(name+'.log')).write_text(result.stdout+'\n'+result.stderr,encoding='utf-8')
    return result
def chunk(tag,payload):return struct.pack('>I',len(payload))+tag+payload+struct.pack('>I',zlib.crc32(tag+payload)&0xffffffff)
def png():
    pixels=b''.join(b'\0'+b''.join(bytes((225,180,70,255) if (x//8+y//8)%2 else (25,80,125,255)) for x in range(64)) for y in range(64))
    return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',64,64,8,6,0,0,0))+chunk(b'sRGB',b'\0')+chunk(b'IDAT',zlib.compress(pixels))+chunk(b'IEND',b'')
image=png();(run/'checker.png').write_bytes(image)

# A seamed cube has authored face normals, tangent handedness and one texture.
faces=[([0,0,1],[1,0,0],[( -1,-1,1),(1,-1,1),(1,1,1),(-1,1,1)]),
       ([0,0,-1],[-1,0,0],[(1,-1,-1),(-1,-1,-1),(-1,1,-1),(1,1,-1)]),
       ([1,0,0],[0,0,-1],[(1,-1,1),(1,-1,-1),(1,1,-1),(1,1,1)]),
       ([-1,0,0],[0,0,1],[(-1,-1,-1),(-1,-1,1),(-1,1,1),(-1,1,-1)]),
       ([0,1,0],[1,0,0],[(-1,1,1),(1,1,1),(1,1,-1),(-1,1,-1)]),
       ([0,-1,0],[1,0,0],[(-1,-1,-1),(1,-1,-1),(1,-1,1),(-1,-1,1)])]
positions=[];normals=[];uv=[];tangents=[];indices=[]
for i,(normal,tangent,corners) in enumerate(faces):
    positions.extend(v for pt in corners for v in pt);normals.extend(normal*4)
    uv.extend([0,0,1,0,1,1,0,1]);tangents.extend((tangent+[1])*4)
    indices.extend(i*4+j for j in [0,1,2,0,2,3])
chunks=[struct.pack('<72f',*positions),struct.pack('<72f',*normals),struct.pack('<48f',*uv),struct.pack('<96f',*tangents),struct.pack('<36H',*indices),image]
binary=b'';views=[]
for payload in chunks:
    binary+=b'\0'*((-len(binary))%4)
    views.append(dict(buffer=0,byteOffset=len(binary),byteLength=len(payload)));binary+=payload
accessors=[dict(bufferView=i,componentType=5126,count=24,type=typ) for i,typ in enumerate(['VEC3','VEC3','VEC2','VEC4'])]
accessors[0].update(min=[-1,-1,-1],max=[1,1,1]);accessors.append(dict(bufferView=4,componentType=5123,count=36,type='SCALAR'))
doc=dict(asset=dict(version='2.0'),buffers=[dict(byteLength=len(binary))],bufferViews=views,accessors=accessors,
    images=[dict(bufferView=5,mimeType='image/png')],textures=[dict(source=0)],
    materials=[dict(pbrMetallicRoughness=dict(baseColorFactor=[1,1,1,1],baseColorTexture=dict(index=0),metallicFactor=0.0,roughnessFactor=0.35))],
    meshes=[dict(primitives=[dict(attributes=dict(POSITION=0,NORMAL=1,TEXCOORD_0=2,TANGENT=3),indices=4,material=0)])],
    nodes=[dict(mesh=0)],scenes=[dict(nodes=[0])],scene=0)
j=json.dumps(doc,separators=(',',':')).encode();j+=b' '*((-len(j))%4);binary+=b'\0'*((-len(binary))%4)
(run/'fixture.glb').write_bytes(struct.pack('<III',0x46546c67,2,28+len(j)+len(binary))+struct.pack('<II',len(j),0x4e4f534a)+j+struct.pack('<II',len(binary),0x004e4942)+binary)
source='''requires [capability("capability.invoke")];
let mesh=capability.invoke("hypermedia.manifold.import_mesh",{input:"fixture.glb",format:"glb"});
let sealed=capability.invoke("hypermedia.manifold.compile_10d_pbr",{mesh:mesh});
let image=capability.invoke("hypermedia.radiance.import_rgba_png",{input:"checker.png"});
let png=capability.invoke("hypermedia.radiance.encode_rgba_png",{image:image});
let layout=capability.invoke("hypermedia.layout.solve_box_tree",{width:512.0,height:128.0,boxes:[{id:"label",role:"header",label:"LOCAL ASSET REVIEW",state:"focus"}]});
let theme=capability.invoke("hypermedia.layout.bind_theme_scope",{scope:"app",mode:"holosonic-dark"});
let ui=capability.invoke("hypermedia.layout.project_ui_svg",{layout:layout,theme:theme});
let asset={files:[{key:"fixture.10d",resource:sealed},{key:"checker.png",resource:png},{key:"label.svg",resource:ui}]};
'''
# One authoritative UTF-8 source byte stream for CLI, MCP and Grok.
(run/'fixture.vibe').write_bytes(source.encode('utf-8'))
assert (run/'fixture.vibe').read_bytes() == source.encode('utf-8')
bindings=[dict(key=name,kind=kind,path=str(run/name)) for name,kind in [('fixture.glb','glb'),('checker.png','png')]]
cli=[args.synth,'produce','--source',run/'fixture.vibe','--out',run/'cli.hmc','--export-dir',run/'cli']
for binding in bindings:cli+=['--input',binding['key'],binding['kind'],binding['path']]
cli_result=execute(cli,'cli');check('CLI retained fixture',cli_result.returncode==0,stderr=cli_result.stderr[-2000:])
base=dict(source=source,inputs=bindings)
calls=[('contracts','synth_operation_contracts',{}),
       ('production','synth_produce_present',dict(base,out_hmc='mcp.hmc',export_dir='mcp',receipt_json='receipt.json',preview_assets=['fixture.10d'])),
       ('unknown','synth_produce_present',dict(base,typo=True)),
       ('outside','synth_produce_present',dict(base,inputs=[dict(key='outside',kind='json',path=str(ROOT/'Cargo.toml'))])),
       ('wrong_host','vibe_invoke',dict(id='hypermedia.locus.construct',args=dict(shape='circle',radius=1,segments=16)))]
messages=[dict(jsonrpc='2.0',id='init',method='initialize',params=dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='local-parity',version='1'))),dict(jsonrpc='2.0',method='notifications/initialized')]
messages += [dict(jsonrpc='2.0',id=ident,method='tools/call',params=dict(name=name,arguments=arguments)) for ident,name,arguments in calls]
response=execute([args.synth,'mcp'],'mcp',input=''.join(json.dumps(m)+'\n' for m in messages))
responses={x.get('id'):x for line in response.stdout.splitlines() if line.startswith('{') for x in [json.loads(line)]}
(run/'mcp-responses.json').write_text(json.dumps(responses,indent=2),encoding='utf-8')
for ident,_,_ in calls:
    result=responses.get(ident,{})
    failed=bool(result.get('error') or result.get('result',{}).get('isError'))
    check('MCP '+ident, bool(result) and failed==(ident in ['unknown','outside','wrong_host']))
check('CLI/MCP identical HMC', (run/'cli.hmc').exists() and (run/'mcp.hmc').exists() and sha(run/'cli.hmc')==sha(run/'mcp.hmc'))
manifest=run/'inputs.json';manifest.write_text(json.dumps(bindings),encoding='utf-8')
helper=execute(['pwsh','-NoProfile','-File',ROOT/'vendor/grok/scripts/invoke.ps1','-Source',run/'fixture.vibe','-InputManifest',manifest,'-RunsDir',run,'-Name','grok-parity'],'grok')
check('Grok shared helper',helper.returncode==0,stderr=helper.stderr[-1500:])
packages=list(run.glob('*grok-parity*.hmc'))
check('Grok/CLI identical HMC', len(packages)==1 and (run/'cli.hmc').exists() and sha(packages[0])==sha(run/'cli.hmc'))
one_shot=execute(['pwsh','-NoProfile','-File',ROOT/'vendor/grok/scripts/invoke.ps1','-Morphism','hypermedia.radiance.create_rgba','-ArgsJson','{"width":8,"height":8,"color":[12,34,56,255]}','-Encode','rgba','-Key','pixel.png','-RunsDir',run,'-Name','json-parity'],'grok-json')
check('Grok JSON typed helper',one_shot.returncode==0,stderr=one_shot.stderr[-1500:])
help_result=execute([args.synth,'produce','--help'],'profile-help')
check('Shared quality profile help',help_result.returncode==0 and '--quality-profile preview|production16' in help_result.stdout)
print(run,flush=True)
raise SystemExit(0 if all(x['passed'] for x in results) else 1)
