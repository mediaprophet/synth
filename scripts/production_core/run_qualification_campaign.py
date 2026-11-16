"""P22 corpus qualification runner.

Invokes existing binaries and oracles, records honest receipts, and never
invents GPU results. Mesh fixtures are passed to the admitted geometry example
as `--corpus-json`; `--empty` and `--target-faces 0` remain documented negatives.
Holdout and maximum-size descriptors are not invoked.
"""
from __future__ import annotations

from pathlib import Path
import argparse
import datetime
import json
import sys
import time

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))
_VENDOR = _SCRIPT_DIR / 'vendor'
if _VENDOR.is_dir() and str(_VENDOR) not in sys.path:
    sys.path.insert(0, str(_VENDOR))

from qualification_corpus import FAMILIES, digest_bytes, hash_backend
from oracle_harness import OracleError, evaluate_fixture, observe_known_bad_gates
from admitted_geometry_probe import (
    discover_worktrees,
    inspect_admitted_geometry,
    invoke_admitted_geometry,
)
from production_core_artifacts import DETERMINISM_BOUNDARY, inspect_whole_surface_bound, sha
from decode_10d_harness import (
    b0_identity_ablation,
    compact_decode_record,
    inspect_collapse_provenance,
    inspect_sealed_10d_dir,
)

SCHEMA = 'production-core.qualification-campaign/v1'
BUILTIN_GEOMETRY_FIXTURE = 'normalized-tilted-two-chart-grid'

# Named seam/analytic fixtures the campaign always tries to load as corpus JSON.
SEAM_ANALYTIC_FIXTURE_IDS = frozenset({
    'analytic-plane-scale-1',
    'identity-two-subpart-seam',
    'mixed-provenance-domains',
    'uv-duplicated-seam',
})
NEGATIVE_INVOCATIONS = {
    'empty-mesh': ('--empty',),
    'impossible-ratio': ('--target-faces', '0'),
}
SKIP_INVOCATION_FAMILIES = frozenset({
    'holdout',
    'maximum_size_descriptor',
    'known_bad',
})

ABLATION_DEFINITIONS = (
    dict(
        id='B0',
        title='identity/no reduction',
        purpose='Reference geometry, semantics, and response.',
        relevance='required',
    ),
    dict(
        id='B1',
        title='existing geometric QEM',
        purpose='Establish value beyond a cheap current route.',
        relevance='required',
    ),
    dict(
        id='B2',
        title='existing semantic QEM under the same hard constraints',
        purpose='Isolate benefit beyond semantic penalties and locks.',
        relevance='required',
    ),
    dict(
        id='B3',
        title='current remeshing/anisotropic route',
        purpose='Compare element-quality and geometry objectives.',
        relevance='when_appropriate',
    ),
    dict(
        id='B4',
        title='fixed supported Forge schedule',
        purpose='Measure the value and cost of tuning or generation changes.',
        relevance='required',
    ),
    dict(
        id='B5',
        title='current fine physical/operator model',
        purpose='Establish response and operator errors.',
        relevance='required',
    ),
)


def parse_family_filters(values):
    """Parse repeated/comma-separated --families values. None means all families."""
    if not values:
        return None
    names = []
    for item in values:
        for part in str(item).split(','):
            name = part.strip()
            if not name:
                continue
            if name not in FAMILIES:
                raise ValueError(f'unknown family {name}')
            if name not in names:
                names.append(name)
    return names or None


def create_only_dir(path):
    path = Path(path)
    path.mkdir(parents=False, exist_ok=False)
    return path


def write_create_only_text(path, text):
    path = Path(path)
    with path.open('x', encoding='utf-8', newline='\n') as handle:
        handle.write(text if text is not None else '')


def cpu_wall_timing(wall_seconds=None, source='python_perf_counter', samples_ms=None):
    record = dict(
        kind='cpu_wall',
        wall_seconds=wall_seconds,
        gpu=False,
        synthetic=False,
        source=source,
        note='CPU wall time. Not a GPU timestamp sample.',
    )
    if samples_ms is not None:
        record['samples_ms'] = samples_ms
        record['units'] = 'milliseconds'
    return record


def absent_timing():
    return dict(
        kind='not_executed',
        wall_seconds=None,
        gpu=False,
        synthetic=False,
        note='No timing sample. Absence is not a zero GPU measurement.',
    )


def fixture_mesh_payload(fixture):
    """Return the triangle-mesh object from a published fixture, or None."""
    data = fixture.get('data') or {}
    if data.get('kind') == 'representation_candidate':
        payload = data.get('input') or {}
    else:
        payload = data
    vertices = payload.get('vertices')
    triangles = payload.get('triangles')
    if isinstance(vertices, list) and isinstance(triangles, list) and vertices and triangles:
        return payload
    return None


EXISTING_SURFACE_RECEIPTS = (
    ('admitted-geometry-run-01', 'geometry-receipt.json'),
    ('geometry-corpus-analytic-plane-01', 'geometry-receipt.json'),
    ('geometry-corpus-identity-seam-01', 'geometry-receipt.json'),
    ('joined-tracer-01', 'geometry-1', 'geometry-receipt.json'),
    ('qualification-campaign-run-02', 'geometry-identity-two-subpart-seam', 'geometry-receipt.json'),
    ('qualification-campaign-run-02', 'geometry-analytic-plane-scale-1', 'geometry-receipt.json'),
)


def existing_geometry_receipt_paths(synth_root):
    evidence = Path(synth_root) / 'docs' / 'production-core' / 'evidence'
    paths = []
    for parts in EXISTING_SURFACE_RECEIPTS:
        path = evidence.joinpath(*parts)
        if path.is_file():
            paths.append(path)
    return paths


def whole_surface_bound_from_existing_receipts(synth_root, extra_paths=None):
    """Read-only inspect of geometry-receipt files. Never overwrites those dirs."""
    paths = list(existing_geometry_receipt_paths(synth_root))
    for path in extra_paths or []:
        path = Path(path)
        if path.is_file() and path not in paths:
            paths.append(path)
    reports = [inspect_whole_surface_bound(path) for path in paths]
    enclosing = [item for item in reports if item.get('class') == 'enclosing_bound']
    if enclosing:
        return {
            'status': 'executed_not_qualified',
            'class': 'enclosing_bound',
            'qualified': False,
            'inspected': len(reports),
            'enclosing_count': len(enclosing),
            'enclosing_receipts': [item['path'] for item in enclosing],
            'note': 'geometry-receipt reports enclosing_bound class. Execution is not qualification. Evidence directories were not overwritten.',
        }
    return {
        'status': 'not_measured',
        'class': None,
        'qualified': False,
        'inspected': len(reports),
        'enclosing_receipts': [],
        'receipts': [item.get('path') for item in reports],
        'note': 'geometry-receipt files have no enclosing_bound class. Status is not_measured, not 0.0.',
    }


def campaign_geometry_receipt_paths(out):
    root = Path(out)
    if not root.is_dir():
        return []
    return sorted(root.glob('geometry-*/geometry-receipt.json'))


def b1_invoked_from_records(records):
    """True only when a fixture actually ran with --driver geometric."""
    for record in records:
        if record.get('execution_status') != 'executed':
            continue
        argv = (record.get('geometry_invocation') or {}).get('argv') or []
        text = [str(part) for part in argv]
        for i, part in enumerate(text):
            if part == '--driver' and i + 1 < len(text) and text[i + 1] == 'geometric':
                return True
    return False


def campaign_provenance_summary(records, search_dir=None):
    """Fixture-level snapshots win over a missing campaign-root file. Absence is not 0.0."""
    present = [
        item for item in records
        if (item.get('collapse_provenance') or {}).get('status') == 'present'
    ]
    not_supplied = [
        item for item in records
        if (item.get('collapse_provenance') or {}).get('status') == 'not_supplied'
    ]
    if present:
        return dict(
            status='present_on_fixtures',
            sha256=None,
            fixture_count=len(present),
            not_supplied_count=len(not_supplied),
            qualified=False,
            note='QCLN0001 snapshots were hashed on invoked fixtures. Hashing is not collapse-provenance qualification.',
        )
    root = inspect_collapse_provenance(search_dir=search_dir)
    if not_supplied and root.get('status') == 'not_supplied':
        root = dict(root)
        root['fixture_not_supplied_count'] = len(not_supplied)
    return root


def geometry_example_driver_support(inspection=None, example_source_text=None):
    """CLI --driver geometric is planned until the example argv parser contains --driver."""
    snippets = {}
    if inspection:
        snippets = ((inspection.get('sources') or {}).get('example') or {}).get('required_snippets') or {}
    cli = bool(snippets.get('--driver'))
    if example_source_text is not None:
        cli = '--driver' in example_source_text
    exe_path = None
    if inspection:
        exe_path = ((inspection.get('capabilities') or {}).get('admitted_geometry_example') or {}).get('executable')
    exe_ok = Path(exe_path).is_file() if exe_path else False
    return dict(
        driver_flag_in_example_source=cli,
        planned_argv=['--driver', 'geometric'],
        present=cli or bool(snippets.get('decimate_qem')),
        callable=bool(cli and exe_ok),
        executed=False,
        qualified=False,
        note=(
            'B1 uses --driver geometric when the example argv parser contains --driver. '
            'A Driver enum or decimate_qem import is not a callable CLI. '
            'Absence keeps B1 not_executed; the flag is not passed to an exe that would reject unknown arguments.'
        ),
    )


def geometry_invocation_args(
    fixture_id, *, corpus=None, fixture_file=None, fixture=None, driver=None, supports_driver=False,
):
    """Return extra argv for the admitted example, or None if it does not support the fixture.

    `--driver geometric` is appended only for mesh corpus JSON when the example
    source actually parses `--driver`. Known-bad, holdout, and descriptor
    fixtures never receive invocation args.
    """
    if fixture_id in NEGATIVE_INVOCATIONS:
        return list(NEGATIVE_INVOCATIONS[fixture_id])
    family = (fixture or {}).get('family')
    partition = (fixture or {}).get('partition')
    if family in SKIP_INVOCATION_FAMILIES or partition == 'holdout':
        return None
    payload = fixture_mesh_payload(fixture) if fixture is not None else None
    has_mesh = payload is not None or fixture_id in SEAM_ANALYTIC_FIXTURE_IDS
    if not has_mesh:
        return None
    if corpus is None:
        return None
    file_name = fixture_file or f'{fixture_id}.json'
    extra = ['--corpus-json', str(Path(corpus) / file_name)]
    if driver and supports_driver:
        extra.extend(['--driver', str(driver)])
    return extra


def geometry_exe_plan(
    fixture, *, inspect_only, geometry_exe_present, corpus=None, fixture_file=None,
    driver=None, supports_driver=False,
):
    """Decide whether to invoke the admitted geometry example for one fixture."""
    fid = fixture['id']
    family = fixture['family']
    holdout = family == 'holdout' or fixture.get('partition') == 'holdout'
    must_fail = family == 'known_bad' or (fixture.get('oracle') or {}).get('must_fail') is True
    must_pass = family == 'identity' or (fixture.get('oracle') or {}).get('must_pass') is True
    extra = None if inspect_only else geometry_invocation_args(
        fid, corpus=corpus, fixture_file=fixture_file, fixture=fixture,
        driver=driver, supports_driver=supports_driver,
    )
    consumes = bool(extra and '--corpus-json' in extra)
    base = dict(
        fixture_id=fid,
        family=family,
        holdout=holdout,
        must_fail=must_fail,
        must_pass=must_pass,
        in_denominator=True,
        used_for_threshold_selection=False,
        consumes_corpus_bytes=consumes,
        builtin_fixture=BUILTIN_GEOMETRY_FIXTURE,
    )
    if family == 'maximum_size_descriptor':
        base.update(
            action='descriptor_only',
            execution_status='descriptor_only',
            extra_args=None,
            reason='maximum-size remains a descriptor; 50k-vertex mesh was not materialized',
        )
        return base
    if inspect_only:
        base.update(
            action='skip',
            execution_status='not_executed',
            extra_args=None,
            reason='inspect-only; capabilities recorded without invoking executables',
        )
        return base
    if extra is None:
        base.update(
            action='skip',
            execution_status='not_executed',
            extra_args=None,
            reason='geometry executable does not consume this corpus fixture',
        )
        return base
    if not geometry_exe_present:
        base.update(
            action='skip',
            execution_status='not_executed',
            extra_args=extra,
            reason='geometry executable absent; no passing receipt invented',
        )
        return base
    reason = (
        'admitted example --corpus-json for this published fixture'
        if consumes
        else 'admitted example documented negative flag'
    )
    base.update(
        action='invoke',
        execution_status='pending',
        extra_args=extra,
        reason=reason,
    )
    return base


def known_bad_verdict(
    fixture, *, execution_status, exe_passed, observed_failed_gates, unevaluated_gates=None,
):
    """Known-bad fixtures stay in the denominator. A passing exe is not success.

    Named gates are observed on the published CANDIDATE by oracle_harness.
    Unobserved gates stay unevaluated, never a fake fail. Invoking the geometry
    exe on a welded candidate as if it were an input mesh is forbidden; a
    passing executable without failed candidate gates is a corpus defect.
    """
    oracle = fixture.get('oracle') or {}
    required = list(oracle.get('failing_tests') or [])
    observed = list(observed_failed_gates or [])
    unevaluated = list(unevaluated_gates or [])
    gates_failed = bool(required) and set(required).issubset(set(observed))
    production_exe_invoked = execution_status == 'executed'
    verdict = dict(
        must_fail=True,
        pass_is_corpus_defect=oracle.get('pass_is_corpus_defect', True),
        failing_tests=required,
        observed_failed_gates=observed,
        unevaluated_gates=unevaluated,
        named_gates_failed=gates_failed,
        in_denominator=True,
        qualified=False,
        production_exe_invoked=production_exe_invoked,
        oracle_route='oracle_harness.observe_known_bad_gates',
    )
    if exe_passed and production_exe_invoked and not gates_failed:
        verdict.update(
            corpus_success=False,
            reason='known-bad: passing executable is a corpus defect unless named candidate gates failed',
        )
        return verdict
    if gates_failed:
        verdict.update(
            corpus_success=True,
            reason=(
                'known-bad named gates failed as required on the published candidate '
                '(oracle_harness). Geometry exe was not used as an input mesh. '
                'Execution is still not qualification.'
            ),
        )
        return verdict
    if unevaluated and not observed:
        verdict.update(
            corpus_success=False,
            reason='known-bad remains in the denominator; named gates were unobserved, not fake-failed',
        )
        return verdict
    if execution_status in ('not_executed', 'descriptor_only') and not observed:
        verdict.update(
            corpus_success=False,
            reason='known-bad remains in the denominator; named gates were not observed',
        )
        return verdict
    verdict.update(
        corpus_success=False,
        reason='known-bad: named gates did not fail',
    )
    return verdict


def fixture_verdict(
    fixture, *, execution_status, exe_passed, observed_failed_gates, consumed_corpus_bytes,
    unevaluated_gates=None,
):
    family = fixture['family']
    oracle = fixture.get('oracle') or {}
    if family == 'known_bad' or oracle.get('must_fail') is True:
        return known_bad_verdict(
            fixture,
            execution_status=execution_status,
            exe_passed=exe_passed,
            observed_failed_gates=observed_failed_gates,
            unevaluated_gates=unevaluated_gates,
        )
    if oracle.get('must_pass') is True:
        if execution_status != 'executed' or not consumed_corpus_bytes or not exe_passed:
            return dict(
                corpus_success=False,
                qualified=False,
                must_pass=True,
                in_denominator=True,
                reason='must_pass not demonstrated on corpus fixture bytes; builtin exe mesh is not this candidate',
            )
        return dict(
            corpus_success=True,
            qualified=False,
            must_pass=True,
            in_denominator=True,
            reason='must_pass gates observed on corpus bytes; execution is still not qualification',
        )
    return dict(
        corpus_success=False,
        qualified=False,
        in_denominator=True,
        reason='execution is not qualification',
    )


def b1_geometric_ablation(*, supports_driver=False, invoked=False):
    """Plan --driver geometric. Do not invoke until the example argv parser contains --driver."""
    planned = ['--driver', 'geometric']
    executed = bool(invoked and supports_driver)
    slot = dict(
        id='B1',
        title='existing geometric QEM',
        purpose='Establish value beyond a cheap current route.',
        relevance='required',
        execution_status='executed' if executed else 'not_executed',
        compared=False,
        qualified=False,
        error=None,
        cost=None,
        timing=None,
        improvement=None,
        gpu=False,
        planned_argv=planned,
        driver_flag_in_example_source=bool(supports_driver),
        production_exe_invoked=executed,
        note=(
            'B1 planned argv is --driver geometric on mesh corpus JSON. '
            + (
                'Example source parses --driver; this campaign still does not mark B1 qualified.'
                if supports_driver else
                'Example source does not parse --driver; B1 stays not_executed until the coordinator rebuilds. The flag was not passed to the exe.'
            )
            + ' Unimplemented ablations are not_executed, not zero.'
        ),
    )
    if executed:
        slot['timing'] = dict(kind='cpu_wall', gpu=False, synthetic=False)
        slot['note'] = (
            'Admitted example invoked with --driver geometric. Executed is not qualified; '
            'no error/cost zeros invented.'
        )
    return slot


def ablation_slots(*, b2_executed=False, b0_record=None, b1_record=None):
    """B0-B5 typed records. Unimplemented slots are not_executed, never a zero measurement."""
    notes = {
        'B0': 'Identity corpus candidates are publication-checked; no production no-reduction path was compared.',
        'B1': 'Geometric QEM --driver geometric is planned; not invoked unless the example parses --driver.',
        'B2': (
            'Admitted production_core_geometry.exe invoked on its builtin seam/analytic fixture '
            if b2_executed else
            'Admitted semantic example was not invoked for B2.'
        ),
        'B3': 'Remeshing/anisotropic route was not invoked.',
        'B4': 'Forge schedule ablation was not invoked; GPU timing is not synthesized.',
        'B5': 'Physical/operator model ablation was not invoked.',
    }
    slots = []
    for item in ABLATION_DEFINITIONS:
        if item['id'] == 'B0' and b0_record is not None:
            slot = dict(b0_record)
            slot.setdefault('title', item['title'])
            slot.setdefault('purpose', item['purpose'])
            slot.setdefault('relevance', item['relevance'])
            slot['qualified'] = False
            slot['gpu'] = False
            if slot.get('error') in (0, 0.0):
                slot['error'] = None
            if slot.get('cost') in (0, 0.0):
                slot['cost'] = None
            slots.append(slot)
            continue
        if item['id'] == 'B1':
            slot = dict(b1_record) if b1_record is not None else b1_geometric_ablation()
            slot.setdefault('title', item['title'])
            slot.setdefault('purpose', item['purpose'])
            slot.setdefault('relevance', item['relevance'])
            slot.setdefault('planned_argv', ['--driver', 'geometric'])
            slot['qualified'] = False
            slot['gpu'] = False
            if slot.get('error') in (0, 0.0):
                slot['error'] = None
            if slot.get('cost') in (0, 0.0):
                slot['cost'] = None
            if slot.get('execution_status') != 'executed':
                slot['execution_status'] = 'not_executed'
            slots.append(slot)
            continue
        executed = item['id'] == 'B2' and b2_executed
        slot = dict(
            id=item['id'],
            title=item['title'],
            purpose=item['purpose'],
            relevance=item['relevance'],
            execution_status='executed' if executed else 'not_executed',
            compared=False,
            qualified=False,
            error=None,
            cost=None,
            timing=None,
            improvement=None,
            gpu=False,
            note=notes[item['id']] + ' Unimplemented ablations are not_executed, not zero.',
        )
        if executed:
            slot['timing'] = dict(kind='cpu_wall', gpu=False, synthetic=False)
            slot['note'] = notes[item['id']] + ' Executed is not qualified; no error/cost zeros invented.'
        slots.append(slot)
    return slots


def denominator_summary(records):
    counts = dict(
        selected=len(records),
        in_denominator=sum(1 for item in records if item.get('in_denominator')),
        executed=sum(1 for item in records if item.get('execution_status') == 'executed'),
        not_executed=sum(1 for item in records if item.get('execution_status') == 'not_executed'),
        descriptor_only=sum(1 for item in records if item.get('execution_status') == 'descriptor_only'),
        failed_execution=sum(1 for item in records if item.get('execution_status') == 'execution_failed'),
        holdout=sum(1 for item in records if item.get('holdout')),
        must_fail=sum(1 for item in records if item.get('must_fail')),
        exclusions=sum(1 for item in records if item.get('exclusions')),
    )
    return dict(
        **counts,
        failures_and_exclusions_in_denominator=True,
        holdout_in_denominator=True,
        holdout_used_for_threshold_selection=False,
        note='Every selected fixture including failures, exclusions, holdout and descriptor-only stays in the denominator.',
    )


def load_manifest(corpus_dir):
    corpus_dir = Path(corpus_dir)
    path = corpus_dir / 'manifest.json'
    raw = path.read_bytes()
    manifest = json.loads(raw.decode('utf-8'))
    if manifest.get('schema') != 'production-core.qualification-corpus/v1':
        raise ValueError('unexpected corpus manifest schema')
    return manifest


def selected_entries(manifest, families):
    entries = list(manifest.get('fixtures') or [])
    if families:
        entries = [entry for entry in entries if entry['family'] in families]
    return entries


def verify_and_load_fixture(corpus_dir, entry, algo, primary_fn):
    path = Path(corpus_dir) / entry['file']
    raw = path.read_bytes()
    digest = digest_bytes(raw, algo, primary_fn)
    if digest['digest'] != entry.get('digest'):
        raise ValueError(f"{entry['file']}: digest mismatch")
    if digest['sha256'] != entry.get('sha256'):
        raise ValueError(f"{entry['file']}: sha256 mismatch")
    case = json.loads(raw.decode('utf-8'))
    if case.get('id') != entry['id'] or case.get('family') != entry['family']:
        raise ValueError(f"{entry['file']}: identity mismatch")
    return case, digest


def publication_check(case):
    try:
        result = evaluate_fixture(case)
        return dict(status='consistent', result=result)
    except OracleError as error:
        return dict(status='inconsistent', error=str(error))


def label_exe_timings(timings_ms):
    if not timings_ms:
        return None
    return cpu_wall_timing(
        wall_seconds=None,
        source='geometry_exe_instant_wall',
        samples_ms=timings_ms,
    )


def interpret_invocation(fixture, plan, invocation, exe_out):
    status = invocation.get('status')
    exit_code = invocation.get('exit_code')
    executed = status == 'executed'
    exe_passed = executed and exit_code == 0
    expected_negative = fixture['id'] in NEGATIVE_INVOCATIONS
    execution_status = 'not_executed'
    if status == 'missing_executable':
        execution_status = 'not_executed'
    elif status == 'timeout':
        execution_status = 'execution_failed'
    elif executed:
        execution_status = 'executed'
    receipt_path = Path(exe_out) / 'geometry-receipt.json'
    geometry_receipt = None
    corpus_consumed = bool(plan.get('consumes_corpus_bytes'))
    if receipt_path.is_file():
        payload = json.loads(receipt_path.read_text(encoding='utf-8'))
        fixture_meta = payload.get('fixture') or {}
        corpus_consumed = bool(fixture_meta.get('consumes_corpus_json')) or corpus_consumed
        geometry_receipt = dict(
            path=str(receipt_path),
            status=payload.get('status'),
            builtin_fixture=fixture_meta.get('name') if fixture_meta.get('source') == 'builtin' else None,
            corpus_fixture=fixture_meta.get('name') if fixture_meta.get('consumes_corpus_json') else None,
            corpus_fixture_consumed=bool(fixture_meta.get('consumes_corpus_json')),
            gpu_stage=False,
            timings=label_exe_timings(payload.get('timings_ms')),
        )
        if payload.get('status') == 'passed':
            exe_passed = executed and (exit_code == 0 or expected_negative is False)
    observed_failed_gates = []
    if expected_negative and executed and exit_code not in (None, 0):
        expected = (fixture.get('oracle') or {}).get('expected_status')
        receipt_status = (geometry_receipt or {}).get('status')
        if expected and receipt_status == expected:
            observed_failed_gates.append(expected)
        elif expected == 'invalid_input' and (receipt_status == 'invalid_input' or exit_code == 2):
            observed_failed_gates.append('invalid_input')
        elif expected == 'unsatisfied_constraint' and receipt_status == 'unsatisfied_constraint':
            observed_failed_gates.append('unsatisfied_constraint')
    verdict = fixture_verdict(
        fixture,
        execution_status=execution_status,
        exe_passed=exe_passed,
        observed_failed_gates=observed_failed_gates,
        consumed_corpus_bytes=corpus_consumed,
    )
    timing = cpu_wall_timing(invocation.get('wall_seconds'), source='python_perf_counter') if executed else absent_timing()
    return dict(
        execution_status=execution_status,
        exe_passed=exe_passed,
        expected_negative=expected_negative,
        geometry_receipt=geometry_receipt,
        observed_failed_gates=observed_failed_gates,
        verdict=verdict,
        timing=timing,
        consumes_corpus_bytes=corpus_consumed,
    )


def fixture_campaign_record(
    entry, case, digest, plan, publication, interpretation=None, invocation=None, exe_out=None,
    oracle_observation=None,
):
    oracle = case.get('oracle') or {}
    record = dict(
        id=case['id'],
        family=case['family'],
        partition=case['partition'],
        file=entry['file'],
        seed=case.get('seed', entry.get('seed')),
        hashes=dict(
            hash_algorithm=digest.get('hash_algorithm') or entry.get('hash_algorithm'),
            digest=digest.get('digest') or entry.get('digest'),
            sha256=digest.get('sha256') or entry.get('sha256'),
            blake3=digest.get('blake3') or entry.get('blake3'),
        ),
        holdout=plan['holdout'],
        must_fail=plan['must_fail'],
        must_pass=plan['must_pass'],
        in_denominator=True,
        used_for_threshold_selection=False,
        exclusions=list(case.get('exclusions') or []),
        execution_status=plan['execution_status'] if plan['execution_status'] != 'pending' else 'not_executed',
        execution_action=plan['action'],
        execution_reason=plan['reason'],
        consumes_corpus_bytes=bool(plan.get('consumes_corpus_bytes')),
        publication_check=publication['status'],
        timing=absent_timing(),
        qualified=False,
        corpus_success=False,
        invented_passing_geometry_receipt=False,
        gpu_timing_claimed=False,
    )
    if plan['holdout']:
        record['holdout_policy'] = oracle.get('holdout_policy') or 'must_not_be_used_for_tuning_or_threshold_search'
    if case['family'] == 'maximum_size_descriptor':
        record['materialized'] = False
        record['vertex_count'] = None
        record['triangle_count'] = None
        record['maximum_size'] = 'descriptor_only'
    if publication['status'] != 'consistent':
        record['publication_error'] = publication.get('error')
    if invocation is not None:
        record['geometry_invocation'] = dict(
            argv=invocation.get('argv'),
            status=invocation.get('status'),
            exit_code=invocation.get('exit_code'),
            out=str(exe_out) if exe_out is not None else None,
            extra_args=plan.get('extra_args'),
            builtin_fixture=BUILTIN_GEOMETRY_FIXTURE,
            note=invocation.get('note'),
        )
    if interpretation is not None:
        record['execution_status'] = interpretation['execution_status']
        record['timing'] = interpretation['timing']
        record['geometry_receipt'] = interpretation.get('geometry_receipt')
        record['exe_passed'] = interpretation.get('exe_passed')
        record['corpus_success'] = interpretation['verdict'].get('corpus_success', False)
        record['verdict'] = interpretation['verdict']
        record['observed_failed_gates'] = interpretation.get('observed_failed_gates')
        record['consumes_corpus_bytes'] = interpretation.get('consumes_corpus_bytes', record['consumes_corpus_bytes'])
    elif plan['must_fail']:
        observed = list((oracle_observation or {}).get('observed') or ())
        unevaluated = list((oracle_observation or {}).get('unevaluated') or ())
        record['verdict'] = known_bad_verdict(
            case,
            execution_status=record['execution_status'],
            exe_passed=False,
            observed_failed_gates=observed,
            unevaluated_gates=unevaluated,
        )
        record['corpus_success'] = record['verdict'].get('corpus_success', False)
        record['observed_failed_gates'] = observed
        record['unevaluated_gates'] = unevaluated
        record['oracle_gate_evaluation'] = dict(
            status='executed' if oracle_observation is not None else 'not_executed',
            route='oracle_harness.observe_known_bad_gates',
            production_exe_invoked=False,
            gpu=False,
            observed=observed,
            unevaluated=unevaluated,
            publication_defect=(oracle_observation or {}).get('publication_defect'),
            note=(oracle_observation or {}).get('note') or (
                'Published candidate evaluated by oracle_harness. Geometry exe was not invoked.'
            ),
        )
    return record


def campaign_status(inspect_only, records):
    if inspect_only:
        return 'inspected_not_executed'
    if any(item.get('execution_status') == 'executed' for item in records):
        return 'executed_partial_not_qualified'
    return 'recorded_not_executed'


def run_campaign(
    corpus,
    out,
    geometry_exe=None,
    inspect_only=False,
    families=None,
    worktree_root=None,
    invoke_geometry=None,
    driver=None,
):
    """Run an inspect or invoke campaign into a create-only directory."""
    invoke_geometry = invoke_geometry or invoke_admitted_geometry
    out = create_only_dir(Path(out).resolve())
    corpus = Path(corpus)
    family_filter = parse_family_filters(families)

    inspection = inspect_admitted_geometry(
        worktrees=discover_worktrees(worktree_root),
        geometry_exe=geometry_exe,
    )
    driver_support = geometry_example_driver_support(inspection)
    supports_driver = bool(driver_support.get('driver_flag_in_example_source'))
    driver_for_invoke = driver if (driver and supports_driver) else None
    if geometry_exe is not None:
        geometry_exe = Path(geometry_exe)
    else:
        geometry_exe = Path(inspection['capabilities']['admitted_geometry_example']['executable'])
    exe_present = geometry_exe.is_file()
    manifest = load_manifest(corpus)
    algo, primary_fn, algo_note = hash_backend()
    entries = selected_entries(manifest, family_filter)

    receipt = dict(
        schema=SCHEMA,
        status='running',
        started_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
        mode='inspect' if inspect_only else 'campaign',
        corpus=str(corpus.resolve()),
        out=str(out),
        family_filter=family_filter,
        geometry_exe=dict(
            path=str(geometry_exe),
            present=exe_present,
            sha256=sha(geometry_exe) if exe_present else None,
            consumes_corpus_json=True,
            builtin_fixture=BUILTIN_GEOMETRY_FIXTURE,
            supported_fixture_ids=sorted(SEAM_ANALYTIC_FIXTURE_IDS | set(NEGATIVE_INVOCATIONS)),
            driver_flag_in_example_source=supports_driver,
            requested_driver=driver,
            driver_for_invoke=driver_for_invoke,
            b1_planned_argv=['--driver', 'geometric'],
        ),
        inspection=inspection,
        limitations=[
            'Qualification runner, not full production-core acceptance.',
            'Mesh fixtures are passed as --corpus-json. The builtin two-chart grid is used only when no corpus JSON is supplied.',
            'Known-bad published candidates are evaluated by oracle_harness; the geometry exe is not invoked on welded candidates as input meshes.',
            'Known-bad passing executable output is a corpus defect unless named candidate gates failed.',
            'B1 --driver geometric is planned when the example argv parser contains --driver; absence keeps B1 not_executed.',
            'Holdout fixtures are labelled and never used to pick thresholds.',
            'Maximum-size remains descriptor_only; the 50k mesh is not materialized.',
            'Unimplemented B0-B5 ablations are not_executed, not zero.',
            'B0 identity ablation is inspect/CPU only; GPU is not invoked; executed is not qualified.',
            'Independent QuantizedMesh decode of produced simplified.10d does not trust geometry-receipt.json.',
            'Collapse provenance snapshot absence is not_supplied, not 0.0.',
            'CPU wall times are labelled cpu_wall. Synthetic values are never stored as GPU.',
            DETERMINISM_BOUNDARY,
        ],
        invented_passing_geometry_receipt=False,
        gpu_identical_timing_claimed=False,
        maximum_size_materialized=False,
        holdout_used_for_threshold_selection=False,
        visual_capture='recipe_only_not_executed',
        visual_capture_recipe=manifest.get('visual_capture_recipe'),
        hash_algorithm=algo,
        hash_algorithm_note=algo_note,
        fixtures=[],
        ablations=[],
        collapse_provenance=inspect_collapse_provenance(search_dir=out),
        whole_surface_bound=whole_surface_bound_from_existing_receipts(inspection['worktrees']['synth']),
        independent_decode_summary=dict(executed=0, mismatches=0, missing=0, failed=0),
        denominator=None,
        threshold_selection=dict(
            performed=False,
            used_holdout=False,
            note='This campaign does not search thresholds. Holdout is excluded from any future pick.',
        ),
    )

    def save():
        (out / 'qualification-receipt.json').write_text(
            json.dumps(receipt, indent=2, allow_nan=False) + '\n',
            encoding='utf-8',
        )

    save()
    records = []
    b2_executed = False
    identity_cases = []
    try:
        for entry in entries:
            case, digest = verify_and_load_fixture(corpus, entry, algo, primary_fn)
            if case.get('family') == 'identity':
                identity_cases.append(case)
            plan = geometry_exe_plan(
                case,
                inspect_only=inspect_only,
                geometry_exe_present=exe_present,
                corpus=corpus,
                fixture_file=entry.get('file'),
                driver=driver_for_invoke,
                supports_driver=supports_driver,
            )
            publication = publication_check(case)
            oracle_observation = None
            if case.get('family') == 'known_bad' or (case.get('oracle') or {}).get('must_fail') is True:
                oracle_observation = observe_known_bad_gates(case.get('data') or {}, case.get('oracle') or {})
            interpretation = None
            invocation = None
            exe_out = None
            if plan['action'] == 'invoke':
                exe_out = out / f"geometry-{case['id']}"
                started = time.perf_counter()
                try:
                    invocation = invoke_geometry(geometry_exe, exe_out, extra_args=plan['extra_args'])
                except Exception as error:
                    invocation = dict(
                        label='admitted-geometry',
                        argv=[str(geometry_exe), '--out', str(exe_out), *(plan['extra_args'] or [])],
                        status='execution_failed',
                        exit_code=None,
                        wall_seconds=time.perf_counter() - started,
                        stdout='',
                        stderr=str(error),
                        note=str(error),
                    )
                stdout_text = invocation.get('stdout')
                stderr_text = invocation.get('stderr')
                if stdout_text is not None:
                    write_create_only_text(out / f"geometry-{case['id']}.stdout.txt", stdout_text)
                if stderr_text is not None:
                    write_create_only_text(out / f"geometry-{case['id']}.stderr.txt", stderr_text)
                interpretation = interpret_invocation(case, plan, invocation, exe_out)
                if (
                    interpretation['execution_status'] == 'executed'
                    and case['id'] in SEAM_ANALYTIC_FIXTURE_IDS
                ):
                    b2_executed = True
            record = fixture_campaign_record(
                entry, case, digest, plan, publication,
                interpretation=interpretation, invocation=invocation, exe_out=exe_out,
                oracle_observation=oracle_observation,
            )
            if exe_out is not None:
                decode = inspect_sealed_10d_dir(exe_out)
                record['independent_decode'] = compact_decode_record(decode)
                record['collapse_provenance'] = inspect_collapse_provenance(search_dir=exe_out)
            else:
                record['independent_decode'] = dict(
                    status='not_executed',
                    qualified=False,
                    trusted_geometry_receipt=False,
                    gpu=False,
                    note='Geometry executable was not invoked for this fixture; independent decode did not run.',
                )
                record['collapse_provenance'] = inspect_collapse_provenance(search_dir=None)
            records.append(record)
            receipt['fixtures'] = records
            save()
        b1_invoked = b1_invoked_from_records(records)
        receipt['ablations'] = ablation_slots(
            b2_executed=b2_executed,
            b0_record=b0_identity_ablation(identity_cases),
            b1_record=b1_geometric_ablation(supports_driver=supports_driver, invoked=b1_invoked),
        )
        receipt['collapse_provenance'] = campaign_provenance_summary(records, search_dir=out)
        receipt['whole_surface_bound'] = whole_surface_bound_from_existing_receipts(
            inspection['worktrees']['synth'],
            extra_paths=campaign_geometry_receipt_paths(out),
        )
        receipt['driver_support'] = driver_support
        if b1_invoked:
            receipt['driver_support'] = dict(driver_support)
            receipt['driver_support']['executed'] = True
            receipt['driver_support']['note'] = (
                'B1 --driver geometric was appended to mesh corpus JSON and at least one fixture executed. '
                'Executed is not qualified.'
            )
        decode_statuses = [item.get('independent_decode', {}).get('status') for item in records]
        receipt['independent_decode_summary'] = dict(
            executed=sum(status == 'executed' for status in decode_statuses),
            mismatches=sum(
                1 for item in records
                if ((item.get('independent_decode') or {}).get('cross_check') or {}).get('mismatch')
            ),
            missing=sum(status in ('missing_file', 'not_supplied', 'not_executed') for status in decode_statuses),
            failed=sum(status == 'decode_failed' for status in decode_statuses),
            qualified=False,
            trusted_geometry_receipt=False,
        )
        receipt['denominator'] = denominator_summary(records)
        receipt['status'] = campaign_status(inspect_only, records)
        receipt['finished_at'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        receipt['b2_semantic_example_executed'] = b2_executed
        save()
    except Exception as error:
        receipt['status'] = 'failed'
        receipt['reason'] = str(error)
        receipt['fixtures'] = records
        receipt['ablations'] = ablation_slots(
            b2_executed=b2_executed,
            b0_record=b0_identity_ablation(identity_cases),
            b1_record=b1_geometric_ablation(
                supports_driver=supports_driver,
                invoked=b1_invoked_from_records(records),
            ),
        )
        receipt['driver_support'] = driver_support
        receipt['denominator'] = denominator_summary(records)
        save()
        raise
    return receipt


def main(argv=None):
    parser = argparse.ArgumentParser(description='P22 corpus qualification campaign runner')
    parser.add_argument('--corpus', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--geometry-exe', type=Path)
    parser.add_argument('--inspect-only', action='store_true', default=False)
    parser.add_argument('--families', action='append', default=None, help='Optional family filter; repeat or comma-separate')
    parser.add_argument('--worktree-root', type=Path)
    parser.add_argument(
        '--driver',
        choices=['admitted', 'geometric'],
        default=None,
        help='Extra argv for mesh fixtures when the geometry example supports --driver. Geometric is B1; ignored until the example parses --driver.',
    )
    args = parser.parse_args(argv)
    receipt = run_campaign(
        corpus=args.corpus,
        out=args.out,
        geometry_exe=args.geometry_exe,
        inspect_only=args.inspect_only,
        families=args.families,
        worktree_root=args.worktree_root,
        driver=args.driver,
    )
    summary = dict(
        status=receipt['status'],
        out=receipt['out'],
        fixtures=len(receipt['fixtures']),
        invented_passing_geometry_receipt=False,
        gpu_identical_timing_claimed=False,
        maximum_size_materialized=False,
        holdout_used_for_threshold_selection=False,
    )
    print(json.dumps(summary, indent=2))
    return receipt


if __name__ == '__main__':
    main()
