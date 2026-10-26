//! Mixed native-codec restart fixture, separate from the process lifecycle.
use super::{checkpoint, CheckpointEditorHost, EditorHost, ExpectedFile, Path, FILE};
use crate::graphical::checkpoints::SaveReceipt;

pub(super) fn add(
    editor: &mut CheckpointEditorHost,
    receipt: SaveReceipt,
    root: &Path,
) -> SaveReceipt {
    // Actual native compiled geometry + native Q42/lexicon, alongside opaque
    // editable sources/metadata, survive a real process exit and material edit.
    let compiled = qualia_core_db::render::compile_10d::compile_asset(
        b"v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n",
        Some("obj"),
        "urn:poet:restart:independent-codec-fixture",
        "obj",
    )
    .unwrap();
    let captured = checkpoint::load(&receipt.path, Default::default()).unwrap();
    let packaged = checkpoint::encode(
        &captured,
        vec![("codec/model.10d".into(), compiled)],
        Default::default(),
    )
    .unwrap();
    let parsed = checkpoint::decode(&packaged, Default::default()).unwrap();
    let mut edits = parsed
        .resources()
        .iter()
        .cloned()
        .map(crate::graphical::checkpoints::ResourceEdit::Add)
        .collect::<Vec<_>>();
    for (key, kind, bytes) in [
        (
            "masters/source.obj",
            "text/plain",
            b"# retained editable source fixture\nv 1 2 3\n".as_slice(),
        ),
        (
            "rigs/source.json",
            "application/json",
            b"{\"joints\":[],\"fixture\":true}".as_slice(),
        ),
        ("textures/source.bytes", "future/texture", &[0, 42, 255]),
        (
            "unknown/evidence.bin",
            "future/evidence",
            b"unreviewed opaque evidence".as_slice(),
        ),
        (
            "inert/never-run.vibe",
            "text/vibescript",
            b"grant(\"filesystem\"); effect(\"must-never-run\");".as_slice(),
        ),
    ] {
        edits.push(crate::graphical::checkpoints::ResourceEdit::Add(
            crate::graphical::checkpoints::ResourceData {
                key: key.into(),
                kind: kind.into(),
                bytes: bytes.to_vec(),
                meta: Some(vec![0xa1, 0x61, b'x', 1]),
            },
        ));
    }
    let receipt = editor
        .edit_package_resources(
            FILE,
            ExpectedFile::Sha256(receipt.digest),
            edits,
            editor.snapshot().unwrap().source,
        )
        .unwrap();
    let baseline = std::fs::read(&receipt.path).unwrap();
    checkpoint::save(
        &root.join("resource-baseline.hmc"),
        &baseline,
        ExpectedFile::Absent,
        Default::default(),
    )
    .unwrap();
    receipt
}
