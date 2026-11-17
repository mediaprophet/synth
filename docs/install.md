# Install Synth

Tagged builds live at <https://github.com/mediaprophet/synth/releases>.

Each release ships:

- `synth-windows-x86_64.exe`
- `synth-macos-aarch64`
- `synth-linux-x86_64`

plus `.sig` (Ed25519 over SHA-256) and `.sha256` sidecars.

## First install

Download the binary for your OS. On macOS and Linux:

```bash
chmod +x synth-macos-aarch64   # or synth-linux-x86_64
./synth-linux-x86_64 operator
```

On Windows, run `synth-windows-x86_64.exe operator`.

`synth operator` opens the browser workbench at `/operator` and, when built with `native-ui`, the Aether desktop tray.

## Verify and apply a later tag

```text
synth update check
synth update verify --artifact synth-windows-x86_64.exe
synth update apply --artifact synth-windows-x86_64.exe
```

Apply copies to a sibling path. It never overwrites a locked `synth.exe` (the live MCP stdio child). Point MCP at the new sidecar, then `/mcps` reload.

Signatures use QualiaDB Ed25519 `sign_with_context` over the SHA-256 of the file (`synth.release.sha256.v1`). The public key is compiled into the binary (`.synth/update-public.hex`). The private seed is the GitHub Actions secret `SYNTH_UPDATE_SEED` and local `.synth/update-seed.hex` (gitignored).
