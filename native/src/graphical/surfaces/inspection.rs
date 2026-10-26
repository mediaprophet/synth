//! Explicit selected-part inspection on the existing authorized native worker.
use super::{uv_atlas, TextureResources};
use crate::graphical::preview::{self, PreviewFrame, PreviewLimits, SourceRef};
use qualia_core_db::specialized_libs::computational_geometry::uv_atlas::AtlasLimits;
use std::collections::BTreeSet;
use vibe::Value;
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceInspection {
    pub source: SourceRef,
    pub part_id: String,
    pub uv_image: String,
    pub texture_image: Option<String>,
    /// SHA-256 of the actual resolved encoded base-colour resource.
    pub texture_sha256: Option<[u8; 32]>,
    pub summary: String,
    pub edges: Vec<[u32; 2]>,
}
pub fn atlas_limits(limits: PreviewLimits) -> AtlasLimits {
    AtlasLimits {
        vertices: limits.vertices,
        triangles: limits.indices / 3,
        ..AtlasLimits::default()
    }
}
fn stream<const N: usize>(value: &Value, count: usize) -> Result<Vec<[f32; N]>, String> {
    let Value::List(values) = value else {
        return Err("Geometry stream must be a list".into());
    };
    let mut result = Vec::new();
    if matches!(values.first(), Some(Value::List(_))) {
        if values.len() != count {
            return Err("Geometry stream count differs".into());
        }
        for value in values {
            let Value::List(row) = value else {
                return Err("Mixed geometry rows".into());
            };
            if row.len() != N {
                return Err("Geometry row width differs".into());
            }
            let mut out = [0.; N];
            for (o, v) in out.iter_mut().zip(row) {
                *o = crate::graphical::authoring::source::number(v)? as f32;
            }
            result.push(out);
        }
    } else {
        if values.len() != count.checked_mul(N).ok_or("Stream count overflow")? {
            return Err("Geometry stream count differs".into());
        }
        for row in values.chunks_exact(N) {
            let mut out = [0.; N];
            for (o, v) in out.iter_mut().zip(row) {
                *o = crate::graphical::authoring::source::number(v)? as f32;
            }
            result.push(out);
        }
    }
    Ok(result)
}
fn line(pixels: &mut [u8], mut a: [i32; 2], b: [i32; 2], colour: [u8; 4]) {
    let dx = (b[0] - a[0]).abs();
    let sx = if a[0] < b[0] { 1 } else { -1 };
    let dy = -(b[1] - a[1]).abs();
    let sy = if a[1] < b[1] { 1 } else { -1 };
    let mut err = dx + dy;
    for _ in 0..514 {
        let i = (a[1] as usize * 256 + a[0] as usize) * 4;
        pixels[i..i + 4].copy_from_slice(&colour);
        if a == b {
            break;
        }
        let e = 2 * err;
        if e >= dy {
            err += dy;
            a[0] += sx;
        }
        if e <= dx {
            err += dx;
            a[1] += sy;
        }
    }
}
fn uri(source: &SourceRef, pixels: Vec<u8>, side: u32) -> Result<String, String> {
    preview::native_image_uri(&PreviewFrame {
        source: source.clone(),
        session_revision: None,
        execution_epoch: None,
        side,
        rgba8: pixels,
    })
}
fn clipped(a: [f32; 2], b: [f32; 2]) -> Option<([i32; 2], [i32; 2])> {
    let a = [a[0] as f64, a[1] as f64];
    let d = [b[0] as f64 - a[0], b[1] as f64 - a[1]];
    let (mut lo, mut hi) = (0f64, 1f64);
    for (p, q) in [
        (-d[0], a[0]),
        (d[0], 1. - a[0]),
        (-d[1], a[1]),
        (d[1], 1. - a[1]),
    ] {
        if p == 0. {
            if q < 0. {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0. {
                lo = lo.max(r);
            } else {
                hi = hi.min(r);
            }
            if lo > hi {
                return None;
            }
        }
    }
    let point = |t: f64| {
        [
            ((a[0] + d[0] * t).clamp(0., 1.) * 255.).round() as i32,
            ((1. - (a[1] + d[1] * t).clamp(0., 1.)) * 255.).round() as i32,
        ]
    };
    Some((point(lo), point(hi)))
}
pub fn inspect(
    parts: &Value,
    id: &str,
    source: SourceRef,
    resources: Option<&TextureResources>,
    vertices: usize,
) -> Result<SurfaceInspection, String> {
    source.check_current()?;
    let Value::List(parts) = parts else {
        return Err("Missing authored parts".into());
    };
    if parts.len() > 256 || id.is_empty() || id.len() > 128 {
        return Err("Surface inspection bounds exceeded".into());
    }
    let selected=parts.iter().find(|p|matches!(p,Value::Record(r) if matches!(r.get("id"),Some(Value::String(s)) if s==id))).ok_or("Selected surface part disappeared")?;
    let Value::Record(p) = selected else {
        return Err("Selected part record required".into());
    };
    let limits = PreviewLimits {
        side: 256,
        vertices,
        indices: vertices.checked_mul(6).ok_or("Inspection index overflow")?,
    };
    // Inspect the current binding even when its last source cell returned a scalar.
    // Shared borrowed preflight precedes all selected geometry/pixel construction.
    if parts.iter().filter(|p|matches!(p,Value::Record(r) if matches!(r.get("id"),Some(Value::String(s)) if s==id))).count()!=1{return Err("Ambiguous selected part identifier".into());}
    let (v, t, out) =
        crate::graphical::assembly::inspect_preflight(selected, limits, atlas_limits(limits))?;
    if t > 32768
        || v.max(out)
            .checked_mul(128)
            .and_then(|n| t.checked_mul(96).and_then(|t| n.checked_add(t)))
            .is_none_or(|n| n > 64 * 1024 * 1024)
    {
        return Err("Selected UV diagnostic triangle/scratch admission exceeded".into());
    }
    let resolved = super::resolution::resolve_parts(std::slice::from_ref(selected), resources)?;
    let g = p.get("geometry").ok_or("No selected geometry")?;
    let Value::Record(gr) = g else {
        return Err("Geometry record required".into());
    };
    let mut mesh = preview::mesh_from_invoke(g, limits)?;
    let mut edges = BTreeSet::new();
    for t in &mesh.triangles {
        for i in 0..3 {
            let a = t[i];
            let b = t[(i + 1) % 3];
            let e = [a.min(b), a.max(b)];
            if !edges.contains(&e) && edges.len() == 65536 {
                return Err("Seam edge inspection budget exceeded".into());
            }
            edges.insert(e);
        }
    }
    let normals = gr
        .get("normals")
        .map(|v| stream::<3>(v, mesh.positions.len()))
        .transpose()?;
    let mut uv = gr
        .get("uv0")
        .map(|v| stream::<2>(v, mesh.positions.len()))
        .transpose()?;
    let mut summary = "Stored UV0; atlas packing is not asserted.".to_string();
    if let Some(d) = p.get("uv_atlas").filter(|v| !matches!(v, Value::Null)) {
        let draft = uv_atlas::from_value(d)?;
        let applied = uv_atlas::apply(
            &draft,
            &mesh,
            normals.as_deref(),
            atlas_limits(limits),
            &|| false,
        )?;
        summary=format!("{} disk charts; {} split vertices; {} px atlas; {} px reserved outer gutter; target {} area texels/local metre BEFORE part TRS. Measured triangle density range {}–{}; harmonic mapping may distort.",applied.charts.len(),applied.mesh.positions.len(),draft.atlas_size,draft.gutter_pixels,draft.texels_per_metre,
            applied.charts.iter().map(|c|c.min_local_texels_per_metre).fold(f64::INFINITY,f64::min),applied.charts.iter().map(|c|c.max_local_texels_per_metre).fold(0.,f64::max));
        mesh = applied.mesh;
        uv = Some(applied.uv0);
    } else if let Some(d) = p.get("uv_projection").filter(|v| !matches!(v, Value::Null)) {
        uv = Some(super::uv::project(
            &super::uv::projection_from_value(d)?,
            &mesh.positions,
        )?);
        summary = "Explicit planar projection; no unwrap/atlas claim.".into();
    }
    if let (Some(uv), Some(t)) = (uv.as_mut(), p.get("uv_transform")) {
        super::uv::apply(&super::uv::from_value(t)?, uv)?;
    }
    let mut pixels = vec![0u8; 256 * 256 * 4];
    for p in pixels.chunks_exact_mut(4) {
        p.copy_from_slice(&[20, 28, 36, 255]);
    }
    if let Some(uv) = uv.as_ref() {
        for triangle in &mesh.triangles {
            for i in 0..3 {
                let a = uv[triangle[i] as usize];
                let b = uv[triangle[(i + 1) % 3] as usize];
                if let Some((a, b)) = clipped(a, b) {
                    line(&mut pixels, a, b, [80, 220, 188, 255]);
                }
            }
        }
    } else {
        summary.push_str(" No authored UV channel available.");
    }
    let authored = crate::graphical::materials::material_from_value(
        p.get("material").ok_or("No selected material")?,
    )?;
    let material = resolved.materials.get(&authored.id).unwrap_or(&authored);
    let texture_image = if material.base_color_texture != [0; 32] {
        let r = resolved
            .resources
            .as_ref()
            .ok_or("Selected texture resources unavailable")?;
        let live = resources
            .map_or(0, TextureResources::snapshot_bytes)
            .checked_add(
                resolved
                    .original_inputs
                    .iter()
                    .map(|d| d.bytes.len())
                    .sum::<usize>(),
            )
            .and_then(|n| v.max(out).checked_mul(128).and_then(|g| n.checked_add(g)))
            .and_then(|n| t.checked_mul(96).and_then(|g| n.checked_add(g)))
            .and_then(|n| n.checked_add(3 * 256 * 256 * 4))
            .ok_or("Selected inspection composite size overflow")?;
        let available = super::SurfaceTextureLimits::default()
            .working_bytes
            .checked_sub(live)
            .ok_or("Selected inspection live resources exceed combined working admission")?;
        let input = r.preview_input(material.base_color_texture, available)?;
        let mut small = vec![0; 256 * 256 * 4];
        for y in 0..256usize {
            for x in 0..256usize {
                let from = ((y * input.height as usize / 256) * input.width as usize
                    + x * input.width as usize / 256)
                    * 4;
                small[(y * 256 + x) * 4..(y * 256 + x) * 4 + 4]
                    .copy_from_slice(&input.rgba[from..from + 4]);
            }
        }
        Some(uri(&source, small, 256)?)
    } else {
        None
    };
    let texture_sha256 = (material.base_color_texture != [0; 32]).then_some(material.base_color_texture);
    if let Some(digest) = texture_sha256 {
        summary.push_str(&format!(" Base colour resource SHA-256 {}.", super::binding::digest_hex(&digest)));
    }
    source.check_current()?;
    Ok(SurfaceInspection {
        source: source.clone(),
        part_id: id.into(),
        uv_image: uri(&source, pixels, 256)?,
        texture_image,
        texture_sha256,
        summary,
        edges: edges.into_iter().collect(),
    })
}
