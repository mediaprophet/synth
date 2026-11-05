//! Clean-room parallel stroke and contour path offsetting (PA-04).
//!
//! Provides offset contour generation for polylines with configurable join styles
//! (Miter, Round, Bevel) and end cap styles (Butt, Round, Square).

/// 2D point representation for path offsetting.
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

/// Line join style at polyline vertices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JoinStyle {
    Miter { limit: f32 },
    Round { segments: u16 },
    Bevel,
}

/// Line end cap style for open polylines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapStyle {
    Butt,
    Round,
    Square,
}

/// Errors occurring during path offset calculations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathOffsetError {
    EmptyPath,
    InsufficientVertices { count: usize },
    NonFiniteDistance,
    NonFiniteCoordinate,
    InvalidMiterLimit { limit_bits: u32 },
    ZeroLengthSegment,
}

impl std::fmt::Display for PathOffsetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPath => write!(f, "path is empty"),
            Self::InsufficientVertices { count } => write!(f, "insufficient vertices: {count}"),
            Self::NonFiniteDistance => write!(f, "offset distance is non-finite"),
            Self::NonFiniteCoordinate => write!(f, "non-finite point coordinate in path"),
            Self::InvalidMiterLimit { limit_bits } => write!(f, "miter limit must be >= 1.0 (bits: {limit_bits:#x})"),
            Self::ZeroLengthSegment => write!(f, "zero-length segment detected in polyline"),
        }
    }
}
impl std::error::Error for PathOffsetError {}

#[inline]
fn segment_dir_and_normal(p1: Point2D, p2: Point2D) -> Result<((f64, f64), (f64, f64)), PathOffsetError> {
    let (dx, dy) = (p2.x - p1.x, p2.y - p1.y);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-9 {
        return Err(PathOffsetError::ZeroLengthSegment);
    }
    let dir = (dx / len, dy / len);
    let normal = (dir.1, -dir.0);
    Ok((dir, normal))
}

fn append_arc(
    center: Point2D,
    radius: f64,
    start_angle: f64,
    sweep_angle: f64,
    segments: usize,
    out: &mut Vec<Point2D>,
) {
    let segs = segments.max(2);
    for s in 0..=segs {
        let t = s as f64 / segs as f64;
        let angle = start_angle + sweep_angle * t;
        out.push(Point2D::new(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        ));
    }
}

fn add_corner_join(
    pt: Point2D,
    d1: (f64, f64),
    n1: (f64, f64),
    d2: (f64, f64),
    n2: (f64, f64),
    dist: f64,
    join: JoinStyle,
    out: &mut Vec<Point2D>,
) {
    let cross = d1.0 * d2.1 - d1.1 * d2.0;
    let n_avg = ((n1.0 + n2.0) * 0.5, (n1.1 + n2.1) * 0.5);
    let avg_len = (n_avg.0 * n_avg.0 + n_avg.1 * n_avg.1).sqrt();

    let p_in = Point2D::new(pt.x + dist * n1.0, pt.y + dist * n1.1);
    let p_out = Point2D::new(pt.x + dist * n2.0, pt.y + dist * n2.1);

    if avg_len < 1e-7 {
        out.push(p_in);
        out.push(p_out);
        return;
    }

    let n_norm = (n_avg.0 / avg_len, n_avg.1 / avg_len);
    let cos_half = n1.0 * n_norm.0 + n1.1 * n_norm.1;
    let miter_len = dist / cos_half.max(1e-6);
    let is_outer = dist * cross < 0.0;

    if is_outer {
        match join {
            JoinStyle::Miter { limit } => {
                let ratio = (miter_len / dist).abs();
                if ratio <= limit as f64 {
                    out.push(Point2D::new(pt.x + miter_len * n_norm.0, pt.y + miter_len * n_norm.1));
                } else {
                    out.push(p_in);
                    out.push(p_out);
                }
            }
            JoinStyle::Bevel => {
                out.push(p_in);
                out.push(p_out);
            }
            JoinStyle::Round { segments } => {
                let a1 = (dist * n1.1).atan2(dist * n1.0);
                let a2 = (dist * n2.1).atan2(dist * n2.0);
                let mut sweep = a2 - a1;
                while sweep > std::f64::consts::PI { sweep -= std::f64::consts::TAU; }
                while sweep < -std::f64::consts::PI { sweep += std::f64::consts::TAU; }
                append_arc(pt, dist.abs(), a1, sweep, segments as usize, out);
            }
        }
    } else {
        out.push(Point2D::new(pt.x + miter_len * n_norm.0, pt.y + miter_len * n_norm.1));
    }
}

/// Computes an offset contour for an open or closed polyline path.
pub fn offset_polyline_contour(
    points: &[Point2D],
    distance: f64,
    join: JoinStyle,
    cap: CapStyle,
    is_closed: bool,
) -> Result<Vec<Point2D>, PathOffsetError> {
    if !distance.is_finite() {
        return Err(PathOffsetError::NonFiniteDistance);
    }
    if points.is_empty() {
        return Err(PathOffsetError::EmptyPath);
    }
    let min_verts = if is_closed { 3 } else { 2 };
    if points.len() < min_verts {
        return Err(PathOffsetError::InsufficientVertices { count: points.len() });
    }
    if let JoinStyle::Miter { limit } = join {
        if limit < 1.0 || !limit.is_finite() {
            return Err(PathOffsetError::InvalidMiterLimit { limit_bits: limit.to_bits() });
        }
    }
    for p in points {
        if !p.is_finite() {
            return Err(PathOffsetError::NonFiniteCoordinate);
        }
    }

    let n = points.len();
    let mut dirs = Vec::with_capacity(n);
    let mut normals = Vec::with_capacity(n);

    let seg_count = if is_closed { n } else { n - 1 };
    for i in 0..seg_count {
        let p1 = points[i];
        let p2 = points[(i + 1) % n];
        let (d, norm) = segment_dir_and_normal(p1, p2)?;
        dirs.push(d);
        normals.push(norm);
    }

    let mut out = Vec::new();

    if is_closed {
        for i in 0..n {
            let prev_idx = (i + n - 1) % n;
            add_corner_join(
                points[i],
                dirs[prev_idx],
                normals[prev_idx],
                dirs[i],
                normals[i],
                distance,
                join,
                &mut out,
            );
        }
    } else {
        // Forward (left side at +distance)
        let first_norm = normals[0];
        out.push(Point2D::new(points[0].x + distance * first_norm.0, points[0].y + distance * first_norm.1));

        for i in 1..(n - 1) {
            add_corner_join(
                points[i],
                dirs[i - 1],
                normals[i - 1],
                dirs[i],
                normals[i],
                distance,
                join,
                &mut out,
            );
        }

        let last_norm = normals[n - 2];
        let last_dir = dirs[n - 2];
        let p_end = points[n - 1];
        let p_end_left = Point2D::new(p_end.x + distance * last_norm.0, p_end.y + distance * last_norm.1);
        let p_end_right = Point2D::new(p_end.x - distance * last_norm.0, p_end.y - distance * last_norm.1);
        out.push(p_end_left);

        // End Cap
        match cap {
            CapStyle::Butt => {
                out.push(p_end_right);
            }
            CapStyle::Square => {
                out.push(Point2D::new(p_end_left.x + distance * last_dir.0, p_end_left.y + distance * last_dir.1));
                out.push(Point2D::new(p_end_right.x + distance * last_dir.0, p_end_right.y + distance * last_dir.1));
                out.push(p_end_right);
            }
            CapStyle::Round => {
                let start_a = (distance * last_norm.1).atan2(distance * last_norm.0);
                append_arc(p_end, distance.abs(), start_a, -std::f64::consts::PI, 8, &mut out);
            }
        }

        // Return (right side at -distance)
        for i in (1..(n - 1)).rev() {
            add_corner_join(
                points[i],
                (-dirs[i].0, -dirs[i].1),
                (-normals[i].0, -normals[i].1),
                (-dirs[i - 1].0, -dirs[i - 1].1),
                (-normals[i - 1].0, -normals[i - 1].1),
                distance,
                join,
                &mut out,
            );
        }

        let p_start = points[0];
        let p_start_right = Point2D::new(p_start.x - distance * first_norm.0, p_start.y - distance * first_norm.1);
        let p_start_left = Point2D::new(p_start.x + distance * first_norm.0, p_start.y + distance * first_norm.1);
        out.push(p_start_right);

        // Start Cap
        match cap {
            CapStyle::Butt => {
                out.push(p_start_left);
            }
            CapStyle::Square => {
                let first_dir = dirs[0];
                out.push(Point2D::new(p_start_right.x - distance * first_dir.0, p_start_right.y - distance * first_dir.1));
                out.push(Point2D::new(p_start_left.x - distance * first_dir.0, p_start_left.y - distance * first_dir.1));
                out.push(p_start_left);
            }
            CapStyle::Round => {
                let start_a = (-distance * first_norm.1).atan2(-distance * first_norm.0);
                append_arc(p_start, distance.abs(), start_a, -std::f64::consts::PI, 8, &mut out);
            }
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_closed_polygon_miter() {
        let rect = [
            Point2D::new(0.0, 0.0),
            Point2D::new(4.0, 0.0),
            Point2D::new(4.0, 4.0),
            Point2D::new(0.0, 4.0),
        ];
        let offset = offset_polyline_contour(&rect, 1.0, JoinStyle::Miter { limit: 2.0 }, CapStyle::Butt, true)
            .expect("valid offset");
        assert_eq!(offset.len(), 4);
        assert!((offset[0].x - (-1.0)).abs() < 1e-5);
        assert!((offset[0].y - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn test_offset_open_polyline_caps() {
        let line = [Point2D::new(0.0, 0.0), Point2D::new(10.0, 0.0)];

        // Butt cap
        let butt = offset_polyline_contour(&line, 1.0, JoinStyle::Bevel, CapStyle::Butt, false).expect("butt");
        assert!(butt.len() >= 4);

        // Square cap
        let square = offset_polyline_contour(&line, 1.0, JoinStyle::Bevel, CapStyle::Square, false).expect("square");
        assert!(square.len() >= 6);

        // Round cap
        let round = offset_polyline_contour(&line, 1.0, JoinStyle::Round { segments: 4 }, CapStyle::Round, false)
            .expect("round");
        assert!(round.len() > butt.len());
    }

    #[test]
    fn test_negative_cases() {
        // Non-finite distance
        let err1 = offset_polyline_contour(&[Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)], f64::NAN, JoinStyle::Bevel, CapStyle::Butt, false);
        assert!(matches!(err1, Err(PathOffsetError::NonFiniteDistance)));

        // Empty path
        let err2 = offset_polyline_contour(&[], 1.0, JoinStyle::Bevel, CapStyle::Butt, false);
        assert!(matches!(err2, Err(PathOffsetError::EmptyPath)));

        // Insufficient vertices
        let err3 = offset_polyline_contour(&[Point2D::new(0.0, 0.0)], 1.0, JoinStyle::Bevel, CapStyle::Butt, false);
        assert!(matches!(err3, Err(PathOffsetError::InsufficientVertices { count: 1 })));

        // Miter limit < 1.0
        let err4 = offset_polyline_contour(
            &[Point2D::new(0.0, 0.0), Point2D::new(1.0, 0.0)],
            1.0,
            JoinStyle::Miter { limit: 0.5 },
            CapStyle::Butt,
            false,
        );
        assert!(matches!(err4, Err(PathOffsetError::InvalidMiterLimit { .. })));

        // Zero length segment
        let err5 = offset_polyline_contour(
            &[Point2D::new(0.0, 0.0), Point2D::new(0.0, 0.0)],
            1.0,
            JoinStyle::Bevel,
            CapStyle::Butt,
            false,
        );
        assert!(matches!(err5, Err(PathOffsetError::ZeroLengthSegment)));
    }
}
