//! Actual native host/codec tests. Diagnostic construction grants no human authority.
use super::*;
use crate::graphical::{
    checkpoints::{CheckpointEditorHost, CheckpointService, TrustedBinding},
    controller::{CanonicalEditorHost, PreviewProvider},
    geometry_host::GeometryHost,
    preview::PreviewFrame,
};
use qualia_core_db::{
    bundle::BundleReader,
    poet_host::invoke::studio_store::{self, checkpoint as core, StudioArtifact},
};
use std::{
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};
use vibe::{session::SessionLimits, Value};

static NEXT: AtomicU64 = AtomicU64::new(1);
struct Headless;
impl PreviewProvider for Headless {
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("No GPU frame in resource command fixture".into())
    }
}
fn encoded(id: &str, stamp: &studio_store::StoreStamp, changed: bool) -> Vec<u8> {
    let source = if changed {
        b"v 0 0 0\nv 2 0 0\nv 0 2 0\nf 1 2 3\n".as_slice()
    } else {
        b"v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n".as_slice()
    };
    let asset = qualia_core_db::render::compile_10d::compile_asset(
        source,
        Some("obj"),
        "urn:fixture:atomic-resource-geometry",
        "obj",
    )
    .unwrap();
    core::encode(
        &core::capture(id, stamp, Default::default()).unwrap(),
        vec![("model.10d".into(), asset)],
        Default::default(),
    )
    .unwrap()
}
fn host(
    root: &Path,
    library: &Path,
    budget: usize,
    immutable: bool,
) -> (ResourceImportHost, SaveReceipt, Vec<u8>) {
    let id = format!(
        "urn:poet:resource-plan:{}:{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let (_, stamp) = studio_store::create_artifact_if_absent(StudioArtifact::new(
        &id,
        "Geometry",
        "Atomic resource fixture",
    ))
    .unwrap();
    let original = encoded(&id, &stamp, false);
    let base = core::decode(&original, Default::default())
        .unwrap()
        .apply_resource_edits(
            vec![ResourceEdit::Add(ResourceData {
                key: "masters/unchanged.bin".into(),
                kind: "future/master".into(),
                bytes: vec![0, 42, 255],
                meta: Some(vec![0xa1, 0x61, b'x', 1]),
            })],
            Default::default(),
        )
        .unwrap();
    let original = core::encode(&base, vec![], Default::default()).unwrap();
    let saved = core::save(
        &root.join("working.hmc"),
        &original,
        ExpectedFile::Absent,
        Default::default(),
    )
    .unwrap();
    let updated = encoded(&id, &stamp, true);
    let reader = BundleReader::parse(&updated).unwrap();
    std::fs::write(library.join("model.10d"), reader.get("model.10d").unwrap()).unwrap();
    std::fs::write(
        library.join("model.q42"),
        reader.get("graphs/model.10d.q42").unwrap(),
    )
    .unwrap();
    let binding = TrustedBinding {
        project_id: "fixture-project".into(),
        document_id: id.clone(),
        source_id: id.clone(),
        domain: "Geometry".into(),
    };
    let policy = if immutable {
        crate::graphical::checkpoints::PackageWritePolicy::ImmutableRelease {
            filename: "working.hmc".into(),
            digest: saved.digest,
        }
    } else {
        crate::graphical::checkpoints::PackageWritePolicy::WorkingCopy
    };
    let service =
        CheckpointService::new_with_policy(root, binding, Default::default(), policy).unwrap();
    let canonical = CanonicalEditorHost::open(
        id,
        SessionLimits::default(),
        GeometryHost::new(vec![], 200_000),
        Box::new(Headless),
    )
    .unwrap();
    let checkpoint = CheckpointEditorHost::new(Box::new(canonical), service).unwrap();
    (
        ResourceImportHost::new(
            Box::new(checkpoint),
            ResourceLibrary::new(library, budget).unwrap(),
        ),
        saved,
        original,
    )
}
fn paired_plan(inventory: &PackageInventory, remove: bool) -> Vec<LibraryEdit> {
    ["model.10d", "graphs/model.10d.q42"]
        .into_iter()
        .map(|key| {
            let entry = inventory
                .resources
                .iter()
                .find(|entry| entry.key == key)
                .unwrap();
            if remove {
                LibraryEdit::Remove {
                    key: key.into(),
                    expected_resource: entry.digest,
                }
            } else {
                LibraryEdit::Import(ImportRequest {
                    key: key.into(),
                    filename: if key == "model.10d" {
                        "model.10d"
                    } else {
                        "model.q42"
                    }
                    .into(),
                    kind: String::new(),
                    expected_resource: Some(entry.digest),
                })
            }
        })
        .collect()
}

#[test]
fn atomic_native_plan_replaces_then_removes_real_10d_and_q42_together_preserving_master() {
    let root = tempfile::tempdir().unwrap();
    let library = tempfile::tempdir().unwrap();
    let (mut host, saved, original) = host(root.path(), library.path(), 16 * 1024 * 1024, false);
    let expected = host.snapshot().unwrap().source;
    let before = host
        .package_inventory("working.hmc", expected.clone())
        .unwrap();
    let stale_plan = paired_plan(&before, false);
    let plan = paired_plan(&before, false);
    assert!(host
        .apply_resource_plan(
            "working.hmc",
            ExpectedFile::Sha256(saved.digest),
            vec![plan[0].clone()],
            expected.clone()
        )
        .is_err());
    assert_eq!(std::fs::read(&saved.path).unwrap(), original);
    let changed = host
        .apply_resource_plan(
            "working.hmc",
            ExpectedFile::Sha256(saved.digest),
            plan,
            expected.clone(),
        )
        .unwrap();
    let after = host
        .package_inventory("working.hmc", expected.clone())
        .unwrap();
    for entry in &before.resources {
        let next = after
            .resources
            .iter()
            .find(|next| next.key == entry.key)
            .unwrap();
        if entry.key == "masters/unchanged.bin" {
            assert_eq!(entry, next);
        } else {
            assert_ne!(entry.digest, next.digest);
        }
    }
    assert!(host
        .apply_resource_plan(
            "working.hmc",
            ExpectedFile::Sha256(changed.digest),
            stale_plan,
            expected.clone()
        )
        .is_err());
    let removed = host
        .apply_resource_plan(
            "working.hmc",
            ExpectedFile::Sha256(changed.digest),
            paired_plan(&after, true),
            expected.clone(),
        )
        .unwrap();
    let final_inventory = host.package_inventory("working.hmc", expected).unwrap();
    assert_eq!(final_inventory.digest, removed.digest);
    assert_eq!(final_inventory.resources.len(), 1);
    assert_eq!(
        final_inventory.resources[0],
        *before
            .resources
            .iter()
            .find(|entry| entry.key == "masters/unchanged.bin")
            .unwrap()
    );
}

#[test]
fn all_intents_preflight_before_library_access_and_aggregate_budget_failure_is_atomic() {
    let root = tempfile::tempdir().unwrap();
    let library = tempfile::tempdir().unwrap();
    let (mut host, saved, original) = host(root.path(), library.path(), 1, false);
    let expected = host.snapshot().unwrap().source;
    let inventory = host
        .package_inventory("working.hmc", expected.clone())
        .unwrap();
    let mut invalid = paired_plan(&inventory, false);
    if let LibraryEdit::Import(request) = &mut invalid[0] {
        request.filename = "missing-file.bin".into();
    }
    if let LibraryEdit::Import(request) = &mut invalid[1] {
        request.expected_resource = Some([0; 32]);
    }
    let error = host
        .apply_resource_plan(
            "working.hmc",
            ExpectedFile::Sha256(saved.digest),
            invalid,
            expected.clone(),
        )
        .unwrap_err();
    assert!(error.contains("before any file access"), "{error}");
    assert!(host
        .apply_resource_plan(
            "working.hmc",
            ExpectedFile::Sha256(saved.digest),
            paired_plan(&inventory, false),
            expected.clone()
        )
        .is_err());
    assert_eq!(std::fs::read(&saved.path).unwrap(), original);
    let single_limit = std::fs::metadata(library.path().join("model.10d"))
        .unwrap()
        .len()
        .max(
            std::fs::metadata(library.path().join("model.q42"))
                .unwrap()
                .len(),
        ) as usize;
    host.library = ResourceLibrary::new(library.path(), single_limit).unwrap();
    assert!(host
        .apply_resource_plan(
            "working.hmc",
            ExpectedFile::Sha256(saved.digest),
            paired_plan(&inventory, false),
            expected.clone()
        )
        .is_err());
    assert_eq!(std::fs::read(&saved.path).unwrap(), original);
    let request = LibraryEdit::Import(ImportRequest {
        filename: "model.10d".into(),
        key: "new-resource".into(),
        kind: "opaque".into(),
        expected_resource: None,
    });
    assert!(host
        .apply_resource_plan(
            "working.hmc",
            ExpectedFile::Sha256(saved.digest),
            vec![request.clone(), request],
            expected
        )
        .unwrap_err()
        .contains("duplicate"));
    let root = tempfile::tempdir().unwrap();
    let library = tempfile::tempdir().unwrap();
    let (mut release, saved, _) = self::host(root.path(), library.path(), 1024, true);
    let expected = release.snapshot().unwrap().source;
    let inventory = release
        .package_inventory("working.hmc", expected.clone())
        .unwrap();
    std::fs::remove_file(library.path().join("model.10d")).unwrap();
    assert!(release
        .apply_resource_plan(
            "working.hmc",
            ExpectedFile::Sha256(saved.digest),
            paired_plan(&inventory, false),
            expected
        )
        .unwrap_err()
        .contains("immutable"));
}

#[test]
fn ui_mcp_typed_command_draft_is_strict_bounded_and_has_no_paths() {
    let digest = "ab".repeat(32);
    let draft = serde_json::json!([
        {"op":"replace","filename":"model.10d","key":"model.10d","expected_resource_sha256":digest},
        {"op":"replace","filename":"model.q42","key":"graphs/model.10d.q42","expected_resource_sha256":"cd".repeat(32)}
    ]);
    assert_eq!(
        parse_resource_plan_json(&draft.to_string()).unwrap().len(),
        2
    );
    for invalid in [
        serde_json::json!([]),
        serde_json::json!([{"op":"add","filename":"../secret","key":"new","kind":"opaque"}]),
        serde_json::json!([{"op":"remove","key":"model.10d","expected_resource_sha256":"ab".repeat(32),"principal":"owner"}]),
    ] {
        assert!(parse_resource_plan_json(&invalid.to_string()).is_err());
    }
    let item = serde_json::json!({"op":"remove","key":"model.10d","expected_resource_sha256":"ab".repeat(32)});
    assert!(
        parse_resource_plan_json(&serde_json::Value::Array(vec![item; 65]).to_string()).is_err()
    );
}
