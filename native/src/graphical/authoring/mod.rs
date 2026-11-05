//! Native authoring intents over the canonical source session, never a second document.
use crate::graphical::preview::SourceRef;
mod assembly_source;
#[cfg(test)]
mod atlas_detail_tests;
pub mod botanical;
pub(crate) mod junction_session;
mod junction_controls;
pub(crate) mod botanical_session;
mod cad_controls;
mod cad_recipe;
#[cfg(test)]
mod cad_tests;
pub(crate) mod controls;
mod detail_controls;
mod geometry_source;
pub(crate) mod rig_source;
pub(crate) mod material_motion;
pub(crate) mod timeline;
pub(crate) mod timeline_associations;
#[cfg(test)]
mod timeline_associations_tests;
mod timeline_controls;
mod timeline_frame_entry;
mod timeline_create_controls;
pub(crate) mod timeline_media;
pub(crate) mod sound_field;
mod sound_field_controls;
#[cfg(test)]
mod timeline_tests;
mod material_motion_controls;
#[cfg(test)]
mod material_motion_tests;
pub(crate) mod package;
mod parameter_safety;
#[cfg(test)]
mod parameter_safety_restart_tests;
#[cfg(test)]
mod parameter_safety_tests;
mod profile_controls;
mod profile_recipe;
#[cfg(test)]
mod profile_tests;
pub(crate) mod source;
#[cfg(test)]
mod source_layer_tests;
pub(crate) mod source_layers;
mod surface_binding_controls;
mod surface_controls;
mod surface_tools;
mod uv_atlas_controls;
use crate::graphical::surfaces::uv::PlanarUvDraft;
use crate::graphical::surfaces::{SurfaceDraft, UvTransformDraft};
pub use profile_recipe::ProfileRecipe;
mod structure_controls;
#[cfg(test)]
mod structure_tests;
#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct PartView {
    pub uv_atlas: Option<crate::graphical::surfaces::uv_atlas::AtlasDraft>,
    pub detail: Option<crate::graphical::surfaces::detail::DetailRecipe>,
    pub surface: SurfaceDraft,
    pub uv_transform: UvTransformDraft,
    pub uv_projection: Option<PlanarUvDraft>,
    pub id: String,
    pub parent: Option<String>,
    pub semantic_id: u64,
    pub translation: [f64; 3],
    pub rotation: [f64; 3],
    pub scale: [f64; 3],
    pub material_id: u64,
    pub base_color: [f64; 4],
    pub roughness: f64,
    pub metallic: f64,
    pub vertices: usize,
    pub triangles: usize,
}

#[derive(Clone, Debug)]
pub struct AuthoringView {
    pub sound_field: Option<sound_field::SoundField>,
    pub timeline: Option<timeline::Timeline>,
    pub timeline_media: Vec<timeline_associations::Association>,
    pub botanical: Option<botanical::BotanicalRecipe>,
    pub junction: Option<botanical::JunctionSource>,
    pub rig: Option<crate::graphical::rig_binding::RigSourceDescriptor>,
    pub material_motion: Vec<material_motion::MaterialMotionTrack>,
    pub source: SourceRef,
    pub parts: Vec<PartView>,
    pub parameters: Vec<(String, f64)>,
}

#[derive(Clone, Debug)]
pub enum AuthoringEdit {
    SoundField { edit: sound_field::SoundFieldEdit },
    Timeline { edit: timeline::TimelineEdit },
    Rig { resource: crate::graphical::checkpoints::ResourceInspectionRequest, control: crate::graphical::rig_binding::RigControl },
    BotanicalJunctionAdopt { source: vibe::Value },
    BotanicalJunctionPosition { target: String, organ: String, position_m: [f64; 3] },
    BotanicalJunctionSettings { target: String, settings: botanical::JunctionSettings },
    Botanical {
        edit: botanical::BotanicalEdit,
    },
    MaterialMotion {
        edit: material_motion::MaterialMotionEdit,
    },
    /// Explicitly adopt an existing procedural recipe without resetting edited parts.
    EnableSourceLayers,
    UvAtlas {
        id: String,
        atlas: Option<crate::graphical::surfaces::uv_atlas::AtlasDraft>,
    },
    TextureDetail {
        material_id: u64,
        detail: Option<crate::graphical::surfaces::detail::DetailRecipe>,
    },
    /// Morph one existing PaintLayer mask through the same authorized native source owner.
    TextureMaskMorphology {
        material_id: u64,
        layer_index: usize,
        operation: crate::graphical::surfaces::detail::MaskMorphologyOperation,
        radius: f32,
    },
    Surface {
        material_id: u64,
        surface: SurfaceDraft,
    },
    UvTransform {
        id: String,
        transform: UvTransformDraft,
    },
    UvProjection {
        id: String,
        projection: Option<PlanarUvDraft>,
    },
    /// Changes containment while retaining the exact parent-relative TRS values.
    Reparent {
        id: String,
        parent: Option<String>,
    },
    RemoveLeaf {
        id: String,
    },
    ReplaceGeometry {
        id: String,
        geometry: GeometryRecipe,
    },
    /// Creates an unattached identity-TRS part, copying only the template material.
    AddGeometry {
        id: String,
        semantic_id: u64,
        material_from: String,
        geometry: GeometryRecipe,
    },
    /// Duplicates this part's complete material record under an unused identifier.
    UniqueMaterial {
        id: String,
        material_id: u64,
    },
    Transform {
        id: String,
        translation: [f64; 3],
        rotation: [f64; 3],
        scale: [f64; 3],
    },
    Material {
        material_id: u64,
        base_color: [f64; 4],
        roughness: f64,
        metallic: f64,
    },
    Parameter {
        key: String,
        value: f64,
    },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum GeometryRecipe {
    ProfileSweep {
        profile: ProfileRecipe,
        length: f64,
        caps: bool,
    },
    RevolveProfile {
        points: Vec<[f64; 2]>,
        segments: u32,
    },
    Sphere {
        radius: f64,
        lat_segments: u32,
        lon_segments: u32,
    },
    Torus {
        major_radius: f64,
        minor_radius: f64,
        major_segments: u32,
        minor_segments: u32,
    },
    CircularSweep {
        radius: f64,
        length: f64,
        segments: u32,
        caps: bool,
    },
}
impl GeometryRecipe {
    pub(crate) fn validate(&self) -> Result<(), String> {
        geometry_source::expression(self).map(|_| ())
    }
}
