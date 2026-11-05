//! Staged source-value codec for an opt-in single junction. This module does
//! not submit source or replace geometry: host transaction accounting and
//! stream remapping must be integrated before publication can be enabled.
use super::recipe::{read_recipe, BotanicalLimits, BotanicalRecipe};
use std::collections::BTreeMap;
use vibe::Value;

pub(crate) const SCHEMA: &str = "poet.botanical.single-junction/v1";
pub(crate) const SMOOTH_SCHEMA: &str = "poet.botanical.single-junction/v2";
const NORMAL_POLICY: &str = "regenerate-area-weighted/v1";
const FIELDS: [&str; 10] = ["schema", "graph", "origin_m", "spacing_m", "dimensions", "blend_m", "min_axis_m", "projection_tolerance_m", "max_same_direction_cosine", "min_area_sq_m4"];

#[derive(Clone, Debug, PartialEq)]
pub struct JunctionSettings {
    pub tip_radius_m: f64,
    pub origin_m: [f64; 3],
    pub spacing_m: f64,
    pub dimensions: [usize; 3],
    pub blend_m: f64,
    pub min_axis_m: f64,
    pub projection_tolerance_m: f64,
    pub max_same_direction_cosine: f64,
    pub min_area_sq_m4: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct JunctionSource {
    /// Explicit v2 policy. v1 serialization and its authored-normal semantics
    /// stay unchanged; numeric equality never infers a normal's provenance.
    regenerate_normals: bool,
    /// Preserve the entire existing Vibe graph value; no alternate project format.
    graph: Value,
    pub origin_m: [f64; 3],
    pub spacing_m: f64,
    pub dimensions: [usize; 3],
    pub blend_m: f64,
    pub min_axis_m: f64,
    pub projection_tolerance_m: f64,
    pub max_same_direction_cosine: f64,
    pub min_area_sq_m4: f64,
}

fn number(v: &Value) -> Result<f64, String> {
    let n = match v { Value::F64(x) => *x, Value::I64(x) if x.unsigned_abs() <= 1u64 << 53 => *x as f64,
        Value::U64(x) if *x <= 1u64 << 53 => *x as f64, _ => return Err("Expected exact finite number".into()) };
    if !n.is_finite() { return Err("Nonfinite number".into()); }
    Ok(n)
}
fn triple(v: &Value) -> Result<[f64; 3], String> {
    let Value::List(x) = v else { return Err("Expected triple".into()); };
    if x.len() != 3 { return Err("Expected triple".into()); }
    Ok([number(&x[0])?, number(&x[1])?, number(&x[2])?])
}
fn dimensions(v: &Value) -> Result<[usize; 3], String> {
    let Value::List(x) = v else { return Err("Expected three dimensions".into()); };
    if x.len() != 3 { return Err("Expected three dimensions".into()); }
    let dim = |v: &Value| match v { Value::U64(n) => usize::try_from(*n).map_err(|_| "Dimension overflow"),
        Value::I64(n) if *n >= 0 => usize::try_from(*n).map_err(|_| "Dimension overflow"), _ => Err("Expected integer dimension") };
    Ok([dim(&x[0])?, dim(&x[1])?, dim(&x[2])?])
}
fn graph_for_junction(value: &Value, limits: BotanicalLimits) -> Result<BotanicalRecipe, String> {
    // Bound collection sizes before existing read_recipe allocates owned vectors.
    // The host must also bound the incoming Value/source bytes before this call.
    let Value::Record(fields) = value else { return Err("Invalid graph value".into()); };
    match fields.get("organs") {
        Some(Value::List(organs)) if organs.len() == 4 && organs.len() <= limits.max_organs => {
            for organ in organs {
                let Value::Record(o) = organ else { return Err("Invalid organ".into()); };
                if !matches!(o.get("key"), Some(Value::String(s)) if !s.is_empty() && s.len() <= 128)
                    || !(matches!(o.get("parent"), Some(Value::Null)) || matches!(o.get("parent"), Some(Value::String(s)) if !s.is_empty() && s.len() <= 128)) {
                    return Err("Organ identity outside bounds".into());
                }
            }
        },
        _ => return Err("Junction needs exactly four bounded adopted organs".into()),
    }
    if !matches!(fields.get("part_id"), Some(Value::String(s)) if !s.is_empty() && s.len() <= 128) { return Err("Part identity outside bounds".into()); }
    match fields.get("attractors_m") {
        Some(Value::List(points)) if !points.is_empty() && points.len() <= limits.max_attractors => {},
        _ => return Err("Attractor list exceeds host limit".into()),
    }
    // The Vibe emitter writes f64 literals and signed integer literals. Refuse
    // alternate Value variants rather than silently normalizing authored data.
    let triple_is_f64 = |v: Option<&Value>| matches!(v, Some(Value::List(xs))
        if xs.len() == 3 && xs.iter().all(|x| matches!(x, Value::F64(_))));
    for field in ["node_length_m", "kill_radius_m", "perception_radius_m", "cone_angle_rad", "tip_radius_m", "pipe_exponent"] {
        if !matches!(fields.get(field), Some(Value::F64(_))) { return Err("Junction graph requires canonical float source fields".into()); }
    }
    for field in ["version", "max_iterations", "ring_slices"] {
        if !matches!(fields.get(field), Some(Value::I64(_))) { return Err("Junction graph requires canonical signed integer source fields".into()); }
    }
    if let Some(Value::List(organs)) = fields.get("organs") {
        for organ in organs {
            let Value::Record(row) = organ else { return Err("Invalid organ".into()); };
            if !triple_is_f64(row.get("position_m")) { return Err("Junction organ positions require canonical floats".into()); }
        }
    }
    if let Some(Value::List(attractors)) = fields.get("attractors_m") {
        if attractors.iter().any(|point| !triple_is_f64(Some(point))) { return Err("Junction attractors require canonical floats".into()); }
    }
    let graph = read_recipe(Some(value), limits)?.ok_or("Missing graph")?;
    // Current host's v2 evaluator is a chain-only sweep. Do not reinterpret it.
    if graph.version != 1 { return Err("Junction graph currently requires v1 source".into()); }
    if graph.organs.len() != 4 || graph.organs[0].parent.is_some()
        || graph.organs[1].parent.as_deref() != Some(graph.organs[0].key.as_str())
        || graph.organs[2].parent.as_deref() != Some(graph.organs[1].key.as_str())
        || graph.organs[3].parent.as_deref() != Some(graph.organs[1].key.as_str()) {
        return Err("Expected exactly one adopted three-edge junction".into());
    }
    Ok(graph)
}
impl JunctionSource {
    /// Construct an explicit source value from user-entered graph and settings.
    /// No species, geometry or field settings are inferred by this constructor.
    pub fn from_recipe(graph: &BotanicalRecipe, origin_m: [f64; 3], spacing_m: f64,
        dimensions: [usize; 3], blend_m: f64, min_axis_m: f64, projection_tolerance_m: f64,
        max_same_direction_cosine: f64, min_area_sq_m4: f64,
        limits: BotanicalLimits, max_samples: usize) -> Result<Self, String> {
        graph.validate(limits)?;
        let mut record = BTreeMap::new();
        record.insert("schema".into(), Value::String(super::recipe::SCHEMA.into()));
        record.insert("version".into(), Value::I64(i64::from(graph.version)));
        record.insert("part_id".into(), Value::String(graph.part_id.clone()));
        let mut organs = Vec::new();
        organs.try_reserve_exact(graph.organs.len()).map_err(|_| "Organ allocation refused")?;
        for organ in &graph.organs {
            let mut row = BTreeMap::new();
            row.insert("key".into(), Value::String(organ.key.clone()));
            row.insert("parent".into(), organ.parent.as_ref().map(|s| Value::String(s.clone())).unwrap_or(Value::Null));
            row.insert("position_m".into(), Value::List(organ.position_m.iter().copied().map(Value::F64).collect()));
            organs.push(Value::Record(row));
        }
        record.insert("organs".into(), Value::List(organs));
        record.insert("attractors_m".into(), Value::List(graph.attractors_m.iter().map(|p|
            Value::List(p.iter().copied().map(Value::F64).collect())).collect()));
        record.insert("node_length_m".into(), Value::F64(graph.node_length_m));
        record.insert("kill_radius_m".into(), Value::F64(graph.kill_radius_m));
        record.insert("perception_radius_m".into(), Value::F64(graph.perception_radius_m));
        record.insert("cone_angle_rad".into(), Value::F64(graph.cone_angle_rad));
        record.insert("max_iterations".into(), Value::I64(i64::try_from(graph.max_iterations).map_err(|_| "Iteration count overflow")?));
        record.insert("tip_radius_m".into(), Value::F64(graph.tip_radius_m));
        record.insert("pipe_exponent".into(), Value::F64(graph.pipe_exponent));
        record.insert("ring_slices".into(), Value::I64(i64::try_from(graph.ring_slices).map_err(|_| "Ring count overflow")?));
        let result = Self { regenerate_normals: false, graph: Value::Record(record), origin_m, spacing_m, dimensions, blend_m,
            min_axis_m, projection_tolerance_m, max_same_direction_cosine, min_area_sq_m4 };
        result.validate(limits, max_samples)?;
        Ok(result)
    }    /// The caller provides independent host-selected sample and scalar limits.
    pub fn read(value: &Value, limits: BotanicalLimits, max_samples: usize) -> Result<Self, String> {
        let Value::Record(r) = value else { return Err("Junction source must be a record".into()); };
        let regenerate_normals = match r.get("schema") {
            Some(Value::String(s)) if s == SCHEMA => false,
            Some(Value::String(s)) if s == SMOOTH_SCHEMA => true,
            _ => return Err("Unsupported junction schema".into()),
        };
        if r.len() != FIELDS.len() + usize::from(regenerate_normals) || FIELDS.iter().any(|f| !r.contains_key(*f)) {
            return Err("Unknown or missing junction source field".into());
        }
        if regenerate_normals && !matches!(r.get("normal_policy"), Some(Value::String(s)) if s == NORMAL_POLICY) {
            return Err("Junction v2 requires explicit regenerate-area-weighted/v1 normal policy".into());
        }
        if !matches!(r.get("origin_m"), Some(Value::List(xs)) if xs.len() == 3 && xs.iter().all(|v| matches!(v, Value::F64(_)))) {
            return Err("Junction origin requires canonical floats".into());
        }
        if !matches!(r.get("dimensions"), Some(Value::List(xs)) if xs.len() == 3 && xs.iter().all(|v| matches!(v, Value::I64(_)))) {
            return Err("Junction dimensions require canonical signed integers".into());
        }
        for name in ["spacing_m", "blend_m", "min_axis_m", "projection_tolerance_m", "max_same_direction_cosine", "min_area_sq_m4"] {
            if !matches!(r.get(name), Some(Value::F64(_))) { return Err("Junction settings require canonical floats".into()); }
        }
        let graph = r.get("graph").ok_or("Missing graph")?;
        graph_for_junction(graph, limits)?;
        let get = |k: &str| r.get(k).ok_or_else(|| format!("Missing {k}"));
        let source = Self { regenerate_normals, graph: graph.clone(), origin_m: triple(get("origin_m")?)?,
            spacing_m: number(get("spacing_m")?)?, dimensions: dimensions(get("dimensions")?)?,
            blend_m: number(get("blend_m")?)?, min_axis_m: number(get("min_axis_m")?)?,
            projection_tolerance_m: number(get("projection_tolerance_m")?)?,
            max_same_direction_cosine: number(get("max_same_direction_cosine")?)?, min_area_sq_m4: number(get("min_area_sq_m4")?)? };
        source.validate(limits, max_samples)?;
        Ok(source)
    }
    pub fn validate(&self, limits: BotanicalLimits, max_samples: usize) -> Result<(), String> {
        graph_for_junction(&self.graph, limits)?;
        let samples = self.dimensions.iter().try_fold(1usize, |n, d| n.checked_mul(*d)).ok_or("Sample count overflow")?;
        if self.dimensions.iter().any(|d| *d < 2) || samples > max_samples
            || self.origin_m.iter().any(|n| !n.is_finite() || n.abs() > 1e6)
            || !self.spacing_m.is_finite() || self.spacing_m <= 0.0 || self.spacing_m > 1e6
            || !self.blend_m.is_finite() || self.blend_m < 0.0 || self.blend_m > 1e6
            || !self.min_axis_m.is_finite() || self.min_axis_m <= 0.0 || self.min_axis_m > 1e6
            || !self.projection_tolerance_m.is_finite() || self.projection_tolerance_m <= 0.0 || self.projection_tolerance_m > 1e6
            || !self.min_area_sq_m4.is_finite() || self.min_area_sq_m4 < 0.0 || self.min_area_sq_m4 > 1e24
            || !self.max_same_direction_cosine.is_finite() || !(0.0..1.0).contains(&self.max_same_direction_cosine) {
            return Err("Junction parameters outside host bounds".into());
        }
        for i in 0..3 {
            let extent = self.origin_m[i] + self.spacing_m * (self.dimensions[i] - 1) as f64;
            if !extent.is_finite() || extent.abs() > 1e6 { return Err("Grid extent outside metre bounds".into()); }
        }
        Ok(())
    }
    pub(crate) fn with_settings(&self, settings: &JunctionSettings, limits: BotanicalLimits,
        max_samples: usize) -> Result<Self, String> {
        let mut next = self.clone();
        let Value::Record(graph) = &mut next.graph else { return Err("Invalid graph".into()); };
        if !graph.contains_key("tip_radius_m") { return Err("Missing tip radius".into()); }
        graph.insert("tip_radius_m".into(), Value::F64(settings.tip_radius_m));
        next.origin_m = settings.origin_m;
        next.spacing_m = settings.spacing_m;
        next.dimensions = settings.dimensions;
        next.blend_m = settings.blend_m;
        next.min_axis_m = settings.min_axis_m;
        next.projection_tolerance_m = settings.projection_tolerance_m;
        next.max_same_direction_cosine = settings.max_same_direction_cosine;
        next.min_area_sq_m4 = settings.min_area_sq_m4;
        next.validate(limits, max_samples)?;
        Ok(next)
    }
    /// Edits a copy of the exact graph value. Does not mutate caller's source.
    pub fn with_position(&self, key: &str, position_m: [f64; 3], limits: BotanicalLimits, max_samples: usize) -> Result<Self, String> {
        let mut next = self.clone();
        let Value::Record(graph) = &mut next.graph else { return Err("Invalid graph".into()); };
        let Some(Value::List(organs)) = graph.get_mut("organs") else { return Err("Invalid organs".into()); };
        let mut matched = 0;
        for organ in organs {
            let Value::Record(o) = organ else { return Err("Invalid organ".into()); };
            if matches!(o.get("key"), Some(Value::String(s)) if s == key) {
                o.insert("position_m".into(), Value::List(position_m.iter().copied().map(Value::F64).collect()));
                matched += 1;
            }
        }
        if matched != 1 { return Err("Unknown or duplicate organ key".into()); }
        next.validate(limits, max_samples)?;
        Ok(next)
    }
    pub(crate) fn graph(&self, limits: BotanicalLimits) -> Result<BotanicalRecipe, String> {
        graph_for_junction(&self.graph, limits)
    }
    /// Opt in on a new source draft, before adoption. Existing authored normals
    /// are not migrated or discarded by choosing this generation policy.
    pub fn with_regenerated_normals(mut self) -> Self {
        self.regenerate_normals = true;
        self
    }
    pub fn regenerates_normals(&self) -> bool { self.regenerate_normals }
    /// Vibe source expression for the existing graph and explicit junction
    /// settings. Caller must reserve string allocation in its operation ledger.
    pub(crate) fn source_expression(&self, limits: BotanicalLimits, max_samples: usize) -> Result<String, String> {
        self.validate(limits, max_samples)?;
        let graph = self.graph(limits)?;
        let graph_source = super::source_edit::recipe_source(&graph)?;
        let result = format!(
            "{{schema:\"{}\"{},graph:{},origin_m:[{:?},{:?},{:?}],spacing_m:{:?},dimensions:[{},{},{}],blend_m:{:?},min_axis_m:{:?},projection_tolerance_m:{:?},max_same_direction_cosine:{:?},min_area_sq_m4:{:?}}}",
            if self.regenerate_normals { SMOOTH_SCHEMA } else { SCHEMA },
            if self.regenerate_normals { ",normal_policy:\"regenerate-area-weighted/v1\"" } else { "" },
            graph_source, self.origin_m[0], self.origin_m[1], self.origin_m[2], self.spacing_m,
            self.dimensions[0], self.dimensions[1], self.dimensions[2], self.blend_m, self.min_axis_m,
            self.projection_tolerance_m, self.max_same_direction_cosine, self.min_area_sq_m4);
        if result.len() > limits.max_source_bytes { return Err("Junction source exceeds host bound".into()); }
        Ok(result)
    }
    pub fn to_value(&self, limits: BotanicalLimits, max_samples: usize) -> Result<Value, String> {
        self.validate(limits, max_samples)?;
        let mut r = BTreeMap::new();
        r.insert("schema".into(), Value::String(if self.regenerate_normals { SMOOTH_SCHEMA } else { SCHEMA }.into()));
        if self.regenerate_normals { r.insert("normal_policy".into(), Value::String(NORMAL_POLICY.into())); }
        r.insert("graph".into(), self.graph.clone());
        r.insert("origin_m".into(), Value::List(self.origin_m.iter().copied().map(Value::F64).collect()));
        r.insert("spacing_m".into(), Value::F64(self.spacing_m));
        let dims = self.dimensions.iter().map(|n| i64::try_from(*n).map(Value::I64).map_err(|_| "Dimension overflow")).collect::<Result<Vec<_>, _>>()?;
        r.insert("dimensions".into(), Value::List(dims));
        r.insert("blend_m".into(), Value::F64(self.blend_m));
        r.insert("min_axis_m".into(), Value::F64(self.min_axis_m));
        r.insert("projection_tolerance_m".into(), Value::F64(self.projection_tolerance_m));
        r.insert("max_same_direction_cosine".into(), Value::F64(self.max_same_direction_cosine));
        r.insert("min_area_sq_m4".into(), Value::F64(self.min_area_sq_m4));
        Ok(Value::Record(r))
    }
}
