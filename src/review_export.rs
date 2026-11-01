//! Bounded, create-only review snapshot export through Synth's admitted player.
//! This is a viewer/navigation projection, not a project graph or acceptance.
use ecopunk_terrain::{
    animal_motion::{export_frame_10d, PoseExportLimits},
    species_hmc_review::inspect_admitted,
    species_playback::{time::sample_seconds_to_tick, SpeciesPlaybackSession},
    species_review_resources::ReviewResourceLimits,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

const MAX_REVIEW_BYTES: usize = 256 * 1024 * 1024;
const MAX_FILES: usize = 64;
const MAX_INDEX_BYTES: usize = 256 * 1024;
const MAX_PHASES: usize = 64;
const MAX_PHASE_RECEIPT_BYTES: usize = 16 * 1024;
const VIEWER_10D_LIMIT: usize = 16 * 1024 * 1024;
const VIEWER_HMC_LIMIT: usize = 64 * 1024 * 1024;
const REVIEW_PHASES: &[(&str, &[f64])] = &[
    ("head-look-study", &[0.0, 0.5, 1.0]),
    ("hop-step-study", &[0.0, 0.15, 0.4, 0.6, 0.9, 1.1, 1.2]),
];

struct PendingFile {
    name: String,
    bytes: Vec<u8>,
    time_s: Option<f64>,
}

/// Compute all native outputs before creating `review_dir`. The HMC is the
/// authoritative package; the JSON files are explicitly read-only projections.
pub(crate) fn write_review_export(
    playback: &mut SpeciesPlaybackSession,
    review_dir: &Path,
    package_path: &Path,
    limits: ReviewResourceLimits,
    cancel: &AtomicBool,
) -> Result<Value, String> {
    let asset = playback.asset().ok_or("No reopened Synth candidate")?;
    let master = asset.render.bytes.clone();
    let package = asset.bundle.clone();
    let master_manifest = serde_json::to_vec_pretty(&asset.render.metadata)
        .map_err(|e| format!("Master projection serialization: {e}"))?;
    let package_review = inspect_admitted(1, asset, limits, cancel)
        .map_err(|e| format!("Native HMC review projection: {e}"))?;
    let package_review = serde_json::to_vec_pretty(&package_review)
        .map_err(|e| format!("Native HMC review projection serialization: {e}"))?;
    if package_review.len() > MAX_INDEX_BYTES {
        return Err("Native package review projection exceeds 256 KiB".into());
    }

    let mut pending = vec![
        PendingFile {
            name: "wallaby-motion-master.10d".into(),
            bytes: master.clone(),
            time_s: None,
        },
        PendingFile {
            name: "wallaby-motion-review.hmc".into(),
            bytes: package,
            time_s: None,
        },
        PendingFile {
            name: "master-manifest.json".into(),
            bytes: master_manifest,
            time_s: None,
        },
        PendingFile {
            name: "package-review.json".into(),
            bytes: package_review,
            time_s: None,
        },
    ];

    let pose_limits = PoseExportLimits::default();
    let mut phase_count = 0usize;
    let mut logical_retained_bytes = pending.iter().try_fold(0usize, |sum, file| {
        sum.checked_add(file.bytes.len())
    }).ok_or("Review output byte count overflow")?;
    let mut logical_artifact_count = pending.len();
    for (clip, times) in REVIEW_PHASES {
        let descriptor = playback
            .clip_descriptor(clip)
            .ok_or_else(|| format!("Expected authored study clip is absent: {clip}"))?;
        let clip_duration_s = descriptor.duration_s;
        for (sample_index, time_s) in times.iter().copied().enumerate() {
            if cancel.load(Ordering::Acquire) {
                return Err("Review export cancelled before publication".into());
            }
            if !time_s.is_finite() || time_s < 0.0 || time_s > clip_duration_s {
                return Err(format!("Sample {clip}@{time_s} is outside authored duration"));
            }
            phase_count = phase_count.checked_add(1).ok_or("Phase count overflow")?;
            if phase_count > MAX_PHASES || logical_artifact_count + 3 > MAX_FILES {
                return Err("Review export exceeds the 64 sampled-phase profile".into());
            }
            // Reserve the export owner's full per-container output bound and
            // the sidecar/index caps before asking it to allocate a pose.
            let preflight = logical_retained_bytes
                .checked_add(pose_limits.max_container_bytes)
                .and_then(|n| n.checked_add(MAX_PHASE_RECEIPT_BYTES))
                .and_then(|n| n.checked_add(MAX_INDEX_BYTES))
                .ok_or("Review preflight byte count overflow")?;
            if preflight > MAX_REVIEW_BYTES {
                return Err("Next sampled phase cannot fit the 256 MiB logical review budget".into());
            }
            playback.select_clip(clip)?;
            playback.seek(sample_seconds_to_tick(time_s).map_err(str::to_owned)?)?;
            let context = playback.pose_context()?;
            let exported = playback.with_current_motion_pose(
                context,
                |frame, _current| {
                    export_frame_10d(&master, frame, pose_limits, cancel)
                        .map(|asset| (asset.bytes, asset.metadata))
                },
                cancel,
            )?;
            let (pose_bytes, pose_metadata) = exported;
            let metadata_time = pose_metadata
                .get("time_s")
                .and_then(Value::as_f64)
                .ok_or("Pose export omitted its authored sample time")?;
            let master_sha = hex(&Sha256::digest(&master));
            if (metadata_time - time_s).abs() > 0.000_001
                || pose_metadata.get("master_sha256").and_then(Value::as_str)
                    != Some(master_sha.as_str())
            {
                return Err(format!("Pose export binding differs for {clip}@{time_s}"));
            }
            let phase_json = serde_json::to_vec_pretty(&pose_metadata)
                .map_err(|e| format!("Phase receipt serialization: {e}"))?;
            if phase_json.len() > MAX_PHASE_RECEIPT_BYTES {
                return Err("Sample phase receipt exceeds its 16 KiB bound".into());
            }
            logical_retained_bytes = logical_retained_bytes
                .checked_add(pose_bytes.len())
                .and_then(|n| n.checked_add(phase_json.len()))
                .ok_or("Review output byte count overflow")?;
            if logical_retained_bytes.checked_add(MAX_INDEX_BYTES)
                .ok_or("Review output byte count overflow")? > MAX_REVIEW_BYTES {
                return Err("Sampled phase exceeds the 128 MiB logical review budget".into());
            }
            pending.push(PendingFile {
                name: format!("{clip}-{sample_index}.10d"),
                bytes: pose_bytes,
                time_s: Some(metadata_time),
            });
            pending.push(PendingFile {
                name: format!("{clip}-{sample_index}.json"),
                bytes: phase_json,
                time_s: None,
            });
            logical_artifact_count += 2;
        }
    }

    if pending.len() + 1 > MAX_FILES {
        return Err("Review export exceeds the 64-file profile".into());
    }
    let mut total = pending.iter().try_fold(0usize, |sum, file| {
        sum.checked_add(file.bytes.len())
    }).ok_or("Review export byte count overflow")?;
    if total > MAX_REVIEW_BYTES {
        return Err("Review artifact payload exceeds 128 MiB".into());
    }

    let package_record = pending
        .iter()
        .find(|file| file.name == "wallaby-motion-review.hmc")
        .ok_or("Review HMC link missing")?;
    let package_sha = hex(&Sha256::digest(&package_record.bytes));
    let package_bytes = package_record.bytes.len();
    let viewer_path = viewer_path(review_dir);
    let mut rows = Vec::new();
    let mut viewer_10d_eligible = true;
    for file in &pending {
        let digest = hex(&Sha256::digest(&file.bytes));
        let is_10d = file.name.ends_with(".10d");
        if is_10d && file.bytes.len() > VIEWER_10D_LIMIT {
            viewer_10d_eligible = false;
        }
        let review_url = if is_10d {
            viewer_path.as_ref().map(|relative| {
                format!("http://127.0.0.1:8765/viewer/showasset.html?asset=/{relative}/{}&sha256={digest}", file.name)
            })
        } else {
            None
        };
        let mut row = json!({
            "file": file.name.as_str(),
            "bytes": file.bytes.len(),
            "sha256": digest,
            "human_acceptance": "pending",
            "render_execution": "not established by export",
            "review_url": review_url,
        });
        if let Some(time_s) = file.time_s {
            row["time_s"] = json!(time_s);
        }
        rows.push(row);
    }
    if package_bytes > VIEWER_HMC_LIMIT {
        viewer_10d_eligible = false;
    }
    let file_count = rows.len() + 1;
    let index = json!({
        "schema": "ecopunk.viewer-navigation-projection/v1",
        "scope": "bounded sampled authored-motion pose snapshots; not continuous animation, acceptance, or complete species delivery",
        "manifest_kind": "viewer-navigation-projection-not-canonical-project-graph",
        "authoritative_package": {
            "file": "wallaby-motion-review.hmc",
            "sha256": package_sha,
            "bytes": package_bytes,
            "hmc_path": package_path.to_string_lossy(),
        },
        "q42_status": "Native Q42 resources remain authoritative inside the HMC; package-review.json is a read-only inspector projection.",
        "viewer_constraints": {
            "current_viewer_eligible": viewer_10d_eligible,
            "max_10d_bytes": VIEWER_10D_LIMIT,
            "max_hmc_bytes": VIEWER_HMC_LIMIT,
            "note": if viewer_10d_eligible { "All .10d and HMC outputs fit the current candidate viewer size profile." } else { "One or more .10d/HMC outputs exceed the current candidate viewer size profile; files remain in this bounded export." },
        },
        "human_acceptance": "pending",
        "render_execution": "not established by export",
        "profiles": {"film":"incomplete","open_world_game":"incomplete","cartoon":"incomplete"},
        "files": rows,
    });
    let index_bytes = serde_json::to_vec_pretty(&index)
        .map_err(|e| format!("Viewer index serialization: {e}"))?;
    if index_bytes.len() > MAX_INDEX_BYTES {
        return Err("Viewer navigation projection exceeds 256 KiB".into());
    }
    total = total.checked_add(index_bytes.len()).ok_or("Review total overflow")?;
    if total > MAX_REVIEW_BYTES {
        return Err("Complete review export exceeds 128 MiB".into());
    }

    // Nothing is published until source reopen, Q42 inspection, every pose,
    // every sidecar, index serialization and all aggregate bounds have passed.
    if cancel.load(Ordering::Acquire) {
        return Err("Review export cancelled before directory creation".into());
    }
    fs::create_dir(review_dir).map_err(|e| format!("Create review directory: {e}"))?;
    for file in pending {
        write_new(&review_dir.join(&file.name), &file.bytes)?;
    }
    write_new(&review_dir.join("review-index.json"), &index_bytes)?;
    Ok(json!({"directory": review_dir, "files": file_count, "bytes": total,
        "viewer_eligible": viewer_10d_eligible, "manifest": "review-index.json"}))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("Create-only review artifact {}: {e}", path.display()))?;
    file.write_all(bytes)
        .map_err(|e| format!("Write review artifact {}: {e}", path.display()))
}

fn viewer_path(review_dir: &Path) -> Option<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()?.parent()?.canonicalize().ok()?;
    let studies = root.join("assets/production-studies").canonicalize().ok()?;
    let parent = review_dir.parent()?.canonicalize().ok()?;
    if !parent.starts_with(&studies) {
        return None;
    }
    let target = parent.join(review_dir.file_name()?);
    let relative = target.strip_prefix(&studies).ok()?.to_string_lossy().replace('\\', "/");
    if relative.is_empty() || relative.split('/').any(|part| part.is_empty() || part == "." || part == "..")
        || !relative.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-' | '.'))
    {
        return None;
    }
    Some(format!("assets/production-studies/{relative}"))
}

fn hex(digest: &[u8]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
