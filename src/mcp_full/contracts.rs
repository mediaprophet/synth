//! Explicit production output recipes and a bounded native review subprocess.
use super::*;
use std::{io::Read, process::{Command, Stdio}, time::{Duration, Instant}};

pub(super) fn output_recipes() -> Value {
    json!([{
        "id": "rgba_png",
        "resource_key": "image.png",
        "resource_kind": "png",
        "source": "requires [capability(\"capability.invoke\")]; let image = capability.invoke(\"hypermedia.radiance.create_rgba\", {width:8,height:8,color:[51,102,153,255]}); let png = capability.invoke(\"hypermedia.radiance.encode_rgba_png\", {image:image}); let mut asset = {files:[{key:\"image.png\",resource:png}]};"
    }, {
        "id": "sprite_record_png",
        "resource_key": "sheet.png",
        "resource_kind": "png",
        "source": "requires [capability(\"capability.invoke\")]; let tile = capability.invoke(\"hypermedia.radiance.create_rgba\", {width:8,height:8,color:[51,102,153,255]}); let sheet = capability.invoke(\"hypermedia.radiance.pack_sprite_sheet\", {tiles:[tile],columns:1,padding:0}); let mut asset = {files:[{key:\"sheet.png\",resource:sheet.png}]};"
    }, {
        "id": "lossless_sequence",
        "resource_key": "sequence.json",
        "resource_kind": "json",
        "source": "requires [capability(\"capability.invoke\")]; let tile = capability.invoke(\"hypermedia.radiance.create_rgba\", {width:8,height:8,color:[51,102,153,255]}); let sequence = capability.invoke(\"hypermedia.chronicle.capture_deterministic_movie\", {frames:[tile,tile],fps:{numerator:24000,denominator:1001},profile:\"png-sequence\"}); let mut asset = {files:[{key:\"sequence.json\",resource:sequence.manifest_resource}]};"
    }, {
        "id": "emf",
        "resource_key": "field.emf",
        "resource_kind": "emf",
        "container_magic": "Q42EMF64",
        "container_version": 4,
        "source": "requires [capability(\"capability.invoke\")]; let emf = capability.invoke(\"hypermedia.electromagnetics.compile_emf\", {origin:[0.0,0.0,0.0],voxel_size:0.5,grid_dims:[1,1,1],epoch_s:1700000000.0,sources:[{position:[0.0,0.0,0.0],e_amplitude:[150.0,0.0,0.0],wavelength_nm:550.0,phase:0.0}]}); let mut asset = {files:[{key:\"field.emf\",resource:emf}]};"
    }, {
        "id": "qac",
        "resource_key": "signal.qac",
        "resource_kind": "qac",
        "container_magic": "qac\0\0\0\0\0",
        "container_version": 1,
        "source": "requires [capability(\"capability.invoke\")]; let tone = capability.invoke(\"hypermedia.acoustics.synthesize_tone\", {frequency:440.0,sample_rate:48000,frames:256,amplitude:0.3}); let qac = capability.invoke(\"hypermedia.acoustics.encode_aac\", {audio:tone,format:\"qac\"}); let mut asset = {files:[{key:\"signal.qac\",resource:qac}]};"
    }])
}

pub(super) fn catalogue() -> Value {
    let operations: Vec<Value> = crate::production::assets::ontology::MORPHISMS.iter().map(|operation| {
        let (kind, fields, encoder) = match operation.id {
            "hypermedia.manifold.compile_10d" => ("encoded_resource", json!([]), Value::Null),
            "hypermedia.manifold.compile_10d_pbr" => ("encoded_resource", json!([{"field":"textures","type":"array","items":{"digest":"sha256","mime_type":"image MIME","resource":"encoded_resource"},"packaging":"automatically retained by files/v2 as textures/sha256/<digest>"}]), Value::Null),
            "hypermedia.manifold.sweep_assembly_profile" => ("record", json!([{"field":"container_10d","type":"encoded_resource","kind":"10d"}]), Value::Null),
            "hypermedia.surface.unwrap_slim" => ("record", json!([{"field":"container_10d","type":"encoded_resource","kind":"10d"},{"field":"mesh","type":"mesh_handle"},{"field":"source_vertices","type":"array","when":"mode=atlas"},{"field":"uvs","type":"array"}]), Value::Null),
            "hypermedia.chronicle.capture_deterministic_movie" => ("profile_dependent", json!([{"field":"manifest_resource","type":"encoded_resource","kind":"json","when":"profile=png-sequence","packaging":"leaf PNG/WAV dependencies automatically retained by files/v2"},{"field":"mp4","type":"encoded_resource","when":"profile=preview"}]), Value::Null),
            "hypermedia.surface.pack_pbr_set" => ("record", json!([{"field":"colour","type":"encoded_resource","kind":"png"},{"field":"normal","type":"encoded_resource","kind":"png"},{"field":"orm","type":"encoded_resource","kind":"png"}]), Value::Null),
            "hypermedia.surface.synthesize_maps" => ("record", json!([{"field":"colour","type":"rgba_handle"},{"field":"normal","type":"rgba_handle"},{"field":"orm","type":"rgba_handle"}]), json!({"operation":"hypermedia.surface.pack_pbr_set","arguments":{"colour":"$result.colour","normal":"$result.normal","orm":"$result.orm"}})),
            "hypermedia.radiance.pack_sprite_sheet" => ("record", json!([{"field":"png","type":"encoded_resource","kind":"png"},{"field":"image","type":"rgba_handle"},{"field":"frames","type":"array"}]), Value::Null),
            "hypermedia.surface.bake_pbr_materialx" => ("record", json!([{"field":"materialx_resource","type":"encoded_resource"},{"field":"psd_resource","type":"encoded_resource"},{"field":"ao_image","type":"rgba_handle"},{"field":"normal_image","type":"rgba_handle"}]), Value::Null),
            "hypermedia.radiance.create_rgba" | "hypermedia.layout.render_ui_sheet" => ("rgba_handle", json!([]), json!({"operation":"hypermedia.radiance.encode_rgba_png","arguments":{"image":"$result"}})),
            "hypermedia.radiance.encode_rgba_png" | "hypermedia.radiance.encode_rgb_png" | "hypermedia.locus.project_svg" | "hypermedia.acoustics.encode_wav" | "hypermedia.acoustics.encode_aac" | "hypermedia.electromagnetics.compile_emf" => ("encoded_resource", json!([]), Value::Null),
            _ => ("operation_specific", json!([]), Value::Null),
        };
        let attribute_edit = matches!(operation.id,
            "hypermedia.manifold.boolean_3d" | "hypermedia.manifold.simplify" | "hypermedia.manifold.radial_euler"
            | "hypermedia.manifold.bevel" | "hypermedia.manifold.inset" | "hypermedia.manifold.solidify"
            | "hypermedia.manifold.bisect_plane" | "hypermedia.manifold.subdivide_catmull_clark"
            | "hypermedia.manifold.evaluate_field" | "hypermedia.manifold.realize_instances" | "hypermedia.manifold.mesh_to_sdf_brick"
            | "hypermedia.manifold.sweep_assembly_profile" | "hypermedia.surface.unwrap_slim");
        let extensions = if attribute_edit {json!({"attribute_policy":{"type":"string","enum":["preserve","discard"],"default":"preserve",
            "semantics":"Preserve refuses geometry-only edits of attributed meshes unless a supported remap transports them. Discard explicitly creates a geometry revision requiring surface rebind/bake; source remains intact.",
            "unwrap_exception":"Untextured slim/atlas preserves the supported remapped mesh; atlas exposes source_vertices. Textured rebaking is required."}})} else {json!({})};
        json!({"id": operation.id, "execution_host":"synth.production", "domain": operation.domain,
            "codomain": operation.codomain, "arguments": operation.arguments,
            "input_extensions": extensions,
            "output":{"type":kind,"resource_fields":fields,"encoder":encoder},
            "handle_lifetime":"one source evaluation", "metadata_provider_call":"schema hash only; does not execute production"})
    }).collect();
    let recipes = output_recipes();
    json!({"schema":"synth.operation-contracts/v1", "count":operations.len(), "operations": operations,
        "fallback":"operation_specific means consult the codomain and implementation contract; no namespace-based export inference is permitted",
        "production_profiles":[{"id":"preview","resource_mib":16,"aggregate_mib":64,"package_mib":80,"logical_work_units":50000000},{"id":"production16","resource_mib":64,"aggregate_mib":256,"package_mib":320,"logical_work_units":500000000,"raster_max_dimension":2048,"precision":"RGBA16 UNORM with sRGB/linear PNG metadata"},{"id":"png-sequence","operation":"hypermedia.chronicle.capture_deterministic_movie","frames":64,"raster_max_dimension":2048,"precision":"congruent RGBA8 or RGBA16 frames","timing":"exact rational manifest, optional sample-aligned PCM WAV"}],
        "resource_type":{"type":"encoded_resource", "export_binding":{"key":"output.ext","resource":"$encoded"}},
        "output_recipes": recipes,
        "recipes": recipes})
}

fn preview_command(executable: &Path, asset: &Path, destination: &Path, package: &Path, asset_key: &str) -> Command {
    let mut command = Command::new(executable);
    command.arg("preview").arg("--asset").arg(asset).arg("--out").arg(destination)
        .arg("--hmc").arg(package).arg("--asset-key").arg(asset_key);
    command
}

pub(super) fn native_preview(asset: &Path, destination: &Path, package: &Path, asset_key: &str) -> Result<(), ToolError> {
    let executable = std::env::current_exe().map_err(|_| ToolError::new("preview_failed", "Cannot locate Synth executable"))?;
    let mut child = preview_command(&executable,asset,destination,package,asset_key)
        .stdout(Stdio::null()).stderr(Stdio::piped()).spawn()
        .map_err(|e| ToolError::new("preview_failed", format!("Cannot start native review: {e}")))?;
    let stderr = child.stderr.take().unwrap();
    // Drain stderr even after the diagnostic cap to avoid blocking a renderer subprocess.
    let reader = std::thread::spawn(move || {
        let mut retained = Vec::new();
        let mut stream = stderr;
        let mut buffer = [0u8; 4096];
        while let Ok(count) = stream.read(&mut buffer) {
            if count == 0 { break; }
            let keep = count.min(2048usize.saturating_sub(retained.len()));
            retained.extend_from_slice(&buffer[..keep]);
        }
        String::from_utf8_lossy(&retained).into_owned()
    });
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let diagnostic = reader.join().unwrap_or_default();
                return if status.success() { Ok(()) } else {
                    Err(ToolError::new("preview_failed", format!("Native review exited {status}; {diagnostic}")))
                };
            }
            Ok(None) if start.elapsed() < Duration::from_secs(180) => std::thread::sleep(Duration::from_millis(50)),
            outcome => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                return Err(ToolError::new("preview_failed", if outcome.is_err() { "Native review process could not be observed" } else { "Native review exceeded the 180 second time budget; completed exports remain available" }));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_preview_forwards_canonical_package_and_asset_key_without_shell() {
        let command = preview_command(Path::new("synth.exe"),Path::new("export/mesh.10d"),Path::new("review"),Path::new("asset.hmc"),"outputs/mesh.10d");
        let args:Vec<_> = command.get_args().map(|v|v.to_string_lossy().into_owned()).collect();
        assert_eq!(args, ["preview","--asset","export/mesh.10d","--out","review","--hmc","asset.hmc","--asset-key","outputs/mesh.10d"]);
    }

    #[test]
    fn output_recipes_declare_emf_v4_and_qac_v1_formats() {
        let recipes = output_recipes();
        let list = recipes.as_array().expect("output_recipes returns array");
        let emf = list.iter().find(|r| r["id"] == "emf").expect("emf recipe");
        assert_eq!(emf["resource_key"], "field.emf");
        assert_eq!(emf["resource_kind"], "emf");
        assert_eq!(emf["container_magic"], "Q42EMF64");
        assert_eq!(emf["container_version"], 4);

        let qac = list.iter().find(|r| r["id"] == "qac").expect("qac recipe");
        assert_eq!(qac["resource_key"], "signal.qac");
        assert_eq!(qac["resource_kind"], "qac");
        assert_eq!(qac["container_magic"], "qac\0\0\0\0\0");
        assert_eq!(qac["container_version"], 1);

        let cat = catalogue();
        let compile_emf = cat["operations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|op| op["id"] == "hypermedia.electromagnetics.compile_emf")
            .unwrap();
        assert_eq!(compile_emf["output"]["type"], "encoded_resource");

        let encode_aac = cat["operations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|op| op["id"] == "hypermedia.acoustics.encode_aac")
            .unwrap();
        assert_eq!(encode_aac["output"]["type"], "encoded_resource");
        assert_eq!(cat["output_recipes"], recipes);
        assert_eq!(cat["recipes"], recipes);
    }
}
