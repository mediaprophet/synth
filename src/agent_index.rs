//! Agent discovery index for the 150 Hypermedia morphisms and Synth MCP tools.
//!
//! Synth is the development tool (produce / preview / compile). QualiaPortal and
//! the world host are the visitor runtime. Agents look up work with `synth_find`
//! (`q=xpbd`, `q=nla`, `q=osm`) instead of guessing a third-engine name.
use crate::production::assets::ontology::{AgenticMorphismSpec, MORPHISMS};
use serde_json::{json, Value};

pub const SCHEMA: &str = "synth.find/v1";
pub const PRODUCT_NAME: &str = "Webizen AI Synth";
/// Public release id. Cargo.toml stays SemVer `0.1.0` (Cargo rejects four-part versions).
pub const RELEASE_VERSION: &str = "0.0.41.18";
pub const ROLE: &str = "development tool (produce, preview, compile)";
pub const RUNTIME: &str = "QualiaPortal + world host";

/// Extra search tokens injected into a morphism haystack so agents who type a
/// familiar word still land on the QualiaDB / Etalon / Hypermedia id.
const EXTRA: &[(&str, &str)] = &[
    ("hypermedia.kinetics.step_island_xpbd", "xpbd pbd physics rigidbody rigid-body collision island area3d gravity wind etalon-sim"),
    ("hypermedia.kinetics.query_direct_space", "physics raycast ray-cast sweep hit collider collision mask directspace etalon-sim"),
    ("hypermedia.kinetics.navigate_funnel_orca", "navmesh navigation pathfind pathfinding orca funnel waypoint agent crowd etalon-sim"),
    ("hypermedia.kinetics.evaluate_nla_channelbag", "nla animation tween channelbag action strip blend easing lens"),
    ("hypermedia.kinetics.interpolate_motor_fti", "pose motor pga screw slerp fti interpolation kinematics"),
    ("hypermedia.kinetics.solve_two_bone_ik", "ik inverse-kinematics two-bone fabrik ccd skeletal"),
    ("hypermedia.kinetics.rig_biomechanical_archetype", "ik skeleton fauna skl1 skl2 rig walk"),
    ("hypermedia.kinetics.interpolate_keyframes", "animation keyframe easing timeline"),
    ("hypermedia.kinetics.step_simulation_zone", "particles curl-noise attractor zone"),
    ("hypermedia.geodesy.couple_continuum_xpbd", "xpbd physics terrain contact soil"),
    ("hypermedia.layout.route_input_action", "input hid keyboard gamepad pointer actionmap deadzone hud"),
    ("hypermedia.layout.project_diegetic_surface", "hud diegetic ui quad pick uv camera"),
    ("hypermedia.manifold.compile_10d_pbr", "camera camera3d look-at albedo photograph pbr 10d"),
    ("hypermedia.locus.transcode_geospatial_vector", "gis osm wgs84 lon lat vector map geodesy"),
    ("hypermedia.geodesy.sculpt_terrain", "gis terrain heightfield dem map"),
    ("hypermedia.geodesy.compile_m64", "gis m64 matter dem terrain"),
    ("hypermedia.geodesy.kriging_variogram_morphometry", "gis kriging dem morphometry wgs84"),
    ("hypermedia.geodesy.verify_m64", "gis m64 matter"),
    ("hypermedia.geodesy.advect_m64_grid", "gis m64 advection"),
    ("hypermedia.geodesy.step_sector_continuum_3d", "gis sector continuum"),
    ("hypermedia.kinetics.evaluate_mobility_fracture_risk", "clinical mobility fracture risk skeleton"),
];

const TOOLS: &[ToolHit] = &[
    ToolHit {
        name: "synth_find",
        haystack: "find search discover morphism xpbd nla ik navmesh gis osm n3 sparql hmc q42 kinetics",
        invoke: "Pass q= with a keyword. Default 20 hits.",
    },
    ToolHit {
        name: "synth_produce_present",
        haystack: "produce preview compile hmc 10d vibe capability.invoke asset morphism hypermedia",
        invoke: "capability.invoke inside source; production AssetHost. vibe_invoke is a different Poet snapshot.",
    },
    ToolHit {
        name: "synth_compile_q42",
        haystack: "n3 turtle ttl rdf nt q42 ontology n3logic shacl compile seal",
        invoke: "Preferred source is N3 (N3Logic + SHACL). ttl/nt also compile. Then query_sparql vault_path.",
    },
    ToolHit {
        name: "query_sparql",
        haystack: "sparql sparql-engine graph ontology q42 vault query join gis place catalog",
        invoke: "Internal SPARQL over admitted Q42 ontologies. Pass vault_path for a sealed .q42.",
    },
    ToolHit {
        name: "query_graph",
        haystack: "graph ontology q42 nquin",
        invoke: "QualiaDB graph query on the same Synth MCP server.",
    },
    ToolHit {
        name: "ingest_ontology",
        haystack: "ontology q42 rdf n3 ingest graph",
        invoke: "Load a Q42/RDF ontology into the process graph.",
    },
    ToolHit {
        name: "parse_rdf",
        haystack: "rdf n3 turtle ttl parse",
        invoke: "Parse RDF/N3 before synth_compile_q42.",
    },
    ToolHit {
        name: "synth_operations",
        haystack: "catalogue catalog operations morphisms paged q search",
        invoke: "Paged list. Pass category=kinetics or q=xpbd. Default 40 summaries.",
    },
    ToolHit {
        name: "synth_ontology_morphisms",
        haystack: "ontology morphisms category kinetics geodesy layout",
        invoke: "Same 150 morphisms with teleology. Pass category or q.",
    },
    ToolHit {
        name: "synth_operation_contracts",
        haystack: "contracts encode recipe output host",
        invoke: "Typed output recipes. Do not infer encoders from a namespace.",
    },
    ToolHit {
        name: "synth_bookmarks",
        haystack: "bookmark sparql rdf gis osm dem stac endpoint",
        invoke: "kind=sparql|rdf|gis|rest|csv|page. Secrets via action=set_secret.",
    },
    ToolHit {
        name: "synth_jobs",
        haystack: "job fetch ingest compile osm gis dem stac scheduler inbox",
        invoke: "kind=fetch|ingest|compile. OSM/DEM/STAC ingest is opt-in later; geodesy morphisms bake maps now.",
    },
    ToolHit {
        name: "synth_capabilities",
        haystack: "capabilities providers etalon vibe sdk role runtime product",
        invoke: "Product card plus Vibe/Etalon provider registry.",
    },
    ToolHit {
        name: "synth_ontology_categories",
        haystack: "categories locus manifold kinetics geodesy twelve",
        invoke: "Twelve category summaries plus example morphism ids.",
    },
    ToolHit {
        name: "synth_profiles",
        haystack: "profile producer files quality production16 preview authoring",
        invoke: "Registered AssetProducer profiles.",
    },
    ToolHit {
        name: "synth_environment_bootstrap",
        haystack: "tray studio environment llm vault p64 gguf bootstrap",
        invoke: "Tray/studio bootstrap plus local LLM vault scan.",
    },
    ToolHit {
        name: "synth_operator_status",
        haystack: "operator status tray aether tags git upgrade",
        invoke: "Operator snapshot. fetch=true ls-remote origin tags. Browser GET /operator.",
    },
    ToolHit {
        name: "synth_production_rules",
        haystack: "biosphere taxon evidence encyclopedia ecopunk fiction",
        invoke: "Biosphere non-fiction taxa need evidence .q42 before produce.",
    },
    ToolHit {
        name: "synth_update",
        haystack: "update upgrade release github signed ed25519 installer",
        invoke: "action=check|upgrade|verify|apply. Never overwrites a locked synth.exe.",
    },
    ToolHit {
        name: "synth_colab",
        haystack: "colab ultra trellis trellis2 ovoxel 3d glb pbr a100 l4 burst image-to-3d multiview q42 ontology compile",
        invoke: "action=status|configure|routines|submit|poll|ingest|export_notebook. Offloads TRELLIS.2-4B 3D O-Voxel/PBR and 83GB High-RAM .q42 compilation to Google Colab Ultra, then seals .10d v2 (HOLO 0x0306) and .q42 locally.",
    },
];

const NOTES: &[Note] = &[
    Note {
        tokens: &["runtime", "play", "player", "visitor", "host"],
        text: "Synth is the development tool. QualiaPortal plus the world host play produced HMC/Q42. Do not mint a per-scene Rust file.",
    },
    Note {
        tokens: &["hmc", "drop", "external", "add-asset", "add_asset", "scene"],
        text: "Add scene assets as external HMC / .10d / .q42 files. The host does not rebuild when a new container lands.",
    },
    Note {
        tokens: &["osm", "gis", "dem", "stac", "map", "openstreetmap"],
        text: "Produce-time GIS is hypermedia.geodesy.* and hypermedia.locus.transcode_geospatial_vector. Live OSM/DEM/STAC ingest is opt-in later via synth_jobs and bookmarks kind=gis.",
    },
    Note {
        tokens: &["n3", "turtle", "ttl", "sparql", "q42", "ontology", "shacl", "n3logic"],
        text: "Author logic as N3 (N3Logic + SHACL). synth_compile_q42 seals .q42. query_sparql runs over admitted Q42 ontologies. There is no public SPARQL HTTP server.",
    },
    Note {
        tokens: &["godot", "blender", "unity", "unreal"],
        text: "Those names are not product brands in this tree. The matching work is QualiaDB, Etalon, and Hypermedia morphisms invoked from Synth.",
    },
    Note {
        tokens: &["xpbd", "physics", "nla", "ik", "navmesh", "camera", "input"],
        text: "Kinetics, layout, and Camera3D compile live as catalog morphisms. Invoke them with capability.invoke inside synth_produce_present.",
    },
    Note {
        tokens: &["antigravity", "gemini", "codex", "grok", "client", "mcp", "agents", "category"],
        text: "Adapters: vendor/grok, vendor/codex, vendor/antigravity. Antigravity IDE reads .agents/mcp_config.json and ~/.gemini/config/mcp_config.json. Canonical contract is AGENTS.md; GEMINI.md is the always-on digest. Category tables live in agent-instructions/<category>.md.",
    },
    Note {
        tokens: &["colab", "trellis", "trellis2", "ovoxel", "a100", "l4", "burst"],
        text: "Heavy 24GB+ VRAM / 83GB High-RAM jobs (microsoft/TRELLIS.2-4B O-Voxel 3D + 4K PBR and multi-GB ontology .q42 compilation) run via synth_colab (CLI: synth colab) over HTTPS tunnel or Google Drive queue, automatically sealing Holographic .10d v2 (HOLO 0x0306) and .q42 volumes locally.",
    },
];

struct ToolHit {
    name: &'static str,
    haystack: &'static str,
    invoke: &'static str,
}

struct Note {
    tokens: &'static [&'static str],
    text: &'static str,
}

pub fn product_card() -> Value {
    json!({
        "name": PRODUCT_NAME,
        "version": RELEASE_VERSION,
        "role": ROLE,
        "runtime": RUNTIME,
        "license": "CC-BY-NC-ND-4.0",
        "copyright": "Copyright (c) 2026 Timothy Charles Holborn",
        "build_identity": crate::session_admission::build_identity_and_drift_report(),
        "gpu_runtime_profile": crate::session_admission::detect_runtime_and_gpu_profile(None),
        "morphisms": MORPHISMS.len(),
        "find": "synth_find (q=xpbd|nla|ik|navmesh|camera|input|gis|osm|n3|sparql|hmc)",
        "invoke": "capability.invoke inside synth_produce_present source (production AssetHost)",
        "rdf": "N3 (N3Logic + SHACL); ttl/nt/yaml-ld-q42 also compile via synth_compile_q42",
        "sparql": "query_sparql over admitted Q42 ontologies (internal; not a public SPARQL HTTP server)",
        "scene_assets": "Add via external HMC / .10d / .q42 or convert 3D files with synth_convert_3d. The host does not rebuild.",
        "gis": "Produce-time geodesy morphisms are live. OSM/DEM/STAC ingest is opt-in later via synth_jobs + bookmarks kind=gis.",
        "categories": [
            "locus", "manifold", "surface", "radiance", "glyph", "layout",
            "kinetics", "acoustics", "electromagnetics", "geodesy", "discourse", "chronicle"
        ],
    })
}

pub fn mcp_instructions(input_configured: bool, output_configured: bool) -> String {
    format!(
        "Webizen AI Synth {version} is the development tool (produce/preview/compile). QualiaPortal + world host is the visitor runtime. \
Find QualiaDB/Etalon/Hypermedia work with synth_find q= (xpbd, nla, ik, navmesh, camera, input, gis, osm, n3, sparql, hmc). \
{count} morphisms are paged through synth_operations / synth_ontology_morphisms / synth_operation_contracts (default 40 summaries; pass category, id, q, offset, limit, detail=full). \
Execute morphisms using capability.invoke inside synth_produce_present (production AssetHost); vibe_invoke/vibe_catalog use a separate generic Poet snapshot. \
Author logic as N3 (N3Logic + SHACL); synth_compile_q42 seals .q42; query_sparql / query_graph / ingest_ontology / parse_rdf run on this same server over admitted Q42 ontologies. \
Add scene assets as external HMC/.10d/.q42 files (host does not rebuild). GIS maps bake through hypermedia.geodesy.* now; OSM/DEM/STAC ingest is opt-in later via synth_jobs and bookmarks kind=gis. \
Tool names use underscores. Input root configured: {input}; output root configured: {output}. Inspect returned PNG/SVG/native review files before judging quality.",
        version = RELEASE_VERSION,
        count = MORPHISMS.len(),
        input = input_configured,
        output = output_configured,
    )
}

pub fn morphism_matches(m: &AgenticMorphismSpec, tokens: &[String]) -> bool {
    if tokens.is_empty() {
        return true;
    }
    let hay = morphism_haystack(m);
    tokens.iter().all(|t| hay.contains(t.as_str()))
}

pub fn parse_tokens(q: &str) -> Vec<String> {
    q.split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')))
        .filter(|t| t.len() >= 2)
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

fn morphism_haystack(m: &AgenticMorphismSpec) -> String {
    let mut s = String::with_capacity(512);
    for part in [
        m.id,
        m.category,
        m.morphism,
        m.owner,
        m.summary,
        m.arguments,
        m.teleology,
        m.domain,
        m.codomain,
        m.units_and_frames,
    ] {
        s.push_str(part);
        s.push(' ');
    }
    for alias in m.aliases {
        s.push_str(alias);
        s.push(' ');
    }
    for other in m.composable_with {
        s.push_str(other);
        s.push(' ');
    }
    if let Some((_, extra)) = EXTRA.iter().find(|(id, _)| *id == m.id) {
        s.push_str(extra);
        s.push(' ');
    }
    // Discovery aliases for agents who still type a third-engine name.
    if m.category == "kinetics" || m.id.contains("route_input") || m.id.contains("compile_10d_pbr")
    {
        s.push_str(" godot unity");
    }
    if m.category == "manifold" || m.category == "locus" || m.category == "surface" {
        s.push_str(" blender cad");
    }
    s.to_ascii_lowercase()
}

fn tool_matches(tool: &ToolHit, tokens: &[String]) -> bool {
    if tokens.is_empty() {
        return true;
    }
    let hay = format!("{} {}", tool.name, tool.haystack).to_ascii_lowercase();
    tokens.iter().all(|t| hay.contains(t.as_str()) || tool.name.contains(t.as_str()))
}

pub fn search(q: &str, limit: usize) -> Value {
    let tokens = parse_tokens(q);
    let limit = limit.clamp(1, 150);
    let morphisms: Vec<Value> = MORPHISMS
        .iter()
        .filter(|m| morphism_matches(m, &tokens))
        .take(limit)
        .map(|m| {
            json!({
                "id": m.id,
                "category": m.category,
                "summary": m.summary,
                "arguments": m.arguments,
                "owner": m.owner,
                "invoke": format!("capability.invoke(\"{}\", {{ ... }}) inside synth_produce_present", m.id),
            })
        })
        .collect();
    let tools: Vec<Value> = TOOLS
        .iter()
        .filter(|t| tool_matches(t, &tokens) || tokens.is_empty())
        .map(|t| {
            json!({
                "name": t.name,
                "invoke": t.invoke,
            })
        })
        .collect();
    let notes: Vec<&str> = NOTES
        .iter()
        .filter(|n| {
            tokens.is_empty()
                || tokens
                    .iter()
                    .any(|t| n.tokens.iter().any(|k| *k == t.as_str() || t.contains(k) || k.contains(t.as_str())))
        })
        .map(|n| n.text)
        .collect();
    json!({
        "schema": SCHEMA,
        "product": product_card(),
        "q": q,
        "tokens": tokens,
        "returned": morphisms.len(),
        "limit": limit,
        "morphisms": morphisms,
        "tools": tools,
        "notes": notes,
        "how_to_invoke": "capability.invoke inside synth_produce_present source. vibe_invoke is a different generic Poet snapshot.",
    })
}

pub fn categories_for_agents() -> Value {
    let mut body = crate::production::assets::ontology::categories_json();
    if let Some(cats) = body.get_mut("categories").and_then(Value::as_array_mut) {
        for cat in cats {
            let id = cat.get("id").and_then(Value::as_str).unwrap_or("").to_owned();
            let examples: Vec<&str> = MORPHISMS
                .iter()
                .filter(|m| m.category == id)
                .take(6)
                .map(|m| m.id)
                .collect();
            cat["example_morphisms"] = json!(examples);
            cat["find"] = json!(format!("synth_find q={id}  or  synth_operations category={id}"));
        }
    }
    body["product"] = product_card();
    body["find"] = json!("synth_find q=xpbd|nla|ik|navmesh|gis|n3");
    body
}

pub fn find_arg_schema() -> Value {
    json!({
        "q": {
            "type": "string",
            "maxLength": 128,
            "description": "Keyword: xpbd, nla, ik, navmesh, camera, input, gis, osm, n3, sparql, hmc, q42, kinetics. Case-insensitive."
        },
        "limit": {
            "type": "integer",
            "minimum": 1,
            "maximum": 150,
            "description": "Max morphism hits. Default 20."
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_xpbd_lands_on_island_step() {
        let hit = search("xpbd", 20);
        let ids: Vec<_> = hit["morphisms"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|m| m["id"].as_str())
            .collect();
        assert!(ids.contains(&"hypermedia.kinetics.step_island_xpbd"));
        assert_eq!(hit["product"]["role"], ROLE);
        assert_eq!(hit["product"]["version"], RELEASE_VERSION);
    }

    #[test]
    fn find_nla_and_navmesh_and_n3() {
        let nla = search("nla", 20);
        assert!(nla["morphisms"].as_array().unwrap().iter().any(|m| {
            m["id"] == "hypermedia.kinetics.evaluate_nla_channelbag"
        }));
        let nav = search("navmesh", 20);
        assert!(nav["morphisms"].as_array().unwrap().iter().any(|m| {
            m["id"] == "hypermedia.kinetics.navigate_funnel_orca"
        }));
        let n3 = search("n3", 20);
        assert!(n3["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "synth_compile_q42"));
        assert!(n3["notes"].as_array().unwrap().iter().any(|n| {
            n.as_str().unwrap().contains("N3")
        }));
    }

    #[test]
    fn find_godot_still_returns_kinetics_as_qualia() {
        let hit = search("godot", 20);
        assert!(hit["morphisms"].as_array().unwrap().iter().any(|m| {
            m["id"].as_str().unwrap().starts_with("hypermedia.kinetics.")
        }));
        assert!(hit["notes"].as_array().unwrap().iter().any(|n| {
            n.as_str().unwrap().contains("QualiaDB")
        }));
    }

    #[test]
    fn find_hmc_explains_external_drop() {
        let hit = search("hmc", 20);
        assert!(hit["notes"].as_array().unwrap().iter().any(|n| {
            n.as_str().unwrap().contains("external HMC")
        }));
        assert!(hit["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "synth_produce_present"));
    }

    #[test]
    fn find_operator_and_update_and_antigravity() {
        let op = search("operator", 20);
        assert!(op["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "synth_operator_status"));
        let up = search("upgrade", 20);
        assert!(up["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "synth_update"));
        let ag = search("antigravity", 20);
        assert!(ag["notes"].as_array().unwrap().iter().any(|n| {
            n.as_str().unwrap().contains("vendor/antigravity")
        }));
    }
}
