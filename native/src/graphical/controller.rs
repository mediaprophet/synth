//! Canonical session + Studio source adapter. UI drafts are not document state.
use super::geometry_host::GeometryHost;
use super::preview::SourceRef;
#[cfg(test)]
use super::preview::{self, PreviewFrame};
use qualia_core_db::poet_host::invoke::studio_store::{self, StudioArtifact};
use vibe::{
    session::{SessionLimits, SourceSession},
    Budget, Value,
};
#[path = "execution.rs"]
mod execution;
pub use execution::{RebuildLimits, SessionEpoch};

const SOURCE_PREFIX: &str = "poet.native.submission/";

#[path = "editor_contract.rs"]
mod contract;
pub use contract::{
    EditorHost, EditorOutcome, EditorSnapshot, MeshPreviewProvider, PreviewProvider,
};

#[path = "authoring/controller.rs"]
mod authoring_implementation;

pub struct CanonicalEditorHost {
    surface_resources: Option<crate::graphical::surfaces::TextureResources>,
    artifact_id: String,
    session: SourceSession,
    host: GeometryHost,
    preview: Box<dyn PreviewProvider>,
    reconciliation_required: bool,
    active_stamp: studio_store::StoreStamp,
    factory_limits: SessionLimits,
    rebuild_limits: RebuildLimits,
    execution_epoch: SessionEpoch,
}

#[path = "controller_support.rs"]
mod support;
use support::{material_edit_id, source_cells};
#[path = "controller_viewport.rs"]
mod viewport_implementation;

impl CanonicalEditorHost {
    /// Host-owned factory grants/limits create the canonical interpreter session.
    /// Existing source opens held until explicit read-only profile reconstruction.
    pub fn open(
        artifact_id: String,
        factory_limits: SessionLimits,
        host: GeometryHost,
        preview: Box<dyn PreviewProvider>,
    ) -> Result<Self, String> {
        Self::open_with_rebuild_limits(
            artifact_id,
            factory_limits,
            host,
            preview,
            RebuildLimits::default(),
        )
    }

    pub fn open_with_rebuild_limits(
        artifact_id: String,
        factory_limits: SessionLimits,
        host: GeometryHost,
        preview: Box<dyn PreviewProvider>,
        rebuild_limits: RebuildLimits,
    ) -> Result<Self, String> {
        let (artifact, active_stamp) = studio_store::get_artifact_stamped(&artifact_id)
            .ok_or_else(|| {
                "native Poet requires an existing identified source artifact".to_string()
            })?;
        let reconciliation_required = !source_cells(&artifact)?.is_empty();
        let session = SourceSession::new(host.actual_grants().to_vec(), factory_limits.clone());
        let execution_epoch = SessionEpoch::new();
        let mut preview = preview;
        preview.set_execution_context(&execution_epoch);
        Ok(Self {
            surface_resources: None,
            artifact_id,
            session,
            host,
            preview,
            reconciliation_required,
            active_stamp,
            factory_limits,
            rebuild_limits,
            execution_epoch,
        })
    }

    fn expected(&self, expected: &SourceRef) -> Result<(), String> {
        if expected.artifact_id != self.artifact_id || expected.stamp != self.active_stamp {
            return Err("source/session incarnation mismatch; no evaluation attempted".into());
        }
        expected.check_current()
    }

    fn activate_rebuild(
        &mut self,
        candidate: execution::PreparedRebuild,
    ) -> Result<EditorOutcome, String> {
        // Prepare every fallible read/shape check and the complete response
        // before changing execution. No store read can fail after promotion.
        let latest_source = source_cells(&candidate.artifact)?
            .last()
            .copied()
            .unwrap_or("")
            .to_string();
        let next_epoch = SessionEpoch::new();
        let outcome = EditorOutcome {
            snapshot: EditorSnapshot {
                artifact_id: candidate.artifact.id.clone(), title: candidate.artifact.title.clone(),
                source_revision: candidate.artifact.lamport_clock,
                source: candidate.source.clone(), session_revision: candidate.session.revision(),
                execution_epoch: next_epoch.clone(), rebuild_supported: true, latest_source,
                reconciliation_required: false,
                material_edit_id: material_edit_id(&candidate.session, false),
                checkpoint_supported: false,
                authorisation_stamp: None,
            }, frame: None,
            message: "Execution rebuilt from accepted source under the unchanged read-only geometry profile; no frame published".into(),
        };
        if candidate.source.artifact_id != self.artifact_id
            || candidate.source.check_current().is_err()
        {
            self.reconciliation_required = true;
            return Err(
                "rebuild source could not be revalidated; previous execution/epoch preserved"
                    .into(),
            );
        }
        self.execution_epoch.retire();
        self.session = candidate.session;
        self.host = candidate.host;
        self.active_stamp = candidate.source.stamp;
        self.execution_epoch = next_epoch;
        self.preview.set_execution_context(&self.execution_epoch);
        self.reconciliation_required = false;
        Ok(outcome)
    }
}

impl Drop for CanonicalEditorHost {
    fn drop(&mut self) {
        self.execution_epoch.retire();
    }
}

impl EditorHost for CanonicalEditorHost {
    fn viewport_action(
        &mut self,
        request: crate::graphical::viewport::ViewportCommand,
        expected: SourceRef,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::graphical::viewport_contract::ViewportOutput, String> {
        self.viewport_native(request, expected, cancelled)
    }

    fn retained_surface_carrier(
        &self,
        expected: SourceRef,
        limits: qualia_core_db::poet_host::invoke::studio_store::checkpoint::CheckpointLimits,
    ) -> Result<
        Option<qualia_core_db::poet_host::invoke::studio_store::checkpoint::VerifiedSourceCarrier>,
        String,
    > {
        self.expected(&expected)?;
        // This is inert source retention, available while execution is held.
        // Compilation still requires its existing current-publication gate.
        let carrier = self
            .surface_resources
            .as_ref()
            .map(|r| r.retained_carrier(limits))
            .transpose()?
            .flatten();
        self.expected(&expected)?;
        Ok(carrier)
    }

    fn retained_surface_inputs(
        &self,
        expected: SourceRef,
    ) -> Result<
        Vec<qualia_core_db::poet_host::invoke::studio_store::checkpoint::ResourceData>,
        String,
    > {
        self.expected(&expected)?;
        // Inert immutable input copying does not publish or rebuild execution.
        // The compiler separately retains its current-publication requirement.
        self.surface_resources
            .as_ref()
            .map(|r| r.retained_inputs())
            .transpose()
            .map(|v| v.unwrap_or_default())
    }

    fn inspect_surface(
        &self,
        id: &str,
        expected: SourceRef,
    ) -> Result<crate::graphical::surfaces::inspection::SurfaceInspection, String> {
        self.expected(&expected)?;
        if self.reconciliation_required || !self.execution_epoch.is_current() {
            return Err("Rebuild accepted execution before surface inspection".into());
        }
        let parts = self
            .session
            .value("poet_parts")
            .ok_or("No authored assembly")?;
        crate::graphical::surfaces::inspection::inspect(
            parts,
            id,
            expected,
            self.surface_resources.as_ref(),
            self.host.vertex_limit(),
        )
    }

    fn install_surface_resources(
        &mut self,
        resources: crate::graphical::surfaces::TextureResources,
        expected: SourceRef,
    ) -> Result<(), String> {
        self.expected(&expected)?;
        if let Some(parts) = self.session.value("poet_parts") {
            let value = Value::Record(std::collections::BTreeMap::from([(
                "parts".into(),
                parts.clone(),
            )]));
            self.preview
                .validate_with_resources(&value, Some(&resources))?;
        }
        self.surface_resources = Some(resources);
        self.execution_epoch.retire();
        self.execution_epoch = SessionEpoch::new();
        self.preview.set_execution_context(&self.execution_epoch);
        Ok(())
    }

    fn authoring_view(
        &self,
        e: SourceRef,
    ) -> Result<crate::graphical::authoring::AuthoringView, String> {
        self.inspect_authoring(e)
    }
    fn authoring_edit(
        &mut self,
        a: crate::graphical::authoring::AuthoringEdit,
        e: SourceRef,
    ) -> Result<EditorOutcome, String> {
        self.edit_assembly(a, e)
    }
    fn compiled_assets(
        &self,
        k: &str,
        e: SourceRef,
    ) -> Result<Vec<(String, qualia_core_db::render::compile_10d::CompiledAsset)>, String> {
        self.compile_authored(k, e)
    }

    fn material_edit(
        &mut self,
        source: &str,
        expected: SourceRef,
    ) -> Result<EditorOutcome, String> {
        let snapshot = self.snapshot()?;
        let id = snapshot
            .material_edit_id
            .ok_or("coherent mutable material/geometry bindings are required")?;
        let next = self
            .session
            .revision()
            .checked_add(1)
            .ok_or("material revision exhausted")?;
        super::material_edit::validate(source, next, id)?;
        self.submit(source, expected)
    }
    fn source_cells(&self, expected: SourceRef) -> Result<Vec<String>, String> {
        if expected.artifact_id != self.artifact_id {
            return Err("foreign source read".into());
        }
        expected.check_current()?;
        let (artifact, stamp) = studio_store::get_artifact_stamped(&self.artifact_id)
            .ok_or("source artifact disappeared")?;
        if stamp != expected.stamp {
            return Err("source changed before read".into());
        }
        let cells = source_cells(&artifact)?
            .into_iter()
            .map(str::to_string)
            .collect();
        expected.check_current()?;
        Ok(cells)
    }
    fn snapshot(&self) -> Result<EditorSnapshot, String> {
        let (artifact, stamp) = studio_store::get_artifact_stamped(&self.artifact_id)
            .ok_or_else(|| "source artifact disappeared".to_string())?;
        let latest_source = source_cells(&artifact)?
            .last()
            .copied()
            .unwrap_or("")
            .to_string();
        Ok(EditorSnapshot {
            artifact_id: artifact.id.clone(),
            title: artifact.title,
            source_revision: artifact.lamport_clock,
            session_revision: self.session.revision(),
            execution_epoch: self.execution_epoch.clone(),
            rebuild_supported: true,
            source: SourceRef {
                artifact_id: artifact.id,
                stamp: stamp.clone(),
            },
            latest_source,
            reconciliation_required: self.reconciliation_required || stamp != self.active_stamp,
            material_edit_id: material_edit_id(
                &self.session,
                self.reconciliation_required || stamp != self.active_stamp,
            ),
            checkpoint_supported: false,
            authorisation_stamp: None,
        })
    }

    fn check(&self, source: &str) -> Result<String, String> {
        if self.snapshot()?.reconciliation_required {
            return Err("source/session reconciliation required".into());
        }
        self.session
            .prepare(source)
            .map(|_| "Canonical Vibe check passed; no execution attempted".into())
            .map_err(|diagnostic| {
                format!(
                    "{:?} at {:?}: {}",
                    diagnostic.code, diagnostic.span, diagnostic.message
                )
            })
    }

    fn submit(&mut self, source: &str, expected: SourceRef) -> Result<EditorOutcome, String> {
        if self.reconciliation_required {
            return Err("source/session reconciliation required before further execution".into());
        }
        self.expected(&expected)?;
        if !self.execution_epoch.is_current() {
            return Err("execution epoch has retired".into());
        }
        let (layer_parts, layer_base) = if source.contains("_poet_source_layer") {
            crate::graphical::authoring::source_layers::validation_snapshot_with_budget(
                self.session.value("poet_parts"),
                self.session.value("poet_parameter_parts"),
                self.factory_limits
                    .retained_value_bytes
                    .max(64 * 1024 * 1024),
            )?
        } else {
            (None, None)
        };
        self.host.begin_submission();
        let provider = &self.preview;
        let resources = self.surface_resources.as_ref();
        let result = match self.session.submit_validated(
            source,
            &mut self.host,
            Budget::default(),
            |value| {
                crate::graphical::authoring::source_layers::validate_result(
                    value,
                    layer_parts.as_ref(),
                    layer_base.as_ref(),
                )
                .map_err(|message| {
                    vibe::Diagnostic::new(vibe::DiagCode::E701, vibe::Span::point(0), message)
                })?;
                provider
                    .validate_with_resources(value, resources)
                    .map_err(|message| {
                        vibe::Diagnostic::new(vibe::DiagCode::E701, vibe::Span::point(0), message)
                    })?;
                expected.check_current().map_err(|message| {
                    vibe::Diagnostic::new(vibe::DiagCode::E701, vibe::Span::point(0), message)
                })
            },
        ) {
            Ok(result) => result,
            Err(error) => {
                self.reconciliation_required = error.host_reconciliation_required;
                return Err(format!(
                    "{:?} at {:?}: {}; execution attempted={}; host reconciliation required={}",
                    error.diagnostic.code,
                    error.diagnostic.span,
                    error.diagnostic.message,
                    error.execution_attempted,
                    error.host_reconciliation_required
                ));
            }
        };
        // A source edit can alter recipe bindings without returning them. Inspect
        // the actual resulting session before committing editable source/history.
        // Runtime effects are not rolled back; require explicit rebuild on failure.
        if let Err(error) =
            crate::graphical::authoring::botanical_session::validate(&self.session, &self.host)
        {
            self.reconciliation_required = true;
            return Err(format!(
                "{error}; accepted source preserved; execution rebuild required"
            ));
        }
        // External effects and the session may already have advanced. Do not
        // retry or pretend rollback if another contributor won the store commit.
        let key = format!("{SOURCE_PREFIX}{:020}", result.revision);
        let committed =
            studio_store::mutate_artifact_stamped(&self.artifact_id, &expected.stamp, |artifact| {
                artifact
                    .nodes
                    .entry(key)
                    .or_default()
                    .insert("text".into(), source.to_string());
                Ok(())
            });
        let (artifact, stamp) = match committed {
            Ok(result) => result,
            Err(error) => {
                self.reconciliation_required = true;
                return Err(format!("{error}; session/effects already attempted; host reconciliation required, no retry"));
            }
        };
        self.active_stamp = stamp.clone();
        let binding = SourceRef {
            artifact_id: artifact.id,
            stamp,
        };
        let requested = binding.clone();
        let (frame, message) = match self.preview.preview_with_resources(
            binding,
            &result.value,
            self.surface_resources.as_ref(),
        ) {
            Ok(mut frame) if frame.source == requested => {
                frame.session_revision = Some(result.revision);
                frame.execution_epoch = Some(self.execution_epoch.clone());
                (
                    Some(frame),
                    "Source accepted; native engine preview rendered (review unassessed)".into(),
                )
            }
            Ok(_) => (
                None,
                "Source accepted; provider returned mismatched source; frame withheld".into(),
            ),
            Err(error) => (
                None,
                format!("Source accepted; preview unavailable: {error}"),
            ),
        };
        Ok(EditorOutcome {
            snapshot: self.snapshot()?,
            message,
            frame,
        })
    }

    fn undo(&mut self, expected: SourceRef) -> Result<EditorSnapshot, String> {
        if expected.artifact_id != self.artifact_id {
            return Err("foreign source reference".into());
        }
        let (_, stamp) = studio_store::undo_artifact_stamped(&self.artifact_id, &expected.stamp)?;
        self.active_stamp = stamp;
        // SourceSession has no reverse-effect/runtime rollback contract. Keep
        // the restored editable source and require host reconciliation/reopen.
        self.reconciliation_required = true;
        self.snapshot()
    }

    fn redo(&mut self, expected: SourceRef) -> Result<EditorSnapshot, String> {
        if expected.artifact_id != self.artifact_id {
            return Err("foreign source reference".into());
        }
        let (_, stamp) = studio_store::redo_artifact_stamped(&self.artifact_id, &expected.stamp)?;
        self.active_stamp = stamp;
        self.reconciliation_required = true;
        self.snapshot()
    }

    fn rebuild_execution(&mut self, expected: SourceRef) -> Result<EditorOutcome, String> {
        if expected.artifact_id != self.artifact_id {
            return Err("foreign rebuild source reference".into());
        }
        if !self.execution_epoch.is_current() {
            return Err("execution epoch has retired".into());
        }
        let candidate = match execution::prepare(
            expected,
            &self.host,
            &self.factory_limits,
            &self.rebuild_limits,
            self.preview.as_ref(),
            self.surface_resources.as_ref(),
        ) {
            Ok(candidate) => candidate,
            Err(error) => {
                self.reconciliation_required = true;
                return Err(error);
            }
        };
        self.activate_rebuild(candidate)
    }
}

#[cfg(test)]
#[path = "controller_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "material_admission_tests.rs"]
mod material_admission_tests;
