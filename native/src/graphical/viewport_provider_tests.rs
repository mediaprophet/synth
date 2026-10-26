use super::*;
use crate::graphical::{
    geometry_host::GeometryHost,
    preview::{PreviewCamera, PreviewLimits},
};
use qualia_core_db::{
    container_10d::MaterialRecord,
    poet_host::invoke::studio_store::{self, StudioArtifact},
};
use vibe::{
    session::{SessionLimits, SourceSession},
    Budget,
};

const SOURCE: &str = r#"
let m={id:1,colour_space:"linear",base_color:[0.8,0.15,0.08,1.0],emissive:[0.0,0.0,0.0],metallic:0.1,roughness:0.5,specular:1.0,alpha_cutoff:0.5,opacity_mode:"Opaque",double_sided:true};
let tri={vertices:[[-1.0,-1.0,0.0],[1.0,-1.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]]};
let mut poet_parts=[{id:"rear",semantic_id:101,geometry:tri,material:m,translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]},{id:"front",semantic_id:102,geometry:tri,material:m,translation:[0.0,0.0,1.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}];
({parts:poet_parts});
"#;
fn provider() -> InteractivePreviewProvider {
    InteractivePreviewProvider::new(MeshPreviewProvider {
        material: MaterialRecord::legacy_default(),
        camera: PreviewCamera {
            yaw: 0.0,
            pitch: 0.0,
            distance: 4.0,
            target: [0.0; 3],
        },
        limits: PreviewLimits {
            side: 64,
            vertices: 64,
            indices: 128,
        },
    })
    .unwrap()
}
fn make_source(label: &str) -> SourceRef {
    let id = format!("urn:poet:r9:provider-test:{}:{label}", std::process::id());
    let (_, stamp) = studio_store::create_artifact_if_absent(StudioArtifact::new(
        &id,
        "Geometry",
        "provider calibration",
    ))
    .unwrap();
    SourceRef {
        artifact_id: id,
        stamp,
    }
}

#[test]
fn viewport_provider_cancel_and_stale_source_do_not_create_device() {
    let mut provider = provider();
    let source = make_source("cancel");
    let cancelled = AtomicBool::new(true);
    assert!(provider
        .viewport(
            ViewportCommand::FrameAll,
            source.clone(),
            &Value::Null,
            None,
            &cancelled
        )
        .is_err());
    assert_eq!(provider.counters().device_creations, 0);
    cancelled.store(false, Ordering::Release);
    studio_store::mutate_artifact_stamped(&source.artifact_id, &source.stamp, |a| {
        a.title = "changed".into();
        Ok(())
    })
    .unwrap();
    assert!(provider
        .viewport(
            ViewportCommand::FrameAll,
            source,
            &Value::Null,
            None,
            &cancelled
        )
        .is_err());
    assert_eq!(provider.counters().device_creations, 0);
}

/// Native headless provider, actual Vibe evaluation/assembly admission/Portal readback.
/// No GUI, human operation or asset-completion acceptance is established here.
#[test]
#[ignore = "requires explicit native GPU qualification"]
fn r9_native_gpu_provider_automatic_preview_and_raw_parts_gestures_share_device_upload() {
    let mut session = SourceSession::new(Vec::new(), SessionLimits::default());
    let mut host = GeometryHost::new(Vec::new(), 200_000);
    let evaluated = session
        .submit(SOURCE, &mut host, Budget::default())
        .unwrap();
    let source = make_source("gpu");
    let epoch = SessionEpoch::new();
    let mut provider = provider();
    provider.set_execution_context(&epoch);
    assert!(!provider.viewport_cached(&source));
    let frame = provider
        .preview_with_resources(source.clone(), &evaluated.value, None)
        .unwrap();
    assert!(provider.viewport_cached(&source));
    assert!(frame.execution_epoch.as_ref().is_some_and(|e| e == &epoch));
    assert_eq!(
        provider.counters(),
        ViewportCounters {
            device_creations: 1,
            scene_uploads: 1,
            frames: 1
        }
    );
    let raw = session.value("poet_parts").unwrap();
    let cancel = AtomicBool::new(false);
    let zoomed = provider
        .viewport(
            ViewportCommand::Zoom { steps: 1.0 },
            source.clone(),
            raw,
            None,
            &cancel,
        )
        .unwrap();
    let selected = provider
        .viewport(
            ViewportCommand::Pick {
                frame_id: zoomed.state.frame_id,
                x: 32.0,
                y: 32.0,
            },
            source.clone(),
            raw,
            None,
            &cancel,
        )
        .unwrap();
    assert_eq!(selected.state.selected_part.as_deref(), Some("front"));
    assert!(selected
        .frame
        .rgba8
        .chunks_exact(4)
        .any(|p| p == [255, 194, 45, 255]));
    assert_eq!(
        provider.counters(),
        ViewportCounters {
            device_creations: 1,
            scene_uploads: 1,
            frames: 3
        }
    );
    // Prove a gesture cache hit does not need a copied/admitted value at all.
    // Canonical host retains the actual source; this call supplies a deliberately irrelevant value.
    let moved = provider
        .viewport(
            ViewportCommand::Pan { dx: 1.0, dy: 0.0 },
            source.clone(),
            &Value::Null,
            None,
            &cancel,
        )
        .unwrap();
    assert!(moved.state.frame_id > selected.state.frame_id);
    assert_eq!(provider.counters().scene_uploads, 1);
    // An accepted successor source must re-upload once while preserving the user view.
    let (_, stamp) =
        studio_store::mutate_artifact_stamped(&source.artifact_id, &source.stamp, |a| {
            a.title = "accepted successor viewport source".into();
            Ok(())
        })
        .unwrap();
    let source = SourceRef {
        artifact_id: source.artifact_id.clone(),
        stamp,
    };
    provider
        .preview_with_resources(source.clone(), &evaluated.value, None)
        .unwrap();
    assert_eq!(provider.counters().device_creations, 1);
    assert_eq!(provider.counters().scene_uploads, 2);
    let successor = provider
        .viewport(ViewportCommand::Redraw, source.clone(), raw, None, &cancel)
        .unwrap();
    assert_eq!(successor.state.camera.yaw, moved.state.camera.yaw);
    assert_eq!(successor.state.camera.pitch, moved.state.camera.pitch);
    assert_eq!(successor.state.camera.distance, moved.state.camera.distance);
    assert_eq!(successor.state.camera.target, moved.state.camera.target);
    assert!(successor.state.selected_part.is_none());
    let retired_frame = successor.state.frame_id;
    provider.set_execution_context(&epoch);
    assert!(!provider.viewport_cached(&source));
    assert!(provider
        .viewport(
            ViewportCommand::Pick {
                frame_id: retired_frame,
                x: 32.0,
                y: 32.0
            },
            source.clone(),
            raw,
            None,
            &cancel
        )
        .is_err());
    assert_eq!(provider.counters().device_creations, 1);
    assert_eq!(provider.counters().scene_uploads, 3);
    let redrawn = provider
        .viewport(ViewportCommand::Redraw, source, raw, None, &cancel)
        .unwrap();
    assert!(redrawn.state.frame_id > retired_frame);
    // Reuse the same device for the prior bare-geometry compatibility path.
    // Core generates the normal frames; no synthetic assembly-part identifier is exposed.
    let bare_source = make_source("bare-gpu");
    let bare = session.value("tri").unwrap();
    provider.preview(bare_source.clone(), bare).unwrap();
    let bare_zoom = provider
        .viewport(
            ViewportCommand::Zoom { steps: -1.0 },
            bare_source.clone(),
            bare,
            None,
            &cancel,
        )
        .unwrap();
    assert!(bare_zoom.state.selected_part.is_none());
    assert_eq!(provider.counters().device_creations, 1);
    assert_eq!(provider.counters().scene_uploads, 4);
    assert!(provider
        .viewport(
            ViewportCommand::Pick {
                frame_id: bare_zoom.state.frame_id,
                x: 32.0,
                y: 32.0
            },
            bare_source,
            bare,
            None,
            &cancel
        )
        .is_err());
}

#[test]
fn viewport_view_lighting_presets_reuse_canonical_bounds_without_gpu() {
    use crate::graphical::viewport::ViewLighting;
    ViewLighting::existing_scene().validate().unwrap();
    ViewLighting::neutral_studio().validate().unwrap();
    let mut invalid = ViewLighting::neutral_studio();
    invalid.exposure_ev = f32::NAN;
    assert!(invalid.validate().is_err());
    invalid = ViewLighting::neutral_studio();
    invalid.exposure_ev = 9.0;
    assert!(invalid.validate().is_err());
    invalid = ViewLighting::neutral_studio();
    invalid.sun_direction = [0.0; 3];
    assert!(invalid.validate().is_err());
    invalid = ViewLighting::neutral_studio();
    invalid.ambient_intensity = -1.0;
    let mut provider = provider();
    let source = make_source("invalid-lighting");
    let error = provider
        .viewport(
            invalid.command(),
            source,
            &Value::Null,
            None,
            &AtomicBool::new(false),
        )
        .err()
        .unwrap();
    assert!(error.contains("render settings"));
    assert_eq!(provider.counters().device_creations, 0);
}

/// Explicit headless GPU evidence for view-only look development, not quality acceptance.
#[test]
#[ignore = "requires explicit native GPU qualification"]
fn r9_native_gpu_view_lighting_changes_pixels_without_upload_or_device_recreation() {
    use crate::graphical::viewport::ViewLighting;
    let mut session = SourceSession::new(Vec::new(), SessionLimits::default());
    let mut host = GeometryHost::new(Vec::new(), 200_000);
    let evaluated = session
        .submit(SOURCE, &mut host, Budget::default())
        .unwrap();
    let source = make_source("gpu-view-lighting");
    let cancel = AtomicBool::new(false);
    let mut provider = provider();
    provider.set_execution_context(&SessionEpoch::new());
    let before = provider
        .preview_with_resources(source.clone(), &evaluated.value, None)
        .unwrap();
    let after = provider
        .viewport(
            ViewLighting::neutral_studio().command(),
            source.clone(),
            &Value::Null,
            None,
            &cancel,
        )
        .unwrap();
    assert_eq!(
        provider.counters(),
        ViewportCounters {
            device_creations: 1,
            scene_uploads: 1,
            frames: 2
        }
    );
    let center = (32 * 64 + 32) * 4;
    let difference: u32 = before.rgba8[center..center + 3]
        .iter()
        .zip(&after.frame.rgba8[center..center + 3])
        .map(|(a, b)| u32::from(a.abs_diff(*b)))
        .sum();
    assert!(
        difference >= 10,
        "view lighting must visibly change a covered interior pixel; RGB delta={difference}"
    );
    let changed = before
        .rgba8
        .chunks_exact(4)
        .zip(after.frame.rgba8.chunks_exact(4))
        .filter(|(a, b)| a[..3] != b[..3])
        .count();
    assert!(
        changed > 32,
        "view lighting must change an area of rendered geometry"
    );
    let reset = provider
        .viewport(
            ViewLighting::existing_scene().command(),
            source,
            &Value::Null,
            None,
            &cancel,
        )
        .unwrap();
    assert_eq!(provider.counters().scene_uploads, 1);
    assert_eq!(provider.counters().device_creations, 1);
    assert_ne!(after.frame.rgba8, reset.frame.rgba8);
}
