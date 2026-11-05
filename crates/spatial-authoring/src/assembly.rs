//! Identified local-parent assemblies. Canonical source/history stay with the host.
use crate::{AuthoringError, frames};
use qualia_core_db::{
    container_10d::{MaterialRecord, SubmeshRange, validate_material_bindings},
    render::assets::Mesh,
    specialized_libs::computational_geometry::authoring::{self, Mat4},
};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub translation: [f64; 3],
    pub rotation: [f64; 3],
    pub scale: [f64; 3],
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            rotation: [0.0; 3],
            scale: [1.0; 3],
        }
    }
}
impl Transform {
    fn matrix(&self) -> Result<Mat4, AuthoringError> {
        if self
            .translation
            .iter()
            .chain(&self.rotation)
            .chain(&self.scale)
            .any(|v| !v.is_finite())
            || self.scale.iter().any(|v| *v <= 0.0)
        {
            return Err(AuthoringError::Invalid(
                "TRS needs finite metres/radians and strictly positive scale",
            ));
        }
        let (t, r, s) = (self.translation, self.rotation, self.scale);
        Ok(authoring::compose_trs(
            t[0], t[1], t[2], r[0], r[1], r[2], s[0], s[1], s[2],
        ))
    }
}
#[derive(Clone, Debug)]
pub struct AssemblyPart {
    pub key: String,
    pub parent: Option<String>,
    pub semantic_id: u64,
    pub mesh: Mesh,
    pub material: MaterialRecord,
    pub trs: Transform,
    pub normals: Option<Vec<[f32; 3]>>,
    /// Explicit part-local frames, paired with normals and real UV0. No fallback
    /// frames are admitted as authored/atlas tangents.
    pub tangents: Option<Vec<[f32; 4]>>,
    pub uv0: Option<Vec<[f32; 2]>>,
}
#[derive(Clone, Copy, Debug)]
pub struct AssemblyLimits {
    pub parts: usize,
    pub vertices: usize,
    pub triangles: usize,
    pub scratch_bytes: usize,
}
impl Default for AssemblyLimits {
    fn default() -> Self {
        Self {
            parts: 4096,
            vertices: 2_000_000,
            triangles: 4_000_000,
            scratch_bytes: 256 * 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug)]
pub struct CompiledPart {
    pub key: String,
    pub parent: Option<String>,
    pub semantic_id: u64,
    pub first_index: u32,
    pub index_count: u32,
}
#[derive(Clone, Debug)]
pub struct CompiledAssembly {
    pub mesh: Mesh,
    pub materials: Vec<MaterialRecord>,
    pub ranges: Vec<SubmeshRange>,
    pub parts: Vec<CompiledPart>,
    pub normals: Vec<[f32; 3]>,
    /// Complete derivative stream or absent. Mixed UV-less parts refuse when
    /// another part requests retained frames; no sentinel or fabricated UVs.
    pub tangents: Option<Vec<[f32; 4]>>,
    pub uv0: Vec<[f32; 2]>,
}

fn preflight(
    parts: &[AssemblyPart],
    limits: AssemblyLimits,
) -> Result<(usize, usize), AuthoringError> {
    if parts.is_empty() || parts.len() > limits.parts || parts.len() > 4096 {
        return Err(AuthoringError::Budget("assembly part budget exceeded"));
    }
    let (mut vertices, mut triangles, mut input_vertices, mut text) =
        (0usize, 0usize, 0usize, 0usize);
    for part in parts {
        if part.key.trim().is_empty()
            || part.key.len() > 4096
            || part
                .parent
                .as_ref()
                .is_some_and(|p| p.is_empty() || p.len() > 4096)
            || part.semantic_id == 0
        {
            return Err(AuthoringError::Invalid(
                "assembly needs bounded part keys and nonzero semantic identifiers",
            ));
        }
        let add = if part.normals.is_some() {
            part.mesh.positions.len()
        } else {
            part.mesh
                .triangles
                .len()
                .checked_mul(3)
                .ok_or(AuthoringError::Budget("flat normal vertex overflow"))?
        };
        vertices = vertices
            .checked_add(add)
            .ok_or(AuthoringError::Budget("assembly vertex overflow"))?;
        triangles = triangles
            .checked_add(part.mesh.triangles.len())
            .ok_or(AuthoringError::Budget("assembly triangle overflow"))?;
        input_vertices = input_vertices
            .checked_add(part.mesh.positions.len())
            .ok_or(AuthoringError::Budget("assembly input overflow"))?;
        text = text
            .checked_add(part.key.len() + part.parent.as_ref().map_or(0, String::len))
            .ok_or(AuthoringError::Budget("assembly identifier bytes overflow"))?;
    }
    let with_frames = parts.iter().any(|p| p.tangents.is_some());
    let scratch = vertices
        .checked_mul(if with_frames { 160 } else { 64 })
        .and_then(|v| {
            input_vertices
                .checked_mul(32)
                .and_then(|i| v.checked_add(i))
        })
        .and_then(|v| triangles.checked_mul(if with_frames { 48 } else { 32 }).and_then(|t| v.checked_add(t)))
        .and_then(|v| parts.len().checked_mul(512).and_then(|p| v.checked_add(p)))
        .and_then(|v| text.checked_mul(4).and_then(|t| v.checked_add(t)));
    if vertices == 0
        || triangles == 0
        || vertices > limits.vertices
        || triangles > limits.triangles
        || vertices > u32::MAX as usize
        || triangles > u32::MAX as usize / 3
        || scratch.is_none_or(|b| b > limits.scratch_bytes)
    {
        return Err(AuthoringError::Budget(
            "expanded assembly exceeds caller vertex/triangle/scratch budget",
        ));
    }
    Ok((vertices, triangles))
}
pub fn compile_assembly(
    parts: &[AssemblyPart],
    limits: AssemblyLimits,
) -> Result<CompiledAssembly, AuthoringError> {
    let (nv, nt) = preflight(parts, limits)?;
    let with_frames = parts.iter().any(|p| p.tangents.is_some());
    let mut keys = BTreeMap::new();
    let mut semantics = BTreeSet::new();
    let mut materials = BTreeMap::new();
    for (index, part) in parts.iter().enumerate() {
        if with_frames && part.uv0.is_none() {
            return Err(AuthoringError::Invalid("complete tangent delivery requires real UV0 on every part; mixed UV-less assembly is unsupported"));
        }
        if let Some(tangents) = &part.tangents {
            let normals = part.normals.as_deref().ok_or(AuthoringError::Invalid("explicit tangents require aligned source normals"))?;
            if normals.len() != part.mesh.positions.len() || tangents.len() != part.mesh.positions.len() {
                return Err(AuthoringError::Invalid("explicit tangent/normal count differs from source vertices"));
            }
            validate_tangent_frames(normals, tangents)?;
        }
        if keys.insert(part.key.as_str(), index).is_some() || !semantics.insert(part.semantic_id) {
            return Err(AuthoringError::Invalid(
                "duplicate part key or semantic identifier",
            ));
        }
        if let Some(old) = materials.insert(part.material.id, part.material) {
            if old != part.material {
                return Err(AuthoringError::Invalid(
                    "material identifier has conflicting definitions",
                ));
            }
        }
        if part.mesh.positions.is_empty()
            || part.mesh.triangles.is_empty()
            || part.mesh.positions.iter().flatten().any(|v| !v.is_finite())
            || part
                .mesh
                .triangles
                .iter()
                .flatten()
                .any(|&v| v as usize >= part.mesh.positions.len())
        {
            return Err(AuthoringError::Invalid(
                "assembly mesh is empty, nonfinite or index-invalid",
            ));
        }
        if part
            .normals
            .as_ref()
            .is_some_and(|n| n.len() != part.mesh.positions.len())
            || part.uv0.as_ref().is_some_and(|uv| {
                uv.len() != part.mesh.positions.len() || uv.iter().flatten().any(|v| !v.is_finite())
            })
        {
            return Err(AuthoringError::Invalid(
                "assembly frame/UV stream does not match vertices",
            ));
        }
    }
    let mut world: Vec<Option<Mat4>> = vec![None; parts.len()];
    let mut unresolved = parts.len();
    while unresolved > 0 {
        let before = unresolved;
        for (index, part) in parts.iter().enumerate() {
            if world[index].is_some() {
                continue;
            }
            let local = part.trs.matrix()?;
            let parent = match part.parent.as_deref() {
                Some(parent) => {
                    let parent = *keys
                        .get(parent)
                        .ok_or(AuthoringError::Invalid("assembly parent key is missing"))?;
                    let Some(matrix) = world[parent] else {
                        continue;
                    };
                    matrix
                }
                None => authoring::identity(),
            };
            let matrix = authoring::mat_mul(&parent, &local);
            frames::record(&matrix, part.semantic_id)?;
            world[index] = Some(matrix);
            unresolved -= 1;
        }
        if before == unresolved {
            return Err(AuthoringError::Invalid("assembly parent containment cycle"));
        }
    }
    let mut positions = Vec::with_capacity(nv);
    let mut triangles = Vec::with_capacity(nt);
    let mut normals = Vec::with_capacity(nv);
    let mut uv0 = Vec::with_capacity(nv);
    let mut ranges = Vec::with_capacity(parts.len());
    let mut compiled = Vec::with_capacity(parts.len());
    let (mut min, mut max) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for (index, part) in parts.iter().enumerate() {
        let matrix =
            world[index].ok_or(AuthoringError::Invalid("unresolved assembly transform"))?;
        let transformed = authoring::transform_mesh(&part.mesh, &matrix);
        if transformed
            .positions
            .iter()
            .flatten()
            .any(|v| !v.is_finite())
        {
            return Err(AuthoringError::Invalid(
                "transformed assembly exceeds finite f32 domain",
            ));
        }
        let first_index = (triangles.len() * 3) as u32;
        let base = positions.len() as u32;
        let frame = frames::record(&matrix, part.semantic_id)?;
        if let Some(authored) = &part.normals {
            for n in authored {
                normals.push(frames::normal(&frame, *n)?);
            }
            for triangle in &transformed.triangles {
                frames::face(
                    transformed.positions[triangle[0] as usize],
                    transformed.positions[triangle[1] as usize],
                    transformed.positions[triangle[2] as usize],
                )?;
                triangles.push(triangle.map(|i| base + i));
            }
            positions.extend_from_slice(&transformed.positions);
            match &part.uv0 {
                Some(uv) => uv0.extend_from_slice(uv),
                None => uv0.resize(positions.len(), [0.0; 2]),
            }
        } else {
            for triangle in &transformed.triangles {
                let (a, b, c) = (
                    transformed.positions[triangle[0] as usize],
                    transformed.positions[triangle[1] as usize],
                    transformed.positions[triangle[2] as usize],
                );
                let normal = frames::face(a, b, c)?;
                let base = positions.len() as u32;
                triangles.push([base, base + 1, base + 2]);
                for &i in triangle {
                    positions.push(transformed.positions[i as usize]);
                    normals.push(normal);
                    uv0.push(part.uv0.as_ref().map_or([0.0; 2], |uv| uv[i as usize]));
                }
            }
        }
        let index_count = (triangles.len() * 3) as u32 - first_index;
        ranges.push(SubmeshRange {
            first_index,
            index_count,
            material_id: part.material.id,
            semantic_id: part.semantic_id,
        });
        compiled.push(CompiledPart {
            key: part.key.clone(),
            parent: part.parent.clone(),
            semantic_id: part.semantic_id,
            first_index,
            index_count,
        });
    }
    for p in &positions {
        for a in 0..3 {
            min[a] = min[a].min(p[a]);
            max[a] = max[a].max(p[a]);
        }
    }
    let mesh = Mesh { positions, triangles, min, max };
    let tangents = if with_frames {
        // Strict Core derivation validates actual transformed geometry/UVs and
        // fills only unspecified part frames. Never use fallback UV frames.
        let mut output = derive_tangent_frames(&mesh, &normals, &uv0, limits.scratch_bytes)?;
        let mut first_vertex = 0usize;
        for (index, part) in parts.iter().enumerate() {
            if let Some(input) = &part.tangents {
                let matrix = world[index].ok_or(AuthoringError::Invalid("unresolved tangent world transform"))?;
                for (i, &t) in input.iter().enumerate() {
                    output[first_vertex + i] = frames::tangent(&matrix, normals[first_vertex + i], t)?;
                }
            }
            first_vertex += if part.normals.is_some() { part.mesh.positions.len() } else { part.mesh.triangles.len() * 3 };
        }
        validate_tangent_frames(&normals, &output)?;
        Some(output)
    } else { None };
    let materials: Vec<_> = materials.into_values().collect();
    validate_material_bindings(&materials, &ranges, (mesh.triangles.len() * 3) as u32)
        .map_err(|e| AuthoringError::Kernel(format!("Core material bindings: {e:?}")))?;
    Ok(CompiledAssembly {
        mesh,
        materials,
        ranges,
        parts: compiled,
        normals,
        tangents,
        uv0,
    })
}

/// Typed frame admission shared by assembly, decoded .10d and retained poses.
/// This proves finite unit/perpendicular tangent and signed handedness; it does
/// not infer whether an explicitly authored orientation matches texture intent.
pub fn validate_tangent_frames(normals: &[[f32; 3]], tangents: &[[f32; 4]]) -> Result<(), AuthoringError> {
    if tangents.len() != normals.len() { return Err(AuthoringError::Invalid("tangents are not normal-aligned")); }
    for (normal, t) in normals.iter().zip(tangents) {
        let n = frames::unit(normal.map(f64::from))?;
        let length: f64 = t[..3].iter().map(|v| f64::from(*v).powi(2)).sum();
        let dot: f64 = (0..3).map(|i| f64::from(n[i]) * f64::from(t[i])).sum();
        if t.iter().any(|v| !v.is_finite()) || (length - 1.0).abs() > 1e-3
            || dot.abs() > 1e-3 || !matches!(t[3], -1.0 | 1.0) {
            return Err(AuthoringError::Invalid("tangent must be finite unit perpendicular with handedness +/-1"));
        }
    }
    Ok(())
}

/// Explicit strict derivative: undefined/degenerate UVs refuse instead of
/// creating arbitrary rendering fallback tangents. Caller bounds scene counts
/// and reserves the returned logical scratch envelope in its own operation.
pub fn tangent_scratch_bytes(vertices: usize, triangles: usize) -> Result<usize, AuthoringError> {
    vertices.checked_mul(56).and_then(|n| triangles.checked_mul(12).and_then(|t| n.checked_add(t)))
        .ok_or(AuthoringError::Budget("tangent scratch overflow"))
}
pub fn derive_tangent_frames(mesh: &Mesh, normals: &[[f32; 3]], uv0: &[[f32; 2]],
    max_scratch_bytes: usize) -> Result<Vec<[f32; 4]>, AuthoringError> {
    let n = mesh.positions.len(); let f = mesh.triangles.len();
    if tangent_scratch_bytes(n,f)? > max_scratch_bytes { return Err(AuthoringError::Budget("strict tangent scratch exceeds caller budget")); }
    fn filled<T: Clone>(count: usize, value: T) -> Result<Vec<T>, AuthoringError> {
        let mut v = Vec::new(); v.try_reserve_exact(count).map_err(|_| AuthoringError::Budget("tangent allocation refused"))?;
        v.resize(count,value); Ok(v)
    }
    let mut output = filled(n,[0.0f32;4])?;
    let mut temporary = filled(n,[0.0f32;4])?;
    let mut scratch = filled(n,[0.0f32;6])?;
    let count = f.checked_mul(3).ok_or(AuthoringError::Budget("tangent index overflow"))?;
    let mut indices = Vec::new(); indices.try_reserve_exact(count).map_err(|_| AuthoringError::Budget("tangent index allocation refused"))?;
    for t in &mesh.triangles { indices.extend_from_slice(t); }
    let work = u64::try_from(n).ok().and_then(|n| u64::try_from(f).ok().and_then(|f| n.checked_add(f)))
        .and_then(|n| n.checked_mul(100)).ok_or(AuthoringError::Budget("tangent work overflow"))?;
    qualia_core_db::render::mesh_tangent_frames::tangents_from_uvs_strict_into(
        &mesh.positions, uv0, normals, &indices, work, &mut output, &mut temporary, &mut scratch)
        .map_err(|e| AuthoringError::Kernel(format!("strict assembly tangents refused; retain previous source: {e:?}")))?;
    validate_tangent_frames(normals,&output)?;
    Ok(output)
}
