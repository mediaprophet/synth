# Scope and baseline review

P00 accepted after independent /root/scope_review (GPT-6 Astra/high) review of all38 requirement contracts, source/worktree snapshots, leases and native/browser availability. Execution acceptance of later packets remains open.

Native generated affine:4099 values,zero mismatches,5 warmups/30 GPU timestamp samples on NVIDIA RTX5070LaptopGPU DX12,driver32.0.16.2114. This seeded oracle uses existing CPU f32 mul_add. The later asset phase independently uses scalar f64.

Browser generated affine:4099 values,zero mismatches,independent JavaScript binary64 and deliberately poisoned-output rejection on Chrome157.0.8103.52/Intel gen-12lp. In the raw receipt top-level count means compared values; oracle.count means mismatch count. Exact WGSL SHA256 matches the native emitted file. No browser timing qualification is claimed.

Negative native checks: unavailable Metal route is unsupported before pipeline/oracle; empty vertices fail; existing-output rejection preserves previous output. These do not establish physical device-loss handling.

Actual source defect found during geometry baseline: hard semantic penalty +infinity multiplied by zero transitions creates NaN and suppresses same-domain collapses. Preserve baseline reproduction, then branch zero-transition cost to zero with cross-domain/lock regressions. This is a minimal baseline correction, not full P06 acceptance.

Provenance limits: source HEADs and dirty source hashes are recorded; original overlays were excluded from worktrees. The existing Synth host is used only with its binary SHA256 explicitly recorded; its exact source equivalence is not inferred. All newly compiled examples identify their isolated source and commands.
