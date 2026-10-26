//! Per-controller unsubmitted text. Never part of admission or shared source.
use std::sync::{Arc, Mutex};
#[derive(Clone)]
pub struct DraftBuffer(Arc<Mutex<DraftState>>);
struct DraftState {
    text: String,
    revision: u64,
}

/// A completion may adopt accepted text only if no local edit occurred meanwhile.
/// The revision also detects typing followed by undoing back to the same string.
pub(crate) struct DraftCapture {
    text: String,
    revision: Option<u64>,
    buffer: Option<DraftBuffer>,
}
impl DraftCapture {
    pub fn capture(text: String, buffer: Option<&DraftBuffer>) -> Result<Self, String> {
        let revision = if let Some(buffer) = buffer {
            let state = buffer
                .0
                .lock()
                .map_err(|_| "Local draft buffer unavailable")?;
            if state.text != text || state.revision == u64::MAX {
                return Err("Local draft changed; retry after refreshing the editor".into());
            }
            Some(state.revision)
        } else {
            None
        };
        Ok(Self {
            text,
            revision,
            buffer: buffer.cloned(),
        })
    }
    pub fn accept(
        &self,
        succeeded: bool,
        visible: &str,
        next: &str,
        buffer: Option<&DraftBuffer>,
    ) -> Option<String> {
        if !succeeded || visible != self.text || self.buffer.as_ref() != buffer {
            return None;
        }
        match (self.revision, buffer) {
            (Some(revision), Some(buffer)) => {
                let mut state = buffer.0.lock().ok()?;
                if state.revision != revision || state.text != self.text {
                    return None;
                }
                let next_revision = revision.checked_add(1)?;
                state.text = next.to_owned();
                state.revision = next_revision;
            }
            (None, None) => {}
            _ => return None,
        }
        Some(next.to_owned())
    }
}
impl DraftBuffer {
    pub fn new(text: String) -> Self {
        Self(Arc::new(Mutex::new(DraftState { text, revision: 0 })))
    }
    pub fn read(&self) -> Result<String, String> {
        self.0
            .lock()
            .map(|s| s.text.clone())
            .map_err(|_| "Local draft buffer unavailable".into())
    }
    pub fn set(&self, text: String) {
        if let Ok(mut value) = self.0.lock() {
            // Exhaustion fails closed for automatic completion adoption. Explicit
            // local typing remains retained even at the revision ceiling.
            value.revision = value.revision.saturating_add(1);
            value.text = text;
        }
    }
}
impl PartialEq for DraftBuffer {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

/// Explicit private-draft replacement only. Neither accepted source nor review
/// authority changes. Revalidate the exact displayed source/session at the click.
pub(crate) fn accepted_source_for_draft(
    current: &super::controller::EditorSnapshot,
    confirmed: bool,
) -> Result<String, String> {
    if !confirmed {
        return Err(
            "Confirm replacement of the local draft with the displayed accepted source".into(),
        );
    }
    if !current.publication_is_current() {
        return Err(
            "Accepted source or native session changed; refresh before replacing the draft".into(),
        );
    }
    Ok(current.latest_source.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphical::{
        controller::{CanonicalEditorHost, EditorHost, PreviewProvider},
        geometry_host::GeometryHost,
        preview::{PreviewFrame, SourceRef},
    };
    use qualia_core_db::poet_host::invoke::studio_store::{self, StudioArtifact};
    #[test]
    fn pending_success_adopts_text_only_for_unchanged_private_draft() {
        let buffer = DraftBuffer::new("accepted".into());
        let capture = DraftCapture::capture("accepted".into(), Some(&buffer)).unwrap();
        assert!(capture
            .accept(false, "accepted", "refused", Some(&buffer))
            .is_none());
        assert_eq!(buffer.read().unwrap(), "accepted");
        assert_eq!(
            capture
                .accept(true, "accepted", "next", Some(&buffer))
                .unwrap(),
            "next"
        );
        assert_eq!(buffer.read().unwrap(), "next");
        assert!(capture
            .accept(true, "accepted", "late", Some(&buffer))
            .is_none());
    }
    #[test]
    fn pending_completion_preserves_typing_and_detects_edit_then_undo() {
        let buffer = DraftBuffer::new("accepted".into());
        let capture = DraftCapture::capture("accepted".into(), Some(&buffer)).unwrap();
        buffer.set("new private work".into());
        assert!(capture
            .accept(true, "new private work", "next", Some(&buffer))
            .is_none());
        assert_eq!(buffer.read().unwrap(), "new private work");
        buffer.set("accepted".into());
        assert!(capture
            .accept(true, "accepted", "next", Some(&buffer))
            .is_none());
        assert_eq!(buffer.read().unwrap(), "accepted");
        assert!(DraftCapture::capture("different visible text".into(), Some(&buffer)).is_err());
    }
    #[test]
    fn standalone_completion_and_buffer_context_fail_closed() {
        let capture = DraftCapture::capture("accepted".into(), None).unwrap();
        assert!(capture.accept(false, "accepted", "next", None).is_none());
        assert!(capture.accept(true, "private", "next", None).is_none());
        assert_eq!(
            capture.accept(true, "accepted", "next", None).unwrap(),
            "next"
        );
        let buffer = DraftBuffer::new("accepted".into());
        assert!(capture
            .accept(true, "accepted", "next", Some(&buffer))
            .is_none());
        let capture = DraftCapture::capture("accepted".into(), Some(&buffer)).unwrap();
        let other = DraftBuffer::new("accepted".into());
        assert!(capture
            .accept(true, "accepted", "next", Some(&other))
            .is_none());
        assert_eq!(other.read().unwrap(), "accepted");
    }
    struct NoPixels;
    impl PreviewProvider for NoPixels {
        fn preview(&mut self, _: SourceRef, _: &vibe::Value) -> Result<PreviewFrame, String> {
            Err("headless draft-only fixture".into())
        }
    }
    #[test]
    fn returning_reviewer_explicitly_adopts_current_source_without_changing_shared_state() {
        let id = "poet-private-draft-accepted-replacement";
        studio_store::create_artifact_if_absent(StudioArtifact::new(
            id,
            "Geometry",
            "Draft recovery",
        ))
        .unwrap();
        let mut host = CanonicalEditorHost::open(
            id.into(),
            vibe::session::SessionLimits::default(),
            GeometryHost::default(),
            Box::new(NoPixels),
        )
        .unwrap();
        let initial = host.snapshot().unwrap();
        let accepted = host.submit("1;", initial.source).unwrap().snapshot;
        let private = DraftBuffer::new("previous round or unsubmitted local work".into());
        assert!(accepted_source_for_draft(&accepted, false).is_err());
        assert_eq!(
            private.read().unwrap(),
            "previous round or unsubmitted local work"
        );
        let chosen = accepted_source_for_draft(&accepted, true).unwrap();
        private.set(chosen.clone());
        assert_eq!(chosen, "1;");
        assert_eq!(host.snapshot().unwrap().source, accepted.source);
        studio_store::upsert_artifact(studio_store::get_artifact(id).unwrap());
        assert!(accepted_source_for_draft(&accepted, true).is_err());
        assert_eq!(private.read().unwrap(), chosen);
    }
}
