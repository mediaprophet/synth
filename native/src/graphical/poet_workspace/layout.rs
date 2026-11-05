//! Unified Tool-Chest and Manifolds workspace layout contract.
//!
//! Reconciles native Aether/webizen-desktop Poet workspace structures with
//! canonical WASM Tool-Chest and Manifolds layout conventions, container IDs,
//! honesty badges, and NativeIntentBus action routing.

use super::core::{ActionType, ContextRef, DockPosition, SeedContainer, TargetIdentifier, VibeScriptPayload};
use super::model::WorkspaceProjection;
use serde::{Deserialize, Serialize};

/// Canonical honesty state badge reflecting service readiness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HonestyBadge {
    /// Fully qualified and verified live native service.
    Verified,
    /// Partially mounted service with bounded or transitional capabilities.
    Partial,
    /// Wait-honest placeholder; unmounted service held without missing theatre.
    Held,
}

impl HonestyBadge {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Partial => "partial",
            Self::Held => "held",
        }
    }
}

/// The 5 canonical top-level Tool-Chest sections shared between Native and WASM.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolChestSection {
    Constructs,
    Manifolds,
    Containers,
    Inspector,
    VibeConsole,
}

impl ToolChestSection {
    pub const ALL: [Self; 5] = [
        Self::Constructs,
        Self::Manifolds,
        Self::Containers,
        Self::Inspector,
        Self::VibeConsole,
    ];

    pub fn section_id(&self) -> &'static str {
        match self {
            Self::Constructs => "toolchest-constructs",
            Self::Manifolds => "toolchest-manifolds",
            Self::Containers => "toolchest-containers",
            Self::Inspector => "toolchest-inspector",
            Self::VibeConsole => "toolchest-vibe-console",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Constructs => "Constructs Shelf",
            Self::Manifolds => "Manifolds Pager",
            Self::Containers => "Container Layout",
            Self::Inspector => "Spatial & Part Context Inspector",
            Self::VibeConsole => "Vibe Console",
        }
    }

    pub fn honesty(&self) -> HonestyBadge {
        match self {
            Self::Constructs => HonestyBadge::Partial,
            Self::Manifolds => HonestyBadge::Verified,
            Self::Containers => HonestyBadge::Partial,
            Self::Inspector => HonestyBadge::Partial,
            Self::VibeConsole => HonestyBadge::Verified,
        }
    }

    pub fn canonical_container_id(&self, artifact_id: &str) -> String {
        match self {
            Self::Inspector => format!("studio-inspector:{artifact_id}"),
            _ => format!("{}:{artifact_id}", self.section_id()),
        }
    }

    pub fn default_dock_region(&self) -> DockRegionSpec {
        match self {
            Self::Constructs => DockRegionSpec {
                region_id: "dock-top-shelf".into(),
                dock: DockPosition::Top,
                width: 1440,
                height: 48,
                visible: true,
                title: "Constructs Shelf".into(),
            },
            Self::Manifolds => DockRegionSpec {
                region_id: "dock-top-pager".into(),
                dock: DockPosition::Top,
                width: 1440,
                height: 40,
                visible: true,
                title: "Manifolds Pager".into(),
            },
            Self::Containers => DockRegionSpec {
                region_id: "dock-center-stage".into(),
                dock: DockPosition::Float,
                width: 1280,
                height: 720,
                visible: true,
                title: "Workspace Manifold Canvas".into(),
            },
            Self::Inspector => DockRegionSpec {
                region_id: "dock-right-inspector".into(),
                dock: DockPosition::Right,
                width: 360,
                height: 720,
                visible: true,
                title: "Inspector".into(),
            },
            Self::VibeConsole => DockRegionSpec {
                region_id: "dock-bottom-console".into(),
                dock: DockPosition::Bottom,
                width: 1440,
                height: 180,
                visible: true,
                title: "Vibe Console".into(),
            },
        }
    }

    pub fn emit_payload(&self, artifact_id: &str) -> VibeScriptPayload<serde_json::Value> {
        let (action, params) = match self {
            Self::Constructs | Self::Inspector => (
                ActionType::Query,
                serde_json::json!({ "command": "authoring.inspect" }),
            ),
            Self::Manifolds | Self::Containers => (
                ActionType::Query,
                serde_json::json!({ "command": "source.refresh" }),
            ),
            Self::VibeConsole => (
                ActionType::Query,
                serde_json::json!({ "command": "source.check", "source": "1;" }),
            ),
        };
        let mut payload = VibeScriptPayload::new(
            action,
            TargetIdentifier::component_id(artifact_id),
            params,
        );
        payload.context = ContextRef::Iri("https://qualiadb.org/schema/vibe#".into());
        payload
    }
}

/// The 8 canonical Manifold Lens tabs available in Poet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManifoldLensTab {
    Studio,
    Spatial3D,
    Sheet,
    Document,
    Deck,
    Timeline,
    Audio,
    Governance,
}

impl ManifoldLensTab {
    pub const ALL: [Self; 8] = [
        Self::Studio,
        Self::Spatial3D,
        Self::Sheet,
        Self::Document,
        Self::Deck,
        Self::Timeline,
        Self::Audio,
        Self::Governance,
    ];

    pub fn tab_id(&self) -> &'static str {
        match self {
            Self::Studio => "studio-bay",
            Self::Spatial3D => "spatial-3d",
            Self::Sheet => "sheet",
            Self::Document => "document",
            Self::Deck => "deck",
            Self::Timeline => "timeline",
            Self::Audio => "audio",
            Self::Governance => "governance",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Studio => "Studio",
            Self::Spatial3D => "Spatial 3D",
            Self::Sheet => "Sheet",
            Self::Document => "Document",
            Self::Deck => "Presentation Deck",
            Self::Timeline => "Timeline",
            Self::Audio => "Audio Session",
            Self::Governance => "Governance & Rights",
        }
    }

    pub fn honesty(&self) -> HonestyBadge {
        match self {
            Self::Studio => HonestyBadge::Partial,
            Self::Spatial3D => HonestyBadge::Partial,
            Self::Sheet => HonestyBadge::Partial,
            Self::Document => HonestyBadge::Held,
            Self::Deck => HonestyBadge::Held,
            Self::Timeline => HonestyBadge::Held,
            Self::Audio => HonestyBadge::Held,
            Self::Governance => HonestyBadge::Held,
        }
    }

    pub fn canonical_container_id(&self, artifact_id: &str) -> String {
        match self {
            Self::Studio => format!("studio-source-0:{artifact_id}"),
            Self::Audio => format!("studio-source-2:{artifact_id}"),
            _ => format!("{}:{artifact_id}", self.tab_id()),
        }
    }

    pub fn to_seed_container(&self, artifact_id: &str) -> SeedContainer {
        let (w, h) = (1280.0, 720.0);
        let mut container = SeedContainer::new(
            self.tab_id(),
            self.label(),
            60.0,
            40.0,
            w,
            h,
        );
        container.id = self.canonical_container_id(artifact_id);
        container.honesty = self.honesty().as_str().into();
        container.semantic_uri = artifact_id.into();
        container
    }

    pub fn emit_payload(&self, artifact_id: &str) -> VibeScriptPayload<serde_json::Value> {
        let params = match self {
            Self::Studio | Self::Spatial3D => serde_json::json!({ "command": "authoring.inspect" }),
            Self::Sheet => serde_json::json!({
                "command": "source.check",
                "source": "let mut poet_sheet=[[1,2],[3,4]]; poet_sheet;"
            }),
            Self::Document => serde_json::json!({
                "command": "source.check",
                "source": "// document lens\n"
            }),
            Self::Deck => serde_json::json!({
                "command": "source.check",
                "source": "// presentation deck\n"
            }),
            Self::Timeline => serde_json::json!({
                "command": "source.check",
                "source": "// timeline track\n"
            }),
            Self::Audio => serde_json::json!({
                "command": "source.check",
                "source": "// audio session\n"
            }),
            Self::Governance => serde_json::json!({ "command": "source.refresh" }),
        };
        let mut payload = VibeScriptPayload::new(
            ActionType::Query,
            TargetIdentifier::component_id(artifact_id),
            params,
        );
        payload.context = ContextRef::Iri("https://qualiadb.org/schema/vibe#".into());
        payload
    }
}

/// Dock region specification for multi-pane workspace layout.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockRegionSpec {
    pub region_id: String,
    pub dock: DockPosition,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
    pub title: String,
}

/// Complete unified workspace layout configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct UnifiedWorkspaceLayout {
    pub artifact_id: String,
    pub active_section: ToolChestSection,
    pub active_tab: ManifoldLensTab,
    pub dock_regions: Vec<DockRegionSpec>,
}

impl UnifiedWorkspaceLayout {
    pub fn new(artifact_id: &str) -> Result<Self, String> {
        if artifact_id.is_empty() || artifact_id.len() > 4096 || artifact_id.as_bytes().contains(&0) {
            return Err("Poet workspace requires a bounded actual source component identifier".into());
        }
        let dock_regions = ToolChestSection::ALL
            .iter()
            .map(|s| s.default_dock_region())
            .collect();
        Ok(Self {
            artifact_id: artifact_id.to_string(),
            active_section: ToolChestSection::Containers,
            active_tab: ManifoldLensTab::Studio,
            dock_regions,
        })
    }

    pub fn to_workspace_projection(&self) -> Result<WorkspaceProjection, String> {
        let mut proj = WorkspaceProjection::for_source(&self.artifact_id)?;
        self.reconcile_with_projection(&mut proj);
        Ok(proj)
    }

    pub fn reconcile_with_projection(&self, projection: &mut WorkspaceProjection) {
        let tab_container = self.active_tab.to_seed_container(&self.artifact_id);
        if let Some(existing) = projection
            .manifold
            .containers
            .iter_mut()
            .find(|c| c.id == tab_container.id)
        {
            existing.honesty = self.active_tab.honesty().as_str().into();
            existing.semantic_uri = self.artifact_id.clone();
        } else {
            projection.manifold.containers.push(tab_container);
        }
    }

    pub fn canonical_containers_for_tab(&self, tab: ManifoldLensTab) -> Vec<SeedContainer> {
        vec![tab.to_seed_container(&self.artifact_id)]
    }

    pub fn emit_active_intent(&self) -> VibeScriptPayload<serde_json::Value> {
        self.active_tab.emit_payload(&self.artifact_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_sections_canonical_container_ids_and_payloads() {
        let artifact = "source/canonical-test";
        for section in ToolChestSection::ALL {
            let cid = section.canonical_container_id(artifact);
            assert!(cid.contains(artifact));
            assert!(!cid.is_empty());

            let payload = section.emit_payload(artifact);
            assert_eq!(payload.target_identifier.value, artifact);
            assert!(matches!(
                payload.context,
                ContextRef::Iri(ref iri) if iri == "https://qualiadb.org/schema/vibe#"
            ));
            assert!(payload.provenance.is_none());

            let dock = section.default_dock_region();
            assert!(!dock.region_id.is_empty());
            assert!(dock.width > 0);
            assert!(dock.height > 0);
        }
    }

    #[test]
    fn test_all_lens_tabs_honesty_badges_and_payloads() {
        let artifact = "source/tab-test";
        for tab in ManifoldLensTab::ALL {
            let badge = tab.honesty();
            match badge {
                HonestyBadge::Verified => assert_eq!(badge.as_str(), "verified"),
                HonestyBadge::Partial => assert_eq!(badge.as_str(), "partial"),
                HonestyBadge::Held => assert_eq!(badge.as_str(), "held"),
            }

            let container = tab.to_seed_container(artifact);
            assert_eq!(container.honesty, badge.as_str());
            assert_eq!(container.semantic_uri, artifact);
            assert_eq!(container.id, tab.canonical_container_id(artifact));

            let payload = tab.emit_payload(artifact);
            assert_eq!(payload.target_identifier.value, artifact);
            assert!(matches!(
                payload.context,
                ContextRef::Iri(ref iri) if iri == "https://qualiadb.org/schema/vibe#"
            ));
            assert!(payload.provenance.is_none());
        }
    }

    #[test]
    fn test_unified_workspace_layout_reconciles_projection() {
        let artifact = "source/projection-test";
        let mut layout = UnifiedWorkspaceLayout::new(artifact).expect("valid layout");
        assert_eq!(layout.active_section, ToolChestSection::Containers);
        assert_eq!(layout.active_tab, ManifoldLensTab::Studio);

        let proj = layout.to_workspace_projection().expect("valid projection");
        assert_eq!(proj.construct.id, "poet");
        assert_eq!(proj.manifold.id, "studio");

        // Studio has containers for source and inspector
        assert!(proj.manifold.containers.iter().any(|c| c.id.contains("studio-source-0")));
        assert!(proj.manifold.containers.iter().any(|c| c.id.contains("studio-inspector")));

        // Switch to Sheet tab and reconcile
        layout.active_tab = ManifoldLensTab::Sheet;
        let mut proj2 = WorkspaceProjection::for_source(artifact).expect("valid projection");
        layout.reconcile_with_projection(&mut proj2);
        assert!(proj2
            .manifold
            .containers
            .iter()
            .any(|c| c.id == format!("sheet:{artifact}")));
    }

    #[test]
    fn test_negative_invalid_artifact_id() {
        assert!(UnifiedWorkspaceLayout::new("").is_err());
        assert!(UnifiedWorkspaceLayout::new("a\0b").is_err());
        let huge = "x".repeat(5000);
        assert!(UnifiedWorkspaceLayout::new(&huge).is_err());
    }
}
