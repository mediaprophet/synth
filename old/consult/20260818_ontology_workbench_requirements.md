# Ontology Workbench — Design & Requirements

**Date:** 2026-08-18  
**Principal / copyright holder:** Timothy Charles Holborn <timothy.holborn@gmail.com>  
**Assignment:** COPYRIGHT.md  Licence: LICENSE (CC BY-NC-ND 4.0)

## 1. Vision

A visual, mind-mapping-style workbench for authoring, browsing, mapping, and
selecting ontologies. The workbench provides a semantic graph canvas where
classes, properties, and relations are draggable nodes connected by typed
edges — complemented by toolboxes for vocabulary discovery (WordNet/synset
lookup), ontology library search, SHACL shapes editing, N3/Turtle authoring,
and project-level ontology selection.

The Ontology Workbench is a new manifold ("Ontology") that sits between the
Knowledge manifold (graph exploration, document annotation) and the Logic
Workbench (N3/SHACL/RDF-Star evaluation). It provides the *authoring and
curation* layer that feeds those downstream surfaces.

## 2. Architecture

### 2.0 Multi-Paradigm Semantic Framework

OWL's root predicate `owl:Thing` reduces all entities to "things" — an
abstraction that is inadequate for modelling natural persons, human
experiences, selfhood, and non-commodity social relations. The Ontology
Workbench provides **first-class support** for multiple semantic paradigms,
each with different ontological commitments:

| Paradigm | Root commitment | Suitable for | Key advantage |
|----------|----------------|-------------|---------------|
| **OWL/RDFS** | `owl:Thing` | Commercial, organisational, artefact ontologies | Class-based inference, reasoning |
| **SHACL** | None (shape-based) | Validation of any RDF graph without Thing commitment | SPARQL-based, targets existing graphs |
| **ShEx** | None (grammar-based) | Natural-person, selfhood, guardianship, care-scope | Grammar-based, defines shapes as patterns |
| **SKOS** | None (concept-based) | Knowledge organisation, vocabulary mapping, thesauri | Concept schemes without class hierarchy |

The graph canvas includes a **paradigm selector** in the toolbar, allowing
users to switch between OWL, SHACL, ShEx, and SKOS toolboxes. Each paradigm
has its own set of constructs in the toolbox sidebar. The ShEx editor and
SHACL shapes editor are separate containers, each with paradigm-appropriate
UI and validation.

**Design principle:** No paradigm is privileged. The workbench supports
mixed-paradigm ontologies — a project may use OWL for artefact classes,
SHACL for validation rules, ShEx for natural-person shapes, and SKOS for
vocabulary mapping. The project ontology selector tracks which paradigms
are in use and ensures prefix/namespace isolation.

### 2.1 Manifold: Ontology

| Field | Value |
|-------|-------|
| id | `ontology` |
| label | "Ontology" |
| icon | `ontology` |
| ontology_prefix | `ont` |
| description | "Visual ontology authoring, vocabulary mapping, and project ontology selection." |

### 2.2 P0 Containers (8)

| # | Container type | Purpose |
|---|---------------|---------|
| 1 | `ontology_graph_canvas` | 2D/3D semantic graph canvas: classes as nodes, properties as typed edges. Pan/zoom, add/remove nodes, drag to reposition. Toolbox palette with OWL/RDFS relation types. |
| 2 | `ontology_library` | Searchable library of all N3 ontologies in the QualiaDB hypermedia store. Filter by domain, prefix, license. Preview classes/properties. Select for project. |
| 3 | `vocabulary_mapper` | WordNet/synset lookup to find and define terms. Map natural-language terms to ontology classes/properties. Suggest equivalences between vocabularies. |
| 4 | `relation_builder` | Visual builder for OWL/RDFS relations: subClassOf, equivalentClass, property domain/range, cardinality, inverseOf, transitive, symmetric. Form-based with live N3 preview. |
| 5 | `shacl_shapes` | SHACL shapes editor: node shapes, property shapes, constraints (minCount, maxCount, datatype, nodeKind, class, minInclusive, etc.). Live validation status. |
| 6 | `n3_editor` | Full N3/Turtle text editor with syntax highlighting (mock), prefix manager, compile to CBOR-LD, import from URL/file. |
| 7 | `shex_editor` | ShExC (Shape Expressions Compact) text editor with shape list, triple constraints table, cardinality notation, paradigm note. No owl:Thing commitment. Suitable for natural-person, selfhood, guardianship, care-scope ontologies. |
| 8 | `ontology_compare` | Side-by-side comparison of two ontologies: diff classes, properties, axioms. Highlight missing equivalences, naming conflicts. |
| 9 | `project_ontology_selector` | Project-level ontology selection: which ontologies are loaded for the current project. Toggle on/off, set priority, resolve prefix conflicts. |

### 2.3 P1 Containers (5)

| # | Container type | Purpose |
|---|---------------|---------|
| 9 | `semantic_search` | SPARQL/full-text search across all loaded ontologies. Natural-language query → translated to SPARQL. Results with provenance. |
| 10 | `axiom_inspector` | Inspect individual axioms: subject-predicate-object triple view with provenance, confidence, modality. Edit inline. |
| 11 | `class_hierarchy_tree` | Expandable tree view of class hierarchy (subClassOf graph). Click to expand/collapse subtrees. Drag to reorder. |
| 12 | `property_matrix` | Grid view: classes (rows) × properties (columns). Cell shows domain/range/cardinality. Filter by property type. |
| 13 | `import_wizard` | Step-by-step ontology import: URL/file → parse → preview → validate → select prefix → load into project. |

### 2.4 Toolbox Palettes

The graph canvas includes draggable toolbox palettes:

- **Class palette**: owl:Class, rdfs:Class, owl:NamedIndividual
- **Property palette**: owl:ObjectProperty, owl:DatatypeProperty, owl:AnnotationProperty
- **Relation palette**: rdfs:subClassOf, owl:equivalentClass, owl:inverseOf, rdfs:domain, rdfs:range
- **Restriction palette**: owl:someValuesFrom, owl:allValuesFrom, owl:hasValue, owl:minCardinality
- **Logic palette**: owl:unionOf, owl:intersectionOf, owl:complementOf, owl:disjointWith

### 2.5 Vocabulary Mapper Details

The vocabulary mapper integrates with:
- **WordNet** (or equivalent lexical database): synset lookup, hypernyms, hyponyms, meronyms
- **QualiaDB lexicon**: existing gazetteer entries from the NLP engine
- **FST morphology**: morphological analysis of terms

Workflow:
1. User enters a natural-language term (e.g. "obligation")
2. Mapper shows synsets, definitions, hypernym/hyponym chains
3. Mapper suggests matching ontology classes/properties from loaded ontologies
4. User confirms mapping → creates owl:equivalentClass or skos:exactMatch
5. Mappings are stored as provenance-linked annotations

### 2.6 Ontology Library Details

The library indexes all N3 files in `qualia-ui/ontologies/`:

| Ontology | Domain | Prefix | License |
|----------|--------|--------|---------|
| social.n3 | Social relations | soc | CC BY-NC-ND |
| epistemics.n3 | Epistemic modalities | epi | CC BY-NC-ND |
| agency.n3 | Agent identity | agn | CC BY-NC-ND |
| personhood.n3 | Personhood status | per | CC BY-NC-ND |
| selfhood.n3 | Selfhood sensitivity | self | CC BY-NC-ND |
| provenance.n3 | Provenance tracking | prov | CC BY-NC-ND |
| values.n3 | Values anchor | val | CC BY-NC-ND |
| obligations.n3 | Obligations | obl | CC BY-NC-ND |
| duty-of-care.n3 | Duty of care | doc | CC BY-NC-ND |
| care-scope.n3 | Care scope | cs | CC BY-NC-ND |
| research.n3 | Research methodology | res | CC BY-NC-ND |
| learning-core.n3 | Learning | learn | CC BY-NC-ND |
| communications.n3 | Communications | comm | CC BY-NC-ND |
| social-connections.n3 | Social connections | sc | CC BY-NC-ND |
| spatial-3d.n3 | 3D spatial | sp3d | CC BY-NC-ND |
| audio-production.n3 | Audio production | aud | CC BY-NC-ND |
| hypermedia.n3 | HCF spec | hm | CC BY-NC-ND |
| container.n3 | Container spec | cont | CC BY-NC-ND |
| document.n3 | Document model | docm | CC BY-NC-ND |
| code.n3 | Code artefacts | code | CC BY-NC-ND |
| settings.n3 | Settings | set | CC BY-NC-ND |
| investigation.n3 | Investigation | inv | CC BY-NC-ND |
| guardianship.n3 | Guardianship | grd | CC BY-NC-ND |
| agent-nomenclature.n3 | Agent nomenclature | anm | CC BY-NC-ND |
| ungrounded-generation.n3 | Ungrounded generation | ug | CC BY-NC-ND |
| adversarial-conduct.n3 | Adversarial conduct | adv | CC BY-NC-ND |
| adversarial-relational.n3 | Adversarial relations | advr | CC BY-NC-ND |
| faith-systems.n3 | Faith systems | faith | CC BY-NC-ND |
| game-design.n3 | Game design | gd | CC BY-NC-ND |
| game-world.n3 | Game world | gw | CC BY-NC-ND |
| image-editing.n3 | Image editing | ie | CC BY-NC-ND |
| interactive-hypermedia.n3 | Interactive hypermedia | ih | CC BY-NC-ND |
| learning-experience.n3 | Learning experience | le | CC BY-NC-ND |
| learning-experience-modality.n3 | Learning modality | lem | CC BY-NC-ND |
| portal-worlds.n3 | Portal worlds | pw | CC BY-NC-ND |
| presentation.n3 | Presentation | pres | CC BY-NC-ND |
| production-document.n3 | Production docs | pdoc | CC BY-NC-ND |
| production-events.n3 | Production events | pev | CC BY-NC-ND |
| video-production.n3 | Video production | vp | CC BY-NC-ND |

## 3. File Structure

```
src/browser/ontology_views/
  mod.rs
  graph_canvas.rs          — P0: visual semantic graph editor
  ontology_library.rs      — P0: searchable ontology library
  vocabulary_mapper.rs     — P0: WordNet/synset vocabulary mapping
  relation_builder.rs      — P0: OWL/RDFS relation builder
  shacl_shapes.rs          — P0: SHACL shapes editor
  n3_editor.rs             — P0: N3/Turtle text editor
  ontology_compare.rs      — P0: side-by-side ontology diff
  project_ontology_selector.rs — P0: project ontology selection
  semantic_search.rs       — P1: cross-ontology search
  axiom_inspector.rs       — P1: axiom detail inspector
  class_hierarchy_tree.rs  — P1: class hierarchy tree
  property_matrix.rs       — P1: class × property grid
  import_wizard.rs         — P1: step-by-step import

src/tool_chest/manifolds/
  ontology.rs              — manifold seed
```

## 4. Integration Points

- **Knowledge manifold**: Graph Explorer and Ontology Browser feed from the
  ontology workbench's selected project ontologies
- **Logic Workbench**: N3 Logic Studio and SHACL Validator operate on ontologies
  authored/selected here
- **QualiaDB**: Ontologies are stored as N3 files in the hypermedia library;
  the ontology library container indexes them
- **NLP Engine**: Vocabulary mapper uses the lexicon/gazetteer/FST modules from
  the NLP library for term analysis
- **Agent Nomenclature**: The agent-nomenclature ontology is a first-class
  citizen — the workbench enforces nomenclature isolation rules (§3.5 AGENTS.md)
  by validating that mind-dependent terms are not applied to software agents

## 5. Honesty Labels

All P0 containers use `present` honesty (mock data, not wired to engine).
When wired to QualiaDB ontology_loader, containers that query live data
will transition to `live` honesty.
