//! Optional native supervision of the same agent host. This is Synth, not Poet's UI.
use crate::{NativeContext, SynthHost, TransportCommand};
use dioxus::prelude::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

#[derive(Clone, Default)]
struct View {
    context: Option<NativeContext>,
    clips: Vec<String>,
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
    let attributes = dioxus_native::WindowAttributes::default()
        .with_title("Synth · Asset production")
        .with_surface_size(dioxus_native::LogicalSize::new(1180, 760));
    dioxus_native::launch_cfg_with_props(
        Monitor,
        MonitorProps { host, cancellation },
        Vec::new(),
        vec![Box::new(attributes)],
    );
    Ok(())
}
fn perform(
    host: &Arc<Mutex<SynthHost>>,
    context: &Option<NativeContext>,
    command: TransportCommand,
) -> String {
    let Some(expected) = context else {
        return "No current admitted Synth context".into();
    };
    match host.try_lock() {
        Ok(mut h) => match h.control(expected, command) {
            Ok(()) => "Control accepted; old sound epoch retired".into(),
            Err(_) => "Control refused; refresh the admitted source/session".into(),
        },
        Err(_) => "Agent operation is running; control was not queued".into(),
    }
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
                                "Session {} ({:?}) · source {} · projection {} · clip {} · {} · {}",
                                s.actor_subject,
                                s.actor_kind,
                                s.context.document.revision,
                                s.context.source_generation,
                                s.selected_clip.as_deref().unwrap_or("none"),
                                if s.playing { "playing" } else { "paused" },
                                s.audio_status
                            ),
                            context: Some(s.context),
                            clips: s.clips,
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
    let play_host = props.host.clone();
    let pause_host = props.host.clone();
    let stop_host = props.host.clone();
    let play_context = displayed.context.clone();
    let pause_context = displayed.context.clone();
    let stop_context = displayed.context.clone();
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
                button {disabled:displayed.context.is_none(),onclick:move|_|message.set(perform(&play_host,&play_context,TransportCommand::Play)),"Enable motion clock"}
                button {disabled:displayed.context.is_none(),onclick:move|_|message.set(perform(&pause_host,&pause_context,TransportCommand::Pause)),"Pause motion clock"}
                button {disabled:displayed.context.is_none() || !cfg!(feature="audio-device"),onclick:move|_| {
                    #[cfg(feature="audio-device")]
                    if let Some(expected)=&stop_context {
                        let result=stop_host.try_lock().map_err(|_|()).and_then(|h|h.stop_audio(expected).map_err(|_|()));
                        message.set(if result.is_ok(){"Device stop/reset acknowledged"}else{"Device stop not acknowledged; inspect native output"}.into());
                    }
                },"Stop sound"}
            }
            h2 {"Authored behaviours"}
            p {"Only clips actually present in the admitted source are offered."}
            for clip in displayed.clips {
                {let host=props.host.clone();let context=displayed.context.clone();let name=clip.clone();
                    rsx! {button {style:"margin:4px;padding:10px;",onclick:move|_|message.set(perform(&host,&context,TransportCommand::SelectClip(name.clone()))),"{clip}"}}}
            }
            p {role:"status","{message}"}
        }
    }
}
