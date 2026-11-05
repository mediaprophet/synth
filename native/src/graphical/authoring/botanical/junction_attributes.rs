//! Attribute admission for the existing single-junction source transaction.
//! Grid edges identify local extraction vertices, never botanical organs. A
//! correspondence is valid only in the same source grid and oriented topology.
//! No nearest-neighbour transfer, automatic seam loss, or normal-policy inference.
use super::{push, JunctionLimits, JunctionScratchResult, JunctionSource};
use crate::graphical::{geometry_host::GeometryHost, surfaces::{uv, uv_atlas}};
use std::collections::{BTreeMap, BTreeSet};
use vibe::Value;

pub(super) struct Attributes {
    pub uv0: bool,
    pub normals: bool,
    pub atlas: Option<uv_atlas::AtlasDraft>,
}

pub(super) struct Transfer {
    /// Destination vertex -> prior raw source vertex. Not an organ assignment.
    pub source_vertices: Vec<usize>,
    pub atlas: Option<uv_atlas::AtlasDraft>,
}

pub(super) fn inspect(part: &BTreeMap<String, Value>, host: &mut GeometryHost,
    limits: JunctionLimits) -> Result<Attributes, String> {
    let Some(Value::Record(g)) = part.get("geometry") else { return Err("Missing part geometry".into()); };
    if !g.contains_key("vertices") || !g.contains_key("triangles") || g.len() > 4
        || g.keys().any(|k| !["vertices", "triangles", "normals", "uv0"].contains(&k.as_str())) {
        return Err("Junction cannot remap skin, tangents, morphs or unknown geometry streams".into());
    }
    let Some(Value::List(vertices)) = g.get("vertices") else { return Err("Junction vertices must be rows".into()); };
    if vertices.len() > limits.max_vertices { return Err("Junction attribute vertex limit exceeded".into()); }
    let work = vertices.len().checked_mul(16).ok_or("Junction attribute work overflow")?;
    host.reserve_junction_work(work, 0, 0)?;
    for (key, width) in [("normals", 3), ("uv0", 2)] {
        if let Some(value) = g.get(key) {
            let Value::List(rows) = value else { return Err(format!("Junction {key} requires vertex-aligned rows")); };
            if rows.len() != vertices.len() { return Err(format!("Junction {key} is not vertex-aligned")); }
            for row in rows {
                let Value::List(row) = row else { return Err(format!("Junction {key} requires rows; flat stream migration is not supported")); };
                if row.len() != width { return Err(format!("Junction {key} row width mismatch")); }
                let mut norm = 0.0f64;
                for value in row {
                    let n = f64::from(uv::number(value)?);
                    if key == "uv0" && n.abs() > 1_000_000.0 { return Err("Junction UV exceeds native bound".into()); }
                    norm += n * n;
                }
                if key == "normals" && norm <= f64::EPSILON { return Err("Junction authored normal is degenerate".into()); }
            }
        }
    }
    let projection = part.get("uv_projection").filter(|v| !matches!(v, Value::Null));
    if let Some(value) = projection { uv::projection_from_value(value)?; }
    let transform = part.get("uv_transform").filter(|v| !matches!(v, Value::Null))
        .map(uv::from_value).transpose()?;
    let atlas = part.get("uv_atlas").filter(|v| !matches!(v, Value::Null));
    let atlas = if let Some(value) = atlas {
        // Charge before the shared parser allocates the authored seam vector.
        let Value::Record(r) = value else { return Err("Junction UV atlas must be a record".into()); };
        let Some(Value::List(seams)) = r.get("seams") else { return Err("Junction atlas lacks seam list".into()); };
        if seams.len() > 65_536 { return Err("Junction atlas seam limit exceeded".into()); }
        host.reserve_junction_work(seams.len().checked_mul(16).ok_or("Seam work overflow")?, 0,
            seams.len().checked_mul(16).ok_or("Seam bytes overflow")?)?;
        let draft = uv_atlas::from_value(value)?;
        if draft.seams.iter().any(|s| s[1] as usize >= vertices.len()) {
            return Err("Junction atlas seam exceeds source vertex inventory".into());
        }
        if projection.is_some() || transform.is_some_and(|t|
            t != qualia_core_db::container_10d::TextureTransform::IDENTITY) {
            return Err("Junction atlas cannot coexist with planar projection or nonidentity UV transform".into());
        }
        Some(draft)
    } else { None };
    if atlas.is_none() && projection.is_none() && !g.contains_key("uv0")
        && transform.is_some_and(|t| t != qualia_core_db::container_10d::TextureTransform::IDENTITY) {
        return Err("Junction UV transform needs retained UV0 or explicit planar projection".into());
    }
    Ok(Attributes { uv0: g.contains_key("uv0"), normals: g.contains_key("normals"), atlas })
}

fn oriented(t: [usize; 3]) -> [usize; 3] {
    t.min([t[1], t[2], t[0]]).min([t[2], t[0], t[1]])
}

/// Exact finite correspondence only. Same counts alone are never sufficient.
pub(super) fn correspondence(prior: &JunctionScratchResult, next: &JunctionScratchResult)
    -> Result<Vec<usize>, String> {
    if prior.vertices.len() != next.vertices.len() || prior.triangles.len() != next.triangles.len()
        || prior.sample_edges.len() != prior.vertices.len() || next.sample_edges.len() != next.vertices.len() {
        return Err("Junction topology changed: UV0/normals/atlas require explicit remeshing policy".into());
    }
    let mut sources = BTreeMap::new();
    for (i, &edge) in prior.sample_edges.iter().enumerate() {
        if edge[0] >= edge[1] || sources.insert(edge, i).is_some() {
            return Err("Invalid or duplicate prior junction grid-edge lineage".into());
        }
    }
    let mut mapping = Vec::new();
    mapping.try_reserve_exact(next.vertices.len()).map_err(|_| "Junction correspondence allocation refused")?;
    for edge in &next.sample_edges {
        mapping.push(sources.remove(edge).ok_or("Junction grid-edge lineage changed or duplicated; authored attributes retained")?);
    }
    let mut faces = BTreeSet::new();
    for t in &prior.triangles {
        if t.iter().any(|&i| i as usize >= prior.vertices.len())
            || !faces.insert(oriented(t.map(|i| i as usize))) {
            return Err("Invalid prior junction triangle lineage".into());
        }
    }
    for t in &next.triangles {
        if t.iter().any(|&i| i as usize >= mapping.len())
            || !faces.remove(&oriented(t.map(|i| mapping[i as usize]))) {
            return Err("Junction oriented triangle topology changed; authored attributes retained".into());
        }
    }
    Ok(mapping)
}

pub(super) fn prepare(attributes: &Attributes, prior_source: &JunctionSource, next_source: &JunctionSource,
    prior: &JunctionScratchResult, next: &JunctionScratchResult, host: &mut GeometryHost)
    -> Result<Option<Transfer>, String> {
    let authored_normals = attributes.normals && !prior_source.regenerates_normals();
    if !attributes.uv0 && !authored_normals && attributes.atlas.is_none() { return Ok(None); }
    if prior_source.origin_m != next_source.origin_m || prior_source.spacing_m != next_source.spacing_m
        || prior_source.dimensions != next_source.dimensions {
        return Err("Junction grid changed: UV0/normals/atlas lack corresponding sample-edge identifiers".into());
    }
    // Conservative logical reservation includes map/set nodes and seam copies.
    // This is structural work / logical storage, not an allocator or RSS limit.
    let n = prior.vertices.len().checked_add(next.vertices.len())
        .and_then(|n| n.checked_add(prior.triangles.len()))
        .and_then(|n| n.checked_add(next.triangles.len()))
        .and_then(|n| n.checked_add(attributes.atlas.as_ref().map_or(0, |a| a.seams.len())))
        .ok_or("Junction correspondence size overflow")?;
    host.reserve_junction_work(n.checked_mul(512).ok_or("Junction correspondence work overflow")?, 0,
        n.checked_mul(160).ok_or("Junction correspondence bytes overflow")?)?;
    let source_vertices = correspondence(prior, next)?;
    if authored_normals && source_vertices.iter().enumerate().any(|(i, &j)| {
        let a = prior.vertices[j]; let b = next.vertices[i];
        a.x != b.x || a.y != b.y || a.z != b.z
    }) {
        return Err("Junction moves vertices with authored normals: choose an explicit regenerated-normal recipe before editing; normals were not discarded".into());
    }
    let atlas = if let Some(atlas) = &attributes.atlas {
        let mut old_to_new = vec![0u32; source_vertices.len()];
        for (new, &old) in source_vertices.iter().enumerate() {
            old_to_new[old] = u32::try_from(new).map_err(|_| "Junction seam index overflow")?;
        }
        let mut mapped = atlas.clone();
        for seam in &mut mapped.seams {
            let a = *old_to_new.get(seam[0] as usize).ok_or("Junction seam source missing")?;
            let b = *old_to_new.get(seam[1] as usize).ok_or("Junction seam source missing")?;
            *seam = [a.min(b), a.max(b)];
        }
        mapped.seams.sort_unstable();
        mapped.options()?;
        Some(mapped)
    } else { None };
    Ok(Some(Transfer { source_vertices, atlas }))
}

/// Reuse Core's area-weighted normals on the unsplit extracted topology. This
/// is a declared shading derivative, not preserved authored data or biology.
pub(super) fn regenerated_normals(mesh: &JunctionScratchResult, host: &mut GeometryHost)
    -> Result<Vec<qualia_core_db::specialized_libs::computational_geometry::Point3>, String> {
    use qualia_core_db::specialized_libs::computational_geometry::{Point3,
        ddg_operators::{face_vector_areas, vertex_normals}};
    let count = mesh.vertices.len().checked_add(mesh.triangles.len()).ok_or("Normal work overflow")?;
    host.reserve_junction_work(count.checked_mul(128).ok_or("Normal work overflow")?, 2,
        count.checked_mul(std::mem::size_of::<Point3>()).ok_or("Normal bytes overflow")?)?;
    let zero = Point3::new(0.0, 0.0, 0.0);
    let mut faces = Vec::new();
    faces.try_reserve_exact(mesh.triangles.len()).map_err(|_| "Junction face-normal allocation refused")?;
    faces.resize(mesh.triangles.len(), zero);
    face_vector_areas(&mesh.vertices, &mesh.triangles, &mut faces)
        .map_err(|e| format!("Junction smooth face normals refused: {e:?}"))?;
    let valid = |p: &Point3| {
        let n = p.x * p.x + p.y * p.y + p.z * p.z;
        n.is_finite() && n > 0.0 && p.x.is_finite() && p.y.is_finite() && p.z.is_finite()
    };
    if !faces.iter().all(valid) { return Err("Junction has undefined face normals".into()); }
    drop(faces);
    let mut normals = Vec::new();
    normals.try_reserve_exact(mesh.vertices.len()).map_err(|_| "Junction normal allocation refused")?;
    normals.resize(mesh.vertices.len(), zero);
    vertex_normals(&mesh.vertices, &mesh.triangles, &mut normals)
        .map_err(|e| format!("Junction smooth vertex normals refused: {e:?}"))?;
    if !normals.iter().all(valid) || normals.iter().any(|n|
        [n.x, n.y, n.z].iter().any(|v| !(*v as f32).is_finite())) {
        return Err("Junction has undefined smooth vertex normals".into());
    }
    Ok(normals)
}

/// A v2 normal is regenerable only because the source explicitly declares the
/// policy AND the current projection matches it. Authored edits fail closed.
pub(super) fn validate_normals(geometry: &Value, source: &JunctionSource,
    mesh: &JunctionScratchResult, host: &mut GeometryHost) -> Result<(), String> {
    if !source.regenerates_normals() { return Ok(()); }
    let normals = regenerated_normals(mesh, host)?;
    let Value::Record(g) = geometry else { return Err("Missing junction geometry".into()); };
    let Some(Value::List(rows)) = g.get("normals") else { return Err("Junction v2 requires its generated normal projection".into()); };
    if rows.len() != normals.len() || !rows.iter().zip(normals).all(|(row, n)| {
        matches!(row, Value::List(xs) if xs.len() == 3 && xs.iter().zip([n.x,n.y,n.z])
            .all(|(v,n)| matches!(v, Value::F64(x) if *x == n)))
    }) { return Err("Junction generated normals drifted; authored normal edits were retained".into()); }
    Ok(())
}

/// Expressions retain exact authored Value variants by reading prior rows.
pub(super) fn emit_stream(out: &mut String, key: &str, transfer: &Transfer, cap: usize) -> Result<(), String> {
    push(out, &format!(",{key}:["), cap)?;
    for (i, old) in transfer.source_vertices.iter().enumerate() {
        if i != 0 { push(out, ",", cap)?; }
        push(out, &format!("p.geometry.{key}[{old}]"), cap)?;
    }
    push(out, "]", cap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use qualia_core_db::specialized_libs::computational_geometry::{Point3,
        surface_topology_qualification::SurfaceQualificationReceipt};
    // Receipt is inert fixture data, not a claim this open triangle qualifies.
    fn triangle() -> JunctionScratchResult {
        JunctionScratchResult {
            vertices: vec![Point3::new(0.,0.,0.), Point3::new(1.,0.,0.), Point3::new(0.,1.,0.)],
            triangles: vec![[0,1,2]], sample_edges: vec![[0,1],[1,2],[2,3]],
            topology: SurfaceQualificationReceipt { vertex_count:3, face_count:1, half_edge_count:3, component_count:1 },
        }
    }
    fn host() -> GeometryHost {
        let mut host = GeometryHost::new(["capability.invoke", "ComputationalGeometry.space_colonize_3d",
            "ComputationalGeometry.pipe_model_radii", "ComputationalGeometry.extrude_profile"]
            .into_iter().map(str::to_owned).collect(), 200_000);
        host.begin_junction_operation().unwrap(); host
    }
    fn source() -> JunctionSource {
        use crate::graphical::authoring::botanical::{BotanicalRecipe, BotanicalLimits, Organ};
        let recipe = BotanicalRecipe { version:1, part_id:"branch".into(), organs:vec![
            Organ {key:"r".into(),parent:None,position_m:[0.,0.,-1.]},
            Organ {key:"c".into(),parent:Some("r".into()),position_m:[0.,0.,0.]},
            Organ {key:"a".into(),parent:Some("c".into()),position_m:[1.,0.,1.]},
            Organ {key:"b".into(),parent:Some("c".into()),position_m:[-1.,0.,1.]},
        ], attractors_m:vec![[0.,1.,1.]], node_length_m:0.2, kill_radius_m:0.05,
            perception_radius_m:0.8, cone_angle_rad:2., max_iterations:1,
            tip_radius_m:0.35, pipe_exponent:2., ring_slices:8 };
        JunctionSource::from_recipe(&recipe,[-2.03,-2.02,-2.04],0.5,[9,9,9],0.2,1e-6,1e-8,0.99,1e-20,
            BotanicalLimits::default(),4096).unwrap()
    }
    #[test]
    fn provenance_reordering_preserves_winding_but_reversal_and_missing_edges_refuse() {
        let old = triangle(); let mut new = triangle();
        new.vertices.swap(0,2); new.sample_edges.swap(0,2); new.triangles = vec![[2,1,0]];
        assert_eq!(correspondence(&old,&new).unwrap(), vec![2,1,0]);
        new.triangles = vec![[2,0,1]];
        assert!(correspondence(&old,&new).unwrap_err().contains("oriented triangle"));
        new.triangles = vec![[2,1,0]]; new.sample_edges[0] = [7,8];
        assert!(correspondence(&old,&new).is_err());
        new.sample_edges[0] = new.sample_edges[1];
        assert!(correspondence(&old,&new).is_err());
    }
    #[test]
    fn normal_regeneration_changes_with_shape_and_rejects_degeneracy() {
        let old = triangle(); let mut new = triangle(); new.vertices[2].z = 1.0;
        let mut h = host();
        let before = regenerated_normals(&old,&mut h).unwrap();
        let after = regenerated_normals(&new,&mut h).unwrap();
        let half_sqrt = 0.5f64.sqrt();
        for (a,b) in before.iter().zip(after) {
            assert_eq!([a.x,a.y,a.z], [0.,0.,1.]);
            assert!(b.x.abs() < 1e-12 && (b.y + half_sqrt).abs() < 1e-12 && (b.z-half_sqrt).abs() < 1e-12);
        }
        new.vertices[2] = new.vertices[1];
        assert!(regenerated_normals(&new,&mut h).is_err());
    }
    #[test]
    fn atlas_seams_follow_bijection_and_authored_normals_refuse_movement() {
        let old = triangle(); let mut next = triangle();
        next.vertices.swap(0,2); next.sample_edges.swap(0,2); next.triangles = vec![[2,1,0]];
        let attrs = Attributes { uv0:true, normals:true,
            atlas:Some(uv_atlas::AtlasDraft {version:2,seams:vec![[0,1]],..Default::default()}) };
        let source = source(); let mut h = host();
        let transfer = prepare(&attrs,&source,&source,&old,&next,&mut h).unwrap().unwrap();
        assert_eq!(transfer.source_vertices,vec![2,1,0]);
        assert_eq!(transfer.atlas.unwrap().seams,vec![[1,2]]);
        next.vertices[0].z = 1.;
        assert!(prepare(&attrs,&source,&source,&old,&next,&mut h).err().unwrap().contains("authored normals"));
        let mut changed_grid = source.clone(); changed_grid.origin_m[0] += 0.1;
        assert!(prepare(&attrs,&source,&changed_grid,&old,&next,&mut h).err().unwrap().contains("grid changed"));
    }
    #[test]
    fn procedural_normals_and_planar_uv_do_not_require_old_topology() {
        let old = triangle(); let mut next = triangle();
        next.vertices.push(Point3::new(1.,1.,1.)); next.sample_edges.push([3,4]);
        next.triangles.push([1,3,2]);
        let source = source().with_regenerated_normals();
        let attrs = Attributes {uv0:false,normals:true,atlas:None};
        assert!(prepare(&attrs,&source,&source,&old,&next,&mut host()).unwrap().is_none());
        let attrs = Attributes {uv0:true,normals:true,atlas:None};
        assert!(prepare(&attrs,&source,&source,&old,&next,&mut host()).is_err());
    }
    #[test]
    fn retained_uv_rows_are_evaluated_exactly_in_new_order() {
        use vibe::{session::SourceSession,Budget,LocalHost};
        let transfer = Transfer {source_vertices:vec![2,0,1],atlas:None};
        let mut out = "poet_parts = poet_parts.map(|p| { {geometry:{vertices:[]".to_owned();
        emit_stream(&mut out,"uv0",&transfer,4096).unwrap();
        out.push_str("}} });");
        let mut session = SourceSession::default(); let mut runtime = LocalHost::default();
        session.submit("let mut poet_parts=[{geometry:{uv0:[[0.0,0],[1,0.5],[0.75,1.0]]}}];",&mut runtime,Budget::default()).unwrap();
        session.submit(&out,&mut runtime,Budget::default()).unwrap();
        let expected = Value::List(vec![Value::List(vec![Value::F64(0.75),Value::F64(1.)]),
            Value::List(vec![Value::F64(0.),Value::I64(0)]),Value::List(vec![Value::I64(1),Value::F64(0.5)])]);
        let Some(Value::List(ps)) = session.value("poet_parts") else {panic!()};
        let Value::Record(p) = &ps[0] else {panic!()};
        let Value::Record(g) = &p["geometry"] else {panic!()};
        assert_eq!(g["uv0"],expected);
    }
}
