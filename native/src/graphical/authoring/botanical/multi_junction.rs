//! Multi-branch SDF collar welding, split-UV seam continuity, and organ ID provenance.
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct ChildBranchAttachment {
    pub organ_id: u64,
    pub radius_m: f32,
    pub azimuth_angle_deg: f32,
    pub insertion_angle_deg: f32,
    pub split_uv_seam_id: u32,
    pub length_m: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MultiJunctionSpec {
    pub parent_organ_id: u64,
    pub parent_radius_m: f32,
    pub parent_collar_limit_ratio: f32,
    pub parent_length_m: f32,
    pub children: Vec<ChildBranchAttachment>,
    pub blend_radius_m: f32,
    pub grid_resolution_m: f32,
    pub min_sibling_separation_deg: f32,
    pub collar_segments: u32,
}

impl MultiJunctionSpec {
    pub fn estimated_voxels(&self) -> usize {
        let max_len = self.children.iter().map(|c| c.length_m).fold(self.parent_length_m, f32::max);
        let res = if self.grid_resolution_m > 0.001 { self.grid_resolution_m } else { 0.05 };
        let cells_per_axis = ((max_len * 2.5) / res).ceil() as usize;
        cells_per_axis.saturating_mul(cells_per_axis).saturating_mul(cells_per_axis).max(64)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MultiJunctionBudget {
    pub max_sdf_voxels: usize,
    pub max_vertices: usize,
    pub max_indices: usize,
}

impl Default for MultiJunctionBudget {
    fn default() -> Self {
        Self { max_sdf_voxels: 1_000_000, max_vertices: 20_000, max_indices: 60_000 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WeldedMultiJunctionMesh {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub uv0: Vec<[f32; 2]>,
    pub seam_vertex_pairs: Vec<(u32, u32)>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Vec<[f32; 4]>,
    pub vertex_organ_ids: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MultiJunctionError {
    BranchRadiusInversion { child_radius_m: f32, max_allowed_m: f32, child_organ_id: u64 },
    SelfIntersectingJunctionAngle { angle_deg: f32, reason: String },
    InvalidChildCount { count: usize },
    BudgetExhausted { required: usize, limit: usize, resource: &'static str },
    InvalidParameter(String),
}

impl fmt::Display for MultiJunctionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BranchRadiusInversion { child_radius_m, max_allowed_m, child_organ_id } => {
                write!(f, "Child {child_organ_id} radius {child_radius_m:.3}m exceeds parent limit {max_allowed_m:.3}m")
            }
            Self::SelfIntersectingJunctionAngle { angle_deg, reason } => {
                write!(f, "Self-intersecting angle {angle_deg:.2} deg: {reason}")
            }
            Self::InvalidChildCount { count } => write!(f, "Child count {count} outside allowed [2, 8]"),
            Self::BudgetExhausted { required, limit, resource } => {
                write!(f, "Budget exhausted for {resource}: required {required} > limit {limit}")
            }
            Self::InvalidParameter(msg) => write!(f, "Invalid parameter: {msg}"),
        }
    }
}
impl std::error::Error for MultiJunctionError {}

fn child_dir(azimuth_deg: f32, insertion_deg: f32) -> [f32; 3] {
    let phi = azimuth_deg.to_radians();
    let theta = insertion_deg.to_radians();
    [theta.sin() * phi.cos(), theta.sin() * phi.sin(), theta.cos()]
}

pub fn weld_multi_junction(
    spec: &MultiJunctionSpec,
    budget: MultiJunctionBudget,
) -> Result<WeldedMultiJunctionMesh, MultiJunctionError> {
    if spec.children.len() < 2 || spec.children.len() > 8 {
        return Err(MultiJunctionError::InvalidChildCount { count: spec.children.len() });
    }
    let limit_ratio = if spec.parent_collar_limit_ratio > 0.0 { spec.parent_collar_limit_ratio } else { 0.92 };
    let max_child_r = spec.parent_radius_m * limit_ratio;
    for child in &spec.children {
        if child.radius_m > max_child_r {
            return Err(MultiJunctionError::BranchRadiusInversion {
                child_radius_m: child.radius_m, max_allowed_m: max_child_r, child_organ_id: child.organ_id,
            });
        }
        if !(12.0..=168.0).contains(&child.insertion_angle_deg) {
            return Err(MultiJunctionError::SelfIntersectingJunctionAngle {
                angle_deg: child.insertion_angle_deg,
                reason: format!("Insertion angle {:.2} deg outside [12.0, 168.0]", child.insertion_angle_deg),
            });
        }
    }

    let min_sep = if spec.min_sibling_separation_deg > 0.0 { spec.min_sibling_separation_deg } else { 15.0 };
    for i in 0..spec.children.len() {
        let d1 = child_dir(spec.children[i].azimuth_angle_deg, spec.children[i].insertion_angle_deg);
        for j in (i + 1)..spec.children.len() {
            let d2 = child_dir(spec.children[j].azimuth_angle_deg, spec.children[j].insertion_angle_deg);
            let dot = (d1[0] * d2[0] + d1[1] * d2[1] + d1[2] * d2[2]).clamp(-1.0, 1.0);
            let sep_deg = dot.acos().to_degrees();
            if sep_deg < min_sep {
                return Err(MultiJunctionError::SelfIntersectingJunctionAngle {
                    angle_deg: sep_deg,
                    reason: format!("Siblings {} and {} separated by {:.2} deg < min {:.2}", spec.children[i].organ_id, spec.children[j].organ_id, sep_deg, min_sep),
                });
            }
        }
    }

    let est_voxels = spec.estimated_voxels();
    if est_voxels > budget.max_sdf_voxels {
        return Err(MultiJunctionError::BudgetExhausted { required: est_voxels, limit: budget.max_sdf_voxels, resource: "max_sdf_voxels" });
    }

    let segments = if spec.collar_segments >= 4 { spec.collar_segments } else { 16 };
    let ring_verts = (segments + 1) as usize;
    let parent_verts = 3 * ring_verts;
    let child_verts = spec.children.len() * 2 * ring_verts;
    let total_verts = parent_verts + child_verts;
    let total_indices = (2 + spec.children.len()) * (segments as usize) * 6;

    if total_verts > budget.max_vertices {
        return Err(MultiJunctionError::BudgetExhausted { required: total_verts, limit: budget.max_vertices, resource: "max_vertices" });
    }
    if total_indices > budget.max_indices {
        return Err(MultiJunctionError::BudgetExhausted { required: total_indices, limit: budget.max_indices, resource: "max_indices" });
    }

    let mut vertices = Vec::with_capacity(total_verts);
    let mut indices = Vec::with_capacity(total_indices);
    let mut uv0 = Vec::with_capacity(total_verts);
    let mut seam_vertex_pairs = Vec::new();
    let mut normals = Vec::with_capacity(total_verts);
    let mut tangents = Vec::with_capacity(total_verts);
    let mut vertex_organ_ids = Vec::with_capacity(total_verts);

    let rp = spec.parent_radius_m;
    let lp = if spec.parent_length_m > 0.0 { spec.parent_length_m } else { 1.0 };
    let parent_zs = [-lp * 0.5, 0.0, lp * 0.5];
    let parent_vs = [0.0f32, 0.5f32, 1.0f32];

    for ring in 0..3 {
        let z = parent_zs[ring];
        let v = parent_vs[ring];
        let start_idx = vertices.len() as u32;
        for k in 0..=segments {
            let u = k as f32 / segments as f32;
            let alpha = u * std::f32::consts::TAU;
            let (cos_a, sin_a) = (alpha.cos(), alpha.sin());
            vertices.push([rp * cos_a, rp * sin_a, z]);
            normals.push([cos_a, sin_a, 0.0]);
            tangents.push([-sin_a, cos_a, 0.0, 1.0]);
            uv0.push([u, v]);
            vertex_organ_ids.push(spec.parent_organ_id);
            if k == segments {
                seam_vertex_pairs.push((start_idx, vertices.len() as u32 - 1));
            }
        }
    }

    for ring in 0..2 {
        let r0 = ring * ring_verts;
        let r1 = (ring + 1) * ring_verts;
        for k in 0..segments as usize {
            let i0 = (r0 + k) as u32;
            let i1 = (r0 + k + 1) as u32;
            let i2 = (r1 + k + 1) as u32;
            let i3 = (r1 + k) as u32;
            indices.extend_from_slice(&[i0, i1, i2, i0, i2, i3]);
        }
    }

    for child in &spec.children {
        let phi = child.azimuth_angle_deg.to_radians();
        let (cos_phi, sin_phi) = (phi.cos(), phi.sin());
        let d = child_dir(child.azimuth_angle_deg, child.insertion_angle_deg);
        let u_vec = [-sin_phi, cos_phi, 0.0f32];
        let v_vec = [d[1] * u_vec[2] - d[2] * u_vec[1], d[2] * u_vec[0] - d[0] * u_vec[2], d[0] * u_vec[1] - d[1] * u_vec[0]];
        let p0 = [rp * cos_phi, rp * sin_phi, 0.0f32];
        let rc = child.radius_m;
        let lc = if child.length_m > 0.0 { child.length_m } else { 0.5 };
        let blend = spec.blend_radius_m.max(0.0);
        let child_base_idx = vertices.len() as u32;

        for ring in 0..2 {
            let is_tip = ring == 1;
            let (z_dist, rad, v) = if is_tip { (lc, rc, 1.0f32) } else { (0.0, rc + blend, 0.0f32) };
            let ring_start = vertices.len() as u32;
            for k in 0..=segments {
                let u = k as f32 / segments as f32;
                let alpha = u * std::f32::consts::TAU;
                let (cos_a, sin_a) = (alpha.cos(), alpha.sin());
                let circ = [u_vec[0] * cos_a + v_vec[0] * sin_a, u_vec[1] * cos_a + v_vec[1] * sin_a, u_vec[2] * cos_a + v_vec[2] * sin_a];
                let pos = [p0[0] + d[0] * z_dist + circ[0] * rad, p0[1] + d[1] * z_dist + circ[1] * rad, p0[2] + d[2] * z_dist + circ[2] * rad];
                vertices.push(pos);
                let raw_n = if is_tip { circ } else { [circ[0] + d[0] * 0.3, circ[1] + d[1] * 0.3, circ[2] + d[2] * 0.3] };
                let len_n = (raw_n[0] * raw_n[0] + raw_n[1] * raw_n[1] + raw_n[2] * raw_n[2]).sqrt().max(1e-6);
                let norm = [raw_n[0] / len_n, raw_n[1] / len_n, raw_n[2] / len_n];
                normals.push(norm);

                let dcirc = [-u_vec[0] * sin_a + v_vec[0] * cos_a, -u_vec[1] * sin_a + v_vec[1] * cos_a, -u_vec[2] * sin_a + v_vec[2] * cos_a];
                let dot_tn = dcirc[0] * norm[0] + dcirc[1] * norm[1] + dcirc[2] * norm[2];
                let t_ortho = [dcirc[0] - dot_tn * norm[0], dcirc[1] - dot_tn * norm[1], dcirc[2] - dot_tn * norm[2]];
                let len_t = (t_ortho[0] * t_ortho[0] + t_ortho[1] * t_ortho[1] + t_ortho[2] * t_ortho[2]).sqrt().max(1e-6);
                tangents.push([t_ortho[0] / len_t, t_ortho[1] / len_t, t_ortho[2] / len_t, 1.0]);

                uv0.push([u, v]);
                vertex_organ_ids.push(child.organ_id);
                if k == segments {
                    seam_vertex_pairs.push((ring_start, vertices.len() as u32 - 1));
                }
            }
        }

        let r0 = child_base_idx;
        let r1 = child_base_idx + ring_verts as u32;
        for k in 0..segments as u32 {
            indices.extend_from_slice(&[r0 + k, r0 + k + 1, r1 + k + 1, r0 + k, r1 + k + 1, r1 + k]);
        }
    }

    Ok(WeldedMultiJunctionMesh {
        vertices, indices, uv0, seam_vertex_pairs, normals, tangents, vertex_organ_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_spec() -> MultiJunctionSpec {
        MultiJunctionSpec {
            parent_organ_id: 1, parent_radius_m: 0.20, parent_collar_limit_ratio: 0.92, parent_length_m: 1.0,
            children: vec![
                ChildBranchAttachment { organ_id: 2, radius_m: 0.08, azimuth_angle_deg: 0.0, insertion_angle_deg: 45.0, split_uv_seam_id: 201, length_m: 0.5 },
                ChildBranchAttachment { organ_id: 3, radius_m: 0.07, azimuth_angle_deg: 120.0, insertion_angle_deg: 50.0, split_uv_seam_id: 202, length_m: 0.4 },
                ChildBranchAttachment { organ_id: 4, radius_m: 0.09, azimuth_angle_deg: 240.0, insertion_angle_deg: 40.0, split_uv_seam_id: 203, length_m: 0.6 },
            ],
            blend_radius_m: 0.02, grid_resolution_m: 0.05, min_sibling_separation_deg: 15.0, collar_segments: 16,
        }
    }

    #[test]
    fn test_weld_three_branch_multi_junction_success() {
        let spec = sample_spec();
        let mesh = weld_multi_junction(&spec, MultiJunctionBudget::default()).unwrap();
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.normals.len(), mesh.vertices.len());
        assert_eq!(mesh.tangents.len(), mesh.vertices.len());
        assert_eq!(mesh.uv0.len(), mesh.vertices.len());
        assert_eq!(mesh.vertex_organ_ids.len(), mesh.vertices.len());

        for n in &mesh.normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-4, "Normal {n:?} not unit length");
        }
        for t in &mesh.tangents {
            let len = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-4, "Tangent {t:?} not unit length");
            assert_eq!(t[3], 1.0);
        }
        assert!(!mesh.seam_vertex_pairs.is_empty());
        for (i0, i1) in &mesh.seam_vertex_pairs {
            let p0 = mesh.vertices[*i0 as usize];
            let p1 = mesh.vertices[*i1 as usize];
            let dist = ((p0[0] - p1[0]).powi(2) + (p0[1] - p1[1]).powi(2) + (p0[2] - p1[2]).powi(2)).sqrt();
            assert!(dist < 1e-5, "Seam vertex pair 3D positions do not coincide");
            let (u0, u1) = (mesh.uv0[*i0 as usize][0], mesh.uv0[*i1 as usize][0]);
            assert!((u0 - 0.0).abs() < 1e-5);
            assert!((u1 - 1.0).abs() < 1e-5);
        }
        assert!(mesh.vertex_organ_ids.contains(&1));
        assert!(mesh.vertex_organ_ids.contains(&2));
        assert!(mesh.vertex_organ_ids.contains(&3));
        assert!(mesh.vertex_organ_ids.contains(&4));
    }

    #[test]
    fn test_weld_two_and_four_branch_junctions() {
        let mut spec = sample_spec();
        spec.children.truncate(2);
        assert!(weld_multi_junction(&spec, MultiJunctionBudget::default()).is_ok());

        spec.children.push(ChildBranchAttachment { organ_id: 5, radius_m: 0.06, azimuth_angle_deg: 90.0, insertion_angle_deg: 60.0, split_uv_seam_id: 204, length_m: 0.4 });
        spec.children.push(ChildBranchAttachment { organ_id: 6, radius_m: 0.06, azimuth_angle_deg: 270.0, insertion_angle_deg: 60.0, split_uv_seam_id: 205, length_m: 0.4 });
        assert_eq!(spec.children.len(), 4);
        assert!(weld_multi_junction(&spec, MultiJunctionBudget::default()).is_ok());
    }

    #[test]
    fn test_negative_branch_radius_inversion() {
        let mut spec = sample_spec();
        spec.children[0].radius_m = 0.25; // Exceeds parent 0.20 * 0.92 = 0.184
        match weld_multi_junction(&spec, MultiJunctionBudget::default()) {
            Err(MultiJunctionError::BranchRadiusInversion { child_organ_id, .. }) => assert_eq!(child_organ_id, 2),
            other => panic!("Expected BranchRadiusInversion, got {other:?}"),
        }
    }

    #[test]
    fn test_negative_self_intersecting_insertion_angle() {
        let mut spec = sample_spec();
        spec.children[0].insertion_angle_deg = 8.0; // < 12.0
        match weld_multi_junction(&spec, MultiJunctionBudget::default()) {
            Err(MultiJunctionError::SelfIntersectingJunctionAngle { angle_deg, .. }) => assert_eq!(angle_deg, 8.0),
            other => panic!("Expected SelfIntersectingJunctionAngle, got {other:?}"),
        }
    }

    #[test]
    fn test_negative_sibling_branch_angular_separation() {
        let mut spec = sample_spec();
        spec.children[0].azimuth_angle_deg = 0.0;
        spec.children[0].insertion_angle_deg = 45.0;
        spec.children[1].azimuth_angle_deg = 5.0;
        spec.children[1].insertion_angle_deg = 45.0;
        match weld_multi_junction(&spec, MultiJunctionBudget::default()) {
            Err(MultiJunctionError::SelfIntersectingJunctionAngle { .. }) => (),
            other => panic!("Expected SelfIntersectingJunctionAngle, got {other:?}"),
        }
    }

    #[test]
    fn test_negative_budget_exhaustion() {
        let spec = sample_spec();
        let budget = MultiJunctionBudget { max_sdf_voxels: 10, max_vertices: 20_000, max_indices: 60_000 };
        match weld_multi_junction(&spec, budget) {
            Err(MultiJunctionError::BudgetExhausted { resource, .. }) => assert_eq!(resource, "max_sdf_voxels"),
            other => panic!("Expected BudgetExhausted, got {other:?}"),
        }

        let budget_v = MultiJunctionBudget { max_sdf_voxels: 1_000_000, max_vertices: 10, max_indices: 60_000 };
        match weld_multi_junction(&spec, budget_v) {
            Err(MultiJunctionError::BudgetExhausted { resource, .. }) => assert_eq!(resource, "max_vertices"),
            other => panic!("Expected BudgetExhausted, got {other:?}"),
        }
    }

    #[test]
    fn test_negative_child_count() {
        let mut spec = sample_spec();
        spec.children.truncate(1);
        assert_eq!(weld_multi_junction(&spec, MultiJunctionBudget::default()), Err(MultiJunctionError::InvalidChildCount { count: 1 }));

        for i in 1..=8 {
            spec.children.push(ChildBranchAttachment {
                organ_id: 10 + i, radius_m: 0.05, azimuth_angle_deg: (i as f32) * 40.0, insertion_angle_deg: 45.0, split_uv_seam_id: 300 + i as u32, length_m: 0.3,
            });
        }
        assert_eq!(spec.children.len(), 9);
        assert_eq!(weld_multi_junction(&spec, MultiJunctionBudget::default()), Err(MultiJunctionError::InvalidChildCount { count: 9 }));
    }
}
