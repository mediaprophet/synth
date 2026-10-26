//! Derived native previews over actual Qualia APIs. No model/source authority.
use qualia_core_db::container_10d::{MaterialRecord, SubmeshRange};
use qualia_core_db::poet_host::invoke::studio_store;
use qualia_core_db::render::{assets::Mesh, gpu::PortalGpu, telemetry::SystemTelemetry};
use vibe::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRef {
    pub artifact_id: String,
    pub stamp: studio_store::StoreStamp,
}

impl SourceRef {
    pub fn check_current(&self) -> Result<(), String> {
        if !studio_store::artifact_stamp_is_current(&self.artifact_id, &self.stamp) {
            return Err("source stamp could not be revalidated; result held".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct PreviewLimits {
    pub side: u32,
    pub vertices: usize,
    pub indices: usize,
}

impl Default for PreviewLimits {
    fn default() -> Self {
        Self {
            side: 512,
            vertices: 200_000,
            indices: 1_200_000,
        }
    }
}

#[derive(Clone, Copy)]
pub struct PreviewCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: [f32; 3],
}

#[derive(Clone)]
pub struct PreviewFrame {
    pub source: SourceRef,
    pub session_revision: Option<u64>,
    pub execution_epoch: Option<super::controller::SessionEpoch>,
    pub side: u32,
    pub rgba8: Vec<u8>,
}

/// Existing wave27 extrusion returns nested triples; wave29 swung surfaces
/// return flat streams. Admit both actual public result shapes without a codec.
fn stream_values<'a>(values: &'a [Value], maximum: usize) -> Result<Vec<&'a Value>, String> {
    if matches!(values.first(), Some(Value::List(_))) {
        let count = values
            .len()
            .checked_mul(3)
            .ok_or("preview result size overflow")?;
        if count > maximum {
            return Err("geometry stream exceeds preview budget".into());
        }
        let mut flat = Vec::with_capacity(count);
        for row in values {
            let Value::List(row) = row else {
                return Err("mixed geometry stream shapes".into());
            };
            if row.len() != 3 {
                return Err("geometry rows must contain exactly three values".into());
            }
            flat.extend(row.iter());
        }
        Ok(flat)
    } else {
        if values.len() > maximum || values.len() % 3 != 0 {
            return Err("invalid or oversized flat geometry stream".into());
        }
        Ok(values.iter().collect())
    }
}

/// Marshals actual existing ComputationalGeometry invoke output only.
pub fn mesh_from_invoke(value: &Value, limits: PreviewLimits) -> Result<Mesh, String> {
    let Value::Record(record) = value else {
        return Err("geometry result must be a record".into());
    };
    let Some(Value::List(vertices)) = record.get("vertices") else {
        return Err("geometry result lacks vertices".into());
    };
    let Some(Value::List(triangles)) = record.get("triangles") else {
        return Err("geometry result lacks triangles".into());
    };
    let max_vertex_scalars = limits
        .vertices
        .checked_mul(3)
        .ok_or("preview vertex budget overflow")?;
    let vertices = stream_values(vertices, max_vertex_scalars)?;
    let triangles = stream_values(triangles, limits.indices)?;
    if vertices.is_empty()
        || vertices.len() % 3 != 0
        || vertices.len() / 3 > limits.vertices
        || triangles.is_empty()
        || triangles.len() % 3 != 0
        || triangles.len() > limits.indices
    {
        return Err(
            "geometry result exceeds declared preview bounds or has invalid topology".into(),
        );
    }
    let numbers: Result<Vec<f32>, String> = vertices
        .into_iter()
        .map(|value| {
            let value = match value {
                Value::F64(v) => *v,
                Value::I64(v) => *v as f64,
                Value::U64(v) => *v as f64,
                _ => return Err("vertex is not numeric".into()),
            };
            let result = value as f32;
            if !result.is_finite() {
                return Err("vertex is not finite f32".into());
            }
            Ok(result)
        })
        .collect();
    let positions: Vec<[f32; 3]> = numbers?
        .chunks_exact(3)
        .map(|v| [v[0], v[1], v[2]])
        .collect();
    let indices: Result<Vec<u32>, String> = triangles
        .into_iter()
        .map(|value| {
            let value = match value {
                Value::U64(v) => *v,
                Value::I64(v) if *v >= 0 => *v as u64,
                _ => return Err("index is not an unsigned integer".into()),
            };
            if value >= positions.len() as u64 {
                return Err("geometry index is out of bounds".into());
            }
            Ok(value as u32)
        })
        .collect();
    let triangles = indices?
        .chunks_exact(3)
        .map(|v| [v[0], v[1], v[2]])
        .collect();
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for position in &positions {
        for axis in 0..3 {
            min[axis] = min[axis].min(position[axis]);
            max[axis] = max[axis].max(position[axis]);
        }
    }
    Ok(Mesh {
        positions,
        triangles,
        min,
        max,
    })
}

pub(crate) fn validate_frame(side: u32, camera: PreviewCamera) -> Result<(), String> {
    if !(8..=1024).contains(&side)
        || !camera.distance.is_finite()
        || camera.distance <= 0.0
        || [camera.yaw, camera.pitch]
            .iter()
            .chain(camera.target.iter())
            .any(|v| !v.is_finite())
    {
        return Err("invalid native preview extent/camera".into());
    }
    Ok(())
}

/// Factor-only material preview. Bound texture slots are refused rather than
/// silently rendered with defaults; textured masters use the HMC adapter below.
pub fn render_mesh(
    source: SourceRef,
    mesh: &Mesh,
    material: MaterialRecord,
    camera: PreviewCamera,
    limits: PreviewLimits,
) -> Result<PreviewFrame, String> {
    source.check_current()?;
    validate_frame(limits.side, camera)?;
    super::materials::validate_factor_material(&material)?;
    if mesh.positions.is_empty()
        || mesh.positions.len() > limits.vertices
        || mesh.triangles.is_empty()
        || mesh
            .triangles
            .len()
            .checked_mul(3)
            .is_none_or(|n| n > limits.indices)
        || mesh.positions.iter().flatten().any(|v| !v.is_finite())
        || mesh
            .triangles
            .iter()
            .flatten()
            .any(|i| *i as usize >= mesh.positions.len())
    {
        return Err("invalid or oversized native preview mesh".into());
    }
    let indices: Vec<u32> = mesh.triangles.iter().flatten().copied().collect();
    let colours = vec![[1.0; 4]; mesh.positions.len()];
    let ranges = [SubmeshRange {
        first_index: 0,
        index_count: indices.len() as u32,
        material_id: material.id,
        semantic_id: 0,
    }];
    let mut renderer = PortalGpu::new_offscreen(limits.side, limits.side, 1)?;
    renderer.set_ambient_enabled(false);
    renderer.set_tensor_projection_enabled(false);
    renderer.set_clear_color(0.025, 0.035, 0.05, 1.0);
    renderer.set_camera_target(
        camera.yaw,
        camera.pitch,
        camera.distance,
        camera.target[0],
        camera.target[1],
        camera.target[2],
    );
    renderer.upload_mesh_colored_with_frames_and_materials(
        &mesh.positions,
        &colours,
        None,
        None,
        &indices,
        &[material],
        &ranges,
    )?;
    renderer.render(0.0, &SystemTelemetry::default())?;
    let mut rgba8 = vec![0; renderer.required_rgba8_bytes()];
    renderer.read_rgba8_into(&mut rgba8)?;
    source.check_current()?;
    Ok(PreviewFrame {
        source,
        session_revision: None,
        execution_epoch: None,
        side: limits.side,
        rgba8,
    })
}

/// UNQUALIFIED, UNMOUNTED: role/colour-space and upload-refusal review is still
/// required for the existing SDK loader. This helper is not a product capability.
/// Existing HMC loader owns binary geometry/material/image admission. The host
/// supplies reviewed source→package association; an equal revision alone does
/// not establish derivation, distribution rights or human acceptance.
pub(crate) fn render_hmc_unqualified(
    source: SourceRef,
    hmc: &[u8],
    asset_key: &str,
    camera: PreviewCamera,
    side: u32,
    texture_budget: qualia_core_db::render::texture_stream_policy::TextureStreamBudget,
) -> Result<PreviewFrame, String> {
    source.check_current()?;
    validate_frame(side, camera)?;
    if camera.target != [0.0; 3] {
        return Err("HMC SDK camera target must be origin for this adapter".into());
    }
    let mut renderer = webizen_render::VolumetricRenderer::new_offscreen(side, side, 1)?;
    let (_, admission) =
        renderer.load_hmc_asset_with_texture_budget(hmc, asset_key, texture_budget)?;
    if admission.deferred_interpretations != 0
        || admission.capability_refusals != 0
        || admission.resident_interpretations != admission.requested_interpretations
    {
        return Err("HMC material textures were deferred/refused; preview withheld".into());
    }
    renderer.set_camera(camera.yaw, camera.pitch, camera.distance);
    renderer.render(0.0, &webizen_render::SystemTelemetry::default())?;
    let mut rgba8 = vec![0; renderer.required_rgba8_bytes()];
    renderer.read_rgba8_into(&mut rgba8)?;
    source.check_current()?;
    Ok(PreviewFrame {
        source,
        session_revision: None,
        execution_epoch: None,
        side,
        rgba8,
    })
}

/// Native Aether data-uri provider decodes this image; no browser/JS is needed.
pub fn native_image_uri(frame: &PreviewFrame) -> Result<String, String> {
    use base64::Engine;
    frame.source.check_current()?;
    if frame
        .execution_epoch
        .as_ref()
        .is_some_and(|epoch| !epoch.is_current())
    {
        return Err("preview execution epoch has retired".into());
    }
    if !(8..=1024).contains(&frame.side) {
        return Err("invalid native preview extent".into());
    }
    let expected = frame.side as usize * frame.side as usize * 4;
    if frame.rgba8.len() != expected {
        return Err("invalid native preview frame".into());
    }
    let mut png = vec![0; expected + frame.side as usize + 4096];
    let written = qualia_core_db::render::surface_authoring::encode_surface_png_into(
        frame.side,
        &frame.rgba8,
        &mut png,
    )
    .map_err(|e| format!("native preview PNG: {e:?}"))?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&png[..written])
    ))
}

#[cfg(test)]
mod tests {
    use super::super::geometry_host::{GeometryHost, ORGANIC_SURFACE_SOURCE};
    use super::*;
    use vibe::session::{SessionLimits, SourceSession};

    #[test]
    fn actual_extrusion_session_result_uses_nested_mesh_rows() {
        let grants = vec![
            "capability.invoke".into(),
            "ComputationalGeometry.extrude_profile".into(),
        ];
        let mut session = SourceSession::new(grants.clone(), SessionLimits::default());
        let mut host = GeometryHost::new(grants, 200_000);
        let result = session
            .submit(ORGANIC_SURFACE_SOURCE, &mut host, vibe::Budget::default())
            .unwrap();
        let mesh = mesh_from_invoke(&result.value, PreviewLimits::default()).unwrap();
        assert!(!mesh.positions.is_empty());
        assert!(!mesh.triangles.is_empty());
        assert!(mesh.positions.iter().flatten().all(|v| v.is_finite()));
    }

    #[test]
    fn malformed_result_rows_and_provider_frame_extent_fail_closed() {
        assert!(stream_values(&[Value::List(vec![Value::F64(0.0)])], 9).is_err());
        assert!(
            stream_values(&[Value::List(vec![Value::F64(0.0); 3]), Value::F64(1.0)], 9).is_err()
        );
        let id = format!("urn:poet:frame-extent-test:{}", std::process::id());
        let (_, stamp) = studio_store::create_artifact_if_absent(
            studio_store::StudioArtifact::new(&id, "Geometry", "frame extent"),
        )
        .unwrap();
        let frame = PreviewFrame {
            source: SourceRef {
                artifact_id: id,
                stamp,
            },
            session_revision: None,
            execution_epoch: None,
            side: u32::MAX,
            rgba8: Vec::new(),
        };
        assert!(native_image_uri(&frame).unwrap_err().contains("extent"));
    }
}
