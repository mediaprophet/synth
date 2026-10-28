//! Bounded native botanical preview adapter over existing QualiaCore kernels.
//! Adopted organ keys remain source-owned. Colonization suffix is only a proposal.
use super::authoring::botanical::{BotanicalLimits, BotanicalRecipe};
use qualia_core_db::specialized_libs::computational_geometry as cg;

#[derive(Clone, Debug, PartialEq)]
pub struct BotanicalNode {
    pub key: String,
    pub parent_key: Option<String>,
    pub position_m: [f64; 3],
    pub radius_m: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BotanicalPreview {
    pub adopted: Vec<BotanicalNode>,
    /// Positions and parent indices from the existing solver; no persistent IDs
    /// are assigned until a separate explicit adoption maps them to source keys.
    pub proposed_positions_m: Vec<[f64; 3]>,
    pub proposed_parents: Vec<u32>,
    pub iterations_run: usize,
    pub remaining_attractors: usize,
    pub vertices: Vec<[f64; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

pub fn evaluate(
    recipe: &BotanicalRecipe,
    limits: BotanicalLimits,
) -> Result<BotanicalPreview, String> {
    recipe.validate(limits)?;
    let n = recipe.organs.len();
    let mut indexes = std::collections::BTreeMap::new();
    for (i, organ) in recipe.organs.iter().enumerate() {
        indexes.insert(organ.key.as_str(), i);
    }
    let seeds = recipe
        .organs
        .iter()
        .map(|o| cg::Point3::new(o.position_m[0], o.position_m[1], o.position_m[2]))
        .collect::<Vec<_>>();
    let seed_parents = recipe
        .organs
        .iter()
        .map(|o| {
            o.parent
                .as_ref()
                .map(|p| indexes[p.as_str()] as u32)
                .unwrap_or(cg::ROOT_PARENT)
        })
        .collect::<Vec<_>>();
    let attractors = recipe
        .attractors_m
        .iter()
        .map(|p| cg::Point3::new(p[0], p[1], p[2]))
        .collect::<Vec<_>>();
    let cap = limits.max_proposal_nodes;
    let mut nodes = vec![cg::Point3::default(); cap];
    let mut parents = vec![cg::ROOT_PARENT; cap];
    let config = cg::SpaceColonizationConfig {
        node_length: recipe.node_length_m,
        kill_radius: recipe.kill_radius_m,
        perception_radius: recipe.perception_radius_m,
        cone_angle_rad: recipe.cone_angle_rad,
        min_attractors_per_bud: 1,
        max_iterations: recipe.max_iterations,
    };
    let receipt = cg::space_colonization_3d(
        &attractors,
        &seeds,
        &seed_parents,
        &config,
        &mut nodes,
        &mut parents,
    )
    .map_err(|e| format!("Core space colonization refused: {e:?}"))?;
    // The adopted source graph is the prefix copied by Core. Output index is
    // meaningful only for this call and is never made into an organ identifier.
    if nodes[..n] != seeds || parents[..n] != seed_parents {
        return Err("Core colonization changed the adopted prefix".into());
    }
    let mut scratch = vec![0u32; n];
    let mut radii = vec![0.0f64; n];
    cg::pipe_model_radii(
        &parents[..n],
        recipe.tip_radius_m,
        recipe.pipe_exponent,
        &mut scratch,
        &mut radii,
    )
    .map_err(|e| format!("Core pipe model refused: {e:?}"))?;
    if radii
        .iter()
        .any(|r| !r.is_finite() || *r <= 0.0 || *r > 1.0e6)
    {
        return Err("Pipe radii exceed the finite preview profile".into());
    }
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    let section = (0..recipe.ring_slices)
        .map(|i| {
            let angle = std::f64::consts::TAU * (i as f64) / (recipe.ring_slices as f64);
            cg::Point2::new(angle.cos(), angle.sin())
        })
        .collect::<Vec<_>>();
    let zero = cg::Point3::default();
    if recipe.version == 1 {
        for i in 1..n {
            let p = parents[i] as usize;
            let axis = [nodes[p], nodes[i]];
            let stations = [
                cg::ExtrusionStation {
                    scale_x: radii[p],
                    scale_y: radii[p],
                    twist_rad: 0.0,
                },
                cg::ExtrusionStation {
                    scale_x: radii[i],
                    scale_y: radii[i],
                    twist_rad: 0.0,
                },
            ];
            let mut frames = [cg::AffineFrame3 {
                origin: zero,
                e0: zero,
                e1: zero,
                e2: zero,
            }; 2];
            let mut edge_vertices = vec![zero; 2 * recipe.ring_slices + 2];
            let mut edge_triangles = vec![[0u32; 3]; 4 * recipe.ring_slices];
            let (nv, nt) = cg::extrude_profile_along_path(
                &axis,
                &section,
                Some(&stations),
                true,
                true,
                &mut frames,
                &mut edge_vertices,
                &mut edge_triangles,
            )
            .map_err(|e| format!("Core branch sweep refused: {e:?}"))?;
            let offset =
                u32::try_from(vertices.len()).map_err(|_| "Botanical vertex index overflow")?;
            vertices.extend(edge_vertices[..nv].iter().map(|p| [p.x, p.y, p.z]));
            for tri in &edge_triangles[..nt] {
                triangles.push([
                    tri[0].checked_add(offset).ok_or("Index overflow")?,
                    tri[1].checked_add(offset).ok_or("Index overflow")?,
                    tri[2].checked_add(offset).ok_or("Index overflow")?,
                ]);
            }
        }
    } else {
        // Each edge occurs once. A chain starts at the root or a fork, then
        // continues through nodes having exactly one child. Forks remain hard,
        // capped boundaries: this is NOT a joined branching surface.
        let mut child_count = vec![0usize; n];
        let mut only_child = vec![0usize; n];
        for child in 1..n {
            let parent = parents[child] as usize;
            child_count[parent] += 1;
            only_child[parent] = child;
        }
        for child in 1..n {
            let parent = parents[child] as usize;
            if parent != 0 && child_count[parent] == 1 {
                continue;
            }
            let mut chain = vec![parent, child];
            while child_count[*chain.last().unwrap()] == 1 {
                chain.push(only_child[*chain.last().unwrap()]);
            }
            let axis = chain.iter().map(|&i| nodes[i]).collect::<Vec<_>>();
            let stations = chain
                .iter()
                .map(|&i| cg::ExtrusionStation {
                    scale_x: radii[i],
                    scale_y: radii[i],
                    twist_rad: 0.0,
                })
                .collect::<Vec<_>>();
            let required_vertices = chain
                .len()
                .checked_mul(recipe.ring_slices)
                .and_then(|x| x.checked_add(2))
                .ok_or("Botanical chain vertex overflow")?;
            let required_triangles = chain
                .len()
                .checked_mul(recipe.ring_slices)
                .and_then(|x| x.checked_mul(2))
                .ok_or("Botanical chain triangle overflow")?;
            if vertices
                .len()
                .checked_add(required_vertices)
                .filter(|&x| x <= limits.max_vertices)
                .is_none()
                || triangles
                    .len()
                    .checked_add(required_triangles)
                    .filter(|&x| x <= limits.max_triangles)
                    .is_none()
            {
                return Err("Botanical chain exceeds host-selected working set".into());
            }
            let mut frames = vec![
                cg::AffineFrame3 {
                    origin: zero,
                    e0: zero,
                    e1: zero,
                    e2: zero
                };
                chain.len()
            ];
            let mut chain_vertices = vec![zero; required_vertices];
            let mut chain_triangles = vec![[0u32; 3]; required_triangles];
            let (nv, nt) = cg::extrude_profile_along_path(
                &axis,
                &section,
                Some(&stations),
                true,
                true,
                &mut frames,
                &mut chain_vertices,
                &mut chain_triangles,
            )
            .map_err(|e| format!("Core chain sweep refused: {e:?}"))?;
            let offset =
                u32::try_from(vertices.len()).map_err(|_| "Botanical vertex index overflow")?;
            vertices.extend(chain_vertices[..nv].iter().map(|p| [p.x, p.y, p.z]));
            for tri in &chain_triangles[..nt] {
                triangles.push([
                    tri[0].checked_add(offset).ok_or("Index overflow")?,
                    tri[1].checked_add(offset).ok_or("Index overflow")?,
                    tri[2].checked_add(offset).ok_or("Index overflow")?,
                ]);
            }
        }
    }
    if vertices.len() > limits.max_vertices
        || triangles.len() > limits.max_triangles
        || vertices.iter().flatten().any(|v| !v.is_finite())
    {
        return Err("Botanical generated mesh exceeded admitted bounds".into());
    }
    let adopted = recipe
        .organs
        .iter()
        .enumerate()
        .map(|(i, o)| BotanicalNode {
            key: o.key.clone(),
            parent_key: o.parent.clone(),
            position_m: o.position_m,
            radius_m: radii[i],
        })
        .collect();
    let proposed_positions_m = nodes[n..receipt.node_count]
        .iter()
        .map(|p| [p.x, p.y, p.z])
        .collect();
    let proposed_parents = parents[n..receipt.node_count].to_vec();
    Ok(BotanicalPreview {
        adopted,
        proposed_positions_m,
        proposed_parents,
        iterations_run: receipt.iterations_run,
        remaining_attractors: receipt.remaining_attractors,
        vertices,
        triangles,
    })
}

#[cfg(test)]
pub(crate) mod legacy_v1_snapshot {
    include!("botanical_v1_compat_snapshot.rs");
}
