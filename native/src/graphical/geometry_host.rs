//! Explicit native geometry invoke profile over the actual Qualia host.
use qualia_core_db::poet_host::PoetSnapshot;
use vibe::{DiagCode, Diagnostic, Host, Span, Value};
#[path = "revolve_host.rs"]
pub(crate) mod revolve_host;
#[path = "spatial_host.rs"]
mod spatial_host;
#[path = "turtle_host.rs"]
mod turtle_host;
#[cfg(test)]
#[path = "turtle_host_tests.rs"]
mod turtle_host_tests;
#[cfg(test)]
#[path = "sheet_range_host_tests.rs"]
mod sheet_range_host_tests;

#[derive(Clone, Copy, Debug)]
pub struct GeometryHostLimits {
    pub max_vertices: usize,
    pub submission_work: usize,
    pub dispatches: usize,
    pub scratch_bytes: usize,
    pub profile_points: usize,
}

pub struct GeometryHost {
    inner: PoetSnapshot,
    max_vertices: usize,
    grants: Vec<String>,
    remaining_work: usize,
    remaining_dispatches: usize,
    limits: GeometryHostLimits,
}
impl Default for GeometryHost {
    fn default() -> Self {
        Self::new(Vec::new(), 200_000)
    }
}
impl GeometryHost {
    /// Host-selected working set, not a master-quality ceiling or script grant.
    pub fn new(grants: Vec<String>, max_vertices: usize) -> Self {
        Self::configured(
            grants,
            GeometryHostLimits {
                max_vertices,
                submission_work: max_vertices.saturating_mul(2),
                dispatches: 256,
                scratch_bytes: 256 * 1024 * 1024,
                profile_points: 4096,
            },
        )
    }
    /// Explicit native host budgets; authored source cannot raise these limits.
    pub fn with_limits(grants: Vec<String>, limits: GeometryHostLimits) -> Result<Self, String> {
        if limits.max_vertices == 0
            || limits.submission_work == 0
            || limits.dispatches == 0
            || limits.dispatches > 65536
            || limits.scratch_bytes == 0
            || !(3..=65536).contains(&limits.profile_points)
        {
            return Err("native geometry working limits are invalid".into());
        }
        Ok(Self::configured(grants, limits))
    }
    fn configured(grants: Vec<String>, limits: GeometryHostLimits) -> Self {
        Self {
            inner: PoetSnapshot::default(),
            max_vertices: limits.max_vertices,
            grants,
            remaining_work: limits.submission_work,
            remaining_dispatches: limits.dispatches,
            limits,
        }
    }
    /// Typed native edits require the same actual host capabilities as script effects.
    /// These bounded cold allocations are selected by the host, never by source.
    pub(crate) fn botanical_limits(
        &self,
        has_uv0: bool,
    ) -> Result<crate::graphical::authoring::botanical::BotanicalLimits, String> {
        for required in [
            "capability.invoke",
            "ComputationalGeometry.space_colonize_3d",
            "ComputationalGeometry.pipe_model_radii",
            "ComputationalGeometry.extrude_profile",
        ] {
            if !self.grants.iter().any(|grant| grant == required) {
                return Err(format!(
                    "Botanical operation requires actual host grant {required}"
                ));
            }
        }
        if self.limits.dispatches < 9 || self.limits.scratch_bytes < 4096 {
            return Err("Botanical kernel dispatch/scratch budget unavailable".into());
        }
        let mut limits = crate::graphical::authoring::botanical::BotanicalLimits::default();
        // The base reserve covers the existing three bounded geometry passes.
        // UV-bearing source additionally reserves three two-scalar-per-vertex
        // validations and one three-index-per-triangle correspondence scan;
        // triangles are bounded to twice the vertices below. These are
        // conservative abstract work units, not measured performance.
        let work_per_vertex = if has_uv0 { 21 } else { 9 };
        limits.max_vertices = limits
            .max_vertices
            .min(self.max_vertices)
            .min(self.limits.submission_work / work_per_vertex)
            .min(self.limits.scratch_bytes / 512);
        limits.max_triangles = limits
            .max_triangles
            .min(limits.max_vertices.saturating_mul(2));
        limits.max_proposal_nodes = limits
            .max_proposal_nodes
            .min(self.limits.scratch_bytes / 256);
        limits.max_organs = limits.max_organs.min(limits.max_proposal_nodes);
        limits.max_attractors = limits.max_attractors.min(self.limits.scratch_bytes / 128);
        // Each iteration scans bounded node/attractor pairs. The profile also
        // bounds cold solver work independently of final visible vertex count.
        limits.max_iterations = limits.max_iterations.min(16);
        limits.max_proposal_nodes = limits.max_proposal_nodes.min(512);
        limits.max_attractors = limits.max_attractors.min(256);
        Ok(limits)
    }
    pub(crate) fn vertex_limit(&self) -> usize {
        self.max_vertices
    }
    pub(crate) fn actual_grants(&self) -> &[String] {
        &self.grants
    }
    pub(crate) fn begin_submission(&mut self) {
        self.remaining_work = self.limits.submission_work;
        self.remaining_dispatches = self.limits.dispatches;
    }
    pub(crate) fn for_rebuild(&self, total_work: usize, dispatches: usize) -> Self {
        let mut next = Self::configured(self.grants.clone(), self.limits);
        next.remaining_work = total_work;
        next.remaining_dispatches = dispatches;
        next
    }
    fn admit(&mut self, id: &str, args: &Value, span: Span) -> Result<(), Diagnostic> {
        let Value::Record(record) = args else {
            return Err(denied(span));
        };
        let length = |key: &str| match record.get(key) {
            Some(Value::List(values)) => Some(values.len()),
            _ => None,
        };
        let vertices = match id {
            "ComputationalGeometry.extrude_profile" => length("axis")
                .zip(length("section"))
                .and_then(|(axis, section)| axis.checked_mul(section))
                .and_then(|count| count.checked_add(2)),
            "ComputationalGeometry.swung_surface" => {
                let m = match record.get("profiles") {
                    Some(Value::List(rows)) if rows.len() <= 256 => match rows.first() {
                        Some(Value::List(row)) => Some(row.len() / 2),
                        _ => None,
                    },
                    _ => None,
                };
                let slices = match record.get("slices") {
                    Some(Value::U64(v)) => usize::try_from(*v).ok(),
                    Some(Value::I64(v)) if *v >= 0 => usize::try_from(*v).ok(),
                    _ => None,
                };
                m.zip(slices).and_then(|(m, slices)| m.checked_mul(slices))
            }
            _ => None,
        };
        if self.remaining_dispatches == 0
            || vertices.is_none_or(|count| {
                count == 0 || count > self.max_vertices || count > self.remaining_work
            })
        {
            return Err(Diagnostic::new(
                DiagCode::E400,
                span,
                "geometry exceeds host-selected preview working set",
            ));
        }
        self.remaining_work -= vertices.expect("geometry work admitted");
        self.remaining_dispatches -= 1;
        Ok(())
    }
}

fn denied(span: Span) -> Diagnostic {
    Diagnostic::new(
        DiagCode::E300,
        span,
        "operation is outside the admitted native geometry preview profile",
    )
}

impl Host for GeometryHost {
    fn graph_query(&mut self, _: &[Value], _: u64, span: Span) -> Result<Value, Diagnostic> {
        Err(denied(span))
    }
    fn graph_stage(&mut self, _: &Value, span: Span) -> Result<Value, Diagnostic> {
        Err(denied(span))
    }
    fn graph_commit(&mut self, span: Span) -> Result<Value, Diagnostic> {
        Err(denied(span))
    }
    fn aura_validate(&mut self, _: &Value, _: &Value, span: Span) -> Result<Value, Diagnostic> {
        Err(denied(span))
    }
    fn pulse_publish(&mut self, _: &str, _: &Value, span: Span) -> Result<Value, Diagnostic> {
        Err(denied(span))
    }
    fn capability_invoke(
        &mut self,
        id: &str,
        args: &Value,
        span: Span,
    ) -> Result<Value, Diagnostic> {
        // The canonical checker separately checks the invocation-style grant.
        // Runtime authority always checks the actual target, including dynamic IDs.
        if !self.grants.iter().any(|grant| grant == id) {
            return Err(denied(span));
        }
        match id {
            "Sheet.stats" | "Sheet.sum_range" => {
                // Retain QualiaDB's flat stats and ragged grid semantics. Bound
                // the actual supplied input, not the potentially enormous range.
                let grid = match args { Value::Record(fields) => fields.get("grid").ok_or_else(|| denied(span))?, _ => args };
                let Value::List(rows) = grid else { return Err(denied(span)); };
                if rows.len() > 4096 { return Err(denied(span)); }
                let mut work = rows.len();
                for item in rows {
                    if let Value::List(cells) = item {
                        work = work.checked_add(cells.len()).ok_or_else(|| denied(span))?;
                    } else {
                        // QualiaDB decides whether the flat stats item is numeric.
                        work = work.checked_add(1).ok_or_else(|| denied(span))?;
                    }
                    if work > 65_536 { return Err(denied(span)); }
                }
                if let Value::Record(fields) = args {
                    let spec = fields.get("range").or_else(|| fields.get("cells"));
                    if matches!(spec, Some(Value::String(s)) if s.len() > 128) { return Err(denied(span)); }
                }
                let work = work.max(1);
                if self.remaining_dispatches == 0 || work > self.remaining_work {
                    return Err(Diagnostic::new(DiagCode::E400, span, "native sheet range dispatch/work budget exhausted"));
                }
                self.remaining_work -= work;
                self.remaining_dispatches -= 1;
                self.inner.capability_invoke(id, args, span)
            }
            "Sheet.eval_formula" => {
                // Host checks input shape and reserves a conservative fixed work
                // budget before dispatch. The QualiaDB operator enforces the
                // formula's 64-node/16-depth AST and function/reference rules.
                let Value::Record(fields) = args else { return Err(denied(span)); };
                let Some(Value::String(formula)) = fields.get("formula") else { return Err(denied(span)); };
                if !formula.starts_with('=') || formula.len() > 256 { return Err(denied(span)); }
                if let Some(grid) = fields.get("grid") {
                    let Value::List(rows) = grid else { return Err(denied(span)); };
                    if rows.len() > 32 { return Err(denied(span)); }
                    for row in rows {
                        let Value::List(cells) = row else { return Err(denied(span)); };
                        if cells.len() > 32 { return Err(denied(span)); }
                    }
                }
                // Up to 64 AST nodes may each address a 32x32 range; add
                // room for grid loading and scalar evaluation independently.
                const FORMULA_WORK: usize = 131_072;
                if self.remaining_dispatches == 0 || self.remaining_work < FORMULA_WORK {
                    return Err(Diagnostic::new(DiagCode::E400, span, "native sheet formula dispatch/work budget exhausted"));
                }
                self.remaining_work -= FORMULA_WORK;
                self.remaining_dispatches -= 1;
                self.inner.capability_invoke(id, args, span)
            }
            "Sheet.set_cell" => {
                // Bound replay work before forwarding to the QualiaDB owner.
                let Value::Record(fields) = args else { return Err(denied(span)); };
                let Some(Value::List(rows)) = fields.get("grid") else { return Err(denied(span)); };
                if rows.is_empty() || rows.len() > 32 { return Err(denied(span)); }
                let mut work = 0usize;
                for row in rows {
                    let Value::List(cells) = row else { return Err(denied(span)); };
                    if cells.is_empty() || cells.len() > 32 { return Err(denied(span)); }
                    work = work.checked_add(cells.len()).ok_or_else(|| denied(span))?;
                }
                if self.remaining_dispatches == 0 || work > self.remaining_work {
                    return Err(Diagnostic::new(DiagCode::E400, span, "native sheet dispatch/work budget exhausted"));
                }
                self.remaining_work -= work;
                self.remaining_dispatches -= 1;
                self.inner.capability_invoke(id, args, span)
            }
            "ComputationalGeometry.turtle_3d" => {
                if self.remaining_dispatches == 0 {
                    return Err(turtle_host::bad(
                        span,
                        "native turtle dispatch budget exhausted",
                    ));
                }
                let work = turtle_host::admit(args, self.limits, self.remaining_work, span)?;
                self.remaining_work -= work;
                self.remaining_dispatches -= 1;
                self.inner.capability_invoke(id, args, span)
            }
            "ComputationalGeometry.profile_sweep"
            | "ComputationalGeometry.revolve_profile"
            | "ComputationalGeometry.uv_sphere"
            | "ComputationalGeometry.torus" => {
                if self.remaining_dispatches == 0 || self.remaining_work == 0 {
                    return Err(Diagnostic::new(
                        DiagCode::E400,
                        span,
                        "native geometry dispatch/work budget exhausted",
                    ));
                }
                let (value, count) = spatial_host::invoke(
                    id,
                    args,
                    span,
                    poet_spatial_authoring::GeometryLimits {
                        vertices: self.max_vertices.min(self.remaining_work),
                        triangles: self.max_vertices.min(self.remaining_work).saturating_mul(4),
                        scratch_bytes: self.limits.scratch_bytes,
                        profile_points: self.limits.profile_points,
                    },
                )?;
                self.remaining_work -= count;
                self.remaining_dispatches -= 1;
                Ok(value)
            }
            "ComputationalGeometry.extrude_profile" | "ComputationalGeometry.swung_surface" => {
                self.admit(id, args, span)?;
                self.inner.capability_invoke(id, args, span)
            }
            _ => Err(denied(span)),
        }
    }
}

/// Source-only diagnostic recipe, not a production organism or accepted model.
/// It uses the real editable section/path operator, never primitive counts.
pub const ORGANIC_SURFACE_SOURCE: &str = r#"requires [ capability("capability.invoke"), capability("ComputationalGeometry.extrude_profile") ];
capability.invoke("ComputationalGeometry.extrude_profile", {
  axis: [[0.0, 0.0, -1.0], [0.12, 0.06, -0.4], [0.18, 0.02, 0.3], [0.1, 0.0, 1.0]],
  section: [[-0.20, -0.10], [0.18, -0.12], [0.24, 0.05], [0.02, 0.20], [-0.18, 0.10]],
  closed: true, caps: true
});"#;

#[cfg(test)]
mod tests {
    use super::*;
    use vibe::session::{SessionLimits, SourceSession};
    #[test]
    fn actual_operation_grants_guard_dynamic_targets_at_runtime() {
        let grants = vec![
            "capability.invoke".into(),
            "ComputationalGeometry.extrude_profile".into(),
        ];
        let mut session = SourceSession::new(grants.clone(), SessionLimits::default());
        let mut host = GeometryHost::new(grants, 200_000);
        let source = r#"requires [capability("capability.invoke")];
            let op = "ComputationalGeometry.swung_surface";
            capability.invoke(op, {profiles:[[1.0,0.0,1.0,1.0],[1.1,0.0,1.1,1.0]], angles:[0.0,3.14], slices:8});"#;
        let error = session
            .submit(source, &mut host, vibe::Budget::default())
            .unwrap_err();
        assert_eq!(error.diagnostic.code, DiagCode::E300);
        assert!(error
            .diagnostic
            .message
            .contains("outside the admitted native geometry preview profile"));
        assert!(error.execution_attempted);
        assert_eq!(session.revision(), 0);
    }
    #[test]
    fn granted_source_dispatches_real_native_geometry_kernel() {
        let grants = vec![
            "capability.invoke".into(),
            "ComputationalGeometry.extrude_profile".into(),
        ];
        let mut session = SourceSession::new(grants.clone(), SessionLimits::default());
        let mut host = GeometryHost::new(grants, 200_000);
        let result = session
            .submit(ORGANIC_SURFACE_SOURCE, &mut host, vibe::Budget::default())
            .unwrap();
        let Value::Record(mesh) = result.value else {
            panic!("actual geometry result record")
        };
        assert!(
            matches!(mesh.get("vertices"), Some(Value::List(vertices)) if !vertices.is_empty())
        );
        assert!(
            matches!(mesh.get("triangles"), Some(Value::List(triangles)) if !triangles.is_empty())
        );
        assert_eq!(result.revision, 1);
    }
    #[test]
    fn profile_grants_and_host_dispatch_budgets_are_current_at_execution() {
        let source = r#"requires [capability("capability.invoke"), capability("ComputationalGeometry.profile_sweep")];
            capability.invoke("ComputationalGeometry.profile_sweep", {profile:{kind:"open_channel",width:0.4,height:0.2,wall:0.02,corner_radius:0.01,segments:4},length:1.0,caps:true});"#;
        let grants = vec![
            "capability.invoke".into(),
            "ComputationalGeometry.profile_sweep".into(),
        ];
        let mut host = GeometryHost::with_limits(
            grants.clone(),
            GeometryHostLimits {
                max_vertices: 10000,
                submission_work: 20000,
                dispatches: 1,
                scratch_bytes: 16 * 1024 * 1024,
                profile_points: 1024,
            },
        )
        .unwrap();
        let mut session = SourceSession::new(grants, SessionLimits::default());
        let result = session
            .submit(source, &mut host, vibe::Budget::default())
            .unwrap();
        assert!(
            matches!(result.value, Value::Record(ref mesh) if matches!(mesh.get("vertices"),Some(Value::List(vertices)) if vertices.len()>8))
        );
        assert!(session
            .submit(source, &mut host, vibe::Budget::default())
            .is_err());
        host.begin_submission();
        assert!(session
            .submit(source, &mut host, vibe::Budget::default())
            .is_ok());
        let mut denied_host = GeometryHost::new(vec!["capability.invoke".into()], 10000);
        let dynamic = r#"requires [capability("capability.invoke")];let operation="ComputationalGeometry.profile_sweep";capability.invoke(operation,{profile:{kind:"circle",radius:0.1,segments:16},length:1.0});"#;
        let mut held =
            SourceSession::new(vec!["capability.invoke".into()], SessionLimits::default());
        assert_eq!(
            held.submit(dynamic, &mut denied_host, vibe::Budget::default())
                .unwrap_err()
                .diagnostic
                .code,
            DiagCode::E300
        );
    }
}

#[cfg(test)]
mod botanical_budget_tests {
    use super::*;
    fn host(dispatches:usize, work:usize) -> GeometryHost {
        GeometryHost::with_limits(["capability.invoke", "ComputationalGeometry.space_colonize_3d",
            "ComputationalGeometry.pipe_model_radii", "ComputationalGeometry.extrude_profile"].into_iter().map(str::to_owned).collect(),
            GeometryHostLimits { max_vertices:10_000, submission_work:work, dispatches,
                scratch_bytes:1024*1024, profile_points:1024 }).unwrap()
    }
    #[test]
    fn botanical_host_reserves_projection_edit_and_revalidation_work() {
        assert!(host(8,900).botanical_limits(false).is_err());
        let limits=host(9,900).botanical_limits(false).unwrap();
        assert!(limits.max_vertices*9<=900);
        let uv_limits=host(9,900).botanical_limits(true).unwrap();
        assert!(uv_limits.max_vertices*21<=900);
        assert!(uv_limits.max_vertices < limits.max_vertices);
        assert!(limits.max_proposal_nodes<=512 && limits.max_attractors<=256 && limits.max_iterations<=16);
        let mut denied=host(9,900);
        denied.grants.retain(|g|g!="ComputationalGeometry.pipe_model_radii");
        assert!(denied.botanical_limits(false).is_err());
    }
}

#[cfg(test)]
#[path = "sheet_formula_session_tests.rs"]
mod sheet_formula_session_tests;

#[cfg(test)]
#[path = "sheet_range_session_tests.rs"]
mod sheet_range_session_tests;
