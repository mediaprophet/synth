//! Aether **desktop** operator window + OS tray icon.
//! `tray-icon` is Windows, macOS, and Linux (not iOS/Android).
use super::snapshot;
use crate::web_server::open_in_system_browser;
use aether::desktop::{
    trayicon::{default_tray_icon, init_tray_icon},
    window, WindowCloseBehaviour,
};
use aether::prelude::*;

pub fn launch(studio_url: String) {
    let operator_url = format!("{studio_url}/operator");
    std::env::set_var("SYNTH_OPERATOR_STUDIO_URL", &studio_url);
    std::env::set_var("SYNTH_OPERATOR_URL", &operator_url);
    aether::LaunchBuilder::desktop().launch(operator_root);
}

fn studio_url() -> String {
    std::env::var("SYNTH_OPERATOR_STUDIO_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".into())
}

fn operator_url() -> String {
    std::env::var("SYNTH_OPERATOR_URL").unwrap_or_else(|_| "http://127.0.0.1:8787/operator".into())
}

fn operator_root() -> Element {
    let mut status = use_signal(|| serde_json::to_string_pretty(&snapshot()).unwrap_or_default());
    let mut message = use_signal(String::new);
    use_hook(|| {
        window().set_close_behavior(WindowCloseBehaviour::WindowHides);
        init_tray_icon(default_tray_icon(), None);
    });
    let studio = studio_url();
    let operator = operator_url();
    rsx! {
        div {
            style: "font-family: ui-sans-serif, system-ui; background:#0b1412; color:#d7eee6; min-height:100vh; padding:20px;",
            h1 { style: "margin:0 0 8px 0;", "Synth operator" }
            p { style: "color:#8fb5a7;", "Aether desktop tray on Windows, macOS, and Linux. Close hides to the tray; left-click restores. Browser workbench stays on 127.0.0.1." }
            p { style: "color:#fbbf24;", "{message}" }
            div { style: "display:flex; gap:8px; flex-wrap:wrap; margin:12px 0;",
                button {
                    onclick: move |_| {
                        open_in_system_browser(&operator);
                        message.set(format!("Opened {operator}"));
                    },
                    "Open operator page"
                }
                button {
                    onclick: move |_| {
                        open_in_system_browser(&studio);
                        message.set(format!("Opened {studio}"));
                    },
                    "Open studio"
                }
                button {
                    onclick: move |_| {
                        status.set(serde_json::to_string_pretty(&snapshot()).unwrap_or_default());
                        message.set("Refreshed operator snapshot".into());
                    },
                    "Refresh status"
                }
            }
            h2 { "Biosphere rule" }
            p { "Non-fiction living taxa require sourced .q42 evidence (synth_compile_q42 + query_sparql) before modelling. Encyclopedia is Ecopunk." }
            h2 { "Status" }
            pre { style: "background:#12211e; padding:12px; border-radius:8px; overflow:auto; max-height:55vh; white-space:pre-wrap;", "{status}" }
        }
    }
}
