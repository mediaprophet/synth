//! An inline source descriptor for one adopted, identified botanical graph.
//! This is not a taxon assertion, simulation snapshot, or package format.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use vibe::Value;

pub const SCHEMA: &str = "poet.botanical/v1";
/// Opt-in continuous unbranched-chain sweep; fork surfaces remain separate.
pub const CHAIN_SCHEMA: &str = "poet.botanical/v2";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Organ {
    pub key: String,
    pub parent: Option<String>,
    pub position_m: [f64; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotanicalRecipe {
    pub version: u32,
    pub part_id: String,
    pub organs: Vec<Organ>,
    pub attractors_m: Vec<[f64; 3]>,
    pub node_length_m: f64,
    pub kill_radius_m: f64,
    pub perception_radius_m: f64,
    pub cone_angle_rad: f64,
    pub max_iterations: usize,
    pub tip_radius_m: f64,
    pub pipe_exponent: f64,
    pub ring_slices: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct BotanicalLimits {
    pub max_organs: usize,
    pub max_attractors: usize,
    pub max_proposal_nodes: usize,
    pub max_iterations: usize,
    pub max_vertices: usize,
    pub max_triangles: usize,
    pub max_source_bytes: usize,
}

impl Default for BotanicalLimits {
    fn default() -> Self {
        Self {
            max_organs: 256,
            max_attractors: 1024,
            max_proposal_nodes: 2048,
            max_iterations: 64,
            max_vertices: 20_000,
            max_triangles: 40_000,
            max_source_bytes: 256 * 1024,
        }
    }
}

pub(crate) fn key_valid(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b':' | b'/'))
}

impl BotanicalRecipe {
    pub fn validate(&self, limits: BotanicalLimits) -> Result<(), String> {
        if !matches!(self.version, 1 | 2) || !key_valid(&self.part_id) {
            return Err("Unsupported botanical recipe version or part key".into());
        }
        if self.organs.len() < 2
            || self.organs.len() > limits.max_organs
            || self.attractors_m.is_empty()
            || self.attractors_m.len() > limits.max_attractors
            || limits.max_proposal_nodes < self.organs.len()
            || self.max_iterations > limits.max_iterations
            || !(3..=64).contains(&self.ring_slices)
        {
            return Err("Botanical graph or working budget outside admitted bounds".into());
        }
        let numeric = [
            self.node_length_m,
            self.kill_radius_m,
            self.perception_radius_m,
            self.cone_angle_rad,
            self.tip_radius_m,
            self.pipe_exponent,
        ];
        if numeric
            .iter()
            .any(|n| !n.is_finite() || *n <= 0.0 || *n > 1.0e6)
            || self.perception_radius_m < self.kill_radius_m
            || self.cone_angle_rad > std::f64::consts::TAU
            || self.pipe_exponent < 1.0
            || self.pipe_exponent > 8.0
        {
            return Err("Botanical distances, angle or exponent are invalid".into());
        }
        if self
            .organs
            .iter()
            .flat_map(|o| o.position_m)
            .chain(self.attractors_m.iter().flat_map(|p| *p))
            .any(|n| !n.is_finite() || n.abs() > 1.0e6)
        {
            return Err("Botanical coordinates exceed finite metre bounds".into());
        }
        let mut keys = BTreeMap::new();
        let mut roots = 0;
        for (i, organ) in self.organs.iter().enumerate() {
            if !key_valid(&organ.key) || keys.insert(organ.key.as_str(), i).is_some() {
                return Err("Duplicate or invalid organ key".into());
            }
            match &organ.parent {
                None => roots += 1,
                Some(parent)
                    if !key_valid(parent) || !keys.get(parent.as_str()).is_some_and(|p| *p < i) =>
                {
                    return Err(
                        "Organ parent must be an earlier distinct adopted key; cycle refused"
                            .into(),
                    )
                }
                _ => {}
            }
        }
        if roots != 1 || self.organs[0].parent.is_some() {
            return Err("Botanical graph requires one first root".into());
        }
        for organ in self.organs.iter().skip(1) {
            let p = keys[organ.parent.as_ref().expect("validated parent").as_str()];
            let a = organ.position_m;
            let b = self.organs[p].position_m;
            let d2 = a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum::<f64>();
            if d2 <= 1.0e-18 {
                return Err("Zero-length organ edge refused".into());
            }
        }
        let edges = self.organs.len() - 1;
        let vertices = edges
            .checked_mul(self.ring_slices * 2 + 2)
            .ok_or("Botanical vertex budget overflow")?;
        let triangles = edges
            .checked_mul(self.ring_slices * 4)
            .ok_or("Botanical triangle budget overflow")?;
        // Version 2 additionally scans and retains parent/child chain metadata.
        // Charge two extra vertex-work units per organ against the existing
        // work/scratch-derived limit before allocating those arrays. The old
        // per-edge bound remains a conservative bound for chain mesh output.
        let charged_vertices = if self.version == 2 {
            self.organs
                .len()
                .checked_mul(2)
                .and_then(|n| vertices.checked_add(n))
                .ok_or("Botanical chain work overflow")?
        } else {
            vertices
        };
        if charged_vertices > limits.max_vertices || triangles > limits.max_triangles {
            return Err("Botanical mesh and chain work exceed host-selected working set".into());
        }
        Ok(())
    }
}

fn record<'a>(v: &'a Value) -> Result<&'a BTreeMap<String, Value>, String> {
    match v {
        Value::Record(r) => Ok(r),
        _ => Err("Botanical source must be a record".into()),
    }
}
fn exact<'a>(v: &'a Value, fields: &[&str]) -> Result<&'a BTreeMap<String, Value>, String> {
    let r = record(v)?;
    let expected: BTreeSet<&str> = fields.iter().copied().collect();
    if r.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected {
        return Err("Unknown or missing botanical source field".into());
    }
    Ok(r)
}
fn get<'a>(r: &'a BTreeMap<String, Value>, k: &str) -> Result<&'a Value, String> {
    r.get(k).ok_or_else(|| format!("Missing botanical {k}"))
}
fn string(v: &Value) -> Result<&str, String> {
    match v {
        Value::String(s) => Ok(s),
        _ => Err("Botanical key must be text".into()),
    }
}
fn usize_value(v: &Value) -> Result<usize, String> {
    match v {
        Value::U64(n) => usize::try_from(*n).map_err(|_| "Botanical integer overflow".into()),
        Value::I64(n) if *n >= 0 => {
            usize::try_from(*n).map_err(|_| "Botanical integer overflow".into())
        }
        _ => Err("Botanical integer required".into()),
    }
}
fn number(v: &Value) -> Result<f64, String> {
    let n = match v {
        Value::F64(n) => *n,
        Value::I64(n) if n.unsigned_abs() <= 1u64 << 53 => *n as f64,
        Value::U64(n) if *n <= 1u64 << 53 => *n as f64,
        _ => return Err("Botanical number required".into()),
    };
    if !n.is_finite() {
        return Err("Nonfinite botanical number".into());
    }
    Ok(n)
}
fn point(v: &Value) -> Result<[f64; 3], String> {
    let Value::List(xs) = v else {
        return Err("Botanical point must be a 3-vector".into());
    };
    if xs.len() != 3 {
        return Err("Botanical point must be a 3-vector".into());
    }
    Ok([number(&xs[0])?, number(&xs[1])?, number(&xs[2])?])
}
fn list(v: &Value) -> Result<&[Value], String> {
    match v {
        Value::List(xs) => Ok(xs),
        _ => Err("Botanical list required".into()),
    }
}

pub fn read_recipe(
    current: Option<&Value>,
    limits: BotanicalLimits,
) -> Result<Option<BotanicalRecipe>, String> {
    let Some(v) = current else { return Ok(None) };
    let r = exact(
        v,
        &[
            "schema",
            "version",
            "part_id",
            "organs",
            "attractors_m",
            "node_length_m",
            "kill_radius_m",
            "perception_radius_m",
            "cone_angle_rad",
            "max_iterations",
            "tip_radius_m",
            "pipe_exponent",
            "ring_slices",
        ],
    )?;
    let schema = string(get(r, "schema")?)?;
    let version = usize_value(get(r, "version")?)?;
    if !matches!((schema, version), (SCHEMA, 1) | (CHAIN_SCHEMA, 2)) {
        return Err("Unsupported botanical source schema/version pair".into());
    }
    let organs = list(get(r, "organs")?)?
        .iter()
        .map(|v| {
            let o = exact(v, &["key", "parent", "position_m"])?;
            let parent = match get(o, "parent")? {
                Value::Null => None,
                v => Some(string(v)?.to_owned()),
            };
            Ok(Organ {
                key: string(get(o, "key")?)?.to_owned(),
                parent,
                position_m: point(get(o, "position_m")?)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let recipe = BotanicalRecipe {
        version: u32::try_from(usize_value(get(r, "version")?)?).map_err(|_| "Version overflow")?,
        part_id: string(get(r, "part_id")?)?.to_owned(),
        organs,
        attractors_m: list(get(r, "attractors_m")?)?
            .iter()
            .map(point)
            .collect::<Result<_, _>>()?,
        node_length_m: number(get(r, "node_length_m")?)?,
        kill_radius_m: number(get(r, "kill_radius_m")?)?,
        perception_radius_m: number(get(r, "perception_radius_m")?)?,
        cone_angle_rad: number(get(r, "cone_angle_rad")?)?,
        max_iterations: usize_value(get(r, "max_iterations")?)?,
        tip_radius_m: number(get(r, "tip_radius_m")?)?,
        pipe_exponent: number(get(r, "pipe_exponent")?)?,
        ring_slices: usize_value(get(r, "ring_slices")?)?,
    };
    recipe.validate(limits)?;
    Ok(Some(recipe))
}
