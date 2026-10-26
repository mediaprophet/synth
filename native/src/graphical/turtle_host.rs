//! Admission only: geometry remains in Core's existing turtle host/kernel.
use super::GeometryHostLimits;
use qualia_core_db::specialized_libs::computational_geometry::turtle_3d::{
    TurtleCmd, MAX_POLYGON_VERTS, MAX_RING_SLICES, MAX_TURTLE_STACK,
};
use vibe::{DiagCode, Diagnostic, Span, Value};

const MAX_COMMANDS: usize = 65_536;
// This diagnostic profile bounds arithmetic as well as allocation. These are
// host input limits, not biological parameters or production-quality ceilings.
const MAX_ABS_NUMBER: f64 = 1_000_000.0;

pub(super) fn bad(span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagCode::E400, span, message)
}
fn count(value: Option<&Value>, default: usize, span: Span) -> Result<usize, Diagnostic> {
    let n = match value {
        None => return Ok(default),
        Some(Value::U64(n)) => *n,
        Some(Value::I64(n)) if *n >= 0 => *n as u64,
        _ => return Err(bad(span, "turtle count must be an unsigned integer")),
    };
    usize::try_from(n).map_err(|_| bad(span, "turtle count overflow"))
}
fn number(value: &Value, span: Span) -> Result<f64, Diagnostic> {
    let n = match value {
        Value::F64(n) => *n,
        Value::I64(n) => *n as f64,
        Value::U64(n) => *n as f64,
        _ => return Err(bad(span, "turtle command argument must be numeric")),
    };
    if !n.is_finite() || n.abs() > MAX_ABS_NUMBER {
        return Err(bad(span, "turtle command number exceeds finite host range"));
    }
    Ok(n)
}
fn add(a: usize, b: usize, span: Span) -> Result<usize, Diagnostic> {
    a.checked_add(b)
        .ok_or_else(|| bad(span, "turtle admission arithmetic overflow"))
}
fn mul(a: usize, b: usize, span: Span) -> Result<usize, Diagnostic> {
    a.checked_mul(b)
        .ok_or_else(|| bad(span, "turtle admission arithmetic overflow"))
}

/// Preflights the complete borrowed input before the canonical host allocates.
/// Returns a conservative charge for parsing, zeroing, tessellation and result
/// materialization. It does not evaluate turtle frames or generate geometry.
pub(super) fn admit(
    input: &Value,
    limits: GeometryHostLimits,
    remaining_work: usize,
    span: Span,
) -> Result<usize, Diagnostic> {
    let Value::Record(record) = input else {
        return Err(bad(span, "turtle arguments must be a record"));
    };
    if record
        .keys()
        .any(|k| !["commands", "ring_slices", "capacity"].contains(&k.as_str()))
    {
        return Err(bad(span, "unknown turtle argument"));
    }
    let Some(Value::List(commands)) = record.get("commands") else {
        return Err(bad(span, "turtle commands must be a list"));
    };
    if commands.is_empty() || commands.len() > MAX_COMMANDS || commands.len() > remaining_work {
        return Err(bad(span, "turtle command/work budget exceeded"));
    }
    let slices = count(record.get("ring_slices"), 8, span)?;
    let capacity = count(record.get("capacity"), 8192, span)?;
    if !(3..=MAX_RING_SLICES).contains(&slices)
        || capacity == 0
        || capacity > limits.max_vertices
        || capacity > u32::MAX as usize
    {
        return Err(bad(
            span,
            "turtle tessellation/capacity exceeds host working set",
        ));
    }
    // Core retains capacity Point3 + triangle slots while building nested Value
    // output. Charge both full output streams, parsed commands and a generous
    // fixed allowance for kernel stacks, Vec/record bookkeeping and alignment.
    let output_slot = mul(8, std::mem::size_of::<Value>(), span)?;
    let per_slot = add(36, output_slot, span)?;
    let bytes = add(
        add(
            mul(capacity, per_slot, span)?,
            mul(commands.len(), std::mem::size_of::<TurtleCmd>(), span)?,
            span,
        )?,
        65_536,
        span,
    )?;
    if bytes > limits.scratch_bytes {
        return Err(bad(span, "turtle scratch/output budget exceeded"));
    }
    let mut depth = 0usize;
    let mut polygon: Option<usize> = None;
    let mut cylinder = false;
    let mut vertices = 0usize;
    let mut triangles = 0usize;
    for command in commands {
        let Value::List(row) = command else {
            return Err(bad(span, "turtle command must be a list"));
        };
        let Some(Value::String(tag)) = row.first() else {
            return Err(bad(span, "turtle command tag must be a string"));
        };
        let arity_ok = match tag.as_str() {
            "F" => row.len() == 2 || row.len() == 3,
            "f" | "+" | "-" | "&" | "^" | "\\" | "/" | "!" => row.len() == 2,
            "T" => row.len() == 5,
            "$" | "[" | "]" | "{" | "." | "}" | "G" | "g" => row.len() == 1,
            _ => false,
        };
        if !arity_ok {
            return Err(bad(span, "unsupported turtle command tag or exact arity"));
        }
        for value in &row[1..] {
            number(value, span)?;
        }
        if matches!(tag.as_str(), "F" | "f" | "!") {
            for value in &row[1..] {
                if number(value, span)? <= 0.0 {
                    return Err(bad(span, "turtle dimensions must be positive"));
                }
            }
        }
        match tag.as_str() {
            "[" => {
                depth = add(depth, 1, span)?;
                if depth > MAX_TURTLE_STACK {
                    return Err(bad(span, "turtle stack overflow"));
                }
            }
            "]" => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| bad(span, "turtle stack underflow"))?;
            }
            "G" => {
                if cylinder {
                    return Err(bad(span, "nested turtle cylinder"));
                }
                cylinder = true;
            }
            "g" => {
                if !cylinder {
                    return Err(bad(span, "unopened turtle cylinder"));
                }
                cylinder = false;
            }
            "{" => {
                if polygon.is_some() {
                    return Err(bad(span, "nested turtle polygon"));
                }
                polygon = Some(0);
            }
            "}" => {
                let n = polygon
                    .take()
                    .ok_or_else(|| bad(span, "unopened turtle polygon"))?;
                if n < 3 {
                    return Err(bad(span, "turtle polygon needs three vertices"));
                }
                vertices = add(vertices, n, span)?;
                triangles = add(triangles, n - 2, span)?;
            }
            "T" => {
                if row[1..4]
                    .iter()
                    .all(|v| number(v, span).is_ok_and(|n| n == 0.0))
                {
                    return Err(bad(span, "turtle tropism direction must be nonzero"));
                }
            }
            _ => {}
        }
        if tag == "F" {
            // Every F emits at most two rings and two triangles per slice.
            // Generalized-cylinder welding can only reduce vertex output.
            vertices = add(vertices, mul(2, slices, span)?, span)?;
            triangles = add(triangles, mul(2, slices, span)?, span)?;
        }
        if tag == "." || (tag == "F" && polygon.is_some()) {
            let n = polygon
                .as_mut()
                .ok_or_else(|| bad(span, "vertex outside turtle polygon"))?;
            *n = add(*n, 1, span)?;
            if *n > MAX_POLYGON_VERTS {
                return Err(bad(span, "turtle polygon overflow"));
            }
        }
        if vertices > capacity || triangles > capacity {
            return Err(bad(
                span,
                "turtle capacity below conservative output requirement",
            ));
        }
    }
    if depth != 0 || polygon.is_some() || cylinder {
        return Err(bad(span, "unclosed turtle stack/polygon/cylinder"));
    }
    if vertices == 0 || triangles == 0 {
        return Err(bad(span, "turtle commands emit no mesh"));
    }
    let work = add(
        add(mul(commands.len(), 8, span)?, mul(capacity, 2, span)?, span)?,
        add(vertices, mul(triangles, 3, span)?, span)?,
        span,
    )?;
    if work > remaining_work {
        return Err(bad(span, "turtle cumulative work budget exceeded"));
    }
    Ok(work)
}
