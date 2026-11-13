"""Author deterministic VibeScript reference scenes using existing morphisms."""
from pathlib import Path
import argparse, math, json, re

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--domain", action="append", choices=["biology", "cosmos", "interface", "anatomy", "fauna", "flora", "terrain"])
selected = set(parser.parse_args().domain or [])

ROOT = Path(__file__).resolve().parent
HEADER = 'requires [capability("capability.invoke")];\n'
def data(value):
    if isinstance(value,dict):
        if any(not re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', k) for k in value):
            raise ValueError('Vibe record keys must be identifiers')
        return '{'+','.join(k+':'+data(v) for k,v in value.items())+'}'
    if isinstance(value,list): return '['+','.join(data(v) for v in value)+']'
    return json.dumps(value, separators=(',', ':'), allow_nan=False)
def call(name, operation, arguments):
    return f'let {name} = capability.invoke("hypermedia.{operation}", {arguments});\n'
def emit(name, source):
    if selected and name not in selected: return
    (ROOT / f'{name}.vibe').write_text(HEADER + source, encoding='utf-8')
def assembly(parts, title):
    return call('scene','manifold.sweep_assembly_profile', '{title:'+data(title)+',parts:['+','.join(parts)+']}')
def export_scene(name):
    return 'let asset={files:[{key:"'+name+'.10d",resource:scene.container_10d},{key:"'+name+'-blueprint.svg",resource:scene.blueprint_svg}]};\n'
def part(key, mesh, color, extra=''):
    return '{key:'+data(key)+',mesh:'+mesh+',color:'+data(color)+',roughness:0.38,metallic:0.0,shading:"smooth"'+extra+'}'

# Biology: coarse-grained double helix, explicitly a structural illustration.
s = call('profile', 'locus.construct', '{shape:"circle",radius:0.075,segments:48}')
parts = []
for strand in range(2):
    points = [[1.15*math.cos(i*math.pi/60+strand*math.pi), (i-120)*0.0275,
               1.15*math.sin(i*math.pi/60+strand*math.pi)] for i in range(241)]
    s += call(f'backbone{strand}', 'manifold.extrude', '{profile:profile,path:'+data(points)+',caps:true}')
    parts.append(part(f'backbone_{strand}', f'backbone{strand}', [0.08,0.64,0.7,1] if strand==0 else [0.72,0.17,0.31,1]))
s += call('bond_profile','locus.construct','{shape:"circle",radius:0.045,segments:24}')
for i in range(31):
    angle=i*math.pi/7.5; y=(i-15)*0.22
    a=[1.15*math.cos(angle),y,1.15*math.sin(angle)]
    b=[-a[0],y,-a[2]]; mid=[0,y,0]
    for j, endpoints in enumerate(([a,mid],[mid,b])):
        var=f'bond_{i}_{j}'
        s += call(var,'manifold.extrude','{profile:bond_profile,path:'+data(endpoints)+',caps:true}')
        colors=([0.92,0.66,0.2,1],[0.45,0.38,0.82,1]) if i%2 else ([0.29,0.72,0.43,1],[0.85,0.35,0.22,1])
        parts.append(part(var,var,colors[j]))
s += assembly(parts,'Double helix / coarse-grained structural illustration') + export_scene('biology')
emit('biology',s)

# Cosmos: a deliberately fictitious ringed world; units are local scene units.
profile=[[max(0.00001,math.sin(t*math.pi/128)),math.cos(t*math.pi/128)] for t in range(129)]
s=call('globe','manifold.revolve','{points:'+data(profile)+',segments:192}')
parts=[part('planet','globe',[0.68,0.48,0.27,1])]
for i in range(12):
    r0=1.35+i*0.075; r1=r0+0.064
    profile=[[r0,-0.003],[r1,-0.003],[r1,0.003],[r0,0.003],[r0,-0.003]]
    name=f'ring{i}'
    s+=call(name,'manifold.revolve','{points:'+data(profile)+',segments:192}')
    value=0.46+0.24*(0.5+0.5*math.sin(i*1.9))
    parts.append(part(name,name,[value,value*0.85,value*0.62,1],',rotation:[0.3,0.0,0.18]'))
s += assembly(parts,'Ringed world / local artistic coordinate frame') + export_scene('cosmos')
emit('cosmos',s)

# A reference UI uses retained OpenType outlines; no machine font discovery.
boxes=[]
def box(key,role,label,x,y,w,h,state='default'):
    boxes.append(dict(id=key,role=role,label=label,x=x,y=y,width=w,height=h,state=state))
box('title','header','SYNTH  /  LOCAL STUDIO',24,20,976,58)
box('nav','sidebar','PRODUCTION\n\nAsset library\nGeometry\nMaterials\nAnimation\n\nREVIEW\n\nReference views\nAnnotations\nDeliverables',24,94,210,520)
box('hero','panel','WORKSPACE  /  ANATOMY\n\nAuthored assets. Preserved materials.\nInspected output.',250,94,750,100)
box('geometry','panel','GEOMETRY\n\nLofted anatomy\nSeams and UVs retained',250,210,238,162)
box('surface','panel','SURFACE\n\nPBR materials\nTextures bundled by digest',504,210,238,162)
box('motion','panel','MOTION\n\nBound skeletal data\nPose review',758,210,242,162)
box('review','panel','REVIEW CHECKLIST\n\nFront / side / three-quarter\nReference landmarks and junctions\nTexture seams and surface response\nDeformation through the sequence',250,388,480,226)
box('action','button','PRODUCE PACKAGE',750,388,250,58,'focus')
box('status','panel','LOCAL OUTPUT\n\nCreate-only export\nInput and output hashes\nDurable review receipt',750,462,250,152)
layout={'viewport_width':1024,'viewport_height':640,'boxes':boxes}
s='let layout='+data(layout)+';\n'
s+=call('theme','layout.bind_theme_scope','{scope:"app",mode:"holosonic-dark",accent:[13,191,184,255]}')
s+=call('ui','layout.render_ui_sheet','{layout:layout,theme:theme,font:"ui-font.ttf"}')
s+=call('png','radiance.encode_rgba_png','{image:ui}')
s+=call('svg','layout.project_ui_svg','{layout:layout,theme:theme,font:"ui-font.ttf"}')
s+='let asset={files:[{key:"interface.png",resource:png},{key:"interface.svg",resource:svg}]};\n'
emit('interface',s)

# Keep the reviewed heart revision under its own deterministic authoring source.
from generate_anatomy import generate as generate_heart
if not selected or 'anatomy' in selected:
    (ROOT/'anatomy.vibe').write_text(generate_heart(),encoding='utf-8')

# Copy the common authored fauna source, including its 21-joint bind landmarks;
# retain only the established quality-resolution transform below.
fauna=(ROOT.parent/'swamp_wallaby.vibe').read_text(encoding='utf-8')
fauna=re.sub(r'radial_segs: (\d+)',lambda m:'radial_segs: '+str(min(64,int(m[1])*2)),fauna)
if not selected or 'fauna' in selected:
    (ROOT/'fauna.vibe').write_text(fauna,encoding='utf-8')

s=call('leaf','manifold.compile_botanical_organ_10d','{scale:4,species_id:"corymbia-aparrerinja",peel_01:0.9}')
s+='let asset={files:[{key:"flora.10d",resource:leaf.container_10d},{key:"flora-blueprint.svg",resource:leaf.blueprint_svg},{key:"flora-turnaround.png",resource:leaf.turntable_png}]};\n'
emit('flora',s)

# Physical terrain samples stay separate from coloured relief pixels.
side=64
heights=[]
for row in range(side):
    for col in range(side):
        x=(col-31.5)/31.5; z=(row-31.5)/31.5
        heights.append(round(12+20*math.exp(-((x+0.28)**2+(z-0.12)**2)*6)
                             +14*math.exp(-((x-0.42)**2+(z+0.3)**2)*11)
                             +1.2*math.sin(x*13)*math.cos(z*11),5))
field=dict(side=side,cell_spacing_m=1.0,origin_m=[-31.5,-31.5,0.0],
           height_units='m',no_data='none',samples=heights)
s='let elevation_grid='+data(field)+';\n'
s+=call('terrain','manifold.erode_geomorphic_heightfield','{heightfield:elevation_grid,crater_radius:0.0,terrace_spacing:0.7,thermal_strength:0.15}')
s+=call('physical','geodesy.compile_m64','{heightfield:terrain.heightfield,origin_lat_lon:[-33.86,151.2]}')
s+=call('scene','manifold.sweep_assembly_profile','{title:"Physical elevation / metre samples",parts:[{key:"terrain",mesh:terrain.mesh,color:[0.34,0.46,0.29,1.0],roughness:0.84,metallic:0.0,shading:"smooth"}]}')
s+='let asset={files:[{key:"terrain.10d",resource:scene.container_10d},{key:"terrain.m64",resource:physical},{key:"terrain-blueprint.svg",resource:scene.blueprint_svg}]};\n'
emit('terrain',s)
print('Authored quality sources: ' + ', '.join(sorted(selected)) if selected else 'Authored all seven quality reference sources.')
