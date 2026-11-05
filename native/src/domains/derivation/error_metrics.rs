//! Geometric, normal, UV, and silhouette deviation error metrics (AR-05, ENG-11).
//!
//! Evaluates deviations between a high-grade Master mesh/rig embodiment and a
//! derived target profile mesh (Film, Open-World-Game, Cartoon).
//! Uses only standard library (`std`) types for universal portability.

use std::collections::BTreeSet;
use std::fmt;

/// A 3D surface sample point on a mesh or rig boundary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshSurfaceSample {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub organ_id: u64,
}

impl MeshSurfaceSample {
    pub const fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2], organ_id: u64) -> Self {
        Self { position, normal, uv, organ_id }
    }
}

/// Aggregated deviation metrics computed between master and derived samples.
#[derive(Debug, Clone, PartialEq)]
pub struct DerivationErrorMetrics {
    pub max_hausdorff_distance_mm: f32,
    pub mean_surface_distance_mm: f32,
    pub max_normal_deviation_deg: f32,
    pub max_uv_seam_drift: f32,
    pub silhouette_iou: f32,
    pub preserved_organ_count: usize,
    pub missing_critical_organs: usize,
}

/// Acceptance thresholds for derivation error metrics across delivery profiles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProfileErrorThresholds {
    pub max_hausdorff_mm: f32,
    pub max_normal_deg: f32,
    pub max_uv_drift: f32,
    pub min_silhouette_iou: f32,
}

impl ProfileErrorThresholds {
    /// Reference thresholds for high-detail photorealistic Film profile.
    pub const fn film() -> Self {
        Self { max_hausdorff_mm: 1.0, max_normal_deg: 8.0, max_uv_drift: 0.02, min_silhouette_iou: 0.95 }
    }
    /// Reference thresholds for interactive Open-World-Game profile.
    pub const fn open_world_game() -> Self {
        Self { max_hausdorff_mm: 6.0, max_normal_deg: 25.0, max_uv_drift: 0.05, min_silhouette_iou: 0.85 }
    }
    /// Reference thresholds for stylized Cartoon profile.
    pub const fn cartoon() -> Self {
        Self { max_hausdorff_mm: 15.0, max_normal_deg: 45.0, max_uv_drift: 0.10, min_silhouette_iou: 0.70 }
    }
}

/// Typed errors returned when derivation metrics violate thresholds or prerequisites.
#[derive(Debug, Clone, PartialEq)]
pub enum DerivationMetricError {
    EmptyMasterSamples,
    EmptyDerivedSamples,
    MissingCriticalOrgans { missing_ids: Vec<u64> },
    HausdorffExceeded { value_mm: f32, threshold_mm: f32 },
    NormalDeviationExceeded { value_deg: f32, threshold_deg: f32 },
    UvDriftExceeded { value: f32, threshold: f32 },
    SilhouetteIouDeficient { value: f32, min_iou: f32 },
}

impl fmt::Display for DerivationMetricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMasterSamples => write!(f, "master mesh surface sample set is empty"),
            Self::EmptyDerivedSamples => write!(f, "derived mesh surface sample set is empty"),
            Self::MissingCriticalOrgans { missing_ids } => write!(f, "missing critical organ IDs: {missing_ids:?}"),
            Self::HausdorffExceeded { value_mm, threshold_mm } => write!(f, "Hausdorff {value_mm:.2} mm > max {threshold_mm:.2} mm"),
            Self::NormalDeviationExceeded { value_deg, threshold_deg } => write!(f, "normal dev {value_deg:.2} deg > max {threshold_deg:.2} deg"),
            Self::UvDriftExceeded { value, threshold } => write!(f, "UV drift {value:.4} > max {threshold:.4}"),
            Self::SilhouetteIouDeficient { value, min_iou } => write!(f, "silhouette IoU {value:.3} < min {min_iou:.3}"),
        }
    }
}

impl std::error::Error for DerivationMetricError {}

#[inline]
fn dist_3d(a: [f32; 3], b: [f32; 3]) -> f32 {
    let (dx, dy, dz) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[inline]
fn dist_2d(a: [f32; 2], b: [f32; 2]) -> f32 {
    let (du, dv) = (a[0] - b[0], a[1] - b[1]);
    (du * du + dv * dv).sqrt()
}

#[inline]
fn normal_angle_deg(n1: [f32; 3], n2: [f32; 3]) -> f32 {
    let dot = n1[0] * n2[0] + n1[1] * n2[1] + n1[2] * n2[2];
    let len1 = (n1[0] * n1[0] + n1[1] * n1[1] + n1[2] * n1[2]).sqrt();
    let len2 = (n2[0] * n2[0] + n2[1] * n2[1] + n2[2] * n2[2]).sqrt();
    if len1 < 1e-7 || len2 < 1e-7 { return 0.0; }
    (dot / (len1 * len2)).clamp(-1.0, 1.0).acos().to_degrees()
}

/// 32x32 binary occupancy grid for orthogonal silhouette IoU estimation.
#[derive(Clone, Copy)]
struct SilhouetteGrid {
    rows: [u32; 32],
}

impl SilhouetteGrid {
    fn new() -> Self {
        Self { rows: [0u32; 32] }
    }
    fn mark(&mut self, u: f32, v: f32, min_u: f32, span_u: f32, min_v: f32, span_v: f32) {
        let nu = ((u - min_u) / span_u).clamp(0.0, 0.9999);
        let nv = ((v - min_v) / span_v).clamp(0.0, 0.9999);
        let (col, row) = ((nu * 32.0) as usize, (nv * 32.0) as usize);
        if row < 32 && col < 32 { self.rows[row] |= 1u32 << col; }
    }
    fn iou(&self, other: &Self) -> f32 {
        let (mut inter, mut union) = (0u32, 0u32);
        for i in 0..32 {
            inter += (self.rows[i] & other.rows[i]).count_ones();
            union += (self.rows[i] | other.rows[i]).count_ones();
        }
        if union == 0 { 1.0 } else { inter as f32 / union as f32 }
    }
}

/// Evaluate derivation metrics and enforce profile error thresholds.
pub fn evaluate_derivation_error_metrics(
    master_samples: &[MeshSurfaceSample],
    derived_samples: &[MeshSurfaceSample],
    critical_organ_ids: &[u64],
    thresholds: &ProfileErrorThresholds,
) -> Result<DerivationErrorMetrics, DerivationMetricError> {
    if master_samples.is_empty() { return Err(DerivationMetricError::EmptyMasterSamples); }
    if derived_samples.is_empty() { return Err(DerivationMetricError::EmptyDerivedSamples); }

    // 1. Organ preservation & critical organ verification
    let master_organs: BTreeSet<u64> = master_samples.iter().map(|s| s.organ_id).collect();
    let derived_organs: BTreeSet<u64> = derived_samples.iter().map(|s| s.organ_id).collect();
    let preserved_organ_count = master_organs.intersection(&derived_organs).count();

    let mut missing_ids = Vec::new();
    for &cid in critical_organ_ids {
        if !derived_organs.contains(&cid) { missing_ids.push(cid); }
    }
    if !missing_ids.is_empty() {
        return Err(DerivationMetricError::MissingCriticalOrgans { missing_ids });
    }

    // 2. Hausdorff distance, mean surface distance, normal deviation, and UV drift
    let (mut sum_dist, mut max_h_m_to_d, mut max_normal_deviation_deg, mut max_uv_drift) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    for m in master_samples {
        let (mut min_d, mut min_d_organ) = (f32::MAX, f32::MAX);
        let mut nearest_d: Option<&MeshSurfaceSample> = None;
        let mut nearest_same_organ: Option<&MeshSurfaceSample> = None;
        for d in derived_samples {
            let dist = dist_3d(m.position, d.position);
            if dist < min_d { min_d = dist; nearest_d = Some(d); }
            if d.organ_id == m.organ_id && dist < min_d_organ { min_d_organ = dist; nearest_same_organ = Some(d); }
        }
        sum_dist += min_d;
        if min_d > max_h_m_to_d { max_h_m_to_d = min_d; }
        if let Some(nd) = nearest_d {
            let deg = normal_angle_deg(m.normal, nd.normal);
            if deg > max_normal_deviation_deg { max_normal_deviation_deg = deg; }
        }
        if let Some(ref_d) = nearest_same_organ.or(nearest_d) {
            let drift = dist_2d(m.uv, ref_d.uv);
            if drift > max_uv_drift { max_uv_drift = drift; }
        }
    }

    let mut max_h_d_to_m = 0.0f32;
    for d in derived_samples {
        let mut min_d = f32::MAX;
        for m in master_samples {
            let dist = dist_3d(d.position, m.position);
            if dist < min_d { min_d = dist; }
        }
        sum_dist += min_d;
        if min_d > max_h_d_to_m { max_h_d_to_m = min_d; }
    }

    let max_hausdorff_distance_mm = max_h_m_to_d.max(max_h_d_to_m);
    let total_sample_count = master_samples.len() + derived_samples.len();
    let mean_surface_distance_mm = sum_dist / (total_sample_count as f32);

    // 3. Multi-axis Silhouette IoU (XY, XZ, YZ planes)
    let (mut min_b, mut max_b) = ([f32::MAX; 3], [f32::MIN; 3]);
    for s in master_samples.iter().chain(derived_samples.iter()) {
        for k in 0..3 {
            min_b[k] = min_b[k].min(s.position[k]);
            max_b[k] = max_b[k].max(s.position[k]);
        }
    }
    let mut span = [0.0f32; 3];
    for k in 0..3 { span[k] = (max_b[k] - min_b[k]).max(1e-4); }

    let (mut m_xy, mut d_xy) = (SilhouetteGrid::new(), SilhouetteGrid::new());
    let (mut m_xz, mut d_xz) = (SilhouetteGrid::new(), SilhouetteGrid::new());
    let (mut m_yz, mut d_yz) = (SilhouetteGrid::new(), SilhouetteGrid::new());
    for s in master_samples {
        m_xy.mark(s.position[0], s.position[1], min_b[0], span[0], min_b[1], span[1]);
        m_xz.mark(s.position[0], s.position[2], min_b[0], span[0], min_b[2], span[2]);
        m_yz.mark(s.position[1], s.position[2], min_b[1], span[1], min_b[2], span[2]);
    }
    for s in derived_samples {
        d_xy.mark(s.position[0], s.position[1], min_b[0], span[0], min_b[1], span[1]);
        d_xz.mark(s.position[0], s.position[2], min_b[0], span[0], min_b[2], span[2]);
        d_yz.mark(s.position[1], s.position[2], min_b[1], span[1], min_b[2], span[2]);
    }
    let silhouette_iou = (m_xy.iou(&d_xy) + m_xz.iou(&d_xz) + m_yz.iou(&d_yz)) / 3.0;

    // 4. Threshold validation
    if max_hausdorff_distance_mm > thresholds.max_hausdorff_mm {
        return Err(DerivationMetricError::HausdorffExceeded { value_mm: max_hausdorff_distance_mm, threshold_mm: thresholds.max_hausdorff_mm });
    }
    if max_normal_deviation_deg > thresholds.max_normal_deg {
        return Err(DerivationMetricError::NormalDeviationExceeded { value_deg: max_normal_deviation_deg, threshold_deg: thresholds.max_normal_deg });
    }
    if max_uv_drift > thresholds.max_uv_drift {
        return Err(DerivationMetricError::UvDriftExceeded { value: max_uv_drift, threshold: thresholds.max_uv_drift });
    }
    if silhouette_iou < thresholds.min_silhouette_iou {
        return Err(DerivationMetricError::SilhouetteIouDeficient { value: silhouette_iou, min_iou: thresholds.min_silhouette_iou });
    }

    Ok(DerivationErrorMetrics {
        max_hausdorff_distance_mm,
        mean_surface_distance_mm,
        max_normal_deviation_deg,
        max_uv_seam_drift: max_uv_drift,
        silhouette_iou,
        preserved_organ_count,
        missing_critical_organs: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cube(offset: [f32; 3], normal_tilt_deg: f32, uv_offset: f32) -> Vec<MeshSurfaceSample> {
        let rad = normal_tilt_deg.to_radians();
        let (nx, nz) = (rad.sin(), rad.cos());
        vec![
            MeshSurfaceSample::new([offset[0] + 0.0, offset[1] + 0.0, offset[2] + 0.0], [nx, 0.0, nz], [0.0 + uv_offset, 0.0], 100),
            MeshSurfaceSample::new([offset[0] + 10.0, offset[1] + 0.0, offset[2] + 0.0], [nx, 0.0, nz], [0.5 + uv_offset, 0.0], 100),
            MeshSurfaceSample::new([offset[0] + 0.0, offset[1] + 10.0, offset[2] + 0.0], [0.0, 1.0, 0.0], [0.0 + uv_offset, 0.5], 200),
            MeshSurfaceSample::new([offset[0] + 10.0, offset[1] + 10.0, offset[2] + 0.0], [0.0, 1.0, 0.0], [0.5 + uv_offset, 0.5], 200),
        ]
    }

    #[test]
    fn test_identical_samples_pass_film_thresholds() {
        let master = sample_cube([0.0, 0.0, 0.0], 0.0, 0.0);
        let derived = sample_cube([0.0, 0.0, 0.0], 0.0, 0.0);
        let critical = [100, 200];
        let res = evaluate_derivation_error_metrics(&master, &derived, &critical, &ProfileErrorThresholds::film()).unwrap();
        assert_eq!(res.max_hausdorff_distance_mm, 0.0);
        assert_eq!(res.max_normal_deviation_deg, 0.0);
        assert_eq!(res.max_uv_seam_drift, 0.0);
        assert_eq!(res.silhouette_iou, 1.0);
        assert_eq!(res.preserved_organ_count, 2);
    }

    #[test]
    fn test_small_perturbation_passes_open_world_game() {
        let master = sample_cube([0.0, 0.0, 0.0], 0.0, 0.0);
        let derived = sample_cube([0.2, 0.1, 0.0], 4.0, 0.01);
        let critical = [100, 200];
        let res = evaluate_derivation_error_metrics(&master, &derived, &critical, &ProfileErrorThresholds::open_world_game()).unwrap();
        assert!(res.max_hausdorff_distance_mm <= 6.0);
        assert!(res.silhouette_iou >= 0.85);
    }

    #[test]
    fn test_empty_samples_fail() {
        let s = sample_cube([0.0, 0.0, 0.0], 0.0, 0.0);
        assert_eq!(evaluate_derivation_error_metrics(&[], &s, &[], &ProfileErrorThresholds::film()), Err(DerivationMetricError::EmptyMasterSamples));
        assert_eq!(evaluate_derivation_error_metrics(&s, &[], &[], &ProfileErrorThresholds::film()), Err(DerivationMetricError::EmptyDerivedSamples));
    }

    #[test]
    fn test_missing_critical_organ_fails() {
        let master = sample_cube([0.0, 0.0, 0.0], 0.0, 0.0);
        let derived = vec![MeshSurfaceSample::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0], 100)];
        let res = evaluate_derivation_error_metrics(&master, &derived, &[100, 200], &ProfileErrorThresholds::open_world_game());
        assert_eq!(res, Err(DerivationMetricError::MissingCriticalOrgans { missing_ids: vec![200] }));
    }

    #[test]
    fn test_hausdorff_exceeded_fails() {
        let master = sample_cube([0.0, 0.0, 0.0], 0.0, 0.0);
        let derived = sample_cube([12.0, 0.0, 0.0], 0.0, 0.0);
        let res = evaluate_derivation_error_metrics(&master, &derived, &[], &ProfileErrorThresholds::open_world_game());
        assert!(matches!(res, Err(DerivationMetricError::HausdorffExceeded { .. })));
    }

    #[test]
    fn test_normal_deviation_exceeded_fails() {
        let master = sample_cube([0.0, 0.0, 0.0], 0.0, 0.0);
        let derived = sample_cube([0.0, 0.0, 0.0], 35.0, 0.0);
        let res = evaluate_derivation_error_metrics(&master, &derived, &[], &ProfileErrorThresholds::open_world_game());
        assert!(matches!(res, Err(DerivationMetricError::NormalDeviationExceeded { .. })));
    }

    #[test]
    fn test_uv_drift_exceeded_fails() {
        let master = sample_cube([0.0, 0.0, 0.0], 0.0, 0.0);
        let derived = sample_cube([0.0, 0.0, 0.0], 0.0, 0.08);
        let res = evaluate_derivation_error_metrics(&master, &derived, &[], &ProfileErrorThresholds::open_world_game());
        assert!(matches!(res, Err(DerivationMetricError::UvDriftExceeded { .. })));
    }

    #[test]
    fn test_silhouette_iou_deficient_fails() {
        let master = sample_cube([0.0, 0.0, 0.0], 0.0, 0.0);
        let derived = vec![
            MeshSurfaceSample::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0], 100),
            MeshSurfaceSample::new([0.01, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0], 100),
            MeshSurfaceSample::new([0.0, 0.01, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0], 200),
            MeshSurfaceSample::new([0.01, 0.01, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0], 200),
        ];
        let thresholds = ProfileErrorThresholds { max_hausdorff_mm: 50.0, max_normal_deg: 90.0, max_uv_drift: 1.0, min_silhouette_iou: 0.90 };
        let res = evaluate_derivation_error_metrics(&master, &derived, &[], &thresholds);
        assert!(matches!(res, Err(DerivationMetricError::SilhouetteIouDeficient { .. })));
    }
}
