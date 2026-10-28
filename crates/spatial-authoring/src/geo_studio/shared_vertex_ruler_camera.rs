//! Shared-Vertex Topological Boundary Editing, Geodesic Ruler & Area Protractor,
//! and Damped Orbital / Flythrough Camera & Spatial Bookmark Controller.
//!
//! Adapted from RustGIS (v0.4.0, Apache-2.0) and EarthCraft (v0.2.0, MIT).
//! Portions (c) 2026 RustGIS Contributors, EarthCraft Contributors, and Poet Authors.
//! Licensed under Apache-2.0 or MIT at your option.

#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SharedVertexId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TopologyPolygonId(pub u32);

/// Errors arising during topological boundary editing or geodesic measurements.
#[derive(Debug, Clone, PartialEq)]
pub enum GeoTopologyEditError {
    PolygonNotFound(TopologyPolygonId), VertexNotFound(SharedVertexId),
    EdgeNotFound(SharedVertexId, SharedVertexId), PolygonAlreadyExists(TopologyPolygonId),
    InvalidRing(String), RingOrientationInverted(TopologyPolygonId),
    SelfIntersectionDetected(TopologyPolygonId), BudgetExceeded(String),
    DegeneratePolygon(TopologyPolygonId), EmptyInput,
}

impl std::fmt::Display for GeoTopologyEditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PolygonNotFound(id) => write!(f, "Polygon {} not found", id.0),
            Self::VertexNotFound(id) => write!(f, "Vertex {} not found", id.0),
            Self::EdgeNotFound(a, b) => write!(f, "Edge ({}, {}) not found", a.0, b.0),
            Self::PolygonAlreadyExists(id) => write!(f, "Polygon {} already exists", id.0),
            Self::InvalidRing(msg) => write!(f, "Invalid ring: {msg}"),
            Self::RingOrientationInverted(id) => write!(f, "Orientation inverted in polygon {}", id.0),
            Self::SelfIntersectionDetected(id) => write!(f, "Self-intersection in polygon {}", id.0),
            Self::BudgetExceeded(msg) => write!(f, "Budget exceeded: {msg}"),
            Self::DegeneratePolygon(id) => write!(f, "Degenerate polygon {}", id.0),
            Self::EmptyInput => write!(f, "Empty or insufficient input points"),
        }
    }
}
impl std::error::Error for GeoTopologyEditError {}

/// Shared-vertex topological cadastral mesh for simultaneous multi-parcel boundary editing.
#[derive(Debug, Clone)]
pub struct TopologicalCadastralMesh {
    pub vertices: Vec<[f64; 2]>,
    pub snap_tolerance: f64,
    pub polygons: Vec<(TopologyPolygonId, Vec<SharedVertexId>)>,
    pub byte_budget: usize,
}

impl TopologicalCadastralMesh {
    /// Create a new topological mesh with given vertex snap tolerance and byte budget.
    pub fn new(snap_tolerance: f64, byte_budget: usize) -> Self {
        Self { vertices: Vec::new(), snap_tolerance, polygons: Vec::new(), byte_budget }
    }

    /// Insert a polygon ring, snapping vertices within `snap_tolerance` to existing vertices.
    pub fn insert_polygon(
        &mut self,
        poly_id: TopologyPolygonId,
        ring_coords: &[[f64; 2]],
    ) -> Result<Vec<SharedVertexId>, GeoTopologyEditError> {
        if self.polygons.iter().any(|(id, _)| *id == poly_id) {
            return Err(GeoTopologyEditError::PolygonAlreadyExists(poly_id));
        }
        let coords = if ring_coords.len() > 1 && ring_coords.first() == ring_coords.last() {
            &ring_coords[..ring_coords.len() - 1]
        } else {
            ring_coords
        };
        if coords.len() < 3 {
            return Err(GeoTopologyEditError::InvalidRing("Need >= 3 vertices".into()));
        }
        let mut vids = Vec::with_capacity(coords.len());
        for pt in coords {
            let matched = self.vertices.iter().position(|v| {
                ((pt[0] - v[0]).powi(2) + (pt[1] - v[1]).powi(2)).sqrt() <= self.snap_tolerance
            });
            let vid = match matched {
                Some(idx) => SharedVertexId(idx as u32),
                None => {
                    if (self.vertices.len() * 16 + self.polygons.len() * 64 + 16) > self.byte_budget {
                        return Err(GeoTopologyEditError::BudgetExceeded("Vertex budget".into()));
                    }
                    let id = SharedVertexId(self.vertices.len() as u32);
                    self.vertices.push(*pt);
                    id
                }
            };
            vids.push(vid);
        }
        self.polygons.push((poly_id, vids.clone()));
        Ok(vids)
    }

    /// Move a shared vertex, updating all incident polygons simultaneously without gaps.
    pub fn move_shared_vertex(
        &mut self,
        vid: SharedVertexId,
        new_pos: [f64; 2],
    ) -> Result<usize, GeoTopologyEditError> {
        if (vid.0 as usize) >= self.vertices.len() {
            return Err(GeoTopologyEditError::VertexNotFound(vid));
        }
        let incident: Vec<usize> = self.polygons.iter().enumerate()
            .filter(|(_, (_, vids))| vids.contains(&vid)).map(|(i, _)| i).collect();
        if incident.is_empty() {
            return Err(GeoTopologyEditError::VertexNotFound(vid));
        }
        let orig_areas: Vec<f64> = incident.iter().map(|&i| polygon_signed_area(&self.coords_of_polygon(i))).collect();
        let old_pos = self.vertices[vid.0 as usize];
        self.vertices[vid.0 as usize] = new_pos;

        for (&idx, &orig_area) in incident.iter().zip(orig_areas.iter()) {
            let poly_id = self.polygons[idx].0;
            let pts = self.coords_of_polygon(idx);
            let new_area = polygon_signed_area(&pts);
            if new_area.abs() < 1e-9 || (orig_area.abs() > 1e-9 && new_area.signum() != orig_area.signum()) {
                self.vertices[vid.0 as usize] = old_pos;
                return Err(GeoTopologyEditError::RingOrientationInverted(poly_id));
            }
            if ring_has_self_intersection(&pts) {
                self.vertices[vid.0 as usize] = old_pos;
                return Err(GeoTopologyEditError::SelfIntersectionDetected(poly_id));
            }
        }
        Ok(incident.len())
    }

    /// Split a shared boundary edge `(v_start, v_end)` across all sharing polygons.
    pub fn split_shared_edge(
        &mut self,
        v_start: SharedVertexId,
        v_end: SharedVertexId,
        midpoint: [f64; 2],
    ) -> Result<SharedVertexId, GeoTopologyEditError> {
        if (v_start.0 as usize) >= self.vertices.len() { return Err(GeoTopologyEditError::VertexNotFound(v_start)); }
        if (v_end.0 as usize) >= self.vertices.len() { return Err(GeoTopologyEditError::VertexNotFound(v_end)); }
        let has_edge = self.polygons.iter().any(|(_, vids)| {
            let n = vids.len();
            (0..n).any(|i| {
                let next = (i + 1) % n;
                (vids[i] == v_start && vids[next] == v_end) || (vids[i] == v_end && vids[next] == v_start)
            })
        });
        if !has_edge { return Err(GeoTopologyEditError::EdgeNotFound(v_start, v_end)); }
        if (self.vertices.len() * 16 + self.polygons.len() * 64 + 16) > self.byte_budget {
            return Err(GeoTopologyEditError::BudgetExceeded("Vertex budget".into()));
        }
        let new_vid = SharedVertexId(self.vertices.len() as u32);
        self.vertices.push(midpoint);
        for (_, vids) in &mut self.polygons {
            let n = vids.len();
            for i in 0..n {
                let next = (i + 1) % n;
                if (vids[i] == v_start && vids[next] == v_end) || (vids[i] == v_end && vids[next] == v_start) {
                    vids.insert(i + 1, new_vid);
                    break;
                }
            }
        }
        Ok(new_vid)
    }

    /// Retrieve the current boundary coordinates for a polygon.
    pub fn polygon_coords(&self, poly_id: TopologyPolygonId) -> Option<Vec<[f64; 2]>> {
        let (_, vids) = self.polygons.iter().find(|(id, _)| *id == poly_id)?;
        Some(vids.iter().map(|v| self.vertices[v.0 as usize]).collect())
    }

    fn coords_of_polygon(&self, poly_idx: usize) -> Vec<[f64; 2]> {
        self.polygons[poly_idx].1.iter().map(|v| self.vertices[v.0 as usize]).collect()
    }
}

/// Compute signed planar area of a 2D ring using the shoelace formula.
fn polygon_signed_area(pts: &[[f64; 2]]) -> f64 {
    let n = pts.len();
    if n < 3 { return 0.0; }
    let mut sum = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        sum += pts[i][0] * pts[j][1] - pts[j][0] * pts[i][1];
    }
    0.5 * sum
}

/// Check if non-adjacent segments of a ring self-intersect.
fn ring_has_self_intersection(pts: &[[f64; 2]]) -> bool {
    let n = pts.len();
    if n < 4 { return false; }
    for i in 0..n {
        let (p1, p2) = (pts[i], pts[(i + 1) % n]);
        for j in (i + 1)..n {
            if j == (i + 1) % n || i == (j + 1) % n { continue; }
            if segments_intersect(p1, p2, pts[j], pts[(j + 1) % n]) { return true; }
        }
    }
    false
}

/// Test whether two 2D line segments strictly intersect in their interiors.
fn segments_intersect(p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], p4: [f64; 2]) -> bool {
    fn cross(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
        (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
    }
    let (d1, d2) = (cross(p3, p4, p1), cross(p3, p4, p2));
    let (d3, d4) = (cross(p1, p2, p3), cross(p1, p2, p4));
    ((d1 > 1e-9 && d2 < -1e-9) || (d1 < -1e-9 && d2 > 1e-9))
        && ((d3 > 1e-9 && d4 < -1e-9) || (d3 < -1e-9 && d4 > 1e-9))
}

/// Geodesic ruler distance, azimuth, enclosed area, and interior angle measurements.
#[derive(Debug, Clone, PartialEq)]
pub struct RulerMeasurement {
    pub segment_lengths_m: Vec<f64>, pub total_length_m: f64,
    pub forward_azimuths_deg: Vec<f64>, pub enclosed_area_m2: f64, pub interior_angles_deg: Vec<f64>,
}

/// Measure geodesic ruler lengths, bearings, spherical excess area, and interior angles.
pub fn measure_geodesic_ruler_and_protractor(
    lon_lat_pts: &[[f64; 2]],
) -> Result<RulerMeasurement, GeoTopologyEditError> {
    if lon_lat_pts.len() < 2 { return Err(GeoTopologyEditError::EmptyInput); }
    let mut segment_lengths_m = Vec::with_capacity(lon_lat_pts.len() - 1);
    let mut forward_azimuths_deg = Vec::with_capacity(lon_lat_pts.len() - 1);
    let mut total_length_m = 0.0;
    for w in lon_lat_pts.windows(2) {
        let (d, az) = vincenty_distance_and_azimuth(w[0][0], w[0][1], w[1][0], w[1][1]);
        segment_lengths_m.push(d); forward_azimuths_deg.push(az); total_length_m += d;
    }
    let ring_pts = if lon_lat_pts.len() >= 4 && lon_lat_pts[0] == lon_lat_pts[lon_lat_pts.len() - 1] {
        &lon_lat_pts[..lon_lat_pts.len() - 1]
    } else {
        lon_lat_pts
    };
    let n = ring_pts.len();
    let enclosed_area_m2 = if n >= 3 {
        let r = 6_371_008.8; // WGS84 authalic mean radius (m)
        let mut area_sum = 0.0;
        for i in 0..n {
            let j = (i + 1) % n;
            let (l1, f1) = (ring_pts[i][0].to_radians(), ring_pts[i][1].to_radians());
            let (l2, f2) = (ring_pts[j][0].to_radians(), ring_pts[j][1].to_radians());
            area_sum += (l2 - l1) * ((f1 + f2) * 0.5).sin();
        }
        (area_sum * r * r).abs()
    } else { 0.0 };

    let mut interior_angles_deg = Vec::with_capacity(n);
    if n >= 3 {
        for i in 0..n {
            let (prev, next) = (if i == 0 { n - 1 } else { i - 1 }, (i + 1) % n);
            let (_, az_prev) = vincenty_distance_and_azimuth(ring_pts[i][0], ring_pts[i][1], ring_pts[prev][0], ring_pts[prev][1]);
            let (_, az_next) = vincenty_distance_and_azimuth(ring_pts[i][0], ring_pts[i][1], ring_pts[next][0], ring_pts[next][1]);
            interior_angles_deg.push((az_next - az_prev).rem_euclid(360.0));
        }
        let sum_deg: f64 = interior_angles_deg.iter().sum();
        if sum_deg > (n as f64) * 180.0 {
            for ang in &mut interior_angles_deg { *ang = 360.0 - *ang; }
        }
    }
    Ok(RulerMeasurement { segment_lengths_m, total_length_m, forward_azimuths_deg, enclosed_area_m2, interior_angles_deg })
}

/// Vincenty inverse geodesic calculation for distance (m) and forward azimuth (deg).
fn vincenty_distance_and_azimuth(lon1_deg: f64, lat1_deg: f64, lon2_deg: f64, lat2_deg: f64) -> (f64, f64) {
    let (lon1, lat1) = (lon1_deg.to_radians(), lat1_deg.to_radians());
    let (lon2, lat2) = (lon2_deg.to_radians(), lat2_deg.to_radians());
    let (a, f, b) = (6_378_137.0, 1.0 / 298.257_223_563, 6_356_752.314_245);
    let l = lon2 - lon1;
    let (u1, u2) = (((1.0 - f) * lat1.tan()).atan(), ((1.0 - f) * lat2.tan()).atan());
    let (sin_u1, cos_u1, sin_u2, cos_u2) = (u1.sin(), u1.cos(), u2.sin(), u2.cos());
    let (mut lambda, mut iter) = (l, 100);
    let (mut sin_sigma, mut cos_sigma, mut sigma, mut cos2_alpha, mut cos_2sm) = (0.0, 0.0, 0.0, 0.0, 0.0);
    while iter > 0 {
        let (sin_l, cos_l) = (lambda.sin(), lambda.cos());
        sin_sigma = ((cos_u2 * sin_l).powi(2) + (cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_l).powi(2)).sqrt();
        if sin_sigma.abs() < 1e-12 { return (0.0, 0.0); }
        cos_sigma = sin_u1 * sin_u2 + cos_u1 * cos_u2 * cos_l;
        sigma = sin_sigma.atan2(cos_sigma);
        let sin_alpha = cos_u1 * cos_u2 * sin_l / sin_sigma;
        cos2_alpha = 1.0 - sin_alpha * sin_alpha;
        cos_2sm = if cos2_alpha.abs() < 1e-12 { 0.0 } else { cos_sigma - 2.0 * sin_u1 * sin_u2 / cos2_alpha };
        let c = f / 16.0 * cos2_alpha * (4.0 + f * (4.0 - 3.0 * cos2_alpha));
        let lambda_prev = lambda;
        lambda = l + (1.0 - c) * f * sin_alpha * (sigma + c * sin_sigma * (cos_2sm + c * cos_sigma * (-1.0 + 2.0 * cos_2sm * cos_2sm)));
        if (lambda - lambda_prev).abs() < 1e-12 { break; }
        iter -= 1;
    }
    let u_sq = cos2_alpha * (a * a - b * b) / (b * b);
    let a_val = 1.0 + u_sq / 16384.0 * (4096.0 + u_sq * (-768.0 + u_sq * (320.0 - 175.0 * u_sq)));
    let b_val = u_sq / 1024.0 * (256.0 + u_sq * (-128.0 + u_sq * (74.0 - 47.0 * u_sq)));
    let d_sig = b_val * sin_sigma * (cos_2sm + b_val / 4.0 * (cos_sigma * (-1.0 + 2.0 * cos_2sm * cos_2sm)
        - b_val / 6.0 * cos_2sm * (-3.0 + 4.0 * sin_sigma * sin_sigma) * (-3.0 + 4.0 * cos_2sm * cos_2sm)));
    let s = b * a_val * (sigma - d_sig);
    let az = (cos_u2 * lambda.sin()).atan2(cos_u1 * sin_u2 - sin_u1 * cos_u2 * lambda.cos()).to_degrees().rem_euclid(360.0);
    (s, az)
}

/// Spatial camera bookmark for instant recall or flythrough paths.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraBookmark {
    pub name: String, pub target_xyz: [f64; 3], pub yaw_rad: f64,
    pub pitch_rad: f64, pub distance_m: f64, pub fov_deg: f64,
}

/// Damped orbital / flythrough camera state with bookmark registry.
#[derive(Debug, Clone, PartialEq)]
pub struct OrbitalCameraState {
    pub target_xyz: [f64; 3], pub current_xyz: [f64; 3],
    pub yaw_rad: f64, pub pitch_rad: f64, pub distance_m: f64,
    pub vel_target: [f64; 3], pub vel_angles: [f64; 3],
    pub damping: f64, pub bookmarks: Vec<CameraBookmark>,
}

impl OrbitalCameraState {
    /// Initialize an orbital camera looking at `target_xyz`.
    pub fn new(target_xyz: [f64; 3], yaw_rad: f64, pitch_rad: f64, distance_m: f64, damping: f64) -> Self {
        let current_xyz = compute_orbit_pos(target_xyz, yaw_rad, pitch_rad, distance_m);
        Self {
            target_xyz,
            current_xyz,
            yaw_rad,
            pitch_rad,
            distance_m,
            vel_target: [0.0; 3],
            vel_angles: [0.0; 3],
            damping,
            bookmarks: Vec::new(),
        }
    }

    /// Advance damped camera state towards desired target and spherical angles.
    pub fn step_damped_camera(
        &mut self,
        desired_target: [f64; 3],
        desired_yaw: f64,
        desired_pitch: f64,
        desired_dist: f64,
        dt_sec: f64,
    ) {
        if dt_sec <= 0.0 { return; }
        let alpha = if self.damping <= 0.0 { 1.0 } else { 1.0 - (-self.damping * dt_sec).exp() };
        for i in 0..3 {
            let delta = (desired_target[i] - self.target_xyz[i]) * alpha;
            self.vel_target[i] = delta / dt_sec;
            self.target_xyz[i] += delta;
        }
        let yaw_delta = shortest_angle_diff(desired_yaw, self.yaw_rad) * alpha;
        let pitch_delta = (desired_pitch - self.pitch_rad) * alpha;
        let dist_delta = (desired_dist - self.distance_m) * alpha;
        self.vel_angles = [yaw_delta / dt_sec, pitch_delta / dt_sec, dist_delta / dt_sec];
        self.yaw_rad += yaw_delta;
        self.pitch_rad += pitch_delta;
        self.distance_m += dist_delta;
        self.current_xyz = compute_orbit_pos(self.target_xyz, self.yaw_rad, self.pitch_rad, self.distance_m);
    }
}

/// Compute 3D eye position from orbit target and spherical parameters.
fn compute_orbit_pos(target: [f64; 3], yaw: f64, pitch: f64, dist: f64) -> [f64; 3] {
    let (cos_p, sin_p, cos_y, sin_y) = (pitch.cos(), pitch.sin(), yaw.cos(), yaw.sin());
    [target[0] + dist * cos_p * sin_y, target[1] + dist * sin_p, target[2] + dist * cos_p * cos_y]
}

/// Compute shortest signed angular distance between target and current angles in radians.
fn shortest_angle_diff(target: f64, current: f64) -> f64 {
    let diff = (target - current).rem_euclid(std::f64::consts::TAU);
    if diff > std::f64::consts::PI { diff - std::f64::consts::TAU } else { diff }
}

/// Smooth cubic Hermite interpolation between two bookmarks for flythroughs.
pub fn interpolate_bookmark_flythrough(a: &CameraBookmark, b: &CameraBookmark, t: f64) -> CameraBookmark {
    let s = t.clamp(0.0, 1.0).powi(2) * (3.0 - 2.0 * t.clamp(0.0, 1.0));
    CameraBookmark {
        name: format!("{}_to_{}", a.name, b.name),
        target_xyz: [
            a.target_xyz[0] + (b.target_xyz[0] - a.target_xyz[0]) * s,
            a.target_xyz[1] + (b.target_xyz[1] - a.target_xyz[1]) * s,
            a.target_xyz[2] + (b.target_xyz[2] - a.target_xyz[2]) * s,
        ],
        yaw_rad: a.yaw_rad + shortest_angle_diff(b.yaw_rad, a.yaw_rad) * s,
        pitch_rad: a.pitch_rad + (b.pitch_rad - a.pitch_rad) * s,
        distance_m: a.distance_m + (b.distance_m - a.distance_m) * s,
        fov_deg: a.fov_deg + (b.fov_deg - a.fov_deg) * s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_shared_vertex_move_and_invariants() {
        let mut mesh = TopologicalCadastralMesh::new(0.01, 1024 * 1024);
        let pa = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let pb = [[10.0, 0.0], [20.0, 0.0], [20.0, 10.0], [10.0, 10.0]];
        let va = mesh.insert_polygon(TopologyPolygonId(1), &pa).unwrap();
        let vb = mesh.insert_polygon(TopologyPolygonId(2), &pb).unwrap();
        assert_eq!(va[1], vb[0]);
        assert_eq!(va[2], vb[3]);

        let a_orig = polygon_signed_area(&mesh.polygon_coords(TopologyPolygonId(1)).unwrap());
        let b_orig = polygon_signed_area(&mesh.polygon_coords(TopologyPolygonId(2)).unwrap());
        assert!(((a_orig + b_orig) - 200.0).abs() < 1e-6);

        assert_eq!(mesh.move_shared_vertex(va[2], [12.0, 10.0]).unwrap(), 2);
        let ca = mesh.polygon_coords(TopologyPolygonId(1)).unwrap();
        let cb = mesh.polygon_coords(TopologyPolygonId(2)).unwrap();
        assert_eq!(ca[2], [12.0, 10.0]);
        assert_eq!(cb[3], [12.0, 10.0]);

        let (a_new, b_new) = (polygon_signed_area(&ca), polygon_signed_area(&cb));
        assert!((a_new - 110.0).abs() < 1e-6 && (b_new - 90.0).abs() < 1e-6);
        assert!((a_new + b_new - (a_orig + b_orig)).abs() < 1e-6);

        let res_invert = mesh.move_shared_vertex(va[2], [35.0, 10.0]);
        assert!(matches!(res_invert, Err(GeoTopologyEditError::RingOrientationInverted(_))));
        assert!(mesh.move_shared_vertex(va[2], [5.0, -2.0]).is_err());
    }

    #[test]
    fn test_topological_split_shared_edge() {
        let mut mesh = TopologicalCadastralMesh::new(0.01, 1024 * 1024);
        let pa = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let pb = [[10.0, 0.0], [20.0, 0.0], [20.0, 10.0], [10.0, 10.0]];
        let va = mesh.insert_polygon(TopologyPolygonId(1), &pa).unwrap();
        let _ = mesh.insert_polygon(TopologyPolygonId(2), &pb).unwrap();
        let nvid = mesh.split_shared_edge(va[1], va[2], [10.0, 5.0]).unwrap();
        let (ca, cb) = (mesh.polygon_coords(TopologyPolygonId(1)).unwrap(), mesh.polygon_coords(TopologyPolygonId(2)).unwrap());
        assert_eq!(ca.len(), 5);
        assert_eq!(cb.len(), 5);
        assert!(ca.contains(&[10.0, 5.0]) && cb.contains(&[10.0, 5.0]));
        assert_eq!(mesh.vertices[nvid.0 as usize], [10.0, 5.0]);
    }

    #[test]
    fn test_geodesic_ruler_and_protractor() {
        let sq = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]];
        let r = measure_geodesic_ruler_and_protractor(&sq).unwrap();
        assert_eq!(r.segment_lengths_m.len(), 4);
        assert!((r.total_length_m - 444_000.0).abs() < 5_000.0);
        let az = [90.0, 0.0, 270.0, 180.0];
        for i in 0..4 {
            assert!((r.forward_azimuths_deg[i] - az[i]).abs() < 0.1);
        }
        assert!((r.enclosed_area_m2 - 1.236e10).abs() < 5e8);
        assert_eq!(r.interior_angles_deg.len(), 4);
        for &ang in &r.interior_angles_deg {
            assert!((ang - 90.0).abs() < 0.5);
        }
    }

    #[test]
    fn test_damped_camera_and_flythrough() {
        let mut cam = OrbitalCameraState::new([0.0, 0.0, 0.0], 0.0, 0.0, 100.0, 4.0);
        for _ in 0..60 {
            cam.step_damped_camera([50.0, 20.0, -10.0], 1.5, 0.3, 40.0, 0.05);
        }
        assert!((cam.target_xyz[0] - 50.0).abs() < 0.1 && (cam.target_xyz[1] - 20.0).abs() < 0.1 && (cam.target_xyz[2] + 10.0).abs() < 0.1);
        assert!((cam.yaw_rad - 1.5).abs() < 0.05 && (cam.pitch_rad - 0.3).abs() < 0.05 && (cam.distance_m - 40.0).abs() < 0.1);

        let bm_a = CameraBookmark { name: "A".into(), target_xyz: [0.0; 3], yaw_rad: 0.0, pitch_rad: 0.0, distance_m: 100.0, fov_deg: 60.0 };
        let bm_b = CameraBookmark { name: "B".into(), target_xyz: [100.0, 50.0, 25.0], yaw_rad: std::f64::consts::PI, pitch_rad: 0.5, distance_m: 50.0, fov_deg: 45.0 };
        let mid = interpolate_bookmark_flythrough(&bm_a, &bm_b, 0.5);
        assert_eq!(mid.name, "A_to_B");
        assert!((mid.target_xyz[0] - 50.0).abs() < 1e-4 && (mid.distance_m - 75.0).abs() < 1e-4 && (mid.fov_deg - 52.5).abs() < 1e-4);
    }
}
