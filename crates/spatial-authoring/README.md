# Poet spatial authoring helpers

These internal functions support Poet's existing spatial source authoring and
presentation. They define no new authoring product, suite, UI architecture,
persistent document model, journal or capability namespace.

Existing ownership remains:

| Existing Poet responsibility | Supporting helper |
|---|---|
| Tool-Chest/Manifold/context selection and intent dispatch | Unchanged; the library does not navigate or dispatch intents. |
| Dual Studio authored Vibe cells and canonical native SourceSession | `Profile` is a temporary operator-parameter value; edits remain whole-binding source submissions. |
| Geometry host capability invocation | Explicitly granted `ComputationalGeometry.profile_sweep` uses inherited profile helpers and existing Qualia sweep/cap kernels. |
| Scene-session geometry/material presentation | `compile_assembly` prepares identified parts, inherited transforms, normal/UV streams and canonical material ranges. |
| Source undo/redo, native cooperative authorization, checkpoints and evidence | Unchanged existing owners; no library revision counter or storage mechanism. |

Translation is in metres and rotation in radians; parent transforms compose
through the existing Core matrix API. Optional authored normals use Core's full
world inverse transpose. Missing normals use preflighted flat-corner expansion;
UVs and stable semantic part mappings remain attached. A local part key does not
replace qualified identifiers, rights or graph/provenance records.

The inherited geometry sources are used under MIT with their original copyright
and permission text retained. `provenance/original-NOTICE` is historical source
evidence, including its old optional licence and third-party references; it does
not state the adopted licence of these packages or the entire Poet project.
These new packages select MIT. Existing Poet project licence terms and legitimate
external dependency notices remain separate.

The earlier staging directory and all original application repositories are
preserved. This extraction does not establish a completed recovery of all prior
Poet functionality. Its source and command mapping is a support integration;
graphical usability, artist acceptance and complete model/package acceptance
require their own evidence.
