# Logic Functions and Modalities Audit — Comprehensive Inventory and qualia-ui Gap Analysis

**Audit date:** 2026-08-18
**Workspace:** `C:\Projects\qualia-27062026`
**Target UI:** `C:\Projects\NLP\qualia-ui`
**Principal:** Timothy Charles Holborn

---

## 1. Executive summary

The QualiaDB workspace contains **42+ distinct logic modalities** and a comprehensive native dispatch system with **50+ domain-specific computational functions** — all implemented as live, tested Rust code. The modality ecosystem spans formal logic (deontic, epistemic, paraconsistent, LTL, CTL, ASP, defeasible, linear, description, dialectical), legal/governance reasoning (jural, STIT, causal, responsibility, capacity, delegation, contract, consensus, meta-deontic, legal compose, deontic compose, value flow, interaction governance), argumentation theory (Dung, bipolar, VAF), fuzzy and probabilistic reasoning, graph theory, interval reasoning, manifold logic, and control feedback.

The inference runtime includes a full GGUF bridge (45 files), WGSL Forge (67 files), N-dimensional renderer SDK, linear-algebra privacy engine (BFV HE + DP), model lifecycle orchestrator with thermal governor, and P64 weight container.

**Key numbers:**

| Metric | Count |
|---|---|
| Distinct logic modalities | 42+ |
| Modality source files | 78 |
| Total modality lines of code | ~25,000+ |
| Native dispatch functions | 50+ |
| GGUF bridge modules | 45 files |
| WGSL Forge modules | 67 files |
| Renderer SDK modules | 13 files |
| Privacy engine modules | 4 files |
| MCP logic tools | 3 (evaluate_modality, evaluate_logic_rules, symbolic_logic_infer) |
| Poet host logic invoke modules | 8 |
| Desktop Tauri commands (logic) | 4 (fetch_domain_ontology, execute_sparql_query, validate_shacl_shape, evaluate_logic_rules) |
| webizen-studio UI surfaces for logic | 18/20 categories (mostly stub/partial) |
| qualia-ui UI surfaces for logic | 0/20 categories |

**Honesty labels:**

- **live** — fully implemented and tested in engine
- **partial** — UI exists but incomplete
- **stub** — UI structure only, no real backend wiring
- **missing** — no UI surface at all
- **host-only** — implemented in engine/desktop but no UI

---

## 2. Modality taxonomy — full inventory

### 2.1 Formal logic modalities

#### Deontic Logic

**Source:** `crates/qualia-core-db/src/deontic_logic.rs` (589 lines), `crates/qualia-core-db/src/modalities/logic/deontic.rs` (1614 lines)

**Opcodes:**

| Opcode | Value | Meaning |
|---|---|---|
| `OP_OBLIGATE` | 0x10 | O(φ) — party must perform action |
| `OP_PERMIT` | 0x11 | P(φ) — party may perform action |
| `OP_FORBID` | 0x12 | F(φ)=O(¬φ) — party must not perform action |
| `OP_OPTIONAL` | 0x13 | U(φ) — optionality/indifference |
| `OP_GRATUITOUS` | 0x14 | G(φ) — gratuitousness |
| `OP_CONDITIONAL` | 0x15 | O(q\|p) — conditional obligation |
| `OP_STIT` | 0x16 | Reserved for STIT agency |
| `OP_UNDERCUT` | 0x17 | Undercutting defeater |

**Key structs:** `VcAttributes` (did_hash, roles[8], clearance_level, credential_issuer), `DeonticResult` (KeyRelease/AccessDenied), `DeonticStatus` (Active/Defeated/Expired/Malformed/Pending/Violated/Discharged), `DeonticVerdict` (norm, status, opcode, defeat_kind), `DefeatKind`

**Key functions:** `evaluate_vc_for_subgraph_key_release`, `evaluate_accessible_layers`, `write_vc_claim_quins`, `compile_n3_rule_to_norm`, `evaluate_deontic_contract`, `harvest_defeater_fingerprints`, `norm_has_active_defeater`, `compile_norm_quin`, `defeater_fingerprint`, `extract_deontic_opcode`, `extract_expiry_unix32`

**Constants:** `MAX_DEFEATER_SLOTS = 64`, `DEFEATER_BIT` (predicate bit 63)

**Tests:** 10 tests covering layer access, role/clearance checks, trusted issuer, child medical egress, N3-to-norm compilation

**Status: live**

#### Epistemic Logic

**Source:** `crates/qualia-core-db/src/modalities/epistemic.rs` (434 lines)

**Opcodes:**

| Opcode | Value | Meaning |
|---|---|---|
| `OP_KNOWS` | 0x20 | K_a(φ) — agent knows φ |
| `OP_BELIEVES` | 0x21 | B_a(φ) — agent believes φ |
| `OP_COMMON_KNOWLEDGE` | 0x22 | C(φ) — common knowledge |
| `OP_INTENT_LOCK` | 0x23 | Intent lock |
| `OP_IS_LOCKED` | 0x24 | Is locked check |
| `OP_NAMESPACE_LOCK` | 0x25 | Namespace lock |
| `OP_NAMESPACE_IS_LOCKED` | 0x26 | Namespace is locked check |

**Certainty bands:** KNOWS=255, AFFIRMS=230, BELIEVES=200, RECOGNIZES=200, CONSIDERS=128, SUPPOSES=100, SUSPECTS=80, SPECULATES=50, DOUBTS=20

**Key structs:** `EpistemicStatus` (Active/Uncertain/Skipped), `EpistemicError` (BufferOverflow/NodeLocked/NamespaceLocked), `EpistemicVerdict` (claim, status, certainty)

**Key functions:** `evaluate_epistemic_frame`, `check_node_locks`, `everyone_knows`, `distributed_knowledge`, `common_knowledge_via_announcement`, `positive_introspection`, `negative_introspection`, `muddy_child_knows`

**Tests:** 8 tests including muddy children deduction, common knowledge propagation, AGM belief revision, namespace locks

**Status: live**

#### Paraconsistent Logic

**Source:** `crates/qualia-core-db/src/modalities/paraconsistent.rs` (342 lines)

**Opcodes:**

| Opcode | Value | Meaning |
|---|---|---|
| `OP_ISOLATE` | 0x30 | Isolate contradiction |
| `OP_CONTRADICTION_SCORE` | 0x31 | Score contradiction severity |
| `OP_PARACONSISTENT_MERGE` | 0x32 | Merge paraconsistent contexts |

**Key structs:** `ContradictionStatus` (Consistent/Isolated{severity, isolation_context}), `ParaconsistentError` (BufferOverflow), `Belnap` (Neither/True/False/Both)

**Key functions:** `route_paraconsistent`, `global_saturation`, `local_saturation`, `is_saturated`, `Belnap::from_evidence`, `Belnap::and`, `Belnap::or`

**Tests:** 3 tests (Belnap four-valued tables, paraconsistent routing, saturation metrics)

**Status: live**

#### LTL (Linear Temporal Logic)

**Source:** `crates/qualia-core-db/src/modalities/temporal_ltl.rs` (454 lines)

**Opcodes:**

| Opcode | Value | Meaning |
|---|---|---|
| `OP_LTL_GLOBALLY` | 0x40 | G(φ) — always |
| `OP_LTL_FINALLY` | 0x41 | F(φ) — eventually |
| `OP_LTL_NEXT` | 0x42 | X(φ) — next |
| `OP_LTL_UNTIL` | 0x43 | φ U ψ — until |
| `OP_LTL_RELEASE` | 0x44 | φ R ψ — release |

**Key structs:** `LtlFormula` (Globally/Finally/Next/Until{ante,consequent}/Release{trigger,invariant}), `TemporalError` (AbortedTimeout), `SafetyMonitor` (invariant, violated)

**Key functions:** `evaluate_ltl_trace`, `evaluate_lock_lease`, `holds_within`, `holds_globally_within`, `evaluate_once`, `evaluate_historically`, `evaluate_since`, `predicate_interval`

**Tests:** 9 tests including MTL holds_within, all 5 LTL operators, past-time LTL, streaming safety monitor, metric globally + Allen interval

**Status: live** — fixes the bug mentioned in AGENTS.md §4-B (the legacy `Always/Eventually/Next` opcodes in `logic.rs` are NOT real LTL; use `temporal_ltl.rs` instead)

#### CTL (Computation-Tree Logic)

**Source:** `crates/qualia-core-db/src/modalities/ctl.rs` (561 lines)

**Constants:** `MAX_CTL_STATES = 256`

**Key functions:** `exists_finally` (EF φ — some path eventually reaches), `always_globally` (AG φ — every reachable state satisfies invariant), plus additional CTL operators

**Status: live** — branching-time temporal logic, bounded zero-heap BFS

#### ASP (Answer Set Programming)

**Source:** `crates/qualia-core-db/src/modalities/asp.rs` (459 lines)

**Constants:** `MAX_STABLE_MODELS = 8`, `ASP_MAX_ATOMS = 12`, `ASP_MAX_BODY = 6`

**Key structs:** `AspRule` (head, pos[6], neg[6]), `WeakConstraint` (pos, neg, weight), `AspOutcome` (Stable/NoStableModel)

**Key functions:** `compute_answer_sets` (real Gelfond-Lifschitz reduct), `enumerate_stable_models` (legacy heuristic), `ground_rule`, `model_penalty`, `optimal_answer_set`, `cautious_consequences`, `brave_consequences`, `answer_sets_or_paraconsistent`

**Tests:** 6 tests including even-loop constraints, grounding, weak constraints, cautious/brave consequences, paraconsistent fallback

**Status: live**

#### Defeasible Logic

**Source:** `crates/qualia-core-db/src/modalities/defeasible.rs` (399 lines)

**Opcode:** `OP_DEFEASIBLE_OVERRIDE = 0x50`

**Key structs:** `DefeasibleStatus`, `DefeasibleVerdict`, `DefeasibleRule`, `Conclusion`, `AmbiguityMode`

**Key functions:** `evaluate_defeasible_frame`, `holds_by_default`, `rules_conflict`, `is_superior`, `resolve_conflict`, `grounded_justified_rules`

**Status: live**

#### Linear Logic

**Source:** `crates/qualia-core-db/src/modalities/linear.rs` (405 lines)

**Bit:** `CONSUMED_BIT` (metadata bit 59)

**Key structs:** `Connective`, `StructuralRule`

**Key functions:** `consume_quin`, `is_consumed`, `tensor_consume`, `structural_rule_licensed`, `structural_derivation_valid`, `is_proof_net`, `zk_gated_consume`

**Status: live**

#### Description Logic

**Source:** `crates/qualia-core-db/src/modalities/dl.rs` (289 lines)

**Constants:** `DL_MAX_CLASSES = 256`

**Key functions:** `check_subsumption_quin`, `concepts_disjoint`, `abox_clash`, `role_subsumes`, `is_transitive_role`, `count_qualified_fillers`, `min_cardinality_met`, `max_cardinality_met`, `is_nominal_instance`

**Status: live** — comprehensive SROIQ constructs

#### Dialectical Logic

**Source:** `crates/qualia-core-db/src/modalities/dialectical.rs` (618 lines)

**Bits:** `COUNTERFACTUAL_BIT`, `DO_INTERVENTION_BIT`, `SYNTHESIZED_BIT`

**Key structs:** `IbisNode`

**Key functions:** `do_intervention`, `counterfactual_query`, `are_confounded`, `adjust_for_confounding`, `synthesize_dialectical`, `reachable_avoiding`, `is_necessary_cause`, `is_dialectical_contradiction`, `ibis_position_favoured`, `synthesis_coherence`

**Status: live** — do-calculus, counterfactuals, IBIS dialectical synthesis

#### Modal Logic

**Source:** `crates/qualia-core-db/src/modalities/modal.rs` (457 lines)

**Status: live**

#### Fuzzy Logic

**Sources:**
- `crates/qualia-core-db/src/modalities/fuzzy.rs` (453 lines)
- `crates/qualia-core-db/src/modalities/fuzzy_quantifiers.rs` (105 lines)
- `crates/qualia-core-db/src/modalities/fuzzy_rdf_schema.rs` (119 lines)
- `crates/qualia-core-db/src/modalities/fuzzy_type2.rs` (182 lines)

**Key functions:** `t_norm_godel`, fuzzy quantifiers, type-2 fuzzy sets, fuzzy RDF schema

**Status: live**

#### Probabilistic Logic

**Source:** `crates/qualia-core-db/src/modalities/probabilistic.rs` (406 lines)

**Key structs:** `BayesianNetwork`, `BayesianNode`, `MAX_BAYESIAN_NODES`

**Key functions:** `evaluate_threshold`, Bayesian network evaluation

**Status: live**

#### Argumentation Theory

**Source:** `crates/qualia-core-db/src/modalities/argumentation/` (4 files)

**Submodules:** `mod.rs` (699 lines, Dung-style abstract argumentation), `bipolar.rs` (153 lines, bipolar frameworks), `vaf.rs` (128 lines, value-based argumentation), `generation.rs` (83 lines, framework from trace)

**Key structs:** `Argument` (id, content, premise_quins, conclusion_quin, strength), `Attack` (attacker, target, attack_type, strength), `AttackType` (Rebuttal/Undercut/Undermine), `ArgumentationFramework`

**Bits:** `ARGUMENT_BIT`, `ATTACK_BIT`, `DEFENSE_BIT`

**Key functions:** `grounded_extension`, `stable_extensions`, `complete_extensions`, `grounded_contains` (zero-heap bounded), `framework_from_trace`

**Status: live** — HEAP-based cold reasoning layer (off hot path), with zero-heap `grounded_contains` primitive

#### Abductive Reasoning

**Source:** `crates/qualia-core-db/src/modalities/abductive/` (3 files)

**Submodules:** `mod.rs` (145 lines), `atms.rs` (127 lines, assumption-based truth maintenance), `probabilistic.rs` (100 lines)

**Status: live**

### 2.2 Legal/governance modalities

#### Jural Relations (Hohfeldian)

**Source:** `crates/qualia-core-db/src/modalities/jural.rs` (483 lines)

**Opcodes (0x30–0x37):**

| Opcode | Value | Position |
|---|---|---|
| `JURAL_CLAIM` | 0x30 | Claim |
| `JURAL_DUTY` | 0x31 | Duty |
| `JURAL_PRIVILEGE` | 0x32 | Privilege |
| `JURAL_NO_RIGHT` | 0x33 | No-Right |
| `JURAL_POWER` | 0x34 | Power |
| `JURAL_LIABILITY` | 0x35 | Liability |
| `JURAL_IMMUNITY` | 0x36 | Immunity |
| `JURAL_DISABILITY` | 0x37 | Disability |

**Key functions:** `compile_jural_quin`, `correlative`, `correlative_quin`, `find_unmet_correlatives`, `jural_correlativity_holds`, `jural_opposite`, `personhood_category_error`

**Status: live** — eight Hohfeldian positions with correlatives and opposites

#### STIT Agency

**Source:** `crates/qualia-core-db/src/modalities/stit.rs` (258 lines)

**Key functions:** `brought_about`, `is_duty_bearer`, `agentive_status` (Discharged/Violated/Active), `joint_discharged`, `joint_liable_members`

**Status: live** — "α Sees To It That φ" — binds deontic force to the agent who is the causal force

#### Causal Logic

**Source:** `crates/qualia-core-db/src/modalities/causal.rs` (385 lines)

**Constants:** `MAX_CAUSAL_NODES = 256`

**Key functions:** `but_for_cause`, `caused`, `dependents_voided`, `is_overdetermined`, `is_voided_by`

**Status: live** — but-for causation, root-node dependency, overdetermination (joint liability)

#### Responsibility & Systemic Meta-Guard

**Source:** `crates/qualia-core-db/src/modalities/responsibility.rs` (245 lines)

**Key structs:** `ResponsibilityStatus` (Alleged/Adjudicated/Dismissed)

**Key functions:** `adjudicate`, `is_enforceable_fact`, `rule_of_law_asymmetry`, `enforcer_overreach`, `accountability_vacuum`

**Status: live** — allegation→adjudication gate, systemic meta-guard against capture

#### Capacity

**Source:** `crates/qualia-core-db/src/modalities/capacity.rs` (353 lines)

**Key structs:** `CapacityStatus`

**Key functions:** `effective_principal`, `posthumous_standing`, `stipulation_binding`, `stipulation_voidable`

**Status: live**

#### Delegation

**Source:** `crates/qualia-core-db/src/modalities/delegation.rs` (285 lines)

**Key functions:** `authority_after_revocation`, `has_delegated_authority`, `revoked_descendants`

**Status: live**

#### Contract

**Source:** `crates/qualia-core-db/src/modalities/contract.rs` (245 lines)

**Key structs:** `FormationStage`

**Key functions:** `formation_stage`, `incorporates_by_reference`, `is_binding_contract`

**Status: live**

#### Consensus

**Source:** `crates/qualia-core-db/src/modalities/consensus.rs` (254 lines)

**Key structs:** `TxStatus`

**Key functions:** `can_form_joint_during_partition`, `is_globally_valid`, `survives_partition`, `transaction_status`

**Status: live**

#### Meta-Deontic

**Source:** `crates/qualia-core-db/src/modalities/meta_deontic.rs` (328 lines)

**Key functions:** `breach_predicate`, `breach_provenance`, `build_breach_record`, `endorsement_credential`, `record_breach_to_wal`

**Status: live**

#### Legal Compose

**Source:** `crates/qualia-core-db/src/modalities/legal_compose.rs` (240 lines, native-only)

**Key structs:** `Eligibility`, `MatchStatus`

**Key functions:** `selective_disclosure`, `translation_status`, `zk_eligibility`, `marginal_harm` (native-only), `proportionality_met` (native-only)

**Status: live**

#### Deontic Compose

**Source:** `crates/qualia-core-db/src/modalities/deontic_compose.rs` (525 lines)

**Key structs:** `MensRea`

**Key functions:** `agent_knows`, `classify_mens_rea`, `discharge_obligation`, `obligation_applies_in`, `obligation_globally`, `obligation_until`

**Status: live**

#### Value Flow

**Source:** `crates/qualia-core-db/src/modalities/value_flow.rs` (276 lines)

**Key functions:** `commons_cost`, `is_commons_discharged`, `outstanding`, `pool_after`, `royalty`

**Status: live**

#### Interaction Governance

**Source:** `crates/qualia-core-db/src/modalities/interaction_governance.rs` (287 lines)

**Key structs:** `Governance`, `PolicyMode`

**Key functions:** `govern_verdict`, `map_policy`, `permits_execution`, `policy_action`

**Status: live**

#### Identity Fabric

**Source:** `crates/qualia-core-db/src/modalities/identity_fabric.rs` (232 lines)

**Key functions:** `identifier_is_not_identity`, `identity_survives_loss`, `recompute_fabric`, `surviving_anchors`

**Status: live**

#### Capability Gap

**Source:** `crates/qualia-core-db/src/modalities/capability_gap.rs` (213 lines)

**Key functions:** `capability_gap`, `requirements_met`

**Status: live**

### 2.3 Spatial/temporal/modalities

#### Allen Interval Algebra + RCC8

**Source:** `crates/qualia-core-db/src/modalities/spatio_temporal.rs` (641 lines)

**Allen relations (7):** Before, Meets, Overlaps, Starts, During, Finishes, Equals

**RCC8 spatial relations (8):** Disconnected, ExternallyConnected, PartiallyOverlapping, TangentiallyProperPart, TangentiallyProperPartInverse, NonTangentialProperPart, NonTangentialProperPartInverse, Equal

**Key structs:** `TemporalOp`, `Rcc8Relation`, `SpatialRegion` (region_id, boundary_points, centroid, area)

**Key functions:** `evaluate_temporal`, `evaluate_rcc8`, `evaluate_rcc8_points` (zero-heap), `region_to_quin`, `quin_to_region`, `pack_point`, `unpack_point`, `spatial_index_query`, `minkowski_interval`, `causally_connectable`, `heat_equation_step`

**Tests:** 3 tests (RCC8 basic, RCC8 points zero-heap, temporal relations)

**Status: live**

#### Interval Reasoning

**Source:** `crates/qualia-core-db/src/modalities/interval_reasoning.rs` (549 lines)

**Key struct:** `TemporalInterval`

**Status: live**

#### 10D Manifold

**Source:** `crates/qualia-core-db/src/modalities/manifold.rs` (556 lines)

**Key structs:** `ManifoldCoordinate10D` (scale, attention_depth, epistemic_weight, topological_spin, temporal_decay, entropy_bias, spatial_phase, recurrence_frequency, density_threshold, manifold_curvature), `ManifoldState10D` (state_id, timestamp, coordinate)

**Key functions:** `encode_manifold_state` (2 parity-valid Quins), `decode_manifold_state`, `collect_manifold_states`, `project_manifold_ltl_trace`, `evaluate_manifold_answer_sets` (ASP), `project_10d_to_quaternion`, `solve_smallest_eigenvector`

**Tests:** 3 tests (round-trip, LTL projection, ASP topology)

**Status: live**

#### Manifold Logic

**Source:** `crates/qualia-core-db/src/modalities/manifold_logic.rs` (211 lines)

**Key functions:** `continuous_to_fact`, `integrate_abs`, `wave_val`, `WaveCoord`

**Status: live**

#### Diffusion

**Source:** `crates/qualia-core-db/src/modalities/diffusion.rs` (248 lines, native/wasm-scientific/wasm-full)

**Key functions:** `execute_diffusion_pass`, `trigger_diffusion`

**Status: live**

#### Control Feedback

**Source:** `crates/qualia-core-db/src/modalities/control_feedback/` (2 files, 691 lines)

**Bits:** `CONTROL_BIT`, `FEEDBACK_BIT`, `STABILIZATION_BIT`

**Key struct:** `ControlState`

**Status: live**

#### Calculus

**Source:** `crates/qualia-core-db/src/modalities/calculus/` (101 lines)

**Opcodes:** `OP_RK4_STEP`, `OP_SIMPSONS_INTEGRATION`, `OP_TRAPEZOIDAL_INTEGRATION`, `OP_ADAPTIVE_STEP`, `OP_GPU_INTEGRATION`

**Key structs:** `ContinuousGrid`, `SimdWidth`, `CalculusError`, `AlignmentError`

**Key functions:** `detect_simd_width`, `pack_f32_pair`, `unpack_f32_pair`, `resolve_aligned_byte_offset`

**Status: live** (native/wasm-scientific/wasm-full)

### 2.4 Graph and carrier modalities

#### Graph Theory

**Source:** `crates/qualia-core-db/src/modalities/graph_theory.rs` (1483 lines)

**Constants:** `MAX_HEAP_GRAPH_ANALYSIS_QUINS = 4096`, `MAX_BOUNDED_GRAPH_ANALYSIS_NODES = 128`, `MAX_SUBGRAPH_PATTERN_NODES = 8`

**Key structs:** `CommunitySpan`, `TopNodeScore`, `MotifRecord`, `SubgraphPattern` (adjacency[8][8], node_count), `BoundedGraphAnalysisSummary`

**Two tiers:**
- `analyze_graph_topology_bounded` — zero-heap, fixed-capacity arrays, preferred for hot path
- `analyze_graph_topology` — heap-backed (HashMap/HashSet/Vec), quarantined behind input cap

**Status: live**

#### Carrier

**Source:** `crates/qualia-core-db/src/modalities/carrier.rs` (195 lines)

**Key functions:** `extract_payload`, `media_tag`, `verify_binding`

**Status: live**

#### Epistemic Boundaries

**Source:** `crates/qualia-core-db/src/modalities/epistemic_boundaries.rs` (366 lines)

**Key structs:** `DegradationVector`, `ReferralDomain`, `ReferralTrigger`, `SocraticDegradation`

**Key functions:** `degrade_claim_to_socratic`, `detect_referral_by_severity`, `detect_referral_trigger`, `forbids_definitive_classification`, `identify_degradation_vector`, `requires_physiological_quarantine`

**Constants:** `BIO_DISCLAIMER`, `EMERGENCY_PROMPT`, `LEGAL_DISCLAIMER`, `LEGAL_JEOPARDY_PROMPT`

**Status: live** — safety boundaries for medical/legal classification

#### Likeliness

**Source:** `crates/qualia-core-db/src/modalities/likeliness/` (3 files, 281 lines)

**Key struct:** `Likeliness`

**Status: live**

### 2.5 Opcode summary table

| Range | Modality | Opcodes |
|---|---|---|
| 0x00–0x04 | mini_parser (reserved) | Parser-owned |
| 0x10–0x17 | Deontic | OBLIGATE, PERMIT, FORBID, OPTIONAL, GRATUITOUS, CONDITIONAL, STIT, UNDERCUT |
| 0x20–0x26 | Epistemic | KNOWS, BELIEVES, COMMON_KNOWLEDGE, INTENT_LOCK, IS_LOCKED, NAMESPACE_LOCK, NAMESPACE_IS_LOCKED |
| 0x30–0x37 | Jural/Paraconsistent | ISOLATE, CONTRADICTION_SCORE, PARACONSISTENT_MERGE, JURAL_CLAIM–JURAL_DISABILITY |
| 0x40–0x44 | LTL | GLOBALLY, FINALLY, NEXT, UNTIL, RELEASE |
| 0x50 | Defeasible | DEFEASIBLE_OVERRIDE |

**Note:** The `logic/mod.rs` also defines legacy opcode constants `OP_KNOW=0x20`, `OP_BELIEVE=0x21`, `OP_DOUBT=0x22` (epistemic), `OP_CONTRADICTION=0x30`, `OP_GLUT=0x31`, `OP_RELEVANCE=0x32` (paraconsistent), `OP_NEXT=0x40`, `OP_UNTIL=0x41`, `OP_ALWAYS=0x42`, `OP_EVENTUALLY=0x43`, `OP_RELEASE=0x44` (LTL). These overlap with the canonical opcodes in the modality files — the modality files are authoritative.

---

## 3. Core execution infrastructure

### 3.1 SLG Arena

**Source:** `crates/qualia-core-db/src/governance/webizen/arena.rs` (447 lines)

**Constants:**
- `SLG_ARENA_SIZE = 42 * 1024 * 1024` (42MB)
- `QUIN_SIZE = 48`
- `MAX_SLOTS = 917,504`
- `RECENT_SLOT_RING = 512`
- `MAX_RULE_VARS = 16`
- `MAX_GUARD_CONCLUSIONS = 256`
- `MAX_PREMISE_DEPTH = 16`
- `MAX_FIXPOINT_ROUNDS = 16`

**Key struct:** `SlgArena` (buffer: Vec<NQuin>, head_pointer, recent_slots[512], rule_registry, staged_rules)

**Key functions:** `new`, `register_rule`, `stage_rule`, `activate_staged_rule`, `collect_active_quins`, `fire_registered_rules`, `fire_guard_rules` (forward-chaining), `has_quin`, `write_table`

**Status: live** — 42MB ring buffer, 917,504 Quin slots

### 3.2 Webizen Bytecode VM

**Source:** `crates/qualia-core-db/src/governance/webizen_bytecode.rs` (409 lines)

**Key structs:** `VmError` (OutputBufferFull/InvalidProgram/HaltViolation), `GuardianshipContext` (principal_did, guardian_did), `ExecutionStats` (match_count, vm_cycles, direct_jump_ops, lexicon_lookup_ops)

**MSB dispatch:** MSB=1 → did:q42 topological pointer (direct jump); MSB=0 → FNV-1a dictionary hash (lexicon lookup)

**Opcodes:** `OP_END`, `OP_HALT_IF_FALSE`, `OP_MATCH_SUBJECT/PREDICATE/OBJECT`, `OP_EVAL_PERMIT/OBLIGATE/FORBID`, `OP_HALT_VIOLATION`

**Key functions:** `execute_program`, `execute_program_with_stats`

**Status: live** — SIMD variant with MSB dispatch

### 3.3 WebizenVM (logic.rs)

**Source:** `crates/qualia-core-db/src/modalities/logic/core.rs` (796 lines)

**Key structs:** `WebizenOpcode` (MatchSubject/Predicate/Object, EvalMetadataMask, BindRegister, MatchRegister, HaltIfFalse, EmitQuin, LessThan/GreaterThan/LessOrEqual/GreaterOrEqual, Always/Eventually/Next [BUGGY — not real LTL], EmitCalculatedQuin, YieldConfidence, LoadModel, EvictModel), `WebizenVM` (registers[16], bytecode_buffer[64], scrubbing_lock, yielded_op), `WebizenCompiler`

**Key functions:** `new`, `load_bytecode`, `execute_constraint`, `execute_implication`, `flatten_to_suspended`

**Known bug:** The `Always/Eventually/Next` opcodes compare a float threshold on a single Quin's object field — NOT real LTL. Use `temporal_ltl.rs::evaluate_ltl_trace` instead.

**Status: live** (but LTL opcodes are buggy — use `temporal_ltl.rs`)

### 3.4 Webizen VM (governance)

**Source:** `crates/qualia-core-db/src/governance/webizen/vm.rs` (1588 lines)

**`execute_vm_frame`** dispatches all `SlgOpcode::Native*` variants to real implementations (bioinformatics, clinical, chemistry, physics, economics). See §4 below.

### 3.5 N3 Rule Parser

**Source:** `crates/qualia-core-db/src/modalities/logic/n3_parser.rs` (1027 lines)

**Rule types (4 N3 arrows):**

| Arrow | `RuleType` | Semantics |
|---|---|---|
| `=>` | Strict | Classical modus ponens — forward chaining |
| `~>` | Defeasible | Can be overridden by a Defeater rule |
| `^>` | Defeater | Overtly defeats a matching Defeasible rule |
| `-o` | Linear | Linear logic: premise is consumed on firing |

**Special blocks:** `#asp { ... }` → `N3Event::AspBlock`, `qualia:diffuse { ... }` → `N3Event::DiffuseBlock`

**Rule weights:** Optional float prefix `(0.8) { premise } ~> { conclusion }`

**Key structs:** `Term` (Uri/Variable/Literal/Formula), `Triple`, `StackFormula` (triples[8]), `StackRule` (id, rule_type, weight, premise, conclusion), `N3Parser`, `N3Event`

**Constants:** `MAX_STACK_TRIPLES = 8`, `MAX_PARSE_BRACE_DEPTH = 16`, `MAX_PARSE_STATEMENTS = 65,536`

**Key functions:** `parse_all` (heap-backed), `parse_all_zero_heap` (zero-allocation), `parse_triples_into`, `parse_rule_zero_heap`, `q_hash_formula`, `term_uri_hash`

**Limitation:** Only single-triple formulas in premise/conclusion (multi-triple bodies truncated)

**Status: live**

### 3.6 N3 Compiler

**Source:** `crates/qualia-core-db/src/modalities/logic/n3_compiler.rs` (448 lines, native/wasm-scientific/wasm-full)

**Key structs:** `N3CompiledProgram`, `AgentIntentFrame`, `N3CompileError`, `SentinelError`, `N3OutputMode`

**Constants:** `MAX_CONTEXT_NAMESPACE_SLOTS`, `MAX_INTENT_SCOPE_SLOTS`

**Key functions:** `validate_rule_against_shapes`

**Status: live**

### 3.7 SHACL Compiler

**Source:** `crates/qualia-core-db/src/modalities/logic/shacl/shacl_compiler.rs` (435 lines), `crates/qualia-core-db/src/modalities/logic/shacl/shacl_types.rs` (335 lines), `crates/qualia-core-db/src/modalities/logic/shacl/validate.rs` (1294 lines)

**Key structs:** `ShaclCompiler`, `ShaclConstraint` (40+ variants including MinInclusive, MaxInclusive, MinCount, MaxCount, MinLength, MaxLength, Pattern, In, HasValue, Node, Class, Not, And, Or, Xone, DataType, NodeKind, Equals, LessThan, LessThanOrEquals, DeonticPolicy, DeonticObligate, DeonticPermit, DeonticForbid, DeonticNotExpired, EpistemicConstraint, EpistemicKnowledge, EpistemicBelief, CommonKnowledge, LtlConstraint, ParaconsistentConstraint, CalculusConstraint, GraphConstraint, ArgumentationConstraint, DialecticalConstraint, AspConstraint, ProbabilisticConstraint, DiffusionConstraint, LinearLogicConstraint, ControlFeedbackConstraint, IntervalArithmeticConstraint, EconVaRPositive, EconConvergedModel, EconPositivePrice, EconRiskBelowThreshold, EconWelfareAboveFloor), `CompiledShape`, `ShaclSeverity`, `PropertyPath`, `NodeKindType`, `CalcComputeTarget`, `ClinicalRiskModel`, `ProteinScoringMatrix`

**Key functions:** `compile`, `compile_constraints`, `compile_extension_shape`, `push_constraint`, `push_terminal`

**SHACL extension files:**
- `shacl_extensions/identity.rs` (570 lines) — identity, log, network, security, storage, system tray, tax recipient configurations
- `shacl_extensions/config.rs` (440 lines) — configuration SHACL shapes
- `specialized_libs_shacl.rs` (783 lines) — quantum biology, crypto, engineering, financial, linear algebra, medical, ML, chemistry, QPU, physics, statistics
- `core_modalities_shacl.rs` (754 lines) — ASP, Allen, argumentation, Bayesian, calculus, contradiction, control feedback, DL, dialectical, diffusion, epistemic, graph, interval, LTL, linear, ODE, paraconsistent, probabilistic, spatio-temporal, stable model
- `computational_maths_shacl.rs` (413 lines) — assumption, exact arithmetic, integral transform, interpolation, number theory, numerical method, special function, symbolic calculus, units, vector calculus
- `infrastructure_shacl.rs` (558 lines) — biological/chemical/financial/geospatial/mathematical/physical domain, geometric algebra, obfuscation, solvers
- `geometry_asset_shacl.rs` (405 lines) — geometry manifest validation
- `logic_modalities_shacl.rs` (444 lines) — logic modality shapes
- `owl/shacl_convert.rs` (729 lines) — OWL to SHACL conversion
- `owl/materialize.rs` (594 lines) — OWL materialization

**Status: live** — full vocabulary with extension points. Economics extension opcodes pending real wiring.

### 3.8 Rules Engine

**Source:** `crates/qualia-core-db/src/modalities/logic/rules.rs` (361 lines, native/wasm-scientific/wasm-full)

**Key structs:** `RuleEngine`, `RuleSet`, `GUARDIANSHIP_RULESET`

**Key functions:** `with_contract`, `load_n3`, `evaluate`

**Status: live**

### 3.9 QUBO Compilation

**Source:** `crates/qualia-core-db/src/modalities/logic/qubo.rs` (299 lines)

**Status: live**

### 3.10 N3Logic

**Source:** `crates/qualia-core-db/src/modalities/logic/n3logic.rs` (119 lines)

**Status: live**

### 3.11 CRDT

**Source:** `crates/qualia-core-db/src/foundation/crdt.rs` (216 lines)

**Key structs:** `DelegatedAccess` (principal_did[32], delegate_did[32], context_bound, expiration_timestamp, cryptographic_proof[64]), `CrdtResolver`, `SuspendedTransaction` (agreement_id, threshold, collected_signatures, registers[16], bytecode_buffer[64], yielded_op, suspended_quin), `SuspendedTransactionQueue` (queue[32])

**Key functions:** `resolve_lww`, `verify_delegation`, `push`, `apply_consensus_token`

**Tests:** 2 tests (CRDT resolution, CRDT bifurcation)

**Status: live**

### 3.12 Agency

**Source:** `crates/qualia-core-db/src/identity/agency.rs` (183 lines)

**Key functions:** `compute_scoped_merkle_root`, `sign_agency_root`, `verify_human_agency`, `stamp_fiduciary_metadata`, `scrub_quin_volatile`, `sign_graph_mutation`, `derive_lane_key` (PBKDF2, 310,000 iterations)

**Tests:** 2 tests (human agency verification, lane key determinism)

**Status: live**

### 3.13 Resolver

**Source:** `crates/qualia-core-db/src/query/resolver.rs` (533 lines)

**Type tags (bits 60-62 when MSB=0):**
- `INLINE_TAG_INTEGER = 0b001 << 60` → xsd:integer
- `INLINE_TAG_DECIMAL = 0b010 << 60` → xsd:decimal
- `INLINE_TAG_BOOLEAN = 0b011 << 60` → xsd:boolean
- `INLINE_TAG_FLOAT = 0b101 << 60` → f32 (newly allocated to fix §4-D conflict)

**Key functions:** `resolve_hash`, `write_iri_term`, `write_object_term`, `classify_inline_literal`, `format_ntriples_to`, `format_nquads_to`

**Status: live**

### 3.14 Lexicon

**Source:** `crates/qualia-core-db/src/query/lexicon.rs` (337 lines)

**`SemanticModality` variants:** Text, AudioHash, CeremonialVisual, PhoneticSchema, Visual, Spatial3D, Biosignal, Molecular, QuantumPhysical, SpatioTemporalTrace, Custom

**Key functions:** `generate_60bit_token` (masks to 60 bits), `generate_embedded_triple_id`, `tokenize_modal`, `intern`, `resolve`, `resolve_value`

**Status: live**

### 3.15 Identifier

**Source:** `crates/qualia-core-db/src/identity/identifier.rs` (140 lines)

**Key function:** `parse_did_q42` → u64 with bit 63 always set (FNV-1a + `| (1u64 << 63)`)

**Status: live**

---

## 4. Native dispatch functions

**Source:** `crates/qualia-core-db/src/governance/webizen/vm.rs` (1588 lines)

The `execute_vm_frame` function dispatches `SlgOpcode::Native*` variants to real domain implementations.

### 4.1 Bioinformatics

**Implementation:** `crates/qualia-core-db/src/domains/biological/bioinformatics.rs` (620 lines)

| Opcode | Function | Computes |
|---|---|---|
| `NativeNucleotideAlign` | `align_nucleotide()` | Smith-Waterman DNA alignment with BLAST nucleotide matrix |
| `NativeProteinAlign(u8)` | `align_protein()` | Smith-Waterman protein alignment with BLOSUM62/PAM250 |
| `NativeKmerFrequency(u8)` | `kmer_frequencies()` | K-mer counting with FNV-1a hashing |
| `NativeFastaValidation` | `validate_fasta_record()` | FASTA record validation with alphabet detection |
| `NativeGeneExpression` | `evaluate_gene_expression()` | Gene expression fold-change and significance |
| `NativeMetaboliteSimilarity` | `tanimoto_similarity()` | Tanimoto coefficient for Morgan fingerprints |

**Additional functions:** `smith_waterman`, `needleman_wunsch`, `minhash_sketch`, `build_upgma_tree`

**Status: live** — SIMD-accelerated with AVX2/NEON feature gates

### 4.2 Clinical engine

**Implementation:** `crates/qualia-core-db/src/clinical_engine.rs` (653 lines)

| Opcode | Function | Computes |
|---|---|---|
| `NativeClinicalRisk(0)` | `framingham_10yr_risk()` | Framingham 10-year CVD risk |
| `NativeClinicalRisk(1)` | `cha2ds2_vasc_score()` | CHA₂DS₂-VASc stroke risk |
| `NativeClinicalRisk(2)` | `score2_risk()` | SCORE2 European cardiovascular risk |
| `NativeDrugInteraction` | `check_drug_interactions()` | CYP450-based drug-drug interaction screening (19 pairs) |
| `NativeContraindication` | `check_contraindications()` | Drug-condition contraindication (11 rules) |
| `NativeFhirObservation(u64)` | `validate_fhir_observation()` | FHIR Observation validation with LOINC reference ranges (28 ranges) |

**Key structs:** `FraminghamInput`, `FraminghamResult`, `Cha2ds2VascInput`, `Cha2ds2VascResult`, `Score2Input`, `Score2Result`, `DrugInteraction`, `ContraindicationResult`, `FhirObservation`, `FhirValidationResult`

**Status: live**

### 4.3 Organic chemistry

**Implementation:** `crates/qualia-core-db/src/domains/chemical/organic_chemistry.rs` (627+ lines)

| Opcode | Function | Computes |
|---|---|---|
| `NativeSmilesValidation` | `validate_smiles()` | SMILES structural validation |
| `NativeInchiValidation` | `validate_inchi()` | InChI/InChIKey format validation |
| `NativeMolecularWeight(u64)` | `exact_molecular_weight()` | Exact monoisotopic MW from SMILES |
| `NativeLogP(u32)` | `compute_logp()` | Crippen LogP (atomic contributions) |
| `NativeTPSA(u32)` | `compute_tpsa()` | Ertl TPSA (topological polar surface area) |
| `NativeLipinskiFilter` | `evaluate_lipinski()` | Rule-of-Five drug-likeness |
| `NativeVeberFilter` | `evaluate_veber()` | Veber oral bioavailability |
| `NativeGhoseFilter` | `evaluate_ghose()` | Ghose drug-likeness |
| `NativeEganFilter` | `evaluate_egan()` | Egan passive absorption |
| `NativeFunctionalGroups` | `detect_functional_groups()` | SMARTS-inspired functional group detection (20 groups) |
| `NativePkaEstimate` | `estimate_pka()` | Functional-group-based pKa |
| `NativeChiralCenters` | `count_chiral_centers()` | sp3 C with 4 distinct substituents |
| `NativeCircularFingerprint(u8)` | `circular_fingerprint()` | Morgan fingerprint generation |
| `NativeArrhenius(u32)` | `arrhenius_rate()` | k = A·exp(−Ea/RT) |
| `NativeGibbsEnergy` | `gibbs_free_energy()` | ΔG = ΔH − TΔS |
| `NativeEquilibrium` | `equilibrium_constant()` | K = exp(−ΔG°/RT) |
| `NativeHendersonHasselbalch` | `henderson_hasselbalch()` | pH from pKa + concentration ratio |
| `NativeAtomEconomy` | `atom_economy()` | Trost 1991 green metric |
| `NativeEFactor` | `e_factor()` | Sheldon waste-per-product metric |
| `NativeGreenMetrics` | `green_metrics()` | Full suite: AE, E-factor, PMI, RME, CE |

**Key structs:** `BondOrder`, `Bond`, `Atom`, `Molecule`, `MolecularDescriptors`

**Status: live** — 15 atomic weights, 9 valences, BLOSUM62 matrix, 20 functional groups

### 4.4 Physics

**Implementation:** `crates/qualia-core-db/src/domains/physical/thermodynamics.rs` (242 lines)

| Opcode | Function | Computes |
|---|---|---|
| `NativeThermodynamics` | `metropolis_step()` | MCMC sampling for macroscopic properties |
| `NativeOdeSolver` | `evaluate_continuous_dynamics()` | RK4 ODE solver |
| `NativeRk4Step(u64)` | `Rk4Solver::step_quin()` | RK4 time-stepper with chaining |
| `NativeQuantumDft` | `calculate_ground_state_energy()` | Kohn-Sham DFT approximation |
| `NativeReceptorBinding` | `pinn_predict_receptor_binding()` | PINN binding affinity prediction |

**Additional functions:** `calculate_gibbs_free_energy`, `cell_ocv`, `pack_ocv`, `terminal_voltage`, `deliverable_power`, `max_power_point`, `array_mppt_power`, `heat_loss_rate`, `phase_change_energy`, `thermal_efficiency`

**Key structs:** `EnsembleState`, `ThermodynamicSampler`, `LithiumPack`, `SolarPanel`

**Status: live** — includes off-grid energy models (lithium packs, solar panels, heat transfer)

### 4.5 Economics

**Implementation:** `crates/qualia-core-db/src/domains/financial/economics/stochastic.rs` (274 lines)

| Opcode | Function | Computes |
|---|---|---|
| `NativeEconomics` | `run_monte_carlo_var()` | Monte Carlo VaR with selector dispatch |

**Additional functions:** `simulate_gbm_path`, `simulate_gbm_path_seeded`, `simulate_gbm_steps_into`, `run_monte_carlo_var_seeded_into`

**Key structs:** `StochasticError`, `SplitMix64`

**Status: live** — deterministic seeded variants, caller-buffered execution

---

## 5. Inference runtime

### 5.1 GGUF Bridge

**Directory:** `crates/qualia-core-db/src/gguf_bridge/` (45 files)

**Core modules:**

| File | Purpose | Status |
|---|---|---|
| `mod.rs` (1400+) | QTensor, MC8 chunk uniforms, weight arena bufs | live |
| `init.rs` (1400+) | GPU initialization, model loading, KV cache setup | live |
| `load.rs` (500+) | GGUF parsing, weight loading | live |
| `forward.rs` (2000+) | Layer-by-layer inference | live |
| `attention.rs` (1200+) | Multi-head attention | live |
| `ffn.rs` (600+) | SwiGLU feed-forward network | live |
| `gemm.rs` (600+) | Matrix multiplication | live |
| `output.rs` (900+) | Token sampling, logit processing | live |
| `embedding.rs` (300+) | Token embedding lookup | live |
| `prefill_arena.rs` (400+) | Prefill memory management | live |
| `prefill_async.rs` (1500+) | Asynchronous prefill execution | live |
| `resident_decode.rs` (1200+) | VRAM-resident decoding | live |
| `async_dispatch.rs` (700+) | Asynchronous operation dispatch | live |
| `quant_support.rs` (300+) | GGML quantization types | live |
| `gpu_params.rs` (400+) | Uniform buffer parameters | live |
| `pipeline_cache.rs` (300+) | Computed pipeline caching | live |
| `verify_arena.rs` (500+) | Result verification | live |

**Subdirectories:** `attention/` (fused_tail, preproject), `cuda_decode_plan/` (6 files), `mc8_wasm/` (4 files), `wasm_cpu/` (4 files including Q8_0 kernels), `browser/` (WebGPU support)

**Status: live**

### 5.2 GGUF Sharder + Tokenizer

**Directory:** `crates/qualia-core-db/src/inference/gguf_sharder/` (6 files)

**`GgufTokenizer` struct:** vocab (Vec<String>), bos_token_id, eos_token_id, add_bos_token, pre_type, merge_pairs, merge_rank_index, merge_rank_collision, token_to_id_map, special_tokens, stop_token_ids[8], stop_token_count

**Key functions:** `gguf_name_hash`, `gguf_skip_value`, chat family detection, BPE merges

**Status: live** — GGUF v2/v3 parsing, P64 section serialization

### 5.3 WGSL Forge

**Directory:** `crates/qualia-core-db/src/wgsl_forge/` (67 files)

**Core modules:**

| Module | Purpose | Status |
|---|---|---|
| `mod.rs` | ForgeError, BuiltinKernel, generate_builtin, resolve_execution_backend | live |
| `ir/core.rs` | KernelSpec, Op, BufferSpec | live |
| `ir/q42_bridge.rs` | Q42 opcode bridge (95+ opcode references) | live |
| `dispatch.rs` | ComputeCaps, kernel dispatch | live |
| `schedule.rs` | Schedule, ScheduleSpace | live |
| `emit/wgsl.rs` | WGSL code generation | live |
| `emit/ptx.rs` | PTX (NVIDIA) code generation | live |
| `emit/cuda_c.rs` | CUDA-C code generation | live |
| `emit/hlsl.rs` | HLSL (DirectX) code generation | live |
| `emit/msl.rs` | MSL (Metal) code generation | live |
| `emit/spirv.rs` | SPIR-V code generation | live |
| `validate.rs` | Naga validation | live |
| `oracle/mod.rs` | CPU oracle reference, OracleTolerance, ComparisonReport | live |
| `execute/mod.rs` | WgpuComputeContext, CudaComputeContext | live |
| `tune.rs` | Auto-tuning, TuningConfig, TuningResult | live |
| `cache.rs` | Adapter-keyed cache, ManifestCache | live |
| `roofline.rs` | RooflineEstimate | live |
| `runtime.rs` | ForgeRuntime | live |
| `physics/` | kinematics, molecular_dynamics | live |
| `audio/` | mel spectrograms | live |
| `graph_ops/` (13 files) | broadcast, elementwise, gather_dequant, neighbor, p64_bridge, reduce, scatter, slice, stencil, vision | live |

**CLI integration:** `crates/qualia-cli/src/shader.rs` (486 lines) — ListKernels, Doctor, Roofline, ProfileHardware, Generate, Validate, Certify, Tune, AutoTuneAll

**Status: live**

### 5.4 N-Dimensional Renderer SDK

**Directory:** `crates/qualia-core-db/src/render/gpu/` (507 lines), `crates/webizen-render/` (13 files)

**Key struct:** `PortalGpu` (device, queue, surface, depth_texture, picking_texture, ambient_pipeline, projector_pipeline, mesh_pipeline, bloom)

**Constants:** `MAX_AMBIENT_INSTANCES = 50,000`, `HDR_FORMAT = Rgba16Float`, `BLOOM_THRESHOLD = 1.0`, `BLOOM_INTENSITY = 1.15`, `BLOOM_STRENGTH = 0.85`

**Key functions:** `new_offscreen`, `new_surface` (HWND), `try_new_async` (WASM), `render`, `read_rgba8_into`

**Status: live** — shared wgpu device, Tensor10D projector, depth/bloom/mesh/picking, caller-buffered RGBA8

### 5.5 Linear-Algebra Privacy Engine

**Directory:** `crates/qualia-core-db/src/specialized_libs/linear_algebra/privacy/` (4 files)

**Key structs:** `HomomorphicKey`, `KeyRotationPolicy`, `HomomorphicKeyManager` (keys[8]), `HomomorphicOperations`, `SecureAggregation`, `PrivacyEngine`

**Submodules:** `bfv.rs` (BFV HE, feature-gated), `differential_privacy.rs` (Laplace/Gaussian, basic/advanced/RDP accounting)

**Status: live** — BFV HE + calibrated DP

### 5.6 Orchestrator

**Source:** `crates/qualia-core-db/src/inference/orchestrator.rs` (538+ lines)

**State machine:** `Discovered → MappedToDisk → StreamingVRAM → Active → Scrubbing`

**Key structs:** `ModelLifecycle`, `OrchestrationResult` (Committed/Blocked/Failed), `ThermalStatus` (Cool/Warm/Critical), `ThermalGovernor` (trait), `NullThermalGovernor`, `CalculusThermalGovernor` (RK4 simulation), `TaskOrchestrator`

**Key functions:** `load_model`, `evict_model` (async scrubbing), `orchestrate_inference`, `orchestrate_inference_grounded`, `thermal_state_label`, `routed_shapes`

**Status: live**

### 5.7 P64 Weight Container

**Directory:** `crates/qualia-core-db/src/q42/p64_weight/` (6 files)

**Layout:** 64-byte aligned sections — P64WeightHeader (64B), P64TensorEntry[] (64B each), pad, tensor blob region

**Constants:** `P64_MAGIC = b"p64\0"`, `P64_VERSION = 4`, `P64_DEFAULT_PAGE_LOG2 = 14` (16KB pages)

**Status: live** — 64B headers/entries/manifold records, CRC-32C

---

## 6. MCP and Poet host integration

### 6.1 MCP logic tools

**Source:** `crates/qualia-core-db/src/mcp/mcp_tool_impls/logic.rs` (395 lines)

**Three MCP tools:**

1. **`symbolic_logic_infer`** — SAT solver (BoundedSatSolver) and defeasible forward-chaining (ForwardChainingDefeasible)
2. **`evaluate_modality`** — dispatches to:
   - `ltl` → `evaluate_ltl_trace`
   - `asp` → `compute_answer_sets` (real Gelfond-Lifschitz)
   - `probabilistic` → `evaluate_threshold`
   - `argumentation` → `ArgumentationFramework::grounded_extension`
   - `deontic` → `evaluate_deontic_contract`
   - `epistemic` → `evaluate_epistemic_frame`
   - `dl` → `check_subsumption_quin`
   - `paraconsistent` → `route_paraconsistent`
3. **`evaluate_logic_rules`** — loads N3 rules into RuleEngine, fires in Webizen VM, evaluates Quin against fired conclusions, emits WAL audit events

**Status: live**

### 6.2 Poet host logic invoke

**Source:** `crates/qualia-core-db/src/poet_host/invoke/logic/` (8 files, 257 lines total)

**Modules:**

| File | Lines | Exports |
|---|---|---|
| `mod.rs` | 17 | asp_enumerate, caused, t_norm, deontic_evaluate, epistemic_evaluate, ltl_finally, ltl_globally, paraconsistent_route, subsumption_check |
| `causal.rs` | 77 | `caused` (causal reachability), `t_norm` (Gödel t-norm) |
| `deontic.rs` | 47 | `evaluate` (deontic contract scan on live graph slice) |
| `ltl.rs` | 27 | `finally`, `globally` |
| `subsumption.rs` | 26 | `check` (DL subsumption) |
| `paraconsistent.rs` | 23 | `route` |
| `epistemic.rs` | 21 | `evaluate` |
| `asp.rs` | 19 | `enumerate` |

**Status: live** — all modules feature-gated for wasm-logic/wasm-scientific/wasm-full

### 6.3 Desktop Tauri commands

**Source:** `crates/webizen-desktop/src/commands/semantic_logic.rs` (100 lines)

| Command | What it does |
|---|---|
| `fetch_domain_ontology(domain_id)` | Fetches domain ontology schema via OntologyCompiler |
| `execute_sparql_query(query)` | Executes local SPARQL over resident graph |
| `validate_shacl_shape(node, shape_uri)` | Validates local SHACL shape |
| `evaluate_logic_rules(input)` | Loads N3 rules, fires in Webizen VM, evaluates Quin, returns pass/fail verdicts |

**Status: live** — 4 commands registered in desktop command handler

### 6.4 Wellfair policy commands

**Source:** `crates/webizen-desktop/src/commands/wellfair/policy.rs` (221 lines)

| Command | What it does |
|---|---|
| `wellfair_evaluate_policy` | Evaluates access policy with sensitivity + epistemic status |
| `wellfair_grant_consent` | Grants consent for a scope |
| `wellfair_revoke_consent` | Revokes a consent grant |
| `wellfair_list_consents` | Lists all consent grants |
| `wellfair_add_condition` | Adds a medical condition report |
| `wellfair_add_allergy` | Adds an allergy report |
| `wellfair_add_disputed_diagnosis` | Adds a disputed diagnosis (epistemic boundary) |
| `wellfair_add_housing_safety` | Adds a housing safety report |

**Status: live**

---

## 7. Existing UI surfaces — webizen-studio

### 7.1 Summary by category

| Category | Component | File | Status |
|---|---|---|---|
| Deontic logic | `DeonticLogicEditor` | `components/deontic_logic_editor.rs` (57 lines) | **stub** — static examples, no persistence |
| N3 Logic Studio | `N3LogicStudio` | `components/n3_logic_studio.rs` (63 lines) | **stub** — hardcoded validation, no real parser |
| SHACL Validator | `ShaclValidator` | `components/shacl_validator.rs` (67 lines) | **partial** — honesty chip marks "not implemented" |
| SPARQL Explorer | `SparqlExplorer` | `components/sparql_explorer.rs` (228 lines) | **partial** — real local SPARQL, hex slot dumps not IRI-resolved |
| RDF-Star Editor | `RdfStarEditor` | `components/rdf_star_editor.rs` (53 lines) | **stub** — no persistence |
| Provenance Graph | `ProvenanceGraph` | `components/provenance_graph.rs` (56 lines) | **partial** — queries real provenance via SPARQL |
| Ontology Builder | `PersonalOntologyBuild` | `components/personal_ontology.rs` (38 lines) | **stub** — no compilation |
| Ontology Import | Import wizard | `components/ontology_import_wizard.rs` (400+ lines) | **partial** — real catalog integration |
| CRDT/Sync | Sync panel | `components/wellfair/sync_panel.rs` (52 lines) | **partial** — sync status, no CRDT resolution |
| Sync/Backup | Sync backup panel | `components/wellfair/sync_backup_panel.rs` (163 lines) | **partial** — configuration, no conflict resolution |
| Clinical Risk | `ClinicalRiskScorer` | `components/clinical_risk_scorer.rs` (187 lines) | **partial** — real Framingham via host, CHA₂DS₂-VASc client-side, SCORE2 hardcoded |
| Comorbidity | `ComorbidityAnalyzer` | `components/comorbidity_analyzer.rs` (12 lines) | **partial** — wrapper for wellfair panel |
| DICOM Viewer | `DicomViewer` | `components/dicom_viewer.rs` (121 lines) | **partial** — UI complete, image data is mock (diffusion frame proxy) |
| Chemistry | `ChemistryModeler` | `components/chemistry_modeler.rs` (148 lines) | **partial** — real MW + LogP via host, 2D structure is scaffold |
| Physics | Physics simulator | `components/physics_simulator.rs` (500+ lines) | **partial** — real forge certification, Canvas 2D placeholder |
| ODE Solver | `OdeSolver` | `components/ode_solver.rs` (32 lines) | **stub** — no real ODE solving |
| Bioinformatics | `BioinformaticsLab` | `components/bioinformatics_lab.rs` (36 lines) | **stub** — hardcoded GC content |
| GBM Simulator | `GbmSimulator` | `components/gbm_simulator.rs` (212 lines) | **partial** — real GBM with client-side RNG, no VaR |
| GPU Viewport | `NativeGpuViewport` | `components/native_gpu_viewport.rs` (219 lines) | **partial** — real wgpu surface, no compute probe UI |
| Model Lifecycle | `ModelLifecycle` | `components/model_lifecycle.rs` (95 lines) | **partial** — real lifecycle polling, progress hardcoded |
| Inference Monitor | `InferenceMonitor` | `components/inference_monitor.rs` (39 lines) | **stub** — hardcoded data |
| Model Setup | Settings | `components/settings/model_setup.rs` (264 lines) | **partial** — config UI, no LoRA/tokenizer inspector |
| Key Vault | `KeyVaultManager` | `components/key_vault_manager.rs` (57 lines) | **stub** — no real key operations |
| Agency Panel | Agency panel | `components/wellfair/agency_panel.rs` (322 lines) | **partial** — agency UI, no DID resolver/VC inspector |
| Identity Plane | Settings | `components/settings/identity_plane.rs` (348 lines) | **partial** — identity settings, no Merkle root visualization |
| MCP Inspector | `McpInspector` | `components/mcp_inspector.rs` (27 lines) | **partial** — MCP inspection, no VM state/trace |
| Disclosure Inquiry | Disclosure panel | `components/wellfair/disclosure_inquiry_panel.rs` (283 lines) | **partial** — trace display, no opcode inspector |
| Temporal Scrubber | `TemporalScrubber` | `components/temporal_scrubber.rs` (57 lines) | **partial** — temporal navigation, no Allen interval editor |
| Chora Panel | Chora panel | `components/wellfair/chora_panel.rs` (445 lines) | **partial** — temporal controls, no interval relation visualization |
| Epistemic Status | `EpistemicStatus` | `components/epistemic_status.rs` (69 lines) | **stub** — quantum context management, not epistemic logic |
| 3D Anatomy | Anatomy panel | `components/wellfair/anatomy_3d_panel.rs` (331 lines) | **partial** — 3D rendering for anatomy domain |

### 7.2 Categories with NO UI in either codebase

| Category | Status |
|---|---|
| Privacy/HE/DP | **missing** — no homomorphic encryption or differential privacy UI |
| Epistemic/Paraconsistent logic | **missing** — no belief/knowledge operator editor or contradiction handling UI |
| Jural relations (Hohfeldian) | **missing** — no 8-position correlative/opposite editor |
| STIT agency | **missing** — no duty-bearer/omission/joint liability UI |
| Causal logic | **missing** — no but-for/overdetermination/dependency UI |
| Responsibility/meta-guard | **missing** — no allegation→adjudication UI |
| Capacity | **missing** — no legal capacity UI |
| Delegation | **missing** — no delegated authority UI |
| Contract formation | **missing** — no contract formation stage UI |
| Consensus | **missing** — no partition-tolerant consensus UI |
| Meta-deontic | **missing** — no breach record UI |
| Legal compose | **missing** — no selective disclosure/ZK eligibility UI |
| Deontic compose | **missing** — no mens rea / obligation lifecycle UI |
| Value flow | **missing** — no commons cost / royalty UI |
| Interaction governance | **missing** — no policy mode / governance verdict UI |
| Identity fabric | **missing** — no identity survival / anchor UI |
| Capability gap | **missing** — no capability gap analysis UI |
| Graph theory | **missing** — no topology analysis / motif / subgraph isomorphism UI |
| CTL | **missing** — no branching-time temporal logic UI |
| Defeasible logic | **missing** — no defeasible rule editor (distinct from N3) |
| Linear logic | **missing** — no resource consumption / proof net UI |
| Description logic | **missing** — no SROIQ subsumption / cardinality UI |
| Dialectical logic | **missing** — no do-calculus / counterfactual / IBIS UI |
| Argumentation | **missing** — no Dung framework / bipolar / VAF UI |
| Abductive | **missing** — no ATMS / abductive inference UI |
| Fuzzy logic | **missing** — no fuzzy set / quantifier / type-2 UI |
| Probabilistic | **missing** — no Bayesian network UI |
| Interval reasoning | **missing** — no interval arithmetic UI |
| Manifold logic | **missing** — no 10D manifold coordinate / projection UI |
| Diffusion | **missing** — no diffusion pass UI |
| Control feedback | **missing** — no control state / stabilization UI |
| Calculus | **missing** — no RK4 / integration / SIMD UI |
| Carrier | **missing** — no media binding / payload extraction UI |
| Epistemic boundaries | **missing** — no Socratic degradation / referral trigger UI |
| Likeliness | **missing** — no likeliness algebra UI |
| Modal logic | **missing** — no modal logic UI |
| N3 compiler | **missing** — no N3 compilation / Sentinel validation UI |
| QUBO | **missing** — no QUBO compilation UI |
| OWL conversion | **missing** — no OWL→SHACL conversion / materialization UI |

---

## 8. qualia-ui gap analysis

### 8.1 Current state

`qualia-ui` has **zero dedicated logic/modality UI surfaces**. The only related infrastructure is:
- `tool_chest/core/ontology.rs` (202 lines) — ontology registry infrastructure, no UI
- `tool_chest/core/intent_bus.rs` — RDF-Star triple mentions in payload structures
- `browser/container_views.rs` — DID text labels in rights view
- `browser/diagnostics.rs` — diagnostics infrastructure

### 8.2 Required surfaces

The HyperCanvas must expose the full modality ecosystem through typed capability bindings. The following surfaces are required, organized by priority:

#### Tier 1 — Core reasoning tools (P0)

| Surface | Engine source | Desktop command | qualia-ui state |
|---|---|---|---|
| Deontic rule editor | `deontic.rs` | — | **missing** |
| N3 Logic Studio | `n3_parser.rs`, `n3_compiler.rs` | `evaluate_logic_rules` | **missing** |
| SHACL Validator | `shacl_compiler.rs`, `validate.rs` | `validate_shacl_shape` | **missing** |
| SPARQL Explorer | `query/` | `execute_sparql_query` | **missing** |
| RDF-Star Editor | `resolver.rs`, `lexicon.rs` | — | **missing** |
| Ontology Builder | `shacl_types.rs`, `owl/` | `fetch_domain_ontology` | **missing** |
| Evaluate Modality panel | All modalities | MCP `evaluate_modality` | **missing** |
| Symbolic logic infer | `solvers/symbolic_logic/` | MCP `symbolic_logic_infer` | **missing** |

#### Tier 2 — Legal/governance tools (P1)

| Surface | Engine source | qualia-ui state |
|---|---|---|
| Jural relations editor | `jural.rs` | **missing** |
| STIT agency inspector | `stit.rs` | **missing** |
| Causal liability analyzer | `causal.rs` | **missing** |
| Responsibility/meta-guard | `responsibility.rs` | **missing** |
| Capacity evaluator | `capacity.rs` | **missing** |
| Delegation tracker | `delegation.rs` | **missing** |
| Contract formation | `contract.rs` | **missing** |
| Consensus/partition | `consensus.rs` | **missing** |
| Meta-deontic breach log | `meta_deontic.rs` | **missing** |
| Legal compose (ZK/selective disclosure) | `legal_compose.rs` | **missing** |
| Deontic compose (mens rea) | `deontic_compose.rs` | **missing** |
| Value flow / commons | `value_flow.rs` | **missing** |
| Interaction governance | `interaction_governance.rs` | **missing** |
| Identity fabric | `identity_fabric.rs` | **missing** |
| Capability gap analyzer | `capability_gap.rs` | **missing** |

#### Tier 3 — Advanced logic tools (P1)

| Surface | Engine source | qualia-ui state |
|---|---|---|
| Epistemic logic editor | `epistemic.rs` | **missing** |
| Paraconsistent router | `paraconsistent.rs` | **missing** |
| LTL trace evaluator | `temporal_ltl.rs` | **missing** |
| CTL model checker | `ctl.rs` | **missing** |
| ASP answer set explorer | `asp.rs` | **missing** |
| Defeasible rule editor | `defeasible.rs` | **missing** |
| Linear logic proof net | `linear.rs` | **missing** |
| Description logic subsumption | `dl.rs` | **missing** |
| Dialectical/counterfactual | `dialectical.rs` | **missing** |
| Argumentation framework | `argumentation/` | **missing** |
| Abductive/ATMS | `abductive/` | **missing** |
| Fuzzy logic editor | `fuzzy*.rs` | **missing** |
| Probabilistic/Bayesian | `probabilistic.rs` | **missing** |
| Graph theory analyzer | `graph_theory.rs` | **missing** |
| Interval reasoning | `interval_reasoning.rs` | **missing** |
| Manifold 10D projection | `manifold.rs`, `manifold_logic.rs` | **missing** |
| Epistemic boundaries | `epistemic_boundaries.rs` | **missing** |
| Modal logic | `modal.rs` | **missing** |

#### Tier 4 — Domain computational tools (P1)

| Surface | Engine source | Desktop command | qualia-ui state |
|---|---|---|---|
| Clinical risk scorer | `clinical_engine.rs` | `calculate_framingham_risk` | **missing** |
| DICOM viewer | `dicom.rs`, `dicom_ingest.rs` | `get_diffusion_frame_rgba` | **missing** |
| Comorbidity analyzer | `qapp_api.rs` | — | **missing** |
| Chemistry modeler | `organic_chemistry.rs` | `calculate_chemistry_properties` | **missing** |
| Physics simulator | `thermodynamics.rs` | `certify_forge_physics` | **missing** |
| ODE solver | `calculus/` | — | **missing** |
| Bioinformatics lab | `bioinformatics.rs` | — | **missing** |
| GBM/VaR simulator | `stochastic.rs` | — | **missing** |
| Diffusion controller | `diffusion.rs` | `reconfigure_diffusion` | **missing** |

#### Tier 5 — Infrastructure/inspection tools (P2)

| Surface | Engine source | qualia-ui state |
|---|---|---|
| Bytecode/VM inspector | `webizen_bytecode.rs`, `core.rs` | **missing** |
| SLG arena inspector | `arena.rs` | **missing** |
| Forge compute probe | `wgsl_forge/` | `run_forge_compute_probe` | **missing** |
| Compute profile | `gpu_context.rs` | `get_qualia_compute_profile` | **missing** |
| Privacy/HE/DP panel | `privacy/` | **missing** |
| Model lifecycle | `orchestrator.rs` | `wellfair_get_model_lifecycle_status` | **missing** |
| Inference monitor | `orchestrator.rs` | `wellfair_get_llm_telemetry` | **missing** |
| GGUF tokenizer inspector | `gguf_sharder/tokenizer.rs` | **missing** |
| P64 weight inspector | `p64_weight/` | **missing** |
| CRDT/sync dashboard | `crdt.rs` | **missing** |
| Agency/Merkle inspector | `agency.rs` | **missing** |
| Key vault manager | `key_vault.rs` | **missing** |
| Policy evaluator | `policy.rs` | `wellfair_evaluate_policy` | **missing** |
| Consent manager | `policy.rs` | `wellfair_grant_consent` / `revoke` / `list` | **missing** |
| Carrier/media binding | `carrier.rs` | **missing** |
| Control feedback | `control_feedback/` | **missing** |
| Likeliness | `likeliness/` | **missing** |
| QUBO compiler | `qubo.rs` | **missing** |
| OWL converter | `owl/` | **missing** |

---

## 9. Implementation priorities

### Phase 1 — Core reasoning workbench (P0)

| Priority | Task | Dependencies |
|---|---|---|
| P0 | Deontic rule editor with OBLIGATE/PERMIT/FORBID/conditional/optional | IntentBus + `evaluate_deontic_contract` |
| P0 | N3 Logic Studio with real parser integration | `evaluate_logic_rules` command |
| P0 | SHACL Validator with real shape compilation | `validate_shacl_shape` command |
| P0 | SPARQL Explorer with IRI-resolved results | `execute_sparql_query` command |
| P0 | RDF-Star triple editor with embedded triple annotation | `resolver.rs` + `lexicon.rs` |
| P0 | Ontology Builder with SHACL shape authoring | `fetch_domain_ontology` command |
| P0 | Evaluate Modality dispatch panel (LTL/ASP/deontic/epistemic/DL/paraconsistent) | MCP `evaluate_modality` |

### Phase 2 — Legal/governance console (P1)

| Priority | Task | Dependencies |
|---|---|---|
| P1 | Jural relations editor (8 Hohfeldian positions, correlatives, opposites) | `jural.rs` |
| P1 | STIT agency inspector (duty-bearer, omission, joint liability) | `stit.rs` |
| P1 | Causal liability analyzer (but-for, overdetermination, dependency) | `causal.rs` |
| P1 | Responsibility/meta-guard (allegation→adjudication, rule-of-law asymmetry) | `responsibility.rs` |
| P1 | Contract formation tracker | `contract.rs` |
| P1 | Consensus/partition tolerance | `consensus.rs` |
| P1 | Meta-deontic breach log | `meta_deontic.rs` |
| P1 | Value flow / commons cost | `value_flow.rs` |
| P1 | Interaction governance | `interaction_governance.rs` |

### Phase 3 — Advanced logic tools (P1)

| Priority | Task | Dependencies |
|---|---|---|
| P1 | Epistemic logic editor (K/B/C operators, certainty bands, introspection) | `epistemic.rs` |
| P1 | Paraconsistent router (Belnap 4-valued, saturation metrics) | `paraconsistent.rs` |
| P1 | LTL trace evaluator (G/F/X/U/R, safety monitor, lock-lease) | `temporal_ltl.rs` |
| P1 | ASP answer set explorer (Gelfond-Lifschitz, weak constraints) | `asp.rs` |
| P1 | Argumentation framework (Dung, bipolar, VAF, grounded extension) | `argumentation/` |
| P1 | Graph theory analyzer (topology, motifs, subgraph isomorphism) | `graph_theory.rs` |
| P1 | Manifold 10D projection | `manifold.rs` |
| P1 | Dialectical/counterfactual (do-calculus, IBIS synthesis) | `dialectical.rs` |
| P1 | Fuzzy logic editor | `fuzzy*.rs` |
| P1 | Probabilistic/Bayesian network | `probabilistic.rs` |

### Phase 4 — Domain computational tools (P1)

| Priority | Task | Dependencies |
|---|---|---|
| P1 | Clinical risk scorer (Framingham, CHA₂DS₂-VASc, SCORE2, drug interactions) | `clinical_engine.rs` |
| P1 | Chemistry modeler (SMILES, descriptors, drug-likeness, green metrics) | `organic_chemistry.rs` |
| P1 | Physics simulator (thermodynamics, ODE, DFT, PINN) | `thermodynamics.rs` |
| P1 | Bioinformatics lab (SW alignment, protein, k-mer, FASTA, Tanimoto) | `bioinformatics.rs` |
| P1 | GBM/VaR simulator | `stochastic.rs` |

### Phase 5 — Infrastructure/inspection (P2)

| Priority | Task | Dependencies |
|---|---|---|
| P2 | Bytecode/VM inspector (opcode trace, register state, execution stats) | `webizen_bytecode.rs` |
| P2 | SLG arena inspector (slot usage, rule registry, fixpoint rounds) | `arena.rs` |
| P2 | Forge compute probe (Top-K, GEMM, FFT, certification) | `run_forge_compute_probe` |
| P2 | Privacy/HE/DP panel (BFV key management, DP budget) | `privacy/` |
| P2 | Model lifecycle + inference monitor | `wellfair_get_model_lifecycle_status` |
| P2 | CRDT/sync dashboard (LWW resolution, delegation, suspended transactions) | `crdt.rs` |
| P2 | Agency/Merkle inspector | `agency.rs` |
| P2 | Policy evaluator + consent manager | `wellfair_evaluate_policy` |

---

## 10. Known bugs and limitations

### 10.1 WebizenVM LTL opcodes (AGENTS.md §4-B)

The `Always/Eventually/Next` opcodes in `modalities/logic/core.rs` compare a float threshold on a single Quin's object field — they are NOT evaluating temporal traces. The correct LTL evaluator lives in `modalities/temporal_ltl.rs` with `evaluate_ltl_trace` supporting G/F/X/U/R over Quin traces. The legacy opcodes are left in place to avoid breaking existing tests.

**Action:** UI should use `temporal_ltl.rs` via MCP `evaluate_modality` with `modality: "ltl"`, not the legacy WebizenVM opcodes.

### 10.2 N3 parser multi-triple limitation

The N3 parser only supports single-triple formulas in premise/conclusion. Multi-triple bodies are truncated. This limits the complexity of rules that can be authored.

**Action:** UI should document this limitation. A future enhancement to support multi-triple bodies would require extending `MAX_STACK_TRIPLES` and the parser logic.

### 10.3 SHACL economics extensions pending

The SHACL compiler has economics extension constraints (`EconVaRPositive`, `EconConvergedModel`, `EconPositivePrice`, `EconRiskBelowThreshold`, `EconWelfareAboveFloor`) that are stubbed out and pending real opcode wiring.

### 10.4 Thermal governor

The `NullThermalGovernor` always returns `Cool`. The `CalculusThermalGovernor` uses RK4 simulation but real hardware sensor integration is pending.

### 10.5 Argumentation framework is HEAP-based

The `ArgumentationFramework` uses `HashMap`/`HashSet`/`Vec` — it is the COLD reasoning layer, off the hot path. The zero-heap `grounded_contains` primitive exists for hot-path use. A full zero-heap rewrite (bounded bitmask sets, ≤64 arguments) is a candidate for a future pass.

### 10.6 Resolved bugs (from AGENTS.md)

- **§4-B LTL bug:** Resolved — use `temporal_ltl.rs` instead of legacy opcodes
- **§4-D Object type-tag conflict:** Resolved — `INLINE_TAG_FLOAT = 0b101 << 60` allocated
- **§4-E derive_lane_key:** Resolved — PBKDF2 with 310,000 iterations
- **§4-F DelegatedAccess:** Resolved — `[u8; 32]` DID hashes, `[u8; 64]` Ed25519 signature
- **§4-C execute_differential_diagnostics:** Resolved — caller-supplied buffers

---

## 11. Architecture recommendations for qualia-ui

### 11.1 Typed intent bus for modality dispatch

The HyperCanvas should expose a single `EvaluateModality` intent that dispatches to the MCP `evaluate_modality` tool. The intent payload should include:

```json
{
  "modality": "deontic|epistemic|paraconsistent|ltl|asp|probabilistic|argumentation|dl",
  "quins": [{"subject": 123, "predicate": 456, "object": 789, "context": 0}],
  "formula": {"type": "globally", "predicate": 456},
  "agent_did_hash": 123,
  "world_hash": 456,
  "now_unix": 1697299200
}
```

### 11.2 Reusable modality result components

Each modality returns a different result shape. The UI should define reusable components:

- **Verdict list** — for deontic, epistemic, defeasible (status + opcode + certainty)
- **Boolean result** — for LTL, CTL, causal, responsibility (true/false + explanation)
- **Answer set list** — for ASP (bitmask-decoded atom sets)
- **Routing result** — for paraconsistent (consistent_count, isolated_count, saturation)
- **Extension list** — for argumentation (grounded/stable/complete extensions)
- **Subsumption result** — for DL (subsumed: bool)
- **Graph analysis summary** — for graph theory (centrality, communities, motifs)

### 11.3 Rule authoring pattern

N3 rules, deontic norms, SHACL shapes, defeasible rules, and ASP rules all follow a similar authoring pattern:

1. **Source editor** — textarea with syntax highlighting
2. **Parse + validate** — calls engine parser, shows errors
3. **Compile** — calls engine compiler, shows opcodes
4. **Evaluate** — runs against live graph, shows results
5. **Persist** — saves to WAL with provenance

This pattern should be abstracted into a reusable `RuleAuthoringPanel` component.

### 11.4 Domain tool pattern

Clinical, chemistry, physics, bioinformatics, and economics tools follow a similar pattern:

1. **Input form** — domain-specific parameters
2. **Compute** — calls native dispatch function
3. **Result display** — domain-specific visualization
4. **Provenance** — Q42 provenance hash for audit

This should be abstracted into a reusable `DomainToolPanel` component.

---

## 12. Summary

The QualiaDB workspace contains **42+ distinct logic modalities** spanning formal logic, legal/governance reasoning, argumentation theory, fuzzy/probabilistic reasoning, and spatial/temporal reasoning — all implemented as live, tested Rust code with zero-heap hot paths and 42MB Sentinel arena enforcement. The native dispatch system provides **50+ domain-specific computational functions** across bioinformatics, clinical medicine, organic chemistry, physics, and economics. The inference runtime includes a full GGUF bridge (45 files), WGSL Forge (67 files) with 6 backend targets, N-dimensional renderer SDK, BFV homomorphic encryption + differential privacy, and a model lifecycle orchestrator with thermal governor.

The MCP layer exposes 3 logic tools (`evaluate_modality`, `evaluate_logic_rules`, `symbolic_logic_infer`) covering 8 modalities. The Poet host exposes 9 logic invoke functions. The desktop exposes 4 Tauri commands for SPARQL, SHACL, ontology, and N3 rule evaluation.

`webizen-studio` has UI surfaces for 18/20 categories, but **zero are fully live** — 13 are partial and 5 are stubs. `qualia-ui` has **zero dedicated logic/modality UI surfaces**.

The critical path is:
1. **IntentBus + MCP transport** — without this, no modality can be invoked from the HyperCanvas
2. **Evaluate Modality dispatch panel** — the single entry point for all 8 MCP-exposed modalities
3. **N3 Logic Studio** — wire `evaluate_logic_rules` for real rule authoring
4. **SHACL Validator** — wire `validate_shacl_shape` for real shape validation
5. **SPARQL Explorer** — wire `execute_sparql_query` with IRI resolution
6. **Deontic rule editor** — the most important governance modality
7. **Domain tool panels** — clinical, chemistry, physics, bioinformatics, economics
8. **Infrastructure inspectors** — VM, arena, forge, privacy, model lifecycle

The engine is comprehensive. The UI is the gap.

---

*Generated with [Devin](https://devin.ai) for Timothy Charles Holborn. Audit performed 2026-08-18 against QualiaDB branch 0.0.30/0.0.31-dev and qualia-ui v0.1.0.*
