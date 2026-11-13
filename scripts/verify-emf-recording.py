"""Produce an EMF recording and check original samples and exact timing.

The Vibe calls the production verifier. This independent check reads the sealed
extension to compare payload bytes; it does not substitute for codec integrity
tests, browser consumption, or visual/listening review.
"""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import uuid
import wave

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--synth", type=Path, required=True)
    args = parser.parse_args()
    run = ROOT / "production" / ("emf-recording-" + datetime.datetime.now().strftime("%Y%m%d-%H%M%S") + "-" + uuid.uuid4().hex[:8])
    run.mkdir(parents=True)
    pressure = struct.pack("<4f", 0.0, -0.0, 2.5, -3.75)
    (run / "pressure.f32le").write_bytes(pressure)
    source = ROOT / "examples/quality/emf-recording.vibe"
    base = [str(args.synth), "produce", "--source", str(source), "--input", "pressure", "pcm_f32le", str(run / "pressure.f32le")]
    for label in ("first", "replay"):
        command = base + ["--out", str(run / (label + ".hmc"))]
        if label == "first":
            command += ["--export-dir", str(run / "export")]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=300)
        (run / (label + ".log")).write_text(result.stdout + "\n" + result.stderr, encoding="utf-8")
        if result.returncode:
            raise RuntimeError(f"{label} failed; inspect {run / (label + '.log')}")
    first = (run / "first.hmc").read_bytes()
    assert first == (run / "replay.hmc").read_bytes(), "Replay package differs"
    emf_path, = (run / "export").rglob("recording.emf")
    data = emf_path.read_bytes()
    footer = data[-64:]
    assert footer[:8] == b"Q42EMFX3"
    field_end, json_bytes, pcm_bytes, payload_bytes = struct.unpack_from("<4Q", footer, 8)
    assert field_end + payload_bytes + 128 == len(data)
    metadata = json.loads(data[field_end:field_end + json_bytes])
    timebase = metadata["field"]["timebase"]
    assert timebase == {"epoch": {"scale": "relative", "seconds": 0, "nanoseconds": 0}, "ticks_per_second_num": 48000, "ticks_per_second_den": 1, "slice_start_tick": 0, "slice_step_ticks": 480}
    assert metadata["field"]["colour"] == {"model": "spectral_only"}
    tracks = {track["spec"]["id"]: track for track in metadata["audio_tracks"]}
    assert len(tracks) == 2
    recorded = tracks["pressure"]
    assert recorded["frames"] == 4 and recorded["pcm_bytes"] == len(pressure)
    assert recorded["spec"]["sample_rate_num"] == 1000000
    assert recorded["spec"]["start_tick"] == -24
    assert recorded["spec"]["units"] == {"unit": "pascal"}
    assert recorded["spec"]["units_per_sample"] == 0.1
    pcm_start = field_end + ((json_bytes + 63) & ~63)
    assert pcm_start + pcm_bytes == len(data) - 128
    start = pcm_start + recorded["pcm_offset"]
    assert data[start:start + len(pressure)] == pressure, "Original calibrated samples or signed zero changed"
    tone = tracks["reference-tone"]
    assert tone["frames"] == 24000 and tone["spec"]["sample_rate_num"] == 48000
    wav_path, = (run / "export").rglob("reference-tone.wav")
    with wave.open(str(wav_path), "rb") as wav:
        assert wav.getnframes() == 24000 and wav.getframerate() == 48000
        assert wav.getnchannels() == 1
    result = {"passed": True, "source": str(source), "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
              "synth": str(args.synth.resolve()), "synth_sha256": hashlib.sha256(args.synth.read_bytes()).hexdigest(),
              "package_sha256": hashlib.sha256(first).hexdigest(), "deterministic_replay": True,
              "emf": str(emf_path), "original_pressure_bytes_preserved": True,
              "exact_timebase": timebase, "metadata": metadata,
              "visual_and_listening_review": "pending", "browser_consumption": "pending"}
    (run / "results.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(run, flush=True)


if __name__ == "__main__":
    main()
