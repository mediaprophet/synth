//! One explicit Vibe cell updates the inline descriptor and selected part mesh.
//! The caller supplies admitted current bindings and submits through the existing
//! authority/revision/source-session owner. No independent document is created.
use super::recipe::{read_recipe, BotanicalLimits, BotanicalRecipe, Organ, SCHEMA};
use crate::graphical::botanical_graph_host::evaluate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use vibe::Value;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum BotanicalEdit {
    SetRecipe {
        recipe: BotanicalRecipe,
    },
    SetTipRadius {
        part_id: String,
        tip_radius_m: f64,
    },
    SetOrganPosition {
        part_id: String,
        organ_key: String,
        position_m: [f64; 3],
    },
}

fn quoted(s: &str) -> Result<String, String> {
    serde_json::to_string(s).map_err(|e| e.to_string())
}
fn number(n: f64) -> Result<String, String> {
    if !n.is_finite() || n.abs() > 1.0e6 {
        return Err("Botanical source number outside finite bound".into());
    }
    Ok(format!("{n:?}"))
}
fn point(p: &[f64; 3]) -> Result<String, String> {
    Ok(format!(
        "[{},{},{}]",
        number(p[0])?,
        number(p[1])?,
        number(p[2])?
    ))
}
fn recipe_source(r: &BotanicalRecipe) -> Result<String, String> {
    let mut out = format!(
        "{{schema:{},version:1,part_id:{},organs:[",
        quoted(SCHEMA)?,
        quoted(&r.part_id)?
    );
    for (i, organ) in r.organs.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        out.push_str(&organ_source(organ)?);
    }
    out.push_str("],attractors_m:[");
    for (i, p) in r.attractors_m.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        out.push_str(&point(p)?);
    }
    out.push_str(&format!("],node_length_m:{},kill_radius_m:{},perception_radius_m:{},cone_angle_rad:{},max_iterations:{},tip_radius_m:{},pipe_exponent:{},ring_slices:{}}}",
        number(r.node_length_m)?, number(r.kill_radius_m)?, number(r.perception_radius_m)?,
        number(r.cone_angle_rad)?, r.max_iterations, number(r.tip_radius_m)?,
        number(r.pipe_exponent)?, r.ring_slices));
    Ok(out)
}
fn organ_source(o: &Organ) -> Result<String, String> {
    Ok(format!(
        "{{key:{},parent:{},position_m:{}}}",
        quoted(&o.key)?,
        o.parent
            .as_deref()
            .map(quoted)
            .transpose()?
            .unwrap_or_else(|| "null".into()),
        point(&o.position_m)?
    ))
}

fn geometry_source(vertices: &[[f64; 3]], triangles: &[[u32; 3]]) -> Result<String, String> {
    let mut out = String::from("{vertices:[");
    for (i, p) in vertices.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        out.push_str(&point(p)?);
    }
    out.push_str("],triangles:[");
    for (i, t) in triangles.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        out.push_str(&format!("[{},{},{}]", t[0], t[1], t[2]));
    }
    out.push_str("]}");
    Ok(out)
}

fn valid_member(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .enumerate()
            .all(|(i, b)| b.is_ascii_alphabetic() || b == b'_' || (i > 0 && b.is_ascii_digit()))
}

fn generated_geometry(vertices: &[[f64; 3]], triangles: &[[u32; 3]]) -> Value {
    let vertex_rows = vertices
        .iter()
        .map(|p| Value::List(p.iter().copied().map(Value::F64).collect()))
        .collect();
    let triangle_rows = triangles
        .iter()
        .map(|t| Value::List(t.iter().map(|n| Value::I64(i64::from(*n))).collect()))
        .collect();
    Value::Record(std::collections::BTreeMap::from([
        ("vertices".to_owned(), Value::List(vertex_rows)),
        ("triangles".to_owned(), Value::List(triangle_rows)),
    ]))
}

/// Checks the exact generator-owned mesh projection before replacing it. An
/// authored normal, UV, skin, or extension stream needs explicit remapping and
/// is refused in this first profile. Other selected-part fields are retained.
pub fn validate_projection(
    parts: &Value,
    current: Option<&BotanicalRecipe>,
    next: &BotanicalRecipe,
    limits: BotanicalLimits,
) -> Result<(), String> {
    next.validate(limits)?;
    let Value::List(rows) = parts else {
        return Err("Botanical edit needs authored parts".into());
    };
    if rows.is_empty() || rows.len() > 256 {
        return Err("Botanical part inventory outside bound".into());
    }
    let mut seen = BTreeSet::new();
    let mut target = None;
    for row in rows {
        let Value::Record(p) = row else {
            return Err("Botanical part is not a record".into());
        };
        let Some(Value::String(id)) = p.get("id") else {
            return Err("Botanical part lacks text id".into());
        };
        if !seen.insert(id.as_str()) {
            return Err("Duplicate authored part id".into());
        }
        if id == &next.part_id {
            target = Some(p);
        }
    }
    let part = target.ok_or("Unknown selected botanical part")?;
    let Some(Value::Record(geometry)) = part.get("geometry") else {
        return Err("Selected botanical part lacks a geometry record".into());
    };
    if geometry
        .keys()
        .any(|key| !["vertices", "triangles"].contains(&key.as_str()))
        || !geometry.contains_key("vertices")
        || !geometry.contains_key("triangles")
    {
        return Err(
            "Botanical edit cannot remap authored normals, UV, skin or geometry extensions".into(),
        );
    }
    if let Some(current) = current {
        if current.part_id != next.part_id {
            return Err("Botanical target change needs migration".into());
        }
        let prior = evaluate(current, limits)?;
        if part.get("geometry") != Some(&generated_geometry(&prior.vertices, &prior.triangles)) {
            return Err(
                "Botanical geometry differs from accepted recipe; preserve manual edits".into(),
            );
        }
    }
    Ok(())
}

/// Returns one source cell; current/source guards and accepted revision checks
/// remain with Poet's native controller. The selected part's non-geometry fields
/// are copied by member reference and every other part is returned unchanged.
pub fn edit_source(
    parts: &Value,
    current: Option<&Value>,
    edit: &BotanicalEdit,
    limits: BotanicalLimits,
) -> Result<String, String> {
    let previous = read_recipe(current, limits)?;
    let recipe = match edit {
        BotanicalEdit::SetRecipe { recipe: r } => {
            if let Some(old) = &previous {
                if old.part_id != r.part_id {
                    return Err("Botanical target change needs explicit migration".into());
                }
            }
            r.clone()
        }
        BotanicalEdit::SetTipRadius {
            part_id,
            tip_radius_m,
        } => {
            let mut r = previous.clone().ok_or("No accepted botanical recipe")?;
            if &r.part_id != part_id {
                return Err("Foreign botanical part target".into());
            }
            r.tip_radius_m = *tip_radius_m;
            r
        }
        BotanicalEdit::SetOrganPosition {
            part_id,
            organ_key,
            position_m,
        } => {
            let mut r = previous.clone().ok_or("No accepted botanical recipe")?;
            if &r.part_id != part_id {
                return Err("Foreign botanical part target".into());
            }
            let organ = r
                .organs
                .iter_mut()
                .find(|o| &o.key == organ_key)
                .ok_or("Unknown adopted organ key")?;
            organ.position_m = *position_m;
            r
        }
    };
    validate_projection(parts, previous.as_ref(), &recipe, limits)?;
    let Value::List(rows) = parts else {
        return Err("Botanical edit needs authored parts".into());
    };
    if rows.is_empty() || rows.len() > 256 {
        return Err("Botanical part inventory outside bound".into());
    }
    let mut selected = None;
    let mut seen = BTreeSet::new();
    for row in rows {
        let Value::Record(p) = row else {
            return Err("Botanical part is not a record".into());
        };
        let Some(Value::String(id)) = p.get("id") else {
            return Err("Botanical part lacks text id".into());
        };
        if !seen.insert(id.as_str()) {
            return Err("Duplicate authored part id".into());
        }
        if id == &recipe.part_id {
            selected = Some(p);
        }
    }
    let selected = selected.ok_or("Unknown selected botanical part")?;
    if !selected.contains_key("geometry") {
        return Err("Selected botanical part lacks geometry".into());
    }
    // `evaluate` invokes the three existing Core kernels before any source is
    // emitted. Failed budgets, cycles and degenerate edges leave accepted source untouched.
    let preview = evaluate(&recipe, limits)?;
    let mesh = geometry_source(&preview.vertices, &preview.triangles)?;
    let fields = selected
        .keys()
        .map(|key| {
            if !valid_member(key) {
                return Err(format!(
                    "Unsupported part field {key}; botanical edit refused"
                ));
            }
            Ok(format!(
                "{key}:{}",
                if key == "geometry" {
                    mesh.clone()
                } else {
                    format!("p.{key}")
                }
            ))
        })
        .collect::<Result<Vec<_>, String>>()?
        .join(",");
    let binding = if current.is_some() {
        "poet_botanical = "
    } else {
        "let mut poet_botanical = "
    };
    let selected_source = format!("{{{fields}}}");
    let source = format!("{binding}{};\npoet_parts = poet_parts.map(|p| if p.id == {} {{ {selected_source} }} else {{p}});\n({{parts:poet_parts}});",
        recipe_source(&recipe)?, quoted(&recipe.part_id)?);
    if source.len() > limits.max_source_bytes {
        return Err("Botanical source cell exceeds host-selected bound".into());
    }
    Ok(source)
}
