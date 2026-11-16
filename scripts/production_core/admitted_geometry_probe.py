"""Inspect or invoke the admitted QualiaDB geometry path and record honest receipts.

Default inspect mode does not run cargo, does not touch GPU, and does not
invent a passing geometry receipt. Library present, callable, executed and
qualified remain distinct states.
"""
from pathlib import Path
import argparse, datetime, hashlib, json, subprocess, time

from production_core_artifacts import (
    ADMITTED_SYMBOL, DETERMINISM_BOUNDARY, INCOMPLETE_CORE_CHECKLIST,
    LEGACY_LOWERHEX_FORMAT, LOWERHEX_HISTORY, capability_record,
    documented_commands, sha, sha256_lowerhex,
)

GEOM_EXAMPLE = Path('crates/qualia-core-db/examples/production_core_geometry.rs')
ADMISSION = Path('crates/qualia-core-db/src/specialized_libs/computational_geometry/decimate_3_semantic/admission.rs')
SEMANTIC_MOD = Path('crates/qualia-core-db/src/specialized_libs/computational_geometry/decimate_3_semantic.rs')
FORGE_EXAMPLE = Path('crates/etalon-wgsl-forge/examples/production_core_probe.rs')
GEOM_EXE_REL = Path('target-production-core/debug/examples/production_core_geometry.exe')
FORGE_EXE_REL = Path('target-production-core/debug/examples/production_core_probe.exe')
BUILD_LOG_REL = Path('target-production-core/geometry-build-01.log')
CACHED_SYNTH = Path(r'C:\github\synth\target-prod\debug\synth.exe')


def discover_worktrees(root=None):
    if root is None:
        root = Path(__file__).resolve().parents[2].parent
    root = Path(root)
    return dict(
        root=root,
        synth=root/'synth',
        qualiaDB=root/'qualiaDB',
        etalon=root/'etalon',
        hypermedia=root/'hypermedia-production',
    )


def _read(path):
    return path.read_text(encoding='utf-8', errors='replace') if path.is_file() else None


def _source_record(path, required_snippets=(), forbidden_snippets=()):
    text = _read(path)
    present = text is not None
    snippets = {snip: (snip in text if present else False) for snip in required_snippets}
    forbidden = {snip: (snip in text if present else False) for snip in forbidden_snippets}
    return dict(
        path=str(path),
        present=present,
        sha256=sha(path) if present else None,
        required_snippets=snippets,
        forbidden_snippets=forbidden,
    )


def inspect_admitted_geometry(worktrees=None, geometry_exe=None, forge_exe=None, synth_exe=None):
    worktrees = worktrees or discover_worktrees()
    q = Path(worktrees['qualiaDB'])
    e = Path(worktrees['etalon'])
    s = Path(worktrees['synth'])
    geometry_exe = Path(geometry_exe).resolve() if geometry_exe else q/GEOM_EXE_REL
    forge_exe = Path(forge_exe).resolve() if forge_exe else e/FORGE_EXE_REL
    synth_exe = Path(synth_exe).resolve() if synth_exe else CACHED_SYNTH

    example = _source_record(
        q/GEOM_EXAMPLE,
        required_snippets=(ADMITTED_SYMBOL, 'const HEX:', 'Sha256::digest'),
        forbidden_snippets=(LEGACY_LOWERHEX_FORMAT,),
    )
    admission = _source_record(q/ADMISSION, required_snippets=(f'pub fn {ADMITTED_SYMBOL}',))
    semantic = _source_record(q/SEMANTIC_MOD, required_snippets=('pub mod admission',))
    forge_src = _source_record(e/FORGE_EXAMPLE, required_snippets=('production_core_probe', 'AffineF32'))
    library_present = admission['present'] and admission['required_snippets'].get(f'pub fn {ADMITTED_SYMBOL}', False)
    example_wired = example['present'] and example['required_snippets'].get(ADMITTED_SYMBOL, False)
    lowerhex_repair = (
        example['present']
        and example['required_snippets'].get('const HEX:', False)
        and not example['forbidden_snippets'].get(LEGACY_LOWERHEX_FORMAT, False)
    )
    build_log = q/BUILD_LOG_REL
    log_has_lowerhex = False
    if build_log.is_file():
        log_text = _read(build_log)
        log_has_lowerhex = 'LowerHex' in log_text and 'E0277' in log_text
    smoke_receipt = s/'docs'/'production-core'/'evidence'/'attributed-handoff-smoke'/'receipt.json'
    forge_baseline = s/'docs'/'production-core'/'evidence'/'forge-native-baseline'/'probe.json'
    geometry_callable = geometry_exe.is_file()
    forge_callable = forge_exe.is_file()
    synth_callable = synth_exe.is_file()

    capabilities = dict(
        admitted_geometry_library=capability_record(
            present=library_present,
            is_callable=geometry_callable,
            executed=False,
            qualified=False,
            public_api=f'decimate_3_semantic::admission::{ADMITTED_SYMBOL}',
            parent_export=semantic['required_snippets'].get('pub mod admission', False),
            note='Source presence is not compilation, execution or qualification.',
        ),
        admitted_geometry_example=capability_record(
            present=example['present'],
            is_callable=geometry_callable,
            executed=False,
            qualified=False,
            source_wired_to_admitted_driver=example_wired,
            lowerhex_repair_in_source=lowerhex_repair,
            executable=str(geometry_exe),
            note='Example is the admitted semantic path. Callable means the rebuilt binary exists; inspect does not execute it.',
        ),
        etalon_production_core_probe=capability_record(
            present=forge_src['present'],
            is_callable=forge_callable,
            executed=False,
            qualified=False,
            executable=str(forge_exe),
            prior_native_baseline=str(forge_baseline) if forge_baseline.is_file() else None,
            note='Prior native DX12 baseline is separate evidence and is not replayed by inspect.',
        ),
        producer_glb_smoke=capability_record(
            present=smoke_receipt.is_file(),
            is_callable=synth_callable,
            executed=False,
            qualified=False,
            executable=str(synth_exe),
            prior_attributed_handoff_smoke=str(smoke_receipt) if smoke_receipt.is_file() else None,
            note='Historical packaging smoke is not joined admitted-geometry qualification.',
        ),
    )
    return dict(
        schema='production-core.admitted-geometry-probe/v1',
        worktrees={k: str(v) for k, v in worktrees.items()},
        sources=dict(example=example, admission=admission, semantic_mod=semantic, forge_example=forge_src),
        sha256_lowerhex_history=LOWERHEX_HISTORY,
        lowerhex_repair_in_source=lowerhex_repair,
        previous_build_log=dict(path=str(build_log), present=build_log.is_file(), records_e0277_lowerhex=log_has_lowerhex),
        capabilities=capabilities,
        documented_commands=documented_commands(worktrees, synth_exe=synth_exe),
        determinism_boundary=DETERMINISM_BOUNDARY,
        joined_three_geometry_builds_are_not_three_gpu_runs=True,
        gpu_identical_timing_claimed=False,
        invented_passing_geometry_receipt=False,
        incomplete_core_checklist=INCOMPLETE_CORE_CHECKLIST,
        inspect_executed_gpu=False,
        inspect_ran_cargo=False,
        sha256_lowerhex_encoding=dict(
            algorithm='explicit_nibble_lowercase_hex',
            marker='production-core-lowerhex-marker',
            digest=sha256_lowerhex('production-core-lowerhex-marker'),
            hashlib_hexdigest=hashlib.sha256(b'production-core-lowerhex-marker').hexdigest(),
            note='Matching Python encodings do not qualify the QualiaDB example.',
        ),
    )


def invoke_admitted_geometry(exe, out_dir, extra_args=None, timeout=600):
    """Run the admitted geometry example. Absence is a failed receipt, not a pass."""
    exe = Path(exe); out_dir = Path(out_dir)
    started = time.perf_counter()
    command = [str(exe), '--out', str(out_dir), *(str(x) for x in (extra_args or []))]
    if not exe.is_file():
        return dict(
            label='admitted-geometry',
            argv=command,
            status='missing_executable',
            exit_code=None,
            wall_seconds=0.0,
            note='Geometry executable is missing; no passing receipt invented.',
        )
    process = subprocess.run(command, capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=timeout)
    return dict(
        label='admitted-geometry',
        argv=command,
        status='executed',
        exit_code=process.returncode,
        wall_seconds=time.perf_counter()-started,
        stdout=process.stdout,
        stderr=process.stderr,
        qualified=False,
        note='Execution is not qualification. GPU timing is not produced by this executable.',
    )


def write_receipt(out, receipt):
    out = Path(out)
    (out/'admitted-probe-receipt.json').write_text(json.dumps(receipt, indent=2, allow_nan=False)+'\n', encoding='utf-8')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--worktree-root', type=Path)
    parser.add_argument('--geometry-exe', type=Path)
    parser.add_argument('--forge-exe', type=Path)
    parser.add_argument('--synth-exe', type=Path)
    parser.add_argument('--inspect-only', action='store_true', default=False)
    parser.add_argument('--invoke', action='store_true', help='Run the geometry example once if the executable exists. Never runs GPU.')
    args = parser.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=False, exist_ok=False)
    inspection = inspect_admitted_geometry(
        worktrees=discover_worktrees(args.worktree_root),
        geometry_exe=args.geometry_exe,
        forge_exe=args.forge_exe,
        synth_exe=args.synth_exe,
    )
    receipt = dict(
        schema='production-core.admitted-geometry-probe-receipt/v1',
        status='running',
        started_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
        inspection=inspection,
        sha256_lowerhex_history=LOWERHEX_HISTORY,
        determinism_boundary=DETERMINISM_BOUNDARY,
        joined_three_geometry_builds_are_not_three_gpu_runs=True,
        gpu_identical_timing_claimed=False,
        invented_passing_geometry_receipt=False,
        commands=[],
    )
    try:
        if args.invoke and not args.inspect_only:
            geometry_exe = args.geometry_exe or Path(inspection['capabilities']['admitted_geometry_example']['executable'])
            invocation = invoke_admitted_geometry(geometry_exe, out/'geometry-1')
            receipt['commands'].append({k: invocation[k] for k in invocation if k not in ('stdout', 'stderr')})
            (out/'geometry-1.stdout.txt').write_text(invocation.get('stdout') or '', encoding='utf-8')
            (out/'geometry-1.stderr.txt').write_text(invocation.get('stderr') or '', encoding='utf-8')
            if invocation['status'] != 'executed' or invocation.get('exit_code') != 0:
                receipt['status'] = 'failed'
                receipt['reason'] = invocation.get('note') or f"geometry invoke {invocation['status']}"
                inspection['capabilities']['admitted_geometry_example']['executed'] = invocation['status'] == 'executed'
                inspection['capabilities']['admitted_geometry_example']['state'] = (
                    'executed' if invocation['status'] == 'executed' else inspection['capabilities']['admitted_geometry_example']['state']
                )
            else:
                inspection['capabilities']['admitted_geometry_example']['executed'] = True
                inspection['capabilities']['admitted_geometry_example']['qualified'] = False
                inspection['capabilities']['admitted_geometry_example']['state'] = 'executed'
                receipt['status'] = 'executed_not_qualified'
        else:
            receipt['status'] = 'inspected_not_executed'
        receipt['inspection'] = inspection
    except Exception as error:
        receipt['status'] = 'failed'
        receipt['reason'] = str(error)
        write_receipt(out, receipt)
        raise
    write_receipt(out, receipt)
    print(json.dumps(dict(status=receipt['status'], out=str(out), invented_passing_geometry_receipt=False), indent=2))


if __name__ == '__main__':
    main()
