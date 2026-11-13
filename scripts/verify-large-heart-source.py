"""Verify a >64 KiB authored heart through CLI and real stdio MCP.

The required frozen Synth binary is supplied by the caller. This produces
create-only evidence; PNGs still require human inspection. No provider is used.
"""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import uuid

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'examples/quality/fixtures/anatomy-source-budget.vibe'
EXPECTED_SHA = '208d8934507f2816aa98784486af137d1e90b2195eda36af2353ce57c08df0ea'
parser = argparse.ArgumentParser()
parser.add_argument('--synth', required=True, type=Path)
args = parser.parse_args()
synth = args.synth.resolve(strict=True)
source = FIXTURE.read_bytes()
assert len(source) > 65536 and hashlib.sha256(source).hexdigest() == EXPECTED_SHA
source.decode('utf-8')
run = ROOT / 'production' / ('source-budget-' + datetime.datetime.now().strftime('%Y%m%d-%H%M%S') + '-' + uuid.uuid4().hex[:8])
run.mkdir(parents=True, exist_ok=False)
(run / 'heart.vibe').write_bytes(source)
env = dict(os.environ, SYNTH_INPUT_ROOT=str(run), SYNTH_OUTPUT_ROOT=str(run))
records = []
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def execute(name, argv, input_text=None):
    try:
        result = subprocess.run([str(synth), *map(str, argv)], cwd=ROOT, env=env,
                                input=input_text, capture_output=True, text=True,
                                encoding='utf-8', errors='replace', timeout=600)
    except subprocess.TimeoutExpired as exc:
        raise RuntimeError(f'{name} exceeded 600 seconds') from exc
    (run / (name + '.log')).write_text(result.stdout + '\n' + result.stderr, encoding='utf-8')
    records.append({'name':name, 'returncode':result.returncode, 'log':str(run / (name + '.log'))})
    if result.returncode != 0:
        raise RuntimeError(f'{name} failed: {result.stderr[-2000:]}')
    return result
try:
    for suffix in ('first', 'repeat'):
        execute('cli-' + suffix, ['produce', '--source', run / 'heart.vibe', '--out', run / (suffix + '.hmc')])
    execute('cli-reopen', ['produce', '--reopen', run / 'first.hmc', '--out', run / 'reopen.hmc'])
    messages = [
        {'jsonrpc':'2.0','id':'init','method':'initialize','params':{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'source-budget-review','version':'1'}}},
        {'jsonrpc':'2.0','method':'notifications/initialized'},
        {'jsonrpc':'2.0','id':'produce','method':'tools/call','params':{'name':'synth_produce_present','arguments':{'source_path':'heart.vibe','out_hmc':'mcp.hmc','export_dir':'mcp-export','receipt_json':'mcp-receipt.json','preview_assets':['anatomy.10d']}}},
    ]
    result = execute('mcp', ['mcp'], ''.join(json.dumps(m, separators=(',',':')) + '\n' for m in messages))
    responses = [json.loads(line) for line in result.stdout.splitlines() if line.startswith('{')]
    response = next((x for x in responses if x.get('id') == 'produce'), None)
    if not response or response.get('error') or response.get('result', {}).get('isError'):
        raise RuntimeError('MCP production refused: ' + str(response))
    receipt = json.loads((run / 'mcp-export/.synth/review-completion.json').read_text(encoding='utf-8'))
    limits = receipt['limits']
    assert receipt['source_sha256'] == EXPECTED_SHA and receipt['strict_replay_verified'] is True
    assert limits['source_bytes'] == len(source) and limits['source_available_bytes'] >= len(source)
    assert limits['source_reservation_bytes'] == 128 * len(source)
    assert limits['source_work_units'] == 4 * len(source)
    assert limits['effective_host_bytes'] == limits['total_resource_bytes'] - limits['source_reservation_bytes']
    assert limits['combined_logical_budget_bytes'] == limits['total_resource_bytes'] + limits['evaluation_workspace_bytes']
    assert limits['effective_asset_work_units'] == limits['asset_work_units'] - limits['source_work_units']
    assert receipt['review_status'] == 'rendered_pending_inspection'
    assert (run / 'mcp-export/.synth/source.vibe').read_bytes() == source
    hashes = {name:digest(run / (name + '.hmc')) for name in ('first','repeat','reopen','mcp')}
    assert len(set(hashes.values())) == 1 and hashes['mcp'] == receipt['hmc']['sha256']
    review = run / 'mcp-export-review-0'
    for view in ('front-view-assumed.png','side-view-assumed.png','three-quarter-view-assumed.png'):
        assert (review / view).is_file(), f'missing native view {view}'
    outcome = {'passed':True, 'fixture_sha256':EXPECTED_SHA, 'source_bytes':len(source),
               'hmc_sha256':hashes['mcp'], 'receipt':str(run / 'mcp-export/.synth/review-completion.json'),
               'native_views':str(review), 'visual_acceptance':'pending inspection', 'commands':records}
except Exception as exc:
    outcome = {'passed':False, 'error':str(exc), 'visual_acceptance':'not assessed', 'commands':records}
(run / 'results.json').write_text(json.dumps(outcome, indent=2), encoding='utf-8')
print(json.dumps(outcome, indent=2))
raise SystemExit(0 if outcome['passed'] else 1)
