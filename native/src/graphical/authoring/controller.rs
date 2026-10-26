//! Canonical session projections; imported as a child of controller.rs.
use super::*;
use crate::graphical::authoring::{AuthoringEdit, AuthoringView};

impl CanonicalEditorHost {
    pub(super) fn inspect_authoring(&self, expected: SourceRef) -> Result<AuthoringView, String> {
        self.expected(&expected)?;
        if self.reconciliation_required || !self.execution_epoch.is_current() {
            return Err("Rebuild accepted execution before authoring inspection".into());
        }
        let parts = self
            .session
            .value("poet_parts")
            .ok_or("Source has no poet_parts authoring binding")?;
        let mut view = crate::graphical::authoring::source::view(
            expected,
            parts,
            self.session.value("poet_parameters"),
        )?;
        view.material_motion = crate::graphical::authoring::material_motion::read_tracks(
            self.session.value("poet_material_motion"),
        )?;
        view.botanical =
            super::super::authoring::botanical_session::validate(&self.session, &self.host)?;
        Ok(view)
    }
    pub(super) fn edit_assembly(
        &mut self,
        edit: AuthoringEdit,
        expected: SourceRef,
    ) -> Result<EditorOutcome, String> {
        self.expected(&expected)?;
        if self.reconciliation_required || !self.session.is_mutable("poet_parts") {
            return Err(
                "Rebuild execution and select a mutable poet_parts source before editing".into(),
            );
        }
        if matches!(
            edit,
            AuthoringEdit::Parameter { .. } | AuthoringEdit::EnableSourceLayers
        ) && !self.session.is_mutable("poet_parameters")
        {
            return Err("Source parameters are not mutable".into());
        }
        if matches!(edit, AuthoringEdit::Parameter { .. })
            && self.session.value("poet_parameter_parts").is_some()
            && !self.session.is_mutable("poet_parameter_parts")
        {
            return Err("Source parameter baseline must be mutable; preserve source and migrate the explicit recipe lineage".into());
        }
        let parts = self
            .session
            .value("poet_parts")
            .ok_or("Missing poet_parts")?;
        if matches!(edit, AuthoringEdit::Parameter { .. })
            && self.session.value("poet_source_layer").is_some()
            && !self.session.is_mutable("poet_source_layer")
        {
            return Err(
                "Source layer must be mutable; preserve source and explicitly migrate".into(),
            );
        }
        let revision = self
            .session
            .revision()
            .checked_add(1)
            .ok_or("Authoring revision overflow")?;
        let source = if let AuthoringEdit::Botanical { edit } = &edit {
            if self.session.value("poet_source_layer").is_some()
                || self.session.value("poet_parameter_parts").is_some()
            {
                return Err("Botanical graph edits require explicit source-layer correspondence; accepted recipe preserved".into());
            }
            if self.session.value("poet_botanical").is_some()
                && !self.session.is_mutable("poet_botanical")
            {
                return Err("Botanical binding is immutable; accepted source preserved".into());
            }
            let limits = self.host.botanical_limits()?;
            crate::graphical::authoring::botanical::edit_source(
                parts,
                self.session.value("poet_botanical"),
                edit,
                limits,
            )?
        } else if let AuthoringEdit::MaterialMotion { edit: motion } = &edit {
            if self.session.value("poet_material_motion").is_some()
                && !self.session.is_mutable("poet_material_motion")
            {
                return Err(
                    "Material motion binding is immutable; accepted source preserved".into(),
                );
            }
            crate::graphical::authoring::material_motion::edit_source(
                parts,
                self.session.value("poet_parameters"),
                self.session.value("poet_parameter_parts"),
                self.session.value("poet_source_layer"),
                self.session.value("poet_material_motion"),
                motion,
                revision,
            )?
        } else {
            crate::graphical::authoring::source::edit_source_with_layers(
                parts,
                self.session.value("poet_parameters"),
                self.session.value("poet_parameter_parts"),
                self.session.value("poet_source_layer"),
                &edit,
                revision,
            )?
        };
        self.submit(&source, expected)
    }
    pub(super) fn compile_authored(
        &self,
        key: &str,
        expected: SourceRef,
    ) -> Result<Vec<(String, qualia_core_db::render::compile_10d::CompiledAsset)>, String> {
        self.expected(&expected)?;
        if self.reconciliation_required || !self.execution_epoch.is_current() {
            return Err("Rebuild accepted execution before compiling".into());
        }
        crate::graphical::authoring::botanical_session::validate(&self.session, &self.host)?;
        crate::graphical::package::ResourceLibrary::validate_filename(key)?;
        if key.len() > 96 {
            return Err("Compiled asset key exceeds host limit".into());
        }
        let parts = self
            .session
            .value("poet_parts")
            .ok_or("No authored assembly in the current execution")?;
        let result = Value::Record(std::collections::BTreeMap::from([(
            "parts".into(),
            parts.clone(),
        )]));
        let resolved = crate::graphical::surfaces::resolution::resolve(
            &result,
            self.surface_resources.as_ref(),
        )?;
        let scene = crate::graphical::assembly::admit_assembly_with_materials(
            &result,
            crate::graphical::preview::PreviewLimits {
                side: 512,
                vertices: self.host.vertex_limit(),
                indices: self
                    .host
                    .vertex_limit()
                    .checked_mul(6)
                    .ok_or("Compiled assembly index budget overflow")?,
            },
            &resolved.materials,
        )?;
        let bytes = crate::graphical::assembly::compile_assembly(&scene)?;
        let source = self
            .source_cells(expected.clone())?
            .join(
                "

",
            )
            .into_bytes();
        let source_digest = qualia_core_db::container_10d::crc32c(&source);
        let compiled_digest = qualia_core_db::render::compile_10d::compiled_digest(&bytes);
        let uri = format!("urn:poet:authored-asset:{key}");
        let (quins, lexicon) = qualia_core_db::render::assets::mesh_to_nquins_with_digests(
            &scene.mesh,
            &uri,
            "vibe",
            source_digest,
            compiled_digest,
        );
        expected.check_current()?;
        Ok(vec![(
            key.into(),
            qualia_core_db::render::compile_10d::CompiledAsset {
                mesh: scene.mesh,
                container_10d: bytes,
                texture_dependencies: resolved.dependencies(&scene.materials)?,
                compiled_digest,
                source_digest,
                quins,
                lexicon,
            },
        )])
    }
}
