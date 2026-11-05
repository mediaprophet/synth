//! Controlled software-fixture reference production through Poet's public host.
//! No human admission, production trust anchor, OS credentials or master acceptance.
mod authority;
mod botanical;
mod files;
mod package;
mod produce;
mod render;

use poet_cli::graphical::{
    assembly,
    checkpoints::{CheckpointLimits, PackageWritePolicy},
    controller::PreviewProvider,
    cooperation::{GatewayLimits, Scope},
    native_project::ProjectConfig,
    preview::{PreviewFrame, PreviewLimits, SourceRef},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use vibe::{session::SessionLimits, Value};

const HMC: &str = "growing-station.hmc";
const ASSET: &str = "growing-station.10d";
const Q42_KEY: &str = "graphs/growing-station.10d.q42";
const Q42_ASSET: &str = "growing-station.q42";
const M64_ASSET: &str = "growing-station.m64";
const REQUEST: &str = "software-source-review/1";
const MAX_VERTICES: usize = 1_000_000;
const MAX_INDICES: usize = 6_000_000;
const MASTER_KEY: &str = "masters/growing-station/master.vibe";
const PRESET_KEY: &str = "masters/growing-station/preset.vibe";
const ACCEPTED_KEY: &str = "masters/growing-station/accepted-source.vibe";
const RENDER_WIDTH: u32 = 1280;
const RENDER_HEIGHT: u32 = 720;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunInfo {
    schema: String,
    fixture_only: bool,
    project_id: String,
    document_id: String,
    source_id: String,
    profile: String,
    master_sha256: String,
    preset_sha256: String,
    accepted_source_sha256: String,
    hmc_sha256: String,
    compiled_sha256: String,
    q42_sha256: String,
    m64_sha256: String,
    target: qualia_cooperative_core::authoring::DocumentRevision,
}
impl RunInfo {
    fn scope(&self) -> Scope {
        Scope {
            project_id: self.project_id.clone(),
            document_id: self.document_id.clone(),
            source_id: self.source_id.clone(),
            domain: "Geometry".into(),
        }
    }
    fn validate(&self) -> Result<(), String> {
        if self.schema != "poet-growing-reference/software-fixture/1"
            || !self.fixture_only
            || !self
                .project_id
                .starts_with("urn:poet:software-fixture:cg-gs:")
            || self.document_id != format!("{}:document", self.project_id)
            || self.source_id != format!("{}:source", self.project_id)
            || self.target.project_id != self.project_id
            || self.target.document_id != self.document_id
            || self.target.source_id != self.source_id
            || !matches!(
                self.profile.as_str(),
                "film" | "open_world_game" | "cartoon"
            )
        {
            return Err("unsupported or non-fixture reference metadata".into());
        }
        Ok(())
    }
}
fn limits() -> PreviewLimits {
    PreviewLimits {
        side: 1024,
        vertices: MAX_VERTICES,
        indices: MAX_INDICES,
    }
}
struct ValidateOnly;
impl PreviewProvider for ValidateOnly {
    fn retire_retained_viewport(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn validate(&self, value: &Value) -> Result<(), String> {
        if assembly::is_assembly(value) {
            assembly::admit_assembly(value, limits())?;
        }
        Ok(())
    }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("Pixels are produced separately from the reopened HMC .10d, not live source".into())
    }
}
fn config(scope: Scope, root: &Path) -> ProjectConfig {
    let mut session_limits = SessionLimits::default();
    session_limits.retained_value_bytes = 128 * 1024 * 1024;
    ProjectConfig {
        scope,
        checkpoint_root: root.join("hmc"),
        checkpoint_limits: CheckpointLimits::default(),
        write_policy: PackageWritePolicy::WorkingCopy,
        import_library: None,
        session_limits,
        gateway_limits: GatewayLimits::default(),
        geometry_grants: [
            "capability.invoke",
            "ComputationalGeometry.extrude_profile",
            "ComputationalGeometry.profile_sweep",
            "ComputationalGeometry.uv_sphere",
            "ComputationalGeometry.torus",
            "ComputationalGeometry.space_colonize_3d",
            "ComputationalGeometry.pipe_model_radii",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        max_vertices: MAX_VERTICES,
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("growing_reference failed closed: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("produce") if args.len() == 5 => {
            let root = files::existing_directory(Path::new(&args[2]), true)?;
            let source = files::existing_directory(Path::new(&args[4]), false)?;
            produce::produce(&root, &args[3], &source)?;
            // A fresh process cannot reuse the author's Studio/interpreter state.
            let status = std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
                .arg("reopen-render").arg(&root).status().map_err(|e| e.to_string())?;
            if !status.success() { return Err(format!("fresh-process reopen/render exited {status}")); }
            Ok(())
        }
        Some("reopen-render") if args.len() == 3 => {
            let root = files::existing_directory(Path::new(&args[2]), false)?;
            package::reopen(&root, true, None)
        }
        Some("produce-botanical") if args.len() == 4 => {
            let root = files::existing_directory(Path::new(&args[2]), true)?;
            botanical::produce_botanical(&root, &args[3])?;
            let status = std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
                .arg("reopen-render-botanical").arg(&root).status().map_err(|e| e.to_string())?;
            if !status.success() { return Err(format!("fresh-process botanical reopen/render exited {status}")); }
            Ok(())
        }
        Some("reopen-render-botanical") if args.len() == 3 => {
            let root = files::existing_directory(Path::new(&args[2]), false)?;
            botanical::reopen_botanical(&root)
        }
        Some("review") if args.len() == 5 => {
            let root = files::existing_directory(Path::new(&args[2]), false)?;
            package::reopen(&root, false, Some((&args[3], &args[4])))
        }
        _ => Err("usage: growing_reference produce EXISTING_EMPTY_OUTPUT {film|open_world_game|cartoon} ASSET_SOURCE_ROOT | produce-botanical EXISTING_EMPTY_OUTPUT {film|open_world_game|cartoon} | reopen-render OUTPUT | reopen-render-botanical OUTPUT | review OUTPUT EVIDENCE_FILENAME {accepted|changes-requested|rejected}".into()),
    }
}

fn metadata(root: &Path) -> Result<RunInfo, String> {
    let info: RunInfo = serde_json::from_slice(&files::read(&root.join("run.json"), 64 * 1024)?)
        .map_err(|e| e.to_string())?;
    info.validate()?;
    Ok(info)
}
fn hmc_path(root: &Path) -> PathBuf {
    root.join("hmc").join(HMC)
}
