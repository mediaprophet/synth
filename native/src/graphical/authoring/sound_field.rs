//! Authored spatial clip assignments in the existing Vibe SourceSession.
//! Timeline owns exact clip clocks; associations own explicit HMC byte binding.
//! Entity identifiers are declarations, never identity equivalence or grants.
use super::{source, timeline::Timeline};
use std::collections::{BTreeMap, BTreeSet};
use vibe::Value;

const SCHEMA: &str = "poet.timeline-sound-field/v1";
const MAX_SOURCE_BYTES: usize = 60 * 1024;
const MAX_BINDINGS: usize = 512; // Existing admitted timeline clip profile.

#[derive(Clone, Debug, PartialEq)]
pub struct ClipEmitter {
    pub clip_id: String,
    pub source_identifier: String,
    pub entity_identifier: String,
    pub position_m: [f64; 3],
    pub radius_m: f64,
    pub gain: f64,
    pub priority: i32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SoundField {
    pub listener_position_m: [f64; 3],
    pub listener_yaw_rad: f64,
    pub bindings: Vec<ClipEmitter>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum SoundFieldEdit {
    SetClip {
        clip_id: String,
        entity_identifier: String,
        position_m: [f64; 3],
        radius_m: f64,
        gain: f64,
        priority: i32,
    },
    RemoveClip {
        clip_id: String,
    },
    SetListener {
        position_m: [f64; 3],
        yaw_rad: f64,
    },
}
impl SoundFieldEdit {
    pub fn validate(&self) -> Result<(), String> {
        let clip_ok = |s: &str| {
            !s.is_empty()
                && s.len() <= 128
                && s.bytes().all(|c| {
                    c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'.' | b':' | b'/')
                })
        };
        match self {
            Self::SetListener {
                position_m,
                yaw_rad,
            } if coordinate(*position_m)
                && yaw_rad.is_finite()
                && yaw_rad.abs() <= std::f64::consts::TAU =>
            {
                Ok(())
            }
            Self::RemoveClip { clip_id } if clip_ok(clip_id) => Ok(()),
            Self::SetClip {
                clip_id,
                entity_identifier,
                position_m,
                radius_m,
                gain,
                ..
            } if clip_ok(clip_id)
                && identifier(entity_identifier)
                && coordinate(*position_m)
                && radius_m.is_finite()
                && (0.001..=1_000_000.0).contains(radius_m)
                && gain.is_finite()
                && (0.0..=1.0).contains(gain) =>
            {
                Ok(())
            }
            _ => Err("Invalid sound-field operation fields".into()),
        }
    }
}
fn record<'a>(v: &'a Value, keys: &[&str]) -> Result<&'a BTreeMap<String, Value>, String> {
    let r = source::record(v)?;
    if r.len() != keys.len() || keys.iter().any(|k| !r.contains_key(*k)) {
        return Err("Unknown or missing sound-field field".into());
    }
    Ok(r)
}
fn text(r: &BTreeMap<String, Value>, key: &str) -> Result<String, String> {
    Ok(source::text(source::field(r, key)?)?.to_owned())
}
fn vector(v: &Value) -> Result<[f64; 3], String> {
    let Value::List(items) = v else {
        return Err("Sound-field position requires three coordinates".into());
    };
    if items.len() != 3 {
        return Err("Sound-field position requires three coordinates".into());
    }
    Ok([
        source::number(&items[0])?,
        source::number(&items[1])?,
        source::number(&items[2])?,
    ])
}
fn coordinate(v: [f64; 3]) -> bool {
    v.iter().all(|n| n.is_finite() && n.abs() <= 1_000_000.0)
}
fn identifier(s: &str) -> bool {
    !s.is_empty() && s.len() <= 2048 && !s.chars().any(char::is_control)
}
fn clip<'a>(
    timeline: &'a Timeline,
    id: &str,
) -> Result<(&'a super::timeline::Track, &'a super::timeline::Clip), String> {
    timeline
        .tracks
        .iter()
        .find_map(|track| track.clips.iter().find(|c| c.id == id).map(|c| (track, c)))
        .ok_or_else(|| "Sound-field assignment refers to absent clip".into())
}
impl SoundField {
    pub fn validate(&self, timeline: &Timeline) -> Result<(), String> {
        timeline.validate()?;
        if self.bindings.len() > MAX_BINDINGS
            || !coordinate(self.listener_position_m)
            || !self.listener_yaw_rad.is_finite()
            || self.listener_yaw_rad.abs() > std::f64::consts::TAU
        {
            return Err("Sound-field listener or source profile invalid".into());
        }
        let mut seen = BTreeSet::new();
        let mut entities = BTreeMap::new();
        for b in &self.bindings {
            let (_, c) = clip(timeline, &b.clip_id)?;
            if !seen.insert(b.clip_id.as_str())
                || b.source_identifier != c.source
                || !identifier(&b.entity_identifier)
                || !coordinate(b.position_m)
                || !b.radius_m.is_finite()
                || !(0.001..=1_000_000.0).contains(&b.radius_m)
                || !b.gain.is_finite()
                || !(0.0..=1.0).contains(&b.gain)
            {
                return Err("Duplicate, stale or invalid sound-field clip assignment".into());
            }
            // This first profile has static emitter placement. Conflicting
            // placements for a shared declared entity must not silently win.
            if let Some(prior) =
                entities.insert(b.entity_identifier.as_str(), (b.position_m, b.radius_m))
            {
                if prior != (b.position_m, b.radius_m) {
                    return Err("Shared sound-field entity has conflicting static placement".into());
                }
            }
        }
        Ok(())
    }
}
pub fn read(current: Option<&Value>, timeline: &Timeline) -> Result<Option<SoundField>, String> {
    let Some(value) = current else {
        return Ok(None);
    };
    let r = record(
        value,
        &[
            "schema",
            "listener_position_m",
            "listener_yaw_rad",
            "bindings",
        ],
    )?;
    if text(r, "schema")? != SCHEMA {
        return Err("Unsupported sound-field schema".into());
    }
    let Value::List(items) = source::field(r, "bindings")? else {
        return Err("Sound-field bindings must be a list".into());
    };
    if items.len() > MAX_BINDINGS {
        return Err("Sound-field exceeds admitted timeline profile".into());
    }
    let mut bindings = Vec::with_capacity(items.len());
    for item in items {
        let b = record(
            item,
            &[
                "clip_id",
                "source_identifier",
                "entity_identifier",
                "position_m",
                "radius_m",
                "gain",
                "priority",
            ],
        )?;
        let priority = match source::field(b, "priority")? {
            Value::I64(n) => i32::try_from(*n).map_err(|_| "Sound-field priority outside i32")?,
            Value::U64(n) => i32::try_from(*n).map_err(|_| "Sound-field priority outside i32")?,
            _ => return Err("Sound-field priority requires an exact integer".into()),
        };
        bindings.push(ClipEmitter {
            clip_id: text(b, "clip_id")?,
            source_identifier: text(b, "source_identifier")?,
            entity_identifier: text(b, "entity_identifier")?,
            position_m: vector(source::field(b, "position_m")?)?,
            radius_m: source::number(source::field(b, "radius_m")?)?,
            gain: source::number(source::field(b, "gain")?)?,
            priority,
        });
    }
    let field = SoundField {
        listener_position_m: vector(source::field(r, "listener_position_m")?)?,
        listener_yaw_rad: source::number(source::field(r, "listener_yaw_rad")?)?,
        bindings,
    };
    field.validate(timeline)?;
    Ok(Some(field))
}
pub fn validate_pair(timeline: Option<&Value>, field: Option<&Value>) -> Result<(), String> {
    if field.is_none() {
        return Ok(());
    }
    let timeline =
        super::timeline::read(timeline)?.ok_or("Sound-field lacks an admitted timeline")?;
    read(field, &timeline).map(|_| ())
}
/// Return a normal Vibe source cell. Host authority/mutability/revision checks
/// and validated submission remain the only promotion/persistence mechanism.
pub fn edit_source(
    current: Option<&Value>,
    timeline: &Timeline,
    edit: &SoundFieldEdit,
    has_parts: bool,
) -> Result<String, String> {
    edit.validate()?;
    let mut field = read(current, timeline)?.unwrap_or(SoundField {
        listener_position_m: [0.0; 3],
        listener_yaw_rad: 0.0,
        bindings: Vec::new(),
    });
    match edit {
        SoundFieldEdit::SetListener {
            position_m,
            yaw_rad,
        } => {
            field.listener_position_m = *position_m;
            field.listener_yaw_rad = *yaw_rad;
        }
        SoundFieldEdit::RemoveClip { clip_id } => {
            if clip(timeline, clip_id)?.0.locked {
                return Err("Track is locked".into());
            }
            let index = field
                .bindings
                .iter()
                .position(|b| &b.clip_id == clip_id)
                .ok_or("Clip has no sound-field assignment")?;
            field.bindings.remove(index);
        }
        SoundFieldEdit::SetClip {
            clip_id,
            entity_identifier,
            position_m,
            radius_m,
            gain,
            priority,
        } => {
            let (track, c) = clip(timeline, clip_id)?;
            if track.locked {
                return Err("Track is locked".into());
            }
            let binding = ClipEmitter {
                clip_id: clip_id.clone(),
                source_identifier: c.source.clone(),
                entity_identifier: entity_identifier.clone(),
                position_m: *position_m,
                radius_m: *radius_m,
                gain: *gain,
                priority: *priority,
            };
            if let Some(index) = field.bindings.iter().position(|b| b.clip_id == *clip_id) {
                field.bindings[index] = binding;
            } else {
                field.bindings.push(binding);
            }
        }
    }
    field.validate(timeline)?;
    let q = |s: &str| serde_json::to_string(s).map_err(|e| e.to_string());
    let mut out = if current.is_some() {
        "poet_sound_field = ".to_owned()
    } else {
        "let mut poet_sound_field = ".to_owned()
    };
    out.push_str(&format!(
        "{{schema:\"{SCHEMA}\",listener_position_m:{:?},listener_yaw_rad:{:?},bindings:[",
        field.listener_position_m, field.listener_yaw_rad
    ));
    for (i, b) in field.bindings.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        out.push_str(&format!("{{clip_id:{},source_identifier:{},entity_identifier:{},position_m:{:?},radius_m:{:?},gain:{:?},priority:{}}}", q(&b.clip_id)?, q(&b.source_identifier)?, q(&b.entity_identifier)?, b.position_m, b.radius_m, b.gain, b.priority));
        if out.len() > MAX_SOURCE_BYTES {
            return Err("Sound-field edit exceeds native source budget".into());
        }
    }
    out.push_str(if has_parts {
        "]};\n({parts:poet_parts});"
    } else {
        "]};\n({timeline:poet_timeline});"
    });
    if out.len() > MAX_SOURCE_BYTES {
        return Err("Sound-field edit exceeds native source budget".into());
    }
    Ok(out)
}

#[cfg(test)]
#[path = "sound_field_tests.rs"]
mod tests;
