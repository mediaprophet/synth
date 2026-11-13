"""Post-integration production, replay and native-shot evidence runner.

This records outputs for subsequent human/agent image inspection. It does not
classify visual quality from successful commands or hashes.
"""
from pathlib import Path
import argparse, datetime, hashlib, json, shutil, subprocess, uuid

parser=argparse.ArgumentParser()
parser.add_argument('--synth',type=Path,default=Path(__file__).resolve().parents[1]/'target-gap-review/debug/synth.exe')
parser.add_argument('--output-root',type=Path)
parser.add_argument('--domain',action='append',choices=['biology','biology-cell','anatomy','flora','flora-branch','fauna','terrain','cosmos','interface'])
args=parser.parse_args()
repo=Path(__file__).resolve().parents[1]
parent=args.output_root or repo/'production'
parent.mkdir(parents=True,exist_ok=True)
run=parent/('quality-'+datetime.datetime.now().strftime('%Y%m%d-%H%M%S')+'-'+uuid.uuid4().hex[:8])
run.mkdir()
records=[]

def command(argv, log, timeout=600):
    try:
        p=subprocess.run([str(args.synth),*map(str,argv)],cwd=repo,capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=timeout)
        log.write_text(p.stdout+'\n'+p.stderr,encoding='utf-8')
        return dict(command=[str(args.synth),*map(str,argv)],returncode=p.returncode,log=str(log))
    except subprocess.TimeoutExpired:
        log.write_text(f'Exceeded {timeout} seconds',encoding='utf-8')
        return dict(command=list(map(str,argv)),timeout_seconds=timeout,log=str(log))

def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def save():
    (run/'results.json').write_text(json.dumps({'visual_acceptance':'pending image inspection','results':records},indent=2),encoding='utf-8')

domains=args.domain or ['biology','biology-cell','anatomy','flora','flora-branch','fauna','terrain','cosmos','interface']
for domain in domains:
    source=repo/'examples/quality'/f'{domain}.vibe'
    base=['produce','--source',source]
    item={'domain':domain,'source':str(source),'source_sha256':sha(source),'steps':[]}
    if domain=='interface':
        font=Path('C:/Windows/Fonts/segoeui.ttf')
        staged=run/'ui-font.ttf'
        shutil.copyfile(font,staged)
        base += ['--input','ui-font.ttf','ttf',staged]
        item['font_sha256']=sha(staged)
    hmc=run/f'{domain}.hmc'; export=run/domain
    step=command([*base,'--out',hmc,'--export-dir',export],run/f'{domain}-produce.log')
    item['steps'].append(step)
    if step.get('returncode')==0:
        repeat=run/f'{domain}-replay.hmc'
        replay=command([*base,'--out',repeat],run/f'{domain}-replay.log')
        item['steps'].append(replay)
        item['package_sha256']=sha(hmc)
        item['deterministic_replay']=repeat.exists() and sha(repeat)==sha(hmc)
        meshes=sorted(export.rglob('*.10d'))
        if domain=='fauna': meshes=[p for p in meshes if 'skeleton' not in p.stem and 'ecorche' not in p.stem][:1]
        for index, mesh in enumerate(meshes):
            native=run/f'{domain}-native-{index}'
            key='outputs/'+mesh.relative_to(export).as_posix()
            native_result=command(['preview','--asset',mesh,'--out',native,'--hmc',hmc,'--asset-key',key],run/f'{domain}-native-{index}.log')
            item['steps'].append(native_result)
            if native_result.get('returncode')!=0: continue
            receipt=json.loads((native/'review-receipt.json').read_text())
            projection=receipt['projection']
            shot={'side':1024,'camera_yaw':0.72,'camera_pitch':0.28,
                  'camera_distance':projection['camera_distance'],'camera_target':projection['camera_target'],
                  'sun_direction':[0.4,0.8,0.4],'direct_intensity':2.0,'ambient_intensity':0.28,
                  'clear_color':[0.012,0.018,0.03,1.0],'times':[0.0]}
            if domain=='fauna': shot.update(clip_id=1,times=[0.0,1/3,2/3])
            if domain=='flora':
                shot.update(side=2048,camera_yaw=0.48,camera_pitch=0.20,
                            camera_distance=projection['camera_distance']*0.82,ambient_intensity=0.40)
            if domain=='flora-branch' and 'flower' in mesh.stem:
                shot.update(side=2048,camera_yaw=0.62,camera_pitch=0.24,
                            camera_distance=projection['camera_distance']*0.82,ambient_intensity=0.36)
            if domain=='terrain':
                # The elevation mesh is Z-up. View across its XY plane so
                # terrain relief reads vertically without rotating durable data.
                shot.update(side=2048,camera_yaw=0.45,camera_pitch=-1.05,
                            sun_direction=[0.4,0.3,0.85],ambient_intensity=0.38)
                item['coordinate_frame']={'mesh':'ENU Z-up','horizontal_units':'m','height_units':'m','origin_m':[-31.5,-31.5,0.0],'preview_only_rotation':False}
            shot_file=run/f'{domain}-shot-{index}.json'
            shot_file.write_text(json.dumps(shot,indent=2))
            item['steps'].append(command(['preview','--asset',mesh,'--out',run/f'{domain}-shot-{index}',
                                          '--shot',shot_file,'--hmc',hmc,'--asset-key',key],run/f'{domain}-shot-{index}.log'))
            if domain=='flora':
                # A separate inspected detail view; retain the complete leaf and
                # all three standard native views above as context/geometry evidence.
                detail={**shot,'camera_yaw':1.05,'camera_pitch':0.30,
                        'camera_target':[0.034,-0.006,0.0015],'camera_distance':0.075}
                detail_file=run/f'{domain}-tissue-shot-{index}.json'
                detail_file.write_text(json.dumps(detail,indent=2))
                item['steps'].append(command(['preview','--asset',mesh,'--out',run/f'{domain}-tissue-shot-{index}',
                                              '--shot',detail_file,'--hmc',hmc,'--asset-key',key],run/f'{domain}-tissue-shot-{index}.log'))
        item['output_sha256']={p.relative_to(export).as_posix():sha(p) for p in export.rglob('*') if p.is_file()}
    records.append(item); save()
    print(domain,step.get('returncode','timeout'),flush=True)
print(run,flush=True)
failed=any(any(step.get('returncode')!=0 for step in item['steps']) or item.get('deterministic_replay') is not True for item in records)
raise SystemExit(1 if failed else 0)
