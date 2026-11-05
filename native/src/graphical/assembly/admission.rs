//! Strict bounded transport projection. Poet spatial authoring owns assembly transforms/topology.
use super::super::{
    materials,
    preview::{mesh_from_invoke, PreviewLimits},
};
use poet_spatial_authoring::{
    compile_assembly, AssemblyLimits, AssemblyPart, CompiledAssembly, Transform,
};
use qualia_core_db::container_10d::validate_material_bindings;
use std::collections::{BTreeMap, BTreeSet};
use vibe::Value;

type Record = BTreeMap<String, Value>;
const MAX_PARTS: usize = 256;
pub(super) fn record(value: &Value) -> Result<&Record, String> {
    match value {
        Value::Record(record) => Ok(record),
        _ => Err("assembly value must be a record".into()),
    }
}
pub(super) fn fields(record: &Record, names: &[&str]) -> Result<(), String> {
    if let Some(key) = record.keys().find(|key| !names.contains(&key.as_str())) {
        return Err(format!("unsupported assembly property '{key}'"));
    }
    Ok(())
}
pub(super) fn get<'a>(record: &'a Record, key: &str) -> Result<&'a Value, String> {
    record
        .get(key)
        .ok_or_else(|| format!("assembly lacks '{key}'"))
}
fn key(value: &Value) -> Result<&str, String> {
    match value {
        Value::String(s) if !s.trim().is_empty() && s.len() <= 4096 && !s.contains('\0') => Ok(s),
        _ => Err("assembly part key must be bounded nonempty text".into()),
    }
}
fn uint(value: &Value) -> Result<u64, String> {
    match value {
        Value::U64(v) => Ok(*v),
        Value::I64(v) if *v >= 0 => Ok(*v as u64),
        _ => Err("assembly index/semantic identifier must be unsigned".into()),
    }
}
fn number(value: &Value) -> Result<f64, String> {
    let v = match value {
        Value::F64(v) => *v,
        Value::I64(v) => *v as f64,
        Value::U64(v) => *v as f64,
        _ => return Err("assembly number required".into()),
    };
    if !v.is_finite() || !(v as f32).is_finite() {
        return Err("assembly numbers must be finite f32-representable".into());
    }
    Ok(v)
}
fn vector<const N: usize>(value: &Value) -> Result<[f64; N], String> {
    let Value::List(values) = value else {
        return Err("assembly vector must be a list".into());
    };
    if values.len() != N {
        return Err(format!("assembly vector needs {N} values"));
    }
    let mut result = [0.0; N];
    for (out, value) in result.iter_mut().zip(values) {
        *out = number(value)?;
    }
    Ok(result)
}
/// Visit nested or flat streams without allocating a flattened intermediate.
fn visit(
    value: &Value,
    width: usize,
    maximum: usize,
    mut check: impl FnMut(usize, &Value) -> Result<(), String>,
) -> Result<usize, String> {
    let Value::List(values) = value else {
        return Err("assembly geometry stream must be a list".into());
    };
    let nested = matches!(values.first(), Some(Value::List(_)));
    let count = if nested {
        values
            .len()
            .checked_mul(width)
            .ok_or("assembly stream size overflow")?
    } else {
        values.len()
    };
    if count == 0 || count > maximum || count % width != 0 {
        return Err("assembly geometry stream exceeds bounds or has invalid width".into());
    }
    let mut index = 0;
    for value in values {
        if nested {
            let Value::List(row) = value else {
                return Err("mixed assembly stream shapes".into());
            };
            if row.len() != width {
                return Err("invalid assembly stream row".into());
            }
            for scalar in row {
                check(index, scalar)?;
                index += 1;
            }
        } else {
            check(index, value)?;
            index += 1;
        }
    }
    Ok(count / width)
}
fn stream<const N: usize>(value: &Value, count: usize) -> Result<Vec<[f32; N]>, String> {
    let mut result = Vec::with_capacity(count);
    let mut row = [0.0; N];
    visit(
        value,
        N,
        count.checked_mul(N).ok_or("assembly stream overflow")?,
        |index, value| {
            row[index % N] = number(value)? as f32;
            if index % N == N - 1 {
                result.push(row);
            }
            Ok(())
        },
    )?;
    Ok(result)
}
pub(super) fn geometry_counts(
    geometry: &Record,
    limits: PreviewLimits,
) -> Result<(usize, usize, usize), String> {
    fields(geometry, &["vertices", "triangles", "normals", "uv0"])?;
    let vertices = visit(
        get(geometry, "vertices")?,
        3,
        limits
            .vertices
            .checked_mul(3)
            .ok_or("vertex budget overflow")?,
        |_, v| number(v).map(|_| ()),
    )?;
    let triangles = visit(get(geometry, "triangles")?, 3, limits.indices, |_, v| {
        if uint(v)? >= vertices as u64 {
            return Err("assembly triangle index out of bounds".into());
        }
        Ok(())
    })?;
    let normals = geometry.get("normals");
    if let Some(normals) = normals {
        let mut normal = [0.0; 3];
        let count = visit(
            normals,
            3,
            vertices.checked_mul(3).ok_or("normal budget overflow")?,
            |i, v| {
                normal[i % 3] = number(v)?;
                if i % 3 == 2 {
                    let length2 = normal.iter().map(|v| v * v).sum::<f64>();
                    if !length2.is_finite() || length2 <= f64::EPSILON {
                        return Err("assembly authored normal is degenerate".into());
                    }
                }
                Ok(())
            },
        )?;
        if count != vertices {
            return Err("assembly normal stream is not vertex-aligned".into());
        }
    }
    if let Some(uv) = geometry.get("uv0") {
        let count = visit(
            uv,
            2,
            vertices.checked_mul(2).ok_or("UV budget overflow")?,
            |_, v| {
                if number(v)?.abs() > 1_000_000.0 {
                    return Err("assembly UV exceeds native bound".into());
                }
                Ok(())
            },
        )?;
        if count != vertices {
            return Err("assembly UV stream is not vertex-aligned".into());
        }
    }
    let output_vertices = if normals.is_some() {
        vertices
    } else {
        triangles
            .checked_mul(3)
            .ok_or("flat-normal expansion overflow")?
    };
    Ok((vertices, triangles, output_vertices))
}

pub fn admit_assembly(value: &Value, limits: PreviewLimits) -> Result<CompiledAssembly, String> {
    admit_assembly_with_materials(value, limits, &BTreeMap::new())
}
pub(crate) fn admit_assembly_with_materials(
    value: &Value,
    limits: PreviewLimits,
    overrides: &BTreeMap<u64, qualia_core_db::container_10d::MaterialRecord>,
) -> Result<CompiledAssembly, String> {
    let outer = record(value)?;
    fields(outer, &["parts", "_poet_source_layer"])?;
    // Recognized inert source metadata is excluded from render semantics. This
    // checks its transport shape only; canonical submit/rebuild must separately
    // validate the layer relation against actual preceding source values before
    // promoting source. Standalone rendering makes no source-layer safety claim.
    crate::graphical::authoring::source_layers::validate_render_envelope(value)?;
    let Value::List(parts) = get(outer, "parts")? else {
        return Err("assembly parts must be a list".into());
    };
    if parts.is_empty() || parts.len() > MAX_PARTS || limits.vertices == 0 || limits.indices < 3 {
        return Err("assembly part/geometry budget invalid".into());
    }
    // Preflight every borrowed stream and total output charge BEFORE cloning meshes.
    let atlas_count=parts.iter().filter(|p|matches!(p,Value::Record(r) if r.get("uv_atlas").is_some_and(|v|!matches!(v,Value::Null)))).count().max(1);
    let mut atlas_limits = crate::graphical::surfaces::inspection::atlas_limits(limits);
    atlas_limits.work_units /= atlas_count as u64;
    let mut input_vertices = 0usize;
    let mut output_vertices = 0usize;
    let mut indices = 0usize;
    let mut ids = BTreeSet::new();
    let mut semantics = BTreeSet::new();
    for part_value in parts {
        let part = record(part_value)?;
        fields(part, super::part_input::PART_FIELDS)?;
        if !ids.insert(key(get(part, "id")?)?) {
            return Err("duplicate assembly part key".into());
        }
        let semantic = uint(get(part, "semantic_id")?)?;
        if semantic == 0 || !semantics.insert(semantic) {
            return Err("assembly semantic identifiers must be nonzero and distinct".into());
        }
        if let Some(parent) = part.get("parent").filter(|p| !matches!(p, Value::Null)) {
            key(parent)?;
        }
        if let Some(v) = part.get("uv_transform") {
            crate::graphical::surfaces::uv::from_value(v)?;
        }
        if let Some(v) = part
            .get("uv_projection")
            .filter(|v| !matches!(v, Value::Null))
        {
            crate::graphical::surfaces::uv::projection_from_value(v)?;
        }
        vector::<3>(get(part, "translation")?)?;
        vector::<3>(get(part, "rotation")?)?;
        if vector::<3>(get(part, "scale")?)?.iter().any(|v| *v <= 0.0) {
            return Err(
                "assembly scale must be strictly positive; reflections are unsupported".into(),
            );
        }
        let material = get(part, "material")?;
        if let Some(name) = record(material)?.get("name") {
            key(name)?;
        }
        materials::material_from_value(material)?;
        let (v, t, out) = super::part_input::preflight(part_value, limits, atlas_limits)?;
        input_vertices = input_vertices
            .checked_add(v)
            .ok_or("assembly input size overflow")?;
        output_vertices = output_vertices
            .checked_add(out)
            .ok_or("assembly output size overflow")?;
        indices = indices
            .checked_add(t.checked_mul(3).ok_or("assembly index overflow")?)
            .ok_or("assembly index overflow")?;
        if input_vertices > limits.vertices
            || output_vertices > limits.vertices
            || indices > limits.indices
        {
            return Err("aggregate assembly exceeds host-selected geometry budget".into());
        }
    }
    let mut native = Vec::with_capacity(parts.len());
    for part in parts {
        let p = record(part)?;
        let geometry = get(p, "geometry")?;
        let g = record(geometry)?;
        let mut mesh = mesh_from_invoke(geometry, limits)?;
        let mut normals = g
            .get("normals")
            .map(|v| stream::<3>(v, mesh.positions.len()))
            .transpose()?;
        let mut uv0 = g
            .get("uv0")
            .map(|v| stream::<2>(v, mesh.positions.len()))
            .transpose()?;
        let mut tangents = None;
        if let Some(v) = p.get("uv_atlas").filter(|v| !matches!(v, Value::Null)) {
            let d = crate::graphical::surfaces::uv_atlas::from_value(v)?;
            let applied = crate::graphical::surfaces::uv_atlas::apply(
                &d,
                &mesh,
                normals.as_deref(),
                atlas_limits,
                &|| false,
            )?;
            mesh = applied.mesh;
            normals = applied.normals;
            uv0 = Some(applied.uv0);
            tangents = applied.tangents;
        }
        if let Some(v) = p.get("uv_projection").filter(|v| !matches!(v, Value::Null)) {
            uv0 = Some(crate::graphical::surfaces::uv::project(
                &crate::graphical::surfaces::uv::projection_from_value(v)?,
                &mesh.positions,
            )?);
        }
        if let Some(v) = p.get("uv_transform") {
            let transform = crate::graphical::surfaces::uv::from_value(v)?;
            if let Some(uv) = uv0.as_mut() {
                crate::graphical::surfaces::uv::apply(&transform, uv)?;
            } else if transform != qualia_core_db::container_10d::TextureTransform::IDENTITY {
                return Err(
                    "UV transform requires authored UV0 or explicit planar projection".into(),
                );
            }
        }
        let authored = materials::material_from_value(get(p, "material")?)?;
        let material = overrides.get(&authored.id).copied().unwrap_or(authored);
        if crate::graphical::surfaces::TextureRole::ALL
            .iter()
            .any(|r| r.digest(&material) != [0; 32])
            && uv0.is_none()
        {
            return Err("Textured part requires authored UV0 or explicit planar projection".into());
        }
        native.push(AssemblyPart {
            key: key(get(p, "id")?)?.into(),
            parent: p
                .get("parent")
                .filter(|p| !matches!(p, Value::Null))
                .map(key)
                .transpose()?
                .map(str::to_owned),
            semantic_id: uint(get(p, "semantic_id")?)?,
            mesh,
            material,
            trs: Transform {
                translation: vector(get(p, "translation")?)?,
                rotation: vector(get(p, "rotation")?)?,
                scale: vector(get(p, "scale")?)?,
            },
            normals,
            tangents,
            uv0,
        });
    }
    let scratch_bytes = limits
        .vertices
        .checked_mul(256)
        .and_then(|v| {
            limits
                .indices
                .checked_mul(24)
                .and_then(|i| v.checked_add(i))
        })
        .and_then(|v| v.checked_add(MAX_PARTS * 8192))
        .ok_or("assembly scratch budget overflow")?;
    let result = compile_assembly(
        &native,
        AssemblyLimits {
            parts: MAX_PARTS,
            vertices: limits.vertices,
            triangles: limits.indices / 3,
            scratch_bytes,
        },
    )
    .map_err(|e| format!("native Poet assembly: {e:?}"))?;
    validate_compiled(&result, limits)?;
    Ok(result)
}

/// Public native structs are also checked when supplied directly by an embedding.
pub fn validate_compiled(a: &CompiledAssembly, limits: PreviewLimits) -> Result<(), String> {
    let vertices = a.mesh.positions.len();
    let index_count = a
        .mesh
        .triangles
        .len()
        .checked_mul(3)
        .ok_or("assembly index count overflow")?;
    if vertices == 0
        || vertices > limits.vertices
        || index_count == 0
        || index_count > limits.indices
        || index_count > u32::MAX as usize
        || a.parts.is_empty()
        || a.parts.len() > MAX_PARTS
        || a.parts.len() != a.ranges.len()
        || a.normals.len() != vertices
        || a.uv0.len() != vertices
        || a.mesh.positions.iter().flatten().any(|v| !v.is_finite())
        || a.mesh
            .min
            .iter()
            .zip(a.mesh.max.iter())
            .any(|(lo, hi)| !lo.is_finite() || !hi.is_finite() || lo > hi)
        || a.mesh
            .positions
            .iter()
            .any(|p| (0..3).any(|i| p[i] < a.mesh.min[i] || p[i] > a.mesh.max[i]))
        || a.normals.iter().flatten().any(|v| !v.is_finite())
        || a.normals.iter().any(|n| {
            let l = n.iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>();
            !l.is_finite() || l <= f64::EPSILON
        })
        || a.uv0
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || v.abs() > 1_000_000.0)
        || a.mesh
            .triangles
            .iter()
            .flatten()
            .any(|i| *i as usize >= vertices)
    {
        return Err("invalid/oversized compiled native assembly".into());
    }
    if let Some(tangents) = &a.tangents {
        poet_spatial_authoring::assembly::validate_tangent_frames(&a.normals,tangents)
            .map_err(|e| format!("invalid compiled tangent frames: {e:?}"))?;
    }
    let mut end = 0u32;
    let mut ids = BTreeSet::new();
    let mut semantics = BTreeSet::new();
    for (part, range) in a.parts.iter().zip(&a.ranges) {
        if part.key.trim().is_empty()
            || part.key.len() > 4096
            || !ids.insert(&part.key)
            || part.semantic_id == 0
            || !semantics.insert(part.semantic_id)
            || part.semantic_id != range.semantic_id
            || part.first_index != range.first_index
            || part.index_count != range.index_count
            || range.first_index != end
        {
            return Err("compiled assembly part/range identities differ".into());
        }
        end = end
            .checked_add(range.index_count)
            .ok_or("assembly range overflow")?;
    }
    if end as usize != index_count {
        return Err("assembly ranges do not cover exact geometry".into());
    }
    for material in &a.materials {
        crate::graphical::surfaces::draft::validate(material)?;
    }
    validate_material_bindings(&a.materials, &a.ranges, index_count as u32)
        .map_err(|e| format!("native assembly material ranges: {e}"))?;
    Ok(())
}
