# species & anatomical modeling (1)

Corpus assembly for Qualia species records and the 5-Layer Anatomical & Pathological Compilation Contract. Find: `synth_find q=species` (or `corpus`, `q42`, `anatomy`, `bioportal`).

This is the only `qualia.*` catalog morphism. A species or organ is still a `.vibe` that calls locus / manifold / surface / kinetics ids. Do not add a Rust compiler per species.

## Morphisms

| ID | Args |
|---|---|
| `qualia.species.assemble_corpus` | `corpus` record or `"corpus.q42"` |

Non-fiction biosphere taxa (`domain=biosphere`, `fiction=false`) need evidence `.q42` before `synth_produce_present` (`synth_production_rules`). Encyclopedia sources live in Ecopunk (`C:\github\ecopunk`) and `C:\github\eco-intent\<species-slug>\`. `synth_compile_q42`, `synth_bioportal`, `synth_colab` (`routine="ontology_compile_q42"`), and graph queries stay allowed as the evidence path.

---

## Agent Anatomical & Biological Compilation Contract (`synth_validate_model`)

1. **Never emit unconstrained world-space meshes.** Parameterize every anatomical element relative to its parent skeletal frame $\text{SE}(3)$ or enclosing cavity SDF, identified by its DID and linked to its BioPortal (`FMA`/`UBERON`/`CL`/`DOID`/`HP`/`IDO`) `skos:Concept` reference (`owl:Thing` stripped).
2. **Enforce strict topological nesting & visceral tethering.** Translate all `fma:contained_in` relations into closed manifold SDF cavities; internal organs must satisfy $\phi_{\text{parent}}(\mathbf{p}) \le -\epsilon$ across all growth and respiratory states and declare both `parent_cavity_id` and a compliant serous/fascial `suspended_by_id` anchor (pericardium, pleura, mesentery, retroperitoneal fascia).
3. **Use intrinsic radial tissue coordinates ($d \in [0, 1]$) from bone to epidermis:**
   - `periosteum_capsule`: $d \in [0.00, 0.08]$ (or botanical `pith_heartwood`: $d \in [0.00, 0.45]$)
   - `ligament_deep_tendon`: $d \in [0.05, 0.20]$ (or botanical `sapwood_xylem`: $d \in [0.45, 0.82]$)
   - `musculature`: $d \in [0.15, 0.75]$ (volume-preserving incompressible $\det(\mathbf{F}) \in [0.85, 1.15]$)
   - `subcutis_fascia`: $d \in [0.75, 0.92]$ (or botanical `cambium_phloem`: $d \in [0.82, 0.92]$)
   - `dermis_epidermis`: $d \in [0.92, 1.00]$ (or botanical `periderm_bark_cuticle`: $d \in [0.92, 1.00]$)
4. **Anchor all attachments via topological surface UVs & retinacular pulleys.** Define origins, insertions, and ligament footprints via barycentric coordinates (`max_enthesis_uv_drift_m <= 1e-4` m, `enthesis_bindings.len() >= 2`) with graded Young's modulus ($E_{\text{muscle}} \to E_{\text{tendon}} \to E_{\text{fibrocartilage}} \to E_{\text{bone}}$), longitudinal epiphyseal growth axis $s \in [0, 1]$ (`diaphyseal_growth_s_range`), and `retinaculum_or_compartment_id` toroidal pulleys whenever `crosses_polyaxial_joint = true`.
5. **3-Stage Semantic Pipeline (ShEx $\to$ N3Logic $\to$ SHACL + Etalon Solver):**
   - **Emit ShEx (`_shapes.shex` / `shex_nodes`)** for BioPortal ingestion filters (`owl:Thing` stripping), stratified positive recursion over cyclic anatomical networks (vascular anastomoses, Circle of Willis, closed ribcage/pelvic loops), polymorphic cross-species traits, and modular `.hmc` streaming LoD unions (`@<LoadedHmcPayload>` OR `@<WireframeStub>`).
   - **Emit N3Logic (`_rules.n3`)** for transitive spatial closures (`fma:contained_in*`, `fma:deep_to*`), subcutaneous-to-visceral WAT spillover thresholds, and non-local visceral pathogen (`UBERON:0000160`) $\to$ systemic `IgE_Histamine_Surge` pulse routing.
   - **Emit SHACL (`_shapes.shacl.ttl` / `topological_layers`)** for closed-world numerical invariants ($\phi_{\text{cavity}} \le -\epsilon$, $d_{\text{deep}} < d_{\text{superficial}}$, UV enthesis lock, and the `DOID`/`HP` Pathological Exception Guard).
6. **4-Regime Dynamic Pathological & Metabolic Continuum Modeling:**
   - **Adipose Build-Up (`pathology_regime="adipose_hypertrophy_hyperplasia"`):** Parameterize depots in $d \in [0.75, 0.92]$ by hypertrophic mean cell volume $V_{\text{cell}}$ (`adipose_mean_cell_volume_pl`), hyperplastic density $\rho_{\text{cell}}$ (`adipose_cell_density_per_mm3`), and inextensible `retinacula_cutis_septa_tethers > 0` connecting $d=0.75$ to $d=0.92$, routing spillover above the SAT expandability threshold to visceral WAT (`UBERON:0014454`).
   - **Encapsulated Cysts & Benign Masses (`pathology_regime="encapsulated_cyst_benign"`):** Require `is_encapsulated=true` and `boundary_behavior="displace_only"` (pure hydrostatic mass-effect displacement without crossing $d$ layers unless `"qualia:CystRupture"` is authorized).
   - **Malignant & Invasive Neoplasms (`pathology_regime="malignant_invasive_neoplasm"`):** Dual-zone solid continuum core (`tumor_to_host_stiffness_ratio > 1.0`, `proliferation_kinetics="gompertzian"|"logistic"|"bertalanffy"`) + spiculated invasive margin permitted to cross $d$ layers **only** when paired with a verified `DOID`/`HP`/`MP` `pathology_ontology_ref` and `authorized_exception="qualia:TissueInvasion"|"qualia:Ulceration"`.
   - **Parasites, Bacteria & Cutaneous Rashes (`pathology_regime="cutaneous_pathogen_rash"`):** Prohibit baked static rash macule meshes (`independent_rash_mesh_count = 0`); require `reaction_diffusion_pde_active = true` ($\partial_t c = D \nabla^2_{u,v} c + f_{\text{bistable}}(c, S_{\text{pathogen}}) - \gamma c$) on $d \in [0.92, 1.00]$ driving Etalon shader erythema (hemoglobin SSS), edema micro-displacement ($\Delta d \propto c$), and stratum corneum desquamation.

Back to [README.md](README.md) · contract [../AGENTS.md](../AGENTS.md)
