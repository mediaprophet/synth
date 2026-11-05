//! Explicit source-level clip -> HMC-resource assertions. They are not reads,
//! grants, equivalence claims, nor proof that a selected package contains data.
use super::{source, timeline::Timeline};
use std::collections::{BTreeMap,BTreeSet};
use vibe::Value;

const SCHEMA:&str="poet.timeline-resource-associations/v1";
const MAX_BINDINGS:usize=512;
const MAX_CELL:usize=120*1024;

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Association { pub clip_id:String, pub source_identifier:String, pub resource_key:String, pub resource_digest:[u8;32] }

fn safe_text(s:&str,cap:usize)->bool { !s.is_empty() && s.len()<=cap && !s.chars().any(char::is_control) }
fn hex(bytes:&[u8;32])->String { bytes.iter().map(|v|format!("{v:02x}")).collect() }
fn digest(s:&str)->Result<[u8;32],String> {
    if s.len()!=64 || !s.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)) {return Err("Resource digest must be 64 lowercase hex digits".into());}
    let mut out=[0;32]; for (i,slot) in out.iter_mut().enumerate(){*slot=u8::from_str_radix(&s[i*2..i*2+2],16).map_err(|_|"Invalid digest")?;} Ok(out)
}
fn record<'a>(v:&'a Value,keys:&[&str])->Result<&'a BTreeMap<String,Value>,String>{let r=source::record(v)?; if r.len()!=keys.len()||keys.iter().any(|k|!r.contains_key(*k)){return Err("Unknown or missing timeline association field".into());} Ok(r)}
fn text<'a>(r:&'a BTreeMap<String,Value>,k:&str)->Result<&'a str,String>{source::text(source::field(r,k)?)}
fn clip_source<'a>(timeline:&'a Timeline,id:&str)->Result<&'a str,String>{timeline.tracks.iter().flat_map(|t|&t.clips).find(|c|c.id==id).map(|c|c.source.as_str()).ok_or("Association refers to a missing clip".into())}
fn require_unlocked(timeline:&Timeline,id:&str)->Result<(),String>{let t=timeline.tracks.iter().find(|t|t.clips.iter().any(|c|c.id==id)).ok_or("Missing clip")?;if t.locked {Err("Track is locked".into())}else{Ok(())}}

pub fn read(current:Option<&Value>, timeline:&Timeline)->Result<Vec<Association>,String>{
    timeline.validate()?; let Some(v)=current else{return Ok(Vec::new())};
    let r=record(v,&["schema","bindings"])?; if text(r,"schema")?!=SCHEMA{return Err("Unknown timeline resource association schema".into());}
    let Value::List(items)=source::field(r,"bindings")? else{return Err("Association bindings must be a list".into());};
    if items.len()>MAX_BINDINGS{return Err("Timeline association limit exceeded".into());}
    let mut result=Vec::with_capacity(items.len()); let mut seen=BTreeSet::new();
    for item in items {let b=record(item,&["clip_id","source_identifier","resource_key","resource_digest"])?;
        let clip_id=text(b,"clip_id")?.to_owned(); let source_identifier=text(b,"source_identifier")?.to_owned(); let resource_key=text(b,"resource_key")?.to_owned(); let resource_digest=digest(text(b,"resource_digest")?)?;
        if !seen.insert(clip_id.clone()) || !safe_text(&resource_key,4096) || resource_digest==[0;32] || clip_source(timeline,&clip_id)? != source_identifier {return Err("Invalid, duplicate or stale timeline resource association".into());}
        result.push(Association{clip_id,source_identifier,resource_key,resource_digest});
    }
    Ok(result)
}
pub fn validate_pair(timeline:Option<&Value>,media:Option<&Value>)->Result<(),String>{
    if media.is_none(){return Ok(());} let timeline=super::timeline::read(timeline)?.ok_or("Timeline associations lack a timeline")?;
    read(media,&timeline).map(|_|())
}

/// Create an explicit asserted relation. Caller must separately check the selected
/// package and resource under its live authority before any media access.
pub fn bind(current:Option<&Value>,timeline:&Timeline,clip_id:&str,key:&str,resource_digest:[u8;32],has_parts:bool)->Result<String,String>{
    require_unlocked(timeline,clip_id)?;
    if !safe_text(key,4096)||resource_digest==[0;32]{return Err("Invalid HMC resource key or digest".into());}
    let mut list=read(current,timeline)?; let source_identifier=clip_source(timeline,clip_id)?.to_owned();
    if list.iter().any(|b|b.clip_id==clip_id){return Err("Clip already has a resource association; remove it explicitly first".into());}
    if list.len()>=MAX_BINDINGS{return Err("Timeline association limit exceeded".into());}
    list.push(Association{clip_id:clip_id.into(),source_identifier,resource_key:key.into(),resource_digest});
    source_for(&list,current.is_some(),has_parts)
}
pub fn unbind(current:Option<&Value>,timeline:&Timeline,clip_id:&str,has_parts:bool)->Result<String,String>{
    require_unlocked(timeline,clip_id)?;
    let mut list=read(current,timeline)?; let i=list.iter().position(|b|b.clip_id==clip_id).ok_or("Clip has no resource association")?;
    list.remove(i); source_for(&list,true,has_parts)
}
fn push(out:&mut String,value:&str)->Result<(),String>{if out.len().checked_add(value.len()).is_none_or(|n|n>MAX_CELL){return Err("Timeline associations exceed source bound".into());}out.push_str(value);Ok(())}
fn quote(s:&str)->Result<String,String>{serde_json::to_string(s).map_err(|e|format!("String encoding failed: {e}"))}
fn source_for(bindings:&[Association],existing:bool,has_parts:bool)->Result<String,String>{
    let mut out=if existing{"poet_timeline_media = ".to_owned()}else{"let mut poet_timeline_media = ".to_owned()};
    push(&mut out,&format!("{{schema:\"{SCHEMA}\",bindings:["))?;
    for (i,b) in bindings.iter().enumerate(){if i>0{push(&mut out,",")?;}push(&mut out,&format!("{{clip_id:{},source_identifier:{},resource_key:{},resource_digest:\"{}\"}}",quote(&b.clip_id)?,quote(&b.source_identifier)?,quote(&b.resource_key)?,hex(&b.resource_digest)))?;}
    push(&mut out,if has_parts{"]};\n({parts:poet_parts});"}else{"]};\n({timeline:poet_timeline});"})?;Ok(out)
}
