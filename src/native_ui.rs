//! Webizen AI Synth studio window and resident tray, using the Aether toolkit.
//!
//! Renders the full 6-pillar Synth Studio & Substantive Capabilities Workbench,
//! automatically starts the non-conflicting local HTTP browser server (`127.0.0.1:8787`),
//! supervises authenticated [`SynthHost`] sessions, inspects local `C:\LLM_files`
//! model residency (`Qwen` / `Gemma 4`), and supports hiding to the background.
use crate::{
    present_capabilities::{classify_mcp_domain, format_bytes_human, FullCapabilitiesSnapshot},
    tray_env::{bootstrap_environment, discover_local_models, DiscoveredModel, ProducedEnvironment},
    web_server::{open_in_system_browser, spawn_background_server},
    SynthHost,
};
use aether::prelude::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

#[derive(Clone, Default)]
struct View {
    resources: Vec<String>,
    status: String,
}

#[derive(Props, Clone)]
struct MonitorProps {
    host: Option<Arc<Mutex<SynthHost>>>,
    cancellation: Option<Arc<AtomicBool>>,
    environment: ProducedEnvironment,
    portal_url: String,
}

impl PartialEq for MonitorProps {
    fn eq(&self, other: &Self) -> bool {
        let host_eq = match (&self.host, &other.host) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        let cancel_eq = match (&self.cancellation, &other.cancellation) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        host_eq
            && cancel_eq
            && self.portal_url == other.portal_url
            && self.environment.candidate_digest_hex == other.environment.candidate_digest_hex
    }
}

pub fn launch(host: Arc<Mutex<SynthHost>>) -> Result<(), String> {
    launch_with_cancellation(host, None)
}

pub fn launch_with_cancellation(
    host: Arc<Mutex<SynthHost>>,
    cancellation: Option<Arc<AtomicBool>>,
) -> Result<(), String> {
    host.try_lock()
        .map_err(|_| "Synth host is busy")?
        .inspect()?;
    let environment = bootstrap_environment()?;
    let portal_url =
        spawn_background_server(None).unwrap_or_else(|_| "http://127.0.0.1:8787".to_owned());
    let attributes = aether_native::WindowAttributes::default()
        .with_title(format!(
            "Webizen AI Synth · {} · Studio ({portal_url})",
            environment.mode.label()
        ))
        .with_surface_size(aether_native::LogicalSize::new(1280, 860));
    aether_native::launch_cfg_with_props(
        Monitor,
        MonitorProps {
            host: Some(host),
            cancellation,
            environment,
            portal_url,
        },
        Vec::new(),
        vec![Box::new(attributes)],
    );
    Ok(())
}

/// Launches the standalone Webizen AI Synth tray & studio environment produced by Synth itself
/// (`examples/tray_environment.vibe`) and starts the local browser interface on `127.0.0.1:8787`.
pub fn launch_standalone_environment(
    cancellation: Option<Arc<AtomicBool>>,
) -> Result<(), String> {
    let environment = bootstrap_environment()?;
    let portal_url =
        spawn_background_server(None).unwrap_or_else(|_| "http://127.0.0.1:8787".to_owned());
    let attributes = aether_native::WindowAttributes::default()
        .with_title(format!(
            "Webizen AI Synth · {} · {portal_url}",
            environment.mode.label()
        ))
        .with_surface_size(aether_native::LogicalSize::new(1280, 860));
    aether_native::launch_cfg_with_props(
        Monitor,
        MonitorProps {
            host: None,
            cancellation,
            environment,
            portal_url,
        },
        Vec::new(),
        vec![Box::new(attributes)],
    );
    Ok(())
}

#[allow(non_snake_case)]
fn Monitor(props: MonitorProps) -> Element {
    let snap = use_hook(FullCapabilitiesSnapshot::collect);
    let mut view = use_signal(|| View {
        status: format!(
            "Mode: {} · Web Portal: {} · 12 Ontology Cats ({} Morphisms) · {} MCP Tools · {} Vibe SDK Providers · {} Living-World Pins",
            props.environment.mode.label(),
            props.portal_url,
            crate::production::assets::ontology::MORPHISMS.len(),
            props.environment.ecosystem.total_mcp_tools,
            props.environment.ecosystem.vibe_sdk_families,
            props.environment.ecosystem.living_world_entries,
        ),
        resources: props
            .environment
            .resources
            .iter()
            .map(|r| {
                format!(
                    "{} · {} · {} bytes · sha256:{}",
                    r.key,
                    r.kind,
                    r.bytes,
                    &r.sha256[..12]
                )
            })
            .collect(),
    });
    let mut message = use_signal(String::new);
    let mut active_tab = use_signal(|| "environment".to_owned());
    let mut ontology_cat = use_signal(|| "all".to_owned());
    let mut mcp_domain = use_signal(|| "all".to_owned());
    let mut models_signal = use_signal(|| props.environment.discovered_models.clone());

    let polling = props.host.clone();
    let fallback_env = props.environment.clone();
    let portal_for_poll = props.portal_url.clone();

    use_future(move || {
        let host_opt = polling.clone();
        let env = fallback_env.clone();
        let portal = portal_for_poll.clone();
        async move {
            loop {
                if let Some(host) = &host_opt {
                    let next = match host.try_lock() {
                        Ok(h) => match h.inspect() {
                            Ok(s) => View {
                                status: format!(
                                    "Session {} ({:?}) · rev {} · {}@{} · {} · Portal {}",
                                    s.actor_subject,
                                    s.actor_kind,
                                    s.context.document.revision,
                                    s.producer,
                                    s.producer_version,
                                    env.mode.label(),
                                    portal
                                ),
                                resources: s
                                    .resources
                                    .iter()
                                    .map(|(key, kind, bytes)| {
                                        format!("{key} · {kind} · {bytes} bytes")
                                    })
                                    .collect(),
                            },
                            Err(_) => View {
                                status:
                                    "Native source or session unavailable; supervision controls held"
                                        .into(),
                                ..Default::default()
                            },
                        },
                        Err(_) => View {
                            status: "Agent operation running".into(),
                            ..Default::default()
                        },
                    };
                    view.set(next);
                }
                futures_timer::Delay::new(std::time::Duration::from_secs(1)).await;
            }
        }
    });

    let displayed = view.read().clone();
    let cancel_work = props.cancellation.clone();
    let env = &props.environment;
    let window_handle = aether_native::use_window();
    let tab = active_tab.read().clone();
    let selected_cat = ontology_cat.read().clone();
    let selected_dom = mcp_domain.read().clone();
    let current_models = models_signal.read().clone();
    let portal_url_click = props.portal_url.clone();
    let operator_url_display = format!("{}/operator", props.portal_url);
    let operator_url_click = operator_url_display.clone();

    let categories = snap.categories["categories"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let morphisms = snap.morphisms["morphisms"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let nquin_count = crate::production::assets::ontology::ONTOLOGY_NQUINS.len();
    let providers = snap.vibe_sdk["providers"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let pins = snap.living_world["pins"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    rsx! {
        main {
            style: "font-family: system-ui, sans-serif; background: #0b1318; color: #e5f1ea; min-height: 100vh; padding: 22px;",
            header {
                style: "display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid #1e3330; padding-bottom: 14px; margin-bottom: 16px;",
                div {
                    style: "display: flex; align-items: center; gap: 14px;",
                    img {
                        src: "{env.emblem_svg_data_uri}",
                        width: "48",
                        height: "48",
                        alt: "Synth Emblem"
                    }
                    div {
                        h1 { style: "margin: 0; font-size: 21px;", "Webizen AI Synth · Sovereign Asset, Ontology & MCP Studio" }
                        p { style: "margin: 4px 0 0 0; color: #8fb5a7; font-size: 12px;", "{displayed.status}" }
                    }
                }
                div {
                    style: "display: flex; gap: 8px; flex-wrap: wrap;",
                    button {
                        style: "background: #0f293a; color: #7dd3fc; border: 1px solid #0284c7; border-radius: 6px; padding: 7px 12px; cursor: pointer;",
                        onclick: move |_| {
                            open_in_system_browser(&portal_url_click);
                            message.set(format!("Opened {} in system browser", portal_url_click));
                        },
                        "Open Browser Studio ({props.portal_url})"
                    }
                    button {
                        style: "background: #142523; color: #a7f3d0; border: 1px solid #2b4c46; border-radius: 6px; padding: 7px 12px; cursor: pointer;",
                        onclick: move |_| {
                            open_in_system_browser(&operator_url_click);
                            message.set(format!("Opened {operator_url_click}"));
                        },
                        "Operator ({operator_url_display})"
                    }
                    button {
                        style: "background: #132824; color: #a7f3d0; border: 1px solid #10b981; border-radius: 6px; padding: 7px 12px; cursor: pointer;",
                        onclick: move |_| {
                            window_handle.set_visible(false);
                            message.set("Window hidden to resident tray loop (browser server remains active)".into());
                        },
                        "Hide to Tray"
                    }
                    button {
                        style: "background: #2c1618; color: #fca5a5; border: 1px solid #ef4444; border-radius: 6px; padding: 7px 12px; cursor: pointer;",
                        disabled: props.cancellation.is_none(),
                        onclick: move |_| {
                            if let Some(cancel) = &cancel_work {
                                cancel.store(true, Ordering::Release);
                                message.set("Cancellation requested for the current preparation".into());
                            }
                        },
                        "Cancel preparation"
                    }
                }
            }

            div {
                style: "display: flex; align-items: center; gap: 16px; margin-bottom: 16px;",
                img {
                    src: "{env.banner_png_data_uri}",
                    style: "width: 280px; height: 98px; border-radius: 10px; border: 1px solid #1e3330;",
                    alt: "Synth Self-Produced Studio Banner"
                }
                div {
                    style: "display: flex; gap: 10px; flex-wrap: wrap;",
                    div { style: "background: #12211e; border: 1px solid #233e38; border-radius: 8px; padding: 10px 14px;",
                        p { style: "margin: 0; font-size: 11px; color: #8fb5a7;", "CATEGORICAL ONTOLOGY" }
                        p { style: "margin: 4px 0 0 0; font-size: 15px; font-weight: bold; color: #34d399;", "12 Cats · {morphisms.len()} Morphisms · {nquin_count} NQuins" }
                    }
                    div { style: "background: #12211e; border: 1px solid #233e38; border-radius: 8px; padding: 10px 14px;",
                        p { style: "margin: 0; font-size: 11px; color: #8fb5a7;", "UNIFIED MCP SUITE" }
                        p { style: "margin: 4px 0 0 0; font-size: 15px; font-weight: bold; color: #38bdf8;", "{snap.mcp_tools.len()} Tools (QualiaDB 0.0.41 + Synth)" }
                    }
                    div { style: "background: #12211e; border: 1px solid #233e38; border-radius: 8px; padding: 10px 14px;",
                        p { style: "margin: 0; font-size: 11px; color: #8fb5a7;", "ETALON-VIBE SDK & LIVING WORLD" }
                        p { style: "margin: 4px 0 0 0; font-size: 15px; font-weight: bold; color: #fbbf24;", "{providers.len()} SDK Providers · {pins.len()} Anatomical Pins" }
                    }
                    div { style: "background: #12211e; border: 1px solid #233e38; border-radius: 8px; padding: 10px 14px;",
                        p { style: "margin: 0; font-size: 11px; color: #8fb5a7;", "LOCAL LLM VAULT ({env.vault_root})" }
                        p { style: "margin: 4px 0 0 0; font-size: 15px; font-weight: bold; color: #a7f3d0;", "{current_models.len()} Models (Qwen3.5 & Gemma 4)" }
                    }
                }
            }

            div {
                style: "display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 16px;",
                button {
                    style: "background: #142523; color: #e5f1ea; border: 1px solid #2b4c46; border-radius: 6px; padding: 7px 12px;",
                    onclick: move |_| active_tab.set("environment".into()),
                    "1. Environment & Session ({displayed.resources.len()})"
                }
                button {
                    style: "background: #142523; color: #e5f1ea; border: 1px solid #2b4c46; border-radius: 6px; padding: 7px 12px;",
                    onclick: move |_| active_tab.set("ontology".into()),
                    "2. Ontology (12 Cats · {morphisms.len()} Morphisms)"
                }
                button {
                    style: "background: #142523; color: #e5f1ea; border: 1px solid #2b4c46; border-radius: 6px; padding: 7px 12px;",
                    onclick: move |_| active_tab.set("mcp".into()),
                    "3. MCP Tool Suite ({snap.mcp_tools.len()})"
                }
                button {
                    style: "background: #142523; color: #e5f1ea; border: 1px solid #2b4c46; border-radius: 6px; padding: 7px 12px;",
                    onclick: move |_| active_tab.set("sdk".into()),
                    "4. Etalon-Vibe SDK ({providers.len()})"
                }
                button {
                    style: "background: #142523; color: #e5f1ea; border: 1px solid #2b4c46; border-radius: 6px; padding: 7px 12px;",
                    onclick: move |_| active_tab.set("living".into()),
                    "5. Living-World & Cosmic ({pins.len()} Pins)"
                }
                button {
                    style: "background: #142523; color: #e5f1ea; border: 1px solid #2b4c46; border-radius: 6px; padding: 7px 12px;",
                    onclick: move |_| active_tab.set("models".into()),
                    "6. Local LLM Vault ({current_models.len()})"
                }
            }

            if tab == "environment" {
                section {
                    h2 { "Self-Produced Tray & Studio Environment (examples/tray_environment.vibe)" }
                    p { "Domains: World of God (Flora · Fauna · Biosphere · Space) · World of Man (Cars · Streets · Houses · BIM) · World of Mind (↳ Creative Works [schema:CreativeWork] · ↳ Software & Systems [schema:SoftwareApplication])" }
                    p { "Candidate SHA-256: {env.candidate_digest_hex} · Local Web Workbench: {props.portal_url}" }
                    for res in env.resources.iter() {
                        p { "{res.key} · {res.kind} · {res.bytes} bytes · sha256:{res.sha256}" }
                    }
                    h3 { "Active Host Session Resources" }
                    for resource in displayed.resources {
                        p { "{resource}" }
                    }
                }
            } else if tab == "ontology" {
                section {
                    h2 { "Categorical Ontology · 12 Categories · {morphisms.len()} Morphisms · {nquin_count} NQuin48 Triples" }
                    div {
                        style: "display: flex; gap: 6px; flex-wrap: wrap; margin-bottom: 12px;",
                        button {
                            style: "background: #18302c; color: #6ee7b7; border: 1px solid #10b981; border-radius: 4px; padding: 4px 10px;",
                            onclick: move |_| ontology_cat.set("all".into()),
                            "All ({morphisms.len()})"
                        }
                        for cat in categories.iter() {
                            {
                                let cid = cat["id"].as_str().unwrap_or("").to_owned();
                                let cid_click = cid.clone();
                                let cnt = cat["admitted_morphism_count"].as_u64().unwrap_or(0);
                                rsx! {
                                    button {
                                        style: "background: #142523; color: #e5f1ea; border: 1px solid #2b4c46; border-radius: 4px; padding: 4px 10px;",
                                        onclick: move |_| ontology_cat.set(cid_click.clone()),
                                        "{cid} ({cnt})"
                                    }
                                }
                            }
                        }
                    }
                    for m in morphisms.iter().filter(|m| selected_cat == "all" || m["category"].as_str() == Some(selected_cat.as_str())) {
                        {
                            let id = m["id"].as_str().unwrap_or("");
                            let cat = m["category"].as_str().unwrap_or("");
                            let dom = m["agentic_utility"]["domain"].as_str().unwrap_or("");
                            let cod = m["agentic_utility"]["codomain"].as_str().unwrap_or("");
                            let summary = m["summary"].as_str().unwrap_or("");
                            rsx! {
                                div {
                                    style: "background: #12211e; border: 1px solid #1f3833; border-radius: 6px; padding: 8px 12px; margin-bottom: 6px;",
                                    p { style: "margin: 0; font-weight: bold; color: #34d399;", "{id} [{cat}] — {summary}" }
                                    p { style: "margin: 3px 0 0 0; font-size: 12px; color: #9ab5a7;", "Domain: {dom} -> Codomain: {cod}" }
                                }
                            }
                        }
                    }
                }
            } else if tab == "mcp" {
                section {
                    h2 { "Unified QualiaDB 0.0.41 + Synth MCP Suite ({snap.mcp_tools.len()} Tools)" }
                    div {
                        style: "display: flex; gap: 6px; flex-wrap: wrap; margin-bottom: 12px;",
                        for (dom_id, label) in [
                            ("all", "All Tools"),
                            ("synth-authoring", "Synth & Studio Authoring"),
                            ("vibe-sdk", "VibeScript & WGSL Forge"),
                            ("living-cosmic", "Living-World & Cosmic (0.0.41)"),
                            ("semantic-logic", "SPARQL, SHACL & Modalities"),
                            ("science-sim", "Sim, Geo, Vision, Audio & Quantum"),
                        ] {
                            {
                                let d = dom_id.to_owned();
                                rsx! {
                                    button {
                                        style: "background: #142523; color: #e5f1ea; border: 1px solid #2b4c46; border-radius: 4px; padding: 4px 10px;",
                                        onclick: move |_| mcp_domain.set(d.clone()),
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                    for tool in snap.mcp_tools.iter().filter(|t| selected_dom == "all" || classify_mcp_domain(&t.name) == selected_dom) {
                        div {
                            style: "background: #12211e; border: 1px solid #1f3833; border-radius: 6px; padding: 8px 12px; margin-bottom: 6px;",
                            p { style: "margin: 0; font-weight: bold; color: #38bdf8;", "{tool.name} [{classify_mcp_domain(&tool.name)}]" }
                            p { style: "margin: 3px 0 0 0; font-size: 12px; color: #9ab5a7;", "{tool.description}" }
                        }
                    }
                }
            } else if tab == "sdk" {
                section {
                    h2 { "Admitted Etalon-Vibe SDK Providers ({providers.len()} Providers · 5-Field NQuin48 Parity Verified)" }
                    for p in providers.iter() {
                        {
                            let alias = p["alias"].as_str().unwrap_or("");
                            let iri = p["iri"].as_str().unwrap_or("");
                            let quins = p["ontology_records"].as_u64().unwrap_or(0);
                            let fns = p["functions"].as_array().cloned().unwrap_or_default();
                            let fn_list: Vec<String> = fns.iter().filter_map(|f| f["path"].as_str().map(str::to_owned)).collect();
                            rsx! {
                                div {
                                    style: "background: #12211e; border: 1px solid #1f3833; border-radius: 6px; padding: 8px 12px; margin-bottom: 6px;",
                                    p { style: "margin: 0; font-weight: bold; color: #34d399;", "vibe:0.3/{alias} ({quins} NQuin48 triples · {fn_list.len()} functions) — {iri}" }
                                    p { style: "margin: 3px 0 0 0; font-size: 12px; color: #9ab5a7;", "{fn_list.join(\" · \")}" }
                                }
                            }
                        }
                    }
                }
            } else if tab == "living" {
                section {
                    h2 { "Living-World Biological/Anatomical Engine ({pins.len()} Canonical Pins) & Omniversal Cosmic Coordinates (OCS v2.2.0)" }
                    p { "Botanical Golden Angle: 2.399963 rad · Eucalyptus Pipe Exponent: 2.49 · Vertebrate Max Joints: 64 (Cyclic RMF) · Celestial Reference: Earth & Mars (J2000)" }
                    for pin in pins.iter() {
                        {
                            let pid = pin["pin_id"].as_str().unwrap_or("");
                            let curie = pin["curie"].as_str().unwrap_or("");
                            let term = pin["scientific_term"].as_str().unwrap_or("");
                            let scale = pin["scale"].as_u64().unwrap_or(0);
                            rsx! {
                                div {
                                    style: "background: #12211e; border: 1px solid #1f3833; border-radius: 6px; padding: 7px 12px; margin-bottom: 5px;",
                                    p { style: "margin: 0; font-size: 13px;", "{pid} [Scale {scale} · {curie}] — {term}" }
                                }
                            }
                        }
                    }
                }
            } else {
                section {
                    div {
                        style: "display: flex; align-items: center; justify-content: space-between;",
                        h2 { "Local LLM Model Vault · {env.vault_root}" }
                        button {
                            style: "background: #132824; color: #a7f3d0; border: 1px solid #10b981; border-radius: 6px; padding: 6px 12px; cursor: pointer;",
                            onclick: move |_| {
                                let (_, refreshed) = discover_local_models();
                                let cnt = refreshed.len();
                                models_signal.set(refreshed);
                                message.set(format!("Rescanned vault: {cnt} model artifacts discovered"));
                            },
                            "Rescan Vault"
                        }
                    }
                    p { "Target architectures: Qwen3.5-4B/9B-NVFP4, Qwen3.6-35B-A3B-NVFP4, Gemma-4-E4B-NVFP4, Gemma-4-26B-A4B-NVFP4" }
                    for DiscoveredModel { name, family, format, path, size_bytes } in current_models.iter() {
                        div {
                            style: "background: #12211e; border: 1px solid #1f3833; border-radius: 6px; padding: 8px 12px; margin-bottom: 6px;",
                            p { style: "margin: 0; font-weight: bold; color: #fbbf24;", "{name} [{family} · {format}] — {format_bytes_human(*size_bytes)}" }
                            p { style: "margin: 3px 0 0 0; font-size: 12px; color: #9ab5a7;", "{path}" }
                        }
                    }
                }
            }

            p { role: "status", style: "color: #34d399; margin-top: 14px;", "{message}" }
        }
    }
}
