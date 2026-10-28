//! Poet Terrain Node-Graph Studio, 6-Layer Stratigraphic Column Inspector,
//! and Bi-Directional VibeScript AST <-> IntentBus Live Sync.
//!
//! Adapted from EarthCraft (v0.2.0, MIT) and Nerthus (v2.3.1, MIT).
//! Portions copyright (c) EarthCraft contributors, Nerthus contributors, and Poet authors.
//! Licensed under the MIT License or Apache-2.0 at your option.

#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write;

/// Errors produced by terrain node-graph studio operations.
#[derive(Debug, Clone, PartialEq)]
pub enum GeoStudioGraphError {
    CycleDetected,
    NodeNotFound(u32),
    ParamNotFound(String),
    LayerIndexOutOfBounds(u8),
    ByteBudgetExceeded { current_bytes: usize, budget_bytes: usize },
    InvalidLink(&'static str),
}

impl std::fmt::Display for GeoStudioGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CycleDetected => write!(f, "Cycle detected in terrain graph DAG"),
            Self::NodeNotFound(id) => write!(f, "Node {id} not found in terrain graph"),
            Self::ParamNotFound(p) => write!(f, "Parameter '{p}' not found on node"),
            Self::LayerIndexOutOfBounds(i) => write!(f, "Strata layer {i} out of bounds (0..6)"),
            Self::ByteBudgetExceeded { current_bytes, budget_bytes } => {
                write!(f, "Byte budget exceeded: {current_bytes} > {budget_bytes}")
            }
            Self::InvalidLink(msg) => write!(f, "Invalid link: {msg}"),
        }
    }
}

impl std::error::Error for GeoStudioGraphError {}

/// Functional node kinds supported by the Poet Terrain Node-Graph Studio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StudioNodeKind {
    PerlinFbm,
    SimplexRidged,
    VoronoiWorley,
    SwissTurbulence,
    MountainPrimitive,
    CanyonPrimitive,
    HydraulicErosion824,
    StrataColumn6Layer,
    ThermalSnowGlacier,
    TerraceFilter,
    HeightSlopeSelector,
    BlendCombiner,
    Sentinel2BiomeOutput,
}

impl StudioNodeKind {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::PerlinFbm => "PerlinFbm", Self::SimplexRidged => "SimplexRidged",
            Self::VoronoiWorley => "VoronoiWorley", Self::SwissTurbulence => "SwissTurbulence",
            Self::MountainPrimitive => "MountainPrimitive", Self::CanyonPrimitive => "CanyonPrimitive",
            Self::HydraulicErosion824 => "HydraulicErosion824", Self::StrataColumn6Layer => "StrataColumn6Layer",
            Self::ThermalSnowGlacier => "ThermalSnowGlacier", Self::TerraceFilter => "TerraceFilter",
            Self::HeightSlopeSelector => "HeightSlopeSelector", Self::BlendCombiner => "BlendCombiner",
            Self::Sentinel2BiomeOutput => "Sentinel2BiomeOutput",
        }
    }

    pub fn default_params(&self) -> Vec<StudioNodeParam> {
        let p: &[(&str, f64, f64, f64)] = match self {
            Self::PerlinFbm => &[("octaves", 6.0, 1.0, 16.0), ("frequency", 0.005, 0.0001, 0.1), ("persistence", 0.5, 0.0, 1.0), ("lacunarity", 2.0, 1.0, 4.0)],
            Self::SimplexRidged => &[("octaves", 6.0, 1.0, 16.0), ("frequency", 0.003, 0.0001, 0.1), ("gain", 0.5, 0.0, 1.0)],
            Self::VoronoiWorley => &[("frequency", 0.01, 0.0001, 0.2), ("jitter", 1.0, 0.0, 1.0)],
            Self::SwissTurbulence => &[("frequency", 0.004, 0.0001, 0.1), ("warp_scale", 0.15, 0.0, 1.0)],
            Self::MountainPrimitive => &[("height_m", 1200.0, 0.0, 8848.0), ("radius_m", 2500.0, 100.0, 20000.0)],
            Self::CanyonPrimitive => &[("depth_m", 400.0, 0.0, 3000.0), ("width_m", 600.0, 10.0, 5000.0)],
            Self::HydraulicErosion824 => &[("iterations", 50.0, 1.0, 500.0), ("rain_rate", 0.012, 0.001, 0.1), ("evaporation", 0.02, 0.001, 0.5)],
            Self::StrataColumn6Layer => &[("dip_angle_deg", 5.0, 0.0, 90.0), ("strike_deg", 45.0, 0.0, 360.0)],
            Self::ThermalSnowGlacier => &[("talus_angle", 33.0, 10.0, 60.0), ("snow_line_m", 1800.0, 0.0, 6000.0)],
            Self::TerraceFilter => &[("step_height_m", 25.0, 1.0, 200.0), ("smoothness", 0.2, 0.0, 1.0)],
            Self::HeightSlopeSelector => &[("min_height_m", 0.0, -500.0, 8000.0), ("max_slope_deg", 45.0, 0.0, 90.0)],
            Self::BlendCombiner => &[("blend_factor", 0.5, 0.0, 1.0), ("blend_mode", 0.0, 0.0, 3.0)],
            Self::Sentinel2BiomeOutput => &[("resolution_m", 10.0, 1.0, 100.0), ("ndvi_threshold", 0.3, -1.0, 1.0)],
        };
        p.iter().map(|&(n, v, min, max)| StudioNodeParam::new(n, v, min, max)).collect()
    }
}

/// Dynamic scalar parameter exposed by a studio graph node.
#[derive(Debug, Clone, PartialEq)]
pub struct StudioNodeParam {
    pub name: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
}

impl StudioNodeParam {
    pub fn new(name: &str, value: f64, min: f64, max: f64) -> Self {
        Self { name: name.to_string(), value, min, max }
    }
}

/// A node in the visual terrain graph.
#[derive(Debug, Clone, PartialEq)]
pub struct StudioGraphNode {
    pub node_id: u32,
    pub title: String,
    pub kind: StudioNodeKind,
    pub pos_xy: [f32; 2],
    pub params: Vec<StudioNodeParam>,
    pub dirty: bool,
}

/// A directed connection link between node ports in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StudioGraphLink {
    pub from_node: u32,
    pub from_port: u8,
    pub to_node: u32,
    pub to_port: u8,
}

/// An entry in the 6-layer stratigraphic geological inspector.
#[derive(Debug, Clone, PartialEq)]
pub struct StrataLayerInspectorEntry {
    pub layer_index: u8,
    pub material_name: String,
    pub thickness_m: f32,
    pub erodibility_k: f32,
    pub talus_angle_deg: f32,
    pub porosity: f32,
}

/// Undoable authoring intent events dispatched to the Poet IntentBus.
#[derive(Debug, Clone, PartialEq)]
pub enum StudioIntentEvent {
    NodeAdded { node_id: u32, kind: StudioNodeKind },
    ParamUpdated { node_id: u32, param_name: String, new_value: f64 },
    LinkConnected { from_node: u32, to_node: u32, to_port: u8 },
    StrataLayerEdited { layer_index: u8, thickness_m: f32, erodibility_k: f32 },
}

/// Poet Terrain Node-Graph Studio managing nodes, links, 6-layer strata, and VibeScript sync.
pub struct TerrainGraphStudio {
    pub nodes: Vec<StudioGraphNode>,
    pub links: Vec<StudioGraphLink>,
    pub strata_column: [StrataLayerInspectorEntry; 6],
    pub intent_log: Vec<StudioIntentEvent>,
    pub byte_budget: usize,
    next_node_id: u32,
}

impl TerrainGraphStudio {
    /// Creates a new `TerrainGraphStudio` with the specified memory byte budget.
    pub fn new(byte_budget: usize) -> Self {
        Self {
            nodes: Vec::new(),
            links: Vec::new(),
            strata_column: Self::default_strata_column(),
            intent_log: Vec::new(),
            byte_budget,
            next_node_id: 0,
        }
    }

    /// Initializes canonical 6-layer Earth stratigraphic column.
    pub fn default_strata_column() -> [StrataLayerInspectorEntry; 6] {
        [
            StrataLayerInspectorEntry { layer_index: 0, material_name: "Topsoil / Humus".into(), thickness_m: 1.2, erodibility_k: 0.85, talus_angle_deg: 28.0, porosity: 0.45 },
            StrataLayerInspectorEntry { layer_index: 1, material_name: "Sandstone / Quartzite".into(), thickness_m: 15.0, erodibility_k: 0.35, talus_angle_deg: 38.0, porosity: 0.18 },
            StrataLayerInspectorEntry { layer_index: 2, material_name: "Shale / Mudstone".into(), thickness_m: 25.0, erodibility_k: 0.60, talus_angle_deg: 32.0, porosity: 0.12 },
            StrataLayerInspectorEntry { layer_index: 3, material_name: "Limestone / Karst".into(), thickness_m: 40.0, erodibility_k: 0.40, talus_angle_deg: 42.0, porosity: 0.22 },
            StrataLayerInspectorEntry { layer_index: 4, material_name: "Basalt / Volcanic".into(), thickness_m: 60.0, erodibility_k: 0.15, talus_angle_deg: 45.0, porosity: 0.05 },
            StrataLayerInspectorEntry { layer_index: 5, material_name: "Granite Basement".into(), thickness_m: 120.0, erodibility_k: 0.08, talus_angle_deg: 50.0, porosity: 0.02 },
        ]
    }

    /// Estimates the current allocated byte footprint of studio structures.
    pub fn estimated_bytes(&self) -> usize {
        let mut b = std::mem::size_of::<Self>();
        for node in &self.nodes {
            b += std::mem::size_of::<StudioGraphNode>() + node.title.len();
            for param in &node.params { b += std::mem::size_of::<StudioNodeParam>() + param.name.len(); }
        }
        b += self.links.len() * std::mem::size_of::<StudioGraphLink>();
        b += self.intent_log.len() * std::mem::size_of::<StudioIntentEvent>();
        for s in &self.strata_column { b += s.material_name.len(); }
        b
    }

    fn check_budget(&self) -> Result<(), GeoStudioGraphError> {
        let current = self.estimated_bytes();
        if current > self.byte_budget {
            Err(GeoStudioGraphError::ByteBudgetExceeded { current_bytes: current, budget_bytes: self.byte_budget })
        } else {
            Ok(())
        }
    }

    /// Appends a new node to the graph with default parameters, marked dirty.
    pub fn add_node(&mut self, kind: StudioNodeKind, title: &str, pos_xy: [f32; 2]) -> Result<u32, GeoStudioGraphError> {
        self.check_budget()?;
        let node_id = self.next_node_id;
        self.next_node_id += 1;
        self.nodes.push(StudioGraphNode {
            node_id, title: title.to_string(), kind, pos_xy, params: kind.default_params(), dirty: true,
        });
        self.intent_log.push(StudioIntentEvent::NodeAdded { node_id, kind });
        self.check_budget()?;
        Ok(node_id)
    }

    /// Connects two node ports in the DAG. Rejects cycles with `CycleDetected` and marks downstream nodes dirty.
    pub fn connect_nodes(&mut self, from_node: u32, from_port: u8, to_node: u32, to_port: u8) -> Result<(), GeoStudioGraphError> {
        if !self.nodes.iter().any(|n| n.node_id == from_node) { return Err(GeoStudioGraphError::NodeNotFound(from_node)); }
        if !self.nodes.iter().any(|n| n.node_id == to_node) { return Err(GeoStudioGraphError::NodeNotFound(to_node)); }
        if from_node == to_node || self.path_exists(to_node, from_node) { return Err(GeoStudioGraphError::CycleDetected); }
        if !self.links.iter().any(|l| l.from_node == from_node && l.from_port == from_port && l.to_node == to_node && l.to_port == to_port) {
            self.links.push(StudioGraphLink { from_node, from_port, to_node, to_port });
            let downstream = self.downstream_nodes(to_node);
            for node in &mut self.nodes { if downstream.contains(&node.node_id) { node.dirty = true; } }
            self.intent_log.push(StudioIntentEvent::LinkConnected { from_node, to_node, to_port });
        }
        self.check_budget()?;
        Ok(())
    }

    /// Updates a parameter on a node, clamping to `[min, max]`, logging the intent,
    /// marking downstream nodes dirty, and returning the topological evaluation order of dirty nodes.
    pub fn set_node_param(&mut self, node_id: u32, param_name: &str, value: f64) -> Result<Vec<u32>, GeoStudioGraphError> {
        let node = self.nodes.iter_mut().find(|n| n.node_id == node_id).ok_or(GeoStudioGraphError::NodeNotFound(node_id))?;
        let param = node.params.iter_mut().find(|p| p.name == param_name).ok_or_else(|| GeoStudioGraphError::ParamNotFound(param_name.to_string()))?;
        let clamped = value.clamp(param.min, param.max);
        param.value = clamped;

        self.intent_log.push(StudioIntentEvent::ParamUpdated {
            node_id, param_name: param_name.to_string(), new_value: clamped,
        });

        let downstream = self.downstream_nodes(node_id);
        for n in &mut self.nodes { if downstream.contains(&n.node_id) { n.dirty = true; } }
        let order = self.topological_evaluation_order(&downstream, node_id);
        self.check_budget()?;
        Ok(order)
    }

    /// Edits a stratigraphic layer, updating physical properties and recording an intent event.
    pub fn edit_strata_layer(&mut self, layer_index: u8, thickness_m: f32, erodibility_k: f32, talus_angle_deg: f32) -> Result<(), GeoStudioGraphError> {
        let idx = layer_index as usize;
        if idx >= self.strata_column.len() { return Err(GeoStudioGraphError::LayerIndexOutOfBounds(layer_index)); }
        let layer = &mut self.strata_column[idx];
        layer.thickness_m = thickness_m;
        layer.erodibility_k = erodibility_k;
        layer.talus_angle_deg = talus_angle_deg;

        let strata_node_ids: Vec<u32> = self.nodes.iter().filter(|n| n.kind == StudioNodeKind::StrataColumn6Layer).map(|n| n.node_id).collect();
        for sid in strata_node_ids {
            let down = self.downstream_nodes(sid);
            for n in &mut self.nodes { if down.contains(&n.node_id) { n.dirty = true; } }
        }

        self.intent_log.push(StudioIntentEvent::StrataLayerEdited { layer_index, thickness_m, erodibility_k });
        self.check_budget()?;
        Ok(())
    }

    /// Serializes the current studio graph and stratigraphic column into canonical VibeScript 0.2 source.
    pub fn export_to_vibescript_source(&self, topology_name: &str) -> Result<String, GeoStudioGraphError> {
        self.check_budget()?;
        let mut out = String::with_capacity(1024);
        let _ = writeln!(out, "// VibeScript 0.2 Canonical Terrain Topology");
        let _ = writeln!(out, "topology {topology_name} : TerrainGraphComplex {{");
        let _ = writeln!(out, "    strata_column {{");
        for e in &self.strata_column {
            let _ = writeln!(
                out,
                "        layer[{}]: material = \"{}\", thickness_m = {:.2}, erodibility_k = {:.3}, talus_angle_deg = {:.1}, porosity = {:.3};",
                e.layer_index, e.material_name, e.thickness_m, e.erodibility_k, e.talus_angle_deg, e.porosity
            );
        }
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "    nodes {{");
        for n in &self.nodes {
            let _ = writeln!(out, "        node[{}]: {} \"{}\" @ [{:.1}, {:.1}] {{", n.node_id, n.kind.name(), n.title, n.pos_xy[0], n.pos_xy[1]);
            for p in &n.params {
                let _ = writeln!(out, "            {} = {:.4};", p.name, p.value);
            }
            let _ = writeln!(out, "        }}");
        }
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "    links {{");
        for l in &self.links {
            let _ = writeln!(out, "        connect(node[{}].port[{}] -> node[{}].port[{}]);", l.from_node, l.from_port, l.to_node, l.to_port);
        }
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "}}");

        if self.estimated_bytes() + out.len() > self.byte_budget {
            return Err(GeoStudioGraphError::ByteBudgetExceeded {
                current_bytes: self.estimated_bytes() + out.len(),
                budget_bytes: self.byte_budget,
            });
        }
        Ok(out)
    }

    /// Bi-directional live sync applying VibeScript assignment edits back into node parameters.
    pub fn sync_from_vibescript_assignments(&mut self, assignments: &[(u32, &str, f64)]) -> Result<usize, GeoStudioGraphError> {
        let mut count = 0;
        for &(node_id, param_name, new_val) in assignments {
            self.set_node_param(node_id, param_name, new_val)?;
            count += 1;
        }
        Ok(count)
    }

    fn path_exists(&self, start: u32, target: u32) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start);
        while let Some(curr) = queue.pop_front() {
            if curr == target { return true; }
            for link in &self.links {
                if link.from_node == curr && visited.insert(link.to_node) {
                    queue.push_back(link.to_node);
                }
            }
        }
        false
    }

    fn downstream_nodes(&self, start: u32) -> HashSet<u32> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start);
        while let Some(curr) = queue.pop_front() {
            for link in &self.links {
                if link.from_node == curr && visited.insert(link.to_node) {
                    queue.push_back(link.to_node);
                }
            }
        }
        visited
    }

    fn topological_evaluation_order(&self, subset: &HashSet<u32>, root: u32) -> Vec<u32> {
        let mut in_degrees: HashMap<u32, usize> = subset.iter().map(|&id| {
            let deg = self.links.iter().filter(|l| l.to_node == id && subset.contains(&l.from_node)).count();
            (id, deg)
        }).collect();

        let mut queue = VecDeque::new();
        if in_degrees.get(&root).copied().unwrap_or(0) == 0 { queue.push_back(root); }
        for (&id, &deg) in &in_degrees { if id != root && deg == 0 { queue.push_back(id); } }

        let mut order = Vec::with_capacity(subset.len());
        while let Some(curr) = queue.pop_front() {
            order.push(curr);
            for link in &self.links {
                if link.from_node == curr && subset.contains(&link.to_node) {
                    if let Some(deg) = in_degrees.get_mut(&link.to_node) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 { queue.push_back(link.to_node); }
                    }
                }
            }
        }
        order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acyclic_dag_and_dirty_propagation() {
        let mut s = TerrainGraphStudio::new(100_000);
        let n0 = s.add_node(StudioNodeKind::PerlinFbm, "Base Perlin", [0.0, 0.0]).unwrap();
        let n1 = s.add_node(StudioNodeKind::TerraceFilter, "Terraces", [100.0, 0.0]).unwrap();
        let n2 = s.add_node(StudioNodeKind::BlendCombiner, "Combiner", [200.0, 0.0]).unwrap();
        let n3 = s.add_node(StudioNodeKind::Sentinel2BiomeOutput, "Biome Out", [300.0, 0.0]).unwrap();

        s.connect_nodes(n0, 0, n1, 0).unwrap();
        s.connect_nodes(n1, 0, n2, 0).unwrap();

        assert_eq!(s.connect_nodes(n2, 0, n0, 0).unwrap_err(), GeoStudioGraphError::CycleDetected);
        assert_eq!(s.connect_nodes(n1, 0, n1, 0).unwrap_err(), GeoStudioGraphError::CycleDetected);

        for n in &mut s.nodes { n.dirty = false; }
        let order = s.set_node_param(n1, "smoothness", 0.8).unwrap();
        assert_eq!(order, vec![n1, n2]);

        assert!(!s.nodes.iter().find(|n| n.node_id == n0).unwrap().dirty);
        assert!(s.nodes.iter().find(|n| n.node_id == n1).unwrap().dirty);
        assert!(s.nodes.iter().find(|n| n.node_id == n2).unwrap().dirty);
        assert!(!s.nodes.iter().find(|n| n.node_id == n3).unwrap().dirty);
    }

    #[test]
    fn test_strata_column_inspector_editing() {
        let mut s = TerrainGraphStudio::new(100_000);
        assert_eq!(s.strata_column.len(), 6);
        assert_eq!(s.strata_column[0].material_name, "Topsoil / Humus");

        s.edit_strata_layer(0, 3.5, 0.65, 30.0).unwrap();
        assert_eq!(s.strata_column[0].thickness_m, 3.5);
        assert_eq!(s.strata_column[0].erodibility_k, 0.65);
        assert_eq!(s.strata_column[0].talus_angle_deg, 30.0);

        let out_of_bounds = s.edit_strata_layer(6, 1.0, 0.5, 20.0).unwrap_err();
        assert_eq!(out_of_bounds, GeoStudioGraphError::LayerIndexOutOfBounds(6));

        assert!(s.intent_log.iter().any(|ev| matches!(
            ev,
            StudioIntentEvent::StrataLayerEdited { layer_index: 0, thickness_m, erodibility_k }
            if (*thickness_m - 3.5).abs() < 1e-4 && (*erodibility_k - 0.65).abs() < 1e-4
        )));
    }

    #[test]
    fn test_vibescript_roundtrip_and_budget_fail_closed() {
        let mut s = TerrainGraphStudio::new(100_000);
        let n0 = s.add_node(StudioNodeKind::MountainPrimitive, "Alps Peak", [50.0, 50.0]).unwrap();
        let n1 = s.add_node(StudioNodeKind::HydraulicErosion824, "Erosion", [150.0, 50.0]).unwrap();
        s.connect_nodes(n0, 0, n1, 0).unwrap();

        let source = s.export_to_vibescript_source("AlpineRange").unwrap();
        assert!(source.contains("topology AlpineRange : TerrainGraphComplex"));
        assert!(source.contains("strata_column"));
        assert!(source.contains("MountainPrimitive"));
        assert!(source.contains("HydraulicErosion824"));
        assert!(source.contains("connect(node[0].port[0] -> node[1].port[0])"));

        let updated_count = s.sync_from_vibescript_assignments(&[
            (n0, "height_m", 4810.0),
            (n1, "iterations", 120.0),
        ]).unwrap();
        assert_eq!(updated_count, 2);

        let n0_node = s.nodes.iter().find(|n| n.node_id == n0).unwrap();
        let height_param = n0_node.params.iter().find(|p| p.name == "height_m").unwrap();
        assert_eq!(height_param.value, 4810.0);

        let mut tiny_studio = TerrainGraphStudio::new(100);
        let err = tiny_studio.add_node(StudioNodeKind::SwissTurbulence, "Warp", [0.0, 0.0]).unwrap_err();
        assert!(matches!(err, GeoStudioGraphError::ByteBudgetExceeded { .. }));
    }
}
