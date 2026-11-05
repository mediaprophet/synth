//! Trusted host-startup working profiles. HMC/source/request data never selects limits.
//! These are bounded authoring allocations, not master-quality or rights claims.
use super::{controller::RebuildLimits, preview::PreviewLimits, viewport::ViewportBudget};
use vibe::session::SessionLimits;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeAuthoringProfile {
    Legacy,
    Equipment,
}

/// Read once from the launching process's environment, independently of kernel grants.
pub fn selected() -> NativeAuthoringProfile {
    static PROFILE: std::sync::OnceLock<NativeAuthoringProfile> = std::sync::OnceLock::new();
    *PROFILE.get_or_init(|| {
        if std::env::var("POET_NATIVE_EQUIPMENT_PROFILE").as_deref() == Ok("1") {
            NativeAuthoringProfile::Equipment
        } else {
            NativeAuthoringProfile::Legacy
        }
    })
}

/// Read the named botanical replay selection only from trusted process startup.
/// Existing public profile enum and default `RebuildLimits` remain unchanged.
pub(crate) fn startup_rebuild_limits(profile: NativeAuthoringProfile) -> RebuildLimits {
    static BOTANICAL: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let enabled = *BOTANICAL.get_or_init(||
        std::env::var("POET_NATIVE_BOTANICAL_PROFILE").as_deref() != Ok("0"));
    botanical_rebuild_limits(profile, enabled)
}

fn botanical_rebuild_limits(profile: NativeAuthoringProfile, enabled: bool) -> RebuildLimits {
    let mut limits = profile.rebuild_limits();
    if enabled {
        // One junction replay scope shares the aggregate limits. Values are
        // diagnostic working caps, not guarantees about cost or asset quality.
        limits.total_geometry_work = limits.total_geometry_work.saturating_add(64_000_000);
        limits.total_kernel_dispatches = limits.total_kernel_dispatches.saturating_add(256);
    }
    limits
}
impl NativeAuthoringProfile {
    /// Core's retained-value representation allowance, not measured allocator/RSS bytes.
    pub fn session_limits(self) -> SessionLimits {
        let mut limits = SessionLimits::default();
        if self == Self::Equipment {
            limits.retained_value_bytes = 128 * 1024 * 1024;
        }
        limits
    }
    pub fn max_vertices(self) -> usize {
        match self {
            Self::Legacy => 200_000,
            Self::Equipment => 1_000_000,
        }
    }
    pub fn preview_limits(self) -> PreviewLimits {
        match self {
            Self::Legacy => PreviewLimits::default(),
            Self::Equipment => PreviewLimits {
                side: 512,
                vertices: 1_000_000,
                indices: 6_000_000,
            },
        }
    }
    pub fn viewport_budget(self) -> ViewportBudget {
        match self {
            Self::Legacy => ViewportBudget::default(),
            Self::Equipment => ViewportBudget {
                working_bytes: 256 * 1024 * 1024,
                pick_triangles: 2_000_000,
                overlay_steps: 4_000_000,
            },
        }
    }
    pub fn rebuild_limits(self) -> RebuildLimits {
        match self {
            Self::Legacy => RebuildLimits::default(),
            Self::Equipment => RebuildLimits {
                cells: 16,
                source_bytes: 1024 * 1024,
                total_steps: 2_000_000,
                total_geometry_work: 8_000_000,
                total_kernel_dispatches: 1024,
            },
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Legacy=>"Host working profile: legacy · 200k preview vertices · 512px viewport",
            Self::Equipment=>"Host working profile: equipment · 1m preview vertices · 512px viewport · 256 MiB viewport working budget",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_host_profiles_preserve_defaults_and_bound_equipment_work() {
        let legacy = NativeAuthoringProfile::Legacy;
        let equipment = NativeAuthoringProfile::Equipment;
        let session = SessionLimits::default();
        let preview = PreviewLimits::default();
        let viewport = ViewportBudget::default();
        let rebuild = RebuildLimits::default();
        assert_eq!(
            legacy.session_limits().retained_value_bytes,
            session.retained_value_bytes
        );
        assert_eq!(legacy.preview_limits().vertices, preview.vertices);
        assert_eq!(legacy.preview_limits().indices, preview.indices);
        assert_eq!(
            legacy.viewport_budget().working_bytes,
            viewport.working_bytes
        );
        assert_eq!(
            legacy.rebuild_limits().total_kernel_dispatches,
            rebuild.total_kernel_dispatches
        );
        assert_eq!(legacy.max_vertices(), 200_000);
        assert_eq!(
            equipment.session_limits().retained_value_bytes,
            128 * 1024 * 1024
        );
        assert_eq!(equipment.max_vertices(), 1_000_000);
        let preview = equipment.preview_limits();
        assert_eq!(
            (preview.side, preview.vertices, preview.indices),
            (512, 1_000_000, 6_000_000)
        );
        let viewport = equipment.viewport_budget();
        assert_eq!(
            (
                viewport.working_bytes,
                viewport.pick_triangles,
                viewport.overlay_steps
            ),
            (256 * 1024 * 1024, 2_000_000, 4_000_000)
        );
        let rebuild = equipment.rebuild_limits();
        assert_eq!(
            (
                rebuild.cells,
                rebuild.source_bytes,
                rebuild.total_steps,
                rebuild.total_geometry_work,
                rebuild.total_kernel_dispatches
            ),
            (16, 1024 * 1024, 2_000_000, 8_000_000, 1024)
        );
        // This comparison needs no process environment mutation or authority objects.
        assert_ne!(legacy.label(), equipment.label());
        assert_eq!(botanical_rebuild_limits(legacy, false).total_geometry_work,
            legacy.rebuild_limits().total_geometry_work);
        assert_eq!(botanical_rebuild_limits(legacy, true).total_geometry_work, 66_000_000);
        assert_eq!(botanical_rebuild_limits(legacy, true).total_kernel_dispatches, 512);
        assert_eq!(botanical_rebuild_limits(equipment, true).total_geometry_work, 72_000_000);
    }
}
