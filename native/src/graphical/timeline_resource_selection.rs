//! UI-only selection from an inspected HMC inventory. No source is changed here.
use super::{checkpoints::ResourceInspectionRequest,controller::EditorSnapshot,jobs::Context};
use dioxus::prelude::*;

#[derive(Clone,PartialEq,Eq)]
pub(crate) struct TimelineResourceSelection {pub context:Context,pub request:ResourceInspectionRequest}
impl TimelineResourceSelection {
    pub fn checked_request(&self,current:&EditorSnapshot)->Result<ResourceInspectionRequest,String> {
        if self.context!=Context::from_snapshot(current) || !current.publication_is_current() {return Err("Selected package belongs to an older source or authority context; inspect and select it again".into());}
        self.context.source.check_current()?;
        self.request.validate_bounds()?;
        if self.request.package_digest==[0;32] || self.request.resource_digest==[0;32] {return Err("Selected package and resource need exact digests".into());}
        Ok(self.request.clone())
    }
}

#[component]
pub(crate) fn TimelineResourceSelect(request:ResourceInspectionRequest,current:EditorSnapshot,mut selection:Signal<Option<TimelineResourceSelection>>,busy:Signal<bool>)->Element {
    let available=!busy() && current.publication_is_current() && request.validate_bounds().is_ok();
    rsx! { button { disabled:!available, onclick:move |_| {
        if available { selection.set(Some(TimelineResourceSelection {context:Context::from_snapshot(&current),request:request.clone()})); }
    }, "Select for timeline association" } }
}
