//! BioPortal REST API integration & No-Commodification-of-Life OWL -> Linked References + SHACL + N3Logic
//! streaming refactoring pipeline for Webizen AI Synth + QualiaDB.
//!
//! Implements the architecture specified in:
//! `C:\github\ontology\biomedical\BioPortal Ontology N3Logic and SHACL Pipeline.md`
//!
//! - Uses `Authorization: apikey token=YOUR_KEY` against `https://data.bioontology.org`
//!   (resolved from gitignored `.synth/secrets.json` entry `bioportal` or `BIOPORTAL_API_KEY`).
//! - Strips `owl:Thing`, `owl:Nothing`, and `owl:NamedIndividual` universal object-reification roots.
//! - Converts `owl:Class` into dereferenceable `skos:Concept` linked references (`_refs.ttl`).
//! - Converts `owl:Restriction` blank nodes into closed/contextual-world SHACL shapes (`_shapes.shacl.ttl`).
//! - Compiles class subsumption, transitive partonomies, symmetric/inverse properties into N3Logic (`_rules.n3`).
//! - Seals the resulting knowledge graph into a canonical `.q42` volume via QualiaDB's streaming engine.

use super::*;
use crate::operator::bookmarks;
use qualia_core_db::ingest::{streaming_import_rdf_with_report, IngestMode, IngestReport};
use qualia_core_db::query::ingest_formats::parse_triples_format;
use qualia_core_db::query::ingest_job::infer_rdf_format;
use qualia_mcp::authoring::ToolError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const BIOPORTAL_API_BASE: &str = "https://data.bioontology.org";
pub const DEFAULT_BIOMEDICAL_DIR: &str = r"C:\github\ontology\biomedical";

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
const RDFS_COMMENT: &str = "http://www.w3.org/2000/01/rdf-schema#comment";
const RDFS_SUBCLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
const RDFS_SUBPROPERTY_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subPropertyOf";
const RDFS_IS_DEFINED_BY: &str = "http://www.w3.org/2000/01/rdf-schema#isDefinedBy";
const OWL_THING: &str = "http://www.w3.org/2002/07/owl#Thing";
const OWL_NOTHING: &str = "http://www.w3.org/2002/07/owl#Nothing";
const OWL_NAMED_INDIVIDUAL: &str = "http://www.w3.org/2002/07/owl#NamedIndividual";
const OWL_CLASS: &str = "http://www.w3.org/2002/07/owl#Class";
const OWL_RESTRICTION: &str = "http://www.w3.org/2002/07/owl#Restriction";
const OWL_ON_PROPERTY: &str = "http://www.w3.org/2002/07/owl#onProperty";
const OWL_SOME_VALUES_FROM: &str = "http://www.w3.org/2002/07/owl#someValuesFrom";
const OWL_ALL_VALUES_FROM: &str = "http://www.w3.org/2002/07/owl#allValuesFrom";
const OWL_HAS_VALUE: &str = "http://www.w3.org/2002/07/owl#hasValue";
const OWL_MIN_CARDINALITY: &str = "http://www.w3.org/2002/07/owl#minCardinality";
const OWL_MIN_Q_CARDINALITY: &str = "http://www.w3.org/2002/07/owl#minQualifiedCardinality";
const OWL_MAX_CARDINALITY: &str = "http://www.w3.org/2002/07/owl#maxCardinality";
const OWL_MAX_Q_CARDINALITY: &str = "http://www.w3.org/2002/07/owl#maxQualifiedCardinality";
const OWL_CARDINALITY: &str = "http://www.w3.org/2002/07/owl#cardinality";
const OWL_Q_CARDINALITY: &str = "http://www.w3.org/2002/07/owl#qualifiedCardinality";
const OWL_DISJOINT_WITH: &str = "http://www.w3.org/2002/07/owl#disjointWith";
const OWL_EQUIVALENT_CLASS: &str = "http://www.w3.org/2002/07/owl#equivalentClass";
const OWL_TRANSITIVE_PROP: &str = "http://www.w3.org/2002/07/owl#TransitiveProperty";
const OWL_SYMMETRIC_PROP: &str = "http://www.w3.org/2002/07/owl#SymmetricProperty";
const OWL_INVERSE_OF: &str = "http://www.w3.org/2002/07/owl#inverseOf";
const SKOS_CONCEPT: &str = "http://www.w3.org/2004/02/skos/core#Concept";
const SKOS_BROADER: &str = "http://www.w3.org/2004/02/skos/core#broader";

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct BioPortalArg {
    pub action: Option<String>,
    pub acronym: Option<String>,
    pub q: Option<String>,
    pub source_path: Option<String>,
    pub out_dir: Option<String>,
    pub out_q42: Option<String>,
    pub class_iri: Option<String>,
    pub download_format: Option<String>,
    pub compile_q42: Option<bool>,
    pub refactor_owl: Option<bool>,
    pub ingest_to_graph: Option<bool>,
    pub segment_mib: Option<u64>,
    pub limit: Option<usize>,
    pub api_key: Option<String>,
    pub biomedical_dir: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BNodeRestriction {
    is_restriction: bool,
    on_property: Option<String>,
    some_values_from: Option<String>,
    all_values_from: Option<String>,
    has_value: Option<String>,
    min_count: Option<u64>,
    max_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OwlRefactorStats {
    pub acronym: String,
    pub source_path: String,
    pub total_triples_scanned: u64,
    pub owl_thing_purged: u64,
    pub classes_converted_to_skos_concepts: u64,
    pub linked_reference_triples: u64,
    pub shacl_node_shapes: u64,
    pub shacl_property_shapes: u64,
    pub n3_subsumption_rules: u64,
    pub n3_property_algebra_rules: u64,
    pub refs_ttl_path: String,
    pub shacl_ttl_path: String,
    pub n3_rules_path: String,
    pub shex_path: String,
    pub q42_path: Option<String>,
    pub q42_quins_written: Option<u64>,
    pub q42_lex_terms: Option<usize>,
}

/// Resolve the BioPortal API key from `.synth/secrets.json` (`bioportal`) or `BIOPORTAL_API_KEY`.
pub fn resolve_bioportal_api_key() -> Option<String> {
    if let Ok(Some(sec)) = bookmarks::secret_value("bioportal") {
        if !sec.trim().is_empty() {
            return Some(sec.trim().to_owned());
        }
    }
    if let Ok(env_val) = std::env::var("BIOPORTAL_API_KEY") {
        if !env_val.trim().is_empty() {
            return Some(env_val.trim().to_owned());
        }
    }
    for candidate in [
        r"C:\github\synth\.synth\secrets.json",
        r"C:\github\synth\production\.synth\secrets.json",
    ] {
        if let Ok(bytes) = fs::read(candidate) {
            if let Ok(val) = serde_json::from_slice::<Value>(&bytes) {
                if let Some(s) = val
                    .get("keys")
                    .and_then(|k| k.get("bioportal"))
                    .and_then(|e| e.get("value"))
                    .and_then(Value::as_str)
                {
                    if !s.trim().is_empty() {
                        return Some(s.trim().to_owned());
                    }
                }
            }
        }
    }
    None
}

/// Save a BioPortal API key into `.synth/secrets.json` (gitignored) and ensure the `bioportal` bookmark exists.
pub fn store_bioportal_api_key(secret: &str) -> Result<Value, String> {
    let trimmed = secret.trim();
    if trimmed.is_empty() {
        return Err("API key must not be empty".into());
    }
    let _ = bookmarks::dispatch(bookmarks::BookmarkArg {
        action: Some("upsert".into()),
        id: Some("bioportal".into()),
        url: Some("https://bioportal.bioontology.org/ontologies".into()),
        data_endpoint: Some(BIOPORTAL_API_BASE.into()),
        kind: Some("rest".into()),
        title: Some("NCBO BioPortal Biomedical Ontology Repository".into()),
        requires_api_key: Some(true),
        api_key_env: Some("BIOPORTAL_API_KEY".into()),
        auth: Some("apikey_token".into()),
        domain: Some("biomedical".into()),
        formats: Some(vec![
            "rdfxml".into(),
            "owl".into(),
            "ttl".into(),
            "n3".into(),
            "jsonld".into(),
        ]),
        ..bookmarks::BookmarkArg::default()
    });
    bookmarks::dispatch(bookmarks::BookmarkArg {
        action: Some("set_secret".into()),
        id: Some("bioportal".into()),
        secret: Some(trimmed.to_owned()),
        api_key_env: Some("BIOPORTAL_API_KEY".into()),
        auth: Some("apikey_token".into()),
        ..bookmarks::BookmarkArg::default()
    })
}

pub fn dispatch_mcp(mcp: &StandaloneSynthMcp, args: Value) -> Result<Value, ToolError> {
    let parsed: BioPortalArg = serde_json::from_value(args)
        .map_err(|e| ToolError::new("invalid_arguments", format!("Invalid synth_bioportal arguments: {e}")))?;
    dispatch(Some(mcp), parsed).map_err(|e| ToolError::new("bioportal_failed", e))
}

pub fn dispatch(
    mcp: Option<&StandaloneSynthMcp>,
    arg: BioPortalArg,
) -> Result<Value, String> {
    let action = arg
        .action
        .as_deref()
        .unwrap_or("status")
        .trim()
        .to_ascii_lowercase();
    match action.as_str() {
        "status" => {
            let has_key = resolve_bioportal_api_key().is_some();
            let dir = arg
                .biomedical_dir
                .as_deref()
                .unwrap_or(DEFAULT_BIOMEDICAL_DIR);
            let local = list_local_biomedical(Path::new(dir)).unwrap_or_else(|_| json!([]));
            Ok(json!({
                "schema": "synth.bioportal/v1",
                "endpoint": BIOPORTAL_API_BASE,
                "auth_header_format": "Authorization: apikey token=<BIOPORTAL_API_KEY>",
                "api_key_configured": has_key,
                "biomedical_dir": dir,
                "local_ontologies": local,
            }))
        }
        "set_api_key" | "set_secret" => {
            let key = arg
                .api_key
                .as_deref()
                .ok_or("set_api_key requires api_key")?;
            store_bioportal_api_key(key)
        }
        "list_local" => {
            let dir = arg
                .biomedical_dir
                .as_deref()
                .unwrap_or(DEFAULT_BIOMEDICAL_DIR);
            let items = list_local_biomedical(Path::new(dir))?;
            Ok(json!({
                "schema": "synth.bioportal.local/v1",
                "biomedical_dir": dir,
                "count": items.as_array().map(Vec::len).unwrap_or(0),
                "ontologies": items,
            }))
        }
        "list_remote" | "search" => {
            let limit = arg.limit.unwrap_or(50).min(500);
            let filter = arg.q.as_deref().or(arg.acronym.as_deref());
            list_remote_bioportal_ontologies(filter, limit)
        }
        "download" => {
            let acronym = arg
                .acronym
                .as_deref()
                .ok_or("download requires acronym (e.g. FMA, UBERON, MONDO, HP, CHEBI, GO)")?;
            download_bioportal_ontology(mcp, acronym, &arg)
        }
        "mappings" => {
            let acronym = arg
                .acronym
                .as_deref()
                .ok_or("mappings requires acronym")?;
            let class_iri = arg
                .class_iri
                .as_deref()
                .ok_or("mappings requires class_iri")?;
            fetch_bioportal_class_mappings(acronym, class_iri)
        }
        "refactor_owl" | "refactor" => {
            let src = resolve_ontology_source(mcp, &arg)?;
            let acronym = arg
                .acronym
                .clone()
                .unwrap_or_else(|| infer_acronym_from_path(&src));
            let out_dir = resolve_output_dir(mcp, &arg, &src, &acronym)?;
            let compile_q42 = arg.compile_q42.unwrap_or(true);
            let ingest = arg.ingest_to_graph.unwrap_or(false);
            let stats = refactor_owl_to_linked_shacl_n3(
                &src,
                &acronym,
                &out_dir,
                arg.out_q42.as_deref(),
                compile_q42,
                ingest,
                arg.segment_mib,
            )?;
            Ok(json!({
                "schema": "synth.bioportal.refactor-owl/v1",
                "status": "ok",
                "policy": "No-commodification-of-life rule enforced: owl:Thing, owl:Nothing, and owl:NamedIndividual stripped; classes converted to skos:Concept linked references; owl:Restriction blank nodes converted to SHACL shapes; subsumption & transitive partonomies compiled to N3Logic; cyclic/streaming ShEx grammar emitted.",
                "owl_thing_stripped": stats.owl_thing_purged,
                "skos_concepts_minted": stats.classes_converted_to_skos_concepts,
                "shacl_shapes_emitted": stats.shacl_node_shapes + stats.shacl_property_shapes,
                "n3_rules_emitted": stats.n3_subsumption_rules + stats.n3_property_algebra_rules,
                "refs_ttl": stats.refs_ttl_path,
                "shapes_shacl_ttl": stats.shacl_ttl_path,
                "rules_n3": stats.n3_rules_path,
                "shex": stats.shex_path,
                "q42": {
                    "path": stats.q42_path,
                    "quin_count": stats.q42_quins_written,
                    "lex_terms": stats.q42_lex_terms,
                },
                "stats": stats,
            }))
        }
        "compile_biomedical" => {
            compile_local_biomedical_catalogue(mcp, &arg)
        }
        other => Err(format!(
            "Unknown synth_bioportal action `{other}` (use: status, set_api_key, list_local, list_remote, download, mappings, refactor_owl, compile_biomedical)"
        )),
    }
}

fn list_local_biomedical(root: &Path) -> Result<Value, String> {
    if !root.is_dir() {
        return Err(format!("Biomedical ontology directory not found: {}", root.display()));
    }
    let mut items = Vec::new();
    let entries = fs::read_dir(root).map_err(|e| format!("read_dir {}: {e}", root.display()))?;
    let mut subdirs: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    subdirs.sort();
    for dir in subdirs {
        let acronym = dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("ONT")
            .to_owned();
        let Ok(files) = fs::read_dir(&dir) else {
            continue;
        };
        for f in files.flatten() {
            let path = f.path();
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !matches!(
                ext.as_str(),
                "owl" | "rdf" | "xml" | "ttl" | "turtle" | "n3" | "nt" | "nq" | "trig" | "jsonld" | "q42"
            ) {
                continue;
            }
            let bytes = f.metadata().map(|m| m.len()).unwrap_or(0);
            let q42_sibling = path.with_extension("q42");
            let refs_sibling = dir.join(format!("{acronym}_refs.ttl"));
            let shacl_sibling = dir.join(format!("{acronym}_shapes.shacl.ttl"));
            let n3_sibling = dir.join(format!("{acronym}_rules.n3"));
            items.push(json!({
                "acronym": acronym,
                "path": path.display().to_string(),
                "format": ext,
                "bytes": bytes,
                "mib": (bytes as f64 / 1_048_576.0 * 100.0).round() / 100.0,
                "q42_present": q42_sibling.is_file(),
                "refactored_present": refs_sibling.is_file() && shacl_sibling.is_file() && n3_sibling.is_file(),
            }));
        }
    }
    Ok(Value::Array(items))
}

fn bioportal_get(url: &str, accept: &str) -> Result<ureq::Response, String> {
    let api_key = resolve_bioportal_api_key().ok_or_else(|| {
        "BioPortal API key not configured. Call synth_bioportal action=set_api_key or set BIOPORTAL_API_KEY.".to_string()
    })?;
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(120))
        .redirects(5)
        .user_agent("Webizen-AI-Synth/0.1 (+https://github.com/mediaprophet/synth)")
        .build();
    agent
        .get(url)
        .set("Authorization", &format!("apikey token={api_key}"))
        .set("Accept", accept)
        .call()
        .map_err(|e| format!("BioPortal request to {url} failed: {e}"))
}

fn list_remote_bioportal_ontologies(
    filter: Option<&str>,
    limit: usize,
) -> Result<Value, String> {
    let url = format!("{BIOPORTAL_API_BASE}/ontologies");
    let resp = bioportal_get(&url, "application/json")?;
    let text = resp
        .into_string()
        .map_err(|e| format!("Failed reading BioPortal response: {e}"))?;
    let parsed: Value =
        serde_json::from_str(&text).map_err(|e| format!("Invalid BioPortal JSON: {e}"))?;
    let arr = parsed
        .as_array()
        .ok_or_else(|| "Expected JSON array from BioPortal /ontologies".to_string())?;
    let needle = filter.map(|s| s.to_ascii_lowercase());
    let mut matches = Vec::new();
    for item in arr {
        let acronym = item
            .get("acronym")
            .and_then(Value::as_str)
            .unwrap_or("");
        let name = item.get("name").and_then(Value::as_str).unwrap_or("");
        if let Some(ref q) = needle {
            if !acronym.to_ascii_lowercase().contains(q) && !name.to_ascii_lowercase().contains(q)
            {
                continue;
            }
        }
        matches.push(json!({
            "acronym": acronym,
            "name": name,
            "id": item.get("@id").and_then(Value::as_str).unwrap_or(""),
            "ontologyType": item.get("ontologyType").and_then(Value::as_str).unwrap_or(""),
            "download_url": format!("{BIOPORTAL_API_BASE}/ontologies/{acronym}/download?download_format=rdf"),
        }));
        if matches.len() >= limit {
            break;
        }
    }
    Ok(json!({
        "schema": "synth.bioportal.remote-list/v1",
        "total_catalog_ontologies": arr.len(),
        "returned": matches.len(),
        "filter": filter,
        "ontologies": matches,
    }))
}

fn download_bioportal_ontology(
    mcp: Option<&StandaloneSynthMcp>,
    acronym: &str,
    arg: &BioPortalArg,
) -> Result<Value, String> {
    let clean_acr: String = acronym
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if clean_acr.is_empty() {
        return Err("Invalid ontology acronym".into());
    }
    let dl_fmt = arg.download_format.as_deref().unwrap_or("rdf");
    let url = if dl_fmt == "native" || dl_fmt.is_empty() {
        format!("{BIOPORTAL_API_BASE}/ontologies/{clean_acr}/download")
    } else {
        format!("{BIOPORTAL_API_BASE}/ontologies/{clean_acr}/download?download_format={dl_fmt}")
    };

    let resp = bioportal_get(
        &url,
        "application/rdf+xml, text/turtle;q=0.9, application/owl+xml;q=0.9, */*;q=0.5",
    )?;
    let content_type = resp
        .header("Content-Type")
        .unwrap_or("application/rdf+xml")
        .to_owned();
    let disp = resp.header("Content-Disposition").unwrap_or("").to_owned();

    let ext = if disp.to_ascii_lowercase().contains(".ttl")
        || content_type.to_ascii_lowercase().contains("turtle")
    {
        "ttl"
    } else if disp.to_ascii_lowercase().contains(".n3")
        || content_type.to_ascii_lowercase().contains("n3")
    {
        "n3"
    } else {
        "owl"
    };

    let base_dir = if let Some(out) = arg.out_dir.as_deref() {
        PathBuf::from(out)
    } else {
        let bio_root = Path::new(
            arg.biomedical_dir
                .as_deref()
                .unwrap_or(DEFAULT_BIOMEDICAL_DIR),
        );
        if bio_root.is_dir() {
            bio_root.join(&clean_acr)
        } else if let Ok(out_root) = std::env::var("SYNTH_OUTPUT_ROOT") {
            Path::new(out_root.trim()).join(&clean_acr)
        } else {
            PathBuf::from(&clean_acr)
        }
    };
    fs::create_dir_all(&base_dir).map_err(|e| format!("mkdir {}: {e}", base_dir.display()))?;
    let file_path = base_dir.join(format!("{}.{ext}", clean_acr.to_ascii_lowercase()));

    let mut reader = resp.into_reader();
    let mut out_file = BufWriter::new(
        File::create(&file_path)
            .map_err(|e| format!("create {}: {e}", file_path.display()))?,
    );
    let mut hasher = Sha256::new();
    let mut total_bytes = 0u64;
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("download stream read error: {e}"))?;
        if n == 0 {
            break;
        }
        total_bytes += n as u64;
        if total_bytes > 4 * 1024 * 1024 * 1024 {
            return Err("Downloaded ontology exceeds 4 GiB safety cap".into());
        }
        hasher.update(&buf[..n]);
        out_file
            .write_all(&buf[..n])
            .map_err(|e| format!("write {}: {e}", file_path.display()))?;
    }
    out_file.flush().map_err(|e| e.to_string())?;
    let sha = format!("{:x}", hasher.finalize());

    let mut refactor_result = None;
    if arg.refactor_owl.unwrap_or(true) {
        let stats = refactor_owl_to_linked_shacl_n3(
            &file_path,
            &clean_acr,
            &base_dir,
            arg.out_q42.as_deref(),
            arg.compile_q42.unwrap_or(true),
            arg.ingest_to_graph.unwrap_or(false),
            arg.segment_mib,
        )?;
        refactor_result = Some(stats);
    } else if arg.compile_q42.unwrap_or(false) {
        let q42_dest = match arg.out_q42.as_deref() {
            Some(name) => resolve_q42_out(mcp, &base_dir, name),
            None => base_dir.join(format!("{}.q42", clean_acr.to_ascii_lowercase())),
        };
        let fmt = super::q42::infer_compile_format(None, Some(&file_path.to_string_lossy()))
            .map_err(|e| e.message)?;
        let (quins, lex_terms, q42_sha) = super::q42::compile_file_to_q42(
            &file_path,
            total_bytes,
            &q42_dest,
            fmt,
            Some(&format!(
                "https://bioportal.bioontology.org/ontologies/{clean_acr}"
            )),
            arg.ingest_to_graph.unwrap_or(false),
            arg.segment_mib,
            false,
        )
        .map_err(|e| e.message)?;
        return Ok(json!({
            "schema": "synth.bioportal.download/v1",
            "acronym": clean_acr,
            "source_url": url,
            "content_type": content_type,
            "downloaded_path": file_path.display().to_string(),
            "bytes": total_bytes,
            "sha256": sha,
            "q42_path": q42_dest.display().to_string(),
            "q42_quins": quins,
            "q42_lex_terms": lex_terms,
            "q42_sha256": q42_sha,
        }));
    }

    Ok(json!({
        "schema": "synth.bioportal.download/v1",
        "acronym": clean_acr,
        "source_url": url,
        "content_type": content_type,
        "downloaded_path": file_path.display().to_string(),
        "bytes": total_bytes,
        "sha256": sha,
        "refactor_owl": refactor_result,
    }))
}

fn percent_encode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn fetch_bioportal_class_mappings(acronym: &str, class_iri: &str) -> Result<Value, String> {
    let clean_acr: String = acronym
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    let encoded_cls = percent_encode_component(class_iri);
    let url =
        format!("{BIOPORTAL_API_BASE}/ontologies/{clean_acr}/classes/{encoded_cls}/mappings");
    let resp = bioportal_get(&url, "application/json")?;
    let text = resp
        .into_string()
        .map_err(|e| format!("Failed reading BioPortal mappings: {e}"))?;
    let parsed: Value =
        serde_json::from_str(&text).map_err(|e| format!("Invalid BioPortal mappings JSON: {e}"))?;

    let mut skos_triples = Vec::new();
    if let Some(arr) = parsed.as_array() {
        for m in arr {
            let source = m
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("SKOS");
            let pred = if source.eq_ignore_ascii_case("LOOM") {
                "http://www.w3.org/2004/02/skos/core#closeMatch"
            } else {
                "http://www.w3.org/2004/02/skos/core#exactMatch"
            };
            if let Some(classes) = m.get("classes").and_then(Value::as_array) {
                for c in classes {
                    if let Some(target_id) = c.get("@id").and_then(Value::as_str) {
                        if target_id != class_iri {
                            skos_triples.push(format!("<{class_iri}> <{pred}> <{target_id}> ."));
                        }
                    }
                }
            }
        }
    }

    Ok(json!({
        "schema": "synth.bioportal.mappings/v1",
        "acronym": clean_acr,
        "class_iri": class_iri,
        "mapping_triples_count": skos_triples.len(),
        "skos_turtle": skos_triples.join("\n"),
        "raw_mappings": parsed,
    }))
}

fn resolve_ontology_source(
    mcp: Option<&StandaloneSynthMcp>,
    arg: &BioPortalArg,
) -> Result<PathBuf, String> {
    if let Some(raw) = arg.source_path.as_deref() {
        let p = PathBuf::from(raw);
        if p.is_file() {
            return Ok(p);
        }
        if let Some(m) = mcp {
            if let Some(in_root) = m.input_root.as_ref() {
                if let Ok((canon, _)) = in_root.resolve_path(raw, 4 * 1024 * 1024 * 1024) {
                    return Ok(canon);
                }
            }
        }
        let resolved = qualia_mcp::tools::resolve_audit_path(Some(raw));
        if resolved.is_file() {
            return Ok(resolved);
        }
        return Err(format!("Ontology source_path not found: {raw}"));
    }
    if let Some(acr) = arg.acronym.as_deref() {
        let bio_root = Path::new(
            arg.biomedical_dir
                .as_deref()
                .unwrap_or(DEFAULT_BIOMEDICAL_DIR),
        );
        for candidate_dir in [
            bio_root.join(acr),
            bio_root.join(acr.to_ascii_uppercase()),
            bio_root.join(acr.to_ascii_lowercase()),
        ] {
            if candidate_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&candidate_dir) {
                    for e in entries.flatten() {
                        let p = e.path();
                        let ext = p
                            .extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_ascii_lowercase();
                        if matches!(ext.as_str(), "owl" | "rdf" | "xml" | "ttl")
                            && !p
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .contains("_refs.")
                            && !p
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .contains("_shapes.")
                        {
                            return Ok(p);
                        }
                    }
                }
            }
        }
    }
    Err("Provide source_path or acronym of a local biomedical ontology".into())
}

fn infer_acronym_from_path(path: &Path) -> String {
    if let Some(parent_name) = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
    {
        if !parent_name.is_empty() && parent_name != "biomedical" && parent_name != "ontology" {
            return parent_name.to_ascii_uppercase();
        }
    }
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("ONT")
        .to_ascii_uppercase()
}

fn resolve_output_dir(
    mcp: Option<&StandaloneSynthMcp>,
    arg: &BioPortalArg,
    src: &Path,
    acronym: &str,
) -> Result<PathBuf, String> {
    if let Some(out) = arg.out_dir.as_deref() {
        let pb = PathBuf::from(out);
        if pb.is_absolute() {
            fs::create_dir_all(&pb).map_err(|e| e.to_string())?;
            return Ok(pb);
        }
        if let Some(m) = mcp {
            if let Some(root) = m.output_root.as_ref() {
                if let Ok(dest) = root.destination_with_ext(out, "", "invalid out_dir") {
                    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
                    return Ok(dest);
                }
            }
        }
        fs::create_dir_all(&pb).map_err(|e| e.to_string())?;
        return Ok(pb);
    }
    if let Ok(out_root) = std::env::var("SYNTH_OUTPUT_ROOT") {
        let trimmed = out_root.trim();
        if !trimmed.is_empty() {
            let dir = Path::new(trimmed).join(format!("biomedical-{}", acronym.to_ascii_lowercase()));
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            return Ok(dir);
        }
    }
    let dir = src
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn resolve_q42_out(mcp: Option<&StandaloneSynthMcp>, out_dir: &Path, name: &str) -> PathBuf {
    let pb = PathBuf::from(name);
    if pb.is_absolute() {
        return pb;
    }
    if let Some(m) = mcp {
        if let Some(root) = m.output_root.as_ref() {
            if let Ok(dest) = root.destination_with_ext(name, ".q42", "invalid out_q42") {
                return dest;
            }
        }
    }
    out_dir.join(name)
}

fn is_iri(term: &str) -> bool {
    !term.starts_with("_:")
        && !term.contains(|c: char| {
            c.is_whitespace() || matches!(c, '<' | '>' | '"' | '{' | '}' | '|' | '^' | '`' | '\\')
        })
        && (term.starts_with("http://")
            || term.starts_with("https://")
            || term.starts_with("urn:")
            || term.starts_with("did:")
            || term.starts_with("obo:"))
}

fn escape_turtle_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

fn format_turtle_object(obj: &str, packed: Option<u64>) -> String {
    if is_iri(obj) {
        return format!("<{obj}>");
    }
    if obj.starts_with('"') && obj.contains("\"^^<") {
        return obj.to_owned();
    }
    if let Some(at_idx) = obj.rfind('@') {
        let lang = &obj[at_idx + 1..];
        let valid_bcp47 = !lang.is_empty()
            && lang.len() <= 16
            && !obj[..at_idx].contains("://")
            && lang
                .split('-')
                .next()
                .is_some_and(|primary| (2..=3).contains(&primary.len()) && primary.chars().all(|c| c.is_ascii_alphabetic()))
            && lang
                .split('-')
                .all(|sub| !sub.is_empty() && sub.len() <= 8 && sub.chars().all(|c| c.is_ascii_alphanumeric()));
        if valid_bcp47 {
            return format!("\"{}\"@{lang}", escape_turtle_literal(&obj[..at_idx]));
        }
    }
    if packed.is_some() {
        if obj.parse::<i64>().is_ok() || obj == "true" || obj == "false" {
            return obj.to_owned();
        }
    }
    format!("\"{}\"", escape_turtle_literal(obj))
}

/// Streaming OWL -> Linked References (`_refs.ttl`) + SHACL Shapes (`_shapes.shacl.ttl`)
/// + N3Logic Rules (`_rules.n3`) + sealed `.q42` volume pipeline.
pub fn refactor_owl_to_linked_shacl_n3(
    source_path: &Path,
    acronym: &str,
    out_dir: &Path,
    explicit_out_q42: Option<&str>,
    compile_q42: bool,
    ingest_to_graph: bool,
    segment_mib: Option<u64>,
) -> Result<OwlRefactorStats, String> {
    fs::create_dir_all(out_dir).map_err(|e| format!("mkdir {}: {e}", out_dir.display()))?;
    let acr_upper = acronym.to_ascii_uppercase();
    let acr_lower = acronym.to_ascii_lowercase();
    let bioportal_ont_iri = format!("https://bioportal.bioontology.org/ontologies/{acr_upper}");

    let refs_path = out_dir.join(format!("{acr_lower}_refs.ttl"));
    let shacl_path = out_dir.join(format!("{acr_lower}_shapes.shacl.ttl"));
    let n3_path = out_dir.join(format!("{acr_lower}_rules.n3"));
    let shex_path = out_dir.join(format!("{acr_lower}_shapes.shex"));

    let mut refs_out = BufWriter::with_capacity(
        256 * 1024,
        File::create(&refs_path).map_err(|e| format!("create {}: {e}", refs_path.display()))?,
    );
    let mut n3_out = BufWriter::with_capacity(
        256 * 1024,
        File::create(&n3_path).map_err(|e| format!("create {}: {e}", n3_path.display()))?,
    );

    writeln!(
        refs_out,
        "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix skos: <http://www.w3.org/2004/02/skos/core#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n"
    )
    .map_err(|e| e.to_string())?;

    writeln!(
        n3_out,
        "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix skos: <http://www.w3.org/2004/02/skos/core#> .\n\
         @prefix log: <http://www.w3.org/2000/10/swap/log#> .\n"
    )
    .map_err(|e| e.to_string())?;

    let fmt = infer_rdf_format(&source_path.to_string_lossy());
    let file = File::open(source_path)
        .map_err(|e| format!("cannot open {}: {e}", source_path.display()))?;
    let reader = BufReader::with_capacity(256 * 1024, file);

    let mut total_triples = 0u64;
    let mut owl_thing_purged = 0u64;
    let mut classes_converted = 0u64;
    let mut linked_ref_triples = 0u64;
    let mut n3_subsumption_rules = 0u64;
    let mut n3_property_rules = 0u64;

    let mut bnodes: HashMap<String, BNodeRestriction> = HashMap::new();
    let mut class_bnode_parents: Vec<(String, String)> = Vec::new();
    let mut class_disjoints: Vec<(String, String)> = Vec::new();

    let default_base = qualia_core_db::ingest::catalog_base_iri(source_path);
    parse_triples_format(fmt, reader, default_base, &mut |raw| {
        total_triples += 1;
        let s = raw.subject.as_str();
        let p = raw.predicate.as_str();
        let o = raw.object.as_str();

        // 1. No-Commodification-of-Life Rule: strip owl:Thing, owl:Nothing, owl:NamedIndividual
        if s == OWL_THING
            || s == OWL_NOTHING
            || o == OWL_THING
            || o == OWL_NOTHING
            || o == OWL_NAMED_INDIVIDUAL
        {
            owl_thing_purged += 1;
            return;
        }

        // 2. Accumulate blank-node OWL Restrictions for SHACL translation
        if s.starts_with("_:") {
            let entry = bnodes.entry(raw.subject.clone()).or_default();
            match p {
                RDF_TYPE if o == OWL_RESTRICTION => entry.is_restriction = true,
                OWL_ON_PROPERTY if is_iri(o) => entry.on_property = Some(raw.object),
                OWL_SOME_VALUES_FROM if is_iri(o) => entry.some_values_from = Some(raw.object),
                OWL_ALL_VALUES_FROM if is_iri(o) => entry.all_values_from = Some(raw.object),
                OWL_HAS_VALUE => entry.has_value = Some(raw.object),
                OWL_MIN_CARDINALITY | OWL_MIN_Q_CARDINALITY => {
                    entry.min_count = o.parse::<u64>().ok();
                }
                OWL_MAX_CARDINALITY | OWL_MAX_Q_CARDINALITY => {
                    entry.max_count = o.parse::<u64>().ok();
                }
                OWL_CARDINALITY | OWL_Q_CARDINALITY => {
                    if let Ok(c) = o.parse::<u64>() {
                        entry.min_count = Some(c);
                        entry.max_count = Some(c);
                    }
                }
                _ => {}
            }
            return;
        }

        if !is_iri(s) {
            return;
        }

        // 3. Convert owl:Class declarations on named IRIs into skos:Concept linked references
        if p == RDF_TYPE && o == OWL_CLASS {
            classes_converted += 1;
            let _ = writeln!(
                refs_out,
                "<{s}> <{RDF_TYPE}> <{SKOS_CONCEPT}> .\n<{s}> <{RDFS_IS_DEFINED_BY}> <{bioportal_ont_iri}> ."
            );
            linked_ref_triples += 2;
            return;
        }

        // 4. Route rdfs:subClassOf / owl:equivalentClass
        if p == RDFS_SUBCLASS_OF || p == OWL_EQUIVALENT_CLASS {
            if o.starts_with("_:") {
                class_bnode_parents.push((raw.subject, raw.object));
            } else if is_iri(o) {
                n3_subsumption_rules += 1;
                let _ = writeln!(n3_out, "{{ ?entity a <{s}> }} => {{ ?entity a <{o}> }} .");
                let _ = writeln!(refs_out, "<{s}> <{SKOS_BROADER}> <{o}> .");
                linked_ref_triples += 1;
            }
            return;
        }

        // 5. Route owl:disjointWith to SHACL sh:not
        if p == OWL_DISJOINT_WITH && is_iri(o) {
            class_disjoints.push((raw.subject, raw.object));
            return;
        }

        // 6. Compile property algebra into N3Logic rules
        if p == RDF_TYPE && o == OWL_TRANSITIVE_PROP {
            n3_property_rules += 1;
            let _ = writeln!(
                n3_out,
                "{{ ?x <{s}> ?y . ?y <{s}> ?z }} => {{ ?x <{s}> ?z }} ."
            );
        } else if p == RDF_TYPE && o == OWL_SYMMETRIC_PROP {
            n3_property_rules += 1;
            let _ = writeln!(n3_out, "{{ ?x <{s}> ?y }} => {{ ?y <{s}> ?x }} .");
        } else if p == OWL_INVERSE_OF && is_iri(o) {
            n3_property_rules += 1;
            let _ = writeln!(n3_out, "{{ ?x <{s}> ?y }} => {{ ?y <{o}> ?x }} .");
        } else if p == RDFS_SUBPROPERTY_OF && is_iri(o) {
            n3_property_rules += 1;
            let _ = writeln!(n3_out, "{{ ?x <{s}> ?y }} => {{ ?x <{o}> ?y }} .");
        }

        // 7. Preserve all dereferenceable linked-reference & annotation triples (excluding anonymous bnodes)
        if !o.starts_with("_:") {
            let obj_turtle = format_turtle_object(o, raw.packed_object);
            let _ = writeln!(refs_out, "<{s}> <{p}> {obj_turtle} .");
            linked_ref_triples += 1;
        }
    })?;

    refs_out.flush().map_err(|e| e.to_string())?;
    n3_out.flush().map_err(|e| e.to_string())?;

    // 8. Emit SHACL NodeShapes & PropertyShapes from collected OWL Restrictions and Disjointnesses
    let mut shacl_out = BufWriter::with_capacity(
        256 * 1024,
        File::create(&shacl_path).map_err(|e| format!("create {}: {e}", shacl_path.display()))?,
    );
    writeln!(
        shacl_out,
        "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix sh: <http://www.w3.org/ns/shacl#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n"
    )
    .map_err(|e| e.to_string())?;

    let mut shapes_by_class: BTreeMap<String, Vec<BNodeRestriction>> = BTreeMap::new();
    for (cls, bnode_id) in class_bnode_parents {
        if let Some(rest) = bnodes.get(&bnode_id) {
            if rest.on_property.is_some() {
                shapes_by_class
                    .entry(cls)
                    .or_default()
                    .push(rest.clone());
            }
        }
    }
    let mut disjoints_by_class: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (cls, other) in class_disjoints {
        disjoints_by_class.entry(cls).or_default().push(other);
    }

    let mut shacl_node_shapes = 0u64;
    let mut shacl_property_shapes = 0u64;
    let mut all_shape_classes: Vec<String> = shapes_by_class
        .keys()
        .chain(disjoints_by_class.keys())
        .cloned()
        .collect();
    all_shape_classes.sort();
    all_shape_classes.dedup();

    for cls in &all_shape_classes {
        shacl_node_shapes += 1;
        let shape_uri = format!("{cls}_Shape");
        writeln!(
            shacl_out,
            "<{shape_uri}> a sh:NodeShape ;\n    sh:targetClass <{cls}> ."
        )
        .map_err(|e| e.to_string())?;

        if let Some(restrictions) = shapes_by_class.get(cls) {
            for (idx, r) in restrictions.iter().enumerate() {
                let Some(on_prop) = r.on_property.as_deref() else {
                    continue;
                };
                shacl_property_shapes += 1;
                let prop_shape_uri = format!("{shape_uri}_prop_{idx}");
                writeln!(
                    shacl_out,
                    "<{shape_uri}> sh:property <{prop_shape_uri}> .\n\
                     <{prop_shape_uri}> a sh:PropertyShape ;\n    sh:path <{on_prop}> ."
                )
                .map_err(|e| e.to_string())?;

                if let Some(some_val) = r.some_values_from.as_deref() {
                    if some_val != OWL_THING {
                        let qvs_uri = format!("{prop_shape_uri}_qvs");
                        writeln!(
                            shacl_out,
                            "<{prop_shape_uri}> sh:qualifiedValueShape <{qvs_uri}> ;\n    sh:qualifiedMinCount 1 .\n\
                             <{qvs_uri}> sh:class <{some_val}> ."
                        )
                        .map_err(|e| e.to_string())?;
                    }
                }
                if let Some(all_val) = r.all_values_from.as_deref() {
                    if all_val != OWL_THING {
                        let pred = if all_val.starts_with("http://www.w3.org/2001/XMLSchema#") {
                            "sh:datatype"
                        } else {
                            "sh:class"
                        };
                        writeln!(shacl_out, "<{prop_shape_uri}> {pred} <{all_val}> .")
                            .map_err(|e| e.to_string())?;
                    }
                }
                if let Some(has_val) = r.has_value.as_deref() {
                    let obj = format_turtle_object(has_val, None);
                    writeln!(
                        shacl_out,
                        "<{prop_shape_uri}> sh:hasValue {obj} ."
                    )
                    .map_err(|e| e.to_string())?;
                }
                if let Some(min_c) = r.min_count {
                    writeln!(
                        shacl_out,
                        "<{prop_shape_uri}> sh:minCount {min_c} ."
                    )
                    .map_err(|e| e.to_string())?;
                }
                if let Some(max_c) = r.max_count {
                    writeln!(
                        shacl_out,
                        "<{prop_shape_uri}> sh:maxCount {max_c} ."
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }

        if let Some(disjoints) = disjoints_by_class.get(cls) {
            for (d_idx, dis) in disjoints.iter().enumerate() {
                let not_uri = format!("{shape_uri}_not_{d_idx}");
                writeln!(
                    shacl_out,
                    "<{shape_uri}> sh:not <{not_uri}> .\n\
                     <{not_uri}> sh:class <{dis}> ."
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    shacl_out.flush().map_err(|e| e.to_string())?;

    // 8b. Emit ShEx (Shape Expressions) cyclic/streaming topological & LoD grammar (.shex)
    //     for stratified recursion over cyclic anatomy/biology (anastomoses, closed bone loops,
    //     botanical branching) and modular .hmc streaming (LoadedHmcPayload OR WireframeStub).
    {
        let mut shex_out = BufWriter::with_capacity(
            64 * 1024,
            File::create(&shex_path)
                .map_err(|e| format!("create {}: {e}", shex_path.display()))?,
        );
        writeln!(
            shex_out,
            "PREFIX qualia: <https://qualia.org/ns/anatomy#>\n\
             PREFIX hmc:    <https://qualia.org/ns/hmc#>\n\
             PREFIX skos:   <http://www.w3.org/2004/02/skos/core#>\n\
             PREFIX rdfs:   <http://www.w3.org/2000/01/rdf-schema#>\n\
             PREFIX xsd:    <http://www.w3.org/2001/XMLSchema#>\n\
             \n\
             # Stratified recursion & streaming .hmc LoD Shape Expressions for {acr_upper}\n\
             ABSTRACT <BiologicalEntity> {{\n\
               hmc:did                 IRI ;\n\
               rdfs:isDefinedBy        [ <{bioportal_ont_iri}> ] ;\n\
               qualia:radialDepthMin   xsd:float ? ;\n\
               qualia:radialDepthMax   xsd:float ? ;\n\
               qualia:containedIn      @<AnatomicalOrBiologicalCavity> ? ;\n\
               hmc:streamState         ( @<LoadedHmcPayload> OR @<WireframeStub> )\n\
             }}\n\
             \n\
             <AnatomicalOrBiologicalCavity> EXTENDS @<BiologicalEntity> {{\n\
               qualia:sdfClearanceEpsM xsd:float ;\n\
               qualia:encloses         @<BiologicalEntity> *\n\
             }}\n\
             \n\
             <StructuralScaffoldOrgan> EXTENDS @<BiologicalEntity> {{\n\
               qualia:radialDepthMin   [ 0.0 ] ;\n\
               qualia:articulatesWith  @<ArticularOrBranchJoint> * ;\n\
               qualia:enthesisSite     @<EnthesisOrAttachmentPatch> *\n\
             }}\n\
             \n\
             <ArticularOrBranchJoint> {{\n\
               hmc:did                 IRI ;\n\
               qualia:connectsElement  @<StructuralScaffoldOrgan> {{2,*}} ;\n\
               qualia:stabilizedBy     @<CompliantTether> * ;\n\
               qualia:retinacularWrap  @<RetinaculumOrSheathPulley> *\n\
             }}\n\
             \n\
             <CompliantTether> EXTENDS @<BiologicalEntity> {{\n\
               qualia:originPatch      @<EnthesisOrAttachmentPatch> ;\n\
               qualia:insertionPatch   @<EnthesisOrAttachmentPatch> ;\n\
               qualia:geodesicWrapSdf  IRI *\n\
             }}\n\
             \n\
             <EnthesisOrAttachmentPatch> {{\n\
               qualia:hostSurfaceIri   IRI ;\n\
               qualia:barycentricUv    xsd:string ;\n\
               qualia:stiffnessGradMpa xsd:string ?\n\
             }}\n\
             \n\
             <RetinaculumOrSheathPulley> {{\n\
               hmc:did                 IRI ;\n\
               qualia:anchoredTo       @<StructuralScaffoldOrgan> +\n\
             }}\n\
             \n\
             <LoadedHmcPayload> {{\n\
               hmc:sdfBufferOffset     xsd:unsignedLong ;\n\
               hmc:tenDVectorRef       IRI\n\
             }}\n\
             \n\
             <WireframeStub> {{\n\
               hmc:boundingSphereRadius xsd:float ;\n\
               hmc:chunkEndpoint        IRI\n\
             }}\n"
        )
        .map_err(|e| e.to_string())?;
        for cls in all_shape_classes.iter().take(256) {
            if let Some(restrictions) = shapes_by_class.get(cls) {
                writeln!(
                    shex_out,
                    "<{cls}_ShEx> EXTENDS @<BiologicalEntity> EXTRA rdf:type {{"
                )
                .map_err(|e| e.to_string())?;
                for r in restrictions {
                    if let Some(prop) = r.on_property.as_deref() {
                        if let Some(some_val) = r.some_values_from.as_deref() {
                            writeln!(shex_out, "  <{prop}> [ <{some_val}>~ ] + ;")
                                .map_err(|e| e.to_string())?;
                        } else if let Some(all_val) = r.all_values_from.as_deref() {
                            writeln!(shex_out, "  <{prop}> [ <{all_val}>~ ] * ;")
                                .map_err(|e| e.to_string())?;
                        }
                    }
                }
                writeln!(shex_out, "  rdf:type [ skos:Concept ] ?\n}}")
                    .map_err(|e| e.to_string())?;
            }
        }
        shex_out.flush().map_err(|e| e.to_string())?;
    }

    let mut q42_path_str = None;
    let mut q42_quins_written = None;
    let mut q42_lex_terms = None;

    if compile_q42 {
        let q42_dest = match explicit_out_q42 {
            Some(name) => {
                let pb = PathBuf::from(name);
                if pb.is_absolute() {
                    pb
                } else if let Ok(root) = std::env::var("SYNTH_OUTPUT_ROOT") {
                    let r = root.trim();
                    if !r.is_empty() {
                        Path::new(r).join(name)
                    } else {
                        out_dir.join(name)
                    }
                } else {
                    out_dir.join(name)
                }
            }
            None => out_dir.join(format!("{acr_lower}_refactored.q42")),
        };
        if q42_dest.exists() {
            let _ = fs::remove_file(&q42_dest);
        }

        // Append SHACL shapes into a combined Turtle stream or append via append_rdf_to_root
        let combined_ttl = out_dir.join(format!("{acr_lower}_combined.ttl"));
        {
            let mut comb = BufWriter::with_capacity(
                256 * 1024,
                File::create(&combined_ttl).map_err(|e| e.to_string())?,
            );
            let mut r1 = File::open(&refs_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut r1, &mut comb).map_err(|e| e.to_string())?;
            let mut r2 = File::open(&shacl_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut r2, &mut comb).map_err(|e| e.to_string())?;
            comb.flush().map_err(|e| e.to_string())?;
        }

        let max_seg = segment_mib.map(|m| m.saturating_mul(1024 * 1024));
        let written = streaming_import_rdf_with_report(
            &combined_ttl.to_string_lossy(),
            &q42_dest.to_string_lossy(),
            IngestMode::Complete,
            max_seg,
            IngestReport::silent(),
        )
        .map_err(|e| format!("QualiaDB Q42 compile of refactored ontology failed: {e}"))?;
        let _ = fs::remove_file(&combined_ttl);

        let lex_count = qualia_core_db::q42_volume::Q42Volume::open(&q42_dest)
            .ok()
            .and_then(|v| v.lex_view().ok().map(|l| l.entry_count()))
            .unwrap_or(0);

        if ingest_to_graph {
            let quins = qualia_core_db::ontology_loader::load_q42_file(&q42_dest);
            qualia_core_db::daemon_graph::extend_with_ontology_quins_slice(&quins);
        }

        q42_path_str = Some(q42_dest.display().to_string());
        q42_quins_written = Some(written);
        q42_lex_terms = Some(lex_count);
    }

    Ok(OwlRefactorStats {
        acronym: acr_upper,
        source_path: source_path.display().to_string(),
        total_triples_scanned: total_triples,
        owl_thing_purged,
        classes_converted_to_skos_concepts: classes_converted,
        linked_reference_triples: linked_ref_triples,
        shacl_node_shapes,
        shacl_property_shapes,
        n3_subsumption_rules,
        n3_property_algebra_rules: n3_property_rules,
        refs_ttl_path: refs_path.display().to_string(),
        shacl_ttl_path: shacl_path.display().to_string(),
        n3_rules_path: n3_path.display().to_string(),
        shex_path: shex_path.display().to_string(),
        q42_path: q42_path_str,
        q42_quins_written,
        q42_lex_terms,
    })
}

fn compile_local_biomedical_catalogue(
    mcp: Option<&StandaloneSynthMcp>,
    arg: &BioPortalArg,
) -> Result<Value, String> {
    let bio_root = Path::new(
        arg.biomedical_dir
            .as_deref()
            .unwrap_or(DEFAULT_BIOMEDICAL_DIR),
    );
    let local = list_local_biomedical(bio_root)?;
    let arr = local.as_array().ok_or("Expected array")?;
    let filter_acr = arg.acronym.as_deref().map(|s| s.to_ascii_uppercase());
    let do_refactor = arg.refactor_owl.unwrap_or(true);
    let mut results = Vec::new();

    for entry in arr {
        let acr = entry
            .get("acronym")
            .and_then(Value::as_str)
            .unwrap_or("ONT");
        if let Some(ref want) = filter_acr {
            if !acr.eq_ignore_ascii_case(want) {
                continue;
            }
        }
        let path_str = entry.get("path").and_then(Value::as_str).unwrap_or("");
        let src = PathBuf::from(path_str);
        let out_dir = resolve_output_dir(mcp, arg, &src, acr)?;
        if do_refactor && path_str.to_ascii_lowercase().ends_with(".owl") {
            let stats = refactor_owl_to_linked_shacl_n3(
                &src,
                acr,
                &out_dir,
                arg.out_q42.as_deref(),
                arg.compile_q42.unwrap_or(true),
                arg.ingest_to_graph.unwrap_or(false),
                arg.segment_mib,
            )?;
            results.push(json!({
                "acronym": acr,
                "mode": "refactored_linked_shacl_n3_q42",
                "stats": stats,
            }));
        } else {
            let file_len = fs::metadata(&src).map(|m| m.len()).unwrap_or(0);
            let q42_dest = out_dir.join(format!("{}.q42", acr.to_ascii_lowercase()));
            if q42_dest.exists() {
                let _ = fs::remove_file(&q42_dest);
            }
            let fmt = super::q42::infer_compile_format(None, Some(path_str))
                .map_err(|e| e.message)?;
            let (quins, lex_terms, sha) = super::q42::compile_file_to_q42(
                &src,
                file_len,
                &q42_dest,
                fmt,
                Some(&format!("https://bioportal.bioontology.org/ontologies/{acr}")),
                arg.ingest_to_graph.unwrap_or(false),
                arg.segment_mib,
                false,
            )
            .map_err(|e| e.message)?;
            results.push(json!({
                "acronym": acr,
                "mode": "direct_q42",
                "source": path_str,
                "q42_path": q42_dest.display().to_string(),
                "quin_count": quins,
                "lex_terms": lex_terms,
                "sha256": sha,
            }));
        }
    }

    Ok(json!({
        "schema": "synth.bioportal.compile-biomedical/v1",
        "count": results.len(),
        "results": results,
    }))
}
