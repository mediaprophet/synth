//! Clean-room rich-text inline span annotation store with undo/redo and span validation.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationError {
    InvertedSpan { start: usize, end: usize },
    OutOfBounds { span_end: usize, document_len: usize },
    AnnotationNotFound(u64),
    NothingToUndo,
    NothingToRedo,
}

impl fmt::Display for AnnotationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvertedSpan { start, end } => {
                write!(f, "Inverted text span: start ({start}) > end ({end})")
            }
            Self::OutOfBounds { span_end, document_len } => {
                write!(
                    f,
                    "Span end ({span_end}) exceeds document length ({document_len})"
                )
            }
            Self::AnnotationNotFound(id) => write!(f, "Annotation id {id} not found"),
            Self::NothingToUndo => write!(f, "No actions to undo in annotation store"),
            Self::NothingToRedo => write!(f, "No actions to redo in annotation store"),
        }
    }
}

impl std::error::Error for AnnotationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TextSpan {
    pub start_utf8: usize,
    pub end_utf8: usize,
}

impl TextSpan {
    pub const fn new(start_utf8: usize, end_utf8: usize) -> Self {
        Self { start_utf8, end_utf8 }
    }

    pub fn len(&self) -> usize {
        self.end_utf8.saturating_sub(self.start_utf8)
    }

    pub fn is_empty(&self) -> bool {
        self.start_utf8 == self.end_utf8
    }

    pub fn validate(&self, doc_len: usize) -> Result<(), AnnotationError> {
        if self.start_utf8 > self.end_utf8 {
            return Err(AnnotationError::InvertedSpan {
                start: self.start_utf8,
                end: self.end_utf8,
            });
        }
        if self.end_utf8 > doc_len {
            return Err(AnnotationError::OutOfBounds {
                span_end: self.end_utf8,
                document_len: doc_len,
            });
        }
        Ok(())
    }

    pub fn intersects(&self, other: &TextSpan) -> bool {
        self.start_utf8 < other.end_utf8 && other.start_utf8 < self.end_utf8
    }

    pub fn contains_offset(&self, offset: usize) -> bool {
        offset >= self.start_utf8 && offset < self.end_utf8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnnotationKind {
    Comment,
    CitationAnchor,
    GlossaryTerm,
    TrackedInsertion,
    TrackedDeletion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentAnnotation {
    pub id: u64,
    pub span: TextSpan,
    pub kind: AnnotationKind,
    pub author_did_hash: u64,
    pub content: String,
    pub created_at_epoch_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AnnotationOp {
    Add(DocumentAnnotation),
    Remove(DocumentAnnotation),
    UpdateSpan { id: u64, old_span: TextSpan, new_span: TextSpan },
}

#[derive(Debug, Clone, Default)]
pub struct DocumentAnnotationStore {
    annotations: Vec<DocumentAnnotation>,
    undo_stack: Vec<AnnotationOp>,
    redo_stack: Vec<AnnotationOp>,
    next_id: u64,
}

impl DocumentAnnotationStore {
    pub fn new() -> Self {
        Self {
            annotations: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            next_id: 1,
        }
    }

    pub fn count(&self) -> usize {
        self.annotations.len()
    }

    pub fn annotations(&self) -> &[DocumentAnnotation] {
        &self.annotations
    }

    pub fn find(&self, id: u64) -> Option<&DocumentAnnotation> {
        self.annotations.iter().find(|a| a.id == id)
    }

    pub fn add_annotation(
        &mut self,
        span: TextSpan,
        kind: AnnotationKind,
        author_did_hash: u64,
        content: impl Into<String>,
        created_at_epoch_ms: u64,
        doc_len: usize,
    ) -> Result<u64, AnnotationError> {
        span.validate(doc_len)?;
        let id = self.next_id;
        self.next_id += 1;

        let annotation = DocumentAnnotation {
            id,
            span,
            kind,
            author_did_hash,
            content: content.into(),
            created_at_epoch_ms,
        };

        self.annotations.push(annotation.clone());
        self.undo_stack.push(AnnotationOp::Add(annotation));
        self.redo_stack.clear();
        Ok(id)
    }

    pub fn remove_annotation(&mut self, id: u64) -> Result<(), AnnotationError> {
        let index = self
            .annotations
            .iter()
            .position(|a| a.id == id)
            .ok_or(AnnotationError::AnnotationNotFound(id))?;

        let removed = self.annotations.remove(index);
        self.undo_stack.push(AnnotationOp::Remove(removed));
        self.redo_stack.clear();
        Ok(())
    }

    pub fn update_span(
        &mut self,
        id: u64,
        new_span: TextSpan,
        doc_len: usize,
    ) -> Result<(), AnnotationError> {
        new_span.validate(doc_len)?;
        let annotation = self
            .annotations
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or(AnnotationError::AnnotationNotFound(id))?;

        let old_span = annotation.span;
        annotation.span = new_span;
        self.undo_stack.push(AnnotationOp::UpdateSpan {
            id,
            old_span,
            new_span,
        });
        self.redo_stack.clear();
        Ok(())
    }

    pub fn query_intersecting(&self, span: TextSpan) -> Vec<&DocumentAnnotation> {
        self.annotations
            .iter()
            .filter(|a| a.span.intersects(&span))
            .collect()
    }

    pub fn undo(&mut self) -> Result<(), AnnotationError> {
        let op = self.undo_stack.pop().ok_or(AnnotationError::NothingToUndo)?;
        match op {
            AnnotationOp::Add(annot) => {
                let id = annot.id;
                self.annotations.retain(|a| a.id != id);
                self.redo_stack.push(AnnotationOp::Add(annot));
            }
            AnnotationOp::Remove(annot) => {
                self.annotations.push(annot.clone());
                self.redo_stack.push(AnnotationOp::Remove(annot));
            }
            AnnotationOp::UpdateSpan { id, old_span, new_span } => {
                if let Some(a) = self.annotations.iter_mut().find(|a| a.id == id) {
                    a.span = old_span;
                }
                self.redo_stack.push(AnnotationOp::UpdateSpan {
                    id,
                    old_span,
                    new_span,
                });
            }
        }
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), AnnotationError> {
        let op = self.redo_stack.pop().ok_or(AnnotationError::NothingToRedo)?;
        match op {
            AnnotationOp::Add(annot) => {
                self.annotations.push(annot.clone());
                self.undo_stack.push(AnnotationOp::Add(annot));
            }
            AnnotationOp::Remove(annot) => {
                let id = annot.id;
                self.annotations.retain(|a| a.id != id);
                self.undo_stack.push(AnnotationOp::Remove(annot));
            }
            AnnotationOp::UpdateSpan { id, old_span, new_span } => {
                if let Some(a) = self.annotations.iter_mut().find(|a| a.id == id) {
                    a.span = new_span;
                }
                self.undo_stack.push(AnnotationOp::UpdateSpan {
                    id,
                    old_span,
                    new_span,
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUTHOR_DID: u64 = 0x1234_5678;

    #[test]
    fn test_add_and_query_annotations() {
        let mut store = DocumentAnnotationStore::new();
        let doc_len = 100;

        let id1 = store
            .add_annotation(
                TextSpan::new(10, 25),
                AnnotationKind::Comment,
                AUTHOR_DID,
                "Clarify terminology",
                1_700_000_000,
                doc_len,
            )
            .expect("add annotation 1");

        let _id2 = store
            .add_annotation(
                TextSpan::new(30, 45),
                AnnotationKind::CitationAnchor,
                AUTHOR_DID,
                "Reference [Smith 2024]",
                1_700_000_000,
                doc_len,
            )
            .expect("add annotation 2");

        assert_eq!(store.count(), 2);
        assert_eq!(store.find(id1).unwrap().content, "Clarify terminology");

        let hits = store.query_intersecting(TextSpan::new(20, 35));
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn test_inverted_span_rejected() {
        let mut store = DocumentAnnotationStore::new();
        let res = store.add_annotation(
            TextSpan::new(50, 20), // Inverted
            AnnotationKind::GlossaryTerm,
            AUTHOR_DID,
            "Bad span",
            1_700_000_000,
            100,
        );
        assert_eq!(
            res,
            Err(AnnotationError::InvertedSpan { start: 50, end: 20 })
        );
    }

    #[test]
    fn test_out_of_bounds_span_rejected() {
        let mut store = DocumentAnnotationStore::new();
        let res = store.add_annotation(
            TextSpan::new(80, 120), // Exceeds doc length 100
            AnnotationKind::TrackedInsertion,
            AUTHOR_DID,
            "Overflow text",
            1_700_000_000,
            100,
        );
        assert_eq!(
            res,
            Err(AnnotationError::OutOfBounds {
                span_end: 120,
                document_len: 100,
            })
        );
    }

    #[test]
    fn test_undo_redo_stack() {
        let mut store = DocumentAnnotationStore::new();
        let doc_len = 200;

        let id = store
            .add_annotation(
                TextSpan::new(5, 15),
                AnnotationKind::TrackedDeletion,
                AUTHOR_DID,
                "deleted words",
                1_700_000_000,
                doc_len,
            )
            .unwrap();

        assert_eq!(store.count(), 1);

        // Undo addition
        store.undo().expect("undo");
        assert_eq!(store.count(), 0);

        // Redo addition
        store.redo().expect("redo");
        assert_eq!(store.count(), 1);
        assert_eq!(store.find(id).unwrap().span, TextSpan::new(5, 15));
    }

    #[test]
    fn test_empty_undo_redo_errors() {
        let mut store = DocumentAnnotationStore::new();
        assert_eq!(store.undo(), Err(AnnotationError::NothingToUndo));
        assert_eq!(store.redo(), Err(AnnotationError::NothingToRedo));
    }
}
