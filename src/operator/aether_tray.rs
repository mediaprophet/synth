//! Webizen AI Synth operator window + OS tray icon and menu.
//! Uses the Aether desktop toolkit. `tray-icon` is Windows, macOS, and Linux.
use super::{branding, snapshot};
use crate::web_server::open_in_system_browser;
use aether::desktop::{
    icon_from_memory,
    trayicon::{
        menu::{Menu, MenuItem, PredefinedMenuItem},
        init_tray_icon, AetherTrayIcon,
    },
    use_tray_menu_event_handler, window, Config, WindowBuilder, WindowCloseBehaviour,
};
use aether::prelude::*;

#[derive(Clone)]
struct TrayMenuIds {
    show: String,
    operator: String,
    studio: String,
    exit: String,
}

pub fn launch(studio_url: String) {
    let operator_url = format!("{studio_url}/operator");
    std::env::set_var("SYNTH_OPERATOR_STUDIO_URL", &studio_url);
    std::env::set_var("SYNTH_OPERATOR_URL", &operator_url);
    let mut cfg = Config::new()
        .with_window(WindowBuilder::new().with_title(branding::APP_DISPLAY_NAME))
        .with_exits_when_last_window_closes(false)
        .with_tray_icon_show_window_on_click(true);
    if let Ok(icon) = icon_from_memory::<aether::desktop::tao::window::Icon>(branding::ICON_PNG) {
        cfg = cfg.with_icon(icon);
    }
    aether::LaunchBuilder::desktop()
        .with_cfg(cfg)
        .launch(operator_root);
}

fn studio_url() -> String {
    std::env::var("SYNTH_OPERATOR_STUDIO_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".into())
}

fn operator_url() -> String {
    std::env::var("SYNTH_OPERATOR_URL").unwrap_or_else(|_| "http://127.0.0.1:8787/operator".into())
}

fn exit_operator() {
    aether::desktop::app().shutdown();
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(250));
        std::process::exit(0);
    });
}

fn tray_icon() -> Option<AetherTrayIcon> {
    icon_from_memory::<AetherTrayIcon>(branding::ICON_32_PNG)
        .or_else(|_| icon_from_memory::<AetherTrayIcon>(branding::ICON_64_PNG))
        .ok()
}

fn operator_root() -> Element {
    let mut status = use_signal(|| serde_json::to_string_pretty(&snapshot()).unwrap_or_default());
    let mut message = use_signal(|| {
        format!(
            "Running · pid {} · {}",
            std::process::id(),
            operator_url()
        )
    });
    let ids = use_hook(|| {
        window().set_close_behavior(WindowCloseBehaviour::WindowHides);
        window().set_title(branding::APP_DISPLAY_NAME);
        let show = MenuItem::new("Show Webizen AI Synth", true, None);
        let operator = MenuItem::new("Open operator page", true, None);
        let studio = MenuItem::new("Open studio", true, None);
        let exit = MenuItem::new("Exit", true, None);
        let ids = TrayMenuIds {
            show: show.id().as_ref().to_string(),
            operator: operator.id().as_ref().to_string(),
            studio: studio.id().as_ref().to_string(),
            exit: exit.id().as_ref().to_string(),
        };
        let menu = Menu::new();
        let _ = menu.append_items(&[
            &show,
            &operator,
            &studio,
            &PredefinedMenuItem::separator(),
            &exit,
        ]);
        let tray = init_tray_icon(menu, tray_icon());
        let _ = tray.set_tooltip(Some(branding::APP_DISPLAY_NAME));
        ids
    });
    let studio = studio_url();
    let operator = operator_url();
    {
        let studio = studio.clone();
        let operator = operator.clone();
        let ids = ids.clone();
        use_tray_menu_event_handler(move |event| {
            let id = event.id.as_ref();
            if id == ids.show {
                window().window.set_visible(true);
                window().window.set_focus();
            } else if id == ids.operator {
                window().window.set_visible(true);
                open_in_system_browser(&operator);
            } else if id == ids.studio {
                open_in_system_browser(&studio);
            } else if id == ids.exit {
                exit_operator();
            }
        });
    }
    let emblem = branding::emblem_data_uri();
    let banner = branding::banner_data_uri();
    rsx! {
        div {
            style: "font-family: ui-sans-serif, system-ui; background:#0b1412; color:#d7eee6; min-height:100vh; padding:20px;",
            div { style: "display:flex; align-items:center; gap:14px; margin-bottom:12px;",
                img { src: "{emblem}", width: "48", height: "48", alt: "Webizen AI Synth" }
                div {
                    h1 { style: "margin:0 0 4px 0;", "Webizen AI Synth" }
                    p { style: "margin:0; color:#8fb5a7;", "Operator workbench · system tray is running" }
                }
                img { src: "{banner}", style: "width:280px; height:98px; border-radius:10px; border:1px solid #1e3330;", alt: "Webizen AI Synth banner" }
            }
            p { style: "color:#8fb5a7;", "Close hides to the tray (look for the emerald Webizen mark). Right-click the tray icon for Show / Operator / Studio / Exit. Left-click restores this window." }
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
                        window().window.set_visible(false);
                        message.set("Hidden to the system tray. Right-click the tray icon to Exit.".into());
                    },
                    "Hide to tray"
                }
                button {
                    onclick: move |_| {
                        status.set(serde_json::to_string_pretty(&snapshot()).unwrap_or_default());
                        message.set(format!("Running · pid {} · {}", std::process::id(), operator_url()));
                    },
                    "Refresh status"
                }
                button {
                    style: "background:#2c1618; color:#fca5a5; border:1px solid #ef4444; border-radius:6px; padding:7px 12px;",
                    onclick: move |_| exit_operator(),
                    "Exit"
                }
            }
            h2 { "Biosphere rule" }
            p { "Non-fiction living taxa require sourced .q42 evidence (synth_compile_q42 + query_sparql) before modelling. Encyclopedia is Ecopunk." }
            h2 { "Status" }
            pre { style: "background:#12211e; padding:12px; border-radius:8px; overflow:auto; max-height:55vh; white-space:pre-wrap;", "{status}" }
        }
    }
}
