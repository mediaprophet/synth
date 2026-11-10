use super::*;

/// Two independent produces of subdivide+bevel+compile_10d_pbr must byte-match
/// (ordered topology maps). Reopen of the sealed HMC must match the same digest.
#[test]
fn subdivide_bevel_compile_two_produces_match_sha256() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let source = r##"
        requires [capability("capability.invoke")];
        let profile = capability.invoke("hypermedia.locus.construct", {
            shape: "polygon",
            points: [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]]
        });
        let mesh = capability.invoke("hypermedia.manifold.extrude", {
            profile: profile,
            path: [[0.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
            caps: true
        });
        let subdivided = capability.invoke("hypermedia.manifold.subdivide_catmull_clark", {
            mesh: mesh,
            levels: 1,
            crease: 0.25
        });
        let beveled = capability.invoke("hypermedia.manifold.bevel", {
            mesh: subdivided,
            width: 0.05,
            segments: 1
        });
        let compiled = capability.invoke("hypermedia.manifold.compile_10d_pbr", {
            mesh: beveled
        });
        let asset = {files: [
            {key: "body.10d", resource: compiled}
        ]};
    "##;

    let digest_of = |source: &str| {
        let mut session = registry
            .session("files", Vec::new(), ProductionLimits::default())
            .unwrap();
        session.submit(source, &cancel).unwrap();
        let candidate = session.candidate().unwrap();
        (candidate.sha256(), candidate.bundle().to_vec())
    };

    let (sha_a, bundle_a) = digest_of(source);
    let (sha_b, bundle_b) = digest_of(source);
    assert_eq!(sha_a, sha_b);
    assert_eq!(bundle_a, bundle_b);

    let reopened = registry
        .reopen(&bundle_a, ProductionLimits::default(), &cancel)
        .unwrap();
    let replayed = reopened.candidate().unwrap();
    assert_eq!(replayed.sha256(), sha_a);
    assert_eq!(replayed.bundle(), bundle_a.as_slice());
}

/// Two produces of mixed compiled members (10d + png) must match SHA-256, and
/// the HMC bag records each entry's resource.kind (no sidecar).
#[test]
fn mixed_compiled_members_two_produces_match_sha256_and_kinds() {
    use qualia_core_db::bundle::BundleReader;

    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let source = r##"
        requires [capability("capability.invoke")];
        let profile = capability.invoke("hypermedia.locus.construct", {
            shape: "polygon",
            points: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
        });
        let mesh = capability.invoke("hypermedia.manifold.extrude", {
            profile: profile,
            path: [[0.0, 0.0, 0.0], [0.0, 0.0, 0.5]],
            caps: true
        });
        let compiled = capability.invoke("hypermedia.manifold.compile_10d", {
            mesh: mesh
        });
        let preview = capability.invoke("hypermedia.radiance.create_rgba", {
            width: 8,
            height: 8,
            color: [32, 160, 64, 255]
        });
        let previewPng = capability.invoke("hypermedia.radiance.encode_rgba_png", {
            image: preview
        });
        let asset = {files: [
            {key: "body.10d", resource: compiled},
            {key: "preview.png", resource: previewPng}
        ]};
    "##;

    let digest_of = |source: &str| {
        let mut session = registry
            .session("files", Vec::new(), ProductionLimits::default())
            .unwrap();
        session.submit(source, &cancel).unwrap();
        let candidate = session.candidate().unwrap();
        let outputs: Vec<(String, String)> = candidate
            .outputs()
            .iter()
            .map(|r| (r.key.clone(), r.kind.clone()))
            .collect();
        (candidate.sha256(), candidate.bundle().to_vec(), outputs)
    };

    let (sha_a, bundle_a, outs_a) = digest_of(source);
    let (sha_b, bundle_b, outs_b) = digest_of(source);
    assert_eq!(sha_a, sha_b);
    assert_eq!(bundle_a, bundle_b);
    assert_eq!(outs_a, outs_b);
    assert_eq!(
        outs_a,
        vec![
            ("body.10d".into(), "10d".into()),
            ("preview.png".into(), "png".into()),
        ]
    );

    let limits = ProductionLimits::default();
    let reader = BundleReader::parse_with_limits(
        &bundle_a,
        qualia_core_db::bundle::BundleReadLimits {
            file_bytes: limits.package_bytes,
            entries: limits
                .resources
                .checked_mul(2)
                .and_then(|n| n.checked_add(1))
                .expect("entry count"),
            key_bytes: 1040,
            ..qualia_core_db::bundle::BundleReadLimits::default()
        },
    )
    .expect("HMC parses with production limits");
    let ten_d = reader
        .entry("outputs/body.10d")
        .expect("bagged 10d member");
    assert_eq!(ten_d.kind, "10d");
    let png = reader
        .entry("outputs/preview.png")
        .expect("bagged png member");
    assert_eq!(png.kind, "png");
    assert!(reader.entries().iter().all(|e| reader.verify_entry(&e.key)));

    let reopened = registry
        .reopen(&bundle_a, ProductionLimits::default(), &cancel)
        .unwrap();
    let replayed = reopened.candidate().unwrap();
    assert_eq!(replayed.sha256(), sha_a);
    assert_eq!(replayed.bundle(), bundle_a.as_slice());
}

#[test]
fn wave7_locus_fillet_rmf_and_manifold_radial_euler_bevel_sdf_fspm_botany() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", Vec::new(), ProductionLimits::default())
        .unwrap();
    let source = r##"
        requires [capability("capability.invoke")];
        let baseProfile = capability.invoke("hypermedia.locus.construct", {
            shape: "rectangle",
            width: 4.0,
            height: 3.0,
            radius: 0.25,
            segments: 4
        });
        let filleted = capability.invoke("hypermedia.locus.fillet", {
            profile: baseProfile,
            radius: 0.2,
            segments: 4,
            closed: true
        });
        let resampled = capability.invoke("hypermedia.locus.resample", {
            profile: filleted,
            mode: "count",
            count: 24,
            closed: true
        });
        let trimmed = capability.invoke("hypermedia.locus.trim", {
            profile: resampled,
            start: 0.0,
            end: 0.75,
            close_chord: true
        });
        let rmf = capability.invoke("hypermedia.locus.bishop_rmf_cyclic", {
            path: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, -1.0, 0.0]],
            closed: true,
            initial_normal: [0.0, 0.0, 1.0]
        });

        let baseMesh = capability.invoke("hypermedia.manifold.extrude", {
            profile: trimmed,
            path: [[0.0, 0.0, 0.0], [0.0, 0.0, 2.0]],
            caps: false
        });
        let geodesic = capability.invoke("hypermedia.locus.shortest_edge_path", {
            mesh: baseMesh,
            start_vertex: 0,
            end_vertex: 3
        });
        let geodesicSvg = capability.invoke("hypermedia.locus.project_svg", {
            profile: geodesic.profile_2d,
            padding: 2.0
        });

        let radialShell = capability.invoke("hypermedia.manifold.radial_euler", {
            mesh: baseMesh,
            crease_sharpness: 0.5
        });
        let beveled = capability.invoke("hypermedia.manifold.bevel", {
            mesh: radialShell.mesh,
            width: 0.05,
            segments: 1
        });
        let insetMesh = capability.invoke("hypermedia.manifold.inset", {
            mesh: beveled,
            thickness: 0.15,
            depth: 0.04
        });
        let solidified = capability.invoke("hypermedia.manifold.solidify", {
            mesh: insetMesh,
            thickness: 0.08,
            offset: -1.0
        });
        let bisected = capability.invoke("hypermedia.manifold.bisect_plane", {
            mesh: solidified,
            plane_origin: [0.0, 0.0, 1.0],
            plane_normal: [0.0, 0.0, 1.0],
            clear_outer: false,
            clear_inner: false,
            fill_cap: true
        });
        let subdivided = capability.invoke("hypermedia.manifold.subdivide_catmull_clark", {
            mesh: baseMesh,
            levels: 1,
            crease: 0.25
        });
        let displacedField = capability.invoke("hypermedia.manifold.evaluate_field", {
            mesh: subdivided,
            domain: "point",
            expression: "radial_wave",
            amplitude: 0.08,
            frequency: 1.5,
            apply_displacement: true
        });
        let sdfBrick = capability.invoke("hypermedia.manifold.mesh_to_sdf_brick", {
            mesh: displacedField.mesh,
            resolution: 12,
            padding: 0.2
        });
        let dcMesh = capability.invoke("hypermedia.manifold.sdf_brick_to_mesh", {
            sdf: sdfBrick,
            iso_level: 0.0
        });
        let instanced = capability.invoke("hypermedia.manifold.instance_on_points", {
            prototype: dcMesh,
            points: [[0.0, 0.0, 0.0], [2.5, 0.0, 0.0]],
            scale: 0.5
        });
        let realized = capability.invoke("hypermedia.manifold.realize_instances", {
            instances: instanced
        });
        let botany = capability.invoke("hypermedia.manifold.fspm_vascular_grow", {
            generations: 3,
            base_radius: 0.18,
            base_length: 1.0,
            branch_angle_deg: 28.0,
            turgor_kpa: 880.0,
            gdd_accumulated: 420.0
        });
        let botany10d = capability.invoke("hypermedia.manifold.compile_10d", {
            mesh: botany.mesh
        });
        let realized10d = capability.invoke("hypermedia.manifold.compile_10d", {
            mesh: realized
        });

        let asset = {files: [
            {key: "geodesic.svg", resource: geodesicSvg},
            {key: "radial-shell.m64", resource: radialShell.shell_m64},
            {key: "brick.sdf", resource: sdfBrick},
            {key: "botany.10d", resource: botany10d},
            {key: "realized.10d", resource: realized10d}
        ]};
    "##;
    session.submit(source, &cancel).unwrap();
    let candidate = session.candidate().unwrap();
    assert_eq!(candidate.outputs().len(), 5);
    let m64_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "radial-shell.m64")
        .unwrap()
        .bytes;
    assert_eq!(&m64_bytes[..4], b"m64\0");
    let sdf_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "brick.sdf")
        .unwrap()
        .bytes;
    assert_eq!(&sdf_bytes[..8], b"Q42SDF16");
}

#[test]
fn wave8_surface_slim_mikktspace_bsdf_and_radiance_kuwahara_cryptomatte_sdfgi_light_tree() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", Vec::new(), ProductionLimits::default())
        .unwrap();
    let source = r##"
        requires [capability("capability.invoke")];
        let profile = capability.invoke("hypermedia.locus.construct", {
            shape: "polygon",
            points: [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]]
        });
        let mesh = capability.invoke("hypermedia.manifold.extrude", {
            profile: profile,
            path: [[0.0, 0.0, 0.0], [0.0, 0.0, 1.5]],
            caps: false
        });
        let slim = capability.invoke("hypermedia.surface.unwrap_slim", {
            mesh: mesh,
            iterations: 8,
            chart_padding: 0.04
        });
        let mikkt = capability.invoke("hypermedia.surface.compute_mikktspace", {
            mesh: mesh
        });
        let bsdf = capability.invoke("hypermedia.surface.compile_bsdf_graph", {
            closure: "principled_v2",
            base_color: [0.85, 0.42, 0.18],
            roughness: 0.25,
            metallic: 0.8,
            thin_film_nm: 380.0,
            preview_size: 32
        });
        let gaborField = capability.invoke("hypermedia.surface.sample_procedural_field", {
            basis: "gabor",
            width: 32,
            height: 32,
            scale: 6.0,
            seed: 42
        });
        let kuwahara = capability.invoke("hypermedia.radiance.filter_kuwahara_anisotropic", {
            image: gaborField,
            radius: 2,
            sharpness: 8.0,
            eccentricity: 1.0
        });
        let crypto = capability.invoke("hypermedia.radiance.decode_cryptomatte_q42", {
            image: kuwahara,
            target_id: "did:q42:entity:canopy",
            luma_min: 0.15,
            luma_max: 0.9,
            feather: 0.05
        });
        let blurred = capability.invoke("hypermedia.radiance.vector_motion_blur", {
            image: crypto.isolated_rgba,
            velocity_px: [3.0, 1.0],
            samples: 5,
            shutter: 0.5
        });
        let sdfgi = capability.invoke("hypermedia.radiance.solve_sdfgi_froxel", {
            image: blurred,
            cascades: 3,
            fog_density: 0.2,
            scattering_anisotropy_g: 0.4,
            sun_direction: [0.2, 0.9, 0.3],
            sky_radiance: [0.4, 0.65, 0.95],
            bounce_intensity: 0.6
        });
        let lightTree = capability.invoke("hypermedia.radiance.build_light_tree", {
            lights: [
                {position: [0.0, 4.0, 0.0], direction: [0.0, -1.0, 0.0], color: [1.0, 0.9, 0.8], power: 12.0, angle_rad: 1.0},
                {position: [2.0, 3.0, 1.0], direction: [-0.4, -0.9, 0.0], color: [0.3, 0.6, 1.0], power: 6.0, angle_rad: 1.2}
            ],
            width: 32,
            height: 32
        });
        let finalPng = capability.invoke("hypermedia.radiance.encode_rgba_png", {
            image: sdfgi.image
        });
        let lightTreePng = capability.invoke("hypermedia.radiance.encode_rgba_png", {
            image: lightTree.preview_rgba
        });

        let asset = {files: [
            {key: "uv-chart.svg", resource: slim.uv_chart_svg},
            {key: "bsdf.json", resource: bsdf.bsdf_json},
            {key: "sdfgi.png", resource: finalPng},
            {key: "light-tree.png", resource: lightTreePng}
        ]};
    "##;
    session.submit(source, &cancel).unwrap();
    let candidate = session.candidate().unwrap();
    assert_eq!(candidate.outputs().len(), 4);
    let bsdf_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "bsdf.json")
        .unwrap()
        .bytes;
    let bsdf_str = std::str::from_utf8(bsdf_bytes).unwrap();
    assert!(bsdf_str.contains("principled_v2"));
}

#[test]
fn wave9_kinetics_motor_fti_xpbd_direct_space_funnel_orca_and_vox0_dda_metamer_geodesy() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", Vec::new(), ProductionLimits::default())
        .unwrap();
    let source = r##"
        requires [capability("capability.invoke")];
        let fti = capability.invoke("hypermedia.kinetics.interpolate_motor_fti", {
            prev_position: [0.0, 0.0, 0.0],
            curr_position: [4.0, 2.0, 1.0],
            alpha: 0.5,
            teleport: false
        });
        let xpbd = capability.invoke("hypermedia.kinetics.step_island_xpbd", {
            bodies: [
                {id: "rover_chassis", position: [0.0, 1.2, 0.0], velocity: [0.5, -1.0, 0.0], radius: 0.5, inv_mass: 1.0},
                {id: "cargo_pod", position: [0.0, 2.2, 0.0], velocity: [0.0, -0.5, 0.0], radius: 0.5, inv_mass: 1.0}
            ],
            steps: 4,
            substeps: 4,
            ground_y: 0.0
        });
        let spaceHit = capability.invoke("hypermedia.kinetics.query_direct_space", {
            colliders: [
                {id: "boulder_01", center: [0.0, 1.0, 0.0], half_extents: [1.0, 1.0, 1.0], layer_mask: 1, is_static: true}
            ],
            query_mode: "intersect_ray",
            ray_origin: [0.0, 1.0, -4.0],
            ray_dir: [0.0, 0.0, 1.0],
            collision_mask: 1
        });
        let nav = capability.invoke("hypermedia.kinetics.navigate_funnel_orca", {
            start: [0.0, 0.0, 0.0],
            goal: [0.0, 0.0, 12.0],
            agent_radius: 0.4,
            max_speed: 4.5
        });
        let nla = capability.invoke("hypermedia.kinetics.evaluate_nla_channelbag", {
            strips: [
                {name: "stride", start_time: 0.0, end_time: 2.0, weight: 1.0, blend_mode: "replace", start_val: [0.0, 0.0, 0.0], end_val: [4.0, 0.0, 0.0], transition: "cubic"},
                {name: "bounce", start_time: 0.0, end_time: 2.0, weight: 0.5, blend_mode: "add", start_val: [0.0, 0.0, 0.0], end_val: [0.0, 1.0, 0.0], transition: "sine"}
            ],
            eval_time: 1.0
        });
        let simZone = capability.invoke("hypermedia.kinetics.step_simulation_zone", {
            points: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            iterations: 6,
            dt: 0.05,
            curl_strength: 0.6,
            attractor: fti.position
        });

        let dda = capability.invoke("hypermedia.electromagnetics.raymarch_vox0_dda", {
            width: 24,
            height: 24,
            deflection_mass: 0.05,
            step_limit: 32
        });
        let metamer = capability.invoke("hypermedia.electromagnetics.project_metamer_nullspace", {
            nullspace_scale: 0.6
        });
        let advected = capability.invoke("hypermedia.geodesy.advect_m64_grid", {
            grid_side: 16,
            steps: 3,
            dt: 0.08,
            poisson_iterations: 20
        });
        let continuum = capability.invoke("hypermedia.geodesy.couple_continuum_xpbd", {
            wheel_load_n: 4200.0,
            contact_width_m: 0.35,
            contact_length_m: 0.45,
            soil_kc: 15000.0,
            soil_kphi: 850000.0,
            soil_n: 0.8,
            soil_cohesion_pa: 12000.0,
            soil_friction_deg: 30.0,
            submerged_volume_m3: 0.5,
            wave_amplitude_m: 0.3
        });

        let ddaPng = capability.invoke("hypermedia.radiance.encode_rgba_png", {
            image: dda.image
        });
        let advectedPng = capability.invoke("hypermedia.radiance.encode_rgba_png", {
            image: advected.preview_rgba
        });

        let asset = {files: [
            {key: "vox0-dda.png", resource: ddaPng},
            {key: "advected-grid.png", resource: advectedPng}
        ]};
    "##;
    session.submit(source, &cancel).unwrap();
    let candidate = session.candidate().unwrap();
    assert_eq!(candidate.outputs().len(), 2);
}

#[test]
fn wave10_glyph_bidi_msdf_diegetic_layout_audio_bus_dag_fiduciary_rid_and_deterministic_movie() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", Vec::new(), ProductionLimits::default())
        .unwrap();
    let source = r##"
        requires [capability("capability.invoke")];
        let bidi = capability.invoke("hypermedia.glyph.shape_harfbuzz_bidi", {
            text: "שלום Webizen",
            font_size: 20.0
        });
        let msdf = capability.invoke("hypermedia.glyph.pack_msdf_font_atlas", {
            charset: "WEBIZEN012",
            cell_size: 24,
            px_range: 4.0
        });
        let uiBase = capability.invoke("hypermedia.radiance.create_rgba", {
            width: 32,
            height: 32,
            color: [30, 64, 175, 255]
        });
        let diegetic = capability.invoke("hypermedia.layout.project_diegetic_surface", {
            ui_image: uiBase,
            viewport_width: 256,
            viewport_height: 256,
            yaw_deg: 12.0,
            pitch_deg: -6.0,
            pick_screen_xy: [128.0, 128.0]
        });
        let routed = capability.invoke("hypermedia.layout.route_input_action", {
            event_type: "pointer_down",
            pointer_xy: [128.0, 128.0],
            diegetic_quad_bounds: [64.0, 64.0, 128.0, 128.0]
        });

        let toneA = capability.invoke("hypermedia.acoustics.synthesize_tone", {
            frequency: 440.0,
            sample_rate: 48000,
            frames: 4800,
            amplitude: 0.4
        });
        let spatialTone = capability.invoke("hypermedia.acoustics.spatialize_pga_zone", {
            audio: toneA,
            source_pos: [4.0, 0.0, 1.0],
            listener_pos: [0.0, 0.0, 0.0],
            source_vel: [-8.0, 0.0, 0.0],
            occlusion: 0.15,
            reverb_room_size: 0.4
        });
        let busMix = capability.invoke("hypermedia.acoustics.route_bus_dag", {
            buses: [
                {name: "SFX", audio: spatialTone, send_to: "Master", gain_db: -1.5, state: "playing", lowpass_hz: 16000.0, reverb_wet: 0.15},
                {name: "Master", send_to: "", gain_db: 0.0}
            ]
        });
        let masterWav = capability.invoke("hypermedia.acoustics.encode_wav", {
            audio: busMix
        });

        let telemetry = capability.invoke("hypermedia.discourse.inspect_engine_telemetry", {
            frame_count: 32,
            target_fps: 60
        });
        let fiduciary = capability.invoke("hypermedia.discourse.verify_fiduciary_rid", {
            resource_did: "did:q42:manifold:diegetic_ui",
            deontic_mode: "permit",
            sensitivity_class: 2,
            dependent_rids: ["did:q42:glyph:msdf_atlas"]
        });

        let movieMp4 = capability.invoke("hypermedia.chronicle.capture_deterministic_movie", {
            frames: [uiBase, uiBase],
            fps: 24
        });
        let syncMotors = capability.invoke("hypermedia.chronicle.sync_spatial_motors", {
            motors: [
                {entity_id: 1, position: [1.25, 0.5, -2.0], rotation: [0.0, 0.0, 0.0, 1.0], sequence: 10}
            ]
        });

        let asset = {files: [
            {key: "msdf-metrics.json", resource: msdf.metrics_json},
            {key: "spatial-master.wav", resource: masterWav},
            {key: "engine-telemetry.json", resource: telemetry.telemetry_json},
            {key: "deterministic-capture.mp4", resource: movieMp4},
            {key: "spatial-motors.bin", resource: syncMotors.packet_resource}
        ]};
    "##;
    session.submit(source, &cancel).unwrap();
    let candidate = session.candidate().unwrap();
    assert_eq!(candidate.outputs().len(), 5);
    let mp4_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "deterministic-capture.mp4")
        .unwrap()
        .bytes;
    assert_eq!(&mp4_bytes[4..8], b"ftyp");
}
