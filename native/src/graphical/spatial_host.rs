//! Typed Vibe projection onto the Poet spatial authoring helpers. No mesh algorithms here.
use poet_spatial_authoring::{GeometryLimits, Profile};
use qualia_core_db::render::assets::Mesh;
use vibe::{DiagCode, Diagnostic, Span, Value};
fn bad(span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagCode::E400, span, message)
}
fn record<'a>(
    v: &'a Value,
    allowed: &[&str],
    span: Span,
) -> Result<&'a std::collections::BTreeMap<String, Value>, Diagnostic> {
    let Value::Record(r) = v else {
        return Err(bad(span, "spatial authoring arguments must be a record"));
    };
    if r.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err(bad(span, "Unknown spatial authoring operator parameter"));
    }
    Ok(r)
}
fn number(
    r: &std::collections::BTreeMap<String, Value>,
    key: &str,
    span: Span,
) -> Result<f64, Diagnostic> {
    let n = match r.get(key) {
        Some(Value::F64(v)) => *v,
        Some(Value::I64(v)) => *v as f64,
        Some(Value::U64(v)) => *v as f64,
        _ => {
            return Err(bad(
                span,
                format!("spatial authoring parameter {key} must be numeric"),
            ))
        }
    };
    if !n.is_finite() {
        return Err(bad(
            span,
            format!("spatial authoring parameter {key} must be finite"),
        ));
    }
    Ok(n)
}
fn count(
    r: &std::collections::BTreeMap<String, Value>,
    key: &str,
    span: Span,
) -> Result<usize, Diagnostic> {
    match r.get(key) {
        Some(Value::U64(v)) => {
            usize::try_from(*v).map_err(|_| bad(span, "spatial authoring count overflow"))
        }
        Some(Value::I64(v)) if *v >= 0 => {
            usize::try_from(*v).map_err(|_| bad(span, "spatial authoring count overflow"))
        }
        _ => Err(bad(
            span,
            format!("spatial authoring parameter {key} must be an unsigned integer"),
        )),
    }
}
fn profile(v: &Value, span: Span, max_points: usize) -> Result<Profile, Diagnostic> {
    let r = record(
        v,
        &[
            "kind",
            "width",
            "height",
            "radius",
            "wall",
            "corner_radius",
            "segments",
            "points",
        ],
        span,
    )?;
    let allowed: &[&str] = match r.get("kind") {
        Some(Value::String(s)) if s == "rounded_rectangle" => {
            &["kind", "width", "height", "radius", "segments"]
        }
        Some(Value::String(s)) if s == "circle" => &["kind", "radius", "segments"],
        Some(Value::String(s)) if s == "open_channel" => &[
            "kind",
            "width",
            "height",
            "wall",
            "corner_radius",
            "segments",
        ],
        Some(Value::String(s)) if s == "polyline" => &["kind", "points", "segments"],
        _ => return Err(bad(span, "Unknown spatial authoring profile kind")),
    };
    if r.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(bad(
            span,
            "Parameter does not apply to this spatial authoring profile kind",
        ));
    }
    let segments = count(r, "segments", span)?;
    let n = |key| number(r, key, span);
    match r.get("kind") {
        Some(Value::String(s)) if s == "rounded_rectangle" => Ok(Profile::RoundedRectangle {
            width: n("width")?,
            height: n("height")?,
            radius: n("radius")?,
            segments,
        }),
        Some(Value::String(s)) if s == "circle" => Ok(Profile::Circle {
            radius: n("radius")?,
            segments,
        }),
        Some(Value::String(s)) if s == "open_channel" => Ok(Profile::OpenChannel {
            width: n("width")?,
            height: n("height")?,
            wall: n("wall")?,
            corner_radius: n("corner_radius")?,
            segments,
        }),
        Some(Value::String(s)) if s == "polyline" => {
            if segments != 1 {
                return Err(bad(span, "Polyline segments must be exactly 1"));
            }
            let Some(Value::List(rows)) = r.get("points") else {
                return Err(bad(span, "Polyline points must be a list"));
            };
            if rows.len() < 3 || rows.len() > max_points {
                return Err(bad(span, "Polyline exceeds host point budget"));
            }
            let mut points = Vec::with_capacity(rows.len());
            for row in rows {
                let Value::List(values) = row else {
                    return Err(bad(span, "Polyline rows need two coordinates"));
                };
                if values.len() != 2 {
                    return Err(bad(span, "Polyline rows need two coordinates"));
                }
                let mut point = [0.0; 2];
                for (i, value) in values.iter().enumerate() {
                    point[i] = match value {
                        Value::F64(n) => *n,
                        Value::I64(n) => *n as f64,
                        Value::U64(n) => *n as f64,
                        _ => return Err(bad(span, "Polyline coordinate must be numeric")),
                    };
                }
                points.push(point);
            }
            Ok(Profile::Polyline { points })
        }
        _ => Err(bad(span, "Unknown spatial authoring profile kind")),
    }
}
fn mesh_value(mesh: Mesh) -> Value {
    let vertices = mesh
        .positions
        .into_iter()
        .map(|p| Value::List(p.into_iter().map(|v| Value::F64(v as f64)).collect()))
        .collect();
    let triangles = mesh
        .triangles
        .into_iter()
        .map(|t| Value::List(t.into_iter().map(|i| Value::U64(i as u64)).collect()))
        .collect();
    Value::Record(std::collections::BTreeMap::from([
        ("vertices".into(), Value::List(vertices)),
        ("triangles".into(), Value::List(triangles)),
    ]))
}
pub(super) fn invoke(
    id: &str,
    args: &Value,
    span: Span,
    limits: GeometryLimits,
) -> Result<(Value, usize), Diagnostic> {
    let result = match id {
        "ComputationalGeometry.revolve_profile" => {
            let r=record(args,&["points","segments"],span)?;
            let Some(Value::List(rows))=r.get("points") else {return Err(bad(span,"Revolution needs radius-height pairs"));};
            if !(2..=256).contains(&rows.len()) || rows.len()>limits.profile_points {
                return Err(bad(span,"Revolution exceeds host profile-point budget"));
            }
            let mut points=Vec::with_capacity(rows.len());
            for row in rows {
                let Value::List(values)=row else {return Err(bad(span,"Revolution station must be a radius-height pair"));};
                if values.len()!=2 {return Err(bad(span,"Revolution station must have exactly radius and height"));}
                let mut point=[0.0;2];
                for (i,value) in values.iter().enumerate() {
                    point[i]=match value {Value::F64(n)=>*n,Value::I64(n)=>*n as f64,Value::U64(n)=>*n as f64,_=>return Err(bad(span,"Revolution radius-height coordinates must be numeric metres"))};
                }
                points.push(point);
            }
            super::revolve_host::mesh(&points,count(r,"segments",span)?,limits)
        }
        "ComputationalGeometry.profile_sweep" => {
            let r = record(args, &["profile", "axis", "length", "caps"], span)?;
            let p = profile(
                r.get("profile")
                    .ok_or_else(|| bad(span, "spatial authoring sweep needs profile"))?,
                span,
                limits.profile_points,
            )?;
            let caps = match r.get("caps") {
                Some(Value::Bool(v)) => *v,
                None => true,
                _ => return Err(bad(span, "spatial authoring caps must be boolean")),
            };
            let axis = match (r.get("axis"), r.get("length")) {
                (Some(Value::List(rows)), None) if (2..=4096).contains(&rows.len()) => {
                    let mut axis = Vec::with_capacity(rows.len());
                    for row in rows {
                        let Value::List(values) = row else {
                            return Err(bad(
                                span,
                                "spatial authoring axis rows need three coordinates",
                            ));
                        };
                        if values.len() != 3 {
                            return Err(bad(
                                span,
                                "spatial authoring axis rows need three coordinates",
                            ));
                        }
                        let mut point = [0.0; 3];
                        for (i, value) in values.iter().enumerate() {
                            point[i] = match value {
                                Value::F64(n) => *n,
                                Value::I64(n) => *n as f64,
                                Value::U64(n) => *n as f64,
                                _ => {
                                    return Err(bad(
                                        span,
                                        "spatial authoring axis coordinate must be numeric",
                                    ))
                                }
                            };
                        }
                        axis.push(point);
                    }
                    axis
                }
                (None, Some(_)) => vec![[0.0; 3], [0.0, 0.0, number(r, "length", span)?]],
                _ => {
                    return Err(bad(
                        span,
                        "spatial authoring sweep needs length XOR a bounded axis list",
                    ))
                }
            };
            poet_spatial_authoring::profile_sweep(&p, &axis, caps, limits)
        }
        "ComputationalGeometry.uv_sphere" => {
            let r = record(args, &["radius", "lat_segments", "lon_segments"], span)?;
            poet_spatial_authoring::sphere(
                number(r, "radius", span)?,
                count(r, "lat_segments", span)?,
                count(r, "lon_segments", span)?,
                limits,
            )
        }
        "ComputationalGeometry.torus" => {
            let r = record(
                args,
                &[
                    "major_radius",
                    "minor_radius",
                    "major_segments",
                    "minor_segments",
                ],
                span,
            )?;
            poet_spatial_authoring::torus(
                number(r, "major_radius", span)?,
                number(r, "minor_radius", span)?,
                count(r, "major_segments", span)?,
                count(r, "minor_segments", span)?,
                limits,
            )
        }
        _ => {
            return Err(bad(
                span,
                "Unknown spatial authoring-backed native geometry operator",
            ))
        }
    }
    .map_err(|e| bad(span, e.to_string()))?;
    let count = result.positions.len();
    Ok((mesh_value(result), count))
}
