//! Native selected-surface tools share the existing source, actor and job owner.
use super::{AuthoringEdit, AuthoringView};
use crate::graphical::{
    controller::EditorSnapshot,
    jobs::{self, Action, Context, HostHandle},
    surfaces::inspection::SurfaceInspection,
};
use dioxus::prelude::*;
#[derive(Default)]
pub(crate) struct DraftCursor {
    last: Option<(Context, String)>,
    generation: u64,
}
impl DraftCursor {
    pub(crate) fn token(&mut self, context: Context, part: &str) -> Option<String> {
        let next = (context, part.to_owned());
        if self.last.as_ref() != Some(&next) {
            self.generation = self.generation.checked_add(1)?;
            self.last = Some(next);
        }
        Some(self.generation.to_string())
    }
}
#[component]
pub(super) fn SurfaceTools(
    host: HostHandle,
    view: Signal<EditorSnapshot>,
    selected: Signal<String>,
    inspected: Signal<Option<AuthoringView>>,
    fresh: bool,
    mut busy: Signal<bool>,
    mut status: Signal<String>,
    onedit: EventHandler<AuthoringEdit>,
) -> Element {
    let mut inspection = use_signal(|| None::<(Context, SurfaceInspection)>);
    let current = view();
    let captured = current.source.clone();
    let id = selected();
    let part = inspected().and_then(|v| v.parts.into_iter().find(|p| p.id == id));
    let shown = inspection()
        .filter(|(context, data)| {
            *context == Context::from_snapshot(&current)
                && data.part_id == id
                && data.source == current.source
                && current.publication_is_current()
        })
        .map(|(_, data)| data);
    let cursor = use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(DraftCursor::default())));
    let token = cursor
        .borrow_mut()
        .token(Context::from_snapshot(&current), &id);
    let context_valid = token.is_some();
    let token = token.unwrap_or_default();
    let atlas = part.as_ref().and_then(|p| p.uv_atlas.clone());
    let detail = part.as_ref().and_then(|p| p.detail.clone());
    let material_id = part.as_ref().map(|p| p.material_id).unwrap_or(0);
    let fresh = fresh && context_valid && part.is_some() && !busy();
    let inspect_host = host.clone();
    let inspect_id = id.clone();
    let atlas_id = id.clone();
    let oninspect = EventHandler::new(move |_: ()| {
        if busy() {
            return;
        }
        busy.set(true);
        let origin = Context::from_snapshot(&view());
        let expected = captured.clone();
        let host = inspect_host.clone();
        let guard = host.clone();
        let id = inspect_id.clone();
        spawn(async move {
            match crate::graphical::native_dispatch::run(
                host,
                Action::InspectSurface { id, expected },
                origin,
            )
            .await
            {
                Ok(reply)
                    if reply.operation_succeeded
                        && jobs::completion_is_current(&guard, &view(), &reply) =>
                {
                    if let Some(data) = reply.surface {
                        if data.part_id == selected() {
                            inspection.set(Some((Context::from_snapshot(&view()), data)));
                        }
                    }
                    status.set(reply.message);
                }
                Ok(reply) => status.set(format!(
                    "Surface inspection withheld or refused: {}",
                    reply.message
                )),
                Err(e) => status.set(e),
            }
            busy.set(false);
        });
    });
    rsx! {section{style:"margin-top:10px;padding:6px;border:1px solid #405264;",
        h4{"UV and texture/detail workspace"}
        button{disabled:!fresh,onclick:move |_|oninspect.call(()),"Inspect current selected UV and texture pixels"}
        if let Some(data)=shown.clone(){p{"{data.summary}"}img{src:"{data.uv_image}",alt:"Current part UV triangle layout, clipped to unit tile",style:"width:256px;height:256px;image-rendering:pixelated;"}}
        super::uv_atlas_controls::UvAtlasControls{atlas,context_token:token.clone(),fresh,inspection:shown.clone(),onapply:move|atlas|onedit.call(AuthoringEdit::UvAtlas{id:atlas_id.clone(),atlas})}
        super::detail_controls::DetailControls{detail,context_token:token,fresh,preview_uri:shown.as_ref().and_then(|s|s.texture_image.clone()),onapply:move|detail|onedit.call(AuthoringEdit::TextureDetail{material_id,detail}),onmorphology:move|(layer_index,operation,radius)|onedit.call(AuthoringEdit::TextureMaskMorphology{material_id,layer_index,operation,radius}),oninspect}
    }}
}
