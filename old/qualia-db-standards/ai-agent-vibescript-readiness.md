# AI Agent Readiness & Knowledge Graph Specification for VibeScript
> *"Ensuring Autonomous AI Agents and Transformers Fluently Read, Author, and Validate VibeScript Mindware"*

**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.  
**Principal / inventor:** Timothy Charles Holborn &lt;timothy.holborn@gmail.com&gt;  
Assignment: [`COPYRIGHT.md`](../COPYRIGHT.md) · Licence: [`LICENSE`](../LICENSE)

---

## 1. Executive Summary & Problem Statement

Because **VibeScript** is a greenfield domain-specific language (DSL), foundation Large Language Models (LLMs) and autonomous agents have zero pre-training exposure to its syntax, semantics, or runtime rules. 

To enable AI agents (both host coding assistants and in-app decentralized agents) to fluently **generate, interpret, debug, and execute VibeScript without ungrounded generation**, the system must provide a machine-readable knowledge foundation, constrained decoding grammars, automated diagnostic feedback, and a standardized reference ontology.

---

## 2. Core Requirements Matrix

```
┌──────────────────────────────────────────────────────────────────────────┐
│                   AI Agent VibeScript Enablement Stack                   │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │ 1. Formal Language Knowledge Graph (Vibe Ontology & SHACL Shapes) │  │
│  │    - Machine-readable specification of grammar, AST, and APIs      │  │
│  └─────────────────────────────────┬──────────────────────────────────┘  │
│                                    │                                     │
│  ┌─────────────────────────────────▼──────────────────────────────────┐  │
│  │ 2. Grammar-Guided Constrained Decoding (GBNF / EBNF / JSON-Schema) │  │
│  │    - Forces LLMs to produce 100% syntactically valid VibeScript   │  │
│  └─────────────────────────────────┬──────────────────────────────────┘  │
│                                    │                                     │
│  ┌─────────────────────────────────▼──────────────────────────────────┐  │
│  │ 3. Golden Reference Corpus & Few-Shot Synthetic Examples           │  │
│  │    - Natural language ↔ VibeScript ↔ State delta pairs            │  │
│  └─────────────────────────────────┬──────────────────────────────────┘  │
│                                    │                                     │
│  ┌─────────────────────────────────▼──────────────────────────────────┐  │
│  │ 4. Rust/WASM Diagnostic Compiler Loop (Self-Correction Engine)     │  │
│  │    - Structured AST error spans returned to agents for auto-repair │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Pillar 1: The VibeScript Reference Knowledge Graph (Ontology)

The language itself must be formally represented as an RDF/OWL knowledge graph (persisted in `.q42` and `.ttl` format) that agents can query via SPARQL or retrieve via GraphRAG.

### A. Ontology Schema Namespaces
- `vibe:` (`https://qualiadb.org/schema/vibe#`) — Core syntax, AST constructs, operators.
- `pulse:` (`https://qualiadb.org/schema/pulse#`) — Event streams, networking, telemetry protocols.
- `aura:` (`https://qualiadb.org/schema/aura#`) — Ontological typing, SHACL shapes, context resolution.
- `poet:` (`https://qualiadb.org/schema/poet#`) — Runtime engine semantics, gas costs, error codes.

### B. Machine-Readable Construct Definitions
Each VibeScript keyword, namespace, and standard function is modeled with explicit signature definitions and usage examples:
```turtle
vibe:PulseBroadcast a vibe:BuiltinFunction ;
    rdfs:label "pulse.broadcast" ;
    rdfs:comment "Broadcasts a telemetry or alert payload across a collaborative topic or network peer." ;
    vibe:namespace "pulse" ;
    vibe:parameter [
        vibe:paramName "topic" ;
        vibe:paramType "xsd:string" ;
        vibe:required true
    ] ;
    vibe:parameter [
        vibe:paramName "payload" ;
        vibe:paramType "vibe:Expression" ;
        vibe:required true
    ] ;
    vibe:returns "vibe:Void" ;
    vibe:gasCost 15 ;
    vibe:example """pulse.broadcast("topic:alerts", "Critical condition detected");""" .

vibe:AuraApplySchema a vibe:BuiltinFunction ;
    rdfs:label "aura.apply_schema" ;
    rdfs:comment "Applies an ontological schema or SHACL shape to an entity node in the Q42 graph." ;
    vibe:namespace "aura" ;
    vibe:parameter [
        vibe:paramName "targetNode" ;
        vibe:paramType "vibe:IRI" ;
        vibe:required true
    ] ;
    vibe:parameter [
        vibe:paramName "schemaIri" ;
        vibe:paramType "xsd:string" ;
        vibe:required true
    ] ;
    vibe:returns "vibe:Bool" ;
    vibe:gasCost 25 ;
    vibe:example """aura.apply_schema(clinic_node, "snomed:HealthcareFacility");""" .
```

---

## 4. Pillar 2: Constrained Grammar-Guided Decoding

To eliminate syntax errors before token sampling completes, the **Poet Engine** must export formal grammar artifacts compatible with modern LLM inference engines:

1. **GBNF (GGML BNF Grammar):** For local inference (llama.cpp, Ollama, local Webizen LLM runtimes) forcing valid Vibe token generation.
2. **JSON-Schema / Structural Constraints:** For OpenAI, Anthropic, and Gemini structured output endpoints.
3. **W3C RDF 1.2 Syntax Locks:** Strict grammar rules enforcing correct syntax:
   - Abstract proposition terms: `<<( s p o )>>`
   - Reified assertions with reifier ID: `<< s p o ~ reifier >> [ ... ]`
   - 48-byte Qualia Nquins: `<<[ s p o g prov ]>>`
4. **Declared Capability Blocks:** Enforcing explicit `requires [ capability("name", ...) ]` declarations at the module header.

---

## 5. Pillar 3: Canonical Pattern Library (Golden Corpus)

The repository must maintain a benchmark suite of canonical VibeScript patterns covering all core use cases. This corpus serves as:
1. Few-shot prompt exemplars for code-generation agents.
2. Automated evaluation benchmarks to test model comprehension.

### Target Pattern Categories:
1. **Reactive Cell Formulas:** Spreadsheet calculations aggregating graph queries (`=COUNT(...)`, `=SUM(...)`).
2. **Aura Contextual Resolvers & SHACL-AF Rules:** Inspecting entities on user hover/click, generating `sh:NodeShape` constraints, and executing forward-chaining graph deductions.
3. **Pulse Event Handlers:** Parsing incoming IoT telemetry and emitting alert pulses when thresholds exceed bounds.
4. **Universal Linguistic & Grapheme Resolvers:** Querying unencoded `qualia:OntologicalGrapheme` nodes, mapping PUA codepoints, and linking phonetic audio / 3D `.d10` glyph meshes.
5. **3D Animation, Rigging & Procedural Kinematics:** Controlling skeletal rigs, evaluating 2D blend spaces, solving look-at / two-bone foot IK, and executing 60fps/120fps `on_tick` / `on_anim_event` loops.
6. **Spatiotemporal GeoSPARQL Rules:** Checking polygon intersections and proximity queries.
7. **Decentralized Multi-Agent Transactions:** Signing graph delta commits with DIDs and CRDT clocks.
8. **Homoiconic Code-as-Data Output:** Generating VibeScript AST directly as `yaml-ld-q42` graph nodes for zero-parsing runtime execution.

---

## 6. Pillar 4: Rust/WASM Self-Correction Compiler Loop

When an AI agent writes VibeScript, it must not blindly execute the code. The platform will provide a **Poet Language Diagnostics API** in Rust/WASM that agents can call in a sub-turn loop:

```
┌──────────────┐     1. Generates VibeScript / yaml-ld-q42     ┌────────────────────┐
│   AI Agent   │──────────────────────────────────────────────►│ Poet Validator API │
│  (LLM Model) │                                               │   (Rust / WASM)    │
│              │◄──────────────────────────────────────────────│                    │
└──────────────┘   2. Returns Structured Lints & SHACL Errors  └────────────────────┘
                      {
                        "valid": false,
                        "error_code": "VIBE_E0412",
                        "span": [42, 58],
                        "message": "Unclosed triple term <<(",
                        "suggested_fix": "Add ')>>' to close proposition",
                        "shacl_violations": []
                      }
```

- **Zero Runtime Panics:** Fast in-memory parsing without disk access.
- **SHACL Shape Pre-Validation:** Validates that graph assertions created by the script conform to the document's target ontology before committing deltas.

---

## 7. Pillar 5: Agent Skill & Context Protocol

A standardized markdown skill specification (e.g. `skills/vibescript/SKILL.md`) will be maintained within the project so coding agents automatically load:
- High-level syntax cheat sheets.
- Ontological Grapheme mapping patterns for non-Unicode and specialized phonetic systems.
- Common anti-patterns (e.g. attempting to use JavaScript/Python standard libraries instead of `pulse`/`aura`).
- Instructions on querying the local Q42 reference ontology when encountering unknown domain predicates.

---

## 8. Implementation Milestones

| Milestone | Deliverable | Target Artifact |
| :--- | :--- | :--- |
| **M1: Ontology** | Complete RDF/OWL & SHACL ontology for VibeScript AST, Graphemes, & APIs | `ontologies/vibescript.q42` / `.ttl` |
| **M2: Grammar Compilers** | EBNF to GBNF & JSON-Schema export tools | `crates/poet-grammar/` |
| **M3: Diagnostic API** | Standalone WASM validator returning structured JSON diagnostics | `crates/poet-validator/` |
| **M4: Golden Corpus** | 100+ validated (NL Prompt ↔ VibeScript ↔ State Delta) pairs | `tests/golden_corpus/` |
| **M5: Agent Skill** | Agent skill definition and cheatsheets for IDE pairing | `skills/vibescript/SKILL.md` |
