//! Explicit replay of the exact native read-only geometry profile only.
use super::super::{geometry_host::GeometryHost, preview::SourceRef};
use super::source_cells;
use qualia_core_db::poet_host::invoke::studio_store;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use vibe::{
    session::{SessionLimits, SourceSession},
    Budget,
};

#[derive(Clone, Debug)]
pub struct SessionEpoch(Arc<AtomicBool>);
impl SessionEpoch {
    pub(crate) fn new() -> Self {
        Self(Arc::new(AtomicBool::new(true)))
    }
    pub fn is_current(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
    pub(super) fn retire(&self) {
        self.0.store(false, Ordering::Release);
    }
}
impl PartialEq for SessionEpoch {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for SessionEpoch {}

#[derive(Clone)]
pub struct RebuildLimits {
    pub cells: usize,
    pub source_bytes: usize,
    pub total_steps: u64,
    /// Aggregate conservative output-vertex work over every replayed builder.
    pub total_geometry_work: usize,
    pub total_kernel_dispatches: usize,
}
impl Default for RebuildLimits {
    fn default() -> Self {
        Self {
            cells: 64,
            source_bytes: 1024 * 1024,
            total_steps: 1_000_000,
            total_geometry_work: 2_000_000,
            total_kernel_dispatches: 256,
        }
    }
}

pub(super) struct PreparedRebuild {
    pub source: SourceRef,
    pub artifact: studio_store::StudioArtifact,
    pub session: SourceSession,
    pub host: GeometryHost,
}

/// Source was already accepted; replay here is a separate explicit native act.
/// No GPU, preview producer, graph/pulse/network or generic Host is called.
pub(super) fn prepare(
    source: SourceRef,
    profile: &GeometryHost,
    limits: &SessionLimits,
    bound: &RebuildLimits,
    validator: &dyn super::PreviewProvider,
    resources: Option<&crate::graphical::surfaces::TextureResources>,
) -> Result<PreparedRebuild, String> {
    source.check_current()?;
    let (artifact, stamp) = studio_store::get_artifact_stamped(&source.artifact_id)
        .ok_or_else(|| "rebuild source disappeared".to_string())?;
    if stamp != source.stamp {
        return Err("rebuild source changed before capture".into());
    }
    let cells = source_cells(&artifact)?;
    let bytes = cells
        .iter()
        .fold(0usize, |total, cell| total.saturating_add(cell.len()));
    if cells.len() > bound.cells
        || bytes > bound.source_bytes
        || bound.total_steps == 0
        || bound.total_geometry_work == 0
        || bound.total_kernel_dispatches == 0
    {
        return Err(
            "explicit execution rebuild exceeds host-selected total work/source budget".into(),
        );
    }
    let count = cells.len().max(1) as u64;
    let per_cell_steps = bound.total_steps / count;
    if per_cell_steps == 0 {
        return Err("rebuild total instruction budget is too small".into());
    }
    let mut session = SourceSession::new(profile.actual_grants().to_vec(), limits.clone());
    let mut host = profile.for_rebuild(bound.total_geometry_work, bound.total_kernel_dispatches);
    // One shared replay ledger for every cell and tentative validation. This
    // local host is dropped on errors; on success end the scope before return.
    host.begin_junction_rebuild()?;
    for cell in cells {
        source.check_current()?;
        let had_junction = session.value("poet_botanical_junction")
            .is_some_and(crate::graphical::authoring::junction_session::is_junction_binding);
        let (layer_parts, layer_base) = if cell.contains("_poet_source_layer") {
            crate::graphical::authoring::source_layers::validation_snapshot_with_budget(
                session.value("poet_parts"),
                session.value("poet_parameter_parts"),
                limits.retained_value_bytes.max(64 * 1024 * 1024),
            )?
        } else {
            (None, None)
        };
        session
            .submit_validated_bindings(
                cell,
                &mut host,
                Budget {
                    steps_left: per_cell_steps,
                    workspace_left: (limits.retained_value_bytes as u64).min(vibe::SENTINEL_BYTES),
                },
                |value, bindings, _actual_host| {
                    crate::graphical::authoring::timeline_associations::validate_pair(bindings.value("poet_timeline"),bindings.value("poet_timeline_media"))
                        .map_err(|message|vibe::Diagnostic::new(vibe::DiagCode::E701,vibe::Span::point(0),message))?;
                    crate::graphical::authoring::sound_field::validate_pair(bindings.value("poet_timeline"), bindings.value("poet_sound_field"))
                        .map_err(|message|vibe::Diagnostic::new(vibe::DiagCode::E701,vibe::Span::point(0),message))?;
                    crate::graphical::authoring::source_layers::validate_result(
                        value,
                        layer_parts.as_ref(),
                        layer_base.as_ref(),
                    )
                    .map_err(|message| {
                        vibe::Diagnostic::new(vibe::DiagCode::E701, vibe::Span::point(0), message)
                    })?;
                    validator
                        .validate_with_resources(value, resources)
                        .map_err(|message| {
                            vibe::Diagnostic::new(
                                vibe::DiagCode::E701,
                                vibe::Span::point(0),
                                message,
                            )
                        })?;
                    if had_junction && !bindings.value("poet_botanical_junction")
                        .is_some_and(crate::graphical::authoring::junction_session::is_junction_binding) {
                        return Err(vibe::Diagnostic::new(vibe::DiagCode::E701,
                            vibe::Span::point(0), "Junction binding removal requires explicit migration"));
                    }
                    Ok(())
                },
            )
            .map_err(|error| {
                format!(
                    "rebuild held: {:?} at {:?}: {}",
                    error.diagnostic.code, error.diagnostic.span, error.diagnostic.message
                )
            })?;
    }
    // Reconstruct bounded inline source first; qualify the final junction once.
    // This does not certify each historical intermediate geometry.
    crate::graphical::authoring::junction_session::validate_tentative(session.bindings(), &mut host)?;
    crate::graphical::authoring::botanical_session::validate(&session, &host)?;
    source.check_current()?;
    host.end_junction_operation();
    Ok(PreparedRebuild {
        source,
        artifact,
        session,
        host,
    })
}
