# Independent terrain shadow review — paused

Evidence: `C:/github/synth/production/shadow-final-20261113-093345-3de3c46b`.
Reviewed the actual 2048px terrain-on and terrain-off PNGs. Sparse fine diagonal
and angular dark flecks occur only with shadows enabled, mostly across the lower
right and central surface. The off image retains the broad terrain shading but
does not show those flecks. Numeric receipt passes (terrain p95 darkening 0,
fraction over four levels 0.0022299455); these metrics do not establish that the
remaining flecks are correct geometric shadows.

Read the current `mesh.wgsl`, `shadow_receiver.wgsl`, CPU receiver reference and
tests, and static sun-shadow vertex path in Etalon. Receiver derivatives precede
discard and cascade selection; PCF compensates each tap using a local receiver
plane and phase-dependent bilinear support. Tests cover planar receivers and a
nearby caster, not a tap crossing adjacent noncoplanar terrain triangles. That
is a possible residual mechanism, not a confirmed cause. No provider change was
made.

A bounded, read-only Python numerical diagnostic decoded the actual exported
terrain quantized mesh using the layout already documented in
`scripts/inspect-10d-topology.py`. It found 4096 vertices, 7938 triangles,
geometric normal Z range 0.3906184..1, and normal dot authored light
(0.4, 0.3, 0.85) range -0.0097148..0.9999286. Seven triangles face away from that
light. Therefore a blanket claim that this heightfield cannot self-shadow would
be false. The diagnostic did not authenticate the container, reproduce GPU
shadow sampling, or spatially correlate those seven triangles with the flecks.

Pending: establish the actual preview world/model transform, project geometric
faces and light occlusion into this exact camera, and correlate fleck positions
with genuine occlusion versus PCF crossing triangle boundaries. Inspect the
paired cosmos/caster images as well (their numeric receipts were read, but this
review did not visually inspect those PNGs). Interfaces separately owns the
fresh flora-canopy shadow-on/off comparison; coordinate that evidence before any
shader fix. No additional Cargo/GPU jobs were launched by this reviewer.

Status: residual appearance observed; cause and acceptance unresolved. User
requested pause before further diagnosis. No active tool calls remain.
