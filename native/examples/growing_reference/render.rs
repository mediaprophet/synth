//! Actual native PortalGpu views of the decoded, sealed .10d derivative.
use crate::*;
use poet_cli::graphical::{
    assembly::{AssemblyRenderSettings, CompiledAssembly, CompiledPart},
    authoring::AuthoringView,
    controller::EditorHost,
    preview::PreviewCamera,
};
use qualia_core_db::{
    container_10d::{self, MaterialRecord, SubmeshRange},
    render::{
        compile_10d::{decode_10d_mesh, decode_10d_mesh_frames},
        lighting::environment_lighting::{
            EnvironmentProbe, EnvironmentProbeSet, ENVIRONMENT_SPECULAR_LEVELS,
        },
        surface_authoring::encode_surface_png_into,
    },
};
use serde_json::json;

pub fn decode(bytes: &[u8], view: &AuthoringView) -> Result<CompiledAssembly, String> {
    // Input bytes are already bounded by canonical checkpoint limits and bound
    // to the source-reconstructed exact .10d before this native decode.
    let mesh = decode_10d_mesh(bytes).map_err(|e| format!("native mesh decode: {e:?}"))?;
    if mesh.positions.len() > MAX_VERTICES || mesh.triangles.len() > MAX_INDICES / 3 {
        return Err("decoded geometry budget exceeded".into());
    }
    let normals = decode_10d_mesh_frames(bytes, mesh.positions.len())
        .map_err(|e| format!("native frames: {e:?}"))?
        .and_then(|f| f.normals)
        .ok_or("compiled authored normal stream absent")?;
    let (materials_count, ranges_count) = container_10d::material_container_counts(bytes)
        .map_err(|e| format!("native materials: {e:?}"))?;
    if ranges_count != view.parts.len() || ranges_count > 256 || materials_count > 256 {
        return Err("decoded part/material count mismatch".into());
    }
    let mut materials = vec![MaterialRecord::legacy_default(); materials_count];
    let mut ranges = vec![
        SubmeshRange {
            first_index: 0,
            index_count: 0,
            material_id: 0,
            semantic_id: 0
        };
        ranges_count
    ];
    let read = container_10d::read_material_section_into(
        bytes,
        (mesh.triangles.len() * 3) as u32,
        &mut materials,
        &mut ranges,
    )
    .map_err(|e| format!("native material ranges: {e:?}"))?;
    if read != (materials_count, ranges_count) {
        return Err("partial material/range decode".into());
    }
    let mut uv0 = vec![[0.0; 2]; mesh.positions.len()];
    let uv_count =
        container_10d::read_texture_coordinate_section_into(bytes, mesh.positions.len(), &mut uv0)
            .map_err(|e| format!("native UV decode: {e:?}"))?;
    if uv_count != mesh.positions.len() {
        return Err("partial UV decode".into());
    }
    let parts = view
        .parts
        .iter()
        .map(|part| {
            let matches = ranges
                .iter()
                .filter(|r| r.semantic_id == part.semantic_id && r.material_id == part.material_id)
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(format!("stable range missing/ambiguous for {}", part.id));
            }
            let range = matches[0];
            Ok(CompiledPart {
                key: part.id.clone(),
                parent: part.parent.clone(),
                semantic_id: part.semantic_id,
                first_index: range.first_index,
                index_count: range.index_count,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let scene = CompiledAssembly {
        mesh,
        materials,
        ranges,
        parts,
        normals,
        uv0,
    };
    assembly::validate_compiled(&scene, limits())?;
    Ok(scene)
}

pub fn images(
    root: &Path,
    scene: &CompiledAssembly,
    host: &dyn EditorHost,
) -> Result<Vec<serde_json::Value>, String> {
    let cameras = [
        (
            "hero",
            PreviewCamera {
                yaw: 0.65,
                pitch: 0.32,
                distance: 4.5,
                target: [0.0, 0.88, 0.0],
            },
        ),
        (
            "opposite",
            PreviewCamera {
                yaw: 0.65 + std::f32::consts::PI,
                pitch: 0.32,
                distance: 4.5,
                target: [0.0, 0.88, 0.0],
            },
        ),
        (
            "channel-closeup",
            PreviewCamera {
                yaw: 0.45,
                pitch: 0.62,
                distance: 1.65,
                target: [0.0, 1.46, -0.14],
            },
        ),
        (
            "cabinet-closeup",
            PreviewCamera {
                yaw: 0.35,
                pitch: 0.16,
                distance: 1.40,
                target: [0.0, 0.36, -0.30],
            },
        ),
    ];
    let mut probes = EnvironmentProbeSet::new();
    if !probes.push(EnvironmentProbe {
        position: [0.0, 0.8, 0.0],
        radius: 100.0,
        irradiance: [0.42, 0.45, 0.48],
        specular_levels: [[0.55, 0.58, 0.62]; ENVIRONMENT_SPECULAR_LEVELS],
        dominant_direction: [0.4, 0.8, 0.3],
        valid: true,
    }) {
        return Err("synthetic probe rejected".into());
    }
    let mut receipts = Vec::new();
    for (name, camera) in cameras {
        let snapshot = host.snapshot()?;
        if !snapshot.publication_is_current() {
            return Err("render origin unavailable".into());
        }
        let mut settings = AssemblyRenderSettings::for_camera(camera);
        settings.clear_color = [0.14, 0.17, 0.19, 1.0];
        settings.sun_direction = [0.45, 0.82, 0.35];
        settings.sun_intensity = 2.2;
        settings.ambient_intensity = 0.13;
        settings.exposure_ev = 0.0;
        let mut frame = assembly::render_assembly_with_settings(
            snapshot.source.clone(),
            scene,
            settings,
            limits(),
            Some(&probes),
        )?;
        frame.session_revision = Some(snapshot.session_revision);
        frame.execution_epoch = Some(snapshot.execution_epoch.clone());
        if !snapshot.publication_is_current()
            || frame.source != snapshot.source
            || !frame
                .execution_epoch
                .as_ref()
                .is_some_and(|e| e.is_current())
        {
            return Err("render publication retired".into());
        }
        let mut png = vec![0; frame.rgba8.len() + frame.side as usize + 4096];
        let written = encode_surface_png_into(frame.side, &frame.rgba8, &mut png)
            .map_err(|e| format!("native PNG: {e:?}"))?;
        png.truncate(written);
        let filename = format!("{name}.png");
        files::write(&root.join("images").join(&filename), &png)?;
        receipts.push(json!({"file":format!("images/{filename}"),"sha256":files::hash(&png),
            "rgba_sha256":files::hash(&frame.rgba8),"width":frame.side,"height":frame.side,
            "camera":{"yaw":camera.yaw,"pitch":camera.pitch,"distance":camera.distance,"target":camera.target},
            "native_pipeline":"PortalGpu offscreen/material ranges/authored normals/UV0/native PNG",
            "sun":{"direction":settings.sun_direction,"intensity":settings.sun_intensity},
            "ambient":settings.ambient_intensity,"exposure_ev":settings.exposure_ev,"shadows":settings.shadows,
            "screen_space_ao":settings.screen_space_ao,"floor_geometry":"none","acceptance":"unassessed"}));
        println!("Rendered {}", root.join("images").join(filename).display());
    }
    Ok(receipts)
}
