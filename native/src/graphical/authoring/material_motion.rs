//! Bounded material-key authoring over Poet's existing Vibe source owner.
//!
//! Workflow adaptation reference: EffectCraft key/property commands and per-key
//! outgoing interpolation semantics. No EffectCraft solver, clock, Value model,
//! application host or implementation bytes are copied here. Named easing uses
//! the existing Vibe animation owner. Sampling is an explicit source edit, not
//! a playback clock. SourceSession/authority/history/checkpoint remain upstream.
use super::{source, AuthoringEdit};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vibe::{animation::EasingCurve, Value};

pub const SCHEMA: &str = "poet.material-motion/v1";
pub const MAX_TRACKS: usize = 8;
pub const MAX_KEYS: usize = 64;
pub const MAX_TIME_MS: u32 = 86_400_000;
const MAX_SOURCE_BYTES: usize = 60 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotionInterpolation {
    Hold,
    Linear,
    CubicInOut,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialMotionKey {
    pub time_ms: u32,
    pub base_color: [f64; 4],
    pub roughness: f64,
    pub metallic: f64,
    pub interpolation: MotionInterpolation,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialMotionDraft {
    pub version: u32,
    pub material_id: u64,
    pub keys: Vec<MaterialMotionKey>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialMotionTrack {
    pub draft: MaterialMotionDraft,
    pub sample_ms: u32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum MaterialMotionEdit {
    Set {
        draft: MaterialMotionDraft,
        sample_ms: u32,
    },
    Sample {
        material_id: u64,
        sample_ms: u32,
    },
    /// Removes the recipe only; the currently authored material is unchanged.
    Remove {
        material_id: u64,
    },
}

impl MaterialMotionEdit {
    /// Shared pre-context guard for the strict UI/native/MCP command adapters.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Set { draft, sample_ms } => {
                draft.validate()?;
                time(*sample_ms)
            }
            Self::Sample {
                material_id,
                sample_ms,
            } => {
                if *material_id == 0 {
                    return Err("Material motion needs a nonzero target".into());
                }
                time(*sample_ms)
            }
            Self::Remove { material_id } => {
                if *material_id == 0 {
                    return Err("Material motion needs a nonzero target".into());
                }
                Ok(())
            }
        }
    }
}

fn time(value: u32) -> Result<(), String> {
    if value > MAX_TIME_MS {
        return Err("Material motion time exceeds the one-day profile".into());
    }
    Ok(())
}
fn factors(key: &MaterialMotionKey) -> Result<(), String> {
    if key
        .base_color
        .iter()
        .chain([&key.roughness, &key.metallic])
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
    {
        return Err("Material motion factors must be finite values in [0,1]".into());
    }
    Ok(())
}
impl MaterialMotionDraft {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.material_id == 0
            || self.keys.is_empty()
            || self.keys.len() > MAX_KEYS
        {
            return Err("Unsupported material motion version, target or key count".into());
        }
        let mut previous = None;
        for key in &self.keys {
            time(key.time_ms)?;
            factors(key)?;
            if previous.is_some_and(|p| p >= key.time_ms) {
                return Err(
                    "Material motion keys require strictly increasing distinct times".into(),
                );
            }
            previous = Some(key.time_ms);
        }
        Ok(())
    }
    /// Samples the three bounded authored factors; never advances a clock or
    /// mutates a session. Linear RGBA factors remain separate from spectral data.
    pub fn sample_factors(&self, sample_ms: u32) -> Result<([f64; 4], f64, f64), String> {
        let value = sample(self, sample_ms)?;
        Ok((value.base_color, value.roughness, value.metallic))
    }
}
fn record<'a>(value: &'a Value, fields: &[&str]) -> Result<&'a BTreeMap<String, Value>, String> {
    let r = source::record(value)?;
    if r.len() != fields.len() || fields.iter().any(|key| !r.contains_key(*key)) {
        return Err("Unknown or incomplete material motion source fields".into());
    }
    Ok(r)
}
fn u32_value(value: &Value) -> Result<u32, String> {
    match value {
        Value::U64(n) => {
            u32::try_from(*n).map_err(|_| "Material motion integer outside u32".into())
        }
        Value::I64(n) => {
            u32::try_from(*n).map_err(|_| "Material motion integer outside u32".into())
        }
        _ => Err("Material motion time/version requires an exact integer".into()),
    }
}
fn key_from_value(value: &Value) -> Result<MaterialMotionKey, String> {
    let r = record(
        value,
        &[
            "time_ms",
            "base_color",
            "roughness",
            "metallic",
            "interpolation",
        ],
    )?;
    let Value::List(color) = source::field(r, "base_color")? else {
        return Err("Material motion base_color must contain four factors".into());
    };
    if color.len() != 4 {
        return Err("Material motion base_color must contain four factors".into());
    }
    let mut base_color = [0.0; 4];
    for (slot, value) in base_color.iter_mut().zip(color) {
        *slot = source::number(value)?;
    }
    let interpolation = match source::text(source::field(r, "interpolation")?)? {
        "hold" => MotionInterpolation::Hold,
        "linear" => MotionInterpolation::Linear,
        "cubic_in_out" => MotionInterpolation::CubicInOut,
        _ => return Err("Unsupported material motion interpolation".into()),
    };
    Ok(MaterialMotionKey {
        time_ms: u32_value(source::field(r, "time_ms")?)?,
        base_color,
        roughness: source::number(source::field(r, "roughness")?)?,
        metallic: source::number(source::field(r, "metallic")?)?,
        interpolation,
    })
}

/// Reads only the declared bounded schema, never silently repairs unknown data.
pub fn read_tracks(current: Option<&Value>) -> Result<Vec<MaterialMotionTrack>, String> {
    let Some(current) = current else {
        return Ok(Vec::new());
    };
    let r = record(current, &["schema", "tracks"])?;
    if source::text(source::field(r, "schema")?)? != SCHEMA {
        return Err(
            "Unsupported material motion source schema; preserve it and migrate explicitly".into(),
        );
    }
    let Value::List(rows) = source::field(r, "tracks")? else {
        return Err("Material motion tracks must be a bounded list".into());
    };
    if rows.len() > MAX_TRACKS {
        return Err("Material motion track limit exceeded".into());
    }
    let mut tracks: Vec<MaterialMotionTrack> = Vec::with_capacity(rows.len());
    for row in rows {
        let row = record(row, &["draft", "sample_ms"])?;
        let draft = record(
            source::field(row, "draft")?,
            &["version", "material_id", "keys"],
        )?;
        let Value::List(keys) = source::field(draft, "keys")? else {
            return Err("Material motion keys must be a bounded list".into());
        };
        if keys.is_empty() || keys.len() > MAX_KEYS {
            return Err("Material motion key count outside profile".into());
        }
        let draft = MaterialMotionDraft {
            version: u32_value(source::field(draft, "version")?)?,
            material_id: source::integer(source::field(draft, "material_id")?)?,
            keys: keys.iter().map(key_from_value).collect::<Result<_, _>>()?,
        };
        draft.validate()?;
        if tracks
            .iter()
            .any(|t| t.draft.material_id == draft.material_id)
        {
            return Err("Duplicate material motion target".into());
        }
        let sample_ms = u32_value(source::field(row, "sample_ms")?)?;
        time(sample_ms)?;
        tracks.push(MaterialMotionTrack { draft, sample_ms });
    }
    Ok(tracks)
}

fn sample(draft: &MaterialMotionDraft, sample_ms: u32) -> Result<MaterialMotionKey, String> {
    draft.validate()?;
    time(sample_ms)?;
    let first = draft.keys.first().ok_or("Material motion needs a key")?;
    if sample_ms <= first.time_ms {
        return Ok(first.clone());
    }
    let last = draft.keys.last().ok_or("Material motion needs a key")?;
    if sample_ms >= last.time_ms {
        return Ok(last.clone());
    }
    // Segment lookup is bounded adapter work; easing mathematics stays in Vibe.
    for pair in draft.keys.windows(2) {
        let [a, b] = pair else {
            return Err("Invalid material motion segment".into());
        };
        if sample_ms < b.time_ms {
            let progress = f64::from(sample_ms - a.time_ms) / f64::from(b.time_ms - a.time_ms);
            let weight = match a.interpolation {
                MotionInterpolation::Hold => 0.0,
                MotionInterpolation::Linear => EasingCurve::Linear.eval(progress),
                MotionInterpolation::CubicInOut => EasingCurve::CubicInOut.eval(progress),
            };
            let mix = |left: f64, right: f64| left + (right - left) * weight;
            let mut value = a.clone();
            for i in 0..4 {
                value.base_color[i] = mix(a.base_color[i], b.base_color[i]);
            }
            value.roughness = mix(a.roughness, b.roughness);
            value.metallic = mix(a.metallic, b.metallic);
            factors(&value)?;
            return Ok(value);
        }
    }
    Err("Material motion sample has no admitted segment".into())
}

struct Source(String);
impl Source {
    fn push(&mut self, text: &str) -> Result<(), String> {
        if self
            .0
            .len()
            .checked_add(text.len())
            .is_none_or(|n| n > MAX_SOURCE_BYTES)
        {
            return Err("Material motion source exceeds the 60 KiB bound".into());
        }
        self.0.push_str(text);
        Ok(())
    }
}
fn binding_source(tracks: &[MaterialMotionTrack], existing: bool) -> Result<String, String> {
    let mut out = Source(String::new());
    out.push(if existing {
        "poet_material_motion = "
    } else {
        "let mut poet_material_motion = "
    })?;
    out.push("{schema:\"poet.material-motion/v1\",tracks:[")?;
    for (i, track) in tracks.iter().enumerate() {
        if i != 0 {
            out.push(",")?;
        }
        out.push(&format!(
            "{{draft:{{version:1,material_id:{}u64,keys:[",
            track.draft.material_id
        ))?;
        for (i, key) in track.draft.keys.iter().enumerate() {
            if i != 0 {
                out.push(",")?;
            }
            let interpolation = match key.interpolation {
                MotionInterpolation::Hold => "hold",
                MotionInterpolation::Linear => "linear",
                MotionInterpolation::CubicInOut => "cubic_in_out",
            };
            out.push(&format!(
                "{{time_ms:{},base_color:{},roughness:{},metallic:{},interpolation:\"{}\"}}",
                key.time_ms,
                source::array(&key.base_color)?,
                source::literal(key.roughness)?,
                source::literal(key.metallic)?,
                interpolation
            ))?;
        }
        out.push(&format!("]}},sample_ms:{}}}", track.sample_ms))?;
    }
    out.push("]};\n")?;
    Ok(out.0)
}

/// Builds one explicit source edit. The controller must check binding mutability
/// and current authority/revision before submission, as for existing authoring.
#[allow(clippy::too_many_arguments)]
pub fn edit_source(
    parts: &Value,
    parameters: Option<&Value>,
    parameter_baseline: Option<&Value>,
    source_layer: Option<&Value>,
    current: Option<&Value>,
    edit: &MaterialMotionEdit,
    revision: u64,
) -> Result<String, String> {
    edit.validate()?;
    let mut tracks = read_tracks(current)?;
    let material_id = match edit {
        MaterialMotionEdit::Set { draft, sample_ms } => {
            draft.validate()?;
            time(*sample_ms)?;
            draft.material_id
        }
        MaterialMotionEdit::Sample {
            material_id,
            sample_ms,
        } => {
            time(*sample_ms)?;
            *material_id
        }
        MaterialMotionEdit::Remove { material_id } => *material_id,
    };
    let Value::List(rows) = parts else {
        return Err("Material motion needs authored parts".into());
    };
    if material_id == 0 || rows.is_empty() || rows.len() > 256 {
        return Err("Material motion target outside part profile".into());
    }
    let mut material_ids = std::collections::BTreeSet::new();
    for row in rows {
        let part = source::record(row)?;
        let material = source::record(source::field(part, "material")?)?;
        let id = source::integer(source::field(material, "id")?)?;
        if (id == material_id || tracks.iter().any(|t| t.draft.material_id == id))
            && !matches!(material.get("colour_space"), Some(Value::String(s)) if s == "linear")
        {
            return Err("Material motion requires an explicitly linear material target".into());
        }
        material_ids.insert(id);
    }
    if !material_ids.contains(&material_id) {
        return Err("Unknown material motion target".into());
    }
    if tracks
        .iter()
        .any(|t| !material_ids.contains(&t.draft.material_id))
    {
        return Err(
            "Material motion has an orphaned target; preserve source and migrate explicitly".into(),
        );
    }
    let slot = tracks
        .iter()
        .position(|t| t.draft.material_id == material_id);
    let sampled = match edit {
        MaterialMotionEdit::Set { draft, sample_ms } => {
            let track = MaterialMotionTrack {
                draft: draft.clone(),
                sample_ms: *sample_ms,
            };
            let value = sample(draft, *sample_ms)?;
            if let Some(slot) = slot {
                tracks[slot] = track;
            } else if tracks.len() < MAX_TRACKS {
                tracks.push(track);
            } else {
                return Err("Material motion track limit exceeded".into());
            }
            Some(value)
        }
        MaterialMotionEdit::Sample { sample_ms, .. } => {
            let track = tracks
                .get_mut(slot.ok_or("No material motion track for target")?)
                .ok_or("Material motion track disappeared")?;
            let value = sample(&track.draft, *sample_ms)?;
            track.sample_ms = *sample_ms;
            Some(value)
        }
        MaterialMotionEdit::Remove { .. } => {
            tracks.remove(slot.ok_or("No material motion track for target")?);
            None
        }
    };
    let mut out = Source(String::new());
    if let Some(value) = sampled {
        out.push(&source::edit_source_with_layers(
            parts,
            parameters,
            parameter_baseline,
            source_layer,
            &AuthoringEdit::Material {
                material_id,
                base_color: value.base_color,
                roughness: value.roughness,
                metallic: value.metallic,
            },
            revision,
        )?)?;
        out.push("\n")?;
    }
    out.push(&binding_source(&tracks, current.is_some())?)?;
    out.push("({parts:poet_parts});")?;
    Ok(out.0)
}
