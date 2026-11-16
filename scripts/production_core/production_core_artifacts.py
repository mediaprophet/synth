"""Independent tracer checks and a bounded attributed GLB handoff."""
import hashlib, json, math, struct
from pathlib import Path

HEX = '0123456789abcdef'
CAPABILITY_STATES = ('absent', 'present', 'callable', 'executed', 'qualified')
ADMITTED_SYMBOL = 'decimate_semantic_admitted_3'
LEGACY_LOWERHEX_FORMAT = 'format!("{:x}", Sha256::digest'
DETERMINISM_BOUNDARY = (
    'The joined tracer checking three geometry asset builds and three HMC builds '
    'is not three independent GPU runs with byte-identical timing. Repeated '
    'geometry/HMC SHA-256 values test packaging determinism of one fixed Vibe, '
    'input and embedded receipt. GPU timestamp samples and wall times are not '
    'byte-deterministic receipts and are not claimed identical.'
)
LOWERHEX_HISTORY = dict(
    schema='production-core.sha256-lowerhex-history/v1',
    previous_failure=dict(
        error='E0277',
        message='the trait bound Array<u8, ...>: LowerHex is not satisfied',
        example='crates/qualia-core-db/examples/production_core_geometry.rs',
        source_line_then='format!("{:x}", Sha256::digest(bytes))',
        rustc_note='generic SHA2 digest array does not implement fmt::LowerHex',
        log='qualiaDB/target-production-core/geometry-build-01.log',
        long_type='qualiaDB/target-production-core/debug/examples/production_core_geometry.long-type-10344221273783986174.txt',
        cargo_exit=101,
        observed_log_span_utc=['2026-11-16T06:52:05.969Z', '2026-11-16T07:32:30.054Z'],
        observed_log_span_is_not_full_command_duration=True,
    ),
    repair=dict(
        location='examples/production_core_geometry.rs sha256()',
        method='explicit nibble encoding over Sha256::digest(bytes).iter()',
        status='rebuilt',
        passing_receipt='tracer_run_not_qualification',
        rebuild_required=False,
        binary='qualiaDB/target-production-core/debug/examples/production_core_geometry.exe',
        tracer_receipt='synth/docs/production-core/evidence/admitted-geometry-run-01/geometry-receipt.json',
    ),
)
INCOMPLETE_CORE_CHECKLIST = [
    dict(item='geometry_example_rebuilt_after_lowerhex_repair', status='done'),
    dict(item='admitted_geometry_executable', status='present'),
    dict(item='joined_tracer_execution', status='executed_not_qualified'),
    dict(item='p01_p03_accepted', status='open'),
    dict(item='public_hypermedia_simplify_semantic', status='compiled_unit_tested'),
    dict(item='whole_surface_bound', status='not_measured'),
    dict(item='collapse_provenance_qualification', status='not_executed'),
    dict(item='operator_or_physical_qualification', status='not_executed'),
    dict(item='final_10d_sealing_independently_decoded', status='not_executed'),
    dict(item='gpu_timing_byte_identical_across_repeats', status='not_claimed'),
    dict(item='cached_synth_exe_source_equivalence', status='not_established'),
    dict(item='p02_p05_p06_p07_compiled_together', status='compiled'),
]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def sha256_lowerhex(data):
    """Explicit lowercase hex of SHA-256 digest bytes.

    Rust `format!("{:x}", Sha256::digest(bytes))` failed E0277 because the
    generic digest array does not implement `LowerHex`. The geometry example
    repair iterates digest bytes and encodes nibbles; this helper matches that
    encoding. A matching Python digest is not a passing QualiaDB receipt.
    """
    if isinstance(data, str):
        data = data.encode('utf-8')
    digest = hashlib.sha256(data).digest()
    return ''.join(HEX[byte >> 4] + HEX[byte & 15] for byte in digest)

def classify_capability(*, present=False, is_callable=False, executed=False, qualified=False):
    """Distinct ladder: present, callable, executed, qualified.

    Library source on disk is not a callable binary. A callable binary is not
    an executed run. An executed run is not qualification. Qualification may
    not skip the earlier states.
    """
    present=bool(present);is_callable=bool(is_callable);executed=bool(executed);qualified=bool(qualified)
    if executed and not is_callable:
        raise ValueError('executed requires callable')
    if qualified and not (present and is_callable and executed):
        raise ValueError('qualified requires present, callable, and executed')
    if qualified:return 'qualified'
    if executed:return 'executed'
    if is_callable:return 'callable'
    if present:return 'present'
    return 'absent'

def capability_record(*, present=False, is_callable=False, executed=False, qualified=False, **extra):
    record=dict(
        present=bool(present),
        callable=bool(is_callable),
        executed=bool(executed),
        qualified=bool(qualified),
        state=classify_capability(present=present,is_callable=is_callable,executed=executed,qualified=qualified),
    )
    record.update(extra)
    return record

def documented_commands(worktrees, synth_exe=None):
    """Commands to run later. Recording them is not execution evidence."""
    q=Path(worktrees['qualiaDB']);e=Path(worktrees['etalon']);s=Path(worktrees['synth'])
    synth_exe=Path(synth_exe) if synth_exe is not None else Path(r'C:\github\synth\target-prod\debug\synth.exe')
    geometry_exe=q/'target-production-core'/'debug'/'examples'/'production_core_geometry.exe'
    forge_exe=e/'target-production-core'/'debug'/'examples'/'production_core_probe.exe'
    tracer=s/'scripts'/'production_core'/'run_production_core_tracer.py'
    probe=s/'scripts'/'production_core'/'admitted_geometry_probe.py'
    return [
        dict(
            id='qualiadb-production-core-geometry',
            title='QualiaDB example production_core_geometry (admitted semantic path)',
            cwd=str(q),
            notes=[
                'Example source calls decimate_semantic_admitted_3; this is not a compiled binary.',
                'Previous cargo build exited 101 on SHA-256 LowerHex formatting; source repair is unbuilt.',
                'Do not invent a passing geometry receipt. Do not rebuild into synth target or target-studio.',
            ],
            build_powershell=[
                "$env:CARGO_BUILD_JOBS = '1'",
                "$env:CARGO_INCREMENTAL = '0'",
                "$env:RUSTFLAGS = '-C debuginfo=0'",
                'cargo --config .cargo/production-core.toml build --offline --no-default-features '
                '-p qualia-core-db --example production_core_geometry --target-dir target-production-core',
            ],
            run_powershell=[
                f'& "{geometry_exe}" --out <new-create-only-dir>',
                f'& "{geometry_exe}" --out <new-create-only-dir> --empty',
                f'& "{geometry_exe}" --out <new-create-only-dir> --target-faces 0',
            ],
        ),
        dict(
            id='etalon-production-core-probe',
            title='Etalon production_core_probe',
            cwd=str(e),
            notes=[
                'Fixed AffineF32 kernel only. Optional --vertices-json requires a real decoded geometry asset.',
                'Prior native DX12 baseline is separate evidence; this command list is not a GPU replay.',
                'GPU timestamp samples are not byte-identical across runs.',
            ],
            build_powershell=[
                f"$env:CARGO_TARGET_DIR = '{e / 'target-production-core'}'",
                'cargo build --locked --offline -p etalon-wgsl-forge --example production_core_probe --features gpu-native-dx12',
            ],
            run_powershell=[
                "$env:ETALON_WGPU_BACKEND = 'dx12'",
                f'& "{forge_exe}" <new-create-only-dir> NVIDIA',
                f'& "{forge_exe}" <new-create-only-dir> NVIDIA --vertices-json <decoded-vertices.json> --asset-sha256 <simplified.10d-sha256>',
            ],
        ),
        dict(
            id='producer-glb-smoke',
            title='Existing producer GLB smoke / joined tracer packaging',
            cwd=str(s),
            notes=[
                'Attributed-handoff-smoke already recorded a packaging pass of a six-vertex fixture.',
                'Joined tracer produce/preview uses the same cached producer binary identity; source equivalence is not assumed.',
                'Three HMC repeats test packaging hashes, not three independent GPU timing receipts.',
            ],
            historical_smoke=str(s/'docs'/'production-core'/'evidence'/'attributed-handoff-smoke'/'receipt.json'),
            inspect_powershell=[
                f'python "{probe}" --out <new-create-only-dir> --inspect-only',
            ],
            run_powershell=[
                f'python "{tracer}" --inspect-only --out <new-create-only-dir>',
                f'python "{tracer}" --geometry-exe "{geometry_exe}" --forge-exe "{forge_exe}" '
                f'--synth-exe "{synth_exe}" --out <new-create-only-dir>',
            ],
        ),
    ]

def f32(value):
    if isinstance(value,bool) or not isinstance(value,(float,int)) or not math.isfinite(value):
        raise ValueError('expected finite numeric input')
    try:result=struct.unpack('<f',struct.pack('<f',value))[0]
    except (OverflowError,struct.error) as error:raise ValueError('outside f32 domain') from error
    if not math.isfinite(result):raise ValueError('outside finite f32 domain')
    return result

def check_affine(source,output):
    if len(source)!=len(output) or not source:
        raise ValueError('vertex count mismatch or empty input')
    maximum=0.0
    for a,b in zip(source,output):
        if len(a)!=3 or len(b)!=3:
            raise ValueError('expected xyz triples')
        for x,y in zip(a,b):
            expected=2.0*f32(x)+.25
            actual=f32(y)
            error=abs(actual-expected)
            if not math.isfinite(expected) or not math.isfinite(error) or error>1e-6+1e-5*abs(expected):
                raise ValueError('independent affine oracle mismatch')
            maximum=max(maximum,error)
    return dict(compared=3*len(source),maximum_absolute_error=maximum,reference='independent Python binary64 on source f32',absolute_tolerance=1e-6,relative_tolerance=1e-5)

def attributed_glb(positions,triangles,attributes):
    if not 0<len(positions)<=21845 or len(attributes)!=len(positions) or len(triangles)>100000:
        raise ValueError('GLB handoff exceeds admitted fixture or attribute count')
    for p in positions:
        if len(p)!=3:raise ValueError('position must be xyz')
        for x in p:f32(x)
    positions=[[f32(x) for x in p] for p in positions]
    for a in attributes:
        for key,width in [('uv',2),('normal',3)]:
            if not isinstance(a.get(key),list) or len(a[key])!=width:raise ValueError('invalid attribute shape')
            for x in a[key]:f32(x)
        if any(abs(x)>1e6 for x in a['uv']):raise ValueError('UV outside codec domain')
    blob=bytearray();views=[];accessors=[]
    def accessor(rows,kind,component,pack,target):
        while len(blob)%4:blob.append(0)
        offset=len(blob);flat=[x for row in rows for x in row]
        blob.extend(struct.pack('<'+str(len(flat))+pack,*flat))
        views.append(dict(buffer=0,byteOffset=offset,byteLength=len(blob)-offset,target=target))
        item=dict(bufferView=len(views)-1,componentType=component,count=len(rows),type=kind)
        if kind=='VEC3':
            item.update(min=[min(r[i] for r in rows) for i in range(3)],max=[max(r[i] for r in rows) for i in range(3)])
        accessors.append(item);return len(accessors)-1
    pos=accessor(positions,'VEC3',5126,'f',34962)
    uv=accessor([a['uv'] for a in attributes],'VEC2',5126,'f',34962)
    normals=accessor([a['normal'] for a in attributes],'VEC3',5126,'f',34962)
    primitives=[]
    for material,label in enumerate([101,202]):
        indices=[]
        for tri in triangles:
            if len(tri)!=3 or any(isinstance(i,bool) or not isinstance(i,int) or i<0 or i>=len(positions) for i in tri):raise ValueError('bad triangle')
            labels={attributes[i]['subpart_tag'] for i in tri}
            if len(labels)!=1 or not labels.issubset({101,202}):raise ValueError('mixed or unknown semantic domain')
            if next(iter(labels))==label:indices.extend([[i] for i in tri])
        if not indices:raise ValueError('missing protected subpart')
        idx=accessor(indices,'SCALAR',5125,'I',34963)
        primitives.append(dict(attributes={'POSITION':pos,'NORMAL':normals,'TEXCOORD_0':uv},indices=idx,material=material,mode=4))
    materials=[dict(name=name,doubleSided=True,pbrMetallicRoughness=dict(baseColorFactor=color,metallicFactor=0.,roughnessFactor=.65)) for name,color in [('subpart-101',[.1,.4,.85,1]),('subpart-202',[.9,.25,.12,1])]]
    document=dict(asset=dict(version='2.0',generator='production-core-attributed-handoff-v1'),buffers=[dict(byteLength=len(blob))],bufferViews=views,accessors=accessors,materials=materials,meshes=[dict(primitives=primitives)],nodes=[dict(mesh=0)],scenes=[dict(nodes=[0])],scene=0)
    header=json.dumps(document,separators=(',',':'),allow_nan=False).encode()
    header+=b' '*((-len(header))%4);blob+=b'\0'*((-len(blob))%4)
    result=struct.pack('<III',0x46546c67,2,12+8+len(header)+8+len(blob))+struct.pack('<II',len(header),0x4e4f534a)+header+struct.pack('<II',len(blob),0x004e4942)+blob
    if len(result)>4*1024*1024:raise ValueError('GLB handoff exceeds 4MiB')
    return result
