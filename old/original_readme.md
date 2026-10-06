# NLP
NLP Dev

This project is intended to work towards updating QualiaDB (webizen); to support,

- A Document application, like an advanced word-processing app, that supports NLP related applications, including but not limited to RDF graphs

- Enhancement of the document processing engine, to support more advanced NLP capabilities, built upon RDF graphs

To achieve these goals, the project seeks to develop an advanced, modern, and comprehensive NLP library, that is designed for the webizen / qualiadb ecosystem, which is built in rust. 

## Dev Methods and technology selection
Whilst most NLP libraries are built in python, and many are relatively old, the opportunity and related goals for this project is to build an advanced ecosystem for rust in particular, and therefore also, WASM.

## Functionality

There's a few goals.

-  

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
- Graph Context Linearization & Serialization (RDF Turtle, JSON-LD, Context Tables)
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

