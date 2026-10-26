//! Native control actions against the same Core player used by frame preview.
//! The resulting descriptor JSON is a source edit candidate; the shared native
//! session owner must check authority, revision and source before promoting it.
use super::{BoundRig, JointDeltaSource, RigSourceDescriptor};
use qualia_core_db::render::skeletal::{Motor, SkeletalPlayer};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", deny_unknown_fields)]
pub enum RigControl {
    SelectClip { clip_id: u64, fade_seconds: f64 },
    Seek { seconds: f64 },
    SetJointDelta { joint_id: u64, motor: [f64; 8] },
}

fn motor(a: [f64; 8]) -> Motor {
    Motor {
        r_w: a[0],
        r_x: a[1],
        r_y: a[2],
        r_z: a[3],
        d_w: a[4],
        d_x: a[5],
        d_y: a[6],
        d_z: a[7],
    }
}

pub(super) fn restore_source_controls(
    descriptor: &RigSourceDescriptor,
    player: &mut SkeletalPlayer,
) -> Result<(), String> {
    player
        .select_clip(descriptor.selected_clip_id, 0.0)
        .map_err(|e| format!("selected source clip refused: {e:?}"))?;
    player
        .seek(descriptor.seek_seconds)
        .map_err(|e| format!("source seek refused: {e:?}"))?;
    let mut seen = std::collections::BTreeSet::new();
    for delta in &descriptor.joint_deltas {
        if !seen.insert(delta.joint_id) || !descriptor.joint_ids.contains(&delta.joint_id) {
            return Err("duplicate or foreign joint control".into());
        }
        player
            .set_joint_delta(delta.joint_id, motor(delta.motor))
            .map_err(|e| format!("source joint delta refused: {e:?}"))?;
    }
    player
        .advance(0.0, false)
        .map_err(|e| format!("source pose refused: {e:?}"))?;
    Ok(())
}

/// Applies one checked native action and updates the source-owned descriptor.
/// Submit the returned descriptor through the existing expected-source edit
/// route; calling this method alone does not authorize or persist a change.
pub fn apply_control(
    rig: &mut BoundRig,
    action: RigControl,
) -> Result<&RigSourceDescriptor, String> {
    rig.check_current()?;
    match action {
        RigControl::SelectClip {
            clip_id,
            fade_seconds,
        } => {
            if !rig.descriptor.clip_ids.contains(&clip_id) {
                return Err("clip is not in the selected source rig".into());
            }
            if fade_seconds != 0.0 {
                return Err("authored clip fade is unsupported until its transition state can be reconstructed".into());
            }
            // This authored command means "start the selected clip at t=0".
            // Core select_clip is a no-op for the already-active clip, which
            // would leave live time at t>0 while serializing seek_seconds=0.
            rig.player
                .restart_clip(clip_id, 0.0)
                .map_err(|e| format!("clip restart refused: {e:?}"))?;
            rig.descriptor.selected_clip_id = clip_id;
            rig.descriptor.seek_seconds = 0.0;
        }
        RigControl::Seek { seconds } => {
            rig.player
                .seek(seconds)
                .map_err(|e| format!("seek refused: {e:?}"))?;
            rig.descriptor.seek_seconds = seconds;
        }
        RigControl::SetJointDelta {
            joint_id,
            motor: values,
        } => {
            if !rig.descriptor.joint_ids.contains(&joint_id) {
                return Err("joint is not in the selected source rig".into());
            }
            rig.player
                .set_joint_delta(joint_id, motor(values))
                .map_err(|e| format!("joint delta refused: {e:?}"))?;
            if let Some(entry) = rig
                .descriptor
                .joint_deltas
                .iter_mut()
                .find(|d| d.joint_id == joint_id)
            {
                entry.motor = values;
            } else {
                rig.descriptor.joint_deltas.push(JointDeltaSource {
                    joint_id,
                    motor: values,
                });
                rig.descriptor.joint_deltas.sort_by_key(|d| d.joint_id);
            }
        }
    }
    rig.player
        .advance(0.0, false)
        .map_err(|e| format!("controlled pose refused: {e:?}"))?;
    rig.check_current()?;
    Ok(&rig.descriptor)
}

/// Canonical JSON source-resource literal for the existing source/HMC edit path.
/// The shared source owner must bind this literal to its authenticated source
/// revision and selected resource; it is never interpreted as a permission.
pub fn descriptor_source_literal(descriptor: &RigSourceDescriptor) -> Result<String, String> {
    serde_json::to_string(descriptor).map_err(|e| format!("rig source literal: {e}"))
}

pub fn parse_descriptor_source_literal(source: &str) -> Result<RigSourceDescriptor, String> {
    if source.len() > 2 * 1024 * 1024 {
        return Err("rig source literal exceeds bound".into());
    }
    serde_json::from_str(source).map_err(|e| format!("rig source literal refused: {e}"))
}
