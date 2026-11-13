# Local quality reference scenes

These VibeScript sources exercise the actual production paths. They are candidates for visual review, not pre-approved film assets. `generate_fixtures.py` authors deterministic geometry parameters and copies the existing portable fauna/anatomy recipes into this review set. It does not render images or add a modeller.

- Biology: a coarse-grained double-helix illustration; it is not an atomically complete molecular model.
- Anatomy: the existing authored heart, with denser circular profiles and cubic lofts. It must still pass the project's landmark/junction annotations in several views. The candidate uses retained assembly materials rather than a photograph stretched over unmatched surfaces.
- Flora: the existing species-labelled leaf/histology route. Review morphology and material response separately from its labels.
- Fauna: the portable swamp-wallaby specification, with increased radial sampling. Inspect full skin binding and actual poses as well as the rest silhouette.
- Terrain: direct floating-point heightfield composition, erosion, physical metadata and a mesh review.
- Cosmos: a fictitious ringed planet with an explicitly local artistic coordinate frame; it is not a numerical astronomical reconstruction.
- Interface: a finished labelled local-workspace sheet using an explicit retained static font (`ui-font.ttf`), shared outline geometry in PNG/SVG, and real component states.

The integration runner uses a fresh create-only output directory, records source/input/package digests, renders native views and shot frames, and retains the inspected outputs. Visual acceptance is recorded only after images have been opened and reviewed. A procedural recipe, higher polygon count or passing test does not constitute acceptance.
