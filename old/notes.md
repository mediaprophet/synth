Based on this Symbolic AI + Knowledge Graph / Ontology focus, here are the specific capabilities and structural mechanisms that are vital to include:

1. Symbolic & Rule-Based Extraction (Lightweight, Deterministic in Rust/WASM)
Gazetteer & Fast Lexicon Matching: High-performance trie/Aho-Corasick pattern matchers over personal and domain ontologies (instant matching against custom dictionaries).
Rule-Based Pattern / Grammar Matchers (JAPE / Token Matchers / PEG): Rule systems for extracting entities and relations over token, POS, and dependency streams without neural overhead.
Deterministic Coreference & Anaphora Resolution: Rule-based multi-pass sieves (e.g., Hobbs algorithm or Centering Theory) to resolve "it", "they", "the company" back to exact RDF entity nodes.
Morphological Analysis & Lemmatization Dictionaries: Finite-state transducers (FST) or lookup tables for exact inflection and root extraction.
2. Semantic Web, Ontology Grounding & Formal Semantics
Entity Disambiguation & IRI/URI Linking: Explicitly mapping text spans to specific RDF classes, instances, and properties in your personal or global ontologies (e.g., schema:, foaf:, or custom domain ontologies).
Frame Semantics (FrameNet / PropBank): Mapping predicates and actions into semantic frames (e.g., identifying who is the Agent, Patient, Instrument, and Result in a deterministic structure).
Lightweight Symbolic Reasoning (RDFS / OWL-RL / Datalog): In-memory forward/backward chaining to infer implicit facts (e.g., if $x is_a Employee, infer $x is_a Person).
Discourse & Rhetorical Structure (RST / DRT): Identifying narrative relations (causation, contrast, elaboration, temporal sequence) across sections and paragraphs.
3. Quantitative, Temporal & Spatial Normalization
TimeML / TIMEX3 Temporal Normalization: Resolving relative temporal expressions ("two weeks prior to the launch", "next quarter") into precise absolute intervals / ISO-8601 timestamps and OWL-Time representations.
Quantity & Unit Normalization (QUDT): Parsing and normalizing numeric quantities, currencies, and measurements into standardized physical units and dimensions.
Spatial / Geo-Grounding (GeoSPARQL): Extracting and normalizing topological and spatial relationships ("within", "adjacent to", "north of").
4. Downstream LLM / Transformer Enablers (The "Neuro-Symbolic Bridge")
Semantic / Knowledge-Aware Chunking: Partitioning documents based on semantic boundaries, discourse units, or subgraphs, rather than arbitrary token counts.
GraphRAG / Subgraph Extraction: Extracting localized RDF subgraphs relevant to a specific query or context window to inject into LLM prompts.
Provenance & Attribution Anchoring: Byte/character offset-to-RDF triple mapping, enabling LLMs to cite exact source spans and verify claims against the symbolic graph.
Schema & Grammar Export (SHACL / JSON-Schema / BNF): Generating formal constraints directly from the document's ontology so LLM outputs can be validated against structured rules.
Graph Linearization / Serializers: Deterministic serialization of RDF graphs into LLM-friendly formats (JSON-LD, Turtle, or structured Markdown context).
Proposed Structure for your README.md
We could organize the NLP Capabilities into clear symbolic and neuro-symbolic tiers:

markdown
## Symbolic & Structural NLP Capabilities (Rust / WASM Native)
### 1. Token & Morphological Processing
- Tokenisation & Sentence Split
- Multi-word Token Expansion & Segmentation
- POS & Morphological Tagging (FST / Rule-assisted)
- Lemmatization & Stemming
- Gazetteer & Lexicon Matching (Aho-Corasick / Tries)
### 2. Syntactic & Relational Parsing
- Dependency Parsing & Chunking
- Rule-based Entity & Relation Matchers (Pattern/Grammar engines)
- Deterministic Coreference & Anaphora Resolution (Multi-pass Sieve)
### 3. Semantic Grounding & Ontological Enrichment
- Named Entity Recognition & IRI/URI Linking
- Frame Semantics & Semantic Role Labeling (SRL)
- Temporal Normalization (TimeML / TIMEX3)
- Spatial Grounding & Quantitative Normalization (Units/Measures)
- Event Extraction (Tracking, Time, Location, Entity, Relation)
- Lightweight In-memory Reasoning (RDFS / OWL-RL Inference)
### 4. Neuro-Symbolic & LLM Bridge
- Semantic / Discourse-Aware Chunking (for vector/graph indexing)
- Graph Context Linearization (RDF/JSON-LD to prompt context)
- Provenance & Span-to-Triple Anchoring
- Constraint & Schema Generation (SHACL / JSON-Schema / Grammars for LLM decoding)
- GraphRAG Subgraph Extraction