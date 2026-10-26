//! Metric operator parameters. Inherited geometry supplies curve tessellation.
use crate::{AuthoringError, finite_positive};
use poet_geometry::{Arc, Circle, PI, Polyline, TAU, Vec2};
#[derive(Clone, Debug, PartialEq)]
pub enum Profile {
    RoundedRectangle {
        width: f64,
        height: f64,
        radius: f64,
        segments: usize,
    },
    Circle {
        radius: f64,
        segments: usize,
    },
    OpenChannel {
        width: f64,
        height: f64,
        wall: f64,
        corner_radius: f64,
        segments: usize,
    },
    Polyline {
        points: Vec<[f64; 2]>,
    },
}
#[derive(Clone, Copy, Debug)]
pub struct ProfileLimits {
    pub points: usize,
    pub scratch_bytes: usize,
}
impl Default for ProfileLimits {
    fn default() -> Self {
        Self {
            points: 4096,
            scratch_bytes: 4 * 1024 * 1024,
        }
    }
}
fn arc(
    out: &mut Vec<Vec2>,
    center: Vec2,
    radius: f64,
    start: f64,
    end: f64,
    segments: usize,
    reverse: bool,
) {
    let mut points = Vec::new();
    let tolerance =
        (radius * (1.0 - ((end - start) / (2.0 * segments as f64)).cos())).max(radius * 1e-6);
    Arc::new(center, radius, start, end).tessellate(tolerance, &mut points);
    if reverse {
        points.reverse();
    }
    out.extend(points);
}
impl Profile {
    pub fn points(&self, limits: ProfileLimits) -> Result<Vec<[f64; 2]>, AuthoringError> {
        if limits.points < 3 || limits.points > 65536 {
            return Err(AuthoringError::Budget(
                "profile point budget must be 3..65536",
            ));
        }
        let (segments, estimate) = match self {
            Self::RoundedRectangle { segments, .. } | Self::OpenChannel { segments, .. } => (
                *segments,
                segments.checked_add(3).and_then(|n| n.checked_mul(8)),
            ),
            Self::Circle { segments, .. } => (*segments, segments.checked_add(3)),
            Self::Polyline { points } => (1, Some(points.len())),
        };
        if segments == 0
            || segments > 1024
            || estimate.is_none_or(|n| {
                n > limits.points || n.checked_mul(64).is_none_or(|b| b > limits.scratch_bytes)
            })
        {
            return Err(AuthoringError::Budget(
                "profile tessellation exceeds caller budget",
            ));
        }
        let mut out = Vec::new();
        match self {
            Self::RoundedRectangle {
                width,
                height,
                radius,
                segments,
            } => {
                if !finite_positive(*width)
                    || !finite_positive(*height)
                    || !radius.is_finite()
                    || *radius < 0.0
                    || *radius > width.min(*height) * 0.5
                {
                    return Err(AuthoringError::Invalid(
                        "rounded rectangle dimensions/radius",
                    ));
                }
                let (x, y) = (*width * 0.5, *height * 0.5);
                if *radius == 0.0 {
                    out.extend([
                        Vec2::new(-x, -y),
                        Vec2::new(x, -y),
                        Vec2::new(x, y),
                        Vec2::new(-x, y),
                    ]);
                } else {
                    for (cx, cy, start) in [
                        (x - radius, y - radius, 0.0),
                        (-x + radius, y - radius, PI * 0.5),
                        (-x + radius, -y + radius, PI),
                        (x - radius, -y + radius, PI * 1.5),
                    ] {
                        arc(
                            &mut out,
                            Vec2::new(cx, cy),
                            *radius,
                            start,
                            start + PI * 0.5,
                            *segments,
                            false,
                        );
                    }
                }
            }
            Self::Circle { radius, segments } => {
                if !finite_positive(*radius) || *segments < 8 {
                    return Err(AuthoringError::Invalid("circle radius/segments"));
                }
                let tolerance = radius * (1.0 - (TAU / (2.0 * (*segments as f64))).cos());
                Circle::new(Vec2::new(0.0, 0.0), *radius).tessellate(tolerance, &mut out);
            }
            Self::OpenChannel {
                width,
                height,
                wall,
                corner_radius,
                segments,
            } => {
                if !finite_positive(*width)
                    || !finite_positive(*height)
                    || !finite_positive(*wall)
                    || *wall >= *width * 0.5
                    || *wall >= *height
                    || !corner_radius.is_finite()
                    || *corner_radius < 0.0
                    || *corner_radius > width.min(*height) * 0.5
                {
                    return Err(AuthoringError::Invalid(
                        "open channel dimensions/wall/corner radius",
                    ));
                }
                let (x, y, r) = (*width * 0.5, *height * 0.5, *corner_radius);
                let ri = (r - wall).max(0.0);
                out.push(Vec2::new(-x, y));
                if r > 0.0 {
                    arc(
                        &mut out,
                        Vec2::new(-x + r, -y + r),
                        r,
                        PI,
                        PI * 1.5,
                        *segments,
                        false,
                    );
                    arc(
                        &mut out,
                        Vec2::new(x - r, -y + r),
                        r,
                        PI * 1.5,
                        TAU,
                        *segments,
                        false,
                    );
                } else {
                    out.extend([Vec2::new(-x, -y), Vec2::new(x, -y)]);
                }
                out.extend([Vec2::new(x, y), Vec2::new(x - wall, y)]);
                if ri > 0.0 {
                    arc(
                        &mut out,
                        Vec2::new(x - wall - ri, -y + wall + ri),
                        ri,
                        PI * 1.5,
                        TAU,
                        *segments,
                        true,
                    );
                    arc(
                        &mut out,
                        Vec2::new(-x + wall + ri, -y + wall + ri),
                        ri,
                        PI,
                        PI * 1.5,
                        *segments,
                        true,
                    );
                } else {
                    out.extend([
                        Vec2::new(x - wall, -y + wall),
                        Vec2::new(-x + wall, -y + wall),
                    ]);
                }
                out.push(Vec2::new(-x + wall, y));
            }
            Self::Polyline { points } => {
                if points.len() < 3 || points.iter().flatten().any(|n| !n.is_finite()) {
                    return Err(AuthoringError::Invalid(
                        "profile polygon needs finite points",
                    ));
                }
                let points: Vec<_> = points.iter().map(|p| Vec2::new(p[0], p[1])).collect();
                out = Polyline::from_points(&points, true).tessellate(1e-6);
            }
        }
        out.dedup_by(|a, b| a.near(*b, 1e-10));
        if out.len() > 1
            && out
                .first()
                .zip(out.last())
                .is_some_and(|(a, b)| a.near(*b, 1e-10))
        {
            out.pop();
        }
        if out.len() < 3 || out.len() > limits.points || out.iter().any(|p| !p.is_finite()) {
            return Err(AuthoringError::Invalid(
                "profile tessellation degenerate or over budget",
            ));
        }
        if poet_geometry::shoelace(&out) <= 1e-16 {
            return Err(AuthoringError::Invalid(
                "profile must have positive nondegenerate CCW area",
            ));
        }
        Ok(out.into_iter().map(|p| [p.x, p.y]).collect())
    }
}
