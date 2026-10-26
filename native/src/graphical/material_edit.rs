//! Admission of the narrow MaterialDraft::edit_source form. Canonical Vibe
//! remains the parser/checker/evaluator; this is an action-specific allowlist.
//! Native material factor/mesh validation still runs before session promotion.
use vibe::{Expr, ExprKind, Item, Literal, Stmt, UnOp};

fn denied() -> String {
    "Material edit must only replace the admitted literal material binding".into()
}

// Conservative ingress bounds before recursive canonical parsing. Generated
// drafts are ASCII, small, shallow and contain no comments or operators except
// a literal minus. This is not a second parser or a general Vibe source profile.
fn bound(source: &str) -> Result<(), String> {
    if source.is_empty() || source.len() > 4096 || !source.is_ascii() {
        return Err(denied());
    }
    let bytes = source.as_bytes();
    let mut groups = [0u8; 4];
    let mut depth = 0usize;
    let mut quoted = false;
    let mut words = 0usize;
    let mut word = false;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if quoted {
            if byte == b'"' {
                quoted = false;
            } else if !byte.is_ascii_alphabetic() {
                // Generated strings are only linear/Opaque/Mask/Blend. Reject
                // interpolation/escape syntax before any embedded parser runs.
                return Err(denied());
            }
            continue;
        }
        if byte == b'"' {
            quoted = true;
            word = false;
            continue;
        }
        let in_word = byte.is_ascii_alphanumeric() || byte == b'_';
        if in_word && !word {
            words += 1;
            if words > 128 {
                return Err(denied());
            }
        }
        word = in_word;
        match byte {
            b'(' | b'[' | b'{' => {
                if depth == groups.len() {
                    return Err(denied());
                }
                groups[depth] = byte;
                depth += 1;
            }
            b')' | b']' | b'}' => {
                if depth == 0 {
                    return Err(denied());
                }
                depth -= 1;
                if !matches!(
                    (groups[depth], byte),
                    (b'(', b')') | (b'[', b']') | (b'{', b'}')
                ) {
                    return Err(denied());
                }
            }
            b'+' | b'*' | b'/' | b'%' | b'!' | b'?' | b'<' | b'>' | b'&' | b'|' | b'~' => {
                return Err(denied())
            }
            b'-' if !bytes.get(index + 1).is_some_and(u8::is_ascii_digit) => return Err(denied()),
            b'.' if index == 0
                || !bytes[index - 1].is_ascii_digit()
                || !bytes.get(index + 1).is_some_and(u8::is_ascii_digit) =>
            {
                return Err(denied())
            }
            _ => {}
        }
    }
    if quoted || depth != 0 {
        return Err(denied());
    }
    Ok(())
}

fn ident(expr: &Expr, expected: &str) -> bool {
    matches!(&expr.kind,ExprKind::Ident(name) if name == expected)
}
fn number(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Literal(Literal::Int(_) | Literal::UInt(_)) => true,
        ExprKind::Literal(Literal::Float(bits)) => {
            let value = f64::from_bits(*bits);
            value.is_finite() && (value as f32).is_finite()
        }
        ExprKind::Unary {
            op: UnOp::Neg,
            expr,
        } => matches!(expr.kind, ExprKind::Literal(_)) && number(expr),
        _ => false,
    }
}
fn components(expr: &Expr, count: usize) -> bool {
    matches!(&expr.kind,ExprKind::List(values) if values.len()==count && values.iter().all(number))
}

/// Does not execute source or grant authority. The caller must already hold the
/// MaterialEdit owner decision for the actual current source and target ID.
pub fn validate(source: &str, next_revision: u64, current_material_id: u64) -> Result<(), String> {
    if next_revision == 0 || current_material_id == 0 {
        return Err(denied());
    }
    bound(source)?;
    // The pinned parser consumes but does not retain MaterialDecl's type name.
    // Retain that one generated header constraint before inspecting its AST.
    let surface = format!("poet_surface_{next_revision}");
    let header = format!("material {surface}: Material");
    if !source
        .trim_start()
        .strip_prefix(&header)
        .is_some_and(|tail| tail.as_bytes().first().is_some_and(u8::is_ascii_whitespace))
    {
        return Err(denied());
    }
    let program = vibe::parse_program(source).map_err(|_| denied())?;
    if program.module.is_some()
        || !program.imports.is_empty()
        || !program.prefixes.is_empty()
        || !program.locales.is_empty()
        || !program.requires.is_empty()
        || program.items.len() != 3
    {
        return Err(denied());
    }
    let Item::Material(material) = &program.items[0] else {
        return Err(denied());
    };
    if material.name != surface || material.properties.len() != 10 {
        return Err(denied());
    }
    const FIELDS: [&str; 10] = [
        "id",
        "colour_space",
        "base_color",
        "emissive",
        "metallic",
        "roughness",
        "specular",
        "alpha_cutoff",
        "opacity_mode",
        "double_sided",
    ];
    let mut seen = [false; 10];
    for property in &material.properties {
        let Some(index) = FIELDS.iter().position(|field| *field == property.name) else {
            return Err(denied());
        };
        if seen[index] {
            return Err(denied());
        }
        seen[index] = true;
        let valid = match property.name.as_str() {
            "id" => match &property.value.kind {
                ExprKind::Literal(Literal::UInt(id)) => *id == current_material_id,
                ExprKind::Literal(Literal::Int(id)) => {
                    u64::try_from(*id).ok() == Some(current_material_id)
                }
                _ => false,
            },
            "colour_space" => {
                matches!(&property.value.kind,ExprKind::Literal(Literal::String(value)) if value=="linear")
            }
            "opacity_mode" => {
                matches!(&property.value.kind,ExprKind::Literal(Literal::String(value)) if matches!(value.as_str(),"Opaque"|"Mask"|"Blend"))
            }
            "double_sided" => matches!(property.value.kind, ExprKind::Literal(Literal::Bool(_))),
            "base_color" => components(&property.value, 4),
            "emissive" => components(&property.value, 3),
            _ => number(&property.value),
        };
        if !valid {
            return Err(denied());
        }
    }
    match &program.items[1] {
        Item::Statement(Stmt::Assign { target, value, .. })
            if ident(target, "poet_material") && ident(value, &surface) => {}
        _ => return Err(denied()),
    }
    let Item::Statement(Stmt::Expr { expr, .. }) = &program.items[2] else {
        return Err(denied());
    };
    let ExprKind::Record(fields) = &expr.kind else {
        return Err(denied());
    };
    if fields.len() != 2
        || !fields
            .iter()
            .any(|field| field.name == "geometry" && ident(&field.value, "poet_geometry"))
        || !fields
            .iter()
            .any(|field| field.name == "material" && ident(&field.value, "poet_material"))
    {
        return Err(denied());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> String {
        "material poet_surface_2: Material\n id: 99u64, colour_space: \"linear\", base_color: [0.3,0.2,0.1,1.0], emissive: [0.0,0.0,0.0], metallic: 0.0, roughness: 0.6, specular: 1.0, alpha_cutoff: 0.5, opacity_mode: \"Opaque\", double_sided: false;\npoet_material = poet_surface_2;\n({geometry: poet_geometry, material: poet_material});".into()
    }
    #[test]
    fn canonical_literal_material_edit_is_admitted() {
        validate(&source(), 2, 99).unwrap();
        validate(&source().replace("0.3", "-0.0"), 2, 99).unwrap();
        validate(&source().replace("0.3", "1e-5"), 2, 99).unwrap();
    }
    #[test]
    fn current_revision_material_and_geometry_binding_cannot_be_retargeted() {
        assert!(validate(&source(), 3, 99).is_err());
        assert!(validate(&source(), 2, 100).is_err());
        for source in [
            source().replace("poet_material =", "poet_geometry ="),
            source().replace("geometry: poet_geometry", "geometry: other_geometry"),
            source().replace("material: poet_material", "material: poet_surface_2"),
            source().replace("99u64", "100u64"),
            source().replace(": Material", ": UnadmittedType"),
            source().replace(
                "poet_material = poet_surface_2",
                "poet_material = other_surface",
            ),
        ] {
            assert!(validate(&source, 2, 99).is_err());
        }
    }
    #[test]
    fn host_calls_computation_control_flow_and_extra_items_are_rejected() {
        for source in [
            source().replace(
                "roughness: 0.6",
                "roughness: capability.invoke(\"Anything\", {})",
            ),
            source().replace("roughness: 0.6", "roughness: 0.3 + 0.3"),
            source().replace("roughness: 0.6", "roughness: other_value"),
            source().replace("roughness: 0.6", "roughness: if true { 0.6 } else { 0.4 }"),
            format!("{}poet_geometry = poet_material;", source()),
            format!("requires [capability(\"capability.invoke\")];{}", source()),
            source().replace("double_sided: false", "roughness: 0.4"),
            source().replace("specular: 1.0", "normal_texture: \"unverified\""),
        ] {
            assert!(validate(&source, 2, 99).is_err());
        }
    }
    #[test]
    fn source_depth_and_size_fail_before_recursive_parser() {
        assert!(validate(&"(".repeat(1000), 2, 99).is_err());
        assert!(validate(&"x".repeat(4097), 2, 99).is_err());
        assert!(validate(&source().replace("0.6", &"await ".repeat(200)), 2, 99).is_err());
        assert!(validate(&source().replace("\"linear\"", "\"${((((1))))}\""), 2, 99).is_err());
    }
}
