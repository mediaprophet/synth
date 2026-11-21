//! Cross-host and scoped resource delivery regressions; native GPU review is separate.
use super::*;
use std::sync::atomic::AtomicBool;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("synth-interface-test-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn backend(&self) -> StandaloneSynthMcp {
        StandaloneSynthMcp::default().with_input_root(&self.0).unwrap()
            .with_output_root(NativeOutputRoot::admit(&self.0).unwrap())
    }
}
impl Drop for Fixture { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }

#[test]
fn source_path_and_inline_share_admission_and_exact_replay() {
    let fixture = Fixture::new();
    let source = format!("let data = [{}]; let asset = {{files:[{{key:\"value.bin\",kind:\"bin\",bytes:data}}]}};", vec!["0"; 34_000].join(","));
    std::fs::write(fixture.0.join("source.vibe"), source.as_bytes()).unwrap();
    let mut backend = fixture.backend();
    let inline = backend.call("synth_produce_present", json!({"source":source})).unwrap();
    let path = backend.call("synth_produce_present", json!({"source_path":"source.vibe","export_dir":"path-export"})).unwrap();
    assert_eq!(path["receipt"]["resources"][0]["bytes"], 34_000);
    assert_eq!(inline["receipt"]["source_sha256"], path["receipt"]["source_sha256"]);
    assert_eq!(inline["receipt"]["hmc"]["sha256"], path["receipt"]["hmc"]["sha256"]);
    assert_eq!(std::fs::read(fixture.0.join("path-export/.synth/source.vibe")).unwrap(), source.as_bytes());
    assert_eq!(path["receipt"]["limits"]["source_bytes"], source.len());
    assert_eq!(path["receipt"]["limits"]["source_work_units"], source.len() * 4);
    assert_eq!(path["receipt"]["limits"]["effective_asset_work_units"], 2_000_000_000u64 - source.len() as u64 * 4);
    assert_eq!(backend.call("synth_produce_present", json!({})).unwrap_err().code, "invalid_arguments");
    assert_eq!(backend.call("synth_produce_present", json!({"source":"x","source_path":"source.vibe"})).unwrap_err().code, "invalid_arguments");
    assert_eq!(backend.call("synth_produce_present", json!({"source_path":"../outside.vibe"})).unwrap_err().code, "native_input_refused");
}

#[test]
fn admitted_input_export_matches_direct_production_and_has_durable_receipt() {
    let fixture = Fixture::new();
    std::fs::write(fixture.0.join("reference.bin"), [1, 4, 9, 16]).unwrap();
    let source = r#"let mut asset = {files:[{key:"copies/reference.bin",kind:"bin",input:"reference"}]};"#;
    let mut mcp = fixture.backend();
    let result = mcp.call("synth_produce_present", json!({"source": source,
        "inputs":[{"key":"reference","kind":"bin","path":fixture.0.join("reference.bin")}],
        "out_hmc":"asset.hmc","export_dir":"exported","receipt_json":"receipt.json"})).unwrap();
    let registry = ProducerRegistry::with_builtins();
    let mut direct = registry.session("files", vec![crate::production::Resource {key:"reference".into(),kind:"bin".into(),bytes:vec![1,4,9,16]}], ProductionLimits::default()).unwrap();
    direct.submit(source, &AtomicBool::new(false)).unwrap();
    assert_eq!(std::fs::read(fixture.0.join("asset.hmc")).unwrap(), direct.candidate().unwrap().bundle());
    assert_eq!(std::fs::read(fixture.0.join("exported/copies/reference.bin")).unwrap(), [1,4,9,16]);
    let receipt: Value = serde_json::from_slice(&std::fs::read(fixture.0.join("receipt.json")).unwrap()).unwrap();
    assert_eq!(receipt["hmc"]["sha256"], direct.candidate().unwrap().sha256());
    assert_eq!(receipt["catalog"]["count"], 150);
    assert_eq!(receipt["inputs"][0]["sha256"].as_str().unwrap().len(), 64);
    assert_eq!(receipt["binary"]["sha256"].as_str().unwrap().len(), 64);
    assert_eq!(result["receipt"], receipt);
    assert!(Path::new(receipt["resources"][0]["path"].as_str().unwrap()).is_absolute());
    assert_eq!(std::fs::read_to_string(fixture.0.join("exported/.synth/source.vibe")).unwrap(), source);
    assert!(fixture.0.join("exported/.synth/production-receipt.json").is_file());
}

#[test]
fn input_admission_rejects_unconfigured_outside_traversal_and_unknown_keys_before_outputs() {
    let fixture = Fixture::new();
    let outside = Fixture::new();
    std::fs::write(outside.0.join("outside.bin"), [42]).unwrap();
    let source = r#"let mut asset = {files:[]};"#;
    let mut mcp = fixture.backend();
    for path in [outside.0.join("outside.bin").display().to_string(), "../outside.bin".into()] {
        let error = mcp.call("synth_produce_present", json!({"source":source,"inputs":[{"key":"x","kind":"bin","path":path}],"export_dir":"must-not-exist"})).unwrap_err();
        assert_eq!(error.code, "native_input_refused");
        assert!(!fixture.0.join("must-not-exist").exists());
    }
    let error = StandaloneSynthMcp::default().call("synth_produce_present", json!({"source":source,"inputs":[{"key":"x","kind":"bin","path":"x"}]})).unwrap_err();
    assert_eq!(error.code,"native_input_unconfigured");
    assert_eq!(mcp.call("synth_produce_present",json!({"source":source,"unknown":true})).unwrap_err().code,"invalid_arguments");
    assert_eq!(mcp.call("synth_produce_present",json!({"source":source,"preview_assets":["mesh.10d"]})).unwrap_err().code,"invalid_arguments");
}

#[test]
fn wrong_host_error_reaches_mcp_is_error_and_initialize_names_production_host() {
    let mut server = qualia_mcp::authoring::AuthoringServer::new(StandaloneSynthMcp::default(), comprehensive_protocol_limits()).unwrap();
    let init = server.process(&json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"fixture","version":"1"}}}).to_string()).unwrap();
    let instructions = init["result"]["instructions"].as_str().unwrap();
    assert!(instructions.contains("synth_produce_present"));
    assert!(instructions.contains("synth_find"));
    assert!(instructions.contains("development tool"));
    server.process(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
    let response = server.process(&json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"vibe_invoke","arguments":{"id":"hypermedia.locus.construct","args":{"shape":"circle"}}}}).to_string()).unwrap();
    assert_eq!(response["result"]["isError"],true);
    assert_eq!(response["result"]["structuredContent"]["error"]["code"],"wrong_execution_host");
}

#[test]
fn declared_output_recipes_execute_and_contracts_cover_all_150_ids() {
    let inventory = contracts::catalogue();
    assert_eq!(inventory["count"],150);
    for recipe in inventory["recipes"].as_array().unwrap() {
        let result = StandaloneSynthMcp::default().call("synth_produce_present", json!({"source":recipe["source"]})).unwrap();
        let resources=result["receipt"]["resources"].as_array().unwrap();
        let expected_key=recipe["resource_key"].as_str().expect("recipe declares its encoded output key");
        let selected=resources.iter().find(|resource|resource["key"]==expected_key).expect("declared recipe output exists");
        assert_eq!(selected["kind"],recipe["resource_kind"]);
        if recipe["id"]=="lossless_sequence" {
            let resources=result["receipt"]["resources"].as_array().unwrap();
            assert_eq!(resources.len(),3,"manifest and both PNG leaves must be durable outputs");
            let keys:std::collections::BTreeSet<_>=resources.iter().map(|r|r["key"].as_str().unwrap()).collect();
            assert_eq!(keys,std::collections::BTreeSet::from(["sequence.json","frames/000000.png","frames/000001.png"]));
            for key in ["frames/000000.png","frames/000001.png"] {
                let frame=resources.iter().find(|r|r["key"]==key).unwrap();
                assert_eq!(frame["kind"],"png");
                assert_eq!(frame["sha256"].as_str().unwrap().len(),64);
            }
        }
    }
}

#[test]
fn production16_profile_is_explicit_bounded_and_invalid_profiles_refuse() {
    let source=r#"let mut asset={files:[{key:"note.txt",kind:"txt",text:"bounded"}]};"#;
    let mut backend=StandaloneSynthMcp::default();
    let result=backend.call("synth_produce_present",json!({"source":source,"quality_profile":"production16"})).unwrap();
    assert_eq!(result["receipt"]["limits"]["resource_bytes"],128*1024*1024);
    assert_eq!(result["receipt"]["limits"]["total_resource_bytes"],2u64*1024*1024*1024);
    assert_eq!(result["receipt"]["limits"]["package_bytes"],2u64*1024*1024*1024+256*1024*1024);
    assert_eq!(result["receipt"]["limits"]["asset_work_units"],2_000_000_000);
    let resources=result["receipt"]["resources"].as_array().unwrap();
    assert_eq!(resources.len(),1);
    assert_eq!(resources[0]["key"],"note.txt");
    assert_eq!(resources[0]["bytes"],7);
    let authoring=backend.call("synth_produce_present",json!({"source":source,"quality_profile":"authoring"})).unwrap();
    assert_eq!(authoring["receipt"]["limits"]["total_resource_bytes"],2u64*1024*1024*1024);
    let defaulted=backend.call("synth_produce_present",json!({"source":source})).unwrap();
    assert_eq!(defaulted["receipt"]["limits"]["total_resource_bytes"],2u64*1024*1024*1024);
    assert_eq!(defaulted["receipt"]["quality_profile"],"production16");
    assert_eq!(backend.call("synth_produce_present",json!({"source":source,"quality_profile":"unlimited"})).unwrap_err().code,"invalid_arguments");
}


#[test]
fn full_operation_catalogue_fits_real_stdio_transport_and_smaller_budget_refuses_cleanly() {
    fn exchange(limits: ProtocolLimits) -> Value {
        let input = concat!(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"catalogue-budget-test","version":"1"}}}"#,
            "\n",
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            "\n",
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"synth_operations","arguments":{"detail":"full","limit":150}}}"#,
            "\n"
        );
        let mut wire=Vec::new();
        qualia_mcp::authoring::serve(std::io::Cursor::new(input.as_bytes()),&mut wire,StandaloneSynthMcp::default(),limits).unwrap();
        let replies:Vec<Value>=wire.split(|byte|*byte==b'\n').filter(|line|!line.is_empty())
            .map(|line|serde_json::from_slice(line).unwrap()).collect();
        assert_eq!(replies.len(),2,"initialized notification must not receive a response");
        let initialized=replies.iter().find(|reply|reply["id"]==1).unwrap();
        assert!(initialized.get("error").is_none(),"{initialized}");
        assert_eq!(initialized["result"]["protocolVersion"],qualia_mcp::authoring::AUTHORING_PROTOCOL_VERSION);
        for line in wire.split(|byte|*byte==b'\n').filter(|line|!line.is_empty()) {
            assert!(line.len()<=limits.max_message_bytes,"every success/error frame stays bounded");
        }
        replies.into_iter().find(|reply|reply["id"]==2).unwrap()
    }
    let reply=exchange(comprehensive_protocol_limits());
    assert!(reply.get("error").is_none(),"{reply}");
    let result=&reply["result"];
    assert_eq!(result["isError"],false);
    assert_eq!(result["structuredContent"]["count"],150);
    assert_eq!(result["structuredContent"]["returned"],150);
    assert_eq!(result["structuredContent"]["operations"].as_array().unwrap().len(),150);
    let text:Value=serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(text,result["structuredContent"],"both actual MCP payload representations remain complete");
    let small=ProtocolLimits{max_message_bytes:2*1024*1024,..comprehensive_protocol_limits()};
    let refused=exchange(small);
    assert_eq!(refused["error"]["code"],-32603);
    assert_eq!(refused["error"]["message"],"Result exceeds configured limit; inspect operation state");
    assert!(refused.get("result").is_none(),"no partial catalogue is advertised as a success");
    fn exchange_default(limits: ProtocolLimits) -> Value {
        let input = concat!(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"catalogue-budget-test","version":"1"}}}"#,
            "\n",
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            "\n",
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"synth_operations","arguments":{}}}"#,
            "\n"
        );
        let mut wire=Vec::new();
        qualia_mcp::authoring::serve(std::io::Cursor::new(input.as_bytes()),&mut wire,StandaloneSynthMcp::default(),limits).unwrap();
        let replies:Vec<Value>=wire.split(|byte|*byte==b'\n').filter(|line|!line.is_empty())
            .map(|line|serde_json::from_slice(line).unwrap()).collect();
        replies.into_iter().find(|reply|reply["id"]==2).unwrap()
    }
    let paged=exchange_default(small);
    assert!(paged.get("error").is_none(),"{paged}");
    assert_eq!(paged["result"]["structuredContent"]["count"],150);
    assert_eq!(paged["result"]["structuredContent"]["returned"],40);
    assert_eq!(paged["result"]["structuredContent"]["limit"],40);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn compile_q42_seals_n3_and_refuses_overwrite() {
    let fixture = Fixture::new();
    std::fs::write(fixture.0.join("taxon.n3"), b"@prefix ex: <http://example.org/> .\nex:Corymbia a ex:Taxon .\n").unwrap();
    let mut mcp = fixture.backend();
    let sealed = mcp.call("synth_compile_q42", json!({
        "source_path": "taxon.n3",
        "out_q42": "taxon.q42",
        "ingest_to_graph": true,
        "context_iri": "urn:ecopunk:test:taxon"
    })).unwrap();
    assert_eq!(sealed["quin_count"], 1);
    assert!(sealed["lex_terms"].as_u64().unwrap() >= 3);
    assert_eq!(sealed["sha256"].as_str().unwrap().len(), 64);
    assert!(Path::new(sealed["path"].as_str().unwrap()).is_file());
    assert!(sealed["graph_quin_count"].as_u64().unwrap() >= 1);
    let again = mcp.call("synth_compile_q42", json!({
        "source_path": "taxon.n3",
        "out_q42": "taxon.q42"
    })).unwrap_err();
    assert_eq!(again.code, "native_output_refused");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn compile_q42_processes_owl_and_queries_via_sparql_and_endpoint() {
    let fixture = Fixture::new();
    let fma_sample = br#"<?xml version="1.0"?>
<rdf:RDF xmlns="http://purl.org/sig/ont/fma.owl#"
     xml:base="http://purl.org/sig/ont/fma.owl"
     xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
     xmlns:owl="http://www.w3.org/2002/07/owl#"
     xmlns:xsd="http://www.w3.org/2001/XMLSchema#"
     xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
     xmlns:fma="http://purl.org/sig/ont/fma/">
    <owl:Ontology rdf:about="http://purl.org/sig/ont/fma.owl">
        <owl:versionInfo rdf:datatype="http://www.w3.org/2001/XMLSchema#string">5.0.0</owl:versionInfo>
        <fma:creator rdf:datatype="http://www.w3.org/2001/XMLSchema#string">Cornelius Rosse, MD DSc</fma:creator>
    </owl:Ontology>
    <owl:Class rdf:about="http://purl.org/sig/ont/fma/fma7088">
        <rdfs:label xml:lang="en">Heart</rdfs:label>
        <fma:FMAID rdf:datatype="http://www.w3.org/2001/XMLSchema#integer">7088</fma:FMAID>
    </owl:Class>
    <owl:Class rdf:about="http://purl.org/sig/ont/fma/fma3734">
        <rdfs:label xml:lang="en">Aorta</rdfs:label>
        <rdfs:subClassOf rdf:resource="http://purl.org/sig/ont/fma/fma7088"/>
    </owl:Class>
</rdf:RDF>"#;
    std::fs::write(fixture.0.join("fma-sample.owl"), fma_sample).unwrap();
    let mut mcp = fixture.backend();
    let sealed = mcp
        .call(
            "synth_compile_q42",
            json!({
                "source_path": "fma-sample.owl",
                "out_q42": "fma-sample.q42",
                "ingest_to_graph": true
            }),
        )
        .unwrap();
    assert_eq!(sealed["format"], "rdfxml");
    assert!(sealed["quin_count"].as_u64().unwrap() >= 8);
    assert!(sealed["lex_terms"].as_u64().unwrap() >= 8);

    // 1. Single-triple range query with FILTER(CONTAINS(LCASE(?label), "heart")) and relative vault_path
    let filtered = mcp
        .call(
            "query_sparql",
            json!({
                "vault_path": "fma-sample.q42",
                "query": "SELECT ?organ ?label WHERE { ?organ <http://www.w3.org/2000/01/rdf-schema#label> ?label . FILTER(CONTAINS(LCASE(?label), \"heart\")) }"
            }),
        )
        .unwrap();
    assert_eq!(filtered["returned"], 1);
    assert_eq!(
        filtered["bindings"][0]["?organ"],
        "http://purl.org/sig/ont/fma/fma7088"
    );

    // 2. Multi-triple JOIN query across rdf:type owl:Class and rdfs:label
    let joined = mcp
        .call(
            "query_sparql",
            json!({
                "vault_path": "fma-sample.q42",
                "query": "PREFIX owl: <http://www.w3.org/2002/07/owl#> PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#> SELECT ?cls ?label WHERE { ?cls a owl:Class . ?cls rdfs:label ?label }"
            }),
        )
        .unwrap();
    assert_eq!(joined["returned"], 2);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn bioportal_refactor_owl_enforces_no_commodification_and_emits_shacl_and_n3_rules() {
    let fixture = Fixture::new();
    let owl_sample = br#"<?xml version="1.0"?>
<rdf:RDF xmlns="http://purl.org/sig/ont/fma.owl#"
     xml:base="http://purl.org/sig/ont/fma.owl"
     xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
     xmlns:owl="http://www.w3.org/2002/07/owl#"
     xmlns:xsd="http://www.w3.org/2001/XMLSchema#"
     xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
     xmlns:fma="http://purl.org/sig/ont/fma/">
    <owl:ObjectProperty rdf:about="http://purl.org/sig/ont/fma/regional_part_of">
        <rdf:type rdf:resource="http://www.w3.org/2002/07/owl#TransitiveProperty"/>
    </owl:ObjectProperty>
    <owl:Class rdf:about="http://purl.org/sig/ont/fma/fma7088">
        <rdfs:label xml:lang="en">Heart</rdfs:label>
        <rdfs:subClassOf rdf:resource="http://www.w3.org/2002/07/owl#Thing"/>
    </owl:Class>
    <owl:Class rdf:about="http://purl.org/sig/ont/fma/fma7096">
        <rdfs:label xml:lang="en">Left ventricle</rdfs:label>
        <rdfs:subClassOf rdf:resource="http://purl.org/sig/ont/fma/fma7088"/>
        <rdfs:subClassOf>
            <owl:Restriction>
                <owl:onProperty rdf:resource="http://purl.org/sig/ont/fma/regional_part_of"/>
                <owl:someValuesFrom rdf:resource="http://purl.org/sig/ont/fma/fma7088"/>
            </owl:Restriction>
        </rdfs:subClassOf>
    </owl:Class>
    <owl:NamedIndividual rdf:about="http://purl.org/sig/ont/fma/commodified_specimen">
        <rdf:type rdf:resource="http://www.w3.org/2002/07/owl#Thing"/>
    </owl:NamedIndividual>
</rdf:RDF>"#;
    std::fs::write(fixture.0.join("fma-refactor.owl"), owl_sample).unwrap();
    let mut mcp = fixture.backend();
    let res = mcp
        .call(
            "synth_bioportal",
            json!({
                "action": "refactor_owl",
                "source_path": "fma-refactor.owl",
                "acronym": "FMA_TEST",
                "compile_q42": true
            }),
        )
        .unwrap();
    assert_eq!(res["status"], "ok");
    assert!(res["owl_thing_stripped"].as_u64().unwrap() >= 2);
    assert!(res["skos_concepts_minted"].as_u64().unwrap() >= 2);
    assert!(res["shacl_shapes_emitted"].as_u64().unwrap() >= 1);
    assert!(res["n3_rules_emitted"].as_u64().unwrap() >= 2);

    let refs_text = std::fs::read_to_string(res["refs_ttl"].as_str().unwrap()).unwrap();
    assert!(
        !refs_text.contains("http://www.w3.org/2002/07/owl#Thing"),
        "owl:Thing must be stripped under the No-Commodification-of-Life rule"
    );
    assert!(
        !refs_text.contains("http://www.w3.org/2002/07/owl#NamedIndividual"),
        "owl:NamedIndividual must be stripped"
    );
    assert!(
        refs_text.contains("http://www.w3.org/2004/02/skos/core#Concept"),
        "owl:Class must be refactored to skos:Concept linked reference"
    );
    let shapes_text = std::fs::read_to_string(res["shapes_shacl_ttl"].as_str().unwrap()).unwrap();
    assert!(shapes_text.contains("sh:NodeShape"));
    assert!(shapes_text.contains("sh:qualifiedValueShape"));
    let rules_text = std::fs::read_to_string(res["rules_n3"].as_str().unwrap()).unwrap();
    assert!(rules_text.contains("=>"));
    let shex_text = std::fs::read_to_string(res["shex"].as_str().unwrap()).unwrap();
    assert!(shex_text.contains("ABSTRACT <BiologicalEntity>"));
    assert!(shex_text.contains("@<LoadedHmcPayload> OR @<WireframeStub>"));

    let q42_path = res["q42"]["path"].as_str().unwrap();
    let queried = mcp
        .call(
            "query_sparql",
            json!({
                "vault_path": q42_path,
                "query": "PREFIX skos: <http://www.w3.org/2004/02/skos/core#> PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#> SELECT ?c ?l WHERE { ?c a skos:Concept . ?c rdfs:label ?l }"
            }),
        )
        .unwrap();
    assert_eq!(queried["returned"], 2);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn biosphere_produce_without_evidence_is_blocked_and_ungated_produce_still_runs() {
    let fixture = Fixture::new();
    let mut mcp = fixture.backend();
    let blocked = mcp.call("synth_produce_present", json!({
        "source": r#"let mut asset = {files:[{key:"ok.bin",kind:"bin",bytes:[1]}]};"#,
        "domain": "biosphere",
        "taxon": "corymbia-aparrerinja",
        "fiction": false
    })).unwrap_err();
    assert_eq!(blocked.code, "production_rule_blocked");

    let ok = mcp.call("synth_produce_present", json!({
        "source": r#"let mut asset = {files:[{key:"ok.bin",kind:"bin",bytes:[1]}]};"#
    })).unwrap();
    assert!(ok["receipt"].is_object());

    let rules = mcp.call("synth_production_rules", json!({
        "domain": "biosphere",
        "action": "compile_q42"
    })).unwrap();
    assert_eq!(rules["decision"]["allowed"], true);
    assert_eq!(rules["decision"]["encyclopedia"], "ecopunk");

    let status = mcp.call("synth_operator_status", json!({})).unwrap();
    assert_eq!(status["schema"], "synth.operator-status/v1");
    assert_eq!(status["version"], crate::agent_index::RELEASE_VERSION);
    assert_eq!(status["role"], crate::agent_index::ROLE);
    assert_eq!(status["encyclopedia"], "ecopunk");
    assert_eq!(status["update"]["auto_replace"], false);
    assert!(status["update"]["github"].is_null());
    assert_eq!(
        status["update"]["signing"]["context"],
        "synth.release.sha256.v1"
    );

    let check = mcp.call("synth_update", json!({"action":"check"})).unwrap();
    assert_eq!(check["auto_replace"], false);
    assert_eq!(check["signing"]["public_key_hex"].as_str().unwrap().len(), 64);
    assert!(check["github"].is_object());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn bookmarks_tool_upserts_and_refuses_inline_secrets_on_upsert() {
    let root = std::env::temp_dir().join(format!(
        "synth-mcp-bookmarks-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::env::set_var("SYNTH_BOOKMARK_ROOT", root.to_str().unwrap());
    let mut mcp = StandaloneSynthMcp::default();
    let created = mcp.call("synth_bookmarks", json!({
        "action": "upsert",
        "url": "https://example.org/ontologies/east-coast.ttl",
        "title": "East coast taxa"
    })).unwrap();
    assert_eq!(created["item"]["kind"], "rdf");
    assert_eq!(created["item"]["data_endpoint"], "https://example.org/ontologies/east-coast.ttl");
    let err = mcp.call("synth_bookmarks", json!({
        "action": "upsert",
        "url": "https://example.org/other.ttl",
        "secret": "leak"
    })).unwrap_err();
    assert_eq!(err.code, "bookmark_failed");
    let listed = mcp.call("synth_bookmarks", json!({"action":"list"})).unwrap();
    assert!(!listed.to_string().contains("leak"));
    std::env::remove_var("SYNTH_BOOKMARK_ROOT");
    let _ = std::fs::remove_dir_all(&root);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn jobs_tool_adds_compile_job_and_seals_local_turtle() {
    let stamp = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let root = std::env::temp_dir().join(format!("synth-mcp-jobs-{stamp}"));
    let input = root.join("in");
    let output = root.join("out");
    std::fs::create_dir_all(&input).unwrap();
    std::fs::create_dir_all(&output).unwrap();
    std::fs::write(input.join("taxon.ttl"), b"@prefix ex: <http://example.org/> .\nex:Job a ex:Taxon .\n").unwrap();
    std::env::set_var("SYNTH_BOOKMARK_ROOT", root.to_str().unwrap());
    std::env::set_var("SYNTH_INPUT_ROOT", input.to_str().unwrap());
    std::env::set_var("SYNTH_OUTPUT_ROOT", output.to_str().unwrap());
    let mut mcp = StandaloneSynthMcp::default();
    let created = mcp.call("synth_jobs", json!({
        "action": "add",
        "id": "mcp-local",
        "kind": "compile",
        "source_path": "taxon.ttl",
        "title": "MCP local compile"
    })).unwrap();
    assert_eq!(created["item"]["kind"], "compile");
    let ran = mcp.call("synth_jobs", json!({
        "action": "run",
        "id": "mcp-local"
    })).unwrap();
    assert_eq!(ran["ok"], true);
    assert!(ran["quin_count"].as_u64().unwrap() >= 1);
    std::env::remove_var("SYNTH_BOOKMARK_ROOT");
    std::env::remove_var("SYNTH_INPUT_ROOT");
    std::env::remove_var("SYNTH_OUTPUT_ROOT");
    let _ = std::fs::remove_dir_all(&root);
}
