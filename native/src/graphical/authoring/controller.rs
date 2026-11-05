//! Canonical session projections; imported as a child of controller.rs.
use super::*;
use crate::graphical::authoring::{AuthoringEdit, AuthoringView};

impl CanonicalEditorHost {
    pub(super) fn apply_texture_mask_morphology(
        &mut self,
        material_id: u64,
        layer_index: usize,
        operation: crate::graphical::surfaces::detail::MaskMorphologyOperation,
        radius: f32,
        expected: SourceRef,
    ) -> Result<EditorOutcome, String> {
        self.expected(&expected)?;
        if self.reconciliation_required || !self.session.is_mutable("poet_parts") {
            return Err("Rebuild execution and select mutable poet_parts source before mask editing".into());
        }
        let current_resources = self
            .surface_resources
            .clone()
            .ok_or("Mask morphology requires the selected accepted HMC resource snapshot")?;
        let view = self.inspect_authoring(expected.clone())?;
        let mut selected = None;
        for part in view.parts.into_iter().filter(|part| part.material_id == material_id) {
            let Some(crate::graphical::surfaces::detail::DetailRecipe::Paint(paint)) = part.detail else {
                return Err("Mask morphology requires an existing imported Paint recipe on every shared target".into());
            };
            if selected.as_ref().is_some_and(|prior| prior != &paint) {
                return Err("Shared material recipes conflict; accepted source and resources preserved".into());
            }
            selected = Some(paint);
        }
        let paint = selected.ok_or("Mask morphology target material is unavailable")?;
        let (updated, candidate_resources, _derived_digest) =
            crate::graphical::surfaces::detail::mask_morphology::prepare(
                &paint,
                layer_index,
                operation,
                radius,
                &current_resources,
                crate::graphical::surfaces::detail::DetailLimits::default(),
            )?;


        // The candidate HMC is visible only to this exclusive host call while
        // submit validates the recipe/resource pair. A clean pre-commit refusal
        // restores the prior snapshot. A possible session/source advance keeps
        // the candidate available for the existing reconciliation/reopen path.
        let prior_resources = self.surface_resources.replace(candidate_resources);
        let result = self.edit_assembly(
            AuthoringEdit::TextureDetail {
                material_id,
                detail: Some(crate::graphical::surfaces::detail::DetailRecipe::Paint(updated)),
            },
            expected,
        );
        if result.is_err() && !self.reconciliation_required {
            self.surface_resources = prior_resources;
        }
        result
    }

    pub(super) fn inspect_authoring(&self, expected: SourceRef) -> Result<AuthoringView, String> {
        self.expected(&expected)?;
        if self.reconciliation_required || !self.execution_epoch.is_current() {
            return Err("Rebuild accepted execution before authoring inspection".into());
        }
        let mut view = if let Some(parts)=self.session.value("poet_parts") {
            crate::graphical::authoring::source::view(expected.clone(),parts,self.session.value("poet_parameters"))?
        } else {
            AuthoringView { source:expected.clone(), parts:Vec::new(), parameters:Vec::new(), material_motion:Vec::new(), timeline:None, timeline_media:Vec::new(), sound_field:None, rig:None, botanical:None, junction:None }
        };
        view.material_motion = crate::graphical::authoring::material_motion::read_tracks(
            self.session.value("poet_material_motion"),
        )?;
        view.timeline = crate::graphical::authoring::timeline::read(self.session.value("poet_timeline"))?;
        if let Some(timeline)=&view.timeline { view.timeline_media=crate::graphical::authoring::timeline_associations::read(self.session.value("poet_timeline_media"),timeline)?; }
        else if self.session.value("poet_timeline_media").is_some() {return Err("Timeline associations lack an accepted timeline".into());}
        if let Some(timeline) = &view.timeline {
            view.sound_field = crate::graphical::authoring::sound_field::read(self.session.value("poet_sound_field"), timeline)?;
        } else if self.session.value("poet_sound_field").is_some() { return Err("Sound-field lacks an accepted timeline".into()); }
        view.rig = crate::graphical::authoring::rig_source::read_binding(self.session.value("poet_rig"), &expected)?;
        view.botanical =
            super::super::authoring::botanical_session::validate(&self.session, &self.host)?;
        view.junction = if let Some(binding) = self.session.value("poet_botanical_junction").filter(|v| crate::graphical::authoring::junction_session::is_junction_binding(v)) {
            let mut inspection_host = self.host.for_inspection();
            Some(inspection_host.with_junction_operation(|host| {
                let limits = host.junction_limits()?;
                let cap = limits.botanical.max_source_bytes;
                let copies = cap.checked_mul(3).ok_or("Junction inspection reservation overflow")?;
                crate::graphical::junction_budget::read_accounted(host, binding, limits, cap, cap, copies)
            })?)
        } else { None };
        Ok(view)
    }
    pub(super) fn edit_assembly(
        &mut self,
        edit: AuthoringEdit,
        expected: SourceRef,
    ) -> Result<EditorOutcome, String> {
        self.expected(&expected)?;
        if let AuthoringEdit::SoundField { edit } = &edit {
            if self.reconciliation_required || !self.execution_epoch.is_current() { return Err("Rebuild accepted execution before sound-field editing".into()); }
            let binding = self.session.value("poet_sound_field");
            if binding.is_some() && !self.session.is_mutable("poet_sound_field") { return Err("Sound-field binding is immutable".into()); }
            let timeline = crate::graphical::authoring::timeline::read(self.session.value("poet_timeline"))?.ok_or("Adopt a timeline before a sound-field")?;
            let source = crate::graphical::authoring::sound_field::edit_source(binding, &timeline, edit, self.session.value("poet_parts").is_some())?;
            return self.submit(&source, expected);
        }
        if matches!(&edit, AuthoringEdit::BotanicalJunctionAdopt { .. } | AuthoringEdit::BotanicalJunctionPosition { .. } | AuthoringEdit::BotanicalJunctionSettings { .. }) {
            return self.edit_junction(edit, expected);
        }
        if let AuthoringEdit::Timeline { edit: timeline } = &edit {
            if self.reconciliation_required || !self.execution_epoch.is_current() {
                return Err("Rebuild accepted execution before timeline editing".into());
            }
            if self.session.value("poet_timeline").is_some() && !self.session.is_mutable("poet_timeline") {
                return Err("Timeline binding is immutable; accepted source preserved".into());
            }
            let binding=self.session.value("poet_timeline");
            let media=self.session.value("poet_timeline_media");
            let current=crate::graphical::authoring::timeline::read(binding)?;
            let has_parts=self.session.value("poet_parts").is_some();
            let source=match timeline {
                crate::graphical::authoring::timeline::TimelineEdit::AssociateResource {clip_id,resource_key,resource_digest} => {
                    timeline.validate()?;
                    if media.is_some() && !self.session.is_mutable("poet_timeline_media") {return Err("Timeline association binding is immutable".into());}
                    let current=current.as_ref().ok_or("No accepted timeline")?;
                    crate::graphical::authoring::timeline_associations::bind(media,current,clip_id,resource_key,*resource_digest,has_parts)?
                }
                crate::graphical::authoring::timeline::TimelineEdit::DissociateResource {clip_id} => {
                    timeline.validate()?;
                    if media.is_some() && !self.session.is_mutable("poet_timeline_media") {return Err("Timeline association binding is immutable".into());}
                    let current=current.as_ref().ok_or("No accepted timeline")?;
                    crate::graphical::authoring::timeline_associations::unbind(media,current,clip_id,has_parts)?
                }
                _ => {
                    if media.is_some() {
                        let current=current.as_ref().ok_or("Timeline associations lack an accepted timeline")?;
                        let bindings=crate::graphical::authoring::timeline_associations::read(media,current)?;
                        match timeline {
                            crate::graphical::authoring::timeline::TimelineEdit::RemoveClip{id,..} if bindings.iter().any(|b|&b.clip_id==id) => return Err("Remove clip resource association explicitly before removing the clip".into()),
                            crate::graphical::authoring::timeline::TimelineEdit::UpdateClip{clip,..} if bindings.iter().any(|b|b.clip_id==clip.id && b.source_identifier!=clip.source) => return Err("Remove clip resource association explicitly before changing its source identifier".into()),
                            _=>{}
                        }
                    }
                    crate::graphical::authoring::timeline::edit_source_with_output(binding,timeline,has_parts)?
                }
            };
            return self.submit(&source,expected);
        }
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
            let target = match edit {
                crate::graphical::authoring::botanical::BotanicalEdit::SetRecipe { recipe } => &recipe.part_id,
                crate::graphical::authoring::botanical::BotanicalEdit::SetTipRadius { part_id, .. }
                | crate::graphical::authoring::botanical::BotanicalEdit::SetOrganPosition { part_id, .. } => part_id,
            };
            let limits = self.host.botanical_limits(crate::graphical::authoring::botanical_session::requires_uv_budget(parts, target))?;
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
    fn edit_junction(&mut self, edit: AuthoringEdit, expected: SourceRef) -> Result<EditorOutcome, String> {
        self.expected(&expected)?;
        if self.reconciliation_required || !self.execution_epoch.is_current() || !self.session.is_mutable("poet_parts") {
            return Err("Rebuild and select mutable parts before junction editing".into());
        }
        if self.session.value("poet_botanical").is_some() || self.session.value("poet_source_layer").is_some()
            || self.session.value("poet_parameter_parts").is_some() {
            return Err("Junction requires an explicit procedural-source migration".into());
        }
        if self.session.value("poet_botanical_junction").is_some()
            && !self.session.is_mutable("poet_botanical_junction") {
            return Err("Junction binding is immutable".into());
        }
        crate::graphical::authoring::junction_session::require_replay_limits(&self.rebuild_limits)?;
        let owns_operation = self.host.begin_junction_operation()?;
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<EditorOutcome, String> {
            let limits = self.host.junction_limits()?;
            let parts = self.session.value("poet_parts").ok_or("Missing poet_parts")?;
            let source = match &edit {
                AuthoringEdit::BotanicalJunctionAdopt { source } => {
                    if self.session.value("poet_botanical_junction").is_some() { return Err("Junction already adopted".into()); }
                    let cap = limits.botanical.max_source_bytes;
                    let clones = cap.checked_mul(3).ok_or("Junction source reservation overflow")?;
                    let source = crate::graphical::junction_budget::read_accounted(&mut self.host, source, limits, cap, cap, clones)?;
                    crate::graphical::authoring::botanical::junction_edit::prepare_adoption(parts, &source, limits, &mut self.host)?
                }
                AuthoringEdit::BotanicalJunctionPosition { target, organ, position_m } => {
                    let binding = self.session.value("poet_botanical_junction").ok_or("No accepted junction")?;
                    crate::graphical::authoring::botanical::junction_edit::prepare_position_edit(parts, binding, target, organ, *position_m, limits, &mut self.host)?
                }
                AuthoringEdit::BotanicalJunctionSettings { target, settings } => {
                    let binding = self.session.value("poet_botanical_junction").ok_or("No accepted junction")?;
                    crate::graphical::authoring::botanical::junction_edit::prepare_settings_edit(parts, binding, target, settings, limits, &mut self.host)?
                }                _ => return Err("Not a junction edit".into()),
            };
            // Dedicated submit validates tentative bindings with the SAME host
            // operation before source/history promotion; it must not reset it.
            self.submit_with_junction_operation(&source, expected)
        }));
        if owns_operation { self.host.end_junction_operation(); }
        match outcome { Ok(result) => result, Err(panic) => { self.reconciliation_required = true; std::panic::resume_unwind(panic) } }
    }    pub(super) fn compile_authored(
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
