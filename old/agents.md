# Agent Instructions: Rust NLP & Semantic Enrichment Engine

## 0. Principal, copyright, and assignment (mandatory)

**Principal / inventor / copyright holder:** Timothy Charles Holborn &lt;timothy.holborn@gmail.com&gt;  
See [`COPYRIGHT.md`](COPYRIGHT.md) and [`LICENSE`](LICENSE).

Every artefact you write in this tree is commissioned work **assigned to Timothy Charles Holborn**. Do not name a third party, a fictional person, a provider, or yourself as author, inventor, custodian, or assignee. Demo identities must be Timothy Charles Holborn (`did:qualia:timothy_charles_holborn`). Domain placeholders (a catchment, a clinic) are not people and are not rights-holders.

## 1. Project Mission & Context
This project develops a high-performance, modular, and deterministic **NLP and Semantic Enrichment Library in Rust/WASM** for the **QualiaDB / Webizen** ecosystem.

- **Primary Goal:** Transform unstructured/hypermedia documents into semantically enriched knowledge graphs (RDF-Star, CBOR-LD, JSON-LD) using **Symbolic AI**.
- **Neuro-Symbolic Bridge:** Produce structured, verified substrates that empower downstream Transformer/LLM inference (GraphRAG, guided generation, grounded reasoning) without depending on LLMs for core processing.

## 2. Key References
- **QualiaDB Rustdocs:** https://mediaprophet.github.io/qualiaDB/rustdoc/qualia_core_db/index.html
- **QualiaDB Repository:** https://github.com/mediaprophet/qualiaDB
- **QualiaDB Upstream Feedback:** [qualiadb-updates.md](file:///c:/Projects/NLP/qualiadb-updates.md)
- **Architecture & Capabilities:** [README.md](file:///c:/Projects/NLP/README.md)
- **VibeScript & Poet Engine Spec:** [TechDesign/scriptingLang.md](file:///c:/Projects/NLP/TechDesign/scriptingLang.md)
- **File Format & HCF Spec:** [TechDesign/FileFormat.md](file:///c:/Projects/NLP/TechDesign/FileFormat.md)
- **AI Agent VibeScript Requirements:** [TechDesign/agent_vibescript_requirements.md](file:///c:/Projects/NLP/TechDesign/agent_vibescript_requirements.md)
- **Design Notes:** [notes.md](file:///c:/Projects/NLP/notes.md)
- **Antigravity / Gemini (HyperCanvas mock only):** [consult/antigravity-hypercanvas-mock.md](consult/antigravity-hypercanvas-mock.md)

## 3. Technical Constraints & Rules for Agents
1. **WASM Compatibility:** All code must support `wasm32-unknown-unknown` and `wasm-bindgen`. Avoid platform-native C bindings or OS-specific dependencies in the core crates.
2. **Symbolic First:** Emphasize deterministic algorithms:
   - Aho-Corasick / Tries for gazetteers and lexicon matching over custom ontologies.
   - FSTs (Finite-State Transducers) for morphology and dictionary lookups.
   - Rule-based multi-pass sieves for coreference/anaphora resolution.
   - Frame semantics and TimeML/GeoSPARQL/QUDT normalizers.
3. **Strict Provenance:** Every extracted triple, entity, or annotation must maintain exact byte/character offsets back to the source text span.
4. **QualiaDB Synergy:**
   - Reuse and align with `qualia_core_db` modules (`lexicon`, `graph_index`, `rdf_star`, `ontology_loader`, `cbor_compiler`).
   - If a new NLP feature requires changes or extensions to `qualia_core_db`, document the proposed interface changes in [qualiadb-updates.md](qualiadb-updates.md).
5. **Agent Nomenclature Isolation (mandatory):** Terms that describe the qualities, capacities, or experiences of one agent type (natural, legal, software) must not be applied to another agent type where the underlying STEM mechanism is fundamentally different. Mind-dependent terms ("cognitive", "perception", "belief", "emotion", "hallucination", "deliberate") must not be used for software agents. Intent-dependent terms ("fabricated", "deceptive") must not be used for software agents. Use "enumeration failure" for software agent errors, "ungrounded generation" for statistical output without grounding. See [`qualia-db-standards/agent-nomenclature-rules.md`](qualia-db-standards/agent-nomenclature-rules.md) for the full nomenclature isolation table and rules.

## 4. Documentation & Code Integrity
- Keep [README.md](README.md) and [notes.md](notes.md) updated as new capabilities or modules are implemented.
- Include unit tests and benchmarks demonstrating WASM compatibility and zero-allocation/zero-copy parsing wherever possible.
