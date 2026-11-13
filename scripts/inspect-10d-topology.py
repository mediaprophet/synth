"""Inspect topology of a native 10D quantized mesh without changing it.

This is a geometry diagnostic, not container CRC/BLAKE3 authentication.
Run the native verifier separately. Counts include exact decoded-index edges
and a second comparison of equal quantized coordinate triples.
"""
import argparse
import collections
import json
import math
import struct
from pathlib import Path

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('asset',type=Path)
p.add_argument('--max-bytes',type=int,default=64*1024*1024)
args=p.parse_args()
if args.max_bytes <= 0 or args.asset.stat().st_size > args.max_bytes:
    raise SystemExit('asset exceeds explicit byte budget')
b=args.asset.read_bytes()
if len(b)<64 or b[:4]!=b'10d\0':
    raise SystemExit('invalid or unsupported 10D header')
table,count=struct.unpack_from('<II',b,56)
if count>1024 or table<64 or table+count*24>len(b):
    raise SystemExit('invalid 10D section table')
meshes=[]
for i in range(count):
    typ,_,_,offset,length,_,_,_=struct.unpack_from('<BBHIIIII',b,table+i*24)
    if typ==1: meshes.append((offset,length))
if len(meshes)!=1:
    raise SystemExit('expected exactly one quantized mesh section')
offset,length=meshes[0]
if offset+length>len(b) or length<40:
    raise SystemExit('invalid mesh section range')
flags,reserved,nv,nt=struct.unpack_from('<HHII',b,offset)
if flags & ~1 or reserved or nv>4194304 or nt>4194304:
    raise SystemExit('invalid mesh counts or flags')
lo=struct.unpack_from('<3f',b,offset+12); hi=struct.unpack_from('<3f',b,offset+24)
if not all(math.isfinite(x) for x in (*lo,*hi)) or any(lo[i]>hi[i] for i in range(3)):
    raise SystemExit('invalid mesh bounds')
step=6 if flags&1 else 12
needed=40+nv*6+nt*step
if needed!=length:
    raise SystemExit('invalid mesh section length')
q=[struct.unpack_from('<3H',b,offset+40+i*6) for i in range(nv)]
pos=[tuple(lo[j]+v[j]/65535*(hi[j]-lo[j]) for j in range(3)) for v in q]
start=offset+40+nv*6; fmt='<3H' if flags&1 else '<3I'
raw=collections.defaultdict(lambda:[0,0]); geometric=collections.defaultdict(lambda:[0,0])
used=set();degenerate=0; volume=0.0
for i in range(nt):
    tri=struct.unpack_from(fmt,b,start+i*step)
    if any(index>=nv for index in tri): raise SystemExit('triangle index out of range')
    used.update(tri)
    if len(set(tri))!=3 or len({q[index] for index in tri})!=3: degenerate+=1
    a,c,d=(pos[index] for index in tri)
    u=[c[j]-a[j] for j in range(3)];v=[d[j]-a[j] for j in range(3)]
    n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
    if sum(x*x for x in n)==0 and len({q[index] for index in tri})==3: degenerate+=1
    volume+=sum(a[j]*n[j] for j in range(3))/6
    for j in range(3):
        x,y=tri[j],tri[(j+1)%3]
        if x==y: continue
        r=raw[tuple(sorted((x,y)))];r[0]+=1;r[1]+=1 if x<y else -1
        x,y=q[x],q[y]
        if x==y: continue
        r=geometric[tuple(sorted((x,y)))];r[0]+=1;r[1]+=1 if x<y else -1

def summary(edges):
    return {'edges':len(edges),'boundary_edges':sum(c==1 for c,_ in edges.values()),
            'nonmanifold_edges':sum(c>2 for c,_ in edges.values()),
            'same_direction_pairs':sum(c==2 and direction!=0 for c,direction in edges.values())}
r=summary(raw);w=summary(geometric)
result={'asset':str(args.asset.resolve()),'integrity':'not verified by this geometry diagnostic',
        'vertices':nv,'triangles':nt,'unused_vertices':nv-len(used),
        'unique_quantized_positions':len(set(q)),'degenerate_quantized_faces':degenerate,
        'index_edges':r,'equal_quantized_position_edges':w,'signed_volume_in_asset_units_cubed':volume,
        'closed_oriented_by_index':nt>0 and degenerate==0 and all(r[k]==0 for k in ('boundary_edges','nonmanifold_edges','same_direction_pairs'))}
print(json.dumps(result,indent=2))
raise SystemExit(0 if result['closed_oriented_by_index'] else 1)
