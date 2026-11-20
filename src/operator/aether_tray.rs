//! Webizen AI Synth as an Aether desktop application.
//!
//! Aether is a Dioxus-fork toolkit: `LaunchBuilder`, `Config`, window, native
//! menu, tray, and `app().shutdown()`. This module is the app that fills those
//! in. Toolkit placeholders ("Aether App", fallback 460×460 icon, demo Quit
//! tray) stay unused.
use super::{branding, snapshot};
use crate::web_server::open_in_system_browser;
use aether::desktop::{
    icon_from_memory,
    muda::{Menu as NativeMenu, MenuItem as NativeMenuItem, PredefinedMenuItem, Submenu},
    trayicon::{
        menu::{Menu as TrayMenu, MenuItem as TrayMenuItem, PredefinedMenuItem as TraySeparator},
        init_tray_icon, AetherTrayIcon,
    },
    use_muda_event_handler, use_tray_menu_event_handler, window, Config, LogicalSize,
    WindowBuilder, WindowCloseBehaviour,
};
use aether::prelude::*;

const ID_SHOW: &str = "synth-show";
const ID_OPERATOR: &str = "synth-open-operator";
const ID_STUDIO: &str = "synth-open-studio";
const ID_UPGRADE: &str = "synth-upgrade";
const ID_HIDE: &str = "synth-hide-tray";
const ID_EXIT: &str = "synth-exit";

pub fn launch(studio_url: String) {
    branding::claim_toolkit_identity();
    let operator_url = format!("{studio_url}/operator");
    std::env::set_var("SYNTH_OPERATOR_STUDIO_URL", &studio_url);
    std::env::set_var("SYNTH_OPERATOR_URL", &operator_url);
    aether::LaunchBuilder::desktop()
        .with_cfg(app_config())
        .launch(operator_root);
}

fn app_config() -> Config {
    let window = WindowBuilder::new()
        .with_title(branding::APP_DISPLAY_NAME)
        .with_inner_size(LogicalSize::new(760.0, 640.0));
    let mut cfg = Config::new()
        .with_window(window)
        .with_menu(app_menu_bar())
        .with_close_behaviour(WindowCloseBehaviour::WindowHides)
        .with_exits_when_last_window_closes(false)
        .with_tray_icon_show_window_on_click(true)
        .with_background_color((11, 20, 18, 255));
    if let Some(dir) = branding::webview_data_dir() {
        cfg = cfg.with_data_directory(dir);
    }
    if let Ok(icon) = icon_from_memory::<aether::desktop::tao::window::Icon>(branding::ICON_PNG) {
        cfg = cfg.with_icon(icon);
    }
    cfg
}

fn app_menu_bar() -> NativeMenu {
    let menu = NativeMenu::new();
    let app_menu = Submenu::new(branding::APP_DISPLAY_NAME, true);
    let _ = app_menu.append_items(&[
        &NativeMenuItem::with_id(ID_SHOW, "Show window", true, None),
        &NativeMenuItem::with_id(ID_OPERATOR, "Open operator page", true, None),
        &NativeMenuItem::with_id(ID_STUDIO, "Open studio", true, None),
        &NativeMenuItem::with_id(ID_UPGRADE, "Upgrade", true, None),
        &PredefinedMenuItem::separator(),
        &NativeMenuItem::with_id(ID_HIDE, "Hide to tray", true, None),
        &PredefinedMenuItem::separator(),
        &NativeMenuItem::with_id(ID_EXIT, "Exit", true, None),
    ]);
    let edit = Submenu::new("Edit", true);
    let _ = edit.append_items(&[
        &PredefinedMenuItem::undo(None),
        &PredefinedMenuItem::redo(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::cut(None),
        &PredefinedMenuItem::copy(None),
        &PredefinedMenuItem::paste(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::select_all(None),
    ]);
    let _ = menu.append_items(&[&app_menu, &edit]);
    menu
}

fn studio_url() -> String {
    std::env::var("SYNTH_OPERATOR_STUDIO_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".into())
}

fn operator_url() -> String {
    std::env::var("SYNTH_OPERATOR_URL").unwrap_or_else(|_| "http://127.0.0.1:8787/operator".into())
}

/// End the Aether event loop. Synth's HTTP worker is a non-daemon thread, so
/// follow shutdown with process exit.
fn exit_operator() {
    aether::desktop::app().shutdown();
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(250));
        std::process::exit(0);
    });
}

fn show_window() {
    window().window.set_visible(true);
    window().window.set_focus();
}

fn hide_window() {
    window().window.set_visible(false);
}

fn dispatch_app_command(id: &str) {
    match id {
        ID_SHOW => show_window(),
        ID_OPERATOR => {
            show_window();
            open_in_system_browser(&operator_url());
        }
        ID_STUDIO => open_in_system_browser(&studio_url()),
        ID_UPGRADE => {
            show_window();
            open_in_system_browser(&format!("{}#upgrade", operator_url()));
        }
        ID_HIDE => hide_window(),
        ID_EXIT => exit_operator(),
        _ => {}
    }
}

fn tray_icon() -> Option<AetherTrayIcon> {
    icon_from_memory::<AetherTrayIcon>(branding::ICON_32_PNG)
        .or_else(|_| icon_from_memory::<AetherTrayIcon>(branding::ICON_64_PNG))
        .ok()
}

fn tray_menu() -> TrayMenu {
    let show = TrayMenuItem::with_id(ID_SHOW, "Show Webizen AI Synth", true, None);
    let operator = TrayMenuItem::with_id(ID_OPERATOR, "Open operator page", true, None);
    let studio = TrayMenuItem::with_id(ID_STUDIO, "Open studio", true, None);
    let upgrade = TrayMenuItem::with_id(ID_UPGRADE, "Upgrade", true, None);
    let exit = TrayMenuItem::with_id(ID_EXIT, "Exit", true, None);
    let menu = TrayMenu::new();
    let _ = menu.append_items(&[
        &show,
        &operator,
        &studio,
        &upgrade,
        &TraySeparator::separator(),
        &exit,
    ]);
    menu
}

fn operator_root() -> Element {
    let mut status = use_signal(|| serde_json::to_string_pretty(&snapshot()).unwrap_or_default());
    let mut gpu_cfg = use_signal(crate::session_admission::load_gpu_selection_config);
    let mut message = use_signal(|| {
        format!(
            "Running · pid {} · {}",
            std::process::id(),
            operator_url()
        )
    });
    use_hook(|| {
        let tray = init_tray_icon(tray_menu(), tray_icon());
        let _ = tray.set_tooltip(Some(branding::APP_DISPLAY_NAME));
    });
    use_tray_menu_event_handler(move |event| {
        dispatch_app_command(event.id.as_ref());
    });
    use_muda_event_handler(move |event| {
        dispatch_app_command(event.id().as_ref());
    });
    let studio = studio_url();
    let operator = operator_url();
    let emblem = branding::emblem_data_uri();
    let banner = branding::banner_data_uri();
    let current_cfg = gpu_cfg.read().clone();
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
            p { style: "color:#8fb5a7;", "Close hides to the tray (look for the emerald Webizen mark). The Webizen AI Synth menu and the tray menu both Exit. Left-click the tray icon restores this window. Upgrade opens the operator page and downloads the latest signed GitHub Release next to this process; the running synth.exe is never overwritten." }
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
                    style: "background:#134e4a; color:#99f6e4; border:1px solid #2dd4bf; border-radius:6px; padding:7px 12px;",
                    onclick: move |_| {
                        let url = format!("{}#upgrade", operator_url());
                        open_in_system_browser(&url);
                        message.set("Opened Upgrade on the operator page. The running synth.exe is never overwritten.".into());
                    },
                    "Upgrade"
                }
                button {
                    onclick: move |_| {
                        hide_window();
                        message.set("Hidden to the system tray. Right-click the tray icon to Exit.".into());
                    },
                    "Hide to tray"
                }
                button {
                    onclick: move |_| {
                        gpu_cfg.set(crate::session_admission::load_gpu_selection_config());
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
            h2 { "GPU adapter & backend selection" }
            p { style: "color:#8fb5a7; margin:4px 0 8px 0;",
                "Active policy: preference = {current_cfg.preference} (reject_igpu = {current_cfg.reject_igpu}), backend = {current_cfg.backend}"
            }
            div { style: "display:flex; gap:8px; flex-wrap:wrap; margin:8px 0;",
                button {
                    style: if current_cfg.backend == "vulkan" { "background:#134e4a; color:#99f6e4; border:1px solid #2dd4bf; border-radius:6px; padding:7px 12px;" } else { "" },
                    onclick: move |_| {
                        let mut c = gpu_cfg.read().clone();
                        c.backend = "vulkan".into();
                        if let Ok(_) = crate::session_admission::save_and_apply_gpu_selection_config(c.clone()) {
                            gpu_cfg.set(c);
                            status.set(serde_json::to_string_pretty(&snapshot()).unwrap_or_default());
                            message.set("Selected GPU backend: Vulkan (Discrete GPU)".into());
                        }
                    },
                    "Backend: Vulkan"
                }
                button {
                    style: if current_cfg.backend == "cuda" { "background:#134e4a; color:#99f6e4; border:1px solid #2dd4bf; border-radius:6px; padding:7px 12px;" } else { "" },
                    onclick: move |_| {
                        let mut c = gpu_cfg.read().clone();
                        c.backend = "cuda".into();
                        c.preference = "discrete-only".into();
                        c.reject_igpu = true;
                        if let Ok(_) = crate::session_admission::save_and_apply_gpu_selection_config(c.clone()) {
                            gpu_cfg.set(c);
                            status.set(serde_json::to_string_pretty(&snapshot()).unwrap_or_default());
                            message.set("Selected GPU backend: CUDA (NVIDIA Native Compute + Vulkan Companion)".into());
                        }
                    },
                    "Backend: CUDA (NVIDIA)"
                }
                button {
                    style: if current_cfg.backend == "dx12" { "background:#134e4a; color:#99f6e4; border:1px solid #2dd4bf; border-radius:6px; padding:7px 12px;" } else { "" },
                    onclick: move |_| {
                        let mut c = gpu_cfg.read().clone();
                        c.backend = "dx12".into();
                        if let Ok(_) = crate::session_admission::save_and_apply_gpu_selection_config(c.clone()) {
                            gpu_cfg.set(c);
                            status.set(serde_json::to_string_pretty(&snapshot()).unwrap_or_default());
                            message.set("Selected GPU backend: DirectX 12".into());
                        }
                    },
                    "Backend: DirectX 12"
                }
                button {
                    style: if current_cfg.backend == "metal" { "background:#134e4a; color:#99f6e4; border:1px solid #2dd4bf; border-radius:6px; padding:7px 12px;" } else { "" },
                    onclick: move |_| {
                        let mut c = gpu_cfg.read().clone();
                        c.backend = "metal".into();
                        if let Ok(_) = crate::session_admission::save_and_apply_gpu_selection_config(c.clone()) {
                            gpu_cfg.set(c);
                            status.set(serde_json::to_string_pretty(&snapshot()).unwrap_or_default());
                            message.set("Selected GPU backend: Metal".into());
                        }
                    },
                    "Backend: Metal"
                }
                button {
                    style: if current_cfg.preference == "discrete-only" { "background:#134e4a; color:#99f6e4; border:1px solid #2dd4bf; border-radius:6px; padding:7px 12px;" } else { "" },
                    onclick: move |_| {
                        let mut c = gpu_cfg.read().clone();
                        c.preference = "discrete-only".into();
                        c.reject_igpu = true;
                        if let Ok(_) = crate::session_admission::save_and_apply_gpu_selection_config(c.clone()) {
                            gpu_cfg.set(c);
                            status.set(serde_json::to_string_pretty(&snapshot()).unwrap_or_default());
                            message.set("Selected Adapter Policy: Discrete GPU Only (On-board iGPU blocked)".into());
                        }
                    },
                    "Discrete GPU Only (Block Intel iGPU)"
                }
                button {
                    style: if current_cfg.preference == "integrated" { "background:#134e4a; color:#99f6e4; border:1px solid #2dd4bf; border-radius:6px; padding:7px 12px;" } else { "" },
                    onclick: move |_| {
                        let mut c = gpu_cfg.read().clone();
                        c.preference = "integrated".into();
                        c.reject_igpu = false;
                        if let Ok(_) = crate::session_admission::save_and_apply_gpu_selection_config(c.clone()) {
                            gpu_cfg.set(c);
                            status.set(serde_json::to_string_pretty(&snapshot()).unwrap_or_default());
                            message.set("Selected Adapter Policy: Integrated / Low-Power GPU".into());
                        }
                    },
                    "Allow Integrated GPU"
                }
            }
            h2 { "Biosphere rule" }
            p { "Non-fiction living taxa require sourced .q42 evidence (synth_compile_q42 + query_sparql) before modelling. Encyclopedia is Ecopunk." }
            h2 { "Status" }
            pre { style: "background:#12211e; padding:12px; border-radius:8px; overflow:auto; max-height:55vh; white-space:pre-wrap;", "{status}" }
        }
    }
}
