//! Botanical species reference embodiment (`Corymbia aparrerinja` Ghost Gum)
//! through Poet's native project/session host, space colonization, pipe-model radii,
//! SDF junction blending, smooth DDG vertex normals, UV atlas, .hmc/.10d/.q42/.m64 containers,
//! and PortalGpu 1280x720 (16:9) 4-view PNGs.
use crate::*;
use poet_cli::graphical::{
    authoring::{
        botanical::{BotanicalLimits, BotanicalRecipe, JunctionSource, Organ},
        AuthoringEdit,
    },
    checkpoints::{
        ui_georaster_edit_limits, ui_inspection_limits, GeoRasterCreateRequest,
        ResourceInspectionRequest, SourceDeclaration, SourceStatus, UI_INSPECTION_CHUNKS,
    },
    controller::{EditorHost, RebuildLimits},
    cooperative_workflow::WorkflowAction,
    native_project::NativeProject,
    preview::PreviewCamera,
    session_host::SessionEditorHost,
};
use qualia_core_db::{
    poet_host::invoke::studio_store::checkpoint::{
        self, ExpectedFile, ResourceData, ResourceEdit,
    },
    q42::{m64_matter::M64PayloadInterval, q42_volume::WasmSliceQ42Reader},
    render::lighting::environment_lighting::{
        EnvironmentProbe, EnvironmentProbeSet, ENVIRONMENT_SPECULAR_LEVELS,
    },
};
use serde_json::json;
use std::sync::{atomic::AtomicBool, Arc, Mutex};

const BOTANICAL_HMC: &str = "corymbia-aparrerinja.hmc";
const BOTANICAL_ASSET: &str = "corymbia-aparrerinja.10d";
const BOTANICAL_Q42_KEY: &str = "graphs/corymbia-aparrerinja.10d.q42";
const BOTANICAL_Q42_ASSET: &str = "corymbia-aparrerinja.q42";
const BOTANICAL_M64_ASSET: &str = "corymbia-aparrerinja.m64";
const BOTANICAL_MASTER_KEY: &str = "masters/corymbia-aparrerinja/master.vibe";
const BOTANICAL_ACCEPTED_KEY: &str = "masters/corymbia-aparrerinja/accepted-source.vibe";

fn rebuild_limits() -> RebuildLimits {
    RebuildLimits {
        cells: 16,
        source_bytes: 1024 * 1024,
        total_steps: 2_000_000,
        total_geometry_work: 72_000_000,
        total_kernel_dispatches: 1024,
    }
}

fn master_source(profile: &str) -> Result<String, String> {
    let (lat, lon, bark_color, foliage_color, rough) = match profile {
        "film" => (18, 24, "[0.86, 0.83, 0.78, 1.0]", "[0.23, 0.42, 0.21, 1.0]", 0.68),
        "open_world_game" => (12, 16, "[0.84, 0.80, 0.75, 1.0]", "[0.25, 0.45, 0.22, 1.0]", 0.72),
        "cartoon" => (10, 14, "[0.92, 0.88, 0.82, 1.0]", "[0.30, 0.62, 0.28, 1.0]", 0.84),
        _ => return Err("unknown authored profile".into()),
    };
    Ok(format!(
        r#"requires [capability("capability.invoke"), capability("ComputationalGeometry.profile_sweep"), capability("ComputationalGeometry.uv_sphere")];

pure fn cg_surface(id:i64, color:List, metal:f64, rough:f64) -> Record {{
    return {{id:id, colour_space:"linear", base_color:color, emissive:[0.0,0.0,0.0], metallic:metal, roughness:rough, specular:1.0, alpha_cutoff:0.5, opacity_mode:"Opaque", double_sided:false}};
}}

let bark = cg_surface(8001, {bark_color}, 0.0, {rough});
let foliage = cg_surface(8002, {foliage_color}, 0.0, {rough});
let seed_trunk = capability.invoke("ComputationalGeometry.profile_sweep", {{
    profile: {{kind: "circle", radius: 0.16, segments: 12}},
    axis: [[0.0, 0.0, -1.0], [0.0, 0.0, 0.0], [0.4, 0.0, 1.0]],
    caps: true
}});
let crown = capability.invoke("ComputationalGeometry.uv_sphere", {{
    radius: 0.62,
    lat_segments: {lat},
    lon_segments: {lon}
}});

let mut poet_parts = [
    {{id: "branch", semantic_id: 81001, parent: null, geometry: seed_trunk, material: bark, translation: [0.0, 1.0, 0.0], rotation: [-1.5707963267948966, 0.0, 0.0], scale: [1.0, 1.0, 1.0]}},
    {{id: "canopy/north", semantic_id: 81002, parent: "branch", geometry: crown, material: foliage, translation: [0.95, 0.12, 1.18], rotation: [0.0, 0.0, 0.0], scale: [1.18, 1.12, 0.78]}},
    {{id: "canopy/south", semantic_id: 81003, parent: "branch", geometry: crown, material: foliage, translation: [-0.92, -0.08, 1.14], rotation: [0.0, 0.0, 0.0], scale: [1.12, 1.06, 0.74]}},
    {{id: "canopy/apex", semantic_id: 81004, parent: "branch", geometry: crown, material: foliage, translation: [0.04, 0.22, 1.42], rotation: [0.0, 0.0, 0.0], scale: [1.28, 1.22, 0.82]}}
];
({{parts: poet_parts}});"#
    ))
}

fn junction_recipe() -> BotanicalRecipe {
    BotanicalRecipe {
        version: 1,
        part_id: "branch".into(),
        organs: vec![
            Organ {
                key: "root".into(),
                parent: None,
                position_m: [0.0, 0.0, -1.0],
            },
            Organ {
                key: "center".into(),
                parent: Some("root".into()),
                position_m: [0.0, 0.0, 0.0],
            },
            Organ {
                key: "a".into(),
                parent: Some("center".into()),
                position_m: [1.0, 0.0, 1.0],
            },
            Organ {
                key: "b".into(),
                parent: Some("center".into()),
                position_m: [-1.0, 0.0, 1.0],
            },
        ],
        attractors_m: vec![[0.0, 1.0, 1.0]],
        node_length_m: 0.2,
        kill_radius_m: 0.05,
        perception_radius_m: 0.8,
        cone_angle_rad: 2.0,
        max_iterations: 1,
        tip_radius_m: 0.35,
        pipe_exponent: 2.0,
        ring_slices: 8,
    }
}

pub fn produce_botanical(root: &Path, profile: &str) -> Result<(), String> {
    let started = std::time::Instant::now();
    let master_text = master_source(profile)?;
    let master = master_text.as_bytes().to_vec();
    for name in ["authority", "hmc", "images"] {
        files::create_directory(&root.join(name))?;
    }
    files::write(&root.join("master.vibe"), &master)?;
    let (scope, authority) = authority::create(&root.join("authority"))?;
    let session = authority::login(&authority, false)?;
    let project = Arc::new(NativeProject::open_with_rebuild_limits(
        config(scope.clone(), root),
        authority.clone(),
        session.clone(),
        None,
        Box::new(ValidateOnly),
        rebuild_limits(),
    )?);
    let mut host = SessionEditorHost::new(project, Arc::new(Mutex::new(Some(session))));
    let s0 = host.snapshot()?.source;
    host.submit(&master_text, s0)
        .map_err(|e| format!("botanical master submission: {e}"))?;

    // 1. Adopt space-colonized + pipe-model + SDF-blended botanical junction on "branch"
    let recipe = junction_recipe();
    let j_source = JunctionSource::from_recipe(
        &recipe,
        [-2.03, -2.02, -2.04],
        0.5,
        [9, 9, 9],
        0.2,
        1e-6,
        1e-8,
        0.99,
        1e-20,
        BotanicalLimits::default(),
        4096,
    )?;
    let j_val = j_source.to_value(BotanicalLimits::default(), 4096)?;
    host.authoring_edit(
        AuthoringEdit::BotanicalJunctionAdopt { source: j_val },
        host.snapshot()?.source,
    )
    .map_err(|e| format!("junction adoption: {e}"))?;

    // 2. Edit organ "a" position on the adopted junction
    host.authoring_edit(
        AuthoringEdit::BotanicalJunctionPosition {
            target: "branch".into(),
            organ: "a".into(),
            position_m: [1.1, 0.0, 1.0],
        },
        host.snapshot()?.source,
    )
    .map_err(|e| format!("junction position edit: {e}"))?;

    // 3. Refine bark material roughness
    let after_j = host.authoring_view(host.snapshot()?.source)?;
    let bark_part = after_j
        .parts
        .iter()
        .find(|p| p.material_id == 8001)
        .ok_or("missing bark material 8001")?;
    host.authoring_edit(
        AuthoringEdit::Material {
            material_id: 8001,
            base_color: bark_part.base_color,
            roughness: (bark_part.roughness + 0.02).min(1.0),
            metallic: bark_part.metallic,
        },
        host.snapshot()?.source,
    )?;

    let current = host.snapshot()?;
    let view = host.authoring_view(current.source.clone())?;
    let cells = host.source_cells(current.source.clone())?;
    let accepted = cells.join("\n\n").into_bytes();
    files::write(&root.join("accepted-source.vibe"), &accepted)?;
    files::json(&root.join("parts.json"), &produce::view_json(&view))?;
    let saved = host.save_checkpoint(BOTANICAL_HMC, ExpectedFile::Absent, current.source.clone())?;
    let compiled = host.compile_checkpoint(
        BOTANICAL_HMC,
        BOTANICAL_ASSET,
        ExpectedFile::Sha256(saved.digest),
        current.source.clone(),
    )?;
    let lineage = json!({
        "schema": "poet-reference-lineage/1",
        "synthetic_concept": true,
        "fixture_only": true,
        "dossier": "corymbia-aparrerinja-ghost-gum-v0.1",
        "entity": "ECO-SPECIES:CORYMBIA-APARRERINJA-01",
        "catalogue": "ecopunk-model-deliverables",
        "catalogue_version": "2026-10-24.2",
        "profile": profile,
        "style_lod_target_distinct": true,
        "master_sha256": files::hash(&master),
        "accepted_source_sha256": files::hash(&accepted),
        "operations": [
            "BotanicalJunctionAdopt on branch (space_colonize_3d + pipe_model_radii + SDF junction blend)",
            "BotanicalJunctionPosition organ a -> [0.95, 0.12, 1.05]",
            "Material 8001 roughness +0.02"
        ],
        "containers": [
            "corymbia-aparrerinja.hmc",
            "corymbia-aparrerinja.10d",
            "corymbia-aparrerinja.q42",
            "corymbia-aparrerinja.m64"
        ],
        "film": "incomplete/unaccepted",
        "open_world_game": "incomplete/unaccepted",
        "cartoon": "incomplete/unaccepted"
    });
    let resources = [
        (BOTANICAL_MASTER_KEY, "text/plain", master.clone()),
        (BOTANICAL_ACCEPTED_KEY, "text/plain", accepted.clone()),
        (
            "masters/corymbia-aparrerinja/lineage.json",
            "application/json",
            serde_json::to_vec_pretty(&lineage).map_err(|e| e.to_string())?,
        ),
    ];
    let package = host.edit_package_resources(
        BOTANICAL_HMC,
        ExpectedFile::Sha256(compiled.digest),
        resources
            .into_iter()
            .map(|(key, kind, bytes)| {
                ResourceEdit::Add(ResourceData {
                    key: key.into(),
                    kind: kind.into(),
                    bytes,
                    meta: None,
                })
            })
            .collect(),
        current.source.clone(),
    )?;
    // Produce and embed the canonical Matter-64 (.m64) v3 habitat terrain container inside HMC.
    host.create_georaster(
        GeoRasterCreateRequest {
            filename: BOTANICAL_HMC.into(),
            package_digest: package.digest,
            key: BOTANICAL_M64_ASSET.into(),
            metadata: produce::site_m64_metadata(-23.6980, 133.8807, 545_000),
            lut_bank_epoch: 1,
            samples_mm: produce::site_m64_samples(545_000),
            source: SourceDeclaration {
                status: SourceStatus::Synthetic,
                description: "Synthetic Matter-64 v3 MacDonnell Ranges alluvial terrace for Corymbia aparrerinja".into(),
                evidence_ref: None,
                observation_claim: false,
            },
        },
        current.source.clone(),
        ui_georaster_edit_limits(),
        &AtomicBool::new(false),
    )?;
    files::json(&root.join("lineage.json"), &lineage)?;
    let before = host.workflow(WorkflowAction::Inspect)?;
    host.workflow(WorkflowAction::Contribution {
        target: before.evidence.target.clone(),
        id: "software-native-botanical-production/1".into(),
        description: "Controlled SoftwareAgent botanical junction adoption, organ editing, and HMC/.10d/.q42/.m64 compilation.".into(),
        minutes: u32::try_from(started.elapsed().as_secs() / 60).unwrap_or(0),
        work_status: qualia_cooperative_core::authoring::WorkStatus::Unknown,
    })?;
    let hmc_file = root.join("hmc").join(BOTANICAL_HMC);
    let hmc = files::read(&hmc_file, 256 * 1024 * 1024)?;
    let detached = checkpoint::load(&hmc_file, CheckpointLimits::default())?;
    let bytes = detached
        .resources()
        .iter()
        .find(|r| r.key == BOTANICAL_ASSET && r.kind == "10d")
        .ok_or("compiled .10d missing from botanical HMC")?
        .bytes
        .clone();
    let q42_bytes = detached
        .resources()
        .iter()
        .find(|r| r.key == BOTANICAL_Q42_KEY && r.kind == "q42")
        .ok_or("compiled .q42 missing from botanical HMC")?
        .bytes
        .clone();
    let m64_bytes = detached
        .resources()
        .iter()
        .find(|r| r.key == BOTANICAL_M64_ASSET && r.kind == "m64")
        .ok_or("compiled .m64 missing from botanical HMC")?
        .bytes
        .clone();
    files::write(&root.join(BOTANICAL_ASSET), &bytes)?;
    files::write(&root.join(BOTANICAL_Q42_ASSET), &q42_bytes)?;
    files::write(&root.join(BOTANICAL_M64_ASSET), &m64_bytes)?;
    let requested = host.workflow(WorkflowAction::RequestReview {
        target: before.evidence.target.clone(),
        id: REQUEST.into(),
        evidence: format!(
            "accepted-source.vibe sha256={}; HMC sha256={}; pending distinct SoftwareAgent technical-source review",
            files::hash(&accepted),
            files::hash(&hmc)
        ),
    })?;
    let info = RunInfo {
        schema: "poet-growing-reference/software-fixture/1".into(),
        fixture_only: true,
        project_id: scope.project_id,
        document_id: scope.document_id,
        source_id: scope.source_id,
        profile: profile.into(),
        master_sha256: files::hash(&master),
        preset_sha256: files::hash(&master),
        accepted_source_sha256: files::hash(&accepted),
        hmc_sha256: files::hash(&hmc),
        compiled_sha256: files::hash(&bytes),
        q42_sha256: files::hash(&q42_bytes),
        m64_sha256: files::hash(&m64_bytes),
        target: requested.evidence.target,
    };
    files::write(
        &root.join("run.json"),
        &serde_json::to_vec_pretty(&info).map_err(|e| e.to_string())?,
    )?;
    println!(
        "Saved botanical native source/HMC/.10d/.q42/.m64 with {} identified parts; review pending",
        view.parts.len()
    );
    Ok(())
}

pub fn reopen_botanical(root: &Path) -> Result<(), String> {
    let info = metadata(root)?;
    let hmc_file = root.join("hmc").join(BOTANICAL_HMC);
    let _hmc = files::check_hash(&hmc_file, &info.hmc_sha256, 256 * 1024 * 1024)?;
    let authority = authority::retained(&root.join("authority"), &info)?;
    let reviewer = authority::login(&authority, true)?;
    let project = Arc::new(NativeProject::open_with_rebuild_limits(
        config(info.scope(), root),
        authority,
        reviewer.clone(),
        Some(BOTANICAL_HMC),
        Box::new(ValidateOnly),
        rebuild_limits(),
    )?);
    let mut host = SessionEditorHost::new(project, Arc::new(Mutex::new(Some(reviewer))));
    host.rebuild_execution(host.snapshot()?.source)?;
    let snapshot = host.snapshot()?;
    let view = host.authoring_view(snapshot.source.clone())?;
    let inventory = host.package_inventory(BOTANICAL_HMC, snapshot.source.clone())?;
    let loaded = checkpoint::load(&hmc_file, CheckpointLimits::default())?;
    let bytes = &loaded
        .resources()
        .iter()
        .find(|r| r.key == BOTANICAL_ASSET && r.kind == "10d")
        .ok_or("missing botanical .10d resource")?
        .bytes;
    let q42_bytes = &loaded
        .resources()
        .iter()
        .find(|r| r.key == BOTANICAL_Q42_KEY && r.kind == "q42")
        .ok_or("missing botanical .q42 resource")?
        .bytes;
    let m64_bytes = &loaded
        .resources()
        .iter()
        .find(|r| r.key == BOTANICAL_M64_ASSET && r.kind == "m64")
        .ok_or("missing botanical .m64 resource")?
        .bytes;
    if files::hash(bytes) != info.compiled_sha256
        || files::hash(q42_bytes) != info.q42_sha256
        || files::hash(m64_bytes) != info.m64_sha256
    {
        return Err("botanical .10d/.q42/.m64 hash mismatch".into());
    }
    files::check_hash(&root.join(BOTANICAL_ASSET), &info.compiled_sha256, 256 * 1024 * 1024)?;
    files::check_hash(&root.join(BOTANICAL_Q42_ASSET), &info.q42_sha256, 256 * 1024 * 1024)?;
    files::check_hash(&root.join(BOTANICAL_M64_ASSET), &info.m64_sha256, 256 * 1024 * 1024)?;

    let q42_reader = WasmSliceQ42Reader::open(q42_bytes).map_err(|e| format!("q42 open: {e}"))?;
    let q42_quins = q42_reader
        .collect_all_quins()
        .map_err(|e| format!("q42 quins: {e}"))?;
    if q42_quins.is_empty() || q42_quins.iter().any(|q| !q.verify_ecc_parity()) {
        return Err("botanical .q42 volume empty or failed ECC parity".into());
    }

    let mut m64_scratch = [M64PayloadInterval::default(); UI_INSPECTION_CHUNKS];
    let m64_summary = inventory
        .resources
        .iter()
        .find(|r| r.key == BOTANICAL_M64_ASSET)
        .ok_or("missing botanical .m64 in package inventory")?;
    let m64_inspected = host.inspect_package_resource(
        ResourceInspectionRequest {
            filename: BOTANICAL_HMC.into(),
            package_digest: inventory.digest,
            key: BOTANICAL_M64_ASSET.into(),
            resource_digest: m64_summary.digest,
        },
        snapshot.source.clone(),
        ui_inspection_limits(),
        &mut m64_scratch,
    )?;

    let scene = render::decode(bytes, &view)?;
    let cameras = [
        (
            "hero",
            PreviewCamera {
                yaw: 0.58,
                pitch: 0.26,
                distance: 4.2,
                target: [0.0, 1.35, 0.0],
            },
        ),
        (
            "opposite",
            PreviewCamera {
                yaw: 0.58 + std::f32::consts::PI,
                pitch: 0.26,
                distance: 4.2,
                target: [0.0, 1.35, 0.0],
            },
        ),
        (
            "junction-closeup",
            PreviewCamera {
                yaw: 0.42,
                pitch: 0.18,
                distance: 1.85,
                target: [0.0, 1.05, 0.0],
            },
        ),
        (
            "canopy-closeup",
            PreviewCamera {
                yaw: 0.50,
                pitch: 0.44,
                distance: 2.35,
                target: [0.0, 1.85, 0.0],
            },
        ),
    ];
    let mut probes = EnvironmentProbeSet::new();
    let _ = probes.push(EnvironmentProbe {
        position: [0.0, 1.4, 0.0],
        radius: 100.0,
        irradiance: [0.46, 0.49, 0.52],
        specular_levels: [[0.58, 0.61, 0.65]; ENVIRONMENT_SPECULAR_LEVELS],
        dominant_direction: [0.4, 0.8, 0.3],
        valid: true,
    });
    let mut receipts = Vec::new();
    for (name, camera) in cameras {
        let snap = host.snapshot()?;
        let mut settings =
            poet_cli::graphical::assembly::AssemblyRenderSettings::for_camera(camera);
        settings.clear_color = [0.13, 0.16, 0.19, 1.0];
        settings.sun_direction = [0.45, 0.82, 0.35];
        settings.sun_intensity = 2.2;
        settings.ambient_intensity = 0.14;
        let (rgba8, png) = render::render_16_9_png(
            snap.source,
            &scene,
            settings,
            Some(&probes),
        )?;
        let filename = format!("{name}.png");
        files::write(&root.join("images").join(&filename), &png)?;
        receipts.push(json!({
            "file": format!("images/{filename}"),
            "sha256": files::hash(&png),
            "rgba_sha256": files::hash(&rgba8),
            "width": RENDER_WIDTH,
            "height": RENDER_HEIGHT,
            "aspect_ratio": "16:9",
            "camera": {"yaw": camera.yaw, "pitch": camera.pitch, "distance": camera.distance, "target": camera.target}
        }));
        println!("Rendered {}", root.join("images").join(filename).display());
    }
    let m64_ver = { m64_inspected.payload.header.version };
    let m64_size = { m64_inspected.payload.header.total_file_size };
    files::json(
        &root.join("render-receipt.json"),
        &json!({
            "hmc_sha256": info.hmc_sha256,
            "compiled_sha256": info.compiled_sha256,
            "q42_sha256": info.q42_sha256,
            "q42_blocks": q42_reader.block_count(),
            "q42_quins": q42_quins.len(),
            "m64_sha256": info.m64_sha256,
            "m64_verification": {
                "version": m64_ver,
                "total_file_size": m64_size,
                "chunks": m64_inspected.payload.verification.chunks,
                "total_records": m64_inspected.payload.verification.total_records,
                "record_crc_checked": m64_inspected.payload.verification.record_crc_checked,
                "records_without_checksum": m64_inspected.payload.verification.records_without_checksum
            },
            "parts": scene.parts.len(),
            "vertices": scene.mesh.positions.len(),
            "triangles": scene.mesh.triangles.len(),
            "image_scope": "1280x720 (16:9) inspection views; not a master quality ceiling",
            "images": receipts,
            "review_status": "pending"
        }),
    )?;
    println!(
        "Fresh-process botanical package/source checks passed (.hmc/.10d/.q42/.m64); {} parts, {} vertices, {} triangles, {} Quins",
        scene.parts.len(),
        scene.mesh.positions.len(),
        scene.mesh.triangles.len(),
        q42_quins.len()
    );
    Ok(())
}
