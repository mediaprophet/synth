//! Optional native supervision of the same agent host. This is Synth, not Poet's UI.
use crate::SynthHost;
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
    host: Arc<Mutex<SynthHost>>,
    cancellation: Option<Arc<AtomicBool>>,
}
impl PartialEq for MonitorProps {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.host, &other.host)
            && match (&self.cancellation, &other.cancellation) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}

pub fn launch(host: Arc<Mutex<SynthHost>>) -> Result<(), String> {
    launch_with_cancellation(host, None)
}
pub fn launch_with_cancellation(
    host: Arc<Mutex<SynthHost>>,
    cancellation: Option<Arc<AtomicBool>>,
) -> Result<(), String> {
    // An authenticated host is injected by the native embedding. No source,
    // JSON project file or window title creates an actor/session grant.
    host.try_lock()
        .map_err(|_| "Synth host is busy")?
        .inspect()?;
    let attributes = aether_native::WindowAttributes::default()
        .with_title("Synth · Asset production")
        .with_surface_size(aether_native::LogicalSize::new(1180, 760));
    aether_native::launch_cfg_with_props(
        Monitor,
        MonitorProps { host, cancellation },
        Vec::new(),
        vec![Box::new(attributes)],
    );
    Ok(())
}
#[allow(non_snake_case)]
fn Monitor(props: MonitorProps) -> Element {
    let mut view = use_signal(View::default);
    let mut message = use_signal(String::new);
    let polling = props.host.clone();
    use_future(move || {
        let host = polling.clone();
        async move {
            loop {
                let next = match host.try_lock() {
                    Ok(h) => match h.inspect() {
                        Ok(s) => View {
                            status: format!(
                                "Session {} ({:?}) · revision {} · producer {}@{}",
                                s.actor_subject,
                                s.actor_kind,
                                s.context.document.revision,
                                s.producer,
                                s.producer_version
                            ),
                            resources: s
                                .resources
                                .iter()
                                .map(|(key, kind, bytes)| format!("{key} · {kind} · {bytes} bytes"))
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
                futures_timer::Delay::new(std::time::Duration::from_secs(1)).await;
            }
        }
    });
    let displayed = view.read().clone();
    let cancel_work = props.cancellation.clone();
    rsx! {
        main {style:"font-family:system-ui;background:#0e1715;color:#e5f1ea;min-height:100vh;padding:28px;",
            h1 {"Synth"}
            p {"Agent-driven asset production · QualiaDB"}
            p {"{displayed.status}"}
            div {style:"display:flex;gap:12px;flex-wrap:wrap;",
                button {disabled:props.cancellation.is_none(),onclick:move|_| {
                    if let Some(cancel)=&cancel_work {cancel.store(true,Ordering::Release);message.set("Cancellation requested for the current preparation".into());}
                },"Cancel preparation"}
            }
            h2 {"Produced resources"}
            for resource in displayed.resources {
                p {"{resource}"}
            }
            p {role:"status","{message}"}
        }
    }
}
