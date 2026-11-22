"""Verify registered Synth through the installed Codex app-server.

No model turn or new chat is created. A real tool call requires --thread;
--proxy selects the existing desktop daemon. Fixture parity is optional and
uses explicit source/input/output roots already configured for that server.
Rendered previews remain pending human inspection.
"""
import argparse
import hashlib
import json
import queue
import re
import subprocess
import threading
import time
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT.parent / "hypermedia-production/crates/asset-production-vibe/src/operations_catalog.rs"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def expected_operations(path):
    source = path.read_text(encoding="utf-8").split("\n];", 1)[0]
    ids = re.findall(r'\(\s*"((?:hypermedia|qualia)\.[a-z0-9_.]+)"', source)
    ontology_path = path.parent / "ontology"
    ontology = (ontology_path / "mod.rs").read_text(encoding="utf-8")
    expansion = ontology.split("const EXPANSION:", 1)[1].split("const fn", 1)[0]
    batches = dict.fromkeys(re.findall(r'\b(morphisms_(?:\d+|swarm))::BATCH\b', expansion))
    for batch in batches:
        descriptors = (ontology_path / f"{batch}.rs").read_text(encoding="utf-8")
        ids.extend(re.findall(r'^\s*id: "((?:hypermedia|qualia)\.[a-z0-9_.]+)"', descriptors, re.M))
    if not ids or len(ids) != len(set(ids)):
        raise ValueError("local authoritative catalogue is empty or contains duplicate IDs")
    return set(ids)


def tool_payload(reply):
    if not isinstance(reply, dict) or reply.get("isError"):
        raise RuntimeError("Codex tool call returned an error: " + json.dumps(reply))
    structured = reply.get("structuredContent")
    if isinstance(structured, dict):
        return structured
    if any(k in reply for k in ("operations", "receipt", "status")):
        return reply
    for block in reply.get("content", []):
        if isinstance(block, dict) and block.get("type") == "text":
            try:
                parsed = json.loads(block.get("text", ""))
            except (TypeError, json.JSONDecodeError):
                continue
            if isinstance(parsed, dict):
                return parsed
    raise RuntimeError("Codex tool call did not return structured JSON")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--codex", required=True)
    parser.add_argument("--out", type=Path, required=True, help="new evidence directory")
    parser.add_argument("--server", default="synth-local")
    parser.add_argument("--thread")
    parser.add_argument("--proxy", action="store_true")
    parser.add_argument("--reload", action="store_true")
    parser.add_argument("--isolated-discovery", action="store_true", help="Discover registered tools using a new local state database; no thread calls")
    parser.add_argument("--catalog", type=Path, default=CATALOG)
    parser.add_argument("--fixture-source", type=Path)
    parser.add_argument("--input-manifest", type=Path, help="JSON array of {key,kind,path} bindings")
    parser.add_argument("--synth", type=Path, help="verified binary for independent CLI production")
    parser.add_argument("--output-root", type=Path, help="registered SYNTH_OUTPUT_ROOT")
    parser.add_argument("--preview-key", help="exported .10d key for native preview")
    args = parser.parse_args()
    fixture_flags = [args.fixture_source, args.input_manifest, args.synth, args.output_root, args.preview_key]
    if any(fixture_flags) and (not all(fixture_flags) or not args.thread):
        parser.error("fixture parity requires --thread, --fixture-source, --input-manifest, --synth, --output-root and --preview-key")
    if args.preview_key and not args.preview_key.endswith(".10d"):
        parser.error("--preview-key must identify an exported .10d")
    if args.isolated_discovery and (args.thread or args.proxy or args.reload):
        parser.error("isolated discovery cannot operate on a desktop thread, proxy, or reload")
    expected = expected_operations(args.catalog)
    source_bytes = args.fixture_source.read_bytes() if args.fixture_source else None
    source = source_bytes.decode("utf-8") if source_bytes is not None else None
    bindings = json.loads(args.input_manifest.read_text(encoding="utf-8")) if args.input_manifest else None
    if bindings is not None:
        if not isinstance(bindings, list) or any(not isinstance(b, dict) or set(b) != {"key", "kind", "path"} or not all(isinstance(b[k], str) for k in b) for b in bindings):
            parser.error("input manifest must be an array of {key,kind,path} string records")
        if len({b["key"] for b in bindings}) != len(bindings):
            parser.error("input manifest keys must be unique")
        if any(not Path(b["path"]).is_absolute() for b in bindings):
            parser.error("input manifest paths must be absolute so CLI and registered server read the same files")
        args.fixture_source = args.fixture_source.resolve(strict=True)
        args.synth = args.synth.resolve(strict=True)
    args.out.mkdir(parents=True, exist_ok=False)
    command = [args.codex, "app-server"] + (["proxy"] if args.proxy else ["--stdio"])
    if args.isolated_discovery:
        state_dir = args.out.resolve() / "state"
        state_dir.mkdir()
        command += ["-c", "sqlite_home=" + json.dumps(str(state_dir))]
    replies = queue.Queue()
    with (args.out / "client-stderr.log").open("w", encoding="utf-8") as errors:
        process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=errors, text=True, encoding="utf-8", bufsize=1)

        def receive():
            for line in process.stdout:
                try:
                    replies.put(json.loads(line))
                except json.JSONDecodeError:
                    replies.put({"protocol_error": "Non-JSON stdout from Codex"})
            replies.put({"protocol_error": "Codex stdout closed"})

        threading.Thread(target=receive, daemon=True).start()
        request_id = 0

        def request(method, params):
            nonlocal request_id
            request_id += 1
            process.stdin.write(json.dumps({"id": request_id, "method": method, "params": params}) + "\n")
            process.stdin.flush()
            deadline = time.monotonic() + 600
            while True:
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise TimeoutError(f"Codex did not answer {method}")
                try:
                    reply = replies.get(timeout=remaining)
                except queue.Empty as e:
                    raise TimeoutError(f"Codex did not answer {method}") from e
                if "protocol_error" in reply:
                    raise RuntimeError(reply["protocol_error"])
                if reply.get("id") != request_id:
                    continue
                if "error" in reply:
                    raise RuntimeError(f"{method}: {reply['error']}")
                return reply.get("result")

        def tool(name, arguments):
            return request("mcpServer/tool/call", {"server": args.server, "threadId": args.thread,
                                                   "tool": name, "arguments": arguments})

        try:
            request("initialize", {"clientInfo": {"name": "synth-local-review", "version": "1.0"},
                                   "capabilities": {"experimentalApi": True}})
            process.stdin.write(json.dumps({"method": "initialized"}) + "\n")
            process.stdin.flush()
            if args.reload:
                request("config/mcpServer/reload", None)
            params = {"serverName": args.server, "limit": 100}
            if args.thread:
                params["threadId"] = args.thread
            inventory = request("mcpServerStatus/list", params)
            (args.out / "inventory.json").write_text(json.dumps(inventory, indent=2), encoding="utf-8")
            target = next((s for s in inventory.get("data", []) if s.get("name") == args.server), None)
            if target is None:
                raise RuntimeError("Registered Synth server is absent from the Codex inventory")
            tools = target.get("tools", {})
            names = list(tools) if isinstance(tools, dict) else [t.get("name") for t in tools]
            if "synth_find" not in names or "synth_operations" not in names:
                raise RuntimeError("Codex did not discover synth_find / synth_operations")
            result = {"server": args.server, "tool_count": len(names), "tool_names": names,
                      "desktop_proxy": args.proxy, "existing_chat": args.thread,
                      "catalog_verified": False, "fixture_verified": False,
                      "visual_acceptance": "not assessed"}
            if args.thread:
                reply = tool("synth_operations", {"limit": len(expected), "detail": "ids"})
                (args.out / "operations.json").write_text(json.dumps(reply, indent=2), encoding="utf-8")
                catalog = tool_payload(reply)
                operations = catalog.get("operations")
                if catalog.get("count") != len(expected):
                    raise RuntimeError("Codex catalog count differs from the checkout")
                ids = [item.get("id") if isinstance(item, dict) else item
                       for item in operations] if isinstance(operations, list) else []
                if not ids:
                    ids = list(catalog.get("ids") or [])
                if len(ids) != len(expected) or set(ids) != expected:
                    raise RuntimeError("Codex catalog IDs differ from the checkout")
                result["catalog_verified"] = True
                result["catalog_count"] = len(ids)
            if source is not None:
                if "synth_produce_present" not in names:
                    raise RuntimeError("Codex did not discover synth_produce_present")
                output_root = args.output_root.resolve(strict=True)
                prefix = "codex-parity-" + uuid.uuid4().hex[:12]
                first_name, repeat_name, export_name = prefix + ".hmc", prefix + "-repeat.hmc", prefix + "-export"
                receipt_name = prefix + "-receipt.json"
                base = {"source": source, "inputs": bindings}
                refusal = None
                try:
                    refusal = tool("synth_produce_present", dict(base, typo=True))
                    detail = json.dumps(refusal).lower()
                    failed = isinstance(refusal, dict) and bool(refusal.get("isError"))
                except RuntimeError as e:
                    detail, failed = str(e).lower(), True
                if not failed or ("invalid_arguments" not in detail and "unknown field" not in detail):
                    raise RuntimeError("Codex did not report the expected invalid-argument refusal: " + detail[:1000])
                (args.out / "refusal.json").write_text(json.dumps({"reply": refusal, "detail": detail}, indent=2), encoding="utf-8")
                first = tool_payload(tool("synth_produce_present", dict(base, out_hmc=first_name, export_dir=export_name,
                                                                         receipt_json=receipt_name, preview_assets=[args.preview_key])))
                if first.get("status") == "error":
                    raise RuntimeError("Codex production/preview failed: " + json.dumps(first))
                repeat = tool_payload(tool("synth_produce_present", dict(base, out_hmc=repeat_name)))
                if repeat.get("status") == "error":
                    raise RuntimeError("Codex repeat production failed: " + json.dumps(repeat))
                receipt = first.get("receipt")
                if not isinstance(receipt, dict) or receipt.get("source_sha256") != sha(source_bytes) or receipt.get("catalog", {}).get("count") != len(expected):
                    raise RuntimeError("Codex receipt does not match source bytes/catalog")
                if receipt.get("binary", {}).get("sha256", "").lower() != sha(args.synth.read_bytes()):
                    raise RuntimeError("registered server binary does not match requested CLI binary")
                if receipt.get("strict_replay_verified") is not True or receipt.get("review_status") != "rendered_pending_inspection":
                    raise RuntimeError("Codex receipt lacks strict replay or native preview completion")
                receipt_inputs = {(v.get("key"), v.get("kind"), v.get("sha256")) for v in receipt.get("inputs", [])}
                expected_inputs = {(v["key"], v["kind"], sha(Path(v["path"]).read_bytes())) for v in bindings}
                if receipt_inputs != expected_inputs:
                    raise RuntimeError("Codex receipt input digests do not match fixture")
                cli_hmc = args.out / "cli.hmc"
                cli = [str(args.synth), "produce", "--source", str(args.fixture_source), "--out", str(cli_hmc)]
                for binding in bindings:
                    cli += ["--input", binding["key"], binding["kind"], binding["path"]]
                cli_result = subprocess.run(cli, cwd=ROOT, capture_output=True, timeout=600)
                (args.out / "cli-stdout.log").write_bytes(cli_result.stdout)
                (args.out / "cli-stderr.log").write_bytes(cli_result.stderr)
                if cli_result.returncode != 0:
                    raise RuntimeError("independent Synth CLI fixture production failed")
                hmc = output_root / first_name
                repeat_hmc = output_root / repeat_name
                digest = sha(hmc.read_bytes())
                if digest != sha(repeat_hmc.read_bytes()) or digest != sha(cli_hmc.read_bytes()) or digest != receipt.get("hmc", {}).get("sha256"):
                    raise RuntimeError("Codex, repeat and CLI HMC digests differ")
                exported = output_root / export_name / Path(args.preview_key)
                if not exported.is_file():
                    raise RuntimeError("requested .10d was not exported")
                reviews = receipt.get("native_reviews", [])
                review = next((v for v in reviews if v.get("resource") == args.preview_key), None)
                if not review:
                    raise RuntimeError("receipt omits requested native review")
                review_path = Path(review["path"]).resolve(strict=True)
                if not review_path.is_relative_to(output_root):
                    raise RuntimeError("review path lies outside configured output root")
                for name in ("front-view-assumed.png", "side-view-assumed.png", "three-quarter-view-assumed.png"):
                    if not (review_path / name).read_bytes().startswith(b"\x89PNG\r\n\x1a\n"):
                        raise RuntimeError(f"missing or invalid native view {name}")
                review_receipt = json.loads((review_path / "review-receipt.json").read_text(encoding="utf-8"))
                (args.out / "production.json").write_text(json.dumps(first, indent=2), encoding="utf-8")
                (args.out / "repeat.json").write_text(json.dumps(repeat, indent=2), encoding="utf-8")
                result.update(fixture_verified=True, source_sha256=sha(source_bytes), hmc_sha256=digest,
                              cli_hmc=str(cli_hmc), codex_hmc=str(hmc), export_dir=str(output_root / export_name),
                              preview_dir=str(review_path), review_receipt=review_receipt,
                              refusal_verified=True, visual_acceptance="pending inspection")
            (args.out / "result.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
            print(json.dumps(result))
        except Exception as error:
            (args.out / "failure.json").write_text(json.dumps({"type": type(error).__name__, "message": str(error)}, indent=2), encoding="utf-8")
            raise
        finally:
            if process.poll() is None:
                process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)


if __name__ == "__main__":
    main()
