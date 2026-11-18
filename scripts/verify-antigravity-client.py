"""Verify the real Antigravity config model and local MCP SDK transport.

No Agent.chat(), model provider, cloud request, tunnel, or IDE automation is used.
A transport success is deliberately distinguished from Antigravity harness/IDE acceptance.
Run with --inspect-only while the isolated Synth build is in progress; after that build
passes, reuse --fixture-dir from scripts/verify-local-clients.py for shared-input parity.
"""
from __future__ import annotations
import argparse, asyncio, datetime, hashlib, importlib.metadata, importlib.util
import json, os, shutil, sys, traceback, uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SDK = ROOT / "vendor/antigravity-sdk-python"

def digest(path: Path) -> str:
    with path.open("rb") as file:
        return hashlib.file_digest(file, "sha256").hexdigest()

def module_state(name: str) -> dict:
    try:
        spec = importlib.util.find_spec(name)
        return {"available": spec is not None, "origin": spec.origin if spec else None}
    except (ImportError, ValueError) as error:
        return {"available": False, "error": type(error).__name__}

def structured(result) -> dict:
    value = getattr(result, "structured_content", None)
    if value is None:
        value = getattr(result, "structuredContent", None)
    if isinstance(value, dict):
        return value
    for block in getattr(result, "content", ()):
        if getattr(block, "type", None) == "text":
            try:
                value = json.loads(block.text)
                if isinstance(value, dict):
                    return value
            except (ValueError, TypeError):
                pass
    raise RuntimeError("MCP result has no structured object")

def result_error(result) -> bool:
    return bool(result.is_error if hasattr(result, "is_error") else result.isError)

def session_timeout(seconds: float):
    # MCP1 accepts timedelta; MCP2's public constructor takes float seconds.
    major = int(importlib.metadata.version("mcp").split(".")[0])
    return float(seconds) if major >= 2 else datetime.timedelta(seconds=seconds)

def progress(run: Path, step: str, **details) -> None:
    event = {"at": datetime.datetime.now(datetime.timezone.utc).isoformat(), "step": step, **details}
    with (run / "transport-progress.jsonl").open("a", encoding="utf-8") as file:
        file.write(json.dumps(event) + "\n")

async def bounded(run: Path, step: str, awaitable, seconds: float = 30):
    progress(run, step + "_started", timeout_seconds=seconds)
    try:
        result = await asyncio.wait_for(awaitable, timeout=seconds)
    except BaseException as error:
        progress(run, step + "_failed", error_type=type(error).__name__, message=str(error)[:1024])
        raise
    progress(run, step + "_completed")
    return result

async def roundtrip(config, fixture: Path, run: Path) -> dict:
    # Actual installed client, configured by the real public Antigravity type.
    from mcp import ClientSession, StdioServerParameters
    from mcp.client.stdio import stdio_client
    bindings = [{"key": name, "kind": kind, "path": str(fixture / name)}
                for name, kind in (("fixture.glb", "glb"), ("checker.png", "png"))]
    for item in bindings:
        if not Path(item["path"]).is_file():
            raise RuntimeError("Shared retained fixture is incomplete")
    source = (fixture / "fixture.vibe").read_text(encoding="utf-8")
    params = StdioServerParameters(command=config.command, args=config.args,
        env={**os.environ, **(config.env or {})})
    outcome = {"transport": "installed mcp.ClientSession/stdio_client using real Antigravity McpStdioServer fields",
        "mcp_version": importlib.metadata.version("mcp")}
    progress(run, "stdio_connecting")
    with (run / "server-stderr.txt").open("a", encoding="utf-8") as errlog:
        async with stdio_client(params, errlog=errlog) as streams:
            progress(run, "stdio_connected")
            async with ClientSession(streams[0], streams[1], read_timeout_seconds=session_timeout(180)) as session:
                initialized = await bounded(run, "initialize", session.initialize())
                tools = await bounded(run, "list_tools", session.list_tools())
                names = {tool.name for tool in tools.tools}
                required = {"synth_find", "synth_operations", "synth_operation_contracts", "synth_produce_present"}
                if not required <= names:
                    raise RuntimeError("Synth production tools are absent from the actual stdio catalog")
                operations = await bounded(run, "operations", session.call_tool("synth_operations", {}))
                if result_error(operations) or structured(operations).get("count") != 150:
                    raise RuntimeError("Actual production catalog does not contain 150 operations")
                produced = await bounded(run, "produce", session.call_tool("synth_produce_present", {
                    "source": source, "inputs": bindings, "out_hmc": "antigravity.hmc",
                    "export_dir": "exported", "receipt_json": "production-receipt.json"}), 180)
                production = structured(produced)
                if result_error(produced):
                    raise RuntimeError("SDK-configured MCP production failed: " + json.dumps(production))
                wrong_host = await bounded(run, "wrong_host", session.call_tool("vibe_invoke", {
                    "id": "hypermedia.locus.construct", "args": {"shape": "circle", "radius": 1, "segments": 16}}))
                if not result_error(wrong_host) or structured(wrong_host).get("error", {}).get("code") != "wrong_execution_host":
                    raise RuntimeError("Wrong-host invocation did not return the real MCP execution error")
                unknown = await bounded(run, "unknown_key", session.call_tool("synth_produce_present", {"source": source, "unknown_key": True}))
                if not result_error(unknown):
                    raise RuntimeError("Unknown argument was not refused by the actual client roundtrip")
                outcome.update({"initialize": initialized.model_dump(mode="json", by_alias=True), "tool_count": len(names),
                    "catalog_count": 150, "production": production,
                    "wrong_host_is_error": result_error(wrong_host), "unknown_key_is_error": result_error(unknown)})
    hmc = run / "antigravity.hmc"
    baseline = fixture / "cli.hmc"
    if not hmc.is_file() or not baseline.is_file():
        raise RuntimeError("CLI/MCP parity requires both durable HMC outputs")
    outcome["cli_hmc_sha256"] = digest(baseline)
    outcome["sdk_configured_mcp_hmc_sha256"] = digest(hmc)
    outcome["identical_hmc"] = outcome["cli_hmc_sha256"] == outcome["sdk_configured_mcp_hmc_sha256"]
    if not outcome["identical_hmc"]:
        raise RuntimeError("Actual SDK-configured MCP output differs from the shared CLI fixture")
    progress(run, "fresh_stdio_connecting")
    with (run / "server-stderr.txt").open("a", encoding="utf-8") as errlog:
        async with stdio_client(params, errlog=errlog) as streams:
            async with ClientSession(streams[0], streams[1], read_timeout_seconds=session_timeout(30)) as session:
                await bounded(run, "fresh_initialize", session.initialize())
                result = await bounded(run, "fresh_operations", session.call_tool("synth_operations", {}))
                outcome["fresh_client_reconnect"] = not result_error(result) and structured(result)["count"] == 150
    receipt = json.loads((run / "production-receipt.json").read_text(encoding="utf-8"))
    if receipt["hmc"]["sha256"] != digest(hmc):
        raise RuntimeError("Durable production receipt does not match the saved package")
    outcome["durable_receipt_verified"] = True
    progress(run, "transport_verified")
    return outcome

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--synth", type=Path, default=ROOT / "target-prod/debug/synth-mcp.exe")
    parser.add_argument("--fixture-dir", type=Path)
    parser.add_argument("--inspect-only", action="store_true")
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()
    run = args.out or ROOT / "production" / ("antigravity-" + datetime.datetime.now().strftime("%Y%m%d-%H%M%S") + "-" + uuid.uuid4().hex[:8])
    run.mkdir(parents=True, exist_ok=False)
    report = {"schema": "synth.antigravity-local-verification/v1", "python": sys.executable,
        "python_version": sys.version, "sdk_source": None, "adapter_source": str(ROOT / "vendor/antigravity/synth_local.py"), "provider_calls": 0,
        "antigravity_config_constructed": False, "sdk_configured_mcp_roundtrip": False,
        "antigravity_agent_harness_roundtrip": False, "antigravity_ide_frontend_acceptance": False,
        "scope": "actual SDK public configuration type plus installed MCP client transport; no Agent/model/IDE claim"}
    modules = ["pydantic", "mcp", "google.genai", "google.protobuf", "absl", "uvicorn", "websockets"]
    report["dependencies"] = {name: module_state(name) for name in modules}
    report["harness_on_path"] = shutil.which("localharness")
    report["vendored_harness_exists"] = any((SDK / "google/antigravity/bin" / name).is_file() for name in ("localharness", "localharness.exe"))
    try:
        os.environ["SYNTH_BIN"] = str(args.synth.resolve())
        os.environ["SYNTH_OUTPUT_ROOT"] = str(run.resolve())
        os.environ["SYNTH_INPUT_ROOT"] = str((args.fixture_dir or run).resolve())
        spec = importlib.util.spec_from_file_location("synth_local_adapter", ROOT / "vendor/antigravity/synth_local.py")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        config = module.synth_server()
        from google.antigravity.types import McpStdioServer
        import google.antigravity as actual_sdk
        sdk_path = Path(actual_sdk.__file__).resolve()
        if sdk_path.is_relative_to(SDK.resolve()):
            raise RuntimeError("Verification requires the installed SDK; vendored source shadowed it")
        report["sdk_source"] = str(sdk_path)
        report["sdk_distribution"] = {"name": "google-antigravity", "version": importlib.metadata.version("google-antigravity")}
        report["installed_harness_paths"] = [str(path) for name in ("localharness", "localharness.exe")
            if (path := sdk_path.parent / "bin" / name).is_file()]
        report["source_sha256"] = {str(path): digest(path) for path in
            (Path(__file__).resolve(), ROOT / "vendor/antigravity/synth_local.py", sdk_path)}
        if not isinstance(config, McpStdioServer):
            raise RuntimeError("Adapter did not return the actual public SDK configuration type")
        report["antigravity_config_constructed"] = True
        report["configuration"] = config.model_dump(mode="json", exclude_none=True)
        report["status"] = "configuration_verified"
        if not args.inspect_only:
            if args.fixture_dir is None:
                raise RuntimeError("Actual transport test needs --fixture-dir from verify-local-clients.py")
            if not args.synth.is_file():
                raise RuntimeError("Wait for the isolated Synth build to pass before transport testing")
            report["synth_sha256"] = digest(args.synth)
            progress(run, "asyncio_starting")
            report["roundtrip"] = asyncio.run(asyncio.wait_for(roundtrip(config, args.fixture_dir.resolve(), run.resolve()), timeout=300))
            report["sdk_configured_mcp_roundtrip"] = True
            report["status"] = "sdk_configuration_and_mcp_transport_verified"
        result = 0
    except Exception as error:
        report["status"] = "unavailable_or_failed"
        report["error"] = {"type": type(error).__name__, "message": str(error)[:4096]}
        (run / "diagnostic.txt").write_text(traceback.format_exc(), encoding="utf-8")
        result = 2
    (run / "verification.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps({"status": report["status"], "receipt": str(run / "verification.json"),
        "config_constructed": report["antigravity_config_constructed"],
        "mcp_transport_verified": report["sdk_configured_mcp_roundtrip"],
        "antigravity_ide_verified": False}, indent=2))
    return result

if __name__ == "__main__":
    raise SystemExit(main())
