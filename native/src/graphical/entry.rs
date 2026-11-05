//! Explicit local diagnostic launch. CLI configuration is not production
//! authentication, cooperative admission or authority reconstructed from HMC.
use super::checkpoints::{
    CheckpointEditorHost, CheckpointLimits, CheckpointService, ExpectedFile, SaveReceipt,
    TrustedBinding,
};
use super::controller::{
    CanonicalEditorHost, EditorHost, EditorOutcome, EditorSnapshot, MeshPreviewProvider,
};
use super::geometry_host::GeometryHost;
use super::materials::diagnostic_source;
use super::preview::{PreviewCamera, SourceRef};
use qualia_core_db::poet_host::invoke::studio_store::{self, checkpoint as core, StudioArtifact};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use vibe::session::SourceSession;

static NEXT_DIAGNOSTIC: AtomicU64 = AtomicU64::new(1);
const USAGE: &str = "Usage: poet-native-authoring --diagnostic-source [--checkpoint-root EXISTING_DIRECTORY] [--reopen FILE.hmc] [--grant capability.invoke] [--grant Sheet.set_cell] [--grant Sheet.eval_formula] [--grant Sheet.stats] [--grant Sheet.sum_range] [--grant ComputationalGeometry.extrude_profile] [--grant ComputationalGeometry.swung_surface] [--grant ComputationalGeometry.profile_sweep] [--grant ComputationalGeometry.uv_sphere] [--grant ComputationalGeometry.torus] [--grant ComputationalGeometry.revolve_profile] [--grant ComputationalGeometry.space_colonize_3d] [--grant ComputationalGeometry.pipe_model_radii] [--check-source]. Legacy --diagnostic-new-source admits new source only.";
const DIAGNOSTIC: &str = "LOCAL DIAGNOSTIC · no production authentication or cooperative admission";

#[derive(Default)]
struct Options {
    diagnostic: bool,
    new_only: bool,
    grants: Vec<String>,
    root: Option<PathBuf>,
    reopen: Option<String>,
    check_source: bool,
}
fn options(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--diagnostic-source" => options.diagnostic = true,
            "--diagnostic-new-source" => {
                options.diagnostic = true;
                options.new_only = true;
            }
            "--check-source" => options.check_source = true,
            "--checkpoint-root" | "--reopen" | "--grant" => {
                let value = args
                    .get(index + 1)
                    .filter(|value| !value.is_empty() && !value.starts_with("--"))
                    .ok_or_else(|| USAGE.to_string())?;
                match args[index].as_str() {
                    "--checkpoint-root" if options.root.is_none() => {
                        options.root = Some(PathBuf::from(value))
                    }
                    "--reopen" if options.reopen.is_none() => {
                        filename(value)?;
                        options.reopen = Some(value.clone());
                    }
                    "--grant"
                        if matches!(
                            value.as_str(),
                            "capability.invoke"
                                | "Sheet.set_cell"
                                | "Sheet.eval_formula"
                                | "Sheet.stats"
                                | "Sheet.sum_range"
                                | "ComputationalGeometry.extrude_profile"
                                | "ComputationalGeometry.swung_surface"
                                | "ComputationalGeometry.profile_sweep"
                                | "ComputationalGeometry.uv_sphere"
                                | "ComputationalGeometry.torus"
                                | "ComputationalGeometry.revolve_profile"
                                | "ComputationalGeometry.space_colonize_3d"
                                | "ComputationalGeometry.pipe_model_radii"
                        ) =>
                    {
                        if !options.grants.contains(value) {
                            options.grants.push(value.clone());
                        }
                    }
                    _ => return Err(USAGE.into()),
                }
                index += 1;
            }
            _ => return Err(USAGE.into()),
        }
        index += 1;
    }
    if !options.diagnostic {
        return Err("Explicit local diagnostic profile required; production admission belongs to the native host".into());
    }
    if options.reopen.is_some()
        && (options.new_only || options.root.is_none() || options.check_source)
    {
        return Err("Reopen requires --diagnostic-source and --checkpoint-root; legacy new-only and --check-source cannot reopen".into());
    }
    Ok(options)
}

// Validate before the first discovery load, before the service can bind the
// file's inert source identifier. The native service rechecks its own path.
fn filename(name: &str) -> Result<(), String> {
    if name.len() <= 4
        || name.len() > 128
        || name.starts_with('.')
        || !name.to_ascii_lowercase().ends_with(".hmc")
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    {
        return Err(
            "Reopen requires one plain .hmc filename within the configured directory".into(),
        );
    }
    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
    {
        return Err("Reopen filename is an OS device name".into());
    }
    Ok(())
}
fn checkpoint_limits() -> CheckpointLimits {
    // Diagnostic ingress budget only, never a production master quality ceiling.
    CheckpointLimits {
        file_bytes: 16 * 1024 * 1024,
        source_bytes: 4 * 1024 * 1024,
        entries: 128,
        ..Default::default()
    }
}
fn binding(id: &str, domain: &str) -> TrustedBinding {
    TrustedBinding {
        project_id: "urn:ecopunk:diagnostic:local-checkpoints".into(),
        document_id: format!("urn:ecopunk:diagnostic:document:{id}"),
        source_id: id.into(),
        domain: domain.into(),
    }
}
fn discover(root: &Path, name: &str) -> Result<(String, String), String> {
    filename(name)?;
    let directory = root.canonicalize().map_err(|error| error.to_string())?;
    if !directory.is_dir() {
        return Err("Checkpoint root must be an existing directory".into());
    }
    let path = directory.join(name);
    let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err("Diagnostic reopen requires a regular file, not a link or directory".into());
    }
    let checkpoint = core::load(&path, checkpoint_limits())?;
    let artifact = checkpoint.artifact();
    if artifact.id.is_empty() || artifact.domain != "Geometry" {
        return Err("Diagnostic reopen supports identified Geometry source only".into());
    }
    if root.canonicalize().map_err(|error| error.to_string())? != directory {
        return Err("Configured checkpoint directory changed during discovery".into());
    }
    Ok((artifact.id.clone(), artifact.domain.clone()))
}

pub struct PreparedStudio {
    pub host: Box<dyn EditorHost>,
    pub initial_draft: String,
}

/// Headless factory: no source evaluation, renderer/window launch, authentication or
/// grant recovery. Reopened sources remain held until explicit operator rebuild.
pub fn prepare(args: &[String]) -> Result<PreparedStudio, String> {
    let options = options(args)?;
    if options.check_source {
        return Err("--check-source uses run's no-launch source preparation path".into());
    }
    prepare_options(options)
}
fn prepare_options(options: Options) -> Result<PreparedStudio, String> {
    let profile = super::native_authoring_profile::selected();
    let (id, service, reopened) = if let Some(name) = options.reopen.as_deref() {
        let root = options
            .root
            .as_deref()
            .ok_or("Diagnostic reopen root is missing")?;
        let (id, domain) = discover(root, name)?;
        // These diagnostic URIs are supplied by this explicit local CLI profile;
        // file fields provide inert source data, never a principal or admission.
        let service = CheckpointService::new(root, binding(&id, &domain), checkpoint_limits())?;
        let prepared = service.prepare_reopen(name)?;
        service.activate(prepared)?; // fresh store only; existing IDs never replaced
        (id, Some(service), true)
    } else {
        let id = format!(
            "urn:ecopunk:diagnostic:poet-source:{}:{}",
            std::process::id(),
            NEXT_DIAGNOSTIC.fetch_add(1, Ordering::Relaxed)
        );
        let service = options
            .root
            .as_deref()
            .map(|root| CheckpointService::new(root, binding(&id, "Geometry"), checkpoint_limits()))
            .transpose()?;
        studio_store::create_artifact_if_absent(StudioArtifact::new(&id, "Geometry", DIAGNOSTIC))?;
        (id, service, false)
    };
    let mut material = qualia_core_db::container_10d::MaterialRecord::legacy_default();
    material.id = qualia_core_db::q_hash("urn:ecopunk:diagnostic:preview-material");
    material.base_color = [0.36, 0.20, 0.085, 1.0];
    material.roughness = 0.65;
    let provider = MeshPreviewProvider {
        material,
        camera: PreviewCamera {
            yaw: 0.45,
            pitch: 0.2,
            distance: 3.2,
            target: [0.0; 3],
        },
        limits: profile.preview_limits(),
    };
    let mut rebuild_limits = super::native_authoring_profile::startup_rebuild_limits(profile);
    if options.grants.iter().any(|g| {
        g == "ComputationalGeometry.space_colonize_3d"
            || g == "ComputationalGeometry.pipe_model_radii"
    }) {
        rebuild_limits.total_geometry_work = rebuild_limits.total_geometry_work.max(66_000_000);
        rebuild_limits.total_kernel_dispatches = rebuild_limits.total_kernel_dispatches.max(512);
    }
    let canonical = CanonicalEditorHost::open_with_rebuild_limits(
        id,
        profile.session_limits(),
        GeometryHost::new(options.grants, profile.max_vertices()),
        Box::new(
            super::viewport_provider::InteractivePreviewProvider::with_budget(
                provider,
                profile.viewport_budget(),
            )?,
        ),
        rebuild_limits,
    )?;
    let host: Box<dyn EditorHost> = if let Some(service) = service {
        Box::new(CheckpointEditorHost::new(Box::new(canonical), service)?)
    } else {
        Box::new(canonical)
    };
    let initial_draft = if reopened {
        host.snapshot()?.latest_source
    } else {
        diagnostic_source()
    };
    Ok(PreparedStudio {
        host: Box::new(DiagnosticEditor {
            inner: host,
            pending_reopen: reopened,
        }),
        initial_draft,
    })
}

pub fn run(args: &[String]) -> Result<(), String> {
    if args.first().is_some_and(|arg| arg == "--native-project") {
        return super::native_entry::run(&args[1..]);
    }
    let options = options(args)?;
    eprintln!("{DIAGNOSTIC}. Checkpoint files do not restore grants; reopened execution requires Rebuild.");
    if options.check_source {
        SourceSession::new(
            options.grants,
            super::native_authoring_profile::selected().session_limits(),
        )
        .prepare(&diagnostic_source())
        .map_err(|error| error.message)?;
        println!("Canonical diagnostic source preparation passed; no evaluation/render/window attempted.");
        return Ok(());
    }
    let prepared = prepare(args)?;
    super::launch(prepared.host, prepared.initial_draft)
}

// Label the UI projection without changing accepted source bytes, stored title,
// revision, provenance or HMC digest. Even an empty reopened source requires an
// explicit local rebuild before execution. This wrapper grants no authority.
struct DiagnosticEditor {
    inner: Box<dyn EditorHost>,
    pending_reopen: bool,
}
fn label(mut snapshot: EditorSnapshot, held: bool, checkpoints: bool) -> EditorSnapshot {
    if !snapshot.title.starts_with(DIAGNOSTIC) {
        snapshot.title = format!("{DIAGNOSTIC} · {}", snapshot.title);
    }
    snapshot.reconciliation_required |= held;
    snapshot.checkpoint_supported = checkpoints;
    if held {
        snapshot.material_edit_id = None;
    }
    snapshot
}
fn outcome(mut result: EditorOutcome, held: bool, checkpoints: bool) -> EditorOutcome {
    result.snapshot = label(result.snapshot, held, checkpoints);
    result
}
impl DiagnosticEditor {
    fn ready(&self) -> Result<(), String> {
        if self.pending_reopen {
            Err("Diagnostic reopened source is held; explicitly Rebuild with current CLI grants before checking or executing edits".into())
        } else {
            Ok(())
        }
    }
}
impl EditorHost for DiagnosticEditor {
    fn retained_surface_carrier(
        &self,
        expected: SourceRef,
        limits: qualia_core_db::poet_host::invoke::studio_store::checkpoint::CheckpointLimits,
    ) -> Result<
        Option<qualia_core_db::poet_host::invoke::studio_store::checkpoint::VerifiedSourceCarrier>,
        String,
    > {
        self.inner.retained_surface_carrier(expected, limits)
    }
    fn retained_surface_inputs(
        &self,
        expected: SourceRef,
    ) -> Result<
        Vec<qualia_core_db::poet_host::invoke::studio_store::checkpoint::ResourceData>,
        String,
    > {
        self.inner.retained_surface_inputs(expected)
    }

    fn inspect_surface(
        &self,
        id: &str,
        expected: SourceRef,
    ) -> Result<crate::graphical::surfaces::inspection::SurfaceInspection, String> {
        self.inner.inspect_surface(id, expected)
    }

    fn numeric_sheet_view(&self, expected: SourceRef) -> Result<crate::graphical::numeric_sheet::SheetView, String> { self.ready()?; self.inner.numeric_sheet_view(expected) }
    fn numeric_sheet_edit(&mut self, edit: crate::graphical::numeric_sheet::SheetEdit, expected: SourceRef) -> Result<EditorOutcome, String> { self.ready()?; let r = self.inner.numeric_sheet_edit(edit, expected)?; Ok(outcome(r, false, self.inner.checkpoint_supported())) }
    fn authoring_view(
        &self,
        e: SourceRef,
    ) -> Result<crate::graphical::authoring::AuthoringView, String> {
        self.ready()?;
        self.inner.authoring_view(e)
    }
    fn timeline_media(&self, request: crate::graphical::checkpoints::TimelineMediaRequest, expected: SourceRef, cancelled: &std::sync::atomic::AtomicBool) -> Result<crate::graphical::checkpoints::TimelineMediaSelection, String> {
        self.ready()?;
        self.inner.timeline_media(request, expected, cancelled)
    }
    fn authoring_edit(
        &mut self,
        a: crate::graphical::authoring::AuthoringEdit,
        e: SourceRef,
    ) -> Result<EditorOutcome, String> {
        self.ready()?;
        let r = self.inner.authoring_edit(a, e)?;
        Ok(outcome(r, false, self.inner.checkpoint_supported()))
    }
    fn compiled_assets(
        &self,
        k: &str,
        e: SourceRef,
    ) -> Result<Vec<(String, qualia_core_db::render::compile_10d::CompiledAsset)>, String> {
        self.ready()?;
        self.inner.compiled_assets(k, e)
    }
    fn compile_checkpoint(
        &mut self,
        n: &str,
        k: &str,
        f: ExpectedFile,
        e: SourceRef,
    ) -> Result<SaveReceipt, String> {
        self.ready()?;
        self.inner.compile_checkpoint(n, k, f, e)
    }

    fn snapshot(&self) -> Result<EditorSnapshot, String> {
        self.inner.snapshot().map(|snapshot| {
            label(
                snapshot,
                self.pending_reopen,
                self.inner.checkpoint_supported(),
            )
        })
    }
    fn check(&self, source: &str) -> Result<String, String> {
        self.ready()?;
        self.inner.check(source)
    }
    fn source_cells(&self, expected: SourceRef) -> Result<Vec<String>, String> {
        self.inner.source_cells(expected)
    }
    fn submit(&mut self, source: &str, expected: SourceRef) -> Result<EditorOutcome, String> {
        self.ready()?;
        self.inner.submit(source, expected).map(|result| {
            outcome(
                result,
                self.pending_reopen,
                self.inner.checkpoint_supported(),
            )
        })
    }
    fn material_edit(
        &mut self,
        source: &str,
        expected: SourceRef,
    ) -> Result<EditorOutcome, String> {
        self.ready()?;
        self.inner.material_edit(source, expected).map(|result| {
            outcome(
                result,
                self.pending_reopen,
                self.inner.checkpoint_supported(),
            )
        })
    }
    fn undo(&mut self, expected: SourceRef) -> Result<EditorSnapshot, String> {
        self.inner.undo(expected).map(|snapshot| {
            label(
                snapshot,
                self.pending_reopen,
                self.inner.checkpoint_supported(),
            )
        })
    }
    fn redo(&mut self, expected: SourceRef) -> Result<EditorSnapshot, String> {
        self.inner.redo(expected).map(|snapshot| {
            label(
                snapshot,
                self.pending_reopen,
                self.inner.checkpoint_supported(),
            )
        })
    }
    fn rebuild_execution(&mut self, expected: SourceRef) -> Result<EditorOutcome, String> {
        let result = self.inner.rebuild_execution(expected)?;
        self.pending_reopen = false;
        Ok(outcome(result, false, self.inner.checkpoint_supported()))
    }
    fn checkpoint_supported(&self) -> bool {
        self.inner.checkpoint_supported()
    }
    fn save_checkpoint(
        &mut self,
        name: &str,
        file: core::ExpectedFile,
        expected: SourceRef,
    ) -> Result<core::SaveReceipt, String> {
        self.inner.save_checkpoint(name, file, expected)
    }
}

#[cfg(test)]
#[path = "entry_tests.rs"]
mod tests;
