//! Clean-room presentation deck storyboard domain engine with transitions and timing budget.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum DeckError {
    DuplicateSlideId(SlideId),
    SlideNotFound(SlideId),
    IndexOutOfBounds { index: usize, count: usize },
    NegativeDuration(f64),
    NothingToUndo,
    NothingToRedo,
}

impl fmt::Display for DeckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSlideId(id) => write!(f, "Duplicate slide ID: {:?}", id),
            Self::SlideNotFound(id) => write!(f, "Slide ID not found: {:?}", id),
            Self::IndexOutOfBounds { index, count } => {
                write!(f, "Slide index {index} out of bounds (slide count: {count})")
            }
            Self::NegativeDuration(d) => {
                write!(f, "Invalid slide duration: {d} seconds (must be non-negative finite)")
            }
            Self::NothingToUndo => write!(f, "No actions to undo in deck storyboard"),
            Self::NothingToRedo => write!(f, "No actions to redo in deck storyboard"),
        }
    }
}

impl std::error::Error for DeckError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SlideId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlideTransitionKind {
    Cut,
    CrossDissolve,
    PushHorizontal,
    MorphCamera10d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlideSpec {
    pub id: SlideId,
    pub title: String,
    pub duration_seconds: f64,
    pub transition: SlideTransitionKind,
    pub speaker_notes: String,
}

impl SlideSpec {
    pub fn new(
        id: SlideId,
        title: impl Into<String>,
        duration_seconds: f64,
        transition: SlideTransitionKind,
        speaker_notes: impl Into<String>,
    ) -> Result<Self, DeckError> {
        if !duration_seconds.is_finite() || duration_seconds < 0.0 {
            return Err(DeckError::NegativeDuration(duration_seconds));
        }
        Ok(Self {
            id,
            title: title.into(),
            duration_seconds,
            transition,
            speaker_notes: speaker_notes.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
enum StoryboardOp {
    AddSlide { index: usize, spec: SlideSpec },
    RemoveSlide { index: usize, spec: SlideSpec },
    MoveSlide { from: usize, to: usize },
    UpdateNotes { id: SlideId, old_notes: String, new_notes: String },
    UpdateDuration { id: SlideId, old_dur: f64, new_dur: f64 },
}

#[derive(Debug, Clone, Default)]
pub struct DeckStoryboard {
    slides: Vec<SlideSpec>,
    undo_stack: Vec<StoryboardOp>,
    redo_stack: Vec<StoryboardOp>,
}

impl DeckStoryboard {
    pub fn new() -> Self {
        Self {
            slides: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn slide_count(&self) -> usize { self.slides.len() }
    pub fn slides(&self) -> &[SlideSpec] { &self.slides }
    pub fn find_slide(&self, id: SlideId) -> Option<&SlideSpec> { self.slides.iter().find(|s| s.id == id) }
    pub fn total_duration_seconds(&self) -> f64 { self.slides.iter().map(|s| s.duration_seconds).sum() }

    pub fn add_slide(&mut self, spec: SlideSpec) -> Result<(), DeckError> {
        if !spec.duration_seconds.is_finite() || spec.duration_seconds < 0.0 {
            return Err(DeckError::NegativeDuration(spec.duration_seconds));
        }
        if self.slides.iter().any(|s| s.id == spec.id) {
            return Err(DeckError::DuplicateSlideId(spec.id));
        }
        let index = self.slides.len();
        self.slides.push(spec.clone());
        self.undo_stack.push(StoryboardOp::AddSlide { index, spec });
        self.redo_stack.clear();
        Ok(())
    }

    pub fn insert_slide(&mut self, index: usize, spec: SlideSpec) -> Result<(), DeckError> {
        if index > self.slides.len() {
            return Err(DeckError::IndexOutOfBounds {
                index,
                count: self.slides.len(),
            });
        }
        if !spec.duration_seconds.is_finite() || spec.duration_seconds < 0.0 {
            return Err(DeckError::NegativeDuration(spec.duration_seconds));
        }
        if self.slides.iter().any(|s| s.id == spec.id) {
            return Err(DeckError::DuplicateSlideId(spec.id));
        }
        self.slides.insert(index, spec.clone());
        self.undo_stack.push(StoryboardOp::AddSlide { index, spec });
        self.redo_stack.clear();
        Ok(())
    }

    pub fn remove_slide(&mut self, id: SlideId) -> Result<SlideSpec, DeckError> {
        let index = self
            .slides
            .iter()
            .position(|s| s.id == id)
            .ok_or(DeckError::SlideNotFound(id))?;

        let spec = self.slides.remove(index);
        self.undo_stack.push(StoryboardOp::RemoveSlide {
            index,
            spec: spec.clone(),
        });
        self.redo_stack.clear();
        Ok(spec)
    }

    pub fn move_slide(&mut self, from_index: usize, to_index: usize) -> Result<(), DeckError> {
        let count = self.slides.len();
        if from_index >= count {
            return Err(DeckError::IndexOutOfBounds {
                index: from_index,
                count,
            });
        }
        if to_index >= count {
            return Err(DeckError::IndexOutOfBounds {
                index: to_index,
                count,
            });
        }
        if from_index == to_index {
            return Ok(());
        }
        let item = self.slides.remove(from_index);
        self.slides.insert(to_index, item);
        self.undo_stack.push(StoryboardOp::MoveSlide {
            from: from_index,
            to: to_index,
        });
        self.redo_stack.clear();
        Ok(())
    }

    pub fn update_speaker_notes(
        &mut self,
        id: SlideId,
        notes: impl Into<String>,
    ) -> Result<(), DeckError> {
        let slide = self
            .slides
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or(DeckError::SlideNotFound(id))?;

        let old_notes = slide.speaker_notes.clone();
        let new_notes = notes.into();
        slide.speaker_notes = new_notes.clone();
        self.undo_stack.push(StoryboardOp::UpdateNotes {
            id,
            old_notes,
            new_notes,
        });
        self.redo_stack.clear();
        Ok(())
    }

    pub fn update_duration(
        &mut self,
        id: SlideId,
        duration_seconds: f64,
    ) -> Result<(), DeckError> {
        if !duration_seconds.is_finite() || duration_seconds < 0.0 {
            return Err(DeckError::NegativeDuration(duration_seconds));
        }
        let slide = self
            .slides
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or(DeckError::SlideNotFound(id))?;

        let old_dur = slide.duration_seconds;
        slide.duration_seconds = duration_seconds;
        self.undo_stack.push(StoryboardOp::UpdateDuration {
            id,
            old_dur,
            new_dur: duration_seconds,
        });
        self.redo_stack.clear();
        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), DeckError> {
        let op = self.undo_stack.pop().ok_or(DeckError::NothingToUndo)?;
        match op {
            StoryboardOp::AddSlide { index, spec } => {
                self.slides.remove(index);
                self.redo_stack.push(StoryboardOp::AddSlide { index, spec });
            }
            StoryboardOp::RemoveSlide { index, spec } => {
                self.slides.insert(index, spec.clone());
                self.redo_stack.push(StoryboardOp::RemoveSlide { index, spec });
            }
            StoryboardOp::MoveSlide { from, to } => {
                let item = self.slides.remove(to);
                self.slides.insert(from, item);
                self.redo_stack.push(StoryboardOp::MoveSlide { from, to });
            }
            StoryboardOp::UpdateNotes { id, old_notes, new_notes } => {
                if let Some(s) = self.slides.iter_mut().find(|s| s.id == id) {
                    s.speaker_notes = old_notes.clone();
                }
                self.redo_stack.push(StoryboardOp::UpdateNotes {
                    id,
                    old_notes,
                    new_notes,
                });
            }
            StoryboardOp::UpdateDuration { id, old_dur, new_dur } => {
                if let Some(s) = self.slides.iter_mut().find(|s| s.id == id) {
                    s.duration_seconds = old_dur;
                }
                self.redo_stack.push(StoryboardOp::UpdateDuration {
                    id,
                    old_dur,
                    new_dur,
                });
            }
        }
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), DeckError> {
        let op = self.redo_stack.pop().ok_or(DeckError::NothingToRedo)?;
        match op {
            StoryboardOp::AddSlide { index, spec } => {
                self.slides.insert(index, spec.clone());
                self.undo_stack.push(StoryboardOp::AddSlide { index, spec });
            }
            StoryboardOp::RemoveSlide { index, spec } => {
                self.slides.remove(index);
                self.undo_stack.push(StoryboardOp::RemoveSlide { index, spec });
            }
            StoryboardOp::MoveSlide { from, to } => {
                let item = self.slides.remove(from);
                self.slides.insert(to, item);
                self.undo_stack.push(StoryboardOp::MoveSlide { from, to });
            }
            StoryboardOp::UpdateNotes { id, old_notes, new_notes } => {
                if let Some(s) = self.slides.iter_mut().find(|s| s.id == id) {
                    s.speaker_notes = new_notes.clone();
                }
                self.undo_stack.push(StoryboardOp::UpdateNotes {
                    id,
                    old_notes,
                    new_notes,
                });
            }
            StoryboardOp::UpdateDuration { id, old_dur, new_dur } => {
                if let Some(s) = self.slides.iter_mut().find(|s| s.id == id) {
                    s.duration_seconds = new_dur;
                }
                self.undo_stack.push(StoryboardOp::UpdateDuration {
                    id,
                    old_dur,
                    new_dur,
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_storyboard_basic_flow() {
        let mut deck = DeckStoryboard::new();
        let s1 = SlideSpec::new(
            SlideId(1),
            "Intro",
            15.0,
            SlideTransitionKind::Cut,
            "Welcome team",
        )
        .unwrap();
        let s2 = SlideSpec::new(
            SlideId(2),
            "Methodology",
            45.0,
            SlideTransitionKind::CrossDissolve,
            "Explain 10d manifold",
        )
        .unwrap();

        deck.add_slide(s1).unwrap();
        deck.add_slide(s2).unwrap();

        assert_eq!(deck.slide_count(), 2);
        assert_eq!(deck.total_duration_seconds(), 60.0);

        // Move slide 0 to 1
        deck.move_slide(0, 1).unwrap();
        assert_eq!(deck.slides()[0].id, SlideId(2));
        assert_eq!(deck.slides()[1].id, SlideId(1));

        // Undo move
        deck.undo().unwrap();
        assert_eq!(deck.slides()[0].id, SlideId(1));
    }

    #[test]
    fn test_negative_duplicate_slide_id() {
        let mut deck = DeckStoryboard::new();
        let s1 = SlideSpec::new(SlideId(1), "S1", 10.0, SlideTransitionKind::Cut, "").unwrap();
        let s1_dup = SlideSpec::new(SlideId(1), "S1 Again", 20.0, SlideTransitionKind::PushHorizontal, "").unwrap();

        deck.add_slide(s1).unwrap();
        assert_eq!(deck.add_slide(s1_dup), Err(DeckError::DuplicateSlideId(SlideId(1))));
    }

    #[test]
    fn test_negative_duration_rejected() {
        let res = SlideSpec::new(SlideId(99), "Bad", -10.0, SlideTransitionKind::Cut, "");
        assert_eq!(res, Err(DeckError::NegativeDuration(-10.0)));

        let res_nan = SlideSpec::new(SlideId(99), "Bad", f64::NAN, SlideTransitionKind::Cut, "");
        assert!(matches!(res_nan, Err(DeckError::NegativeDuration(_))));
    }

    #[test]
    fn test_negative_move_out_of_bounds() {
        let mut deck = DeckStoryboard::new();
        let s1 = SlideSpec::new(SlideId(1), "S1", 10.0, SlideTransitionKind::Cut, "").unwrap();
        deck.add_slide(s1).unwrap();

        assert_eq!(
            deck.move_slide(0, 5),
            Err(DeckError::IndexOutOfBounds { index: 5, count: 1 })
        );
    }
}
