"""Deterministic external-heart study, Y up, +X patient left, +Z anterior.
Geometric anatomical illustration, not a clinical or internal-chamber model.
"""
import json, math
from pathlib import Path
ROOT=Path(__file__).resolve().parent

def val(x): return json.dumps(x,separators=(',',':'),allow_nan=False)
def data(x):
    if isinstance(x,dict): return '{'+','.join(k+':'+data(v) for k,v in x.items())+'}'
    if isinstance(x,list): return '['+','.join(data(v) for v in x)+']'
    return val(x)
def invoke(n,op,args): return 'let '+n+' = capability.invoke("hypermedia.'+op+'", '+data(args)+');\n'
def ring_y(y,cx,cz,rx,rz,n=32):
    return [[round(cx+rx*math.cos(t*2*math.pi/n),5),y,round(cz+rz*math.sin(t*2*math.pi/n),5)] for t in range(n)]
def loft(n,rows):
    return invoke(n,'manifold.loft',{'sections':[ring_y(*r) for r in rows],'interpolation':'cubic','samples':5,'caps':True})
def ventricular_loft(rows):
    # One continuous surface avoids the open cuts returned by the boolean
    # kernel on these overlapping curved inputs. Off-centre rings form the
    # oblique apex, and cross-section skew differentiates the two sides.
    rings=[]
    for y,cx,cz,rx,rz in rows:
        ring=[]
        for k in range(32):
            angle=2*math.pi*k/32
            c,s=math.cos(angle),math.sin(angle)
            ring.append([round(cx+rx*c*(1+0.10*s),5),y,round(cz+rz*s*(1+0.13*c),5)])
        rings.append(ring)
    return invoke('myocardium','manifold.loft',{'sections':rings,'interpolation':'cubic','samples':5,'caps':True})
def tube(n,path,radii,hollow=False,radial_segs=32):
    # Circumferential sections perpendicular to the sampled centreline. Dense
    # authored samples plus cubic loft preserve the arch without elbow facets.
    rings=[]
    for i,p in enumerate(path):
        a=path[max(0,i-1)]; b=path[min(len(path)-1,i+1)]
        d=[b[j]-a[j] for j in range(3)]; l=math.sqrt(sum(v*v for v in d)); d=[v/l for v in d]
        ref=[0,0,1] if abs(d[2])<0.9 else [0,1,0]
        u=[d[1]*ref[2]-d[2]*ref[1],d[2]*ref[0]-d[0]*ref[2],d[0]*ref[1]-d[1]*ref[0]]
        l=math.sqrt(sum(v*v for v in u)); u=[v/l for v in u]
        # Loft triangles are [ring_i, next_station_i, next_station_i+1].
        # Use v = u × d so their side normals point outwards.
        v=[u[1]*d[2]-u[2]*d[1],u[2]*d[0]-u[0]*d[2],u[0]*d[1]-u[1]*d[0]]
        rings.append([[round(p[j]+radii[i]*(u[j]*math.cos(2*math.pi*k/radial_segs)+v[j]*math.sin(2*math.pi*k/radial_segs)),3) for j in range(3)] for k in range(radial_segs)])
    if not hollow:
        return invoke(n,'manifold.loft',{'sections':rings,'interpolation':'cubic','samples':4,'caps':True})
    # Thicken the open tapered surface inwards; solidify creates the connected
    # inner wall and boundary rims without spelling a second set of rings.
    s=invoke(n,'manifold.loft',{'sections':rings,'interpolation':'cubic','samples':4,'caps':False})
    s+='let '+n+'_shell = capability.invoke("hypermedia.manifold.solidify", {mesh:'+n+',thickness:0.06,offset:-1});\n'
    return s
def p(key,mesh,color): return {'key':key,'mesh':None,'color':color,'roughness':0.46,'metallic':0.0}

def generate(radial_segs=32):
    if not isinstance(radial_segs, int) or not 32 <= radial_segs <= 128:
        raise ValueError("radial_segs must be an integer from 32 through 128")
    def vessel(n,path,radii,hollow=False):
        return tube(n,path,radii,hollow,radial_segs)
    s='requires [capability("capability.invoke")];\n// Reference study: centimetres; illustrative exterior, no chamber interiors.\n'
    # Asymmetric myocardium. A narrow displaced lowest section makes a true
    # oblique LV apex; the broader RV mass occupies the anterior/right side.
    body_rows=[
        (-5.10,1.32,0.05,.12,.13),(-4.80,1.16,.04,.65,.59),(-4.25,.82,.08,1.28,1.06),(-3.3,.38,.10,1.98,1.56),(-2.1,.02,.12,2.42,1.88),(-.7,-.20,.09,2.73,2.03),(.8,-.25,.02,2.75,1.96),(2.0,-.22,-.12,2.35,1.60),(2.75,-.10,-.26,1.50,1.07),(3.0,.02,-.27,.35,.40)]
    # Shorten the inferior mass while keeping the vessel roots fixed. The
    # displaced apex and the coronary path use these same stations.
    body_rows=[(y*.80 if y < 0 else y,cx,cz,rx,rz) for y,cx,cz,rx,rz in body_rows]
    s+=ventricular_loft(body_rows)
    # Both atrial bodies and projecting auricles are closed, thick lobes rather
    # than planar wings. They sit around the vessel roots.
    s+=loft('right_atrium',[(1.1,-1.85,-.15,.28,.28),(1.45,-2.0,-.15,.85,.70),(2.15,-2.08,-.16,1.02,.82),(2.85,-2.04,-.20,.84,.72),(3.45,-1.94,-.25,.40,.38),(3.58,-1.90,-.26,.08,.08)])
    s+=loft('left_atrium',[(1.35,1.50,-.90,.18,.20),(1.75,1.55,-.88,.72,.63),(2.4,1.48,-.88,.98,.82),(3.0,1.35,-.90,.80,.68),(3.48,1.25,-.93,.30,.29),(3.57,1.22,-.94,.06,.06)])
    s+=loft('right_auricle',[(1.40,-2.00,1.00,.08,.08),(1.62,-2.06,1.12,.32,.22),(1.95,-2.00,1.18,.53,.32),(2.28,-1.88,1.12,.58,.34),(2.62,-1.65,.94,.45,.27),(2.90,-1.42,.74,.15,.12)])
    s+=loft('left_auricle',[(1.53,1.78,.85,.07,.07),(1.72,1.93,.93,.28,.20),(1.98,2.02,.94,.43,.29),(2.26,1.98,.85,.47,.30),(2.53,1.80,.70,.30,.21),(2.70,1.62,.58,.08,.07)])
    # Main aorta: rising root curves into arch and descending segment.
    s+=vessel('aorta',[[.20,2.05,-.10],[.18,2.7,-.12],[.18,3.4,-.15],[.25,4.1,-.22],[.42,4.75,-.40],[.72,5.20,-.64],[1.12,5.40,-.90],[1.53,5.28,-1.12],[1.78,4.95,-1.30],[1.90,4.48,-1.43],[1.90,3.95,-1.52],[1.88,3.40,-1.57],[1.84,2.75,-1.58],[1.80,2.10,-1.54]],[.76,.70,.67,.65,.65,.65,.64,.62,.60,.58,.56,.54,.52,.50],True)
    # One continuous centreline removes the visible overlap/cap at the former
    # pulmonary trunk/branch junction. This remains a simplified single branch.
    s+=vessel('pulmonary',[[ -.60,1.90,1.05],[-.55,2.7,1.12],[-.45,3.3,1.10],[-.25,3.85,.95],[.05,4.23,.72],[.42,4.40,.47],[.85,4.48,.30],[1.40,4.45,.10],[1.95,4.30,-.12],[2.32,4.08,-.25]],[.58,.56,.55,.53,.50,.46,.40,.37,.34,.30],True)
    s+=vessel('svc',[[-1.80,2.55,-.20],[-1.86,3.15,-.23],[-1.90,3.8,-.25],[-1.92,4.45,-.27],[-1.93,5.05,-.29]],[.52,.50,.48,.46,.44],True)
    # Schematic anterior interventricular artery, centred just inside the
    # authored front-most ring vertices. It follows the same loft stations,
    # so it cannot float off the interpolated surface as the prior trial did.
    artery=[[cx,y,cz+rz-.015] for y,cx,cz,rx,rz in reversed(body_rows[2:8])]
    s+=vessel('anterior_coronary',artery,[.055,.052,.048,.044,.040,.035])
    for n,path in [('arch_1',[[.58,5.15,-.55],[.48,5.65,-.49],[.35,6.12,-.42],[.27,6.6,-.35]]),('arch_2',[[1.05,5.38,-.85],[1.05,5.85,-.82],[1.03,6.28,-.78],[1.01,6.7,-.73]]),('arch_3',[[1.50,5.29,-1.10],[1.68,5.72,-1.12],[1.88,6.1,-1.14],[2.05,6.5,-1.14]])]:
        s+=vessel(n,path,[.22,.21,.20,.19],True)
    parts=[]
    for n,c in [('myocardium',[.55,.12,.14,1]),('right_atrium',[.43,.105,.14,1]),('left_atrium',[.43,.105,.14,1]),('right_auricle',[.48,.13,.17,1]),('left_auricle',[.48,.13,.17,1]),('aorta',[.62,.15,.16,1]),('arch_1',[.62,.15,.16,1]),('arch_2',[.62,.15,.16,1]),('arch_3',[.62,.15,.16,1]),('pulmonary',[.33,.23,.43,1]),('svc',[.30,.22,.41,1]),('anterior_coronary',[.36,.055,.05,1])]:
        # Assembly/10D uses metres; authored coordinates above are centimetres.
        hollow = n in ('aorta','arch_1','arch_2','arch_3','pulmonary','svc')
        parts.append('{key:'+val(n)+',mesh:'+n+('_shell' if hollow else '')+',scale:[0.01,0.01,0.01],color:'+val(c)+',shading:"smooth",roughness:0.46,metallic:0.0}')
    s+='let heart = capability.invoke("hypermedia.manifold.sweep_assembly_profile", {title:"External heart anatomy study",parts:['+','.join(parts)+']});\n'
    s+='let asset={files:[{key:"anatomy.10d",resource:heart.container_10d},{key:"anatomy-blueprint.svg",resource:heart.blueprint_svg}]};\n'
    return s
if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser()
    parser.add_argument('--radial-segs',type=int,default=32)
    parser.add_argument('--output',type=Path,default=ROOT/'anatomy.vibe')
    args=parser.parse_args()
    args.output.write_text(generate(args.radial_segs),encoding='utf-8')
