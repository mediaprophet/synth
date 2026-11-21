//! Compile sourced OWL / RDF/XML / N3 / Turtle / N-Triples / N-Quads / TriG / JSON-LD / YAML-LD
//! into a sealed `.q42` volume under SYNTH_OUTPUT_ROOT using QualiaDB's streaming ingest engine,
//! and execute SPARQL queries against `.q42` volumes, the resident daemon graph, or SPARQL endpoints.
use super::*;
use qualia_core_db::ingest::{
    streaming_import_rdf_with_report, IngestMode, IngestReport,
};
use qualia_core_db::query::ingest_job::IngestRdfFormat;
use qualia_mcp::authoring::ToolError;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Inline `rdf_data` string cap (16 MiB).
const MAX_INLINE_RDF_BYTES: usize = 16 * 1024 * 1024;
/// Streaming file cap for local ontologies such as `fma.owl` (4 GiB).
const MAX_STREAM_RDF_BYTES: u64 = 4 * 1024 * 1024 * 1024;
/// Threshold above which local file compilation uses QualiaDB's multi-threaded streaming sorter.
const STREAM_THRESHOLD_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CompileArg {
    pub source_path: Option<String>,
    pub rdf_data: Option<String>,
    pub format: Option<String>,
    pub out_q42: Option<String>,
    #[serde(default)]
    pub ingest_to_graph: bool,
    pub context_iri: Option<String>,
    pub segment_mib: Option<u64>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SparqlQueryArg {
    pub query: String,
    pub vault_path: Option<String>,
    pub q42_path: Option<String>,
    pub endpoint: Option<String>,
    pub bookmark_id: Option<String>,
    pub limit: Option<usize>,
    pub format: Option<String>,
}

fn context_hash(iri: Option<&str>) -> u64 {
    match iri {
        Some(s) if !s.is_empty() => qualia_core_db::q_hash(s),
        _ => qualia_core_db::q_hash("urn:synth:compile_q42"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedCompileFormat {
    Rdf(IngestRdfFormat, &'static str),
    YamlLdQ42,
    Q42,
}

impl ResolvedCompileFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rdf(_, label) => label,
            Self::YamlLdQ42 => "yaml",
            Self::Q42 => "q42",
        }
    }
}

pub fn infer_compile_format(
    explicit: Option<&str>,
    path: Option<&str>,
) -> Result<ResolvedCompileFormat, ToolError> {
    if let Some(raw) = explicit {
        return match raw.trim().to_ascii_lowercase().as_str() {
            "owl" | "rdf" | "rdfxml" | "rdf/xml" | "xml" => {
                Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::RdfXml, "rdfxml"))
            }
            "ttl" | "turtle" => Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::Turtle, "ttl")),
            "n3" => Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::N3, "n3")),
            "nt" | "ntriples" | "n-triples" => {
                Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::NTriples, "nt"))
            }
            "nq" | "nquads" | "n-quads" => {
                Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::NQuads, "nq"))
            }
            "trig" => Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::TriG, "trig")),
            "jsonld" | "json-ld" | "json" => {
                Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::JsonLd, "jsonld"))
            }
            "yamlld" | "yaml-ld" => {
                Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::YamlLd, "yamlld"))
            }
            "yaml" | "yml" | "yaml-ld-q42" => Ok(ResolvedCompileFormat::YamlLdQ42),
            "cbor" | "cborld" | "cbor-ld" => {
                Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::CborLd, "cborld"))
            }
            "rj" | "rdfjson" | "rdf/json" => {
                Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::RdfJson, "rdfjson"))
            }
            "q42" => Ok(ResolvedCompileFormat::Q42),
            other => Err(ToolError::new(
                "invalid_arguments",
                format!(
                    "unsupported RDF/ontology format {other}; supported: owl, rdfxml, rdf, xml, n3, ttl, turtle, nt, ntriples, nq, nquads, trig, jsonld, yaml-ld-q42, yaml, cborld, rdfjson, q42"
                ),
            )),
        };
    }

    let raw_path = path.unwrap_or("inline.ttl").to_ascii_lowercase();
    let stem = raw_path
        .trim_end_matches(".gz")
        .trim_end_matches(".zst")
        .trim_end_matches(".zstd");
    let ext = Path::new(stem)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("ttl");
    match ext {
        "owl" | "rdf" | "xml" | "rdfxml" => {
            Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::RdfXml, "rdfxml"))
        }
        "ttl" | "turtle" => Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::Turtle, "ttl")),
        "n3" => Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::N3, "n3")),
        "nt" | "ntriples" => Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::NTriples, "nt")),
        "nq" | "nquads" => Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::NQuads, "nq")),
        "trig" => Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::TriG, "trig")),
        "jsonld" | "json" => Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::JsonLd, "jsonld")),
        "yamlld" => Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::YamlLd, "yamlld")),
        "yaml" | "yml" => Ok(ResolvedCompileFormat::YamlLdQ42),
        "cbor" | "cborld" => Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::CborLd, "cborld")),
        "rj" | "rdfjson" => Ok(ResolvedCompileFormat::Rdf(IngestRdfFormat::RdfJson, "rdfjson")),
        "q42" => Ok(ResolvedCompileFormat::Q42),
        other => Err(ToolError::new(
            "invalid_arguments",
            format!(
                "cannot infer format from extension .{other}; pass format=owl|rdfxml|n3|ttl|nt|nq|trig|jsonld|yaml-ld-q42|q42"
            ),
        )),
    }
}

pub fn sha256_file(path: &Path) -> Result<String, ToolError> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| ToolError::new("execution_failed", format!("cannot open {}: {e}", path.display())))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| ToolError::new("execution_failed", format!("cannot read {}: {e}", path.display())))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn inspect_sealed_q42(dest: &Path) -> (usize, usize) {
    if let Ok(vol) = qualia_core_db::q42_volume::Q42Volume::open(dest) {
        let lex_count = vol.lex_view().map(|v| v.entry_count()).unwrap_or(0);
        let quins = qualia_core_db::q42_reader::read_q42_quins(dest)
            .map(|q| q.len())
            .unwrap_or(0);
        return (quins, lex_count);
    }
    let quins = qualia_core_db::ontology_loader::load_q42_file(dest);
    (quins.len(), 0)
}

fn maybe_ingest_q42_to_daemon_graph(dest: &Path, ingest_to_graph: bool) {
    if !ingest_to_graph {
        return;
    }
    let quins = qualia_core_db::ontology_loader::load_q42_file(dest);
    qualia_core_db::daemon_graph::extend_with_ontology_quins_slice(&quins);
}

/// Compile a local file path directly into `dest` `.q42` using QualiaDB's RDF/OWL/YAML-LD/Q42 pipeline.
pub fn compile_file_to_q42(
    source_file: &Path,
    file_len: u64,
    dest: &Path,
    format: ResolvedCompileFormat,
    context_iri: Option<&str>,
    ingest_to_graph: bool,
    segment_mib: Option<u64>,
    progress_stderr: bool,
) -> Result<(usize, usize, String), ToolError> {
    if dest.exists() {
        return Err(ToolError::new("native_output_refused", "Output already exists"));
    }
    let ctx = context_hash(context_iri);
    let lower_name = source_file.to_string_lossy().to_ascii_lowercase();
    let is_compressed =
        lower_name.ends_with(".gz") || lower_name.ends_with(".zst") || lower_name.ends_with(".zstd");

    match format {
        ResolvedCompileFormat::Q42 => {
            std::fs::copy(source_file, dest).map_err(|e| {
                ToolError::new("execution_failed", format!("cannot copy q42 volume: {e}"))
            })?;
            maybe_ingest_q42_to_daemon_graph(dest, ingest_to_graph);
            let (quin_count, lex_terms) = inspect_sealed_q42(dest);
            let sha = sha256_file(dest)?;
            Ok((quin_count, lex_terms, sha))
        }
        ResolvedCompileFormat::YamlLdQ42 => {
            let bytes = std::fs::read(source_file).map_err(|e| {
                ToolError::new("execution_failed", format!("cannot read yaml source: {e}"))
            })?;
            let (quins, lex, _shape) =
                qualia_core_db::q42::yaml_ld_q42::compile_yaml_ld_q42_auto(&bytes, ctx, 1)
                    .map_err(|e| {
                        ToolError::new("execution_failed", format!("yaml-ld-q42 parse failed: {e}"))
                    })?;
            if quins.is_empty() {
                return Err(ToolError::new(
                    "execution_failed",
                    "yaml-ld-q42 parsed to zero NQuins; refusing empty Q42 volume",
                ));
            }
            let lex_terms = lex.len();
            qualia_core_db::q42_volume::write_sorted_quins_volume_with_lex(dest, &quins, &lex, 0)
                .map_err(|e| ToolError::new("execution_failed", format!("Q42 write failed: {e}")))?;
            if ingest_to_graph {
                qualia_core_db::daemon_graph::extend_graph_lexicon(lex);
                qualia_core_db::daemon_graph::extend_with_ontology_quins_slice(&quins);
            }
            let sha = sha256_file(dest)?;
            Ok((quins.len(), lex_terms, sha))
        }
        ResolvedCompileFormat::Rdf(rdf_fmt, _) => {
            let use_streaming = file_len > STREAM_THRESHOLD_BYTES
                || is_compressed
                || segment_mib.is_some();
            if use_streaming {
                let max_seg = segment_mib.map(|m| m.saturating_mul(1024 * 1024));
                let report = if progress_stderr {
                    IngestReport::text_stderr(1)
                } else {
                    IngestReport::silent()
                };
                let written = streaming_import_rdf_with_report(
                    &source_file.to_string_lossy(),
                    &dest.to_string_lossy(),
                    IngestMode::Complete,
                    max_seg,
                    report,
                )
                .map_err(|e| ToolError::new("execution_failed", format!("QualiaDB streaming RDF ingest failed: {e}")))?;
                if written == 0 {
                    let _ = std::fs::remove_file(dest);
                    return Err(ToolError::new(
                        "execution_failed",
                        "RDF/OWL source parsed to zero triples; refusing empty Q42 volume",
                    ));
                }
                let lex_terms = qualia_core_db::q42_volume::Q42Volume::open(dest)
                    .ok()
                    .and_then(|v| v.lex_view().ok().map(|l| l.entry_count()))
                    .unwrap_or(0);
                maybe_ingest_q42_to_daemon_graph(dest, ingest_to_graph);
                let sha = sha256_file(dest)?;
                Ok((written as usize, lex_terms, sha))
            } else {
                let bytes = std::fs::read(source_file).map_err(|e| {
                    ToolError::new("execution_failed", format!("cannot read RDF source: {e}"))
                })?;
                compile_inline_bytes_to_q42(&bytes, dest, rdf_fmt, ctx, ingest_to_graph)
            }
        }
    }
}

fn compile_inline_bytes_to_q42(
    bytes: &[u8],
    dest: &Path,
    rdf_fmt: IngestRdfFormat,
    ctx: u64,
    ingest_to_graph: bool,
) -> Result<(usize, usize, String), ToolError> {
    let (quins, lex): (Vec<qualia_core_db::NQuin>, HashMap<u64, String>) =
        qualia_core_db::ontology_loader::parse_rdf_bytes_to_quins_and_lex(bytes, rdf_fmt, ctx)
            .map_err(|e| ToolError::new("execution_failed", format!("RDF parse failed: {e}")))?;
    if quins.is_empty() {
        return Err(ToolError::new(
            "execution_failed",
            "RDF parsed to zero triples; refusing empty Q42 volume",
        ));
    }
    let lex_terms = lex.len();
    qualia_core_db::q42_volume::write_sorted_quins_volume_with_lex(dest, &quins, &lex, 0)
        .map_err(|e| ToolError::new("execution_failed", format!("Q42 write failed: {e}")))?;
    if ingest_to_graph {
        qualia_core_db::daemon_graph::extend_graph_lexicon(lex);
        qualia_core_db::daemon_graph::extend_with_ontology_quins_slice(&quins);
    }
    let sha = sha256_file(dest)?;
    Ok((quins.len(), lex_terms, sha))
}

pub(super) fn compile(mcp: &StandaloneSynthMcp, args: Value) -> Result<Value, ToolError> {
    let parsed: CompileArg = serde_json::from_value(args)
        .map_err(|_| ToolError::new("invalid_arguments", "Invalid compile_q42 arguments"))?;
    if parsed.source_path.is_some() == parsed.rdf_data.is_some() {
        return Err(ToolError::new(
            "invalid_arguments",
            "Provide exactly one of source_path or rdf_data",
        ));
    }
    let out_name = parsed.out_q42.as_deref().ok_or_else(|| {
        ToolError::new(
            "invalid_arguments",
            "out_q42 is required (create-only *.q42 under SYNTH_OUTPUT_ROOT)",
        )
    })?;
    let dest = mcp.confined_destination(
        out_name,
        ".q42",
        "Use a visible .q42 filename without directory components",
    )?;
    if dest.exists() {
        return Err(ToolError::new("native_output_refused", "Output already exists"));
    }

    let format = infer_compile_format(parsed.format.as_deref(), parsed.source_path.as_deref())?;
    let ctx = context_hash(parsed.context_iri.as_deref());

    let (quin_count, lex_terms, sha, source_label) = if let Some(path) = parsed.source_path.as_deref()
    {
        let root = mcp.input_root.as_ref().ok_or_else(|| {
            ToolError::new(
                "native_input_unconfigured",
                "SYNTH_INPUT_ROOT is required to compile a source_path",
            )
        })?;
        let (canonical, file_len) = root.resolve_path(path, MAX_STREAM_RDF_BYTES)?;
        let label = canonical.display().to_string();
        let (q_count, l_terms, sha_hex) = compile_file_to_q42(
            &canonical,
            file_len,
            &dest,
            format,
            parsed.context_iri.as_deref(),
            parsed.ingest_to_graph,
            parsed.segment_mib,
            false,
        )?;
        (q_count, l_terms, sha_hex, label)
    } else {
        let data = parsed.rdf_data.as_deref().unwrap_or("");
        if data.len() > MAX_INLINE_RDF_BYTES {
            return Err(ToolError::new(
                "invalid_arguments",
                format!("rdf_data is {} bytes; max is {MAX_INLINE_RDF_BYTES}", data.len()),
            ));
        }
        let bytes = data.as_bytes();
        let (q_count, l_terms, sha_hex) = match format {
            ResolvedCompileFormat::Q42 => {
                std::fs::write(&dest, bytes).map_err(|e| {
                    ToolError::new("execution_failed", format!("cannot write q42 copy: {e}"))
                })?;
                maybe_ingest_q42_to_daemon_graph(&dest, parsed.ingest_to_graph);
                let (q, l) = inspect_sealed_q42(&dest);
                (q, l, format!("{:x}", Sha256::digest(bytes)))
            }
            ResolvedCompileFormat::YamlLdQ42 => {
                let (quins, lex, _shape) =
                    qualia_core_db::q42::yaml_ld_q42::compile_yaml_ld_q42_auto(bytes, ctx, 1)
                        .map_err(|e| {
                            ToolError::new("execution_failed", format!("yaml-ld-q42 parse failed: {e}"))
                        })?;
                if quins.is_empty() {
                    return Err(ToolError::new(
                        "execution_failed",
                        "yaml-ld-q42 parsed to zero NQuins; refusing empty Q42 volume",
                    ));
                }
                let lex_terms = lex.len();
                qualia_core_db::q42_volume::write_sorted_quins_volume_with_lex(
                    &dest, &quins, &lex, 0,
                )
                .map_err(|e| ToolError::new("execution_failed", format!("Q42 write failed: {e}")))?;
                if parsed.ingest_to_graph {
                    qualia_core_db::daemon_graph::extend_graph_lexicon(lex);
                    qualia_core_db::daemon_graph::extend_with_ontology_quins_slice(&quins);
                }
                let sha = sha256_file(&dest)?;
                (quins.len(), lex_terms, sha)
            }
            ResolvedCompileFormat::Rdf(rdf_fmt, _) => {
                compile_inline_bytes_to_q42(bytes, &dest, rdf_fmt, ctx, parsed.ingest_to_graph)?
            }
        };
        (q_count, l_terms, sha_hex, "inline:rdf_data".into())
    };

    Ok(json!({
        "schema": "synth.compile-q42/v1",
        "source": source_label,
        "format": format.label(),
        "path": dest.display().to_string(),
        "quin_count": quin_count,
        "lex_terms": lex_terms,
        "sha256": sha,
        "ingested_to_graph": parsed.ingest_to_graph,
        "graph_quin_count": qualia_core_db::daemon_graph::graph_quin_count(),
        "query": {
            "tool": "query_sparql",
            "vault_path": dest.display().to_string(),
            "example": "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 20"
        }
    }))
}

/// Resolve a `.q42` vault path against CWD, `output_root`, `input_root`, and process env roots.
pub fn resolve_q42_vault_path(
    mcp: Option<&StandaloneSynthMcp>,
    raw: &str,
) -> PathBuf {
    let direct = qualia_mcp::tools::resolve_audit_path(Some(raw));
    if direct.exists() {
        return direct;
    }
    let rel = Path::new(raw);
    if !rel.is_absolute() {
        if let Some(m) = mcp {
            if let Some(out_root) = m.output_root.as_ref() {
                if let Ok(dest) = out_root.destination_with_ext(raw, ".q42", "invalid .q42 name") {
                    if dest.exists() {
                        return dest;
                    }
                }
            }
            if let Some(in_root) = m.input_root.as_ref() {
                if let Ok((canon, _)) = in_root.resolve_path(raw, MAX_STREAM_RDF_BYTES) {
                    return canon;
                }
            }
        }
    }
    direct
}

/// Query a remote or bookmarked HTTP(S) SPARQL endpoint.
pub fn query_remote_sparql_endpoint(
    endpoint_url: Option<&str>,
    bookmark_id: Option<&str>,
    query: &str,
    accept_format: Option<&str>,
) -> Result<Value, String> {
    let mut url = endpoint_url.map(str::to_owned);
    let mut secret = None;
    if let Some(bid) = bookmark_id {
        let bm = crate::operator::bookmarks::get_bookmark(bid)?
            .ok_or_else(|| format!("Unknown bookmark `{bid}`"))?;
        if url.is_none() {
            url = bm
                .data_endpoint
                .clone()
                .filter(|s| !s.is_empty())
                .or(Some(bm.url.clone()));
        }
        if bm.requires_api_key {
            secret = crate::operator::bookmarks::secret_value(bid)?;
            if secret.is_none() {
                if let Some(env_name) = bm.api_key_env.as_deref() {
                    if let Ok(v) = std::env::var(env_name) {
                        if !v.is_empty() {
                            secret = Some(v);
                        }
                    }
                }
            }
        }
    }
    let url = url.ok_or_else(|| "endpoint URL or bookmark_id is required".to_string())?;
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("SPARQL endpoint URL must start with http:// or https://".into());
    }
    let accept = match accept_format.unwrap_or("json").to_ascii_lowercase().as_str() {
        "xml" => "application/sparql-results+xml, application/rdf+xml;q=0.8, */*;q=0.1",
        "tsv" => "text/tab-separated-values, */*;q=0.1",
        "csv" => "text/csv, */*;q=0.1",
        "ntriples" | "nt" | "turtle" | "ttl" => {
            "application/n-triples, text/turtle;q=0.9, application/rdf+xml;q=0.8, */*;q=0.1"
        }
        _ => "application/sparql-results+json, application/json;q=0.9, text/turtle;q=0.8, application/n-triples;q=0.8, */*;q=0.1",
    };

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(30))
        .redirects(5)
        .user_agent("Webizen-AI-Synth/0.1 (+https://github.com/mediaprophet/synth)")
        .build();
    let mut req = agent
        .post(&url)
        .set("Content-Type", "application/sparql-query; charset=utf-8")
        .set("Accept", accept);
    if let Some(token) = secret.as_deref() {
        req = req.set("Authorization", &format!("Bearer {token}"));
    }
    let resp = req
        .send_string(query)
        .map_err(|e| format!("SPARQL endpoint request to {url} failed: {e}"))?;
    let content_type = resp
        .header("Content-Type")
        .unwrap_or("application/sparql-results+json")
        .to_owned();
    let text = resp
        .into_string()
        .map_err(|e| format!("Failed reading SPARQL endpoint response: {e}"))?;

    if let Ok(parsed_json) = serde_json::from_str::<Value>(&text) {
        let bindings_count = parsed_json
            .get("results")
            .and_then(|r| r.get("bindings"))
            .and_then(Value::as_array)
            .map(Vec::len);
        return Ok(json!({
            "endpoint": url,
            "content_type": content_type,
            "returned": bindings_count,
            "sparql_results": parsed_json,
        }));
    }

    Ok(json!({
        "endpoint": url,
        "content_type": content_type,
        "body": text,
    }))
}

/// Execute `query_sparql` across a local `.q42` vault, a remote/bookmarked SPARQL endpoint, or the daemon graph.
pub fn execute_sparql_tool(
    mcp: Option<&StandaloneSynthMcp>,
    args: Value,
) -> Result<Value, ToolError> {
    let query = args
        .get("query")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ToolError::new("invalid_arguments", "query is required"))?;
    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(1_000) as usize;
    let format = args.get("format").and_then(Value::as_str);
    let endpoint = args.get("endpoint").and_then(Value::as_str);
    let bookmark_id = args.get("bookmark_id").and_then(Value::as_str);
    let vault_opt = args
        .get("vault_path")
        .or_else(|| args.get("q42_path"))
        .and_then(Value::as_str);

    if endpoint.is_some()
        || bookmark_id.is_some()
        || vault_opt.is_some_and(|v| v.starts_with("http://") || v.starts_with("https://"))
    {
        let ep = endpoint.or(vault_opt);
        return query_remote_sparql_endpoint(ep, bookmark_id, query, format)
            .map_err(|e| ToolError::new("execution_failed", e));
    }

    if let Some(raw_vault) = vault_opt {
        let resolved = resolve_q42_vault_path(mcp, raw_vault);
        return qualia_mcp::tools::query_q42_vault(&resolved, query, limit).map_err(|e| {
            ToolError::new(
                "execution_failed",
                format!("SPARQL query on {} failed: {e}", resolved.display()),
            )
        });
    }

    let raw_args = serde_json::to_vec(&args).unwrap_or_else(|_| b"{}".to_vec());
    let text = qualia_mcp::tools::query_sparql(&raw_args)
        .map_err(|e| ToolError::new("execution_failed", e.to_string()))?;
    serde_json::from_str::<Value>(&text)
        .map_err(|e| ToolError::new("execution_failed", e.to_string()))
}
