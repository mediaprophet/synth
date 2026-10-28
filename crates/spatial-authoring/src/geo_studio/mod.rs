// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Poet & QualiaDB Contributors (adapted from EarthCraft v0.2.0, Nerthus v2.3.1, and RustGIS v0.4.0)
//! Round 9 (`GEO-S9`) Poet Unified Geospatial, Terrain DAG, Shared-Vertex Topology &
//! Cartography Studio.
#![forbid(unsafe_code)]

pub mod cartography_layout_cvd;
pub mod cow_delta_undo;
pub mod shared_vertex_ruler_camera;
pub mod terrain_graph_studio;

pub use cartography_layout_cvd::{
    audit_cartographic_palette, bivariate_choropleth_3x3_index, classify_jenks_fisher_exact,
    classify_thematic_breaks, compose_publication_map_svg, simulate_cvd_rgba8,
    wcag21_contrast_ratio, CartographyError, ClassificationMethod, CvdDeficiencyType, PageSizeMm,
    PaletteAccessibilityReport, PrintLayoutSpec, ScaleBarSpec,
};
pub use cow_delta_undo::{
    BrushFalloff, BrushOperation, CowTerrainTileGrid, GeoStudioError, StrokeSample, TileDeltaPatch,
    UndoCommitEntry,
};
pub use shared_vertex_ruler_camera::{
    interpolate_bookmark_flythrough, measure_geodesic_ruler_and_protractor, CameraBookmark,
    GeoTopologyEditError, OrbitalCameraState, RulerMeasurement, SharedVertexId,
    TopologicalCadastralMesh, TopologyPolygonId,
};
pub use terrain_graph_studio::{
    GeoStudioGraphError, StrataLayerInspectorEntry, StudioGraphLink, StudioGraphNode,
    StudioIntentEvent, StudioNodeKind, StudioNodeParam, TerrainGraphStudio,
};
