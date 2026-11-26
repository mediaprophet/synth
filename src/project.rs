//! Attached Synth project: working files, delivered files, and user annotations.
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

const SCHEMA: &str = "synth.project/v1";
const ANNOTATION_SCHEMA: &str = "synth.project.annotations/v1";
const MAX_LISTED_FILES: usize = 400;
const MAX_FILE_BYTES: u64 = 32 * 1024 * 1024;

static PROJECT: OnceLock<Project> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub name: String,
    pub brief: String,
    pub sources: Vec<PathBuf>,
    pub working: PathBuf,
    pub delivered: PathBuf,
    pub store: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ProjectFile {
    #[serde(default)]
    schema: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    brief: String,
    #[serde(default)]
    sources: Vec<String>,
    #[serde(default)]
    working: String,
    #[serde(default)]
    delivered: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationLink {
    pub url: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub created_at: String,
    pub lane: String,
    pub file: String,
    #[serde(default)]
    pub aspect: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub rating: u8,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub links: Vec<AnnotationLink>,
    #[serde(default)]
    pub camera_yaw: Option<f32>,
    #[serde(default)]
    pub camera_pitch: Option<f32>,
    #[serde(default)]
    pub camera_distance: Option<f32>,
    #[serde(default)]
    pub time_seconds: Option<f32>,
    #[serde(default)]
    pub frame_index: Option<u32>,
    #[serde(default)]
    pub screen_pin_uv: Option<[f32; 2]>,
    #[serde(default)]
    pub addressed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnnotationStore {
    schema: String,
    items: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListedFile {
    pub lane: String,
    pub rel: String,
    pub bytes: u64,
    pub kind: String,
    pub previewable: bool,
}

impl Project {
    fn from_root(root: PathBuf) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|e| format!("project path: {e}"))?;
        if !root.is_dir() {
            return Err("project path must be a directory".into());
        }
        let yaml_manifest = root.join("manifest.q42.yaml");
        let yaml_project = root.join("synth.project.yaml");
        let json_project = root.join("synth.project.json");
        let parsed: ProjectFile = if yaml_manifest.is_file() || yaml_project.is_file() {
            let ypath = if yaml_manifest.is_file() {
                &yaml_manifest
            } else {
                &yaml_project
            };
            let raw = fs::read(ypath).map_err(|e| e.to_string())?;
            let cfg = qualia_core_db::q42::yaml_ld_q42::extract_synth_project_from_yaml_ld(&raw)
                .map_err(|e| format!("{}: {e}", ypath.display()))?;
            ProjectFile {
                schema: SCHEMA.into(),
                name: cfg.name,
                brief: cfg.brief,
                sources: cfg.sources,
                working: cfg.working,
                delivered: cfg.delivered,
            }
        } else if json_project.is_file() {
            let raw = fs::read_to_string(&json_project).map_err(|e| e.to_string())?;
            serde_json::from_str(&raw).map_err(|e| format!("synth.project.json: {e}"))?
        } else {
            ProjectFile {
                schema: SCHEMA.into(),
                name: root
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("project")
                    .into(),
                brief: String::new(),
                sources: Vec::new(),
                working: "working".into(),
                delivered: "delivered".into(),
            }
        };
        let name = if parsed.name.trim().is_empty() {
            root.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("project")
                .to_string()
        } else {
            parsed.name
        };
        let working_rel = if parsed.working.trim().is_empty() {
            "working"
        } else {
            parsed.working.trim()
        };
        let delivered_rel = if parsed.delivered.trim().is_empty() {
            "delivered"
        } else {
            parsed.delivered.trim()
        };
        let working = resolve_dir(&root, working_rel, true)?;
        let delivered = resolve_dir(&root, delivered_rel, false)?;
        let mut sources = Vec::new();
        for rel in parsed.sources {
            let path = root.join(rel);
            if path.is_dir() || path.is_file() {
                sources.push(path);
            }
        }
        if sources.is_empty() {
            let production = root.join("master").join("production");
            if production.is_dir() {
                sources.push(production);
            }
        }
        let store = root.join(".synth");
        fs::create_dir_all(&store).map_err(|e| format!(".synth: {e}"))?;
        let _ = parsed.schema;
        Ok(Self {
            root,
            name,
            brief: parsed.brief,
            sources,
            working,
            delivered,
            store,
        })
    }

    pub fn annotation_path(&self) -> PathBuf {
        self.store.join("annotations.json")
    }

    pub fn list_lane(&self, lane: &str) -> Result<Vec<ListedFile>, String> {
        let dir = match lane {
            "working" => &self.working,
            "delivered" => &self.delivered,
            _ => return Err("lane must be working or delivered".into()),
        };
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        collect_files(dir, dir, lane, &mut out)?;
        out.sort_by(|a, b| a.rel.cmp(&b.rel));
        if out.len() > MAX_LISTED_FILES {
            out.truncate(MAX_LISTED_FILES);
        }
        Ok(out)
    }

    pub fn list_sources(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for src in &self.sources {
            if src.is_file() {
                files.push(src.clone());
            } else if src.is_dir() {
                if let Ok(rd) = fs::read_dir(src) {
                    for entry in rd.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("vibe") {
                            files.push(path);
                        }
                    }
                }
            }
        }
        files.sort();
        files
    }

    pub fn resolve_file(&self, lane: &str, rel: &str) -> Result<PathBuf, String> {
        let base = match lane {
            "working" => &self.working,
            "delivered" => &self.delivered,
            _ => return Err("lane must be working or delivered".into()),
        };
        if rel.is_empty() || rel.contains('\0') {
            return Err("empty path".into());
        }
        let joined = base.join(rel.replace('\\', "/"));
        let canon = joined
            .canonicalize()
            .map_err(|_| "file not in project".to_string())?;
        if !canon.starts_with(base) {
            return Err("file not in project".into());
        }
        if !canon.is_file() {
            return Err("not a file".into());
        }
        let meta = fs::metadata(&canon).map_err(|e| e.to_string())?;
        if meta.len() > MAX_FILE_BYTES {
            return Err("file exceeds 32 MiB preview bound".into());
        }
        Ok(canon)
    }

    pub fn load_annotations(&self) -> Result<Vec<Annotation>, String> {
        let path = self.annotation_path();
        if !path.is_file() {
            return Ok(Vec::new());
        }
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let store: AnnotationStore =
            serde_json::from_str(&raw).map_err(|e| format!("annotations: {e}"))?;
        Ok(store.items)
    }

    pub fn add_annotation(&self, mut item: Annotation) -> Result<Annotation, String> {
        if item.file.trim().is_empty() {
            return Err("annotation needs a file".into());
        }
        if item.comment.trim().is_empty()
            && item.tags.is_empty()
            && item.links.is_empty()
            && item.rating == 0
        {
            return Err("write a comment, tag, rating, or link".into());
        }
        item.lane = match item.lane.as_str() {
            "delivered" => "delivered".into(),
            _ => "working".into(),
        };
        item.rating = item.rating.min(5);
        item.tags = item
            .tags
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        item.links.retain(|l| !l.url.trim().is_empty());
        if item.id.is_empty() {
            item.id = next_id();
        }
        if item.created_at.is_empty() {
            item.created_at = now_rfc3339();
        }
        let mut items = self.load_annotations()?;
        items.insert(0, item.clone());
        let store = AnnotationStore {
            schema: ANNOTATION_SCHEMA.into(),
            items,
        };
        let bytes = serde_json::to_vec_pretty(&store).map_err(|e| e.to_string())?;
        fs::write(self.annotation_path(), bytes).map_err(|e| e.to_string())?;
        Ok(item)
    }

    pub fn status_json(&self) -> Result<serde_json::Value, String> {
        let anns = self.load_annotations()?;
        let open_count = anns.iter().filter(|a| !a.addressed).count();
        Ok(json!({
            "schema": SCHEMA,
            "name": self.name,
            "brief": self.brief,
            "root": self.root,
            "working": self.working,
            "delivered": self.delivered,
            "working_files": self.list_lane("working")?,
            "delivered_files": self.list_lane("delivered")?,
            "sources": self.list_sources(),
            "open_annotations_count": open_count,
            "annotations": anns,
        }))
    }

    pub fn mark_annotation_addressed(&self, id: &str) -> Result<bool, String> {
        let mut items = self.load_annotations()?;
        let mut found = false;
        for item in &mut items {
            if item.id == id {
                item.addressed = true;
                found = true;
            }
        }
        if found {
            let store = AnnotationStore {
                schema: ANNOTATION_SCHEMA.into(),
                items,
            };
            let bytes = serde_json::to_vec_pretty(&store).map_err(|e| e.to_string())?;
            fs::write(self.annotation_path(), bytes).map_err(|e| e.to_string())?;
        }
        Ok(found)
    }
}

pub fn dispatch_annotations_mcp(args: serde_json::Value) -> Result<serde_json::Value, String> {
    let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("list");
    let proj = if let Some(dir) = args.get("project_dir").and_then(|v| v.as_str()) {
        Project::from_root(PathBuf::from(dir))?
    } else if let Some(p) = current() {
        p.clone()
    } else if let Ok(root) = std::env::var("SYNTH_OUTPUT_ROOT") {
        Project::from_root(PathBuf::from(root.trim()))?
    } else {
        return Err("No active project; pass project_dir or attach a project".into());
    };

    match action {
        "list" => {
            let items = proj.load_annotations()?;
            let open_count = items.iter().filter(|a| !a.addressed).count();
            Ok(json!({
                "schema": ANNOTATION_SCHEMA,
                "project": proj.name,
                "open_annotations_count": open_count,
                "items": items
            }))
        }
        "mark_addressed" => {
            let id = args
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or("id is required for mark_addressed")?;
            let found = proj.mark_annotation_addressed(id)?;
            Ok(json!({
                "status": if found { "addressed" } else { "not_found" },
                "id": id
            }))
        }
        "add" => {
            let mut input = args;
            let fields = input
                .as_object_mut()
                .ok_or("annotation args must be an object")?;
            fields.entry("id").or_insert_with(|| json!(""));
            fields.entry("created_at").or_insert_with(|| json!(""));
            fields.entry("lane").or_insert_with(|| json!("working"));
            let item: Annotation =
                serde_json::from_value(input).map_err(|e| format!("annotation args: {e}"))?;
            let saved = proj.add_annotation(item)?;
            Ok(json!({"status": "added", "annotation": saved}))
        }
        other => Err(format!(
            "Unknown action `{other}`; expected `list`, `add`, or `mark_addressed`"
        )),
    }
}

pub fn attach(path: Option<PathBuf>) -> Result<(), String> {
    let chosen = match path {
        Some(p) => p,
        None => match std::env::var("SYNTH_PROJECT") {
            Ok(v) if !v.trim().is_empty() => PathBuf::from(v),
            _ => {
                let cwd_file = std::env::current_dir()
                    .map_err(|e| e.to_string())?
                    .join("synth.project.json");
                if cwd_file.is_file() {
                    cwd_file
                        .parent()
                        .map(Path::to_path_buf)
                        .ok_or_else(|| "invalid project".to_string())?
                } else {
                    return Ok(());
                }
            }
        },
    };
    let project = Project::from_root(chosen)?;
    let _ = PROJECT.set(project);
    Ok(())
}

pub fn current() -> Option<&'static Project> {
    PROJECT.get()
}

fn resolve_dir(root: &Path, rel: &str, create: bool) -> Result<PathBuf, String> {
    let path = root.join(rel);
    if create {
        fs::create_dir_all(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    if path.exists() {
        path.canonicalize()
            .map_err(|e| format!("{}: {e}", path.display()))
    } else {
        Ok(path)
    }
}

fn collect_files(
    base: &Path,
    dir: &Path,
    lane: &str,
    out: &mut Vec<ListedFile>,
) -> Result<(), String> {
    if out.len() >= MAX_LISTED_FILES {
        return Ok(());
    }
    let rd = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(()),
    };
    let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if out.len() >= MAX_LISTED_FILES {
            break;
        }
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect_files(base, &path, lane, out)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let kind = file_kind(&rel);
        let previewable = matches!(kind, "png" | "svg" | "jpg" | "html");
        out.push(ListedFile {
            lane: lane.into(),
            rel,
            bytes,
            kind: kind.into(),
            previewable,
        });
    }
    Ok(())
}

pub fn file_kind(rel: &str) -> &'static str {
    match rel.rsplit('.').next().unwrap_or("").to_ascii_lowercase().as_str() {
        "png" | "ppm" => "png",
        "jpg" | "jpeg" => "jpg",
        "webp" => "webp",
        "svg" => "svg",
        "html" | "htm" => "html",
        "10d" => "10d",
        "emf" => "emf",
        // Presentation classification only: byte admission remains the codec's job.
        "qac" => "qac",
        "m64" => "m64",
        "e64" => "e64",
        "p64" => "p64",
        "hmc" => "hmc",
        "vibe" => "vibe",
        "wav" => "wav",
        "json" => "json",
        "q42" => "q42",
        _ => "file",
    }
}

pub fn mime_for(kind: &str) -> &'static str {
    match kind {
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "html" => "text/html; charset=utf-8",
        "wav" => "audio/wav",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}

fn next_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("ann-{nanos}")
}

fn now_rfc3339() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn annotation_mcp_assigns_generated_fields() {
        let dir = std::env::temp_dir().join(format!("synth-project-{}", next_id()));
        fs::create_dir_all(&dir).unwrap();
        let added = dispatch_annotations_mcp(json!({
            "action": "add",
            "project_dir": dir,
            "file": "heart.10d",
            "comment": "Verified the native cutaway silhouette"
        }))
        .unwrap();

        assert_eq!(added["status"], "added");
        let item = &added["annotation"];
        assert!(item["id"].as_str().unwrap().starts_with("ann-"));
        assert!(!item["created_at"].as_str().unwrap().is_empty());
        assert_eq!(item["lane"], "working");
        let reopened = dispatch_annotations_mcp(json!({
            "project_dir": dir,
            "action": "list"
        }))
        .unwrap();
        assert_eq!(&reopened["items"][0], item);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn annotation_requires_substance() {
        let dir = std::env::temp_dir().join(format!("synth-project-{}", next_id()));
        fs::create_dir_all(dir.join("working")).unwrap();
        fs::create_dir_all(dir.join("delivered")).unwrap();
        let project = Project::from_root(dir.clone()).unwrap();
        let err = project
            .add_annotation(Annotation {
                id: String::new(),
                created_at: String::new(),
                lane: "working".into(),
                file: "heart.10d".into(),
                aspect: String::new(),
                tags: vec![],
                rating: 0,
                comment: String::new(),
                links: vec![],
                camera_yaw: None,
                camera_pitch: None,
                camera_distance: None,
                time_seconds: None,
                frame_index: None,
                screen_pin_uv: None,
                addressed: false,
            })
            .unwrap_err();
        assert!(err.contains("comment"));
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn file_kind_classifies_hmc_siblings() {
        assert_eq!(file_kind("mesh.10d"), "10d");
        assert_eq!(file_kind("field.emf"), "emf");
        assert_eq!(file_kind("sound.qac"), "qac");
        assert_eq!(file_kind("legacy.Q4AU"), "file");
        assert_eq!(file_kind("legacy.bin"), "file");
        assert_eq!(mime_for("qac"), "application/octet-stream");
        assert_eq!(file_kind("matter.m64"), "m64");
        assert_eq!(file_kind("alias.e64"), "e64");
        assert_eq!(file_kind("weights.p64"), "p64");
        assert_eq!(file_kind("logic.q42"), "q42");
        assert_eq!(file_kind("pack.hmc"), "hmc");
    }
}
