# N3Logic, Prolog-like Reasoning, and SPARQL/OWL Integration — Supplementary Audit

**Audit date:** 2026-08-18 (supplementary to `20260818_logic-modalities-audit.md`)
**Workspace:** `C:\Projects\qualia-27062026`
**Target UI:** `C:\Projects\NLP\qualia-ui`
**Principal:** Timothy Charles Holborn

---

## 1. Why this supplement exists

The primary logic-modalities audit (`20260818_logic-modalities-audit.md`) catalogues 42+ logic modalities and 50+ native dispatch functions. However, it under-documents the **N3Logic ecosystem** — the parser, compiler, forward-chaining engine, symbolic logic solvers, SPARQL 1.1 library, and OWL RL materialization — and does not clearly state what Prolog-like capabilities exist versus what is absent.

This document fills those gaps so the UI team can build complete surfaces for rule authoring, query, reasoning, and ontology materialization.

---

## 2. N3Logic ecosystem — full capability inventory

### 2.1 N3 Parser

**Source:** `crates/qualia-core-db/src/modalities/logic/n3_parser.rs` (1027 lines)

The N3 parser is a native, zero-copy Notation3 parser with two complementary front-ends:

| Front-end | AST | Allocation | Use case |
|---|---|---|---|
| Heap-backed | `Formula`/`Rule` (Vec-backed) | Allocates | Cold path — convenience for the compiler |
| Zero-allocation | `StackFormula`/`StackRule` (fixed arrays) | **Zero heap** (proven by `dhat` test) | Hot path, edge hardware |

**Supported N3 surface syntax:**

| Feature | Syntax | Status |
|---|---|---|
| Triples | `s p o .` | live |
| Predicate lists | `s p1 o1 ; p2 o2 .` | live |
| Object lists | `s p o1 , o2 , o3 .` | live |
| rdf:type keyword | `s a Class .` (kept as bare `a` token, not expanded to `rdf:type`) | live |
| Strict implication | `{ premise } => { conclusion } .` | live |
| Defeasible implication | `{ premise } ~> { conclusion } .` | live |
| Defeater | `{ premise } ^> { conclusion } .` | live |
| Linear implication | `{ premise } -o { conclusion } .` | live |
| Rule ID annotation | `[rule_id] { ... } => { ... } .` | live |
| Rule weight | `(0.8) { ... } ~> { ... } .` | live |
| Formula quoting/reification | `{ ... }` as a `Term::Formula` (identified by canonical hash) | live |
| ASP blocks | `#asp { ... }` → `N3Event::AspBlock` | live |
| Diffusion blocks | `qualia:diffuse { ... }` → `N3Event::DiffuseBlock` | live |
| Comments | `#` line comments | live |
| Variables | `?x`, `?y` (prefixed with `?`) | live |

**Term types (4):**

```rust
pub enum Term<'a> {
    Uri(&'a str),       // IRI or prefixed name
    Variable(&'a str),  // ?x
    Literal(&'a str),   // "string" or numeric
    Formula(&'a str),   // { ... } quoted graph (reification)
}
```

**Rule types (4 N3 arrows):**

| Arrow | `RuleType` | Semantics | Compiler output |
|---|---|---|---|
| `=>` | Strict | Classical modus ponens — forward chaining | `Unify, Call, Halt` |
| `~>` | Defeasible | Can be overridden by a Defeater rule | `CheckDefeaters, Unify, Call, WarnOnly` |
| `^>` | Defeater | Overtly defeats a matching Defeasible rule | `NativeUnless, Halt` |
| `-o` | Linear | Linear logic: premise is consumed on firing | `NativeLinearConsume, Unify, Call, Halt` |

**Resource caps (anti-DoS):**

| Constant | Value | Purpose |
|---|---|---|
| `MAX_STACK_TRIPLES` | 8 | Max triples in a zero-allocation `StackFormula` |
| `MAX_PARSE_BRACE_DEPTH` | 16 | Max `{` nesting depth |
| `MAX_PARSE_STATEMENTS` | 65,536 | Max top-level statements per parse |

**Formula identity hashing:** `q_hash_formula` produces a canonical 64-bit FNV-1a hash of a quoted formula's text, with runs of Unicode whitespace collapsed to a single space. This gives a **stable identity** for quoted statements so `{ :a :b :c }` and `{  :a  :b  :c  }` denote the same node. Unicode-aware (handles CJK ideographic space U+3000, non-breaking space U+00A0, etc.).

**Known limitation:** Only single-triple formulas in premise/conclusion are fully supported. Multi-triple bodies are truncated to `MAX_STACK_TRIPLES = 8`. The audit notes this in §10.2.

**Status: live**

---

### 2.2 N3 Compiler

**Source:** `crates/qualia-core-db/src/modalities/logic/n3_compiler.rs` (448 lines, native/wasm-scientific/wasm-full)

The N3 compiler is the bridge from parsed N3 rules to executable Sentinel bytecode. It validates rules against compiled SHACL shapes, then lowers them to `SlgOpcode` sequences for the Webizen VM.

**Compilation pipeline:**

```
N3 text
  → n3_parser::parse_all           (parse rules)
  → validate_rule_against_shapes   (SHACL firewall — numeric range checks)
  → compile_rule_to_zero_heap      (lower to fixed-size CompiledRule)
  → compile_rule_to_opcodes        (emit SlgOpcode sequence)
  → compile_rule_to_quin           (emit deontic norm NQuin)
  → execute_compiled_program       (run in 42MB SlgArena)
```

**Compiled term types:**

```rust
pub enum CompiledTerm {
    Uri(u64),       // q_hash of IRI
    Variable(u64),  // q_hash of variable name
    Literal(u64),   // q_hash of literal text
}
```

**Compiled structures:**

| Struct | Purpose |
|---|---|
| `CompiledTriple` | (subject, predicate, object) as `CompiledTerm` |
| `CompiledFormula` | Up to 8 `CompiledTriple` in a fixed array |
| `CompiledRule` | id_hash, rule_type, weight, premise, conclusion |
| `N3CompiledProgram` | opcode_count, quin_count |

**SHACL firewall (`validate_rule_against_shapes`):**
- Validates each rule's conclusion against compiled SHACL shapes BEFORE compilation
- Numeric range checks happen here (before hashing literals away)
- Fail-closed: any shape violation aborts the entire batch
- Zero-heap: validation reads the `Rule` (no allocation); compile step is stack-only

**Rule type → opcode lowering:**

| RuleType | Opcodes emitted | Count |
|---|---|---|
| Strict | `Unify, Call, Halt` | 3 |
| Defeasible | `CheckDefeaters, Unify, Call, WarnOnly` | 4 |
| Defeater | `NativeUnless, Halt` | 2 |
| Linear | `NativeLinearConsume, Unify, Call, Halt` | 4 |

**Agent intent frame:**

```rust
pub struct AgentIntentFrame {
    pub intent_predicate: u64,
    pub principal_did_hash: u64,
    pub mcp_intent_frame_hash: u64,
    pub ilp_offer_micro_cents: u64,
    pub scope_count: u8,
    pub context_namespace_count: u8,
    pub requires_network: bool,
    pub output_mode: N3OutputMode,  // FreeText | N3Assertions | GraphMutation | SummarizeOnly
    pub clearance_ceiling: u8,
    pub max_sentinel_depth: u8,
    pub graph_scope: [u64; 16],
    pub context_namespaces: [u64; 16],
}
```

**Constants:**

| Constant | Value |
|---|---|
| `MAX_COMPILED_OPCODES` | 256 |
| `MAX_COMPILED_QUINS` | 64 |
| `MAX_INTENT_SCOPE_SLOTS` | 16 |
| `MAX_CONTEXT_NAMESPACE_SLOTS` | 16 |

**Tests:** 3 tests (strict rule compilation, SHACL gate rejection, zero-heap batch compile with `dhat` proof)

**Status: live**

---

### 2.3 N3Logic routing engine

**Source:** `crates/qualia-core-db/src/modalities/logic/n3logic.rs` (119 lines)

`infer_logic_bindings` is the **routing layer** that reads raw triples/rules submitted in an Agent Intent and dynamically infers which specialized logic modalities must be invoked.

**Mandatory bindings (always enforced):**

| Binding | Reason |
|---|---|
| `modality:epistemic` | Sandbox Isolator — always enforced |
| `modality:deontic` | Human-rights alignment — always enforced |

**Predicate-triggered routing:**

| Predicate hash | Binding | Purpose |
|---|---|---|
| `OP_INTENT_LOCK` (0x23) or `q42:actionPayload` | `modality:temporal` | Enforce temporal leases on intent locks |
| `q42:structuralRefactor` | `modality:epistemic:namespace_lock` | Lock namespace during refactors |
| `q42:burnsTokens` | `modality:defeasible` | Cost-overrun exceptions |
| `q42:usesPGA` | `modality:geometric_algebra` | PGA routing |
| `q42:quantumOptimize` | **Error: HardwareUnavailable** | QPU assumed offline — hard gate |
| `q42:solveCalculus` | `modality:solver:calculus` | Calculus solver routing |
| `q42:paraconsistentCheck` | `modality:paraconsistent` | Paraconsistent routing |
| `q42:spatialRegion` | `modality:spatio_temporal` | Spatial reasoning routing |
| `q42:chemistryModeling` | `modality:specialized:chemistry` | Chemistry library routing |
| `q42:physicsSimulation` | `modality:specialized:physics` | Physics library routing |
| `q42:medicalComputing` | `modality:specialized:medical` | Medical library routing |
| `q42:cryptography` | `modality:specialized:crypto` | Crypto library routing |

**Status: live**

---

### 2.4 Rules Engine

**Source:** `crates/qualia-core-db/src/modalities/logic/rules.rs` (361 lines)

The Rules Engine is the public API surface for loading N3 rules, evaluating them against live Quins through the real `SlgArena` → `execute_vm_frame` pipeline, and emitting WAL audit events.

**Pipeline:**

```
N3 text
  → n3_parser::parse_all           (parse rules)
  → RuleEngine::load_n3            (register into internal SlgArena)
  → SlgArena::fire_registered_rules (compile to norms + bytecode, execute)
  → RuleEngine::evaluate(quin)     (match quin against fired conclusions)
  → wal::log_rule_evaluation       (durable audit event)
  → Vec<RuleResult>                (public result)
```

**Key structures:**

| Struct | Purpose |
|---|---|
| `RuleEngine` | Owns rulesets + SlgArena + contract_hash |
| `RuleSet` | Named collection of rules with raw N3 source for audit/replay |
| `Rule` | Metadata (name, condition, action) — actual logic lives in VM |
| `RuleResult` | ruleset_name, rule_name, passed (bool), message |

**Evaluation semantics:**
- After `fire_registered_rules`, the arena contains deontic norms derived from rule premises
- Each norm's predicate encodes the property path (shifted left 8 bits) and a deontic opcode in the low byte
- `evaluate_silent` extracts the property path and checks whether the input Quin's (subject, predicate, object) matches any norm's premise pattern
- A rule "passes" when its norm's premise pattern matches the input Quin — meaning the rule fires for this input
- Each `evaluate` call emits a `q42:ruleEvaluation` WAL audit event

**Tests:** 5 tests (ruleset constant, creation, add, get, load_n3 + evaluate match/no-match, empty engine)

**Status: live**

---

### 2.5 SLG Arena — forward chaining with variable binding

**Source:** `crates/qualia-core-db/src/governance/webizen/arena.rs` (447 lines)

The SLG Arena is the 42MB Webizen VM that executes compiled N3 rules. It has two execution paths:

**Path 1: Ground rules → deontic norms (`fire_registered_rules`)**

Ground rules (no variables) are compiled to a single deontic norm via `compile_n3_rule_to_norm` and written to the arena table. The norm's predicate encodes the property path and deontic opcode.

**Path 2: Variable rules → forward chaining (`fire_guard_rules`)**

Variable (non-ground) rules cannot compile to a single ground norm. They are grounded by forward-chaining their premise over the facts live in the arena:

1. **Snapshot live facts** — collect up to 1024 active Quins from the arena
2. **Conjunctive backtracking join** — for each rule with variables, join the premise triples against the facts to find satisfying variable bindings
3. **Variable substitution** — for each satisfying binding, instantiate the conclusion triples with the bound values
4. **Assert conclusions** — write the newly inferred Quins to the arena
5. **Fixpoint iteration** — repeat for up to `MAX_FIXPOINT_ROUNDS = 16` rounds (a conclusion asserted in one round may satisfy another rule's premise next round)
6. **Termination** — `has_quin` idempotency guarantees termination

**Worked example — agency.n3 G1 (corporate-capture guard):**

```n3
{ ?c a values:CorporatePerson ; values:claims ?r .
  ?r a values:Right ; values:heldBy values:NaturalPerson }
  => { ?c values:flag values:PersonhoodCategoryError } .
```

Given facts that a corporate person claims a natural-person right, the guard asserts `(?c, values:flag, values:PersonhoodCategoryError)` — the personhood category error (a Deny).

**Constants:**

| Constant | Value | Purpose |
|---|---|---|
| `SLG_ARENA_SIZE` | 42 MB | Arena buffer size |
| `QUIN_SIZE` | 48 | Bytes per Quin |
| `MAX_SLOTS` | 917,504 | Maximum Quin slots |
| `RECENT_SLOT_RING` | 512 | Recent-slot ring buffer |
| `MAX_RULE_VARS` | 16 | Max variables per rule |
| `MAX_GUARD_CONCLUSIONS` | 256 | Max conclusions per fixpoint round |
| `MAX_PREMISE_DEPTH` | 16 | Recursion-depth ceiling for premise join |
| `MAX_FIXPOINT_ROUNDS` | 16 | Max forward-chaining rounds |

**Key functions:**

| Function | Purpose |
|---|---|
| `register_rule` | Register a parsed N3 rule into the VM |
| `stage_rule` | Stage a rule for later activation |
| `activate_staged_rule` | Activate a staged rule by ID |
| `fire_registered_rules` | Compile + execute all registered rules (ground + variable) |
| `fire_guard_rules` | Forward-chain variable rules to fixpoint |
| `collect_active_quins` | Snapshot live Quins from the arena |
| `has_quin` | Check if a Quin exists in the arena (idempotent) |
| `write_table` | Write a Quin to the arena table |

**Status: live**

---

### 2.6 Webizen Bytecode VM

**Source:** `crates/qualia-core-db/src/governance/webizen_bytecode.rs` (409 lines)

The Webizen Bytecode VM executes `SlgOpcode` sequences produced by the N3 compiler. It is the execution engine inside the SLG Arena.

**MSB dispatch:** MSB=1 → `did:q42` topological pointer (direct jump); MSB=0 → FNV-1a dictionary hash (lexicon lookup)

**Opcodes:**

| Opcode | Purpose |
|---|---|
| `OP_END` | End of program |
| `OP_HALT_IF_FALSE` | Conditional halt |
| `OP_MATCH_SUBJECT/PREDICATE/OBJECT` | Pattern matching against Quin fields |
| `OP_EVAL_PERMIT/OBLIGATE/FORBID` | Deontic evaluation |
| `OP_HALT_VIOLATION` | Halt on violation |

**Key structures:**

| Struct | Purpose |
|---|---|
| `VmError` | OutputBufferFull / InvalidProgram / HaltViolation |
| `GuardianshipContext` | principal_did, guardian_did |
| `ExecutionStats` | match_count, vm_cycles, direct_jump_ops, lexicon_lookup_ops |

**Key functions:** `execute_program`, `execute_program_with_stats`

**Status: live** — SIMD variant with MSB dispatch

---

### 2.7 Webizen VM (logic core)

**Source:** `crates/qualia-core-db/src/modalities/logic/core.rs` (796 lines)

The logic-core VM is a register-based VM with 16 registers and a 64-byte bytecode buffer.

**WebizenOpcode variants:**

| Opcode | Purpose |
|---|---|
| `MatchSubject/Predicate/Object` | Pattern matching |
| `EvalMetadataMask` | Metadata mask evaluation |
| `BindRegister` | Bind a register to a value |
| `MatchRegister` | Match against a register |
| `HaltIfFalse` | Conditional halt |
| `EmitQuin` | Emit a Quin to the output buffer |
| `LessThan/GreaterThan/LessOrEqual/GreaterOrEqual` | Comparison |
| `Always/Eventually/Next` | **BUGGY — not real LTL** (use `temporal_ltl.rs` instead) |
| `EmitCalculatedQuin` | Emit a computed Quin |
| `YieldConfidence` | Yield a confidence score |
| `LoadModel/EvictModel` | Model lifecycle |

**Known bug:** The `Always/Eventually/Next` opcodes compare a float threshold on a single Quin's object field — NOT real LTL. Use `temporal_ltl.rs::evaluate_ltl_trace` instead (see primary audit §2.1 LTL entry).

**Status: live** (but LTL opcodes are buggy — use `temporal_ltl.rs`)

---

## 3. Symbolic logic solvers

**Source:** `crates/qualia-core-db/src/solvers/symbolic_logic/mod.rs`

The symbolic logic solver layer provides bounded SAT and defeasible reasoning, exposed via MCP `symbolic_logic_infer`.

### 3.1 Bounded SAT Solver

| Feature | Detail |
|---|---|
| Max clauses | 50 |
| Max variables | 20 |
| Max literals per clause | 5 |
| Conflict-driven | Yes — conflict-driven clause learning |
| Variable assignments | `VariableAssignment` array (20 slots) |
| Result | `SatState { satisfiable, num_decisions, ... }` |

### 3.2 Forward Chaining Defeasible Solver

| Feature | Detail |
|---|---|
| Max rules | 100 |
| Max facts | 50 |
| Max antecedents per rule | 5 |
| Rule types | Strict, Defeasible, Defeater |
| Resolution strategies | Priority, Specificity |
| Conflict detection | `ConflictType::Contradiction / Defeat` |
| Result | `DefeasibleState { iteration, converged, ... }` |

**MCP exposure:** `symbolic_logic_infer` dispatches to either SAT or defeasible solver based on the `solver` parameter (`"sat"` or `"defeasible"`).

**Status: live**

---

## 4. SPARQL 1.1 library — full capability inventory

**Directory:** `crates/qualia-core-db/src/sparql_library/` (55 files)

The primary audit mentions SPARQL Explorer as a UI surface but does not document the full SPARQL library. This is a significant gap — the library is comprehensive.

### 4.1 Query grammar

**Source:** `sparql_grammar/` (4 files: tokenizer.rs, expr.rs, pattern.rs, update.rs)

**Supported query forms:**

| Form | Status |
|---|---|
| SELECT | live |
| ASK | live |
| CONSTRUCT | live |
| DESCRIBE | live |

**Supported WHERE clause patterns:**

| Pattern | Status |
|---|---|
| Basic triple patterns | live |
| OPTIONAL (left-join) | live |
| UNION | live |
| MINUS (anti-join) | live |
| Nested groups | live (flattened — join-associative) |
| FILTER expressions | live |
| BIND | live |
| Quoted triples (RDF-star) | live (`<< s p o >>`) |

**Supported SPARQL Update:**

| Operation | Status |
|---|---|
| INSERT DATA | live |
| DELETE DATA | live |
| DELETE/INSERT WHERE | live |
| CLEAR | live |
| CREATE | live |
| DROP | live |
| LOAD | live |

### 4.2 Query execution

**Source:** `sparql_executor.rs` (2400+ lines)

**Two execution modes:**

| Mode | Path | Use case |
|---|---|---|
| Resident executor | `QueryExecutor::new(quins)` | In-memory Quin slice |
| Range-backed executor | `Q42RangeSparqlCursor` | On-disk Q42 volume segments (zero-heap, paginated) |

**Range-backed operations:**

| Operation | Struct |
|---|---|
| Triple pattern scan | `Q42RangeTripleSelectPlan` |
| Nested-loop join | `Q42RangeNestedLoopJoinPlan` |
| Volume-set triple scan | `Q42RangeVolumeSetTripleSelectCursor` |
| Volume-set nested-loop join | `Q42RangeVolumeSetNestedLoopJoinState` |

### 4.3 Extension functions

**GeoSPARQL (`geof:`):**

| Function | Status |
|---|---|
| `geof:distance` | live (haversine great-circle distance) |
| `geof:sfContains` | live (polygon contains point/geometry) |
| `geof:sfWithin` | live (inverse of contains) |
| `geof:sfIntersects` | live (point-in-polygon, shared vertices, segment crossings) |
| `geof:sfTouches` | live (shared boundary, no interior) |

**QISP (Qualia Immersive SPARQL Protocol):**

3D/volumetric/tensor extensions that defer to `geof:` for 2D geometry. Includes mesh, volumetric, and higher-dimensional operations.

**Other builtins:** `STR`, `LANG`, `DATATYPE` (via `LiteralTable` text resolver — cold parse-time structure, not on hot path)

### 4.4 RDF format parsers (12 formats)

**Source:** `sparql_library/parsers/` (12 files)

| Format | File | Status |
|---|---|---|
| N3-star | `n3_star.rs` | live |
| NQuads-star | `nquads_star.rs` | live |
| NTriples-star | `ntriples_star.rs` | live |
| TriG-star | `trig_star.rs` | live |
| Turtle | `turtle_doc.rs` | live |
| Turtle-star | `turtle_star.rs` | live |
| JSON-LD stream | `json_ld_stream.rs` | live |
| JSON | `json_parser.rs` | live |
| CSV | `csv_parser.rs` | live |
| CBOR | `cbor_parser.rs` | live |
| CHK | `chk_parser.rs` | live |

### 4.5 RDF format serializers

**Source:** `sparql_library/serialisers/` (5 files)

| Format | Status |
|---|---|
| CSV | live |
| JSON | live |
| RDF (N-Triples/N-Quads) | live |
| SPARQL Results XML/JSON | live |

### 4.6 Additional SPARQL modules

| Module | File | Purpose | Status |
|---|---|---|---|
| Aggregates | `sparql_aggregates.rs` | GROUP BY, COUNT, SUM, AVG, MIN, MAX | live |
| Federated | `sparql_federated.rs` | SERVICE clause — federated queries | live |
| WebSocket | `sparql_websocket.rs` | SPARQL over WebSocket | live |
| SHACL validation | `sparql_shacl.rs` | SHACL validation via SPARQL | live |
| DID handling | `sparql_did.rs` | DID resolution in SPARQL | live |
| GeoSPARQL | `geosparql.rs` | WKT parsing, geometry predicates | live |
| Immersive | `immersive/` (4 files) | QISP 3D/volumetric functions | live |
| External sort | `external_sort.rs` | Large-dataset sorting | live |
| Planner | `sparql_planner.rs` | Physical plan optimization | live |
| Filter | `sparql_filter.rs` | FILTER expression evaluation | live |
| Update | `sparql_update.rs` | SPARQL Update execution | live |
| Endpoint | `sparql_endpoint.rs` | SPARQL endpoint server | live |
| Vision SHACL | `vision_shacl.rs` | Computer vision SHACL shapes | live |

**Status: live** — comprehensive SPARQL 1.1 implementation

---

## 5. OWL RL materialization

**Source:** `crates/qualia-core-db/src/modalities/logic/owl/` (3 files)

### 5.1 OWL RL rules materialization

**Source:** `owl/materialize.rs` (594 lines)

`materialize_owl_rl` computes the OWL RL closure over a working set of triples:

| Feature | Detail |
|---|---|
| Input | `&mut [RdfTriple]` (caller-buffered, up to `initial_len` populated) |
| Output | `MaterializeSummary { triple_count, inferred_count, contradictions[] }` |
| Property chains | `ChainAxiom { composed, first, second }` — 2-step chains |
| Disjointness | `DisjointnessViolation { individual, class_a, class_b }` — recorded, not fatal |
| Error handling | `MaterializeError::WorkingSetFull / ContradictionBufferFull` |
| Closure | Isolates contradictions (records and continues) rather than halting |

### 5.2 OWL → SHACL conversion

**Source:** `owl/shacl_convert.rs` (729 lines)

| Function | Purpose |
|---|---|
| `parse_healthcare_owl_turtle` | Parse Turtle healthcare vocabulary |
| `parse_healthcare_owl_n3` | Parse N3 healthcare excerpt |
| `parse_healthcare_owl_lines` | Line-oriented parser (handles truncated tails, `unionOf` domains) |
| `parse_healthcare_owl` | Auto-detect format (.n3 or .ttl) |
| `healthcare_owl_to_shacl_ttl` | Emit SHACL node shapes for healthcare IE.* classes and DICOM-tag properties |
| `parse_radlex_relations_xml` | Stream-parse RadLex Pun OWL (RDF/XML) relation axioms |
| `radlex_relations_to_shacl_ttl` | Group RadLex relations by subject RID and emit SHACL node shapes |
| `write_anatomy_shape_bundle` | Write agency baseline + converted ontologies to output directory |

**Key structures:**

| Struct | Purpose |
|---|---|
| `HealthcareOwlModel` | classes (BTreeMap), properties (BTreeMap) |
| `OwlClass` | labels, equivalent class |
| `OwlProperty` | labels, kinds, domains, ranges |
| `RadlexRelation` | subject_rid, predicate_local, object_rid |

**Status: live**

---

## 6. Prolog-like capability comparison

### 6.1 What exists (Prolog-like)

| Prolog feature | QualiaDB equivalent | Status |
|---|---|---|
| Rule definition | N3 rules (`=>`, `~>`, `^>`, `-o`) | live |
| Facts | Quins in the SLG Arena / graph | live |
| Forward chaining | `fire_guard_rules` (fixpoint, bounded) | live |
| Variable binding | Conjunctive backtracking join | live |
| Unification | `Unify` SlgOpcode | live |
| Defeasible reasoning | `~>` / `^>` rules + `CheckDefeaters` opcode | live |
| Linear logic | `-o` rules + `NativeLinearConsume` opcode | live |
| Negation as failure | `NativeUnless` opcode (defeater) | live |
| Cut / commitment | `Halt` opcode | live |
| Rule weights | `(0.8)` annotation | live |
| Rule IDs | `[rule_id]` annotation | live |
| Formula quoting | `{ ... }` as `Term::Formula` | live |
| Pattern matching | `MatchSubject/Predicate/Object` opcodes | live |
| Register binding | `BindRegister` / `MatchRegister` opcodes | live |
| Comparison | `LessThan/GreaterThan/LessOrEqual/GreaterOrEqual` opcodes | live |
| Confidence scoring | `YieldConfidence` opcode | live |
| Audit trail | WAL `q42:ruleEvaluation` events | live |

### 6.2 What does NOT exist (Prolog gaps)

| Prolog feature | QualiaDB status | Impact |
|---|---|---|
| **Backward chaining** (goal-directed resolution) | **Not implemented** | The engine is forward-chaining only. No way to query "prove X" and get a proof tree. |
| **N3 builtins** (math:sum, string:concat, list:member, log:includes, log:conclusion) | **Not implemented** | N3 parser handles the syntax but there are no builtin function implementations. Rules cannot call builtins. |
| **Proof trees / derivation traces** | **Not implemented** | Forward chaining asserts conclusions but does not record the derivation chain. No way to ask "why was this inferred?" |
| **Constraint Logic Programming (CLP)** | **Not implemented** | No constraint propagation over domains. |
| **Tabling / memoization** | **Not implemented** | No result caching for subgoals. |
| **Cut operator (!)** | **Not implemented** | No explicit cut for controlling backtracking. The `Halt` opcode stops execution but is not a Prolog cut. |
| **Dynamic database assert/retract** | **Partial** | `write_table` asserts; no retract. Staged rules can be activated but not deactivated. |
| **Multi-triple premises** | **Limited** | `MAX_STACK_TRIPLES = 8` — premises with more than 8 triples are truncated. The join logic handles multi-triple premises but the parser truncates. |
| **N3 lists** (`(a b c)`) | **Not parsed** | The N3 parser does not parse N3 list syntax. |
| **N3 paths** (`!`, `^`) | **Not parsed** | The N3 parser does not parse N3 property path syntax. |
| **N3 keywords** (`is`, `of`, `has`) | **Not parsed** | The N3 parser does not support these N3 keywords. |
| **N3 quantifiers** (`@forAll`, `@forSome`) | **Not parsed** | The N3 parser does not support universal/existential quantification. |

### 6.3 Architectural distinction from Prolog

QualiaDB's reasoning engine is **not Prolog**. It is a **forward-chaining rule engine with deontic/defeasible/linear logic extensions** that compiles N3 rules to bytecode for execution in a bounded 42MB arena. Key differences:

1. **No SLD resolution** — Prolog's backward-chaining SLD resolution is absent. The engine uses forward chaining (semi-naive fixpoint) instead.
2. **No proof trees** — Conclusions are asserted as facts, not as derivations with provenance to the rules that fired them.
3. **Bounded execution** — All execution is bounded by fixed-size buffers and iteration caps (`MAX_FIXPOINT_ROUNDS = 16`, `MAX_PREMISE_DEPTH = 16`). Prolog is unbounded.
4. **Deontic/defeasible/linear** — The engine extends beyond classical logic with deontic norms, defeasible overrides, and linear resource consumption. Prolog is classical-only.
5. **SHACL firewall** — Rules are validated against SHACL shapes before compilation. Prolog has no such type system.
6. **Zero-heap hot path** — The forward-chaining join uses fixed stack buffers. Prolog typically uses heap-based environments.

---

## 7. MCP and Poet host integration (supplementary detail)

### 7.1 MCP logic tools — full dispatch

**Source:** `crates/qualia-core-db/src/mcp/mcp_tool_impls/logic.rs` (395 lines)

| Tool | Solver | Modalities |
|---|---|---|
| `symbolic_logic_infer` | `BoundedSatSolver` or `ForwardChainingDefeasible` | SAT, defeasible |
| `evaluate_modality` | Direct modality dispatch | LTL, ASP, probabilistic, argumentation, deontic, epistemic, DL, paraconsistent |
| `evaluate_logic_rules` | `RuleEngine` → `SlgArena` → `execute_vm_frame` | N3 rules → norms → bytecode → execution + WAL audit |

### 7.2 Poet host logic invoke — full module list

**Source:** `crates/qualia-core-db/src/poet_host/invoke/logic/` (8 files)

| File | Lines | Exports | Function |
|---|---|---|---|
| `mod.rs` | 17 | Re-exports | asp_enumerate, caused, t_norm, deontic_evaluate, epistemic_evaluate, ltl_finally, ltl_globally, paraconsistent_route, subsumption_check |
| `causal.rs` | 77 | `caused`, `t_norm` | Causal reachability, Gödel t-norm |
| `deontic.rs` | 47 | `evaluate` | Deontic contract scan on live graph slice |
| `ltl.rs` | 27 | `finally`, `globally` | LTL eventually/always |
| `subsumption.rs` | 26 | `check` | DL subsumption |
| `paraconsistent.rs` | 23 | `route` | Paraconsistent routing |
| `epistemic.rs` | 21 | `evaluate` | Epistemic frame evaluation |
| `asp.rs` | 19 | `enumerate` | ASP answer set enumeration |

All modules feature-gated for `wasm-logic`/`wasm-scientific`/`wasm-full`.

### 7.3 Desktop Tauri commands — full list

**Source:** `crates/webizen-desktop/src/commands/semantic_logic.rs` (100 lines)

| Command | What it does |
|---|---|
| `fetch_domain_ontology(domain_id)` | Fetches domain ontology schema via OntologyCompiler |
| `execute_sparql_query(query)` | Executes local SPARQL over resident graph |
| `validate_shacl_shape(node, shape_uri)` | Validates local SHACL shape |
| `evaluate_logic_rules(input)` | Loads N3 rules, fires in Webizen VM, evaluates Quin, returns pass/fail verdicts |

---

## 8. UI surfaces required for N3Logic/Prolog-like functionality

### 8.1 Tier 1 — Core rule authoring (P0)

| Surface | Engine source | Desktop command | qualia-ui state |
|---|---|---|---|
| **N3 Logic Studio** | `n3_parser.rs`, `n3_compiler.rs`, `rules.rs` | `evaluate_logic_rules` | **missing** |
| **N3 Rule Editor** with syntax highlighting | `n3_parser.rs` | — | **missing** |
| **N3 Compile Viewer** (show opcodes) | `n3_compiler.rs::compile_rule_to_opcodes` | — | **missing** |
| **SHACL Validator** | `shacl_compiler.rs`, `validate.rs` | `validate_shacl_shape` | **missing** |
| **SPARQL Explorer** (SELECT/ASK/CONSTRUCT/UPDATE) | `sparql_library/` | `execute_sparql_query` | **missing** |
| **SPARQL Update Editor** | `sparql_grammar/update.rs` | — | **missing** |
| **GeoSPARQL Query Builder** | `geosparql.rs` | — | **missing** |
| **RDF Format Importer** (12 formats) | `sparql_library/parsers/` | — | **missing** |
| **RDF Format Exporter** | `sparql_library/serialisers/` | — | **missing** |
| **Evaluate Modality panel** | All modalities | MCP `evaluate_modality` | **missing** |
| **Symbolic Logic Infer** (SAT + defeasible) | `solvers/symbolic_logic/` | MCP `symbolic_logic_infer` | **missing** |

### 8.2 Tier 2 — Ontology and materialization (P1)

| Surface | Engine source | qualia-ui state |
|---|---|---|
| **OWL RL Materializer** | `owl/materialize.rs` | **missing** |
| **OWL → SHACL Converter** | `owl/shacl_convert.rs` | **missing** |
| **Healthcare OWL Importer** | `parse_healthcare_owl`, `healthcare_owl_to_shacl_ttl` | **missing** |
| **RadLex Relation Importer** | `parse_radlex_relations_xml`, `radlex_relations_to_shacl_ttl` | **missing** |
| **Ontology Builder** | `shacl_types.rs`, `owl/` | **missing** |

### 8.3 Tier 3 — Execution inspection (P2)

| Surface | Engine source | qualia-ui state |
|---|---|---|
| **SLG Arena Inspector** (slot usage, rule registry, fixpoint rounds) | `arena.rs` | **missing** |
| **Webizen VM Inspector** (opcode trace, register state, execution stats) | `webizen_bytecode.rs`, `core.rs` | **missing** |
| **N3Logic Routing Inspector** (show inferred modality bindings) | `n3logic.rs::infer_logic_bindings` | **missing** |
| **Forward Chaining Trace** (show fixpoint rounds, variable bindings) | `arena.rs::fire_guard_rules` | **missing** |
| **WAL Rule Evaluation Log** | `wal::log_rule_evaluation` | **missing** |
| **Agent Intent Frame Inspector** | `n3_compiler.rs::AgentIntentFrame` | **missing** |

### 8.4 Tier 4 — Advanced reasoning (P1)

| Surface | Engine source | qualia-ui state |
|---|---|---|
| **Defeasible Rule Editor** (distinct from N3) | `defeasible.rs`, `solvers/symbolic_logic/` | **missing** |
| **ASP Answer Set Explorer** | `asp.rs` | **missing** |
| **SAT Solver Interface** | `solvers/symbolic_logic/::BoundedSatSolver` | **missing** |
| **Defeasible Forward Chaining Interface** | `solvers/symbolic_logic/::ForwardChainingDefeasible` | **missing** |

---

## 9. N3 rule authoring UI specification

### 9.1 N3 Logic Studio — recommended component architecture

```
N3LogicStudio
├── RuleEditor (textarea with syntax highlighting)
│   ├── Rule type indicator (=> / ~> / ^> / -o)
│   ├── Variable highlighting (?x, ?y)
│   ├── Formula quoting visualization ({ ... })
│   └── Parse error display
├── CompilePanel
│   ├── SHACL validation results
│   ├── Compiled opcode listing (Unify, Call, Halt, etc.)
│   ├── Compiled norm Quin display
│   └── Compilation errors
├── EvaluatePanel
│   ├── Input Quin editor (subject, predicate, object, context)
│   ├── Execute button → evaluate_logic_rules
│   ├── Rule results (pass/fail per rule)
│   └── WAL audit event display
├── ForwardChainingTrace
│   ├── Fixpoint round counter (1-16)
│   ├── Variable bindings per round
│   ├── Newly asserted conclusions per round
│   └── Termination reason (fixpoint reached / round cap)
└── RuleSetManager
    ├── Load N3 from file/text
    ├── Named ruleset list
    ├── Rule count per ruleset
    └── Raw N3 source viewer (audit/replay)
```

### 9.2 SPARQL Explorer — recommended component architecture

```
SparqlExplorer
├── QueryEditor (textarea with SPARQL syntax highlighting)
│   ├── Query type indicator (SELECT/ASK/CONSTRUCT/UPDATE)
│   ├── Prefix manager
│   └── Parse error display
├── ResultPanel
│   ├── SELECT results table (variable bindings)
│   ├── ASK boolean result
│   ├── CONSTRUCT triple list
│   └── Update operation result
├── QueryPlanViewer
│   ├── Physical plan tree
│   ├── Operator types (scan, join, filter, etc.)
│   └── Estimated vs actual cardinalities
├── GeoSPARQLPanel
│   ├── WKT geometry input
│   ├── Map visualization
│   └── geof: function results
├── FormatImporter
│   ├── Format selector (12 formats)
│   ├── File upload / paste
│   └── Parsed triple count
└── FormatExporter
    ├── Format selector (4 formats)
    ├── Result set or graph export
    └── Download
```

---

## 10. Known gaps and limitations (N3Logic-specific)

### 10.1 No backward chaining

The engine is **forward-chaining only**. There is no goal-directed resolution. A UI that wants to answer "prove X" queries would need a new backward-chaining engine or a translation from goal queries to forward-chaining rules.

**UI impact:** The N3 Logic Studio should present rules as "when these facts exist, assert these conclusions" — not as "to prove X, try these rules."

### 10.2 No N3 builtins

N3 builtins (`math:sum`, `string:concat`, `list:member`, `log:includes`, `log:conclusion`) are not implemented. Rules cannot call functions.

**UI impact:** The rule editor should document this limitation. A future builtin implementation would require extending the N3 compiler to emit `SlgOpcode::NativeBuiltin` calls.

### 10.3 No proof trees

Forward chaining asserts conclusions but does not record the derivation chain. There is no way to ask "why was this inferred?" or "which rules fired to produce this conclusion?"

**UI impact:** The Forward Chaining Trace should show which rules fired in each round and what conclusions were asserted, but cannot show a complete proof tree. A future proof-tree feature would require recording the rule → conclusion mapping during forward chaining.

### 10.4 Multi-triple premise truncation

The N3 parser truncates premises to `MAX_STACK_TRIPLES = 8` triples. Complex rules with more than 8 premise triples will be silently truncated.

**UI impact:** The rule editor should warn when a premise exceeds 8 triples.

### 10.5 No N3 list/path/keyword syntax

The N3 parser does not support:
- N3 list syntax: `(a b c)`
- N3 path syntax: `!` (forward), `^` (inverse)
- N3 keywords: `is`, `of`, `has`
- N3 quantifiers: `@forAll`, `@forSome`

**UI impact:** The rule editor should document supported syntax. Unsupported syntax will cause parse errors.

### 10.6 SPARQL filter scoping simplification

FILTER and BIND scoping over a group is simplified — both apply over the whole group's join result, not per-pattern. This is a deliberate simplification.

**UI impact:** The SPARQL Explorer should document this. Complex FILTER scoping may not match standard SPARQL 1.1 semantics exactly.

---

## 11. Summary

The N3Logic ecosystem is substantially more capable than the primary audit documented. The full inventory includes:

| Component | Files | Lines | Status |
|---|---|---|---|
| N3 Parser | 1 | 1,027 | live |
| N3 Compiler | 1 | 448 | live |
| N3Logic routing | 1 | 119 | live |
| Rules Engine | 1 | 361 | live |
| SLG Arena | 1 | 447 | live |
| Webizen Bytecode VM | 1 | 409 | live |
| Webizen VM (logic core) | 1 | 796 | live |
| Symbolic logic solvers | 1 | 800+ | live |
| SPARQL 1.1 library | 55 | 15,000+ | live |
| OWL RL materialization | 3 | 1,300+ | live |
| **Total** | **66** | **~20,000+** | |

The engine provides:
- **Forward chaining** with variable binding and fixpoint iteration (Prolog-like, but not backward chaining)
- **4 rule types** (strict, defeasible, defeater, linear) — beyond classical Prolog
- **SHACL firewall** — rules validated against shapes before compilation
- **Zero-heap hot path** — forward chaining uses fixed stack buffers
- **Full SPARQL 1.1** — SELECT/ASK/CONSTRUCT/UPDATE with GeoSPARQL, federated, WebSocket
- **12 RDF format parsers** and 4 serializers
- **OWL RL materialization** with property chains and disjointness detection
- **OWL → SHACL conversion** for healthcare and RadLex ontologies

The engine does **not** provide:
- Backward chaining (goal-directed resolution)
- N3 builtins (math:sum, string:concat, log:includes, etc.)
- Proof trees / derivation traces
- N3 list/path/keyword/quantifier syntax
- Constraint Logic Programming
- Tabling / memoization

The UI gap is total: `qualia-ui` has **zero** surfaces for any of these capabilities. The critical path is:
1. **N3 Logic Studio** — wire `evaluate_logic_rules` for real rule authoring
2. **SPARQL Explorer** — wire `execute_sparql_query` with IRI resolution
3. **SHACL Validator** — wire `validate_shacl_shape`
4. **OWL Materializer** — wire `materialize_owl_rl`
5. **Evaluate Modality panel** — wire MCP `evaluate_modality`
6. **Symbolic Logic Infer** — wire MCP `symbolic_logic_infer`
7. **SLG Arena Inspector** — show fixpoint rounds and variable bindings
8. **Forward Chaining Trace** — show rule firing and conclusion assertion

---

*Generated with [Devin](https://devin.ai) for Timothy Charles Holborn. Supplementary audit performed 2026-08-18 against QualiaDB branch 0.0.31-dev.*
