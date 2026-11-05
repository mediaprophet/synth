//! Bounded command decode for the original Poet native intent owner.
use super::*;
#[derive(Deserialize)]
#[serde(tag = "command", deny_unknown_fields)]
pub(super) enum Command {
    #[serde(rename = "authoring.terrain_brush")]
    TerrainBrush { draft: crate::graphical::terrain_brush::TerrainBrushDraft },
    #[serde(rename = "authoring.rig")]
    Rig { resource: crate::graphical::checkpoints::ResourceInspectionRequest, control: crate::graphical::rig_binding::RigControl },
    #[serde(rename = "authoring.botanical")]
    Botanical {
        edit: crate::graphical::authoring::botanical::BotanicalEdit,
    },
    #[serde(rename = "authoring.material_motion")]
    MaterialMotion {
        edit: crate::graphical::authoring::material_motion::MaterialMotionEdit,
    },
    #[serde(rename = "authoring.timeline")]
    Timeline { edit: crate::graphical::authoring::timeline::TimelineEdit },
    #[serde(rename = "authoring.sound_field")]
    SoundField { edit: crate::graphical::authoring::sound_field::SoundFieldEdit },
    #[serde(rename = "viewport.navigate")]
    Viewport {
        request: crate::graphical::viewport::ViewportCommand,
    },
    #[serde(rename = "authoring.enable_source_layers")]
    EnableSourceLayers,
    #[serde(rename = "authoring.inspect_surface")]
    InspectSurface { id: String },
    #[serde(rename = "authoring.uv_atlas")]
    UvAtlas {
        id: String,
        atlas: Option<crate::graphical::surfaces::uv_atlas::AtlasDraft>,
    },
    #[serde(rename = "authoring.texture_detail")]
    TextureDetail {
        material_id: u64,
        detail: Option<crate::graphical::surfaces::detail::DetailRecipe>,
    },
    #[serde(rename = "authoring.texture_mask_morphology")]
    TextureMaskMorphology {
        material_id: u64,
        layer_index: usize,
        operation: String,
        radius: f32,
    },
    #[serde(rename = "authoring.surface")]
    Surface {
        material_id: u64,
        surface: crate::graphical::surfaces::SurfaceDraft,
    },
    #[serde(rename = "authoring.uv_transform")]
    UvTransform {
        id: String,
        transform: crate::graphical::surfaces::wire::UvTransformWire,
    },
    #[serde(rename = "authoring.uv_projection")]
    UvProjection {
        id: String,
        projection: Option<crate::graphical::surfaces::uv::PlanarUvDraft>,
    },
    #[serde(rename = "authoring.bind_surface_package")]
    BindSurface {
        request: crate::graphical::surface_host::SurfacePackageRequest,
    },
    #[serde(rename = "authoring.inspect")]
    Inspect,
    #[serde(rename = "source.check")]
    Check { source: String },
    #[serde(rename = "source.submit")]
    Submit { source: String },
    #[serde(rename = "source.undo")]
    Undo,
    #[serde(rename = "source.redo")]
    Redo,
    #[serde(rename = "source.rebuild")]
    Rebuild,
    #[serde(rename = "source.refresh")]
    Refresh,
    #[serde(rename = "package.save")]
    Save { name: String },
    #[serde(rename = "authoring.transform")]
    Transform {
        id: String,
        translation: [f64; 3],
        rotation: [f64; 3],
        scale: [f64; 3],
    },
    #[serde(rename = "authoring.material")]
    Material {
        material_id: u64,
        base_color: [f64; 4],
        roughness: f64,
        metallic: f64,
    },
    #[serde(rename = "authoring.parameter")]
    Parameter { key: String, value: f64 },
    #[serde(rename = "authoring.reparent")]
    Reparent { id: String, parent: Option<String> },
    #[serde(rename = "authoring.remove_leaf")]
    RemoveLeaf { id: String },
    #[serde(rename = "authoring.replace_geometry")]
    ReplaceGeometry {
        id: String,
        geometry: crate::graphical::authoring::GeometryRecipe,
    },
    #[serde(rename = "authoring.add_geometry")]
    AddGeometry {
        id: String,
        semantic_id: u64,
        material_from: String,
        geometry: crate::graphical::authoring::GeometryRecipe,
    },
    #[serde(rename = "authoring.unique_material")]
    UniqueMaterial { id: String, material_id: u64 },
    #[serde(rename = "authoring.compile")]
    Compile {
        name: String,
        key: String,
        #[serde(default)]
        expected_sha256: Option<[u8; 32]>,
    },
}
impl Command {
    pub(super) fn action_type(&self) -> ActionType {
        match self {
            Self::Viewport { .. }
            | Self::InspectSurface { .. }
            | Self::BindSurface { .. }
            | Self::Inspect
            | Self::Check { .. }
            | Self::Refresh => ActionType::Query,
            _ => ActionType::Invoke,
        }
    }
    pub(super) fn scope(&self) -> &'static str {
        match self {
            Self::TerrainBrush { .. } => "Studio.checkpoint",
            Self::InspectSurface { .. } | Self::BindSurface { .. } => "Studio.inspect",
            Self::UvAtlas { .. }
            | Self::TextureDetail { .. }
            | Self::TextureMaskMorphology { .. } => "Studio.edit",
            Self::Surface { .. } | Self::UvTransform { .. } | Self::UvProjection { .. } => {
                "Studio.edit"
            }
            Self::Viewport { .. } => "Studio.inspect",
            Self::Rig { .. } | Self::Botanical { .. } | Self::EnableSourceLayers | Self::MaterialMotion { .. } | Self::Timeline { .. } | Self::SoundField { .. } => {
                "Studio.edit"
            }
            Self::Inspect => "Studio.inspect",
            Self::Check { .. } => "Vibe.check",
            Self::Submit { .. } => "Studio.submit",
            Self::Undo => "Studio.undo",
            Self::Redo => "Studio.redo",
            Self::Rebuild => "Studio.rebuild",
            Self::Refresh => "Studio.inspect",
            Self::Save { .. } => "Studio.checkpoint",
            Self::Reparent { .. }
            | Self::RemoveLeaf { .. }
            | Self::ReplaceGeometry { .. }
            | Self::AddGeometry { .. }
            | Self::UniqueMaterial { .. }
            | Self::Transform { .. }
            | Self::Material { .. }
            | Self::Parameter { .. } => "Studio.edit",
            Self::Compile { .. } => "Studio.compile",
        }
    }
    pub(super) fn action(self, origin: &Context) -> Result<Action, String> {
        let expected = origin.source.clone();
        let text = |s: &str, max| {
            if s.is_empty() || s.len() > max || s.as_bytes().contains(&0) {
                Err("Poet intent text violates native command bounds".to_string())
            } else {
                Ok(())
            }
        };
        Ok(match self {
            Self::TerrainBrush { draft } => { draft.validate()?; Action::BrushGeoRaster { draft, expected } },
            Self::Viewport { request } => {
                crate::graphical::viewport_contract::validate_command(&request)?;
                Action::Viewport { request, expected }
            }
            Self::Rig { resource, control } => {
                resource.validate_bounds()?;
                Action::AuthoringEdit { edit: AuthoringEdit::Rig { resource, control }, expected }
            }
            Self::Botanical { edit } => Action::AuthoringEdit {
                edit: AuthoringEdit::Botanical { edit },
                expected,
            },
            Self::MaterialMotion { edit } => {
                edit.validate()?;
                Action::AuthoringEdit {
                    edit: AuthoringEdit::MaterialMotion { edit },
                    expected,
                }
            }
            Self::Timeline { edit } => {
                edit.validate()?;
                Action::AuthoringEdit { edit: AuthoringEdit::Timeline { edit }, expected }
            }
            Self::SoundField { edit } => {
                edit.validate()?;
                Action::AuthoringEdit { edit: AuthoringEdit::SoundField { edit }, expected }
            }
            Self::EnableSourceLayers => Action::AuthoringEdit {
                edit: AuthoringEdit::EnableSourceLayers,
                expected,
            },
            Self::InspectSurface { id } => {
                text(&id, 128)?;
                Action::InspectSurface { id, expected }
            }
            Self::UvAtlas { id, atlas } => {
                text(&id, 128)?;
                if let Some(a) = &atlas {
                    crate::graphical::surfaces::uv_atlas::source(a)?;
                }
                Action::AuthoringEdit {
                    edit: AuthoringEdit::UvAtlas { id, atlas },
                    expected,
                }
            }
            Self::TextureDetail {
                material_id,
                detail,
            } => {
                if material_id == 0 {
                    return Err("Nonzero material identifier required".into());
                }
                if let Some(d) = &detail {
                    d.source_expression()?;
                }
                Action::AuthoringEdit {
                    edit: AuthoringEdit::TextureDetail {
                        material_id,
                        detail,
                    },
                    expected,
                }
            }
            Self::TextureMaskMorphology {
                material_id,
                layer_index,
                operation,
                radius,
            } => {
                if material_id == 0 || !radius.is_finite() {
                    return Err("Mask morphology requires a nonzero material and finite radius".into());
                }
                let operation = match operation.as_str() {
                    "expand" => crate::graphical::surfaces::detail::MaskMorphologyOperation::Expand,
                    "contract" => crate::graphical::surfaces::detail::MaskMorphologyOperation::Contract,
                    "border" => crate::graphical::surfaces::detail::MaskMorphologyOperation::Border,
                    _ => return Err("Unknown mask morphology operation".into()),
                };
                Action::AuthoringEdit {
                    edit: AuthoringEdit::TextureMaskMorphology {
                        material_id,
                        layer_index,
                        operation,
                        radius,
                    },
                    expected,
                }
            }
            Self::BindSurface { request } => {
                request.validate()?;
                Action::BindSurfacePackage { request, expected }
            }
            Self::Surface {
                material_id,
                surface,
            } => {
                if material_id == 0 {
                    return Err("Material identifier must be nonzero".into());
                }
                surface.updates()?;
                Action::AuthoringEdit {
                    edit: AuthoringEdit::Surface {
                        material_id,
                        surface,
                    },
                    expected,
                }
            }
            Self::UvTransform { id, transform } => {
                text(&id, 128)?;
                let transform = transform.try_into()?;
                Action::AuthoringEdit {
                    edit: AuthoringEdit::UvTransform { id, transform },
                    expected,
                }
            }
            Self::UvProjection { id, projection } => {
                text(&id, 128)?;
                if let Some(p) = &projection {
                    crate::graphical::surfaces::uv::projection_source(p)?;
                }
                Action::AuthoringEdit {
                    edit: AuthoringEdit::UvProjection { id, projection },
                    expected,
                }
            }
            Self::Inspect => Action::AuthoringInspect(expected),
            Self::Check { source } => {
                text(&source, 64 * 1024)?;
                Action::Check(source)
            }
            Self::Submit { source } => {
                text(&source, 64 * 1024)?;
                Action::Submit { source, expected }
            }
            Self::Undo => Action::Undo(expected),
            Self::Redo => Action::Redo(expected),
            Self::Rebuild => Action::Rebuild(expected),
            Self::Refresh => Action::Refresh,
            Self::Save { name } => {
                text(&name, 128)?;
                Action::Save { name, expected }
            }
            Self::Transform {
                id,
                translation,
                rotation,
                scale,
            } => {
                text(&id, 4096)?;
                if translation
                    .iter()
                    .chain(&rotation)
                    .chain(&scale)
                    .any(|n| !n.is_finite())
                    || scale.iter().any(|n| *n <= 0.0)
                {
                    return Err(
                        "Poet transform requires finite coordinates and positive scale".into(),
                    );
                }
                Action::AuthoringEdit {
                    edit: AuthoringEdit::Transform {
                        id,
                        translation,
                        rotation,
                        scale,
                    },
                    expected,
                }
            }
            Self::Material {
                material_id,
                base_color,
                roughness,
                metallic,
            } => {
                if material_id == 0
                    || base_color
                        .iter()
                        .chain([&roughness, &metallic])
                        .any(|n| !n.is_finite())
                {
                    return Err(
                        "Poet material intent needs a nonzero identifier and finite factors".into(),
                    );
                }
                Action::AuthoringEdit {
                    edit: AuthoringEdit::Material {
                        material_id,
                        base_color,
                        roughness,
                        metallic,
                    },
                    expected,
                }
            }
            Self::Parameter { key, value } => {
                text(&key, 4096)?;
                if !value.is_finite() {
                    return Err("Poet parameter must be finite".into());
                }
                Action::AuthoringEdit {
                    edit: AuthoringEdit::Parameter { key, value },
                    expected,
                }
            }
            Self::Reparent { id, parent } => {
                text(&id, 128)?;
                if let Some(p) = &parent {
                    text(p, 128)?;
                }
                Action::AuthoringEdit {
                    edit: AuthoringEdit::Reparent { id, parent },
                    expected,
                }
            }
            Self::RemoveLeaf { id } => {
                text(&id, 128)?;
                Action::AuthoringEdit {
                    edit: AuthoringEdit::RemoveLeaf { id },
                    expected,
                }
            }
            Self::ReplaceGeometry { id, geometry } => {
                text(&id, 128)?;
                geometry.validate()?;
                Action::AuthoringEdit {
                    edit: AuthoringEdit::ReplaceGeometry { id, geometry },
                    expected,
                }
            }
            Self::AddGeometry {
                id,
                semantic_id,
                material_from,
                geometry,
            } => {
                text(&id, 128)?;
                text(&material_from, 128)?;
                geometry.validate()?;
                if semantic_id == 0 || semantic_id > i64::MAX as u64 {
                    return Err("Part identifier outside positive control profile".into());
                }
                Action::AuthoringEdit {
                    edit: AuthoringEdit::AddGeometry {
                        id,
                        semantic_id,
                        material_from,
                        geometry,
                    },
                    expected,
                }
            }
            Self::UniqueMaterial { id, material_id } => {
                text(&id, 128)?;
                if material_id == 0 || material_id > i64::MAX as u64 {
                    return Err("Material identifier outside positive control profile".into());
                }
                Action::AuthoringEdit {
                    edit: AuthoringEdit::UniqueMaterial { id, material_id },
                    expected,
                }
            }
            Self::Compile {
                name,
                key,
                expected_sha256,
            } => {
                text(&name, 128)?;
                text(&key, 4096)?;
                Action::CompileAssembly {
                    name,
                    key,
                    file: expected_sha256.map_or(ExpectedFile::Absent, ExpectedFile::Sha256),
                    expected,
                }
            }
        })
    }
}

#[cfg(test)]
mod sound_field_command_tests {
    use super::*;
    #[test]
    fn field_payload_uses_existing_edit_scope_and_rejects_unknown_or_lossy_fields() {
        let good=serde_json::json!({"command":"authoring.sound_field","edit":{"operation":"set_clip","clip_id":"bird","entity_identifier":"urn:bird:1","position_m":[1.0,0.0,-2.0],"radius_m":25.0,"gain":0.4,"priority":2}});
        let command:Command=serde_json::from_value(good.clone()).unwrap();
        assert_eq!(command.scope(),"Studio.edit");
        assert!(matches!(command.action_type(),ActionType::Invoke));
        let Command::SoundField {edit}=command else {panic!("typed field command absent")};edit.validate().unwrap();
        let mut unknown=good.clone();unknown["edit"]["authority_grant"]=serde_json::json!(true);
        assert!(serde_json::from_value::<Command>(unknown).is_err());
        let mut lossy=good.clone();lossy["edit"]["priority"]=serde_json::json!(0.5);
        assert!(serde_json::from_value::<Command>(lossy).is_err());
        let mut invalid=good;invalid["edit"]["gain"]=serde_json::json!(2.0);
        let Command::SoundField {edit}=serde_json::from_value::<Command>(invalid).unwrap() else {unreachable!()};
        assert!(edit.validate().is_err());
    }
}
