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
    let scratch = vertices
        .checked_mul(64)
        .and_then(|v| {
            input_vertices
                .checked_mul(32)
                .and_then(|i| v.checked_add(i))
        })
        .and_then(|v| triangles.checked_mul(32).and_then(|t| v.checked_add(t)))
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
    let mut keys = BTreeMap::new();
    let mut semantics = BTreeSet::new();
    let mut materials = BTreeMap::new();
    for (index, part) in parts.iter().enumerate() {
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
    let materials: Vec<_> = materials.into_values().collect();
    validate_material_bindings(&materials, &ranges, (triangles.len() * 3) as u32)
        .map_err(|e| AuthoringError::Kernel(format!("Core material bindings: {e:?}")))?;
    Ok(CompiledAssembly {
        mesh: Mesh {
            positions,
            triangles,
            min,
            max,
        },
        materials,
        ranges,
        parts: compiled,
        normals,
        uv0,
    })
}
