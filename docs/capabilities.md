# Synth asset capabilities — current inventory, 7 November 2026

> **SUPERSEDED 10 November 2026.** This file describes an 18-op host. The live catalog is **145** morphisms. Agents must read `C:\github\synth\AGENTS.md` and `C:\github\hypermedia-production\audit-docs\FUNCTIONS.md`. Do not use this inventory to claim tools are missing.

This inventory describes Synth after the general-purpose library integration.
It distinguishes executable product features from library functions and upstream
code that has not been integrated. Test evidence: `production-validation.md`.

## Working production features

- VibeScript source sessions, imports/helpers, retained source history and edits.
- A public `AssetProducer` interface and registry; typed Vibe values and input
  resources are passed to domain producers without a species schema.
- Built-in `files` producer: UTF-8 text, byte arrays, exact copies of retained
  inputs, and encoded resource handles from 18 shared-library operations.
- VibeScript composition of profiles, SVG, extruded/revolved/lofted meshes,
  CPU vector rendering, RGB image processing and PNG export/import, selection
  morphology, synthetic mono audio/WAV, and rational frame timing. See
  `asset-operations.md` for the complete operation contracts and bounds.
- Native HMC packages with source history, producer identity/version and input
  hashes; output resources remain intact files inside the container.
- Strict source/producer replay and exact package identity; unknown producer
  versions and altered results are rejected.
- Configurable source/evaluation/resource/package limits, cooperative cancellation,
  and rejection of invalid edits without replacing the preceding candidate.
- CLI generation, export, fresh reopen, capability inspection and profile listing.
- Authenticated native host: guarded inspection/edit/save/reopen, stale-context
  checks and native save audit. Agent tools expose the same operations.
- Optional native resource/status monitor; it is not a visual asset editor.
- Optional `.10d` to PNG review renderer, exercised on the generated extrusion,
  revolution and loft with nine native offscreen PNG views.

Production examples now generate SVG, three `.10d` meshes, two PNG frames,
processed/mask PNGs and PCM16 WAV. JSON, binary data and retained SVG remain
supported. A separate procedural
PPM raster producer in the external-consumer tests demonstrates adding a generator
without changing the host or package implementation. It is not shipped as a
built-in raster editor or producer.

The `files` producer can retain arbitrary file bytes. This is not equivalent to
parsing, editing, validating or generating every format it can contain.

## Comparison by creative workflow

| Familiar workflow | Synth currently offers | Substantial missing integration |
|---|---|---|
| CAD / parametric modeling | Profile primitives/polygons, affine transforms, path extrusion, revolution, loft and native `.10d` output | Constraint solver UI, boolean/fillet/assembly tools, drawing and CAD exchange workflows |
| Blender / 3D production | Procedural mesh generation and packaged native geometry; optional `.10d` preview | Sculpting, rigs, skinning, UV/material tools and full scene animation/render authoring |
| Illustrator / vector design | Generated profile geometry, transforms, SVG export and antialiased raster rendering | Interactive path editing, booleans, typography/layout and general vector document import |
| Photoshop / raster editing | Profile layers, RGB PNG input/output, blur, linear-light exposure, four blend modes, mask expansion/contraction/border | Interactive layer workspace, brushes, broader filters, transparent image import/export and complete color-management UI |
| After Effects / compositing | Rational frame times, source-driven frame geometry, layered profile rendering and PNG frame outputs | Keyframe/curve editor, media timeline, tracking and video/audio multiplexing/export |
| Audio workstation | Synthetic sine buffers, gain, Etalon PCM conversion and mono WAV export; SDK RMS/peak/filter primitives | Audio graph/sequencer/mixer/device workflows, instrument synthesis and broader audio import |
| Generative AI media | Numerical/inference-related primitives and producer extensibility | A working prompt-to-image/video/audio/model pipeline, model loading/job integration and domain validation |

These gaps describe Synth's exposed workflows, not an assertion that the
underlying engine repositories lack those algorithms.

## SDK extension catalogue

The runtime reports 11 provider namespaces and 37 extension operations. This is
an extension catalogue, not the complete Vibe language/standard-library surface.
34 descriptors are pure computations. The other three require governed effect
handling and are not admitted by Synth's current pure source host. Registering
an operation does not establish physical GPU execution or a complete authoring tool.

Vibe also supplies its language and standard-library computation facilities.
The large engine capability schema is not a list of executable Synth tools:
Synth binds `Host::capability_invoke` to its explicit 18-operation production
catalogue. All other invocation IDs are rejected. Graph mutation/query/publication and external host
services are likewise not granted by this source host. Authenticated document
admission does not silently grant these separate execution capabilities.
### mech

- `mech.kinematics.verlet_step` — pure computation descriptor.
- `mech.kinematics.kinetic_energy` — pure computation descriptor.
- `mech.kinematics.spring_damper_force` — pure computation descriptor.
- `mech.geometry.aabb_volume` — pure computation descriptor.
- `mech.geometry.excavation_prism_volume` — pure computation descriptor.

### gpu

- `gpu.compute.affine_f32` — pure computation descriptor.
- `gpu.compute.gemv4_f32` — pure computation descriptor.
- `gpu.compute.topk2_f32` — pure computation descriptor.
- `gpu.compute.rmsnorm_f32` — pure computation descriptor.
- `gpu.compute.rope_f32` — pure computation descriptor.

### forge

- `forge.shader.tile_schedule_gemm` — pure computation descriptor.
- `forge.shader.iq_power_reference` — pure computation descriptor.
- `forge.shader.coop_matrix_tile_ops` — pure computation descriptor.

### render

- `render.spatial.project_10d_to_3d` — pure computation descriptor.
- `render.spatial.sdf_torus_eval` — pure computation descriptor.
- `render.spatial.skeletal_lerp_joint` — pure computation descriptor.
- `render.spatial.brep_euler_characteristic` — pure computation descriptor.

### signal

- `signal.dsp.signal_rms_peak` — pure computation descriptor.
- `signal.dsp.cfar_cell_average` — pure computation descriptor.
- `signal.dsp.biquad_df1_step` — pure computation descriptor.
- `signal.vision.region_mean_u8` — pure computation descriptor.

### sense

- `sense.telemetry.clock_stamp_drift_ns` — pure computation descriptor.
- `sense.telemetry.spectrum_bin_hz` — pure computation descriptor.
- `sense.telemetry.rssi_log_distance_m` — pure computation descriptor.

### p64

- `p64.runtime.p64_align_offset` — pure computation descriptor.
- `p64.runtime.moe_top2_softmax` — pure computation descriptor.
- `p64.runtime.residency_admit_check` — pure computation descriptor.

### nquin

- `nquin.wire.compute_wire_parity` — pure computation descriptor.
- `nquin.wire.verify_wire_words` — pure computation descriptor.
- `nquin.wire.radix_bucket_byte` — pure computation descriptor.

### infer

- `infer.activations.rms_norm_f64` — pure computation descriptor.
- `infer.activations.softmax_f64` — pure computation descriptor.
- `infer.decode.governed_token_step` — requires governed effects; unavailable in the current pure source session.

### qgraph

- `qgraph.staging.seal_quin` — requires governed effects; unavailable in the current pure source session.
- `qgraph.staging.verify_quin_parity` — requires governed effects; unavailable in the current pure source session.

### modality

- `modality.allen.interval_relation` — pure computation descriptor.
- `modality.deontic.pack_norm_predicate` — pure computation descriptor.

## Shared-library integration boundary

The separate hypermedia-production checkout contains drafting geometry and
profile authoring, raster blend/selection/mask/color operations, vector rendering
adapters, terrain brushes, timeline editing, audio/video codecs and media
containers, and native device/pixel output. Its own README explicitly scopes the
initial extraction and qualification; source presence is not complete application
integration. Synth now binds profile-authoring, drafting-geometry,
vector-cpu-render-adapter, color-transforms, raster-blend-operators,
raster-selection-morphology and timeline-editing. QualiaDB supplies mesh kernels,
native geometry/PNG codecs, CV filters and audio; audio sample conversion uses
Etalon. Other library algorithms remain available for future adapters and are
not counted as exposed Synth operations. No subject-specific host is involved.

## Evidence

- `src/production/`: common production API and built-in files producer.
- `src/host.rs`, `src/agent.rs`: asset-neutral native host and agent surface.
- `tests/general_production.rs`: independently registered generator.
- `tests/asset_operations.rs`, `examples/production.vibe`: shared-library native
  assets, edits, retained PNG input, decoding and replay.
- `tests/authenticated_host.rs`: real native authority with signed test identities.
- `provenance/vibe-capabilities-20261107.json`: full runtime extension descriptors
  exported by Synth, including effects, budgets and target masks.
- `provenance/general-production-validation-20261107.json`: executed CLI result.

No Poet application files were changed for this integration.
