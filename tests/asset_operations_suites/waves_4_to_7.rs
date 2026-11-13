use super::*;

#[test]
fn wave3_manifold_pbr_10d_electromagnetics_emf_and_geodesy_m64_pack() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let obj_text = b"v -1.0 -1.0 0.0\nv 1.0 -1.0 0.0\nv 1.0 1.0 0.0\nv -1.0 1.0 0.0\nv 0.0 0.0 2.0\nf 1 2 5\nf 2 3 5\nf 3 4 5\nf 4 1 5\nf 1 4 3\nf 1 3 2\n".to_vec();
    let mut session = registry
        .session(
            "files",
            vec![Resource {
                key: "pyramid.obj".into(),
                kind: "obj".into(),
                bytes: obj_text,
            }],
            ProductionLimits::for_quality_profile("production16").unwrap(),
        )
        .unwrap();
    let source = r#"
        requires [capability("capability.invoke")];
        let imported = capability.invoke("hypermedia.manifold.import_mesh", {
            input: "pyramid.obj",
            format: "obj"
        });
        let baseCircle = capability.invoke("hypermedia.locus.construct", {
            shape: "circle",
            radius: 1.5,
            segments: 16
        });
        let pillar = capability.invoke("hypermedia.manifold.extrude", {
            profile: baseCircle,
            path: [[0, 0, -1], [0, 0, 1]],
            caps: true
        });
        let unionMesh = capability.invoke("hypermedia.manifold.boolean_3d", {
            a: pillar,
            b: imported,
            op: "union",
            attribute_policy: "discard"
        });
        let simplified = capability.invoke("hypermedia.manifold.simplify", {
            mesh: unionMesh,
            target_triangles: 24,
            max_error: 0.5
        });
        let dxfDrawing = capability.invoke("hypermedia.manifold.project_dxf", {
            profile: baseCircle,
            mesh: simplified,
            layer: "CAD_MANIFOLD"
        });
        let pbr = capability.invoke("hypermedia.surface.synthesize_maps", {
            side: 32, seed: 11, tint: [0.85, 0.9, 1.0], contrast: 0.3, relief: 0.002,
            bands: [6, 2], band_strength: 0.5, roughness: 0.5, metallic: 0.3
        });
        let pbr10d = capability.invoke("hypermedia.manifold.compile_10d_pbr", {
            mesh: simplified,
            albedo: pbr.colour,
            topology: true,
            spatial_index: true
        });

        let emfVol = capability.invoke("hypermedia.electromagnetics.compile_emf", {
            origin: [0.0, 0.0, 0.0],
            voxel_size: 0.25,
            grid_dims: [2, 2, 1],
            epoch_s: 1700000000.0,
            sources: [
                {
                    position: [1.0, 1.0, 0.5],
                    e_amplitude: [120.0, 40.0, 0.0],
                    wavelength_nm: 532.0,
                    phase: 0.0,
                    stokes: [0.5, 0.2, 0.1]
                },
                {
                    position: [2.5, 2.0, 0.5],
                    e_amplitude: [0.0, 90.0, 30.0],
                    wavelength_nm: 650.0,
                    phase: 1.57,
                    stokes: [-0.4, 0.1, 0.0]
                }
            ]
        });
        let emfCheck = capability.invoke("hypermedia.electromagnetics.verify_emf", {
            emf: emfVol
        });
        let emfSliceRgba = capability.invoke("hypermedia.electromagnetics.render_spectral_slice", {
            emf: emfVol,
            axis: "z",
            slice_index: 2,
            width: 64,
            height: 64,
            exposure: 100.0
        });
        let emfSlicePng = capability.invoke("hypermedia.radiance.encode_rgba_png", {
            image: emfSliceRgba
        });

        let terrain = capability.invoke("hypermedia.geodesy.sculpt_terrain", {
            tile_resolution: 32,
            cell_spacing_m: 2.0,
            base_height_m: 350.0,
            strokes: [
                {op: "raise", x: 32.0, y: 32.0, radius: 20.0, strength: 25.0, falloff: "gaussian"},
                {op: "terrace", x: 32.0, y: 32.0, radius: 16.0, strength: 0.8, falloff: "smoothstep", step_m: 5.0}
            ],
            width: 64,
            height: 64
        });
        let reliefPng = capability.invoke("hypermedia.radiance.encode_rgba_png", {
            image: terrain.relief_rgba
        });
        let m64Brick = capability.invoke("hypermedia.geodesy.compile_m64", {
            origin_lat_lon: [-35.78, 148.29],
            base_elevation_m: 350.0,
            grid_side: 8,
            strokes: [
                {op: "raise", x: 4.0, y: 4.0, radius: 3.0, strength: 12.0, falloff: "smoothstep"}
            ]
        });
        let m64Check = capability.invoke("hypermedia.geodesy.verify_m64", {
            m64: m64Brick
        });

        let asset = {files: [
            {key: "assembly.10d", resource: pbr10d},
            {key: "assembly.dxf", resource: dxfDrawing},
            {key: "field.emf", resource: emfVol},
            {key: "field-slice.png", resource: emfSlicePng},
            {key: "terrain.m64", resource: m64Brick},
            {key: "terrain-relief.png", resource: reliefPng}
        ]};
    "#;
    session.submit(source, &cancel).unwrap();
    let candidate = session.candidate().unwrap();
    assert_eq!(candidate.outputs().len(), 6);
    let emf_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "field.emf")
        .unwrap()
        .bytes;
    assert_eq!(&emf_bytes[..8], b"Q42EMF64");
    let m64_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "terrain.m64")
        .unwrap()
        .bytes;
    assert_eq!(&m64_bytes[..4], b"m64\0");
}

#[test]
fn capped_circle_and_pyramid_union_replays_without_centroid_slivers() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let pyramid = b"v -1.0 -1.0 0.0\nv 1.0 -1.0 0.0\nv 1.0 1.0 0.0\nv -1.0 1.0 0.0\nv 0.0 0.0 2.0\nf 1 2 5\nf 2 3 5\nf 3 4 5\nf 4 1 5\nf 1 4 3\nf 1 3 2\n".to_vec();
    let mut session = registry.session("files", vec![Resource {key:"pyramid.obj".into(), kind:"obj".into(), bytes:pyramid}], ProductionLimits::for_quality_profile("production16").unwrap()).unwrap();
    let source = r#"
        requires [capability("capability.invoke")];
        let pyramid=capability.invoke("hypermedia.manifold.import_mesh",{input:"pyramid.obj",format:"obj"});
        let circle=capability.invoke("hypermedia.locus.construct",{shape:"circle",radius:1.5,segments:16});
        let pillar=capability.invoke("hypermedia.manifold.extrude",{profile:circle,path:[[0,0,-1],[0,0,1]],caps:true});
        let joined=capability.invoke("hypermedia.manifold.boolean_3d",{a:pillar,b:pyramid,op:"union",attribute_policy:"discard"});
        let sealed=capability.invoke("hypermedia.manifold.compile_10d",{mesh:joined});
        let asset={files:[{key:"union.10d",resource:sealed}]};
    "#;
    session.submit(source, &cancel).unwrap();
    let candidate = session.candidate().unwrap();
    assert_eq!(candidate.outputs()[0].kind, "10d");
    let reopened = registry.reopen(candidate.bundle(), ProductionLimits::for_quality_profile("production16").unwrap(), &cancel).unwrap();
    assert_eq!(reopened.candidate().unwrap().bundle(), candidate.bundle());
}

#[test]
fn wave4_kinetics_ik_lottie_acoustics_biquad_mix_aac_and_chronicle_mp4_roundtrip() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", Vec::new(), ProductionLimits::default())
        .unwrap();
    let source = r#"
        requires [capability("capability.invoke")];
        let ikSolve = capability.invoke("hypermedia.kinetics.solve_two_bone_ik", {
            root: [0.0, 0.0, 0.0],
            target: [0.0, 1.2, 0.8],
            pole: [0.0, 1.0, 1.0],
            upper_length: 1.0,
            lower_length: 1.0
        });
        let kfCurve = capability.invoke("hypermedia.kinetics.interpolate_keyframes", {
            keyframes: [
                {time: 0.0, value: [64.0, 128.0, 0.0]},
                {time: 0.5, value: [192.0, 128.0, 0.0]}
            ],
            fps: 24,
            duration: 0.5,
            easing: "smoothstep",
            property: "position"
        });
        let lottieDoc = capability.invoke("hypermedia.kinetics.export_lottie", {
            width: 256,
            height: 256,
            fps: 24,
            frames: 48,
            name: "ik_motion_preview",
            shapes: [
                {
                    name: "effector_node",
                    color: [0.15, 0.75, 0.95, 1.0],
                    start_pos: [64.0, 128.0],
                    end_pos: [192.0, 128.0],
                    size: [48.0, 48.0]
                }
            ]
        });

        let toneA = capability.invoke("hypermedia.acoustics.synthesize_tone", {
            frequency: 440.0,
            sample_rate: 48000,
            frames: 12000,
            amplitude: 0.6
        });
        let toneB = capability.invoke("hypermedia.acoustics.synthesize_tone", {
            frequency: 880.0,
            sample_rate: 48000,
            frames: 12000,
            amplitude: 0.4
        });
        let filteredA = capability.invoke("hypermedia.acoustics.filter_biquad", {
            audio: toneA,
            mode: "lowpass",
            cutoff_hz: 1800.0,
            q: 0.707
        });
        let eqB = capability.invoke("hypermedia.acoustics.filter_biquad", {
            audio: toneB,
            mode: "peaking",
            cutoff_hz: 900.0,
            q: 1.2,
            gain_db: 3.0
        });
        let mixedStem = capability.invoke("hypermedia.acoustics.mix_tracks", {
            tracks: [
                {audio: filteredA, gain: 0.85, offset_seconds: 0.0, fade_in_seconds: 0.02, fade_out_seconds: 0.02},
                {audio: eqB, gain: 0.55, offset_seconds: 0.02, fade_in_seconds: 0.01, fade_out_seconds: 0.02}
            ],
            master_gain: 0.9
        });
        let aacStream = capability.invoke("hypermedia.acoustics.encode_aac", {
            audio: mixedStem,
            bitrate: 128000
        });

        let frame0 = capability.invoke("hypermedia.radiance.create_rgba", {
            width: 32,
            height: 32,
            color: [28, 52, 104, 255]
        });
        let frame1 = capability.invoke("hypermedia.radiance.create_rgba", {
            width: 32,
            height: 32,
            color: [52, 204, 128, 255]
        });
        let mp4Clip = capability.invoke("hypermedia.chronicle.mux_mp4", {
            frames: [frame0, frame1],
            fps: 24
        });
        let decodedFrame1 = capability.invoke("hypermedia.chronicle.decode_mp4_frame", {
            mp4: mp4Clip,
            frame_index: 1
        });
        let decodedPng = capability.invoke("hypermedia.radiance.encode_rgba_png", {
            image: decodedFrame1
        });

        let asset = {files: [
            {key: "motion.lottie.json", resource: lottieDoc},
            {key: "stem.aac", resource: aacStream},
            {key: "clip.mp4", resource: mp4Clip},
            {key: "clip-frame1.png", resource: decodedPng}
        ]};
    "#;
    session.submit(source, &cancel).unwrap();
    let candidate = session.candidate().unwrap();
    assert_eq!(candidate.outputs().len(), 4);
    let lottie_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "motion.lottie.json")
        .unwrap()
        .bytes;
    let lottie_str = std::str::from_utf8(lottie_bytes).unwrap();
    assert!(lottie_str.contains("\"v\":\"5.7.4\""));
    let aac_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "stem.aac")
        .unwrap()
        .bytes;
    assert!(aac_bytes.len() > 7);
    assert_eq!(aac_bytes[0], 0xFF);
    assert_eq!(aac_bytes[1] & 0xF0, 0xF0);
    let mp4_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "clip.mp4")
        .unwrap()
        .bytes;
    assert_eq!(&mp4_bytes[4..8], b"ftyp");
}

#[test]
fn wave5_discourse_pdf_docx_pptx_and_chart_svg_pack() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", Vec::new(), ProductionLimits::default())
        .unwrap();
    let source = r##"
        requires [capability("capability.invoke")];
        let pdfDoc = capability.invoke("hypermedia.discourse.compile_pdf", {
            title: "QualiaDB Categorical Ontology Specification",
            author: "Synth + Poet Unified Engine",
            width_pt: 612.0,
            height_pt: 792.0,
            pages: [
                {
                    heading: "1. Objects and Morphisms",
                    lines: [
                        "Every asset production capability is a morphism f: Domain -> Codomain.",
                        "Zero -craft legacy names remain in canonical operation identifiers."
                    ],
                    rects: [
                        {x: 72.0, y: 640.0, width: 468.0, height: 4.0, fill_rgb: [0.23, 0.51, 0.96]}
                    ]
                }
            ]
        });

        let docxDoc = capability.invoke("hypermedia.discourse.compile_prose_docx", {
            title: "Webizen Studio Design Brief",
            sections: [
                {
                    heading: "Architecture Summary",
                    body: "Shared asset-production-vibe morphisms power both Synth and Poet."
                }
            ],
            table: [
                ["Category", "Native Container", "Morphism Count"],
                ["manifold", ".10d", "9"],
                ["electromagnetics", ".emf (Q42EMF64)", "3"],
                ["geodesy", ".m64 (m64\\0)", "3"]
            ]
        });

        let pptxDeck = capability.invoke("hypermedia.discourse.compile_deck_pptx", {
            title: "Hypermedia Production Review Deck",
            slides: [
                {
                    title: "Category-Theoretic Asset Pipeline",
                    bullets: [
                        "12 Canonical Categories across 2D, 3D, Spectral, Audio, and Discourse",
                        "68 Admitted Morphisms with 204 NQuin48 Parity Triples",
                        "Direct Etalon & QualiaDB Kernel Delegation"
                    ]
                }
            ]
        });

        let chartSvg = capability.invoke("hypermedia.discourse.render_chart_svg", {
            chart_type: "bar",
            title: "Morphism Distribution by Wave",
            width: 640,
            height: 360,
            labels: ["Wave 0", "Wave 1", "Wave 2", "Wave 3", "Wave 4", "Wave 5"],
            series: [
                {
                    name: "Cumulative Morphisms",
                    values: [26.0, 38.0, 45.0, 56.0, 64.0, 68.0],
                    color: "#38bdf8"
                }
            ]
        });

        let asset = {files: [
            {key: "ontology-spec.pdf", resource: pdfDoc},
            {key: "design-brief.docx", resource: docxDoc},
            {key: "review-deck.pptx", resource: pptxDeck},
            {key: "wave-chart.svg", resource: chartSvg}
        ]};
    "##;
    session.submit(source, &cancel).unwrap();
    let candidate = session.candidate().unwrap();
    assert_eq!(candidate.outputs().len(), 4);
    let pdf_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "ontology-spec.pdf")
        .unwrap()
        .bytes;
    assert!(pdf_bytes.starts_with(b"%PDF-1.7"));
    let docx_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "design-brief.docx")
        .unwrap()
        .bytes;
    assert_eq!(&docx_bytes[..4], b"PK\x03\x04");
    let pptx_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "review-deck.pptx")
        .unwrap()
        .bytes;
    assert_eq!(&pptx_bytes[..4], b"PK\x03\x04");
    let svg_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "wave-chart.svg")
        .unwrap()
        .bytes;
    let svg_str = std::str::from_utf8(svg_bytes).unwrap();
    assert!(svg_str.contains("<svg"));
    assert!(svg_str.contains("Morphism Distribution by Wave"));
}
