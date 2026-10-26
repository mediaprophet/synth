use crate::*;
use poet_cli::graphical::{
    authoring::{AuthoringEdit, AuthoringView},
    controller::EditorHost,
    cooperative_workflow::WorkflowAction,
    native_project::NativeProject,
    session_host::SessionEditorHost,
};
use qualia_core_db::poet_host::invoke::studio_store::checkpoint::{
    ExpectedFile, ResourceData, ResourceEdit,
};
use serde_json::json;
use std::sync::{Arc, Mutex};

pub fn produce(root: &Path, profile: &str, source_root: &Path) -> Result<(), String> {
    let started = std::time::Instant::now();
    if !matches!(profile, "film" | "open_world_game" | "cartoon") {
        return Err("unknown authored profile".into());
    }
    let master = files::read(&source_root.join("master.vibe"), 64 * 1024)?;
    let preset = files::read(
        &source_root.join("presets").join(format!("{profile}.vibe")),
        64 * 1024,
    )?;
    let master_text = std::str::from_utf8(&master).map_err(|e| e.to_string())?;
    let preset_text = std::str::from_utf8(&preset).map_err(|e| e.to_string())?;
    for name in ["authority", "hmc", "images"] {
        files::create_directory(&root.join(name))?;
    }
    files::write(&root.join("master.vibe"), &master)?;
    files::write(&root.join("selected-preset.vibe"), &preset)?;
    let (scope, authority) = authority::create(&root.join("authority"))?;
    let session = authority::login(&authority, false)?;
    let project = Arc::new(NativeProject::open(
        config(scope.clone(), root),
        authority.clone(),
        session.clone(),
        None,
        Box::new(ValidateOnly),
    )?);
    let mut host = SessionEditorHost::new(project, Arc::new(Mutex::new(Some(session))));
    let source = host.snapshot()?.source;
    host.submit(master_text, source)
        .map_err(|e| format!("master submission: {e}"))?;
    let source = host.snapshot()?.source;
    host.submit(preset_text, source)
        .map_err(|e| format!("preset submission: {e}"))?;
    let initial = host.authoring_view(host.snapshot()?.source)?;
    let lower = initial
        .parameters
        .iter()
        .find(|(k, _)| k == "lower_level")
        .ok_or("missing lower_level")?
        .1;
    host.authoring_edit(
        AuthoringEdit::Parameter {
            key: "lower_level".into(),
            value: lower + 0.01,
        },
        host.snapshot()?.source,
    )?;
    // Rebuild precedes material/transform editing so it cannot reset those edits.
    let after_parameter = host.authoring_view(host.snapshot()?.source)?;
    let frame = after_parameter
        .parts
        .iter()
        .find(|p| p.material_id == 9001)
        .ok_or("missing frame material")?;
    host.authoring_edit(
        AuthoringEdit::Material {
            material_id: 9001,
            base_color: frame.base_color,
            roughness: (frame.roughness + 0.02).min(1.0),
            metallic: frame.metallic,
        },
        host.snapshot()?.source,
    )?;
    let after_material = host.authoring_view(host.snapshot()?.source)?;
    let hatch = after_material
        .parts
        .iter()
        .find(|p| p.id == "cabinet/front-hatch")
        .ok_or("missing hatch")?;
    let mut translation = hatch.translation;
    translation[2] -= 0.002;
    host.authoring_edit(
        AuthoringEdit::Transform {
            id: hatch.id.clone(),
            translation,
            rotation: hatch.rotation,
            scale: hatch.scale,
        },
        host.snapshot()?.source,
    )?;
    let current = host.snapshot()?;
    if !current.publication_is_current() {
        return Err("author publication retired".into());
    }
    let view = host.authoring_view(current.source.clone())?;
    let cells = host.source_cells(current.source.clone())?;
    let accepted = cells.join("\n\n").into_bytes();
    files::write(&root.join("accepted-source.vibe"), &accepted)?;
    files::json(&root.join("parts.json"), &view_json(&view))?;
    let saved = host.save_checkpoint(HMC, ExpectedFile::Absent, current.source.clone())?;
    let compiled = host.compile_checkpoint(
        HMC,
        ASSET,
        ExpectedFile::Sha256(saved.digest),
        current.source.clone(),
    )?;
    let lineage = json!({"schema":"poet-reference-lineage/1","synthetic_concept":true,
        "fixture_only":true,"dossier":"community-grounds-growing-station-v0.1","entity":"CGE-DOSSIER:CGGS-01",
        "catalogue":"ecopunk-model-deliverables","catalogue_version":"2026-10-24.2",
        "catalogue_sha256":"c0cb76151bf5d0776afaae4d59d7427efb65633bf359d3f4c2bda83c361c91cb",
        "profile":profile,"style_lod_target_distinct":true,"master_sha256":files::hash(&master),
        "preset_sha256":files::hash(&preset),"accepted_source_sha256":files::hash(&accepted),
        "operations":["Parameter lower_level +0.01m","Material 9001 roughness +0.02","Transform cabinet/front-hatch local z -0.002m"],
        "consumer_candidate":"rc:asset/hydroponic-channel-frame","consumer_mapping":"candidate only; no equivalence, adoption or compatibility verified",
        "plants":"none; biological dependencies remain separate","rights":"design rights and adoption unassessed",
        "film":"incomplete/unaccepted","open_world_game":"incomplete/unaccepted","cartoon":"incomplete/unaccepted"});
    let resources = [
        (MASTER_KEY, "text/plain", master.clone()),
        (PRESET_KEY, "text/plain", preset.clone()),
        (ACCEPTED_KEY, "text/plain", accepted.clone()),
        (
            "masters/growing-station/lineage.json",
            "application/json",
            serde_json::to_vec_pretty(&lineage).map_err(|e| e.to_string())?,
        ),
    ];
    let package = host.edit_package_resources(
        HMC,
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
    files::json(&root.join("lineage.json"), &lineage)?;
    let before = host.workflow(WorkflowAction::Inspect)?;
    if before.evidence.principal_subject != authority::AUTHOR_SUBJECT || before.evidence.can_review
    {
        return Err("unexpected author scope/review grant".into());
    }
    host.workflow(WorkflowAction::Contribution {
        target: before.evidence.target.clone(), id: "software-native-production/1".into(),
        description: "Controlled SoftwareAgent native source evaluation, parameter/material/transform editing and source/compiled HMC persistence. Effort is measured producer wall minutes rounded down, excludes prior source authorship; payment status unknown, no valuation or rights adoption.".into(),
        minutes: u32::try_from(started.elapsed().as_secs()/60).map_err(|_|"execution time overflow")?,
        work_status: qualia_cooperative_core::authoring::WorkStatus::Unknown,
    })?;
    let requested = host.workflow(WorkflowAction::RequestReview {target:before.evidence.target.clone(),id:REQUEST.into(),
        evidence:format!("accepted-source.vibe sha256={}; HMC sha256={}; pending distinct SoftwareAgent technical-source review; no human/master acceptance",files::hash(&accepted),files::hash(&files::read(&hmc_path(root),256*1024*1024)?))})?;
    let hmc = files::read(&hmc_path(root), 256 * 1024 * 1024)?;
    let detached = qualia_core_db::poet_host::invoke::studio_store::checkpoint::load(
        &hmc_path(root),
        CheckpointLimits::default(),
    )?;
    let bytes = detached
        .resources()
        .iter()
        .find(|r| r.key == ASSET && r.kind == "10d")
        .ok_or("compiled .10d missing from HMC")?
        .bytes
        .clone();
    files::write(&root.join(ASSET), &bytes)?;
    let info = RunInfo {
        schema: "poet-growing-reference/software-fixture/1".into(),
        fixture_only: true,
        project_id: scope.project_id,
        document_id: scope.document_id,
        source_id: scope.source_id,
        profile: profile.into(),
        master_sha256: files::hash(&master),
        preset_sha256: files::hash(&preset),
        accepted_source_sha256: files::hash(&accepted),
        hmc_sha256: files::hash(&hmc),
        compiled_sha256: files::hash(&bytes),
        target: requested.evidence.target.clone(),
    };
    files::write(
        &root.join("run.json"),
        &serde_json::to_vec_pretty(&info).map_err(|e| e.to_string())?,
    )?;
    files::json(
        &root.join("author-receipt.json"),
        &json!({"fixture_only":true,"principal_kind":"SoftwareAgent",
        "principal":requested.evidence.principal_subject,"source_target":info.target,
        "source_review_requests":requested.evidence.requests,"reviews":requested.evidence.reviews,
        "contributions":requested.evidence.contributions,
        "package_writes":requested.evidence.package_writes,"review_status":"pending",
        "source_cells":cells.len(),"parts":view.parts.len(),"vertex_budget":MAX_VERTICES,"index_budget":MAX_INDICES,
        "session_retained_value_accounting_bytes":128*1024*1024,"native_dispatch_budget":256,
        "hmc_sha256":info.hmc_sha256,"compiled_sha256":info.compiled_sha256,
        "checkpoint_durability_warnings":[saved.durability_warning,compiled.durability_warning,package.durability_warning],
        "qualification":"public fixture seeds; technical execution only; no real host adoption, OS credentials, human enrollment, quality approval or complete embodiment"}),
    )?;
    println!(
        "Saved actual native source/HMC with {} identified parts; review pending",
        view.parts.len()
    );
    Ok(()) // all host snapshots and journal ownership drop before child launch
}

pub fn view_json(view: &AuthoringView) -> serde_json::Value {
    json!({"parameters":view.parameters,"parts":view.parts.iter().map(|p|json!({"id":p.id,
        "parent":p.parent,"semantic_id":p.semantic_id,"translation":p.translation,"rotation":p.rotation,
        "scale":p.scale,"material_id":p.material_id,"base_color":p.base_color,"roughness":p.roughness,
        "metallic":p.metallic,"vertices":p.vertices,"triangles":p.triangles})).collect::<Vec<_>>()})
}
