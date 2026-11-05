//! Actual native PortalGpu 16:9 (1280x720) views of the decoded, sealed .10d derivative.
use crate::*;
use poet_cli::graphical::{
    assembly::{AssemblyRenderSettings, CompiledAssembly, CompiledPart},
    authoring::AuthoringView,
    controller::EditorHost,
    preview::{PreviewCamera, SourceRef},
};
use qualia_core_db::{
    container_10d::{self, MaterialRecord, SubmeshRange},
    render::{
        compile_10d::{decode_10d_mesh, decode_10d_mesh_frames},
        gpu::PortalGpu,
        lighting::environment_lighting::{
            EnvironmentProbe, EnvironmentProbeSet, ENVIRONMENT_SPECULAR_LEVELS,
        },
        telemetry::SystemTelemetry,
    },
};
use serde_json::json;

pub fn decode(bytes: &[u8], view: &AuthoringView) -> Result<CompiledAssembly, String> {
    let mesh = decode_10d_mesh(bytes).map_err(|e| format!("native mesh decode: {e:?}"))?;
    if mesh.positions.len() > MAX_VERTICES || mesh.triangles.len() > MAX_INDICES / 3 {
        return Err("decoded geometry budget exceeded".into());
    }
    let frames = decode_10d_mesh_frames(bytes, mesh.positions.len())
        .map_err(|e| format!("native frames: {e:?}"))?
        .ok_or("compiled authored frames absent")?;
    let normals = frames.normals
        .ok_or("compiled authored normal stream absent")?;
    let tangents = frames.tangents;
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
        tangents,
        uv0,
    };
    assembly::validate_compiled(&scene, limits())?;
    Ok(scene)
}

pub fn render_16_9_png(
    source: SourceRef,
    scene: &CompiledAssembly,
    settings: AssemblyRenderSettings,
    probes: Option<&EnvironmentProbeSet>,
) -> Result<(Vec<u8>, Vec<u8>), String> {
    source.check_current()?;
    assembly::validate_compiled(scene, limits())?;
    let s = settings;
    let indices: Vec<u32> = scene.mesh.triangles.iter().flatten().copied().collect();
    let colors = vec![[1.0; 4]; scene.mesh.positions.len()];
    let mut renderer = PortalGpu::new_offscreen(RENDER_WIDTH, RENDER_HEIGHT, 1)?;
    renderer.set_ambient_enabled(false);
    renderer.set_tensor_projection_enabled(false);
    renderer.set_clear_color(
        s.clear_color[0],
        s.clear_color[1],
        s.clear_color[2],
        s.clear_color[3],
    );
    renderer.set_camera_target(
        s.camera.yaw,
        s.camera.pitch,
        s.camera.distance,
        s.camera.target[0],
        s.camera.target[1],
        s.camera.target[2],
    );
    renderer.set_lighting(
        s.sun_direction[0],
        s.sun_direction[1],
        s.sun_direction[2],
        s.sun_intensity,
        s.ambient_intensity,
    );
    renderer.set_shadows_enabled(s.shadows);
    renderer.set_screen_space_ao_enabled(s.screen_space_ao);
    renderer.upload_mesh_colored_with_frames_and_materials_and_uv0(
        &scene.mesh.positions,
        &colors,
        Some(&scene.normals),
        scene.tangents.as_deref(),
        Some(&scene.uv0),
        &indices,
        &scene.materials,
        &scene.ranges,
    )?;
    renderer.set_environment_probes(probes, [0.0; 3]);
    if !renderer.set_exposure_compensation(s.exposure_ev) {
        return Err("native output exposure capability unavailable; image withheld".into());
    }
    renderer.render(0.0, &SystemTelemetry::default())?;
    let mut rgba8 = vec![0; renderer.required_rgba8_bytes()];
    renderer.read_rgba8_into(&mut rgba8)?;
    source.check_current()?;
    let png = encode_png_rgba8(RENDER_WIDTH, RENDER_HEIGHT, &rgba8)?;
    Ok((rgba8, png))
}

fn crc32_ieee(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in bytes {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(bytes: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in bytes {
        a = (a + u32::from(byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}

fn push_png_chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(tag);
    out.extend_from_slice(data);
    let crc = crc32_ieee(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

pub fn encode_png_rgba8(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    let row_bytes = width as usize * 4;
    if rgba.len() != row_bytes * height as usize {
        return Err("RGBA8 buffer size mismatch for PNG encode".into());
    }
    let mut raw = Vec::with_capacity((row_bytes + 1) * height as usize);
    for row in rgba.chunks_exact(row_bytes) {
        raw.push(0); // filter type 0 (None)
        raw.extend_from_slice(row);
    }
    let mut zlib = Vec::with_capacity(raw.len() + (raw.len() / 65_535 + 1) * 5 + 6);
    zlib.extend_from_slice(&[0x78, 0x01]);
    let chunks: Vec<&[u8]> = raw.chunks(65_535).collect();
    for (i, block) in chunks.iter().enumerate() {
        let last = u8::from(i + 1 == chunks.len());
        let len = block.len() as u16;
        let nlen = !len;
        zlib.push(last);
        zlib.extend_from_slice(&len.to_le_bytes());
        zlib.extend_from_slice(&nlen.to_le_bytes());
        zlib.extend_from_slice(block);
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = Vec::with_capacity(zlib.len() + 64);
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    push_png_chunk(&mut png, b"IHDR", &ihdr);
    push_png_chunk(&mut png, b"IDAT", &zlib);
    push_png_chunk(&mut png, b"IEND", &[]);
    Ok(png)
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
        let (rgba8, png) = render_16_9_png(
            snapshot.source.clone(),
            scene,
            settings,
            Some(&probes),
        )?;
        if !snapshot.publication_is_current() || !snapshot.execution_epoch.is_current() {
            return Err("render publication retired".into());
        }
        let filename = format!("{name}.png");
        files::write(&root.join("images").join(&filename), &png)?;
        receipts.push(json!({"file":format!("images/{filename}"),"sha256":files::hash(&png),
            "rgba_sha256":files::hash(&rgba8),"width":RENDER_WIDTH,"height":RENDER_HEIGHT,"aspect_ratio":"16:9",
            "camera":{"yaw":camera.yaw,"pitch":camera.pitch,"distance":camera.distance,"target":camera.target},
            "native_pipeline":"PortalGpu offscreen 1280x720 (16:9)/material ranges/authored normals/UV0/native PNG",
            "sun":{"direction":settings.sun_direction,"intensity":settings.sun_intensity},
            "ambient":settings.ambient_intensity,"exposure_ev":settings.exposure_ev,"shadows":settings.shadows,
            "screen_space_ao":settings.screen_space_ao,"floor_geometry":"none","acceptance":"unassessed"}));
        println!("Rendered {}", root.join("images").join(filename).display());
    }
    Ok(receipts)
}
