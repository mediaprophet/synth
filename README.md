# NLP
NLP Dev

## Current Poet implementation programme (2026-10-11)

Poet is now a standalone workspace: `wasm/` contains the authoring frontend and `native/` currently contains the VibeScript CLI, not a graphical desktop frontend. Dependencies on sibling QualiaDB are intentional. The Webizen OS shell is the target operating-environment UI; useful legacy Webizen Desktop services are to be extracted into toolkit-independent host libraries, not retained as a required competing UI.

The comprehensive cross-repository programme is maintained in the existing QualiaDB planning documents:

- [ST architecture, contracts, app mappings, graphics, supervision and release gates](../qualiaDB/docs/work-in-progress/POET_UPDATED_IMPLEMENTATION_PLAN_2026-09-05.md#st-programme-2026-10-11--storytold-apps-poet-and-webizen-os).
- [ST-00 through ST-39 execution packets and continuation prompt](../qualiaDB/docs/POET_LOWER_COST_AGENT_EXECUTION_PLAYBOOK_2026-09-04.md#18-st-programme-execution-packets-2026-10-11).
- [VibeScript application-language extension specification (ST-N)](../qualiaDB/docs/work-in-progress/POET_UPDATED_IMPLEMENTATION_PLAN_2026-09-05.md#st-n-vibescript-as-the-canonical-application-language).
- [VB-00 through VB-09 language/runtime/binding/tooling migration packets](../qualiaDB/docs/POET_LOWER_COST_AGENT_EXECUTION_PLAYBOOK_2026-09-04.md#19-vb-programme--vibescript-javascript-replacement-extensions).
- [Append-only execution and verification ledger](../qualiaDB/docs/POET_IMPLEMENTATION_SESSION_LEDGER.md).

Read the ST sections before the historical September instructions. New Poet changes belong here, not in retained `qualiaDB/crates/poet` copies. The plan covers all eleven Storytold apps, a shared command/context system, connected command orbs, contextual menus, macro/micro attention, native/WASM execution and first-class supervised agents. Planning is not a completion claim: start with `ST-00`, execute one bounded subunit and retain real test/UAT evidence. No implementation packet is complete solely because its plan exists.

The planning branch is `0.0.40.8-dev` in Poet and QualiaDB. Other repositories require their own checked branch/status before editing. No commit, push, publication, paid provider call or user-data deletion is implicit in the plan.

The original NLP/product notes below are retained historical context; their Webizen Desktop UI destination does not supersede the current ST programme.

### Separation of **Poet** vs **`ecopunk-forge`** (3D Asset Authoring Studio)

- **Poet** (`poet` — see `wasm/`, `old/`, and `../qualiaDB/crates/poet/`) is Timothy's Hypermedia, NLP, Knowledge Document, Context Markup Language, and Tool-Chest & Manifolds application.
- **`ecopunk-forge`** (`cargo run --manifest-path native/Cargo.toml --bin ecopunk-forge --features native-graphical`) is the standalone 3D / CAD / botanical / zoological `.10d` & `.hmc` asset-production and verification harness added inside `native/`. Naming its binary `ecopunk-forge` keeps the 3D asset-production tool available for Ecopunk and `dev.civics.au` without colliding with or undermining Poet.

**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.  
**Principal / inventor:** Timothy Charles Holborn &lt;timothy.holborn@gmail.com&gt;  
See [`COPYRIGHT.md`](COPYRIGHT.md) and [`LICENSE`](LICENSE) (CC BY-NC-ND 4.0).

This project is intended to work towards updating QualiaDB (webizen); to support,

- A Document application, like an advanced word-processing app, that supports NLP related applications, including but not limited to RDF graphs

- Enhancement of the document processing engine, to support more advanced NLP capabilities, built upon RDF graphs

To achieve these goals, the project seeks to develop an advanced, modern, and comprehensive NLP library, that is designed for the webizen / qualiadb ecosystem, which is built in rust. 

## Dev Methods and technology selection
Whilst most NLP libraries are built in python, and many are relatively old, the opportunity and related goals for this project is to build an advanced ecosystem for rust in particular, and therefore also, WASM.

## Functionality

There's a few goals.

## Document creator

The way i imagine this, is that there's a sort of convergence between different traditional document creation apps, to create a sort of knowledge document creation app, that can be used for projects that are orientated both towards fictional or non-fictional topics, concepts, purposes..

Depending on the project, this might incorporate the creation and incorporation of a variety of different types of content assets, which are largely considered to be forms of Hypermedia.

Characteristics
-  WordProcessing like document content
- Spreadsheet like document content, incorporating data-processing in cells, and formulas
- Presentation software like document content, with slide-like structuring of content
- Audio
- Video
- Image
- 3D content, 3d objects, 3d scenes
- Graph
- Rdf
- Ontology

This in-turn acts as a means to advance the 'Hypermedia library' that's already being developed in Webizen Desktop.

A critical feature is to define a UI approach for applying structured data 'annotations' to specific elements within documents.  this is defined as 'context markup language'.

## Development considerations

The application is intended to be developed primarily for WASM in the first instance, which should then enable the meana to create both a web version, and then update it to incorporate it into the Webizen Desktop product..

## Personalised Ontology

One of the challenges is that, whilst there are many existing ontologies, the configuration of ontologies for specific purposes are both customised from the perspective of the application and user; and, in-turn, there's the development of 'personal' ontology, which in-turn also relates to the definition of ontologies that represent the body of knowledge of a project, knowledge base etc..

Ontology configuration should be considered as part of the document creation process, whereby context is sought to be defined as part of the processs relating to NLP processes. 

## NLP Capabilities (Symbolic & Neuro-Symbolic in Rust / WASM)

The primary processing engine is built on deterministic, symbolic AI and ontology-driven semantic enrichment, designed to operate efficiently in Rust and WASM without requiring direct runtime dependence on LLMs. In turn, these enriched documents serve as high-fidelity structured substrates for downstream Transformer/LLM inference, GraphRAG, and reasoning.

### 1. Token & Morphological Processing
- Tokenisation & Sentence Split
- Multi-word Token Expansion & Segmentation
- POS & Morphological Tagging (FST / Rule-assisted)
- Lemmatization & Stemming
- Gazetteer & Lexicon Matching (Fast Aho-Corasick / Trie matching over custom ontologies)

### 2. Syntactic & Relational Parsing
- Dependency Parsing & Chunking (Shallow Parsing)
- Rule-based Entity & Relation Extraction (Pattern & Grammar Matchers)
- Deterministic Coreference & Anaphora Resolution (Rule-based Multi-pass Sieve)

### 3. Semantic Grounding & Ontological Enrichment
- Named Entity Recognition (NER) & IRI/URI Disambiguation / Linking
- Relation Extraction & Frame Semantics (FrameNet / PropBank semantic role labeling)
- Temporal Normalization (TimeML / TIMEX3 to ISO-8601 & OWL-Time)
- Spatial Grounding & Quantitative Normalization (GeoSPARQL & QUDT unit/measure conversion)
- Event Extraction & Tracking
  - Time-Event Extraction
  - Location-Event Extraction
  - Entity-Event Extraction
  - Relation-Event Extraction
- Lightweight In-memory Symbolic Reasoning (RDFS / OWL-RL forward/backward chaining)
- Discourse & Rhetorical Structure (RST / DRT narrative relations)

### 4. Neuro-Symbolic & LLM Bridge (Transformer Enablers)
- Semantic / Knowledge-Aware Chunking (Discourse & subgraph boundary partitioning)
- Graph Context Linearization & Serialization (RDF Turtle, CBOR-LD, JSON-LD, Context Tables)
- Provenance & Span-to-Triple Anchoring (Byte/character offset verification)
- Constraint & Schema Export (SHACL, JSON-Schema, BNF grammars for guided LLM decoding)
- Subgraph Extraction for GraphRAG & In-Context Grounding
- Embeddings & Vector Projections (word2vec, Sentence2Vec, zero-shot classifiers)
- Text Summarisation & Generation Helpers

 
### Language Modalities

- Speech
- Text
- Images
- Video
- Language Context
  - Geographic
  - Cultural
  - Historical
  - Subject Specific
    - Genre
    - Subject Matter
    - register
    - style
    - STEM
    - Law
    - Finance
    - Medicine
    - Philosophy
  - Original Medium
    - Oral
    - Written
    - Digital
    - Pictorial
    - Audio
    - Video
    - Audio-Visual
    - Multi-Modal
  - Time Context
    - Past
    - Present
    - Future
  - Knowledge Representation
    - Explicit Knowledge
    - Implicit Knowledge
    - Procedural Knowledge
    - Declarative Knowledge
    - Tacit Knowledge

- 

