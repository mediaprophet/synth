# VibeScript & Poet Engine: Technical Specification
> *"Poet: Define your mindware, with VibeScript"*

**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.  
**Principal / inventor:** Timothy Charles Holborn &lt;timothy.holborn@gmail.com&gt;  
Assignment: [`COPYRIGHT.md`](../COPYRIGHT.md) · Licence: [`LICENSE`](../LICENSE)

**Status:** Architectural essay (non-normative).  
**Normative language for implementation:** [`vibescript-core.md`](vibescript-core.md) (`vibe-0.1`).  
Do not treat this file’s “v1.0” heading or examples as a parser contract.  
**Target Runtimes:** QualiaDB Microkernel, Webizen Desktop, Browser WASM, Mobile FFI, Edge IoT  

---

## 1. Vision & Architectural Philosophy

**VibeScript** (or **Vibe**, `.vibe`) is a typed, deterministic, and capability-bounded domain-specific language (DSL) evaluated by the **Poet Engine** in Rust/WASM. It is designed for **dynamic hypermedia documents, reactive semantic mindware, spatiotemporal reasoning, 3D kinematic simulation, and decentralized graph automation** within the QualiaDB / Webizen ecosystem.

### Core Architectural Invariants:
1. **Small, Stable Language Core:** The grammar is intentionally minimal, mathematical, and closed. It provides standard control flow, static typing, and generic postfix member invocation (`expr.method(args)`). 
2. **Dynamic Capability Registry:** Domain capabilities (`graph`, `pulse`, `aura`, `anim`, `rig`, `ik`, `physics`, `motor`, `codecs`) are **not hardcoded keywords** in the grammar. They are dynamically registered and resolved through QualiaDB's **Capability Registry**, allowing new native system packages (IPC daemons, WASM components) to be hot-loaded without grammar changes.
3. **No Ambient Authority (Fiduciary Security):** Scripts declare required capabilities at the module header (`requires [ ... ]`). The host Sentinel VM enforces strict capability, deontic, and sensitivity gates prior to execution.
4. **Pure Linked Data Integration:** Native first-class values for W3C RDF 1.2 triple terms (`<<( s p o )>>`) and reified statements (`<< s p o ~ reifier >>`). Qualia 48-byte nquins are produced **only** by `quin.statement(...)` — raw `<<[ s p o g prov ]>>` overlay literals are **forbidden** (core §13 N3).
5. **No JIT Dependency:** Pure safe-Rust AST / bytecode interpreter ensuring deterministic execution and 100% compliance with iOS App Store and resource-constrained edge environments.

---

## 2. 3-Tier Microkernel Architecture

```
┌──────────────────────────────────────────────────────────────────────────┐
│                   Tier 1: Poet Workbench & Hypermedia UI                 │
│  - Authoring: yaml-ld-q42 documents, 3D meshes (D10), tensors (p64)      │
│  - Web Components: <q-doc>, <q-entity>, <q-relation>, <q-cell>, <q-event>│
│  - Legacy Web Partition: Sandboxed <q-web-frame> hosting www/ assets     │
└────────────────────────────────────┬─────────────────────────────────────┘
                                     │ DOM CustomEvents & Reactive Cell Triggers
┌────────────────────────────────────▼─────────────────────────────────────┐
│                 Tier 2: Poet Engine (VibeScript Runtime)                 │
│  - Small, Typed AST & Bytecode Evaluator (Zero-JIT, Safe Rust)           │
│  - Static Effect Checking: Pure, HotZeroHeap, ColdSync, Async, External   │
│  - Execution Budgets: Instruction Gas, Call Depth, and Timeout Limiting  │
│  - Cooperative Multitasking: Async `await`/`yield` & Transaction Blocks   │
└────────────────────────────────────┬─────────────────────────────────────┘
                                     │ Typed Capability Calls & Handle Passing
┌────────────────────────────────────▼─────────────────────────────────────┐
│             Tier 3: QualiaDB Microkernel & Capability Registry           │
│  - Fiduciary Sentinel: Sensitivity gating, DID signatures, rights scopes │
│  - Core Storage: Q42 Merkle-CRDT DAG with State-Based Snapshot Compaction│
│  - Compute Engines: WGSL Forge GPU compute, PGA Motor kinematics, Audio   │
│  - Extension Host: Hot-loadable child process daemons (IPC / zero-copy)  │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Formal EBNF Grammar Specification (v1.0)

```ebnf
(* ===========================================================================
   VibeScript Formal Grammar (v1.0-draft)
   Generic Postfix Syntax with Typed Values & RDF 1.2 Semantic Literals
   =========================================================================== *)

Program           ::= ModuleDecl? ImportDecl* PrefixDecl* RequiresDecl? TopLevelItem* ;

(* --- Module & Capability Declarations --- *)
ModuleDecl        ::= 'module' (IRI | Identifier) ';' ;
ImportDecl        ::= 'import' (IRI | StringLiteral) ('as' Identifier)? ';' ;
PrefixDecl        ::= 'prefix' PrefixName ':' IRI ';' ;
RequiresDecl      ::= 'requires' '[' CapabilityList? ']' ';' ;
CapabilityList    ::= CapabilitySpec (',' CapabilitySpec)* ;
CapabilitySpec    ::= 'capability' '(' StringLiteral (',' PropertyList)? ')' ;

(* --- Top-Level Items --- *)
TopLevelItem      ::= FunctionDecl
                    | HookDecl
                    | ConstDecl
                    | Statement ;

ConstDecl         ::= 'const' Identifier (':' TypeAnnotation)? '=' Expression ';' ;
FunctionDecl      ::= 'async'? 'fn' Identifier '(' ParameterList? ')' ('->' TypeAnnotation)? Block ;
HookDecl          ::= 'on' (Identifier | EventPath) '(' ParameterList? ')' Block ;
EventPath         ::= Identifier (':' Identifier)* ;

ParameterList     ::= Parameter (',' Parameter)* ;
Parameter         ::= Identifier ':' TypeAnnotation ;

(* --- Statements --- *)
Statement         ::= VariableDecl
                    | Assignment
                    | ReactiveCellFormula
                    | TransactionStmt
                    | ControlFlow
                    | Expression ';' ;

VariableDecl      ::= 'let' 'mut'? Identifier (':' TypeAnnotation)? ('=' Expression)? ';' ;
Assignment        ::= PostfixExpr '=' Expression ';' ;
ReactiveCellFormula ::= '=' Expression ;
TransactionStmt   ::= 'transaction' ('(' ArgumentList? ')')? Block ;

(* --- Control Flow --- *)
Block             ::= '{' Statement* '}' ;
ControlFlow       ::= IfStatement | ForLoop | WhileLoop | ReturnStatement | YieldStatement ;
IfStatement       ::= 'if' Expression Block ('else' (IfStatement | Block))? ;
ForLoop           ::= 'for' Identifier 'in' Expression Block ;
WhileLoop         ::= 'while' Expression Block ;
ReturnStatement   ::= 'return' Expression? ';' ;
YieldStatement    ::= 'yield' Expression? ';' ;

(* --- Expressions --- *)
Expression        ::= AwaitExpr ;
AwaitExpr         ::= 'await'? LogicalOrExpr ;
LogicalOrExpr     ::= LogicalAndExpr ('||' LogicalAndExpr)* ;
LogicalAndExpr    ::= EqualityExpr ('&&' EqualityExpr)* ;
EqualityExpr      ::= RelationalExpr (('==' | '!=') RelationalExpr)* ;
RelationalExpr    ::= AdditiveExpr (('<' | '<=' | '>' | '>=') AdditiveExpr)* ;
AdditiveExpr      ::= MultiplicativeExpr (('+' | '-') MultiplicativeExpr)* ;
MultiplicativeExpr::= UnaryExpr (('*' | '/' | '%') UnaryExpr)* ;
UnaryExpr         ::= ('!' | '-' | '+')? PostfixExpr ;

(* --- Generic Postfix Chain (Extensibility Core) --- *)
PostfixExpr       ::= PrimaryExpr (MemberAccess | CallExpr | IndexAccess | TryOperator)* ;
MemberAccess      ::= '.' Identifier ;
CallExpr          ::= '(' ArgumentList? ')' ;
IndexAccess       ::= '[' Expression ']' ;
TryOperator       ::= '?' ;

ArgumentList      ::= Argument (',' Argument)* ;
Argument          ::= (Identifier ':')? Expression ;

(* --- Primary Expressions & Literals --- *)
PrimaryExpr       ::= Literal
                    | Identifier
                    | QueryVariable
                    | IRI
                    | TripleTerm
                    | ReifiedTriple
                    | NquinTerm
                    | ListLiteral
                    | RecordLiteral
                    | '(' Expression ')' ;

(* --- W3C RDF 1.2 & Qualia Graph Primitives --- *)
TripleTerm        ::= '<<(' Subject Predicate Object ')>>' ;
ReifiedTriple     ::= '<<' Subject Predicate Object ('~' ReifierId)? '>>' ('[' PropertyList ']')? ;
NquinTerm         ::= '<<[' Subject Predicate Object GraphContext ProvenanceTag? ']>>' ; (* FORBIDDEN — see core §13 N3. Use quin.statement(...) instead. *)

Subject           ::= IRI | BlankNode | TripleTerm | Identifier ;
Predicate         ::= IRI | Identifier ;
Object            ::= IRI | BlankNode | Literal | TripleTerm | Identifier ;
GraphContext      ::= IRI | BlankNode | Identifier ;
ProvenanceTag     ::= StringLiteral | Identifier ;
ReifierId         ::= IRI | BlankNode | Identifier ;

QueryVariable     ::= '?' Identifier ;
PropertyList      ::= PropertyPair (',' PropertyPair)* ;
PropertyPair      ::= (Identifier | StringLiteral) ':' Expression ;

ListLiteral       ::= '[' (Expression (',' Expression)*)? ']' ;
RecordLiteral     ::= '{' (PropertyPair (',' PropertyPair)*)? '}' ;

(* --- Types & Annotations --- *)
TypeAnnotation    ::= BasicType | GenericType | HandleType ;
BasicType         ::= 'i32' | 'u32' | 'i64' | 'u64' | 'f32' | 'f64' | 'string' | 'bool' | 'bytes' ;
HandleType        ::= 'Iri' | 'BlankNode' | 'Did' | 'Hash' | 'TripleTerm' | 'Quin' | 'QuinRef' | 'TensorRef' | 'AssetRef' | 'Motor' ;
GenericType       ::= ('Option' | 'Result' | 'List' | 'Map') '<' TypeAnnotation (',' TypeAnnotation)* '>' ;

(* --- Lexical Elements & Terminals --- *)
IRI               ::= '<' [^>]+ '>' | PName ;
PName             ::= (PrefixName)? ':' LocalName ;
PrefixName        ::= [a-zA-Z_][a-zA-Z0-9_-]* ;
LocalName         ::= [a-zA-Z_][a-zA-Z0-9_.-]* ;
BlankNode         ::= '_:' [a-zA-Z0-9_-]+ ;

Literal           ::= NumericLiteral | StringLiteral | BooleanLiteral | 'null' ;
NumericLiteral    ::= [0-9]+ ('.' [0-9]+)? ([eE][+-]?[0-9]+)? ;
StringLiteral     ::= '"' ([^"\\] | '\\' .)* '"' | '`' ([^`\\] | '\\' .)* '`' ;
BooleanLiteral    ::= 'true' | 'false' ;
Identifier        ::= [a-zA-Z_][a-zA-Z0-9_]* ;

(* --- Comments --- *)
Comment           ::= LineComment | BlockComment ;
LineComment       ::= '//' [^\r\n]* ;
BlockComment      ::= '/*' ([^*] | '*' [^/])* '*/' ;
```

---

## 4. Value Model & Resource Handles

To prevent precision loss when interacting with 64-bit QualiaDB structures, VibeScript enforces explicit value types:

| Value Family | Representation | Semantic Role |
| :--- | :--- | :--- |
| **Numeric Scalars** | `i32`, `u32`, `i64`, `u64`, `f32`, `f64` | Explicit width and signedness; prevents JS `Number` precision truncation on 64-bit hashes and timestamps. |
| **Semantic Identifiers** | `Iri`, `BlankNode`, `Did`, `Hash` | Strongly typed; prevents mixing up raw strings with cryptographically verified DIDs or RDF nodes. |
| **RDF 1.2 Statements** | `TripleTerm`, `ReifiedStatement`, `Quin` | W3C RDF 1.2 proposition terms and 48-byte Qualia Nquins with character-span provenance. |
| **Opaque Resource Handles** | `QuinRef`, `TensorRef`, `GeometryRef`, `AssetRef` | Validated memory handles pointing to zero-copy shared memory or GPU buffers; scripts never copy raw 4K video or multi-gigabyte tensors into user-space RAM. |
| **Kinematic Types** | `Motor` | 64-byte $\mathbb{R}_{3,0,1}$ Projective Geometric Algebra Motor for singularity-free 3D screw motions. |
| **Control Types** | `Option<T>`, `Result<T, E>` | Explicit, panic-free error handling with the `?` try-operator. |

---

## 5. Static Effect Checking & Execution Limits

VibeScript functions and event handlers declare their effect class, validated at load time:

1. **`Pure`:** Strictly deterministic, side-effect free evaluation (used for reactive spreadsheet cells `=COUNT(...)`).
2. **`HotZeroHeap`:** Fixed-buffer execution over caller-owned memory (real-time 60fps/120fps `on_tick` loop and DSP callbacks).
3. **`ColdBoundedSync`:** Synchronous construction within a declared memory limit.
4. **`AsyncRequired`:** Heavy background computations (model inference, spatial mesh generation) returning `Future<T>`.
5. **`ExternalEffect`:** Modifying graph vaults, network broadcast, hardware actuation, or spending transaction gas.

### Strict Execution Safeguards:
- **Instruction Gas Metering:** Every AST step deducts gas; loops must be bounded.
- **Maximum Stack Depth:** Hard limit (e.g. 64 frames) prevents recursion stack overflow.
- **Memory Allocation Ceiling:** Scripts are bound to a strict memory budget (e.g. 16MB default in WASM).

---

## 6. Idiomatic VibeScript Mindware Examples

### A. Declaring Capabilities & Reacting to a Telemetry Pulse
```vibe
module <https://qualiadb.org/modules/clinical_monitor>;

requires [
    capability("pulse:network:broadcast", scope: "topic:alerts"),
    capability("aura:ontology:mutate", schema: "snomed:ClinicalStatus")
];

// Handles incoming IoT or peer telemetry stream
on pulse:stream(stream: Stream<TelemetryReading>) {
    let reading = stream.read_next()?;
    
    if reading.value > 85.0 {
        // Broadcast cryptographically signed alert pulse
        pulse.publish("topic:alerts", {
            alert_type: "ThresholdExceeded",
            reading_value: reading.value,
            timestamp: time.now()
        })?;
        
        // Update entity status in the local Q42 graph
        let clinic = graph.get_node(<qualia:Clinic_A>)?;
        aura.validate(clinic, "snomed:EmergencyStatus")?;
    }
}
```

### B. W3C RDF 1.2 Claims & Cryptographic Provenance
```vibe
// 1. Abstract proposition term (unasserted claim)
let proposition = <<( <snomed:DrugX> <qualia:treats> <snomed:ConditionY> )>>;

// 2. Asserted proposition with cryptographic provenance and reifier
let asserted_fact = << <snomed:DrugX> <qualia:treats> <snomed:ConditionY> ~ <qualia:claim_42> >> [
    confidence: 0.98,
    author: did.current_user(),
    created_at: time.now()
];

transaction {
    graph.stage(asserted_fact)?;
    graph.commit()?;
}
```

### C. Game Engine–Grade 3D Animation & Procedural IK
```vibe
// Real-time 60fps/120fps simulation loop: Look-at target & procedural foot placement
on tick(dt: f32) {
    let player = space.get_actor("player")?;
    let target_pos = player.get_head_position();
    
    // Solve procedural look-at inverse kinematics
    ik.look_at("mesh:avatar", {
        head_bone: "head",
        target: target_pos,
        clamp_angle: 65.0,
        smooth_time: 0.15
    });
    
    // Smooth camera trailing with PGA spring-damper
    camera.motor = physics.smooth_damp(camera.motor, player.motor, dt, 0.2);
}

// Synchronized AnimNotify event emitted from exact .d10 animation keyframe
on anim_event(event_name: string, payload: Record) {
    if event_name == "footstep_impact" {
        pulse.publish("audio:footstep", {
            surface: payload.surface_type,
            intensity: payload.impact_force,
            pos: payload.foot_pos
        });
    }
}
```

### D. Generative Articulatory Speech Synthesis (Cross-Modal Pulse)
```vibe
// Synthesize speech from an unencoded phonological graph through a .d10 vocal tract
on event:speak_query(intent: ConceptIntent) {
    let phonemes = graph.resolve_phonology(intent.concept, "lang:example_unencoded")?;
    let vocal_tract = hmc.get_mesh("asset://tracts/avatar_voice.d10")?;
    
    // Computes real-time acoustic wave and generates mathematically exact 3D lip-sync
    // audio.speak is a later capability.invoke binding, not 0.1
    capability.invoke("audio.speak", { phonemes: phonemes, tract: vocal_tract, emotion: "warm", pitch_contour: 1.05 })?;
}
```

---

## 7. Multi-Platform Deployment & Runtime Profile Matrix

| Runtime Profile | Host Runtime & Hardware Tier | Poet Engine Mode | Rendering & GPU Pipeline | Storage & Wire Protocol |
| :--- | :--- | :--- | :--- | :--- |
| **Profile A: Native Desktop** | **64-bit Native Rust AOT**, full CPU SIMD (AVX-512) | Embedded Poet AST / Bytecode Interpreter | **Direct WGPU 29 (Vulkan / Metal / DX12)**, shared GPU memory, volumetric 3D | Direct NVMe/SSD memory-mapped `.q42` Merkle DAG, raw TCP/QUIC/libp2p |
| **Profile A2: Native Mobile** | **64-bit iOS/Android** (`.a`/`.so` FFI, NEON SIMD) | Pure Rust AST (Zero JIT, App Store compliant) | Direct Metal / Vulkan GPU via `wgpu`, native low-latency audio | Local sandboxed Q42 vault, cellular/Wi-Fi Merkle sync |
| **Profile B: WASM Web Compatibility** | Mainstream Browser (`wasm32-unknown-unknown`) | WASM Poet Interpreter (`wasm-bindgen`) | WebGPU / WebGL2 through browser origin sandbox, DOM / Web Components | Origin Private File System (OPFS) / IndexedDB, WebSockets / WebRTC |
| **Profile C: Edge & Embedded IoT** | Stripped Native Rust (`no_std`, <2MB RAM) | Micro-Poet AST walker | Headless / Custom Sensor Bus | LoRaWAN / CoAP / MQTT, CBOR-LD delta sync |
