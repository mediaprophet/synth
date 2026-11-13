"""Stage a browser check using a built Portal package and a verified EMF export.

Serve the new output directory through the local studio. The page exercises the
actual WASM reader and preserves test results in visible text for browser review.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

PAGE = r'''<!doctype html><meta charset="utf-8"><title>EMF recording review</title>
<style>body{font:17px system-ui;background:#10151d;color:#e7edf5;max-width:1000px;margin:40px auto;padding:24px}img,canvas{display:block;max-width:100%;width:512px;height:auto;border:1px solid #58677b}pre{white-space:pre-wrap;overflow-wrap:anywhere;background:#1b2432;padding:20px}</style>
<h1>EMF recording review</h1><p id="status">Checking the actual Portal reader and GPU field draw…</p>
<h2>Actual WebGPU field slice</h2><p>The existing phase/sigma shader is a diagnostic projection of the retained field (z=0, t=0, amplitude gain 0.02). This is not calibrated RGB or a complete spectral image.</p>
<canvas id="canvas" width="512" height="512"></canvas>
<h2>Captured field pixels</h2><p>This canvas copies the actual WebGPU draw, so the capture remains visible after browser presentation.</p>
<canvas id="proof" width="512" height="512"></canvas>
<h2>Independent native wavelength preview</h2>
<p>The native image uses a different declared display conversion. Original electromagnetic cells and calibrated recordings remain in the EMF.</p>
<img src="recording.png" alt="Visible derivative of the infrared field"><p>Independent reference tone:</p><audio controls src="reference-tone.wav"></audio>
<pre id="report"></pre>
<script type="module">
import init, { QualiaPortal, portal_init_webgpu, portal_abort_webgpu } from './pkg/qualia.js';
const expected = __EXPECTED__;
const checks = [];
function check(name, condition) { checks.push({name, passed:!!condition}); if(!condition) throw new Error(name); }
function hex(bytes) { return Array.from(bytes,x=>x.toString(16).padStart(2,'0')).join(''); }
function exactJson(text) {
  return JSON.parse(text.replace(/"(?:\\.|[^"\\])*"|-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?/g,
    token => token[0] === '"' ? token : JSON.stringify({numberToken:token})));
}
function canonical(value) {
  if (Array.isArray(value)) return '['+value.map(canonical).join(',')+']';
  if (value && typeof value === 'object') return '{'+Object.keys(value).sort().map(key=>JSON.stringify(key)+':'+canonical(value[key])).join(',')+'}';
  return JSON.stringify(value);
}
async function captureField(portal, canvas) {
  await new Promise(resolve => requestAnimationFrame(resolve));
  const submission = portal.render_emf_preview(0.02);
  check('Existing GPU field pipeline submitted', submission.submitted === true && submission.backend === 'webgpu' && submission.conversion === 'legacy-phase-sigma-diagnostic');
  check('Submission is separate from readback', submission.pixel_readback === 'not-performed');
  check('Declared diagnostic gain and slice', Math.abs(submission.amplitude_gain-0.02)<1e-6 && submission.z_slice===0 && submission.time_slice===0);
  check('Bounded surface dimensions', submission.width === canvas.width && submission.height === canvas.height && canvas.width <= 1024 && canvas.height <= 1024);
  const proof = document.querySelector('#proof');
  proof.width = canvas.width; proof.height = canvas.height;
  const context = proof.getContext('2d', {willReadFrequently:true});
  // No normal Portal tick is interposed: this copies the field draw just submitted.
  context.drawImage(canvas,0,0);
  const pixels = context.getImageData(0,0,proof.width,proof.height).data;
  let lit=0, lo=255, hi=0;
  const colours = new Set();
  for(let i=0;i<pixels.length;i+=4) {
    const brightness=Math.max(pixels[i],pixels[i+1],pixels[i+2]);
    if(brightness>3 && pixels[i+3]>0) lit++;
    lo=Math.min(lo,brightness); hi=Math.max(hi,brightness);
    if(colours.size<256) colours.add((pixels[i]<<16)|(pixels[i+1]<<8)|pixels[i+2]);
  }
  check('Actual field readback has visible content', lit > proof.width*proof.height/100);
  check('Actual field readback has spatial variation', colours.size>4 && hi-lo>8);
  const sha256=hex(new Uint8Array(await crypto.subtle.digest('SHA-256',pixels)));
  return {submission,width:proof.width,height:proof.height,sha256,lit_pixels:lit,distinct_colours_up_to_256:colours.size,brightness_range:[lo,hi]};
}
const result = {status:'running',checks,scope:'Actual WASM metadata/PCM and WebGPU phase/sigma slice with browser canvas readback; independent visual acceptance and calibrated RGB remain separate'};
try {
  await init();
  // Exercise early loading on a separate CPU canvas. The visible WebGPU
  // canvas stays context-free until its awaited initialization below.
  const bootstrap = document.createElement('canvas'); bootstrap.width=32; bootstrap.height=32;
  const portal = new QualiaPortal(bootstrap);
  window.emfReviewPortal = portal;
  check('Current lifecycle and field draw exports', typeof portal.emf_display_state === 'function' && typeof portal.render_emf_preview === 'function');
  const response = await fetch('./recording.emf');
  check('EMF fixture delivered',response.ok);
  const bytes = new Uint8Array(await response.arrayBuffer());
  const inspected = portal.inspect_emf(bytes);
  check('Actual v3 reader export', inspected.container_version === 3);
  check('Full field and extension integrity', inspected.integrity === 'full-blake3-chunks-and-crc32c' && inspected.extension_integrity === 'full-blake3');
  const metadata = JSON.parse(inspected.metadata_json);
  check('Explicit physical field interpretation', metadata.field.colour.model === 'spectral_only');
  check('Exact declared timebase without numeric coercion', canonical(exactJson(inspected.metadata_json).field.timebase) === canonical(exactJson(expected.timebase_json)));
  check('Two independent original tracks', inspected.audio_tracks.length === 2);
  const pressure = inspected.audio_tracks.find(x=>x.id==='pressure');
  check('Original pressure bytes including signed zero', !!pressure && hex(pressure.pcm_f32_le) === expected.pressure_hex);
  const early = portal.load_emf(bytes);
  check('Early load retains field pending WebGPU', early.display_status === 'pending' && portal.emf_display_state() === 1);
  portal_abort_webgpu();
  portal.tick(bootstrap,0);
  check('Explicit initialization abort reports unavailable without fallback', portal.emf_display_state() === 3);
  const canvas = document.querySelector('#canvas');
  check('WebGPU initialization awaited on context-free canvas', await portal_init_webgpu(canvas));
  portal.tick(canvas,0);
  check('GPU adoption replays retained early field', portal.emf_display_state() === 2);
  result.early_field_pixels = await captureField(portal,canvas);
  const loaded = portal.load_emf(bytes);
  check('Ready GPU accepts field immediately', loaded.display_status === 'uploaded' && portal.emf_display_state() === 2);
  check('Current field loaded with interpretation', loaded.container_version === 3 && loaded.cell_count === 512 && loaded.nt === 1);
  check('No implied field-derived audio for recordings', loaded.frequency_hz === null && loaded.sonification === 'no automatic EMF sonification; stored tracks are independent');
  check('Load retains exact metadata and original tracks', loaded.metadata_json === inspected.metadata_json && hex(loaded.audio_tracks.find(x=>x.id==='pressure').pcm_f32_le) === expected.pressure_hex);
  result.ready_field_pixels = await captureField(portal,canvas);
  check('Early replay and ready load draw the same field', result.early_field_pixels.sha256 === result.ready_field_pixels.sha256);
  const damaged = bytes.slice(); damaged[damaged.length - 129] ^= 1;
  let refused = false;
  try { portal.inspect_emf(damaged); } catch { refused = true; }
  check('Corrupted extension refused by actual WASM', refused);
  refused=false;
  try { portal.load_emf(damaged); } catch { refused=true; }
  check('Corrupted replacement refused without losing accepted residency', refused && portal.emf_display_state() === 2);
  result.after_refusal_pixels = await captureField(portal,canvas);
  check('Corrupted replacement preserves actual field pixels', result.after_refusal_pixels.sha256 === result.ready_field_pixels.sha256);
  result.metadata_json = inspected.metadata_json;
  result.status = 'passed';
} catch(error) { result.status='failed'; result.error=String(error); }
window.emfReview = result;
document.querySelector('#status').textContent = result.status === 'passed' ? 'Field readback and recording checks passed. The captured slice still requires visual review.' : 'Portal field/recording checks failed.';
document.querySelector('#report').textContent = JSON.stringify(result,null,2);
</script>'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', type=Path, required=True)
    parser.add_argument('--recording', type=Path, required=True, help='Directory produced by verify-emf-recording.py')
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    report = json.loads((args.recording / 'results.json').read_text(encoding='utf-8'))
    if report.get('passed') is not True:
        raise ValueError('Native recording checks must pass first')
    for name in ('qualia.js', 'qualia_bg.wasm', 'qualia.d.ts', 'qualia_bg.wasm.d.ts'):
        if not (args.package / name).is_file():
            raise ValueError('Incomplete generated Portal package: ' + name)
    args.out.mkdir(parents=True, exist_ok=False)
    shutil.copytree(args.package, args.out / 'pkg')
    for name in ('recording.emf', 'recording.png', 'reference-tone.wav'):
        path, = (args.recording / 'export').rglob(name)
        shutil.copyfile(path, args.out / name)
    expected = {'timebase_json': json.dumps(report['exact_timebase'], separators=(',', ':')), 'pressure_hex': (args.recording / 'pressure.f32le').read_bytes().hex()}
    (args.out / 'index.html').write_text(PAGE.replace('__EXPECTED__', json.dumps(expected)), encoding='utf-8')
    receipts = {'recording_receipt': str((args.recording / 'results.json').resolve()), 'browser_acceptance': 'pending',
                'package_sha256': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in (args.out / 'pkg').iterdir() if p.is_file()}}
    (args.out / 'staging.json').write_text(json.dumps(receipts, indent=2), encoding='utf-8')
    print(args.out.resolve())


if __name__ == '__main__':
    main()
