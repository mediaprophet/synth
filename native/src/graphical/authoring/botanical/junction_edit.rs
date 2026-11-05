//! Staged native Vibe source-cell preparation for the single-junction profile.
//! No submission or publication occurs here. The caller owns authority, revision,
//! host operation accounting and tentative-result validation before promotion.
use super::junction_source::{JunctionSettings, JunctionSource};

use crate::graphical::junction_generate::{JunctionLimits, JunctionScratchResult};
use crate::graphical::geometry_host::GeometryHost;
use std::collections::BTreeSet;
use vibe::Value;
#[path = "junction_attributes.rs"]
mod attributes;

fn push(out: &mut String, text: &str, cap: usize) -> Result<(), String> {
    let len = out.len().checked_add(text.len()).ok_or("Junction source length overflow")?;
    if len > cap { return Err("Junction source exceeds host limit".into()); }
    out.try_reserve(text.len()).map_err(|_| "Junction source allocation refused")?;
    out.push_str(text);
    Ok(())
}
fn valid_member(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.bytes().enumerate().all(|(i,b)|
        b.is_ascii_alphabetic() || b == b'_' || (i > 0 && b.is_ascii_digit()))
}
fn part<'a>(parts: &'a Value, id: &str, host: &mut GeometryHost) -> Result<&'a std::collections::BTreeMap<String, Value>, String> {
    let Value::List(parts) = parts else { return Err("Junction needs parts list".into()); };
    if parts.is_empty() || parts.len() > 256 { return Err("Part inventory outside bound".into()); }
    // Bounded identifier comparisons and BTreeSet entries, reserved before scan.
    let work = parts.len().checked_mul(128 * 10 + 16).ok_or("Part scan overflow")?;
    let bytes = parts.len().checked_mul(128).ok_or("Part scan allocation overflow")?;
    host.reserve_junction_work(work, 0, bytes)?;
    let mut seen = BTreeSet::new(); let mut found = None;
    for part in parts {
        let Value::Record(p) = part else { return Err("Invalid part".into()); };
        let Some(Value::String(key)) = p.get("id") else { return Err("Part lacks identifier".into()); };
        if key.is_empty() || key.len() > 128 { return Err("Part identifier outside bound".into()); }
        if !seen.insert(key.as_str()) { return Err("Duplicate part identifier".into()); }
        if key == id { found = Some(p); }
    }
    let part = found.ok_or("Junction target not found")?;
    if part.len() > 128 { return Err("Part field count outside bound".into()); }
    host.reserve_junction_work(part.len().checked_mul(65).ok_or("Part field work overflow")?, 0, 0)?;
    if part.keys().any(|key| !valid_member(key)) { return Err("Unsupported part field".into()); }
    Ok(part)
}
fn reserve_projection(host: &mut GeometryHost, generated: &JunctionScratchResult) -> Result<(), String> {
    let work = generated.vertices.len().checked_mul(4)
        .and_then(|v| generated.triangles.len().checked_mul(4).and_then(|t| v.checked_add(t)))
        .ok_or("Junction projection work overflow")?;
    host.reserve_junction_work(work, 0, 0)
}
fn same_projection(geometry: &Value, generated: &JunctionScratchResult) -> bool {
    let Value::Record(g) = geometry else { return false; };
    let Some(Value::List(v)) = g.get("vertices") else { return false; };
    let Some(Value::List(t)) = g.get("triangles") else { return false; };
    if v.len() != generated.vertices.len() || t.len() != generated.triangles.len() { return false; }
    let vertices_match = v.iter().zip(&generated.vertices).all(|(row,p)| {
        let Value::List(row) = row else { return false; };
        row.len() == 3 && row.iter().zip([p.x,p.y,p.z]).all(|(v,n)| matches!(v, Value::F64(x) if *x == n))
    });
    let triangles_match = t.iter().zip(&generated.triangles).all(|(row,expected)| {
        let Value::List(row) = row else { return false; };
        row.len() == 3 && row.iter().zip(expected).all(|(v,n)| match v {
            Value::U64(x) => *x == u64::from(*n), Value::I64(x) if *x >= 0 => *x as u64 == u64::from(*n), _ => false
        })
    });
    vertices_match && triangles_match
}

fn parse(host: &mut GeometryHost, value: &Value, limits: JunctionLimits) -> Result<JunctionSource, String> {
    let cap = limits.botanical.max_source_bytes;
    let clones = cap.checked_mul(3).ok_or("Junction source reservation overflow")?;
    crate::graphical::junction_budget::read_accounted(host, value, limits, cap, cap, clones)
}
fn reserve_graph_work(host: &mut GeometryHost, limits: JunctionLimits, multiplier: usize) -> Result<(), String> {
    let cap = limits.botanical.max_source_bytes;
    let work = cap.checked_mul(multiplier).ok_or("Junction graph work overflow")?;
    let bytes = cap.checked_mul(multiplier).ok_or("Junction graph bytes overflow")?;
    host.reserve_junction_work(work, 0, bytes)
}
/// Validate tentative or replayed binding against its exact projection, with
/// all parsing and generation charged to the already-active host operation.
pub(crate) fn validate_binding_projection(parts: &Value, binding: &Value, limits: JunctionLimits,
    host: &mut GeometryHost) -> Result<JunctionSource, String> {
    let source = parse(host, binding, limits)?;
    reserve_graph_work(host, limits, 2)?;
    let graph = source.graph(limits.botanical)?;
    let selected = part(parts, &graph.part_id, host)?;
    attributes::inspect(selected, host, limits)?;
    let generated = crate::graphical::junction_budget::generate_accounted(host, &source, limits)?;
    reserve_projection(host, &generated)?;
    if !same_projection(selected.get("geometry").ok_or("Missing geometry")?, &generated) {
        return Err("Junction binding does not match generated geometry".into());
    }
    attributes::validate_normals(selected.get("geometry").ok_or("Missing geometry")?, &source, &generated, host)?;
    Ok(source)
}

pub(crate) fn prepare_position_edit(parts: &Value, binding: &Value, target: &str, organ: &str,
    position_m: [f64; 3], limits: JunctionLimits, host: &mut GeometryHost) -> Result<String, String> {
    let prior = parse(host, binding, limits)?;
    reserve_graph_work(host, limits, 2)?;
    let graph = prior.graph(limits.botanical)?;
    if graph.part_id != target { return Err("Foreign junction target".into()); }
    let selected = part(parts, target, host)?;
    let attributes = attributes::inspect(selected, host, limits)?;
    let geometry = selected.get("geometry").ok_or("Missing geometry")?;
    let prior_mesh = crate::graphical::junction_budget::generate_accounted(host, &prior, limits)?;
    reserve_projection(host, &prior_mesh)?;
    if !same_projection(geometry, &prior_mesh) { return Err("Junction source geometry drifted; preserve authored edits".into()); }
    attributes::validate_normals(geometry, &prior, &prior_mesh, host)?;
    reserve_graph_work(host, limits, 4)?;
    let next = prior.with_position(organ, position_m, limits.botanical, limits.max_samples)?;
    let next_mesh = crate::graphical::junction_budget::generate_accounted(host, &next, limits)?;
    let transfer = attributes::prepare(&attributes, &prior, &next, &prior_mesh, &next_mesh, host)?;
    source_cell(selected, target, &next, &next_mesh, true, &attributes, transfer.as_ref(), limits, host)
}

/// Update only explicit source settings after checking the exact prior mesh.
pub(crate) fn prepare_settings_edit(parts: &Value, binding: &Value, target: &str,
    settings: &JunctionSettings, limits: JunctionLimits, host: &mut GeometryHost) -> Result<String, String> {
    let prior = parse(host, binding, limits)?;
    reserve_graph_work(host, limits, 2)?;
    let graph = prior.graph(limits.botanical)?;
    if graph.part_id != target { return Err("Foreign junction target".into()); }
    let selected = part(parts, target, host)?;
    let attributes = attributes::inspect(selected, host, limits)?;
    let geometry = selected.get("geometry").ok_or("Missing geometry")?;
    let prior_mesh = crate::graphical::junction_budget::generate_accounted(host, &prior, limits)?;
    reserve_projection(host, &prior_mesh)?;
    if !same_projection(geometry, &prior_mesh) { return Err("Junction source geometry drifted; preserve authored edits".into()); }
    attributes::validate_normals(geometry, &prior, &prior_mesh, host)?;
    reserve_graph_work(host, limits, 4)?;
    let next = prior.with_settings(settings, limits.botanical, limits.max_samples)?;
    let next_mesh = crate::graphical::junction_budget::generate_accounted(host, &next, limits)?;
    let transfer = attributes::prepare(&attributes, &prior, &next, &prior_mesh, &next_mesh, host)?;
    source_cell(selected, target, &next, &next_mesh, true, &attributes, transfer.as_ref(), limits, host)
}
/// First adoption replaces the selected part's base geometry; the controller
/// checks existing bindings and mutability. Source must have been read_accounted.
pub(crate) fn prepare_adoption(parts: &Value, source: &JunctionSource, limits: JunctionLimits,
    host: &mut GeometryHost) -> Result<String, String> {
    reserve_graph_work(host, limits, 2)?;
    let graph = source.graph(limits.botanical)?;
    let selected = part(parts, &graph.part_id, host)?;
    let attributes = attributes::inspect(selected, host, limits)?;
    if attributes.uv0 || attributes.normals || attributes.atlas.is_some() {
        return Err("Initial junction adoption has no prior grid-edge lineage for UV0, normals or atlas seams; preserve the original part and choose an explicit migration".into());
    }
    let mesh = crate::graphical::junction_budget::generate_accounted(host, source, limits)?;
    source_cell(selected, &graph.part_id, source, &mesh, false, &attributes, None, limits, host)
}
fn source_cell(part: &std::collections::BTreeMap<String, Value>, target: &str, source: &JunctionSource,
    mesh: &JunctionScratchResult, existing: bool, attributes: &attributes::Attributes,
    transfer: Option<&attributes::Transfer>, limits: JunctionLimits, host: &mut GeometryHost) -> Result<String, String> {
    // key_valid in the existing recipe admits only ASCII without quotes or escapes.
    let cap = limits.botanical.max_source_bytes;
    let expression = crate::graphical::junction_budget::serialize_accounted(host, source, limits, cap)?;
    let charge = cap.checked_mul(3).ok_or("Junction source output reservation overflow")?;
    host.reserve_junction_work(charge, 0, charge)?;
    let generated_normals = if source.regenerates_normals() {
        Some(attributes::regenerated_normals(mesh, host)?)
    } else { None };
    let mut out = String::new();
    push(&mut out, if existing { "poet_botanical_junction = " } else { "let mut poet_botanical_junction = " }, cap)?;
    push(&mut out, &expression, cap)?;
    push(&mut out, ";\npoet_parts = poet_parts.map(|p| if p.id == \"", cap)?;
    push(&mut out, target, cap)?;
    push(&mut out, "\" { {", cap)?;
    for (i,key) in part.keys().enumerate() {
        if i != 0 { push(&mut out, ",", cap)?; }
        push(&mut out, key, cap)?; push(&mut out, ":", cap)?;
        if key == "uv_atlas" {
            if let Some(atlas) = transfer.and_then(|t| t.atlas.as_ref()) {
                push(&mut out, &crate::graphical::surfaces::uv_atlas::source(atlas)?, cap)?;
                continue;
            }
        }
        if key != "geometry" { push(&mut out, "p.", cap)?; push(&mut out, key, cap)?; continue; }
        push(&mut out, "{vertices:[", cap)?;
        for (i,p) in mesh.vertices.iter().enumerate() {
            if i != 0 { push(&mut out, ",", cap)?; }
            if !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite() { return Err("Nonfinite junction point".into()); }
            push(&mut out, &format!("[{:?},{:?},{:?}]",p.x,p.y,p.z), cap)?;
        }
        push(&mut out, "],triangles:[", cap)?;
        for (i,t) in mesh.triangles.iter().enumerate() {
            if i != 0 { push(&mut out, ",", cap)?; }
            push(&mut out, &format!("[{},{},{}]",t[0],t[1],t[2]), cap)?;
        }
        push(&mut out, "]", cap)?;
        for (key, present) in [("uv0", attributes.uv0), ("normals", attributes.normals && !source.regenerates_normals())] {
            if present {
                attributes::emit_stream(&mut out, key, transfer.ok_or("Missing junction attribute transfer")?, cap)?;
            }
        }
        if let Some(normals) = &generated_normals {
            push(&mut out, ",normals:[", cap)?;
            for (i, n) in normals.iter().enumerate() {
                if i != 0 { push(&mut out, ",", cap)?; }
                push(&mut out, &format!("[{:?},{:?},{:?}]", n.x, n.y, n.z), cap)?;
            }
            push(&mut out, "]", cap)?;
        }
        push(&mut out, "}", cap)?;
    }
    push(&mut out, "} } else {p});\n({parts:poet_parts});", cap)?;
    Ok(out)
}
