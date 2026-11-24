#!/usr/bin/env python3
"""
Webizen AI Synth — Autonomous Agent Bootstrap & Live-Rename Updater (`agent-update-synth.py`)
Copyright (c) 2026 Timothy Charles Holborn
https://www.linkedin.com/in/ubiquitous/ | timothy.holborn@gmail.com
License: Attribution-NonCommercial-NoDerivatives 4.0 International (CC BY-NC-ND 4.0)

Allows any AI agent (Antigravity, Codex, Grok, or Google Colab worker) to:
1. Check `https://api.github.com/repos/mediaprophet/synth/releases/latest` (or a specific `--tag`).
2. Download the platform portable binary (`synth-windows-x86_64.exe`, `synth-macos-aarch64`,
   or `synth-linux-x86_64`), its `.sha256` sidecar, and its Ed25519 `.sig` sidecar.
3. Verify the exact SHA-256 digest (and invoke `synth update verify` for Ed25519
   `synth.release.sha256.v1` verification against the embedded public key).
4. Atomically live-rename-swap both `synth` (`synth.exe`) and `synth-mcp` (`synth-mcp.exe`)
   in the target directory (`--dest`, defaulting to `C:/github/synth/target-prod/debug` on Windows
   or `./bin` on Linux/macOS):
   - On Windows, a running `.exe` (such as a connected `synth-mcp.exe` stdio server) cannot be
     overwritten in-place, but *can* be renamed while running (`synth-mcp.exe` -> `synth-mcp.prev.exe`),
     allowing the newly verified binary to be placed immediately at `synth-mcp.exe` and `synth.exe`.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shutil
import stat
import subprocess
import sys
import urllib.request
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

GITHUB_LATEST_URL = "https://api.github.com/repos/mediaprophet/synth/releases/latest"
GITHUB_TAG_URL_FMT = "https://api.github.com/repos/mediaprophet/synth/releases/tags/{tag}"
USER_AGENT = "Webizen-AI-Synth-Agent-Updater/1.0 (+https://github.com/mediaprophet/synth)"
EMBEDDED_PUBLIC_KEY_HEX = "54eb7716cf658c55b0245849502735f06299e378bb2b2f555794158b9fd21965"


def detect_platform_portable() -> str:
    sys_name = platform.system().lower()
    mach = platform.machine().lower()
    if sys_name == "windows":
        return "synth-windows-x86_64.exe"
    if sys_name == "darwin":
        if "arm" in mach or "aarch64" in mach:
            return "synth-macos-aarch64"
        return "synth-macos-x86_64"
    if sys_name == "linux":
        return "synth-linux-x86_64"
    raise RuntimeError(f"Unsupported platform: {sys_name} / {mach}")


def default_dest_dir() -> Path:
    env_dest = os.environ.get("SYNTH_INSTALL_DIR", "").strip()
    if env_dest:
        return Path(env_dest)
    win_default = Path(r"C:\github\synth\target-prod\debug")
    if platform.system().lower() == "windows" and win_default.parent.exists():
        return win_default
    repo_root = Path(__file__).resolve().parent.parent
    target_prod = repo_root / "target-prod" / "debug"
    if target_prod.exists():
        return target_prod
    return repo_root / "bin"


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        while True:
            chunk = f.read(1024 * 1024)
            if not chunk:
                break
            h.update(chunk)
    return h.hexdigest()


def fetch_json(url: str) -> Dict[str, Any]:
    req = urllib.request.Request(
        url,
        headers={
            "User-Agent": USER_AGENT,
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
        },
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read().decode("utf-8"))


def download_file(url: str, dest: Path) -> int:
    dest.parent.mkdir(parents=True, exist_ok=True)
    tmp = dest.with_name(dest.name + ".part")
    req = urllib.request.Request(
        url,
        headers={
            "User-Agent": USER_AGENT,
            "Accept": "application/octet-stream",
        },
    )
    with urllib.request.urlopen(req, timeout=300) as resp, tmp.open("wb") as out_f:
        shutil.copyfileobj(resp, out_f, length=256 * 1024)
    if platform.system().lower() != "windows":
        tmp.chmod(tmp.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    os.replace(tmp, dest)
    return dest.stat().st_size


def find_asset(assets: List[Dict[str, Any]], name: str) -> Optional[Tuple[str, int]]:
    for a in assets:
        if a.get("name") == name:
            url = a.get("browser_download_url")
            size = int(a.get("size", 0))
            if url:
                return str(url), size
    return None


def live_rename_swap(src_verified: Path, target: Path) -> Dict[str, Any]:
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.is_file():
        try:
            if sha256_file(target).lower() == sha256_file(src_verified).lower():
                return {
                    "target": str(target),
                    "status": "already_current",
                }
        except Exception:
            pass

    # 1. Try direct replacement first (succeeds when binary is not currently locked)
    try:
        shutil.copy2(src_verified, target)
        if platform.system().lower() != "windows":
            target.chmod(target.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
        return {
            "target": str(target),
            "status": "replaced_direct",
        }
    except PermissionError:
        pass

    # 2. Windows PE image lock work-around: rename running `<stem>.exe` -> `<stem>.prev.exe`, then copy new binary to `<stem>.exe`
    prev_path = target.with_name(f"{target.stem}.prev{target.suffix}")
    try:
        if prev_path.exists():
            prev_path.unlink()
    except PermissionError:
        # If .prev is also locked by an older process, use a PID/timestamped .prev
        prev_path = target.with_name(f"{target.stem}.prev-{os.getpid()}{target.suffix}")

    try:
        os.replace(target, prev_path)
        shutil.copy2(src_verified, target)
        if platform.system().lower() != "windows":
            target.chmod(target.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
        return {
            "target": str(target),
            "moved_running_to": str(prev_path),
            "status": "live_rename_swapped",
        }
    except Exception as exc:
        # If copy failed after rename, restore previous binary
        if not target.exists() and prev_path.exists():
            try:
                os.replace(prev_path, target)
            except Exception:
                pass
        return {
            "target": str(target),
            "status": "staged_only",
            "error": str(exc),
        }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Download, verify (SHA-256 + Ed25519), and live-rename-swap Webizen AI Synth binaries for AI agents."
    )
    parser.add_argument(
        "--action",
        choices=["check", "upgrade"],
        default="upgrade",
        help="check latest release or download+verify+swap (default: upgrade)",
    )
    parser.add_argument(
        "--tag",
        default=None,
        help="Optional release tag (e.g. v0.0.41.20); defaults to latest GitHub release",
    )
    parser.add_argument(
        "--dest",
        default=None,
        help="Target directory containing synth / synth-mcp binaries",
    )
    args = parser.parse_args()

    api_url = GITHUB_TAG_URL_FMT.format(tag=args.tag) if args.tag else GITHUB_LATEST_URL
    release = fetch_json(api_url)
    tag_name = str(release.get("tag_name", "unknown"))
    assets = release.get("assets", [])
    portable_name = detect_platform_portable()
    sig_name = f"{portable_name}.sig"
    sha_name = f"{portable_name}.sha256"

    portable_info = find_asset(assets, portable_name)
    sig_info = find_asset(assets, sig_name)
    sha_info = find_asset(assets, sha_name)
    if not portable_info or not sig_info:
        raise RuntimeError(f"Release {tag_name} is missing {portable_name} or {sig_name}")

    dest_dir = Path(args.dest).resolve() if args.dest else default_dest_dir().resolve()
    is_win = platform.system().lower() == "windows"
    ext = ".exe" if is_win else ""
    synth_target = dest_dir / f"synth{ext}"
    synth_mcp_target = dest_dir / f"synth-mcp{ext}"

    if args.action == "check":
        current_sha = sha256_file(synth_target) if synth_target.is_file() else None
        mcp_sha = sha256_file(synth_mcp_target) if synth_mcp_target.is_file() else None
        print(
            json.dumps(
                {
                    "schema": "synth.agent-updater/v1",
                    "action": "check",
                    "tag": tag_name,
                    "portable": portable_name,
                    "portable_url": portable_info[0],
                    "portable_bytes": portable_info[1],
                    "sig_url": sig_info[0],
                    "sha256_url": sha_info[0] if sha_info else None,
                    "dest_dir": str(dest_dir),
                    "current_synth_sha256": current_sha,
                    "current_synth_mcp_sha256": mcp_sha,
                },
                indent=2,
            )
        )
        return 0

    cache_dir = dest_dir / "updates"
    cache_dir.mkdir(parents=True, exist_ok=True)
    artifact_path = cache_dir / portable_name
    sig_path = cache_dir / sig_name
    sha_path = cache_dir / sha_name

    dl_bytes = download_file(portable_info[0], artifact_path)
    download_file(sig_info[0], sig_path)
    expected_sha: Optional[str] = None
    if sha_info:
        download_file(sha_info[0], sha_path)
        raw_sha = sha_path.read_text(encoding="utf-8").strip().split()
        if raw_sha:
            expected_sha = raw_sha[0].lower()

    actual_sha = sha256_file(artifact_path).lower()
    if expected_sha and actual_sha != expected_sha:
        raise RuntimeError(
            f"SHA-256 verification failed for {artifact_path}: expected {expected_sha}, got {actual_sha}"
        )

    # Verify Ed25519 signature (`synth.release.sha256.v1`) using the newly downloaded binary itself
    verify_cmd = [
        str(artifact_path),
        "update",
        "verify",
        "--artifact",
        str(artifact_path),
        "--sig",
        str(sig_path),
    ]
    proc = subprocess.run(verify_cmd, capture_output=True, text=True, check=False)
    if proc.returncode != 0:
        raise RuntimeError(
            f"Ed25519 signature verification failed (exit {proc.returncode}): {proc.stderr or proc.stdout}"
        )
    verify_receipt = json.loads(proc.stdout)

    # Stage `synth.new.exe` and perform live-rename swap onto `synth` and `synth-mcp`
    new_sibling = dest_dir / f"synth.new{ext}"
    shutil.copy2(artifact_path, new_sibling)

    swap_results = [
        live_rename_swap(artifact_path, synth_target),
        live_rename_swap(artifact_path, synth_mcp_target),
    ]

    result = {
        "schema": "synth.agent-updater/v1",
        "ok": True,
        "tag": tag_name,
        "sha256": actual_sha,
        "bytes": dl_bytes,
        "ed25519_verified": verify_receipt.get("ok", False),
        "public_key_hex": verify_receipt.get("public_key_hex", EMBEDDED_PUBLIC_KEY_HEX),
        "staged_new": str(new_sibling),
        "swapped_targets": swap_results,
        "next_step": "Reload MCP (/mcps reload) if an MCP session was already open so it spawns the updated synth-mcp binary.",
    }
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
