//! Fresh-process canonical reopen, source witness checks, and optional explicit review.
use crate::*;
use poet_cli::graphical::{
    controller::EditorHost, cooperative_workflow::WorkflowAction, native_project::NativeProject,
    session_host::SessionEditorHost,
};
use qualia_cooperative_core::authoring::ReviewDecision;
use qualia_core_db::poet_host::invoke::studio_store::checkpoint;
use serde_json::json;
use std::sync::{Arc, Mutex};

pub fn reopen(root: &Path, pixels: bool, decision: Option<(&str, &str)>) -> Result<(), String> {
    let info = metadata(root)?;
    let _hmc = files::check_hash(&hmc_path(root), &info.hmc_sha256, 256 * 1024 * 1024)?;
    let authority = authority::retained(&root.join("authority"), &info)?;
    let reviewer = authority::login(&authority, true)?;
    let project = Arc::new(NativeProject::open(
        config(info.scope(), root),
        authority,
        reviewer.clone(),
        Some(HMC),
        Box::new(ValidateOnly),
    )?);
    let mut host = SessionEditorHost::new(project, Arc::new(Mutex::new(Some(reviewer))));
    host.rebuild_execution(host.snapshot()?.source)?;
    let snapshot = host.snapshot()?;
    if !snapshot.publication_is_current() {
        return Err("reopened source/execution unavailable".into());
    }
    let cells = host.source_cells(snapshot.source.clone())?;
    let accepted = cells.join("\n\n").into_bytes();
    if cells.len() != 5 || files::hash(&accepted) != info.accepted_source_sha256 {
        return Err("reopened five-cell editable source differs from accepted witness".into());
    }
    if files::hash(cells[0].as_bytes()) != info.master_sha256
        || files::hash(cells[1].as_bytes()) != info.preset_sha256
    {
        return Err("reopened original master/preset witness mismatch".into());
    }
    files::check_hash(
        &root.join("accepted-source.vibe"),
        &info.accepted_source_sha256,
        1024 * 1024,
    )?;
    let view = host.authoring_view(snapshot.source.clone())?;
    let workflow = host.workflow(WorkflowAction::Inspect)?;
    if workflow.evidence.principal_subject != authority::REVIEWER_SUBJECT
        || !workflow.evidence.can_review
        || workflow.evidence.target != info.target
        || workflow
            .evidence
            .requests
            .iter()
            .filter(|r| {
                r.id == REQUEST
                    && r.target == info.target
                    && r.requester_subject == authority::AUTHOR_SUBJECT
            })
            .count()
            != 1
    {
        return Err(
            "reopened current target, distinct reviewer grant or exact request unavailable".into(),
        );
    }
    // Also use the same authenticated public inventory API as the native UI.
    let _inventory = host.package_inventory(HMC, snapshot.source.clone())?;
    let loaded = checkpoint::load(&hmc_path(root), CheckpointLimits::default())?;
    let resource = |key: &str| {
        loaded
            .resources()
            .iter()
            .find(|r| r.key == key)
            .ok_or_else(|| format!("missing HMC resource {key}"))
    };
    let bytes = &resource(ASSET)?.bytes;
    if resource(ASSET)?.kind != "10d"
        || files::hash(bytes) != info.compiled_sha256
        || files::hash(&resource(MASTER_KEY)?.bytes) != info.master_sha256
        || files::hash(&resource(PRESET_KEY)?.bytes) != info.preset_sha256
        || resource(ACCEPTED_KEY)?.bytes != accepted
    {
        return Err("HMC asset/master/preset/accepted source integrity mismatch".into());
    }
    let regenerated = host.compiled_assets(ASSET, snapshot.source.clone())?;
    if regenerated.len() != 1
        || regenerated[0].0 != ASSET
        || regenerated[0].1.container_10d != *bytes
    {
        return Err(
            "reconstructed accepted source does not reproduce the exact packaged .10d".into(),
        );
    }
    files::check_hash(&root.join(ASSET), &info.compiled_sha256, 256 * 1024 * 1024)?;
    let scene = render::decode(bytes, &view)?;
    let checks = json!({"fixture_only":true,"principal_kind":"SoftwareAgent",
        "principal":workflow.evidence.principal_subject,"source_target":workflow.evidence.target,
        "hmc_sha256":info.hmc_sha256,"compiled_sha256":info.compiled_sha256,
        "accepted_source_sha256":info.accepted_source_sha256,"source_cells":cells.len(),
        "parts":scene.parts.len(),"materials":scene.materials.len(),"ranges":scene.ranges.len(),
        "vertices":scene.mesh.positions.len(),"triangles":scene.mesh.triangles.len(),
        "performed_checks":["fresh process canonical HMC source reopen under fresh signed reviewer challenge",
            "current exact target and explicit distinct reviewer grant","master/preset/source SHA256 witnesses match HMC",
            "accepted-source reconstruction produces exact packaged .10d bytes","native .10d integrity, normals, UV and material-range decode",
            "stable semantic IDs cover every authored part and package range","source/execution/authority publication guards current"],
        "quality_status":"unassessed; no master, human, whole-HMC or consumer acceptance"});
    if pixels {
        let image_receipts = render::images(root, &scene, &host)?;
        files::json(
            &root.join("render-receipt.json"),
            &json!({"checks":checks,"images":image_receipts,
            "render_source":"decoded .10d resource from reopened HMC; not regenerated mesh",
            "lighting":"explicit synthetic constant prefiltered neutral probe plus native sun/AO/shadows; no photographic environment",
            "image_scope":"1024 inspection views; not a master quality ceiling","review_status":"pending"}),
        )?;
    }
    if let Some((filename, choice)) = decision {
        // Public library ingress owns filename and library-resolution validation.
        let library = poet_cli::graphical::package::ResourceLibrary::new(root, 4096)?;
        let library_bytes = library.read(filename)?;
        let evidence_bytes = files::read(&root.join(filename), 4096)?;
        if evidence_bytes != library_bytes {
            return Err("review evidence changed during bounded guarded read".into());
        }
        let evidence_text = std::str::from_utf8(&evidence_bytes).map_err(|e| e.to_string())?;
        if evidence_text.trim().is_empty() {
            return Err("review requires explicit bounded evidence text".into());
        }
        let review = match choice {
            "accepted" => ReviewDecision::Accepted,
            "changes-requested" => ReviewDecision::ChangesRequested,
            "rejected" => ReviewDecision::Rejected,
            _ => return Err("unsupported review decision".into()),
        };
        // Choice is explicit in a separate invocation. Evidence text is a supplied
        // witness, not automatically treated as verified human/visual acceptance.
        let evidence = format!("SoftwareAgent technical-source scope only; checks repeated by this command. supplied evidence file {filename} sha256={}; HMC sha256={}; .10d sha256={}",files::hash(&evidence_bytes),info.hmc_sha256,info.compiled_sha256);
        let result = host.workflow(WorkflowAction::Review {target:info.target.clone(),request_id:REQUEST.into(),
            id:"software-source-decision/1".into(),decision:review,
            rationale:format!("Explicit controlled SoftwareAgent decision after source/package technical checks. No human, master-quality or whole-package acceptance. Supplied witness: {}",evidence_text.chars().take(1500).collect::<String>()),evidence})?;
        files::json(
            &root.join("review-receipt.json"),
            &json!({"technical_checks":checks,"reviews":result.evidence.reviews,
            "requests":result.evidence.requests,"qualification":"SoftwareAgent source decision only; all master/package acceptance gates remain open"}),
        )?;
    }
    if !snapshot.publication_is_current() {
        return Err("reopen result retired before closeout".into());
    }
    files::check_hash(&hmc_path(root), &info.hmc_sha256, 256 * 1024 * 1024)?;
    println!(
        "Fresh-process native package/source checks passed; {} parts, {} triangles",
        scene.parts.len(),
        scene.mesh.triangles.len()
    );
    Ok(())
}
