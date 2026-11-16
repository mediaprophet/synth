"""Run existing geometry, Forge and Synth APIs with an explicit joined receipt.

All output lives in a caller-selected, create-only directory. Executables are
provided explicitly; their hashes are recorded and are not inferred from HEAD.
Inspect-only mode records admitted-path capability states without GPU or cargo.
"""
from pathlib import Path
import argparse, copy, datetime, json, os, struct, subprocess, sys, time
sys.path.insert(0,str(Path(__file__).resolve().parent/'vendor'))
from blake3 import blake3
from production_core_artifacts import (
    DETERMINISM_BOUNDARY, INCOMPLETE_CORE_CHECKLIST, LOWERHEX_HISTORY,
    attributed_glb, check_affine, f32, sha,
)
from admitted_geometry_probe import discover_worktrees, inspect_admitted_geometry, invoke_admitted_geometry

LIMITATIONS = [
    'Integration tracer, not full production-core acceptance.',
    'Semantic policy is library-level; public simplify remains geometric until later production integration.',
    'No whole-surface bound, collapse provenance map, operator or physical qualification.',
    'Supplied Synth executable provenance is its recorded binary hash; source equivalence is not assumed.',
    'Runner log capture is not P05 total-memory/cancellation qualification; commands have a 600-second timeout.',
    'Final production .10d sealing fidelity is not independently decoded here; visual review cannot certify UV/material retention.',
    DETERMINISM_BOUNDARY,
    'Geometry example previously failed E0277 LowerHex on format!("{:x}", Sha256::digest(bytes)); the repair is now built (qualiaDB/target-production-core/debug/examples/production_core_geometry.exe exists). Execution is not qualification; no GPU receipt is invented.',
    'Library present, callable, executed and qualified are distinct; source on disk is not qualification.',
    'GPU-identical timing is not claimed.',
]

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--geometry-exe',type=Path)
    parser.add_argument('--forge-exe',type=Path)
    parser.add_argument('--synth-exe',type=Path)
    parser.add_argument('--out',required=True,type=Path)
    parser.add_argument('--expected-adapter',default='NVIDIA')
    parser.add_argument('--skip-preview',action='store_true')
    parser.add_argument('--inspect-only',action='store_true',help='Record admitted-path states and commands without invoking executables or GPU.')
    parser.add_argument('--worktree-root',type=Path)
    args=parser.parse_args()
    out=args.out.resolve();out.mkdir(parents=False,exist_ok=False)
    executables={k:Path(v).resolve() for k,v in vars(args).items() if k.endswith('_exe') and v is not None}
    inspection=inspect_admitted_geometry(
        worktrees=discover_worktrees(args.worktree_root),
        geometry_exe=executables.get('geometry_exe'),
        forge_exe=executables.get('forge_exe'),
        synth_exe=executables.get('synth_exe'),
    )
    receipt=dict(
        schema='production-core.joined-tracer/v1',
        status='running',
        started_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
        mode='inspect' if args.inspect_only else 'joined',
        commands=[],
        executables={k:dict(path=str(v),sha256=sha(v) if v.is_file() else None,present=v.is_file()) for k,v in executables.items()},
        limitations=LIMITATIONS,
        capability_states=inspection['capabilities'],
        documented_commands=inspection['documented_commands'],
        sha256_lowerhex_history=LOWERHEX_HISTORY,
        determinism_boundary=DETERMINISM_BOUNDARY,
        joined_three_geometry_builds_are_not_three_gpu_runs=True,
        gpu_identical_timing_claimed=False,
        invented_passing_geometry_receipt=False,
        incomplete_core_checklist=INCOMPLETE_CORE_CHECKLIST,
        admitted_geometry_inspection=inspection,
        geometry_invocation='admitted_semantic_decimate_semantic_admitted_3',
    )
    def save():
        (out/'joined-receipt.json').write_text(json.dumps(receipt,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    save()
    def run(label,command,expected=0):
        started=time.perf_counter();env=os.environ.copy();env['ETALON_WGPU_BACKEND']='dx12'
        try:
            process=subprocess.run([str(x) for x in command],cwd=out,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=600)
        except subprocess.TimeoutExpired as error:
            for stream in ['stdout','stderr']:
                captured=getattr(error,stream) or ''
                if isinstance(captured,bytes):captured=captured.decode('utf-8',errors='replace')
                (out/(label+'.'+stream+'.txt')).write_text(captured,encoding='utf-8')
            receipt['commands'].append(dict(label=label,argv=[str(x) for x in command],cwd=str(out),environment_overrides={'ETALON_WGPU_BACKEND':'dx12'},exit_code=None,status='timeout',timeout_seconds=600,wall_seconds=time.perf_counter()-started,stdout=label+'.stdout.txt',stderr=label+'.stderr.txt'))
            save()
            raise
        (out/(label+'.stdout.txt')).write_text(process.stdout,encoding='utf-8');(out/(label+'.stderr.txt')).write_text(process.stderr,encoding='utf-8')
        receipt['commands'].append(dict(label=label,argv=[str(x) for x in command],cwd=str(out),environment_overrides={'ETALON_WGPU_BACKEND':'dx12'},exit_code=process.returncode,wall_seconds=time.perf_counter()-started,stdout=label+'.stdout.txt',stderr=label+'.stderr.txt'))
        save()
        if process.returncode!=expected:raise RuntimeError(f'{label} exited {process.returncode}; see retained stderr')
        return process
    try:
        if args.inspect_only:
            receipt['status']='inspected_not_executed'
            receipt['reason']='inspect-only; admitted geometry path was not invoked; no GPU run; no passing geometry receipt invented'
            save()
            print(json.dumps(dict(status=receipt['status'],out=str(out),commands=len(receipt['commands']),invented_passing_geometry_receipt=False),indent=2))
            return
        missing=[k for k in ['geometry_exe','forge_exe','synth_exe'] if k not in executables or not executables[k].is_file()]
        if missing:
            probe=invoke_admitted_geometry(executables.get('geometry_exe') or inspection['capabilities']['admitted_geometry_example']['executable'], out/'geometry-missing')
            receipt['commands'].append({k:probe[k] for k in probe if k not in ('stdout','stderr')})
            receipt['status']='failed'
            receipt['reason']='missing executable '+','.join(missing)+'; SHA256 LowerHex repair is built when the geometry exe path is supplied; no passing geometry receipt invented'
            save()
            raise RuntimeError(receipt['reason'])
        for i in range(1,4):run(f'geometry-{i}',[executables['geometry_exe'],'--out',out/f'geometry-{i}'])
        geometry=out/'geometry-1'
        geo=json.loads((geometry/'geometry-receipt.json').read_text())
        if geo['status']!='passed':raise ValueError('geometry not accepted even for tracer')
        receipt['capability_states']['admitted_geometry_example']['executed']=True
        receipt['capability_states']['admitted_geometry_example']['callable']=True
        receipt['capability_states']['admitted_geometry_example']['qualified']=False
        receipt['capability_states']['admitted_geometry_example']['state']='executed'
        receipt['capability_states']['admitted_geometry_library']['executed']=True
        receipt['capability_states']['admitted_geometry_library']['callable']=True
        receipt['capability_states']['admitted_geometry_library']['qualified']=False
        receipt['capability_states']['admitted_geometry_library']['state']='executed'
        stable={}
        for item in geo['artifacts']:
            name=item['file'];digest=sha(geometry/name)
            if digest!=item['sha256']:raise ValueError('geometry artifact hash mismatch')
            hashes=[sha(out/f'geometry-{i}'/name) for i in range(1,4)]
            if len(set(hashes))!=1:raise ValueError('geometry artifact nondeterministic')
            stable[name]=digest
        receipt['geometry_repeat_hashes']=stable
        receipt['geometry_repeat_scope']='three admitted-geometry asset builds from one executable; not three independent GPU runs'
        empty=run('negative-empty',[executables['geometry_exe'],'--out',out/'negative-empty','--empty'],expected=2)
        empty_result=json.loads(empty.stderr.strip().splitlines()[-1])
        if empty_result.get('status')!='invalid_input' or empty_result.get('message')!='empty fixture':raise ValueError('wrong empty-input failure reason')
        run('negative-target',[executables['geometry_exe'],'--out',out/'negative-target','--target-faces','0'],expected=2)
        target=json.loads((out/'negative-target/geometry-receipt.json').read_text())
        if target['status']!='unsatisfied_constraint':raise ValueError('impossible target not explicit')
        input_path=geometry/'decoded-vertices.json';asset_path=geometry/'simplified.10d'
        vertices=json.loads(input_path.read_text());input_before=sha(input_path);asset_before=sha(asset_path)
        run('forge-asset',[executables['forge_exe'],out/'forge-asset',args.expected_adapter,'--vertices-json',input_path,'--asset-sha256',asset_before])
        if input_before!=sha(input_path) or asset_before!=sha(asset_path):raise ValueError('input changed during execution')
        affine=json.loads((out/'forge-asset/asset-affine.json').read_text())
        if affine['outcome']!='passed' or affine['source_asset_sha256']!=asset_before:raise ValueError('Forge source binding mismatch')
        manifest=json.loads((out/'forge-asset/manifest.json').read_text())
        probe=json.loads((out/'forge-asset/probe.json').read_text())
        validation=json.loads((out/'forge-asset/validation.json').read_text())
        wgsl_blake3=blake3((out/'forge-asset/affine.wgsl').read_bytes()).hexdigest()
        if any(item.get('source_hash')!=wgsl_blake3 for item in [manifest,probe,validation,manifest['validation']]):raise ValueError('WGSL stage identity mismatch')
        if probe.get('outcome')!='passed' or manifest.get('validation_level')!='certified' or manifest['oracle'].get('mismatch_count')!=0 or manifest['oracle'].get('compared')!=4099:raise ValueError('seeded Forge qualification incomplete')
        receipt['kernel_identity']=dict(wgsl_blake3=wgsl_blake3,semantic_hash=manifest['semantic_hash'],schedule=manifest['schedule'],adapter=manifest['adapter'],seeded_oracle='existing CPU f32 mul_add; independent binary64 asset oracle recorded separately',gpu_timing_not_byte_deterministic=True)
        receipt['capability_states']['etalon_production_core_probe']['executed']=True
        receipt['capability_states']['etalon_production_core_probe']['callable']=True
        receipt['capability_states']['etalon_production_core_probe']['qualified']=False
        receipt['capability_states']['etalon_production_core_probe']['state']='executed'
        output=json.loads((out/'forge-asset/asset-affine-positions.json').read_text())
        input_flat=[f32(x) for p in vertices for x in p];output_flat=[f32(x) for p in output for x in p]
        actual_hashes=dict(vertices_json_blake3=blake3(input_path.read_bytes()).hexdigest(),decoded_f32_blake3=blake3(struct.pack('<'+str(len(input_flat))+'f',*input_flat)).hexdigest(),output_f32_blake3=blake3(struct.pack('<'+str(len(output_flat))+'f',*output_flat)).hexdigest())
        if any(affine.get(k)!=v for k,v in actual_hashes.items()):raise ValueError('Forge input/output digest mismatch')
        if affine.get('compared')!=len(input_flat) or affine.get('vertex_count')!=len(vertices) or affine.get('mismatch_count')!=0:raise ValueError('Forge counts mismatch')
        check=check_affine(vertices,output)
        poisoned=copy.deepcopy(output);poisoned[0][0]+=1
        try:check_affine(vertices,poisoned)
        except ValueError:check['known_bad_output_rejected']=True
        else:raise ValueError('independent oracle accepted known-bad output')
        receipt['asset_join']=dict(source_10d_sha256=asset_before,decoded_json_sha256=input_before,derived_json_sha256=sha(out/'forge-asset/asset-affine-positions.json'),geometry_decode_method='existing QualiaDB decode_10d_mesh inside geometry executable; artifact hashes checked here',source_binding_verified=True,independently_recomputed_blake3=actual_hashes,oracle=check)
        triangles=json.loads((geometry/'decoded-triangles.json').read_text());attributes=json.loads((geometry/'attributes.json').read_text())
        glb=attributed_glb(output,triangles,attributes);(out/'gpu-derived.glb').write_bytes(glb)
        baseline_bytes=list(asset_path.read_bytes())
        source='''requires [capability("capability.invoke")];
let mesh = capability.invoke("hypermedia.manifold.import_mesh", {input: "gpu-derived.glb", format: "glb"});
let sealed = capability.invoke("hypermedia.manifold.compile_10d_pbr", {mesh: mesh, topology: true, spatial_index: true});
let mut asset = {files: [
  {key: "gpu-derived.10d", resource: sealed},
  {key: "semantic-baseline.10d", kind: "10d", bytes: BASELINE_BYTES},
  {key: "geometry-receipt.json", kind: "json", text: GEOMETRY_RECEIPT},
  {key: "gpu-receipt.json", kind: "json", text: GPU_RECEIPT},
  {key: "semantic-attributes.json", kind: "json", text: SEMANTIC_SIDECAR}
]};
'''.replace('BASELINE_BYTES',json.dumps(baseline_bytes,separators=(',',':'))).replace('GEOMETRY_RECEIPT',json.dumps(json.dumps(geo,separators=(',',':')))).replace('GPU_RECEIPT',json.dumps(json.dumps(affine,separators=(',',':')))).replace('SEMANTIC_SIDECAR',json.dumps(json.dumps(dict(schema='production-core.semantic-sidecar/v1',applies_to='semantic-baseline.10d decoded vertex order',source_asset_sha256=asset_before,attributes=attributes),separators=(',',':'))))
        vibe=out/'production-core.vibe';vibe.write_text(source,encoding='utf-8')
        for i in range(1,4):
            run(f'produce-{i}',[executables['synth_exe'],'produce','--profile','files','--source',vibe,'--input','gpu-derived.glb','glb',out/'gpu-derived.glb','--out',out/f'production-{i}.hmc','--export-dir',out/f'export-{i}','--present',out/f'production-{i}.html'])
        hmc_hashes=[sha(out/f'production-{i}.hmc') for i in range(1,4)]
        if len(set(hmc_hashes))!=1:raise ValueError('three HMC builds differ')
        receipt['production']=dict(source_sha256=sha(vibe),input_glb_sha256=sha(out/'gpu-derived.glb'),hmc_sha256=hmc_hashes,deterministic_three_runs=True,determinism_scope='three geometry asset builds plus three HMC builds from one fixed Vibe/input/embedded receipt; GPU timing receipts not byte deterministic',joined_three_geometry_builds_are_not_three_gpu_runs=True,final_roundtrip_validation='not_executed',derived_10d_sha256=sha(out/'export-1/gpu-derived.10d'))
        receipt['capability_states']['producer_glb_smoke']['executed']=True
        receipt['capability_states']['producer_glb_smoke']['callable']=True
        receipt['capability_states']['producer_glb_smoke']['qualified']=False
        receipt['capability_states']['producer_glb_smoke']['state']='executed'
        if not args.skip_preview:
            run('preview-derived',[executables['synth_exe'],'preview','--asset',out/'export-1/gpu-derived.10d','--out',out/'preview-derived','--hmc',out/'production-1.hmc','--asset-key','outputs/gpu-derived.10d'])
            receipt['visual_review']='rendered_pending_independent_inspection'
        else:receipt['visual_review']='not_executed'
        for k,path in executables.items():
            if sha(path)!=receipt['executables'][k]['sha256']:raise ValueError('executable changed during tracer')
        receipt['status']='passed_numeric_and_packaging_pending_visual_review'
    except Exception as error:
        receipt['status']='failed';receipt['reason']=str(error);save();raise
    save();print(json.dumps(dict(status=receipt['status'],out=str(out),commands=len(receipt['commands']),invented_passing_geometry_receipt=False,gpu_identical_timing_claimed=False),indent=2))

if __name__=='__main__':main()
