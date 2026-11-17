# Install Webizen AI Synth

Tagged builds live at <https://github.com/mediaprophet/synth/releases>.

Prefer the signed installer for first install:

- Windows: `WebizenAI-Synth-Setup-windows-x86_64.exe` (Start Menu + desktop shortcut, user-local, no admin)
- macOS Apple Silicon: `WebizenAI-Synth-macos-aarch64.dmg` (`Webizen AI Synth.app`)
- Linux amd64: `webizen-ai-synth_*_amd64.deb`

Portable CLI binaries remain in the same release (`synth-windows-x86_64.exe`, `synth-macos-aarch64`, `synth-linux-x86_64`) with `.sig` and `.sha256` sidecars.

## First install

Run the installer for your OS. On Windows the shortcut launches `synth operator`. On macOS open `Webizen AI Synth.app`. On Debian/Ubuntu:

```bash
sudo dpkg -i webizen-ai-synth_0.0.41.3_amd64.deb
synth operator
```

The operator workbench is `http://127.0.0.1:8787/operator`. `synth serve` is the Studio HTTP workbench.

## Verify and apply a later portable binary

```text
synth update check
synth update verify --artifact synth-windows-x86_64.exe
synth update apply --artifact synth-windows-x86_64.exe
```

Apply copies to a sibling path. It never overwrites a locked `synth.exe` (the live MCP stdio child). Point MCP at the new sidecar, then `/mcps` reload.

Signatures use QualiaDB Ed25519 `sign_with_context` over the SHA-256 of the file (`synth.release.sha256.v1`). The public key is compiled into the binary (`.synth/update-public.hex`). The private seed is the GitHub Actions secret `SYNTH_UPDATE_SEED` and local `.synth/update-seed.hex` (gitignored).
