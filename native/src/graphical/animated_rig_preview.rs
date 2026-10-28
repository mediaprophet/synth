//! Actual Core player/frame adapter for one source-bound native viewport span.
use super::rig_binding::{BoundRig, ViewportSpan};
use qualia_core_db::render::{gpu::PortalGpu, skeletal::PlaybackReceipt};

pub struct AnimatedRigPreview {
    rig: BoundRig,
    last: Option<PlaybackReceipt>,
}

pub struct AnimatedFrame<'a> {
    pub source_artifact_id: &'a str,
    pub span: ViewportSpan,
    pub receipt: PlaybackReceipt,
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub tangents: &'a [[f32; 4]],
}

impl AnimatedRigPreview {
    pub fn new(rig: BoundRig) -> Self { Self { rig, last: None } }
    pub fn rig(&self) -> &BoundRig { &self.rig }
    pub fn rig_mut(&mut self) -> &mut BoundRig {
        self.last = None;
        &mut self.rig
    }

    /// Core samples from immutable rest data. Runtime time is not serialized.
    pub fn advance(&mut self, dt_seconds: f64, reduced_motion: bool) -> Result<AnimatedFrame<'_>, String> {
        // Any attempt may change player buffers before failing. Retire the
        // previously published frame before entering the fallible operation.
        self.last = None;
        self.rig.check_current()?;
        let receipt = self.rig.player_mut().advance(dt_seconds, reduced_motion)
            .map_err(|e| format!("native rig frame refused: {e:?}"))?;
        self.rig.check_current()?;
        self.last = Some(receipt);
        self.frame().ok_or_else(|| "rig frame unavailable".into())
    }

    pub fn frame(&self) -> Option<AnimatedFrame<'_>> {
        // A previously sampled frame is never a reusable publication after
        // the source incarnation/revision changes.
        self.rig.check_current().ok()?;
        let receipt = self.last?;
        let player = self.rig.player();
        Some(AnimatedFrame {
            source_artifact_id: &self.rig.source().artifact_id,
            span: self.rig.span(), receipt,
            positions: player.positions(), normals: player.normals(), tangents: player.tangents(),
        })
    }

    /// Renderer hook: caller verifies this exact span still belongs to the
    /// selected compiled part. Core updates positions, skin normals/tangents,
    /// material/shadow bounds and visibility through its existing upload API.
    pub fn write_frame(&self, gpu: &mut PortalGpu, compiled_span: ViewportSpan) -> Result<(), String> {
        self.rig.check_current()?;
        let frame = self.frame().ok_or("advance the bound rig before upload")?;
        if frame.span != compiled_span {
            return Err("compiled viewport span changed after rig admission".into());
        }
        if !gpu.write_mesh_animated_frames(
            frame.span.first_vertex, frame.positions, frame.normals, frame.tangents,
        ) { return Err("native animated frame upload refused".into()); }
        self.rig.check_current()?;
        Ok(())
    }
}
