# P00 — initial production-core scope

Status: accepted P00 scope and test contracts after independent review by /root/scope_review (GPT-6 Astra/high). This accepts the scope, not unexecuted implementation tests. The user's 16 November instruction authorizes production-core implementation using plan v0.1; no later winning specification has been supplied. Optional R01–R08 remain uncommissioned.

## Required release domain

- Triangle surfaces, bounded to at most 50,000 vertices / 100,000 triangles for the initial production profile; existing operation-specific lower caps remain binding. Larger inputs require a named profile revision.
- Closed and open two-manifold inputs; disconnected components supported through admission. Non-manifold, nonfinite and degenerate inputs must be correctly rejected unless an explicitly qualified repair path is selected. Repair is never implicit.
- Protected material/subpart/provenance and authored feature boundaries. UV seams are corner-domain data; no promise that geometric QEM currently preserves them.
- Cold geometry budget 256 MiB; caller-bounded output/scratch, zero hot allocation where required, explicit cancellation and capacity failure.
- G1/T1/K1/R1 from specification v0.2. G1 uses decoded final output: symmetric upper bound <=1e-3 D, bracket width <=1e-5 D, protected displacement <=1e-5 D and zero forbidden merges. Failure to prove under budget is inconclusive.
- Operator/physical adapters preserve authoritative full models and measure true residual/transfer behavior. Intrinsic/spectral coarsening, reduced models and a new general contact solver are excluded from the core.
- Native DX12 on the available adapter is the first execution route. A named browser WebGPU route remains required for final release qualification; discovery and fixed-affine execution passed; broader qualification remains pending. Cached hardware inventory is not evidence of support.
- Ordinary f32 for the first numerical example; paired-f32 is separately qualified later. Integer topology/identity assertions remain exact.
- Perspective/orthographic, multiple views, two-sided materials, near-plane and supported deformation bounds belong to the final selection profile. The first tracer only demonstrates fixed asset views. The native target is DX12/NVIDIA GeForce RTX 5070 Laptop GPU (actual DX12 probe passed; driver 32.0.16.2114); browser target is installed Chrome 157.0.8103.52 WebGPU on this machine (actual WebGPU device/dispatch/readback passed on Intel gen-12lp; browser privacy withholds exact device identity).

## First milestone acceptance

Produce a small two-subpart mesh with an authored duplicated UV seam through the existing geometry APIs and production packaging, export and decode .10d, generate and actually execute a fixed Forge kernel, retain oracle/adapter/source identity evidence, and inspect front/side/three-quarter indexed-render previews. See acceptance-matrix.json for every requirement owner, fixture contract and threshold. Three repeated source/asset productions must yield identical HMC hashes where deterministic packaging is claimed. Empty/impossible requests and missing GPU have explicit non-success semantics. Any unconnected semantic/transfer/streaming feature remains visibly incomplete.

The initial tracer may use separately qualified producer and Forge paths while exposing the joining gap; acceptance of the full asset-to-kernel coupling requires a shared input identity, not two unrelated successful demos.

## Ownership and dispatch

Coordinator owns manifests, lockfiles, shared exports, baseline records and integration. Geometry agent discovers exact current APIs; Forge agent owns only its probe example and unique build target; independent QA owns fixtures/review. File leases are exact and non-overlapping. One heavy build and one GPU run at a time until RAM measurements permit more.

No shared source modifications, commits, merges, publication or deployment are included. Worktree-local changes and tests needed for this implementation are authorized. Reforecast after actual baseline/example measurements; the original 20-day target is not automatically shortened by quick scaffolding.
