# Research dossier: geometry optimisation and numerical shader generation

**Prepared:** 16 November 2026 · **Companion to:** `qualia-etalon-specification-competition-v0.2.md`

This dossier records the research choices behind the competition baseline. It is a targeted technical review, not an exhaustive survey or a reproduction study. Primary sources were preferred. Search-result publication timestamps were not used as paper dates: titles, author pages, paper text, and version histories take precedence.

“Adopt” below means adopt an idea or acceptance requirement. It does not prescribe copying an implementation, adding a dependency, or treating an author-reported benchmark as a local result.

## 1. Research decisions that materially change the specification

| Decision | Rationale | Local opportunity | Discriminating experiment |
|---|---|---|---|
| Separate visual, intrinsic, and physical representations | Their objective functions differ; one triangle budget cannot describe all fidelity. | Existing geometry, DDG, solvers, correspondence records, and codecs. | Compare one shared mesh with separate hierarchies at equal total memory and application error. |
| Treat maps as first-class artifacts | Attribute, force, field, and operator transfers need different semantics. | Existing barycentric bindings, geometry remaps, symbolic/units tools. | Constant/rigid-motion, virtual-work, conservation, pose, and cross-domain leakage tests. |
| Separate ranking costs from acceptance measurements | A cheap local objective does not establish a global bound. | Current QEM plus distance/BVH and robust-predicate libraries. | Fixed-quality cost curves with independent distance/response evaluation. |
| Add intrinsic and response metrics | Image quality alone can miss operator or mechanics errors. | Eigen/QR/Cholesky, DDG, calculus, constitutive/physics libraries. | Spectral subspaces, diffusion, force/displacement, and modal response at matched geometry budgets. |
| Use units and assumptions in the lowering boundary | Symbolic simplification can be valid in real algebra and invalid in the executable numerical domain. | Existing units, assumption-aware CAS, derivative routines, Forge IR. | Deliberately invalid units/domains; ill-conditioned and cancellation fixtures; rewrite traces. |
| Make robustness adaptive | Most inputs need not pay maximum precision, but uncertain topology decisions need a stronger path. | Filtered/exact predicates and Etalon numerical routes. | Near-degenerate signs, fallback rate, queue capacity, and total latency. |
| Evaluate total lifecycle cost | Build/tune/transfer costs can erase steady-state improvements. | Existing workspace, tuning, cache, and evidence APIs. | Cold/warm separation and break-even curves over repeated uses. |

These are synthesis decisions for this specification. The cited papers support individual mechanisms; no paper proves the whole proposed architecture.

## 2. Primary research and practical references

### Geometry and attributes

**R01 — Garland and Heckbert. Surface Simplification Using Quadric Error Metrics. SIGGRAPH 1997.** [Author project](https://www.cs.cmu.edu/afs/cs.cmu.edu/user/garland/www/quadrics/index.html). Reviewed: primary project/publication summary. Role: mandatory geometric baseline. Limitation for this specification: its optimisation objective must not be relabelled a maximum-distance certificate; application-specific topology restrictions are explicit policy.

**R02 — Garland and Heckbert. Simplifying Surfaces with Color and Texture using Quadric Error Metrics. IEEE Visualization 1998.** [Author-hosted paper](https://publications.ri.cmu.edu/storage/publications/pub_files/pub2/garland_michael_1998_1/garland_michael_1998_1.pdf). Reviewed: paper metadata and indexed summary. Role: attribute-aware historical baseline. Implication: the competition must demonstrate more than adding generic feature weights to QEM.

**R03 — Hoppe. Progressive Meshes. SIGGRAPH 1996.** [Author project and retrospective](https://hhoppe.com/proj/pm/). Reviewed: project abstract and author retrospective. Role: progressive refinement, appearance attributes, and transition design. Candidate: retain useful collapse/refinement correspondence. A progressive representation is not automatically the same data structure as the existing cluster DAG.

**R04 — Cohen et al. Simplification Envelopes. SIGGRAPH 1996.** [Author-hosted paper](https://www.cs.umd.edu/~varshney/papers/simp_env.pdf). Reviewed: paper abstract/introduction excerpt. Role: geometric deviation as a separately bounded requirement. Candidate: envelope-based admissibility or an equivalent independently justified error test. Transfer of its guarantees requires its preconditions.

**R05 — Liu, Zhang, and Yuksel. Simplifying Textured Triangle Meshes in the Wild. SIGGRAPH Asia 2025.** [Version 2, 9 October 2025](https://arxiv.org/abs/2409.15458v2). Reviewed: version history and abstract. The original 2024 version had a shorter title; the specification cites the updated textured-mesh work. Role: non-manifold input and correspondence-based texture transfer. Candidate: compare mapping/rebaking against hard UV seam preservation. Application provenance still needs an independent policy.

**R06 — Hasselgren et al. Appearance-Driven Automatic 3D Model Simplification. EGSR 2021.** [NVIDIA research page](https://research.nvidia.com/labs/rtr/publication/hasselgren2021diffmodeling/). Reviewed: author abstract and publication metadata. Role: joint geometry/material optimisation through rendered comparisons. Candidate: a constrained appearance track. Evaluate unseen views/lighting; appearance matching does not establish mechanical fidelity.

**R07 — Luo et al. Global Discrete Optimization for Quad-Dominant Mesh Reduction. SIGGRAPH 2026 conference track.** [Author-hosted paper](https://roy-zzz08.github.io/assets/pdf/Quad.pdf). Reviewed: accessible paper introduction and formulation summary. Role: global edge-selection/ILP alternative for quad structure and editing. Candidate: optional cold authoring track. Account for optimisation cost and distinguish the authoring representation from the runtime triangulation.

### Intrinsic geometry and operators

**R08 — Lescoat et al. Spectral Mesh Simplification. Eurographics / CGF 2020.** [Author project](https://perso.telecom-paristech.fr/boubek/papers/SpectralMeshSimplification/). Reviewed: primary abstract and stated output. Role: preserve Laplacian spectral behaviour while retaining a mesh. Candidate: operator-aware comparison route. Spectral quality must be assessed through appropriate eigenspace and application metrics rather than one rendered view.

**R09 — Liu et al. Surface Simplification using Intrinsic Error Metrics. SIGGRAPH / TOG 2023.** [Author project](https://www.markjgillespie.com/Research/intrinsic-coarsening/index.html). Reviewed: project summary, publication metadata, erratum, and accessible paper introduction excerpt. Role: intrinsic coarsening with maps/prolongation for surface computation. Candidate: separate computational hierarchy. Full PDF access through one project link exceeded the browser size limit; this review does not reproduce its guarantees.

**R10 — Sharp and Crane. A Laplacian for Nonmanifold Triangle Meshes. SGP / CGF 2020.** [Author project](https://www.cs.cmu.edu/~kmcrane/Projects/NonmanifoldLaplace/index.html). Reviewed: primary abstract and method summary. Role: robust operator construction for difficult topology. Candidate: tufted-cover/intrinsic-Delaunay route when the domain needs it. Do not claim every existing local Laplacian implements this construction.

**R11 — Crane, Weischedel, and Wardetzky. Geodesics in Heat. TOG 2013.** [Author manuscript record](https://arxiv.org/abs/1204.6216); [later author CACM exposition](https://www.cs.cmu.edu/~kmcrane/Projects/HeatMethod/paperCACM.pdf). Reviewed: abstracts and linear-system method description. Role: amortised intrinsic distance work. Important local distinction: the inspected QualiaDB path performs explicit diffusion and bounded Jacobi; qualify that route rather than borrowing the factorisation strategy’s performance claim.

### Error evaluation, robustness, and virtual geometry

**R12 — Sacht and Jacobson. Cascading Upper Bounds for Triangle Soup Pompeiu–Hausdorff Distance. SGP / CGF 2024.** [Author manuscript](https://arxiv.org/abs/2406.10357). Reviewed: abstract and stated lower/upper-bound subdivision method. Role: tolerance-controlled geometric error evaluation. Candidate: CPU reference or hierarchical checker. Keep unresolved brackets explicit under a resource limit.

**R13 — Fan et al. GPU-accelerated Certified Hausdorff Distance Between Triangle Meshes. SIGGRAPH 2026, as identified by the author project.** [Author project](https://min-tang.github.io/GCHD/). Reviewed: project method summary; full PDF exceeded browser size limit. Role: bounded wavefront worklists and device-capacity-aware distance evaluation. Candidate: GPU acceleration after a trusted reference. The project reports RTX 5090 measurements; no performance number is adopted as a target or local result here. Its numerical safeguards require implementation-level examination before a port claims certification.

**R14 — Shewchuk. Adaptive Precision Floating-Point Arithmetic and Fast Robust Geometric Predicates. DCG 1997.** [Author page and implementation links](https://www.cs.cmu.edu/~quake/robust.html). Reviewed: author explanation of adaptive predicates and arithmetic concerns. Role: strong topology decisions without universal maximum precision. Existing local filtered/exact libraries are the first reuse target.

**R15 — Nanevski, Blelloch, and Harper. Automatic Generation of Staged Geometric Predicates. ICFP 2001.** [Author-hosted paper](https://www.cs.cmu.edu/~rwh/papers/geompred/icfp01.pdf). Reviewed: paper abstract/introduction describing staged filters and exact fallback. Role: bridge between mathematical expressions and generated robust predicates. Candidate: bounded generated filter plus qualified fallback; backend arithmetic assumptions are part of correctness.

**R16 — Karis, Stubbe, and Wihlidal. A Deep Dive into Nanite Virtualized Geometry. SIGGRAPH 2021 Advances in Real-Time Rendering course.** [Official course](https://advances.realtimerendering.com/s2021/); [slides](https://advances.realtimerendering.com/s2021/Karis_Nanite_SIGGRAPH_Advances_2021_final.pdf). Reviewed: official course listing and talk scope, plus indexed slide excerpts. Role: integrated virtual-geometry architecture. This is a production course presentation, not a new proof of portable WebGPU behaviour.

**R17 — Meshoptimizer. Maintainer documentation.** [Repository](https://github.com/zeux/meshoptimizer). Reviewed: current README, core pipeline, attribute simplification, and clustered-LOD references. Role: practical external baseline. Pin a revision and options; library outputs and default quality settings must be interpreted under their own definitions.

### Physics and reduced models

**R18 — Macklin, Müller, and Chentanez. XPBD: Position-Based Simulation of Compliant Constrained Dynamics. MiG 2016.** [Author paper](https://matthias-research.github.io/pages/publications/XPBD.pdf). Reviewed: paper introduction and formulation context. Role: compliant constraints and force-related interpretation. Existing volume kernels are reusable; connected-mesh assembly, contacts, and material response require their own contracts.

**R19 — Macklin et al. Small Steps in Physics Simulation. SCA 2019.** [Author paper](https://matthias-research.github.io/pages/publications/smallsteps.pdf). Reviewed: abstract and experiment motivation. Role: compare substepping with solver iterations at comparable cost. It motivates testing, not a rule that one integrator schedule wins in every model.

**R20 — An, Kim, and James. Optimizing Cubature for Efficient Integration of Subspace Deformations. SIGGRAPH Asia 2008.** [Author publication](https://graphics.stanford.edu/~djames/publication/optimizing-cubature-for-efficient-integration-of-subspace-deformations/). Reviewed: abstract and scope. Role: nonlinear reduced-model evaluation. Candidate: hyper-reduction where nonlinear force evaluation dominates. Training-domain and response-error validation remain essential.

**R21 — Li et al. Incremental Potential Contact: Intersection- and Inversion-free Large Deformation Dynamics. SIGGRAPH 2020.** [Author project](https://ipc-sim.github.io/). Reviewed: primary method and validation summary. Role: a reference for explicit contact feasibility and solver assumptions. Candidate: high-assurance contact track or independent comparison. Do not transfer its guarantees to a different volume penalty or incomplete solver.

### Compiler design and verification

**R22 — Ragan-Kelley et al. Decoupling Algorithms from Schedules for Easy Optimization of Image Processing Pipelines. SIGGRAPH 2012.** [Author project](https://people.csail.mit.edu/jrk/halide12/). Reviewed: publication metadata and primary abstract. Role: separate mathematical meaning from execution decisions. Adopt the separation within Forge; a new language is not required.

**R23 — Bangaru et al. Slang.D: Fast, Modular and Differentiable Shader Programming. SIGGRAPH Asia 2023.** [NVIDIA research page](https://research.nvidia.com/labs/rtr/publication/bangaru2023slangd/). Reviewed: primary compiler/type-system summary. Role: explicit derivative semantics and supported differentiation. Candidate: limited derivative lowerings that reuse existing calculus references. This does not establish that current QualiaDB JVP/VJP helpers already compile arbitrary shaders.

**R24 — Willsey et al. egg: Fast and Extensible Equality Saturation. POPL 2021.** [Versioned author manuscript](https://arxiv.org/abs/2004.03082v3). Reviewed: primary abstract and mechanism summary. Role: bounded exploration of equivalent expressions and analysis-assisted rewriting. Candidate: optional optimisation layer; guarded rewrite validity precedes extraction cost. The 2026 CACM retrospective was identified but is not confused with the original publication.

**R25 — GraphicsFuzz. Maintainer testing methodology.** [Semantics-preserving shader transformations](https://github.com/google/graphicsfuzz/blob/master/docs/glsl-fuzz-intro.md). Reviewed: methodology description. Role: metamorphic compiler testing. Apply the principle to WGSL fixtures; the GLSL toolchain is not itself WGSL conformance coverage.

**R26 — W3C WGSL, Candidate Recommendation Draft, 6 November 2026.** [Versioned specification](https://www.w3.org/TR/2026/CRD-WGSL-20261106/). Reviewed: floating-point evaluation, type/layout, and feature provisions relevant to the draft. Role: authoritative portable-language boundary. The application’s precision claims need to respect allowed transformations and the selected implementation.

**R27 — W3C WebGPU, Candidate Recommendation Draft, 15 September 2026.** [Versioned specification](https://www.w3.org/TR/2026/CRD-webgpu-20260915/). Reviewed: capability and indirect-drawing provisions relevant to the draft. Role: resource/device admission and portable work submission. Native provider extensions need a separate feature contract.

## 3. Deliberate exclusions and cautions

- **Progressive Photorealistic Simplification, SIGGRAPH 2026:** [author project](https://adirosenthal540.github.io/Progressive_Photorealistic_Simplification/) was checked. It removes/inpaints image content; the title does not make it a mesh-coarsening algorithm. It is not used as evidence for geometric simplification or empirical preservation.
- Neural geometry generation and view synthesis may be useful upstream, but their plausibility metrics cannot replace geometric, semantic, or physical acceptance criteria.
- A hardware-specific speedup in a paper is not an Etalon/WGSL prediction. Workgroup size, arithmetic, memory layout, and compilation costs need local evidence.
- An existing symbolic assumption system is valuable, but real-number identities do not by themselves establish floating-point equivalence.
- No inference was made from absent search results that a QualiaDB mathematical capability is missing. Gaps in this specification are integration/qualification questions unless a source audit establishes otherwise.
- No new dependency was installed and no code or model was downloaded for this review. Source inspection and online reading do not establish that a reference implementation builds or reproduces on this machine.

## 4. Priority order for the next agent

1. Reconcile current source and the 38 requirements; identify changed code or incorrect assumptions using the source-evidence file.
2. Choose three competing architectures with meaningfully different costs, rather than three prose variants of the same design.
3. Make transfer semantics, error evidence, and numerical domains concrete before arguing about neural tuning or maximum meshlet throughput.
4. Select the smallest discriminating experiments: protected-seam simplification; independent geometric bound evaluation; intrinsic operator transfer; one physical response; one typed/guarded kernel lowering.
5. Keep both the strongest conservative baseline and the most promising research route until evidence distinguishes them.

Every research result above remains author-reported. This dossier contains no newly reproduced accuracy or performance result.
