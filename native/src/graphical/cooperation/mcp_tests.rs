//! End-to-end protocol/native-source tests with explicitly synthetic authority.
//! Real canonical evaluation and store journalling; no fake rendered frame.
use super::*;
use crate::graphical::{
    materials::{self, MaterialDraft},
    mcp::NativeAuthoringBackend,
    preview::PreviewLimits,
};
use qualia_mcp::authoring::{AuthoringServer, ProtocolLimits, AUTHORING_PROTOCOL_VERSION};
use serde_json::{json, Value};

struct ValidatedNoFrame;
impl PreviewProvider for ValidatedNoFrame {
    fn validate(&self, value: &vibe::Value) -> Result<(), String> {
        materials::authored_scene(
            value,
            qualia_core_db::container_10d::MaterialRecord::legacy_default(),
            PreviewLimits::default(),
        )
        .map(|_| ())
    }
    fn preview(&mut self, _: SourceRef, _: &vibe::Value) -> Result<PreviewFrame, String> {
        Err("protocol fixture intentionally runs no GPU; frame remains unassessed".into())
    }
}
fn server(scope: Scope, owner: Arc<FixtureOwner>) -> AuthoringServer<NativeAuthoringBackend> {
    let grants = vec![
        "capability.invoke".into(),
        "ComputationalGeometry.extrude_profile".into(),
    ];
    let inner = CanonicalEditorHost::open(
        scope.source_id.clone(),
        SessionLimits::default(),
        GeometryHost::new(grants, 200_000),
        Box::new(ValidatedNoFrame),
    )
    .unwrap();
    let host =
        AuthorizedEditorHost::new(Box::new(inner), scope, owner, GatewayLimits::default()).unwrap();
    let mut server = AuthoringServer::new(
        NativeAuthoringBackend::bound(host),
        ProtocolLimits::default(),
    )
    .unwrap();
    let response = server.process(&json!({"jsonrpc":"2.0","id":"init","method":"initialize","params":{
        "protocolVersion":AUTHORING_PROTOCOL_VERSION,"capabilities":{},"clientInfo":{"name":"native policy fixture","version":"0"}}}).to_string()).unwrap();
    assert_eq!(
        response["result"]["protocolVersion"],
        AUTHORING_PROTOCOL_VERSION
    );
    assert!(server
        .process(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#)
        .is_none());
    server
}
fn call(
    server: &mut AuthoringServer<NativeAuthoringBackend>,
    name: &str,
    arguments: Value,
) -> Value {
    server.process(&json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":arguments}}).to_string()).unwrap()
}
fn success(response: Value) -> Value {
    assert!(
        response.get("error").is_none(),
        "protocol failure: {response}"
    );
    assert_eq!(
        response["result"]["isError"], false,
        "native failure: {response}"
    );
    response["result"]["structuredContent"].clone()
}
fn read(server: &mut AuthoringServer<NativeAuthoringBackend>) -> Value {
    success(call(server, "poet_source_read", json!({})))
}

#[test]
fn protocol_real_source_material_undo_redo_and_rebuild_use_same_native_policy() {
    let scope = scope();
    let owner = Arc::new(FixtureOwner::new(&scope, ProjectRole::Contributor));
    let mut server = server(scope.clone(), owner.clone());
    let initial = read(&mut server);
    let source = materials::diagnostic_source();
    let check = success(call(
        &mut server,
        "poet_source_check",
        json!({"context":initial["context"],"source":source}),
    ));
    assert_eq!(check["executed"], false);
    let submitted = success(call(
        &mut server,
        "poet_source_submit",
        json!({"context":initial["context"],"source":source}),
    ));
    assert_eq!(submitted["session_revision"], 1);
    assert_eq!(submitted["preview_rendered"], false);
    assert_eq!(submitted["human_review"], "unassessed");
    assert_eq!(
        call(&mut server, "poet_source_read", json!({}))["result"]["isError"],
        true,
        "native provider must explicitly reconcile current document metadata"
    );
    owner.refresh_document();
    let current = read(&mut server);
    assert_eq!(current["source_cells"].as_array().unwrap().len(), 1);
    let edit = MaterialDraft {
        base_color: [0.05, 0.4, 0.8, 1.0],
        roughness: 0.2,
        metallic: 0.7,
        ..MaterialDraft::default()
    }
    .edit_source(2, materials::diagnostic_material_id())
    .unwrap();
    let edited = success(call(
        &mut server,
        "poet_material_apply",
        json!({"context":current["context"],"source":edit}),
    ));
    assert_eq!(edited["session_revision"], 2);
    assert_eq!(
        call(
            &mut server,
            "poet_source_submit",
            json!({"context":current["context"],"source":"1;"})
        )["result"]["isError"],
        true
    );
    owner.refresh_document();
    let current = read(&mut server);
    assert_eq!(current["source_cells"].as_array().unwrap().len(), 2);
    let undone = success(call(
        &mut server,
        "poet_source_undo",
        json!({"context":current["context"]}),
    ));
    assert_eq!(undone["reconciliation_required"], true);
    owner.refresh_document();
    let held = read(&mut server);
    assert_eq!(held["source_cells"].as_array().unwrap().len(), 1);
    let rebuilt = success(call(
        &mut server,
        "poet_execution_rebuild",
        json!({"context":held["context"]}),
    ));
    assert_eq!(rebuilt["session_revision"], 1);
    assert_eq!(rebuilt["reconciliation_required"], false);
    let current = read(&mut server);
    let redone = success(call(
        &mut server,
        "poet_source_redo",
        json!({"context":current["context"]}),
    ));
    assert_eq!(redone["reconciliation_required"], true);
    owner.refresh_document();
    let held = read(&mut server);
    assert_eq!(held["source_cells"].as_array().unwrap().len(), 2);
    let rebuilt = success(call(
        &mut server,
        "poet_execution_rebuild",
        json!({"context":held["context"]}),
    ));
    assert_eq!(rebuilt["session_revision"], 2);
    assert_eq!(rebuilt["reconciliation_required"], false);
    let current = read(&mut server);
    assert_eq!(
        current["material_edit_id"].as_u64(),
        Some(materials::diagnostic_material_id())
    );
}

#[test]
fn protocol_observer_cannot_write_or_supply_principal_and_no_fake_checkpoint_tool() {
    let scope = scope();
    let owner = Arc::new(FixtureOwner::new(&scope, ProjectRole::Observer));
    let mut server = server(scope.clone(), owner);
    let response = server
        .process(&json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}).to_string())
        .unwrap();
    assert!(!response["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|tool| tool["name"] == "poet_hmc_save"));
    let current = read(&mut server);
    assert_eq!(
        call(
            &mut server,
            "poet_source_submit",
            json!({"context":current["context"],"source":materials::diagnostic_source()})
        )["result"]["isError"],
        true
    );
    assert!(studio_store::get_artifact_stamped(&scope.source_id)
        .unwrap()
        .0
        .nodes
        .is_empty());
    assert_eq!(
        call(
            &mut server,
            "poet_source_read",
            json!({"principal":"test-owner"})
        )["result"]["isError"],
        true
    );
}
