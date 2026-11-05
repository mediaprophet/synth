//! Bounded editorial timeline represented by an ordinary authored Vibe binding.
//! The host owns authority, revision, history and checkpointing; this module
//! supplies no persistence and never treats source trim as timeline placement.
use super::source;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use timeline_editing::{set_transition_span, ClipSpan, Tick, TransitionContext, TransitionSpan};
use vibe::Value;

const SCHEMA: &str = "poet.timeline/v1";
const MAX_TRACKS: usize = 64;
const MAX_CLIPS: usize = 512;
const MAX_TRANSITIONS: usize = 512;
const MAX_TICK: u64 = i64::MAX as u64;
const MAX_SOURCE_BYTES: usize = 120 * 1024;

/// `source_start` is an offset in the referenced media. `start` is the
/// position in this timeline. All times use timeline-editing's tick scale.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clip { pub id: String, pub source: String, pub source_start: u64, pub start: u64, pub duration: u64 }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Track { pub id: String, pub locked: bool, pub clips: Vec<Clip> }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transition { pub id: String, pub kind: String, pub track: String, pub from: String, pub to: String, pub start: u64, pub duration: u64 }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Timeline { pub version: u32, pub frame_ticks: u64, pub tracks: Vec<Track>, pub transitions: Vec<Transition> }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag="operation", rename_all="snake_case", deny_unknown_fields)]
pub enum TimelineEdit {
    /// Explicitly establish a timeline; never replace an existing binding.
    Adopt { timeline: Timeline },
    AddTrack { id: String },
    AddClip { track: String, clip: Clip },
    AddTransition { transition: Transition },
    SetTrackLocked { id: String, locked: bool },
    RemoveTrack { id: String },
    UpdateClip { track: String, clip: Clip },
    RemoveClip { track: String, id: String },
    RemoveTransition { id: String },
    /// Authored assertion only; live package/resource checks remain required at read time.
    AssociateResource { clip_id: String, resource_key: String, resource_digest: [u8;32] },
    DissociateResource { clip_id: String },
    /// Only changes the referenced transition after validating current clips.
    SetTransition { id: String, start: u64, duration: u64 },
}
impl TimelineEdit {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Adopt { timeline } => timeline.validate(),
            Self::AddTrack { id } | Self::SetTrackLocked { id, .. } | Self::RemoveTrack { id } | Self::RemoveTransition { id } => if ident(id) { Ok(()) } else { Err("Invalid timeline ID".into()) },
            Self::AssociateResource { clip_id, resource_key, resource_digest } => if ident(clip_id) && !resource_key.is_empty() && resource_key.len()<=4096 && !resource_key.chars().any(char::is_control) && *resource_digest!=[0;32] { Ok(()) } else {Err("Invalid clip or resource assertion".into())},
            Self::DissociateResource { clip_id } => if ident(clip_id) {Ok(())} else {Err("Invalid clip ID".into())},
            Self::RemoveClip { track, id } => if ident(track) && ident(id) { Ok(()) } else { Err("Invalid clip or track ID".into()) },
            Self::AddClip { track, clip } | Self::UpdateClip { track, clip } => {
                if !ident(track) || !ident(&clip.id) || !source_identifier(&clip.source) || clip.duration == 0 || clip.start.checked_add(clip.duration).is_none_or(|end| end > MAX_TICK) || clip.source_start.checked_add(clip.duration).is_none_or(|end|end > MAX_TICK) { return Err("Invalid clip or track".into()); }
                Ok(())
            }
            Self::AddTransition { transition } => {
                if !ident(&transition.id) || !ident(&transition.kind) || !ident(&transition.track) || !ident(&transition.from) || !ident(&transition.to) || transition.from == transition.to || transition.start > MAX_TICK || transition.duration == 0 || transition.duration > MAX_TICK { return Err("Invalid transition".into()); }
                Ok(())
            }
            Self::SetTransition { id, start, duration } => {
                if !ident(id) || *start > MAX_TICK || *duration == 0 || *duration > MAX_TICK {
                    return Err("Invalid transition edit".into());
                }
                Ok(())
            }
        }
    }
}

fn ident(s: &str) -> bool {
    !s.is_empty() && s.len() <= 128 && s.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c,b'_'|b'-'|b'.'|b':'|b'/'))
}
// Preserve source identifier spelling, including non-ASCII IRI characters and
// URI delimiters. This does not interpret, normalize, resolve or authorize it.
// Control characters are refused because JSON's control escapes do not all
// share Vibe's \u{...} string-escape syntax.
fn source_identifier(s: &str) -> bool {
    !s.is_empty() && s.len() <= 2048 && !s.chars().any(char::is_control)
}
fn push_bounded(out: &mut String, value: &str) -> Result<(), String> {
    if out.len().checked_add(value.len()).is_none_or(|n| n > MAX_SOURCE_BYTES) { return Err("Timeline source exceeds byte bound".into()); }
    out.push_str(value); Ok(())
}
fn bounded(n: u64) -> Result<i64, String> { i64::try_from(n).map_err(|_| "Timeline tick outside i64 range".into()) }
fn record<'a>(v: &'a Value, fields: &[&str]) -> Result<&'a BTreeMap<String, Value>, String> {
    let r = source::record(v)?;
    if r.len() != fields.len() || fields.iter().any(|k| !r.contains_key(*k)) { return Err("Unknown or missing timeline field".into()); }
    Ok(r)
}
fn uint(v: &Value) -> Result<u64, String> {
    match v { Value::U64(n) => Ok(*n), Value::I64(n) => u64::try_from(*n).map_err(|_| "Negative timeline integer".into()), _ => Err("Timeline requires exact integers".into()) }
}
fn str_field(r: &BTreeMap<String, Value>, name: &str) -> Result<String, String> { Ok(source::text(source::field(r, name)?)?.to_owned()) }
fn num_field(r: &BTreeMap<String, Value>, name: &str) -> Result<u64, String> { uint(source::field(r, name)?) }
fn list<'a>(r: &'a BTreeMap<String, Value>, name: &str) -> Result<&'a [Value], String> {
    match source::field(r,name)? { Value::List(xs) => Ok(xs), _ => Err("Timeline field must be a list".into()) }
}

pub fn read(current: Option<&Value>) -> Result<Option<Timeline>, String> {
    let Some(current) = current else { return Ok(None) };
    let r = record(current, &["schema", "frame_ticks", "tracks", "transitions"])?;
    if source::text(source::field(r,"schema")?)? != SCHEMA { return Err("Unsupported timeline source schema".into()); }
    let track_rows = list(r,"tracks")?; let trans_rows = list(r,"transitions")?;
    if track_rows.len() > MAX_TRACKS || trans_rows.len() > MAX_TRANSITIONS { return Err("Timeline exceeds item bounds".into()); }
    let mut tracks = Vec::with_capacity(track_rows.len()); let mut clips_total = 0usize;
    for row in track_rows {
        let t = record(row,&["id","locked","clips"])?;
        let locked = match source::field(t,"locked")? { Value::Bool(b) => *b, _ => return Err("Timeline locked requires boolean".into()) };
        let items = list(t,"clips")?; clips_total = clips_total.checked_add(items.len()).ok_or("Clip count overflow")?;
        if clips_total > MAX_CLIPS { return Err("Timeline clip limit exceeded".into()); }
        let mut clips = Vec::with_capacity(items.len());
        for item in items { let c = record(item,&["id","source","source_start","start","duration"])?;
            clips.push(Clip {id:str_field(c,"id")?,source:str_field(c,"source")?,source_start:num_field(c,"source_start")?,start:num_field(c,"start")?,duration:num_field(c,"duration")?}); }
        tracks.push(Track { id:str_field(t,"id")?, locked, clips });
    }
    let mut transitions = Vec::with_capacity(trans_rows.len());
    for row in trans_rows { let t=record(row,&["id","kind","track","from","to","start","duration"])?;
        transitions.push(Transition{id:str_field(t,"id")?,kind:str_field(t,"kind")?,track:str_field(t,"track")?,from:str_field(t,"from")?,to:str_field(t,"to")?,start:num_field(t,"start")?,duration:num_field(t,"duration")?}); }
    let timeline = Timeline{version:1,frame_ticks:num_field(r,"frame_ticks")?,tracks,transitions};
    timeline.validate()?; Ok(Some(timeline))
}

impl Timeline {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.frame_ticks == 0 || self.frame_ticks > MAX_TICK || self.tracks.len() > MAX_TRACKS || self.transitions.len() > MAX_TRANSITIONS { return Err("Timeline version, frame or bounds invalid".into()); }
        let mut track_ids = BTreeSet::new(); let mut clip_ids = BTreeSet::new(); let mut count=0usize;
        for t in &self.tracks {
            if !ident(&t.id) || !track_ids.insert(&t.id) { return Err("Invalid or duplicate track ID".into()); }
            count = count.checked_add(t.clips.len()).ok_or("Clip count overflow")?;
            if count > MAX_CLIPS { return Err("Timeline clip limit exceeded".into()); }
            for c in &t.clips {
                if !ident(&c.id) || !source_identifier(&c.source) || !clip_ids.insert(&c.id) || c.duration == 0 || c.start.checked_add(c.duration).is_none_or(|n| n > MAX_TICK) || c.source_start.checked_add(c.duration).is_none_or(|n| n > MAX_TICK) { return Err("Invalid clip identity, source or time".into()); }
            }
        }
        let mut ids = BTreeSet::new();
        for tr in &self.transitions {
            if !ident(&tr.id) || !ident(&tr.kind) || !ids.insert(&tr.id) || tr.from == tr.to { return Err("Invalid transition identity".into()); }
            self.check_transition(tr, false)?;
        }
        Ok(())
    }
    fn check_transition(&self, tr: &Transition, respect_lock: bool) -> Result<(), String> {
        let track=self.tracks.iter().find(|t| t.id == tr.track).ok_or("Missing transition track")?;
        let from=track.clips.iter().find(|c| c.id == tr.from).ok_or("Missing outgoing clip")?;
        let to=track.clips.iter().find(|c| c.id == tr.to).ok_or("Missing incoming clip")?;
        if from.start >= to.start || from.start.checked_add(from.duration).is_none_or(|end| end < to.start) { return Err("Transition clips must meet in timeline order".into()); }
        let span = |c: &Clip| -> Result<ClipSpan,String> { Ok(ClipSpan::new(Tick(bounded(c.start)?), Tick(bounded(c.start.checked_add(c.duration).ok_or("Clip overflow")?)?))) };
        let ctx=TransitionContext{from:Some(span(from)?),to:Some(span(to)?),track_locked:respect_lock && track.locked};
        set_transition_span(ctx, TransitionSpan{start:Tick(bounded(tr.start)?),duration:Tick(bounded(tr.duration)?)}, Tick(bounded(self.frame_ticks)?), Tick(bounded(self.frame_ticks)?)).map_err(|e|format!("Invalid transition: {e:?}"))?;
        Ok(())
    }
}

/// Host must check current SourceRef, binding mutability and edit authority,
/// then submit this generated source through its existing SourceSession.
pub fn edit_source(current: Option<&Value>, edit: &TimelineEdit) -> Result<String, String> {
    edit_source_with_output(current,edit,true)
}
/// A timeline-only document does not need a geometry binding. The owning
/// controller selects this output profile from the actual current session.
pub fn edit_source_with_output(current: Option<&Value>, edit: &TimelineEdit, has_parts: bool) -> Result<String, String> {
    edit.validate()?;
    if matches!(edit,TimelineEdit::AssociateResource{..}|TimelineEdit::DissociateResource{..}) {return Err("Resource associations require their source owner".into());}
    let mut timeline=match (read(current)?, edit) {
        (None, TimelineEdit::Adopt{timeline}) => { timeline.validate()?; timeline.clone() },
        (Some(_), TimelineEdit::Adopt{..}) => return Err("Timeline already exists".into()),
        (None, _) => return Err("Adopt a timeline before editing".into()),
        (Some(t), _) => t,
    };
    if let TimelineEdit::SetTransition{id,start,duration}=edit {
        let index=timeline.transitions.iter().position(|t| &t.id==id).ok_or("Missing transition")?;
        let mut candidate=timeline.transitions[index].clone(); candidate.start=*start; candidate.duration=*duration;
        timeline.check_transition(&candidate,true)?; timeline.transitions[index]=candidate;
    }
    match edit {
        TimelineEdit::AddTrack { id } => {
            if timeline.tracks.len() >= MAX_TRACKS || timeline.tracks.iter().any(|t| &t.id == id) { return Err("Track ID already exists or track limit reached".into()); }
            timeline.tracks.push(Track {id:id.clone(),locked:false,clips:Vec::new()});
        }
        TimelineEdit::AddClip { track, clip } => {
            if timeline.tracks.iter().map(|t| t.clips.len()).sum::<usize>() >= MAX_CLIPS || timeline.tracks.iter().flat_map(|t| &t.clips).any(|c| c.id == clip.id) { return Err("Clip ID already exists or clip limit reached".into()); }
            let target=timeline.tracks.iter_mut().find(|t| &t.id == track).ok_or("Missing track")?;
            if target.locked {return Err("Track is locked".into());} target.clips.push(clip.clone());
        }
        TimelineEdit::AddTransition { transition } => {
            if timeline.transitions.len() >= MAX_TRANSITIONS || timeline.transitions.iter().any(|t|t.id==transition.id) {return Err("Transition ID already exists or transition limit reached".into());}
            timeline.check_transition(transition,true)?; timeline.transitions.push(transition.clone());
        }
        TimelineEdit::SetTrackLocked { id, locked } => {
            let target=timeline.tracks.iter_mut().find(|t| &t.id==id).ok_or("Missing track")?;
            target.locked=*locked;
        }
        TimelineEdit::RemoveTrack { id } => {
            let index=timeline.tracks.iter().position(|t|&t.id==id).ok_or("Missing track")?;
            if timeline.tracks[index].locked {return Err("Track is locked".into());}
            if !timeline.tracks[index].clips.is_empty() || timeline.transitions.iter().any(|t|&t.track==id) {return Err("Remove track contents before removing the track".into());}
            timeline.tracks.remove(index);
        }
        TimelineEdit::UpdateClip { track, clip } => {
            let target=timeline.tracks.iter_mut().find(|t|&t.id==track).ok_or("Missing track")?;
            if target.locked {return Err("Track is locked".into());}
            let index=target.clips.iter().position(|c|c.id==clip.id).ok_or("Clip is not on the selected track")?;
            target.clips[index]=clip.clone();
        }
        TimelineEdit::RemoveClip { track, id } => {
            if timeline.transitions.iter().any(|t|&t.from==id || &t.to==id) {return Err("Remove dependent transitions before removing the clip".into());}
            let target=timeline.tracks.iter_mut().find(|t|&t.id==track).ok_or("Missing track")?;
            if target.locked {return Err("Track is locked".into());}
            let index=target.clips.iter().position(|c|&c.id==id).ok_or("Clip is not on the selected track")?;
            target.clips.remove(index);
        }
        TimelineEdit::RemoveTransition { id } => {
            let index=timeline.transitions.iter().position(|t|&t.id==id).ok_or("Missing transition")?;
            let track=timeline.tracks.iter().find(|t|t.id==timeline.transitions[index].track).ok_or("Missing transition track")?;
            if track.locked {return Err("Track is locked".into());}
            timeline.transitions.remove(index);
        }
        _ => {}
    }
    timeline.validate()?;
    let mut out = if current.is_some() { "poet_timeline = ".to_owned() } else { "let mut poet_timeline = ".to_owned() };
    push_bounded(&mut out,&format!("{{schema:\"{}\",frame_ticks:{}u64,tracks:[", SCHEMA,timeline.frame_ticks))?;
    for (i,t) in timeline.tracks.iter().enumerate() { if i>0 {push_bounded(&mut out,",")?;} push_bounded(&mut out,&format!("{{id:\"{}\",locked:{},clips:[",t.id,t.locked))?;
        for (j,c) in t.clips.iter().enumerate() { if j>0 {push_bounded(&mut out,",")?;} let source = serde_json::to_string(&c.source).map_err(|e|format!("Timeline source string encoding failed: {e}"))?; push_bounded(&mut out,&format!("{{id:\"{}\",source:{},source_start:{}u64,start:{}u64,duration:{}u64}}",c.id,source,c.source_start,c.start,c.duration))?; } push_bounded(&mut out,"]}")?; }
    push_bounded(&mut out,"],transitions:[")?;
    for (i,t) in timeline.transitions.iter().enumerate() { if i>0 {push_bounded(&mut out,",")?;} push_bounded(&mut out,&format!("{{id:\"{}\",kind:\"{}\",track:\"{}\",from:\"{}\",to:\"{}\",start:{}u64,duration:{}u64}}",t.id,t.kind,t.track,t.from,t.to,t.start,t.duration))?; }
    push_bounded(&mut out,if has_parts { "]};\n({parts:poet_parts});" } else { "]};\n({timeline:poet_timeline});" })?;
    Ok(out)
}
