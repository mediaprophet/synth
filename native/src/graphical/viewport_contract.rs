//! Derived viewport output. No source, principal or review authority is created.
use super::{controller::SessionEpoch, preview::{PreviewCamera, PreviewFrame, SourceRef}};

#[derive(Clone)]
pub struct ViewportState {
    pub source: SourceRef,
    pub execution_epoch: Option<SessionEpoch>,
    pub frame_id: u64,
    pub selected_part: Option<String>,
    pub camera: PreviewCamera,
}
pub struct ViewportOutput {
    pub frame: PreviewFrame,
    pub state: ViewportState,
}

pub fn validate_command(command: &super::viewport::ViewportCommand) -> Result<(), String> {
    use super::viewport::ViewportCommand::*;
    let valid = match command {
        Orbit {dx,dy} | Pan {dx,dy} => [dx,dy].iter().all(|n| n.is_finite() && n.abs()<=8192.0),
        Zoom {steps} => steps.is_finite() && steps.abs()<=64.0,
        Pick {frame_id,x,y} => *frame_id>0 && [x,y].iter().all(|n| n.is_finite() && **n>=0.0 && **n<=1024.0),
        FramePart {key} => !key.is_empty() && key.len()<=4096 && !key.as_bytes().contains(&0),
        _ => true,
    };
    if valid {Ok(())} else {Err("Viewport command exceeds the native input bounds".into())}
}
