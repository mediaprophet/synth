"""Controlled native shadow regression; inspect the resulting PNGs after metrics."""
from pathlib import Path
import argparse, datetime, hashlib, json, subprocess, uuid
import numpy as np
from PIL import Image

CASTER_SOURCE = """requires [capability("capability.invoke")];
let floor_profile = capability.invoke("hypermedia.locus.construct", {shape:"rectangle",width:4.2,height:4.2});
let floor = capability.invoke("hypermedia.manifold.extrude", {profile:floor_profile,path:[[0,0,0.05],[0,0,0.1]],caps:true});
let box_profile = capability.invoke("hypermedia.locus.construct", {shape:"rectangle",width:0.8,height:0.8});
let box = capability.invoke("hypermedia.manifold.extrude", {profile:box_profile,path:[[0,0,0.105],[0,0,1.25]],caps:true});
let mesh = capability.invoke("hypermedia.manifold.boolean_3d", {a:floor,b:box,op:"union"});
let sealed = capability.invoke("hypermedia.manifold.compile_10d_pbr", {mesh:mesh});
let mut asset = {files:[{key:"shadow-caster.10d",resource:sealed}]};
"""

def sha(path):
    with path.open('rb') as file: return hashlib.file_digest(file,'sha256').hexdigest()

def rgb(path): return np.asarray(Image.open(path).convert('RGB')).astype(np.int16)

def receiver_metrics(on_path, off_path, name):
    on, off = rgb(on_path), rgb(off_path)
    if on.shape != off.shape: raise RuntimeError('Matched shot dimensions differ')
    mask = np.any(on != on[0,0],axis=2)
    if name == 'cosmos':
        h,w=mask.shape; roi=np.zeros_like(mask)
        roi[int(h*.39):int(h*.44),int(w*.47):int(w*.55)]=True
        mask &= roi
    values=np.max(off-on,axis=2)[mask]
    if values.size<100: raise RuntimeError('Receiver comparison ROI is empty')
    p95=float(np.percentile(values,95)); fraction=float(np.mean(values>4))
    return dict(roi_pixels=int(values.size),darkening_p95=p95,fraction_over_four_levels=fraction,
        passed=p95<=3 and fraction<=0.01,on=str(on_path),off=str(off_path))

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--synth',type=Path)
    parser.add_argument('--scene-root',type=Path,required=True)
    parser.add_argument('--out',type=Path)
    parser.add_argument('--assess-existing',type=Path,help='Assess prior controlled diagnostic without launching a renderer')
    args=parser.parse_args();root=Path(__file__).resolve().parents[1]
    run=args.out or root/'production'/('shadow-quality-'+datetime.datetime.now().strftime('%Y%m%d-%H%M%S')+'-'+uuid.uuid4().hex[:8])
    run.mkdir(parents=False,exist_ok=False)
    report=dict(schema='synth.shadow-receiver-regression/v1',visual_review_required=True,scene_root=str(args.scene_root),commands=[],receivers={})
    def command(argv,name):
        result=subprocess.run([str(args.synth),*map(str,argv)],cwd=root,capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=180)
        (run/(name+'.log')).write_text(result.stdout+'\n'+result.stderr,encoding='utf-8')
        report['commands'].append(dict(name=name,argv=[str(args.synth),*map(str,argv)],exit_code=result.returncode))
        if result.returncode: raise RuntimeError(name+' failed; inspect log')
    def shot(asset,hmc,key,settings,name,shadows):
        authored={**settings,'shadows':shadows};path=run/(name+'.json')
        path.write_text(json.dumps(authored,indent=2),encoding='utf-8')
        command(['preview','--asset',asset,'--out',run/name,'--shot',path,'--hmc',hmc,'--asset-key',key],name)
        return run/name/'frame-0000.png'
    try:
        if not args.assess_existing:
            if args.synth is None or not args.synth.is_file(): raise RuntimeError('A frozen --synth binary is required')
            report['runtime']=dict(path=str(args.synth.resolve()),sha256=sha(args.synth))
        for name in ['cosmos','terrain']:
            if args.assess_existing:
                on=args.scene_root/(name+'-shot-0/frame-0000.png')
                off=args.assess_existing/(name+'-no-shadows/frame-0000.png')
            else:
                settings=json.loads((args.scene_root/(name+'-shot-0.json')).read_text(encoding='utf-8'))
                settings['ao_enabled']=True
                asset=args.scene_root/name/(name+'.10d');hmc=args.scene_root/(name+'.hmc')
                on=shot(asset,hmc,'outputs/'+name+'.10d',settings,name+'-on',True)
                off=shot(asset,hmc,'outputs/'+name+'.10d',settings,name+'-off',False)
            report['receivers'][name]=receiver_metrics(on,off,name)
        if not args.assess_existing:
            source=run/'shadow-caster.vibe';source.write_bytes(CASTER_SOURCE.encode('utf-8'))
            command(['produce','--profile','files','--source',source,'--out',run/'caster.hmc','--export-dir',run/'caster-assets'],'caster-produce')
            settings=dict(side=512,camera_yaw=-0.7,camera_pitch=0.55,camera_distance=9.0,camera_target=[0,0,0.5],
                sun_direction=[0.8,0.4,1.0],direct_intensity=2.0,ambient_intensity=0.1,clear_color=[0.012,0.018,0.03,1],times=[0.0],ao_enabled=False)
            on=shot(run/'caster-assets/shadow-caster.10d',run/'caster.hmc','outputs/shadow-caster.10d',settings,'caster-on',True)
            off=shot(run/'caster-assets/shadow-caster.10d',run/'caster.hmc','outputs/shadow-caster.10d',settings,'caster-off',False)
            delta=np.max(rgb(off)-rgb(on),axis=2);pixels=int(np.count_nonzero(delta>=12))
            report['caster']=dict(shadow_contrast_pixels=pixels,max_darkening=int(delta.max()),passed=pixels>=64,on=str(on),off=str(off))
        report['passed']=all(item['passed'] for item in report['receivers'].values()) and (args.assess_existing is not None or report['caster']['passed'])
        report['scope']='previous diagnostic assessment' if args.assess_existing else 'native receivers and real caster comparison'
    except Exception as error:
        report['passed']=False;report['error']=dict(type=type(error).__name__,message=str(error))
    (run/'verification.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(json.dumps(dict(passed=report['passed'],receipt=str(run/'verification.json'),receivers=report['receivers'],caster=report.get('caster'),error=report.get('error')),indent=2))
    return 0 if report['passed'] else 1

if __name__=='__main__': raise SystemExit(main())
