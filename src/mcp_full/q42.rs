//! Compile sourced RDF/Turtle into a sealed `.q42` volume under SYNTH_OUTPUT_ROOT.
use super::*;
use qualia_mcp::authoring::ToolError;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

const MAX_RDF_BYTES: usize = 16 * 1024 * 1024;

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct CompileArg {
    source_path: Option<String>,
    rdf_data: Option<String>,
    format: Option<String>,
    out_q42: Option<String>,
    #[serde(default)]
    ingest_to_graph: bool,
    context_iri: Option<String>,
}

fn context_hash(iri: Option<&str>) -> u64 {
    match iri {
        Some(s) if !s.is_empty() => qualia_core_db::q_hash(s),
        _ => qualia_core_db::q_hash("urn:synth:compile_q42"),
    }
}

fn infer_format(explicit: Option<&str>, path: Option<&str>) -> Result<&'static str, ToolError> {
    if let Some(raw) = explicit {
        return match raw.to_ascii_lowercase().as_str() {
            "ttl" | "turtle" | "n3" | "nt" | "ntriples" => Ok("ttl"),
            "q42" => Ok("q42"),
            other => Err(ToolError::new(
                "invalid_arguments",
                format!("unsupported RDF format {other}; use ttl, n3, nt, or q42"),
            )),
        };
    }
    let ext = path
        .and_then(|p| Path::new(p).extension())
        .and_then(|s| s.to_str())
        .unwrap_or("ttl")
        .to_ascii_lowercase();
    match ext.as_str() {
        "ttl" | "turtle" | "n3" | "nt" | "rdf" => Ok("ttl"),
        "q42" => Ok("q42"),
        other => Err(ToolError::new(
            "invalid_arguments",
            format!("cannot infer format from extension .{other}; pass format=ttl"),
        )),
    }
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
    let out_name = parsed
        .out_q42
        .as_deref()
        .ok_or_else(|| ToolError::new("invalid_arguments", "out_q42 is required (create-only *.q42 under SYNTH_OUTPUT_ROOT)"))?;
    let dest = mcp.confined_destination(
        out_name,
        ".q42",
        "Use a visible .q42 filename without directory components",
    )?;
    if dest.exists() {
        return Err(ToolError::new(
            "native_output_refused",
            "Output already exists",
        ));
    }

    let (bytes, source_label) = if let Some(path) = parsed.source_path.as_deref() {
        let root = mcp.input_root.as_ref().ok_or_else(|| {
            ToolError::new(
                "native_input_unconfigured",
                "SYNTH_INPUT_ROOT is required to compile a source_path",
            )
        })?;
        let (canonical, bytes) = root.read(path, MAX_RDF_BYTES)?;
        (bytes, canonical.display().to_string())
    } else {
        let data = parsed.rdf_data.as_deref().unwrap_or("");
        if data.len() > MAX_RDF_BYTES {
            return Err(ToolError::new(
                "invalid_arguments",
                format!("rdf_data is {} bytes; max is {MAX_RDF_BYTES}", data.len()),
            ));
        }
        (data.as_bytes().to_vec(), "inline:rdf_data".into())
    };

    let format = infer_format(parsed.format.as_deref(), parsed.source_path.as_deref())?;
    let ctx = context_hash(parsed.context_iri.as_deref());
    let (quin_count, lex_terms, sha) = if format == "q42" {
        std::fs::write(&dest, &bytes).map_err(|e| {
            ToolError::new("execution_failed", format!("cannot write q42 copy: {e}"))
        })?;
        let quins = qualia_core_db::ontology_loader::load_q42_file(&dest);
        if parsed.ingest_to_graph {
            qualia_core_db::daemon_graph::extend_with_ontology_quins_slice(&quins);
        }
        (quins.len(), 0usize, format!("{:x}", Sha256::digest(&bytes)))
    } else {
        let (quins, lex): (Vec<qualia_core_db::NQuin>, HashMap<u64, String>) =
            qualia_core_db::ontology_loader::parse_ttl_bytes_to_quins_and_lex(&bytes, ctx)
                .map_err(|e| ToolError::new("execution_failed", format!("RDF parse failed: {e}")))?;
        if quins.is_empty() {
            return Err(ToolError::new(
                "execution_failed",
                "RDF parsed to zero triples; refusing empty Q42 volume",
            ));
        }
        let lex_terms = lex.len();
        qualia_core_db::q42_volume::write_sorted_quins_volume_with_lex(&dest, &quins, &lex, 0)
            .map_err(|e| ToolError::new("execution_failed", format!("Q42 write failed: {e}")))?;
        if parsed.ingest_to_graph {
            qualia_core_db::daemon_graph::extend_with_ontology_quins_slice(&quins);
        }
        let written = std::fs::read(&dest).map_err(|e| {
            ToolError::new("execution_failed", format!("cannot read sealed q42: {e}"))
        })?;
        (
            quins.len(),
            lex_terms,
            format!("{:x}", Sha256::digest(&written)),
        )
    };

    Ok(json!({
        "schema": "synth.compile-q42/v1",
        "source": source_label,
        "format": format,
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
