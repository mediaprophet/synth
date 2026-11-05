//! Clean-room 2D vector polygon and cubic Bezier boolean operations (PA-04).
//!
//! Provides polygon boolean operations (Union, Intersection, Difference, Xor),
//! signed area / winding order computation, adaptive cubic Bezier flattening,
//! and robust validation against self-intersections and degeneracies.

/// 2D point representation for vector operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

/// Supported polygon boolean operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    Union,
    Intersection,
    Difference,
    Xor,
}

/// Errors occurring during vector boolean operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VectorBooleanError {
    InvalidVertexCount { count: usize },
    NonFiniteCoordinate,
    ZeroAreaPolygon { area_bits: u64 },
    SelfIntersectingDegenerate,
    EmptyResult,
    InvalidTolerance,
}

impl std::fmt::Display for VectorBooleanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidVertexCount { count } => write!(f, "polygon vertices < 3: {count}"),
            Self::NonFiniteCoordinate => write!(f, "non-finite coordinate detected"),
            Self::ZeroAreaPolygon { area_bits } => write!(f, "zero area polygon ({area_bits:#x})"),
            Self::SelfIntersectingDegenerate => write!(f, "self-intersecting degenerate rejected"),
            Self::EmptyResult => write!(f, "boolean operation produced empty result"),
            Self::InvalidTolerance => write!(f, "flattening tolerance must be positive finite"),
        }
    }
}
impl std::error::Error for VectorBooleanError {}

/// Computes signed area using Shoelace formula (positive = CCW, negative = CW).
pub fn signed_polygon_area(poly: &[Point2D]) -> Result<f64, VectorBooleanError> {
    if poly.len() < 3 {
        return Err(VectorBooleanError::InvalidVertexCount { count: poly.len() });
    }
    for pt in poly {
        if !pt.is_finite() {
            return Err(VectorBooleanError::NonFiniteCoordinate);
        }
    }
    let n = poly.len();
    let mut sum = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        sum += poly[i].x * poly[j].y - poly[j].x * poly[i].y;
    }
    let area = sum * 0.5;
    if area.abs() < 1e-7 {
        return Err(VectorBooleanError::ZeroAreaPolygon { area_bits: area.to_bits() });
    }
    Ok(area)
}

fn segments_intersect(p1: Point2D, p2: Point2D, p3: Point2D, p4: Point2D) -> Option<(f64, f64, Point2D)> {
    let (d1x, d1y) = (p2.x - p1.x, p2.y - p1.y);
    let (d2x, d2y) = (p4.x - p3.x, p4.y - p3.y);
    let denom = d1x * d2y - d1y * d2x;
    if denom.abs() < 1e-12 {
        return None;
    }
    let (dx, dy) = (p3.x - p1.x, p3.y - p1.y);
    let t = (dx * d2y - dy * d2x) / denom;
    let u = (dx * d1y - dy * d1x) / denom;
    if t > 1e-9 && t < 1.0 - 1e-9 && u > 1e-9 && u < 1.0 - 1e-9 {
        Some((t, u, Point2D::new(p1.x + t * d1x, p1.y + t * d1y)))
    } else {
        None
    }
}

/// Validates polygon integrity against self-intersections.
pub fn validate_polygon_integrity(poly: &[Point2D]) -> Result<(), VectorBooleanError> {
    if poly.len() < 3 {
        return Err(VectorBooleanError::InvalidVertexCount { count: poly.len() });
    }
    for p in poly {
        if !p.is_finite() {
            return Err(VectorBooleanError::NonFiniteCoordinate);
        }
    }
    let n = poly.len();
    for i in 0..n {
        let (p1, p2) = (poly[i], poly[(i + 1) % n]);
        for j in (i + 1)..n {
            if j == (i + 1) % n || i == (j + 1) % n { continue; }
            if segments_intersect(p1, p2, poly[j], poly[(j + 1) % n]).is_some() {
                return Err(VectorBooleanError::SelfIntersectingDegenerate);
            }
        }
    }
    let _ = signed_polygon_area(poly)?;
    Ok(())
}

pub fn point_in_polygon(pt: Point2D, poly: &[Point2D]) -> bool {
    let (mut inside, n) = (false, poly.len());
    let mut j = n - 1;
    for i in 0..n {
        let (pi, pj) = (poly[i], poly[j]);
        if ((pi.y > pt.y) != (pj.y > pt.y))
            && (pt.x < (pj.x - pi.x) * (pt.y - pi.y) / (pj.y - pi.y + 1e-14) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn flatten_bezier_step(
    p0: Point2D, p1: Point2D, p2: Point2D, p3: Point2D,
    tol_sq: f64, depth: u32, out: &mut Vec<Point2D>,
) {
    if depth >= 16 {
        out.push(p3);
        return;
    }
    let (bx, by) = (p3.x - p0.x, p3.y - p0.y);
    let b_len_sq = bx * bx + by * by;
    let (d1x, d1y, d2x, d2y) = (p1.x - p0.x, p1.y - p0.y, p2.x - p0.x, p2.y - p0.y);
    let (d1_sq, d2_sq) = if b_len_sq < 1e-12 {
        (d1x * d1x + d1y * d1y, d2x * d2x + d2y * d2y)
    } else {
        let (c1, c2) = (d1x * by - d1y * bx, d2x * by - d2y * bx);
        ((c1 * c1) / b_len_sq, (c2 * c2) / b_len_sq)
    };
    if d1_sq <= tol_sq && d2_sq <= tol_sq {
        out.push(p3);
    } else {
        let m = |a: Point2D, b: Point2D| Point2D::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
        let (q0, q1, q2) = (m(p0, p1), m(p1, p2), m(p2, p3));
        let (r0, r1) = (m(q0, q1), m(q1, q2));
        let s0 = m(r0, r1);
        flatten_bezier_step(p0, q0, r0, s0, tol_sq, depth + 1, out);
        flatten_bezier_step(s0, r1, q2, p3, tol_sq, depth + 1, out);
    }
}

pub fn flatten_cubic_bezier(
    p0: Point2D, p1: Point2D, p2: Point2D, p3: Point2D, tolerance: f64,
) -> Result<Vec<Point2D>, VectorBooleanError> {
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(VectorBooleanError::InvalidTolerance);
    }
    if !p0.is_finite() || !p1.is_finite() || !p2.is_finite() || !p3.is_finite() {
        return Err(VectorBooleanError::NonFiniteCoordinate);
    }
    let mut pts = Vec::with_capacity(16);
    pts.push(p0);
    flatten_bezier_step(p0, p1, p2, p3, tolerance * tolerance, 0, &mut pts);
    Ok(pts)
}

#[derive(Clone, Debug)]
struct PolyNode {
    pt: Point2D,
    is_intersection: bool,
    is_entry: bool,
    visited: bool,
    neighbor: usize,
}

struct RawIsect {
    s_edge: usize,
    s_t: f64,
    c_edge: usize,
    c_t: f64,
    pt: Point2D,
}

pub fn evaluate_polygon_boolean(
    subject: &[Point2D], clip: &[Point2D], op: BooleanOp,
) -> Result<Vec<Vec<Point2D>>, VectorBooleanError> {
    validate_polygon_integrity(subject)?;
    validate_polygon_integrity(clip)?;
    let mut s_poly = subject.to_vec();
    if signed_polygon_area(&s_poly)? < 0.0 { s_poly.reverse(); }
    let mut c_poly = clip.to_vec();
    if signed_polygon_area(&c_poly)? < 0.0 { c_poly.reverse(); }

    if op == BooleanOp::Xor {
        let mut res = evaluate_polygon_boolean(&s_poly, &c_poly, BooleanOp::Difference)?;
        res.extend(evaluate_polygon_boolean(&c_poly, &s_poly, BooleanOp::Difference)?);
        return Ok(res);
    }

    let (sn, cn) = (s_poly.len(), c_poly.len());
    let mut raw_ixs = Vec::new();
    for i in 0..sn {
        let (s1, s2) = (s_poly[i], s_poly[(i + 1) % sn]);
        for j in 0..cn {
            let (c1, c2) = (c_poly[j], c_poly[(j + 1) % cn]);
            if let Some((st, ct, pt)) = segments_intersect(s1, s2, c1, c2) {
                raw_ixs.push(RawIsect { s_edge: i, s_t: st, c_edge: j, c_t: ct, pt });
            }
        }
    }

    if raw_ixs.is_empty() {
        let (s_in_c, c_in_s) = (point_in_polygon(s_poly[0], &c_poly), point_in_polygon(c_poly[0], &s_poly));
        return match op {
            BooleanOp::Intersection => {
                if s_in_c { Ok(vec![s_poly]) } else if c_in_s { Ok(vec![c_poly]) } else { Ok(vec![]) }
            }
            BooleanOp::Union => {
                if s_in_c { Ok(vec![c_poly]) } else if c_in_s { Ok(vec![s_poly]) } else { Ok(vec![s_poly, c_poly]) }
            }
            BooleanOp::Difference => {
                if s_in_c { Ok(vec![]) } else if c_in_s { Ok(vec![s_poly]) } else { Ok(vec![s_poly]) }
            }
            BooleanOp::Xor => unreachable!(),
        };
    }

    let (mut s_nodes, mut c_nodes) = (Vec::new(), Vec::new());
    let (mut s_map, mut c_map) = (Vec::new(), Vec::new());

    for i in 0..sn {
        s_nodes.push(PolyNode { pt: s_poly[i], is_intersection: false, is_entry: false, visited: false, neighbor: 0 });
        let mut on_e: Vec<_> = raw_ixs.iter().enumerate().filter(|(_, x)| x.s_edge == i).collect();
        on_e.sort_by(|(_, a), (_, b)| a.s_t.partial_cmp(&b.s_t).unwrap_or(std::cmp::Ordering::Equal));
        for (idx, x) in on_e {
            s_map.push((idx, s_nodes.len()));
            s_nodes.push(PolyNode { pt: x.pt, is_intersection: true, is_entry: false, visited: false, neighbor: 0 });
        }
    }
    for j in 0..cn {
        c_nodes.push(PolyNode { pt: c_poly[j], is_intersection: false, is_entry: false, visited: false, neighbor: 0 });
        let mut on_e: Vec<_> = raw_ixs.iter().enumerate().filter(|(_, x)| x.c_edge == j).collect();
        on_e.sort_by(|(_, a), (_, b)| a.c_t.partial_cmp(&b.c_t).unwrap_or(std::cmp::Ordering::Equal));
        for (idx, x) in on_e {
            c_map.push((idx, c_nodes.len()));
            c_nodes.push(PolyNode { pt: x.pt, is_intersection: true, is_entry: false, visited: false, neighbor: 0 });
        }
    }
    for &(id_s, s_idx) in &s_map {
        if let Some(&(_, c_idx)) = c_map.iter().find(|(id_c, _)| *id_c == id_s) {
            s_nodes[s_idx].neighbor = c_idx;
            c_nodes[c_idx].neighbor = s_idx;
        }
    }

    let mut s_in = point_in_polygon(s_poly[0], &c_poly);
    for n in s_nodes.iter_mut() {
        if n.is_intersection { n.is_entry = !s_in; s_in = !s_in; }
    }
    let mut c_in = point_in_polygon(c_poly[0], &s_poly);
    for n in c_nodes.iter_mut() {
        if n.is_intersection { n.is_entry = !c_in; c_in = !c_in; }
    }

    let mut results = Vec::new();
    let (total_s, total_c) = (s_nodes.len(), c_nodes.len());
    for start_idx in 0..total_s {
        let should = if !s_nodes[start_idx].is_intersection || s_nodes[start_idx].visited {
            false
        } else {
            match op {
                BooleanOp::Intersection => s_nodes[start_idx].is_entry,
                BooleanOp::Union | BooleanOp::Difference => !s_nodes[start_idx].is_entry,
                BooleanOp::Xor => false,
            }
        };
        if !should { continue; }

        let mut polygon = Vec::new();
        let mut curr = start_idx;
        let mut on_s = true;
        let mut just_switched = true;

        for _ in 0..(total_s + total_c) * 2 {
            if on_s {
                if s_nodes[curr].is_intersection && !just_switched {
                    s_nodes[curr].visited = true;
                    let nb = s_nodes[curr].neighbor;
                    c_nodes[nb].visited = true;
                    curr = nb;
                    on_s = false;
                    just_switched = true;
                    continue;
                }
                polygon.push(s_nodes[curr].pt);
                s_nodes[curr].visited = true;
                if s_nodes[curr].is_intersection {
                    let nb = s_nodes[curr].neighbor;
                    c_nodes[nb].visited = true;
                }
                just_switched = false;
                curr = (curr + 1) % total_s;
            } else {
                if c_nodes[curr].is_intersection && !just_switched {
                    c_nodes[curr].visited = true;
                    let nb = c_nodes[curr].neighbor;
                    s_nodes[nb].visited = true;
                    curr = nb;
                    on_s = true;
                    just_switched = true;
                    if curr == start_idx { break; }
                    continue;
                }
                polygon.push(c_nodes[curr].pt);
                c_nodes[curr].visited = true;
                if c_nodes[curr].is_intersection {
                    let nb = c_nodes[curr].neighbor;
                    s_nodes[nb].visited = true;
                }
                just_switched = false;
                curr = if op == BooleanOp::Difference { (curr + total_c - 1) % total_c } else { (curr + 1) % total_c };
            }
            if on_s && curr == start_idx && !just_switched { break; }
        }
        if polygon.len() >= 3 && signed_polygon_area(&polygon).is_ok() {
            results.push(polygon);
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signed_polygon_area_ccw_and_cw() {
        let ccw = [Point2D::new(0.0, 0.0), Point2D::new(2.0, 0.0), Point2D::new(2.0, 2.0), Point2D::new(0.0, 2.0)];
        let area = signed_polygon_area(&ccw).expect("valid ccw area");
        assert!((area - 4.0).abs() < 1e-6);

        let mut cw = ccw;
        cw.reverse();
        let area_cw = signed_polygon_area(&cw).expect("valid cw area");
        assert!((area_cw - (-4.0)).abs() < 1e-6);
    }

    #[test]
    fn test_negative_degenerate_cases() {
        let err1 = signed_polygon_area(&[Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)]);
        assert!(matches!(err1, Err(VectorBooleanError::InvalidVertexCount { count: 2 })));

        let err2 = signed_polygon_area(&[Point2D::new(f64::NAN, 0.0), Point2D::new(1.0, 0.0), Point2D::new(0.0, 1.0)]);
        assert!(matches!(err2, Err(VectorBooleanError::NonFiniteCoordinate)));

        let collinear = [Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0)];
        assert!(matches!(signed_polygon_area(&collinear), Err(VectorBooleanError::ZeroAreaPolygon { .. })));

        let bowtie = [Point2D::new(0.0, 0.0), Point2D::new(2.0, 2.0), Point2D::new(0.0, 2.0), Point2D::new(2.0, 0.0)];
        assert!(matches!(validate_polygon_integrity(&bowtie), Err(VectorBooleanError::SelfIntersectingDegenerate)));
    }

    #[test]
    fn test_flatten_cubic_bezier() {
        let (p0, p1, p2, p3) = (Point2D::new(0.0, 0.0), Point2D::new(0.0, 1.0), Point2D::new(1.0, 1.0), Point2D::new(1.0, 0.0));
        let pts = flatten_cubic_bezier(p0, p1, p2, p3, 0.01).expect("flat curve");
        assert!(pts.len() >= 4);
        assert_eq!(pts[0], p0);
        assert_eq!(*pts.last().unwrap(), p3);

        let neg_tol = flatten_cubic_bezier(p0, p1, p2, p3, -1.0);
        assert!(matches!(neg_tol, Err(VectorBooleanError::InvalidTolerance)));
    }

    #[test]
    fn test_polygon_boolean_disjoint_and_overlap() {
        let s = [Point2D::new(0.0, 0.0), Point2D::new(2.0, 0.0), Point2D::new(2.0, 2.0), Point2D::new(0.0, 2.0)];
        let c = [Point2D::new(1.0, 1.0), Point2D::new(3.0, 1.0), Point2D::new(3.0, 3.0), Point2D::new(1.0, 3.0)];

        let isect = evaluate_polygon_boolean(&s, &c, BooleanOp::Intersection).expect("isect");
        assert_eq!(isect.len(), 1);
        let area_isect = signed_polygon_area(&isect[0]).expect("isect area");
        assert!((area_isect.abs() - 1.0).abs() < 1e-4);

        let diff = evaluate_polygon_boolean(&s, &c, BooleanOp::Difference).expect("diff");
        assert_eq!(diff.len(), 1);
        let area_diff = signed_polygon_area(&diff[0]).expect("diff area");
        assert!((area_diff.abs() - 3.0).abs() < 1e-4);
    }
}
