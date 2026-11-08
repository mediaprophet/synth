use super::*;

#[test]
fn shared_libraries_produce_native_assets_edit_and_replay_exactly() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", vec![], ProductionLimits::default())
        .unwrap();
    session.submit(EXAMPLE, &cancel).unwrap();
    let candidate = session.candidate().unwrap();
    let outputs = candidate.outputs();
    assert_eq!(outputs.len(), 9);
    let bytes = |key: &str| {
        outputs
            .iter()
            .find(|r| r.key == key)
            .unwrap()
            .bytes
            .as_slice()
    };
    let (rgb, w, h) = decode_png(bytes("frames/0000.png")).unwrap();
    assert_eq!((w, h), (256, 192));
    assert_eq!(&rgb[..3], &[18, 25, 36]);
    let i = (96 * 256 + 88) * 3;
    assert_eq!(&rgb[i..i + 3], &[248, 214, 31]);
    let (next, _, _) = decode_png(bytes("frames/0001.png")).unwrap();
    assert_ne!(rgb, next);
    // At 24 fps and 240 pixels/second, frame one moves the disc ten pixels.
    let moving_edge = (96 * 256 + 65) * 3;
    assert_eq!(&rgb[moving_edge..moving_edge + 3], &[248, 214, 31]);
    assert_eq!(&next[moving_edge..moving_edge + 3], &[36, 154, 180]);
    let (processed, _, _) = decode_png(bytes("processed.png")).unwrap();
    assert_ne!(rgb, processed);
    let (mask, _, _) = decode_png(bytes("mask.png")).unwrap();
    assert!(mask.contains(&255));
    assert!(mask.contains(&0));
    for key in ["extrusion.10d", "revolution.10d", "loft.10d"] {
        let mesh = decode_10d_mesh(bytes(key)).unwrap();
        assert!(!mesh.positions.is_empty());
        assert!(!mesh.triangles.is_empty());
        assert!(mesh
            .triangles
            .iter()
            .flatten()
            .all(|i| (*i as usize) < mesh.positions.len()));
        assert!(mesh.positions.iter().flatten().all(|v| v.is_finite()));
        assert!(mesh.min.iter().zip(mesh.max).all(|(a, b)| *a <= b));
        if key == "extrusion.10d" {
            let mut edges = std::collections::BTreeMap::new();
            for [a, b, c] in &mesh.triangles {
                for (u, v) in [(*a, *b), (*b, *c), (*c, *a)] {
                    *edges.entry((u.min(v), u.max(v))).or_insert(0usize) += 1;
                }
            }
            assert!(edges.values().all(|count| *count == 2));
            assert_eq!(
                mesh.positions.len() as isize - edges.len() as isize
                    + mesh.triangles.len() as isize,
                2
            );
        }
    }
    let wave = bytes("tone.wav");
    assert_eq!(&wave[..4], b"RIFF");
    assert_eq!(&wave[8..12], b"WAVE");
    assert_eq!(wave.len(), 44 + 12000 * 2);
    assert_eq!(u32::from_le_bytes(wave[24..28].try_into().unwrap()), 48000);
    let samples: Vec<_> = wave[44..]
        .chunks_exact(2)
        .map(|p| i16::from_le_bytes([p[0], p[1]]))
        .collect();
    assert!(samples.iter().any(|s| *s > 6000));
    assert!(samples.iter().all(|s| s.abs() < 6600));
    assert!(std::str::from_utf8(bytes("profile.svg"))
        .unwrap()
        .contains("<polygon"));
    let reopened = registry
        .reopen(candidate.bundle(), ProductionLimits::default(), &cancel)
        .unwrap();
    assert_eq!(reopened.candidate().unwrap().bundle(), candidate.bundle());
    let before = candidate.sha256();
    let edit=format!("{GRANT} let newer = capability.invoke(\"synth.profile.svg\", {{profile: disc}}); asset = {{files:[{{key:\"disc.svg\",resource:newer}}]}};");
    let edited = session.prepare_edit(&edit, &cancel).unwrap();
    assert_ne!(edited.candidate().unwrap().sha256(), before);
    assert_eq!(session.candidate().unwrap().sha256(), before);
    registry
        .reopen(
            edited.candidate().unwrap().bundle(),
            ProductionLimits::default(),
            &cancel,
        )
        .unwrap();
}

#[test]
fn retained_png_is_decoded_transformed_packaged_and_replayed() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let rgb = [32, 64, 128, 200, 150, 100];
    let view =
        qualia_core_db::specialized_libs::computer_vision::cv::RgbView::new(2, 1, 6, &rgb).unwrap();
    let png =
        qualia_core_db::specialized_libs::computer_vision::cv::codecs::png_encode::encode_png(view)
            .unwrap();
    let mut session = registry
        .session(
            "files",
            vec![Resource {
                key: "input.png".into(),
                kind: "png".into(),
                bytes: png,
            }],
            ProductionLimits::default(),
        )
        .unwrap();
    session
        .submit(
            &format!(
                r#"{GRANT}
        let imported = capability.invoke("synth.image.import_png", {{input:"input.png"}});
        let brighter = capability.invoke("synth.image.exposure", {{image:imported,stops:1}});
        let encoded = capability.invoke("synth.image.png", {{image:brighter}});
        let asset = {{files:[{{key:"output.png",resource:encoded}}]}};"#
            ),
            &cancel,
        )
        .unwrap();
    let output = &session.candidate().unwrap().outputs()[0];
    let (out, w, h) = decode_png(&output.bytes).unwrap();
    assert_eq!((w, h), (2, 1));
    assert!(out.iter().zip(rgb).all(|(new, old)| *new > old));
    registry
        .reopen(
            session.candidate().unwrap().bundle(),
            ProductionLimits::default(),
            &cancel,
        )
        .unwrap();
}

#[test]
fn failed_effects_roll_back_artifacts_and_preserve_prior_candidate() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", vec![], ProductionLimits::default())
        .unwrap();
    session.submit(&format!(r#"{GRANT} let p=capability.invoke("synth.profile.create",{{shape:"circle",radius:2}}); let s=capability.invoke("synth.profile.svg",{{profile:p}}); let mut asset={{files:[{{key:"a.svg",resource:s}}]}};"#),&cancel).unwrap();
    let before = session.candidate().unwrap().sha256();
    for bad in [
        r#"let p2=capability.invoke("synth.profile.create",{shape:"circle",radius:3}); asset={files:[{key:"../escape.svg",resource:s}]};"#,
        r#"capability.invoke("filesystem.write",{});"#,
        r#"capability.invoke("synth.profile.svg",{profile:{handle:99,type:"profile"}});"#,
        r#"capability.invoke("synth.profile.svg",{profile:{handle:0,type:"image"}});"#,
        r#"capability.invoke("synth.image.render",{width:65536,height:2,layers:[]});"#,
        r#"capability.invoke("synth.audio.tone",{frequency:48000});"#,
        r#"capability.invoke("synth.timeline.frames",{numerator:1,denominator:1000000,count:256});"#,
        r#"capability.invoke("synth.profile.create",{shape:"circle",radius:2,typo:1});"#,
        r#"let concave=capability.invoke("synth.profile.create",{shape:"polygon",points:[[0,0],[4,0],[2,1],[4,4],[0,4]]}); capability.invoke("synth.mesh.extrude",{profile:concave,path:[[0,0,0],[0,0,2]],caps:true});"#,
    ] {
        assert!(
            session.submit(&format!("{GRANT}{bad}"), &cancel).is_err(),
            "{bad}"
        );
        assert_eq!(session.revision(), 1);
        assert_eq!(session.candidate().unwrap().sha256(), before);
    }
    // A failed cell must not leave an entry at the next handle index.
    assert!(session.submit(&format!(r#"{GRANT} capability.invoke("synth.profile.svg",{{profile:{{handle:2,type:"profile"}}}});"#),&cancel).is_err());
    cancel.store(true, Ordering::Release);
    assert!(session.submit("let no=1;", &cancel).is_err());
    assert_eq!(session.revision(), 1);
    let limits = ProductionLimits {
        asset_work_units: 10,
        ..ProductionLimits::default()
    };
    let mut bounded = registry.session("files", vec![], limits).unwrap();
    cancel.store(false, Ordering::Release);
    assert!(bounded
        .submit(
            &format!(r#"{GRANT} capability.invoke("synth.profile.create",{{shape:"circle"}});"#),
            &cancel
        )
        .is_err());
    assert_eq!(bounded.revision(), 0);
}

#[test]
fn categorical_morphisms_render_vector_compose_rgba_synthesize_surface_and_emit_gallery() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", vec![], ProductionLimits::default())
        .unwrap();
    let source = r#"
        requires [capability("capability.invoke")];
        let pbr = capability.invoke("hypermedia.surface.synthesize_maps", {
            side: 64, seed: 7, tint: [0.9, 0.95, 1.0], contrast: 0.25, relief: 0.003,
            bands: [8, 2], band_strength: 0.6, roughness: 0.4, metallic: 0.2
        });
        let badge = capability.invoke("hypermedia.radiance.render_vector", {
            width: 64, height: 64,
            layers: [
                {
                    path: "M 8 8 L 56 8 L 56 56 L 8 56 Z",
                    fill: {start: [8, 8], end: [56, 56], colors: [[24, 140, 210, 220], [12, 60, 120, 220]]},
                    stroke: [255, 255, 255, 255],
                    stroke_width: 2
                }
            ]
        });
        let filtered = capability.invoke("hypermedia.radiance.filter_rgba", {
            image: badge, operation: "sharpen", sigma: 1.0, amount: 0.5
        });
        let comp = capability.invoke("hypermedia.radiance.compose_rgba", {
            width: 64, height: 64,
            layers: [
                {image: pbr.colour, opacity: 1.0, mode: "normal"},
                {image: filtered, opacity: 0.9, mode: "screen"}
            ]
        });
        let compPng = capability.invoke("hypermedia.radiance.encode_rgba_png", {image: comp});
        let normPng = capability.invoke("hypermedia.radiance.encode_rgba_png", {image: pbr.normal});
        let ormPng = capability.invoke("hypermedia.radiance.encode_rgba_png", {image: pbr.orm});
        let gallery = capability.invoke("hypermedia.discourse.review_gallery", {
            title: "Categorical Asset Suite",
            files: [
                {name: "composite.png", resource: compPng},
                {name: "normal.png", resource: normPng},
                {name: "orm.png", resource: ormPng}
            ]
        });
        let asset = {files: [
            {key: "composite.png", resource: compPng},
            {key: "normal.png", resource: normPng},
            {key: "orm.png", resource: ormPng},
            {key: "review.html", resource: gallery}
        ]};
    "#;
    session.submit(source, &cancel).unwrap();
    let candidate = session.candidate().unwrap();
    assert_eq!(candidate.outputs().len(), 4);
    let html = std::str::from_utf8(
        &candidate
            .outputs()
            .iter()
            .find(|r| r.key == "review.html")
            .unwrap()
            .bytes,
    )
    .unwrap();
    assert!(html.contains("Categorical Asset Suite"));
    assert!(html.contains("data:image/png;base64,"));
    let reopened = registry
        .reopen(candidate.bundle(), ProductionLimits::default(), &cancel)
        .unwrap();
    assert_eq!(reopened.candidate().unwrap().bundle(), candidate.bundle());
}

#[test]
fn wave1_locus_glyph_and_layout_morphisms_produce_webizen_studio_design_pack() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", vec![], ProductionLimits::default())
        .unwrap();
    let source = r#"
        requires [capability("capability.invoke")];
        let splineLocus = capability.invoke("hypermedia.locus.spline", {
            points: [[0, 0], [40, 0], [40, 40], [0, 40]],
            closed: true,
            tolerance: 1.0
        });
        let offsetLocus = capability.invoke("hypermedia.locus.offset", {
            profile: splineLocus,
            distance: 4.0,
            miter_limit: 4.0
        });
        let strokeLocus = capability.invoke("hypermedia.locus.stroke_to_locus", {
            points: [[-10, 20], [50, 20]],
            width: 8.0,
            closed: false
        });
        let baseBox = capability.invoke("hypermedia.locus.construct", {
            shape: "polygon",
            points: [[0, 0], [40, 0], [40, 40], [0, 40]]
        });
        let unionRes = capability.invoke("hypermedia.locus.boolean", {
            a: baseBox,
            b: strokeLocus,
            op: "union"
        });
        let locusSvg = capability.invoke("hypermedia.locus.project_svg", {
            profile: unionRes.profile,
            fill: [56, 189, 248, 255]
        });

        let metrics = capability.invoke("hypermedia.glyph.measure_bounds", {
            text: "Webizen Studio Ω-10D",
            font_size: 24.0,
            line_height: 1.25,
            tracking: 0.02
        });
        let shaped = capability.invoke("hypermedia.glyph.shape_to_svg_path", {
            text: "QualiaDB Δ→Ω",
            origin_x: 8.0,
            origin_y: 8.0,
            font_size: 20.0
        });
        let glyphSvg = capability.invoke("hypermedia.glyph.project_svg", {
            text: "Webizen Desktop UI",
            font_size: 22.0,
            color: [56, 189, 248, 255],
            background: [15, 21, 31, 255]
        });
        let glyphRgba = capability.invoke("hypermedia.glyph.rasterize_rgba", {
            text: "SYNTH + POET",
            width: 240,
            height: 64,
            origin_x: 12.0,
            origin_y: 16.0,
            font_size: 22.0,
            color: [234, 241, 248, 255],
            background: [16, 24, 38, 255],
            stroke_width: 2.2
        });
        let glyphPng = capability.invoke("hypermedia.radiance.encode_rgba_png", {image: glyphRgba});

        let solved = capability.invoke("hypermedia.layout.solve_box_tree", {
            width: 640.0,
            height: 360.0,
            direction: "column",
            padding: 16.0,
            gap: 12.0,
            grid_columns: 2,
            boxes: [
                {id: "nav", role: "header", label: "WEBIZEN STUDIO", state: "default"},
                {id: "tools", role: "panel", label: "ONTOLOGY GRAPH", state: "focus"},
                {id: "viewport", role: "canvas", label: "10D MANIFOLD VIEW", state: "active"},
                {id: "cta", role: "button", label: "PRODUCE HMC", state: "hover"}
            ]
        });
        let theme = capability.invoke("hypermedia.layout.bind_theme_scope", {
            scope: "app",
            mode: "holosonic-dark",
            corner_radius: 10.0
        });
        let uiSheetRgba = capability.invoke("hypermedia.layout.render_ui_sheet", {
            layout: solved,
            theme: theme,
            scale: 1.0
        });
        let uiSheetPng = capability.invoke("hypermedia.radiance.encode_rgba_png", {image: uiSheetRgba});
        let uiSheetSvg = capability.invoke("hypermedia.layout.project_ui_svg", {
            layout: solved,
            theme: theme,
            scale: 1.0
        });
        let gallery = capability.invoke("hypermedia.discourse.review_gallery", {
            title: "Wave 1 Webizen Design Pack",
            files: [
                {name: "locus.svg", resource: locusSvg},
                {name: "glyph.svg", resource: glyphSvg},
                {name: "glyph.png", resource: glyphPng},
                {name: "ui-sheet.png", resource: uiSheetPng},
                {name: "ui-sheet.svg", resource: uiSheetSvg}
            ]
        });
        let asset = {files: [
            {key: "locus.svg", resource: locusSvg},
            {key: "glyph.svg", resource: glyphSvg},
            {key: "glyph.png", resource: glyphPng},
            {key: "ui-sheet.png", resource: uiSheetPng},
            {key: "ui-sheet.svg", resource: uiSheetSvg},
            {key: "review.html", resource: gallery}
        ]};
    "#;
    session.submit(source, &cancel).unwrap();
    let candidate = session.candidate().unwrap();
    assert_eq!(candidate.outputs().len(), 6);
    let ui_svg = std::str::from_utf8(
        &candidate
            .outputs()
            .iter()
            .find(|r| r.key == "ui-sheet.svg")
            .unwrap()
            .bytes,
    )
    .unwrap();
    assert!(ui_svg.contains("--webizen-surface-bg:"));
    assert!(ui_svg.contains("WEBIZEN STUDIO"));
}

#[test]
fn wave2_radiance_color_masks_luts_photo_develop_and_psd_roundtrip() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", vec![], ProductionLimits::default())
        .unwrap();
    let source = r#"
        requires [capability("capability.invoke")];
        let base = capability.invoke("hypermedia.radiance.render_vector", {
            width: 64, height: 64,
            layers: [
                {
                    path: "M 0 0 L 64 0 L 64 64 L 0 64 Z",
                    fill: [24, 32, 48, 255]
                },
                {
                    path: "M 12 12 L 52 12 L 52 52 L 12 52 Z",
                    fill: [220, 120, 60, 255]
                }
            ]
        });
        let overlayShape = capability.invoke("hypermedia.radiance.render_vector", {
            width: 64, height: 64,
            layers: [
                {
                    path: "M 20 20 L 44 20 L 44 44 L 20 44 Z",
                    fill: [80, 200, 250, 255]
                }
            ]
        });
        let borderMask = capability.invoke("hypermedia.radiance.modify_mask", {
            image: overlayShape,
            mode: "border",
            radius: 2.0,
            channel: "alpha"
        });
        let developed = capability.invoke("hypermedia.radiance.develop_photo", {
            image: base,
            exposure: 0.25,
            temperature: 0.1,
            highlights: -0.15,
            shadows: 0.2,
            clarity: 0.2,
            vibrance: 0.15,
            denoise: 0.2
        });
        let graded = capability.invoke("hypermedia.radiance.grade_color", {
            image: developed,
            working_space: "rec709",
            lift: [0.02, 0.0, -0.02],
            gamma: [1.0, 1.0, 1.0],
            gain: [1.05, 1.0, 0.98],
            contrast: 1.1,
            saturation: 1.08
        });
        let lutApplied = capability.invoke("hypermedia.radiance.apply_lut", {
            image: graded,
            preset: "cool-teal-orange",
            intensity: 0.75
        });
        let pqImg = capability.invoke("hypermedia.radiance.convert_transfer", {
            image: lutApplied,
            source_transfer: "srgb",
            target_transfer: "pq"
        });
        let srgbImg = capability.invoke("hypermedia.radiance.convert_transfer", {
            image: pqImg,
            source_transfer: "pq",
            target_transfer: "srgb"
        });
        let psdFile = capability.invoke("hypermedia.radiance.encode_psd", {
            width: 64,
            height: 64,
            layers: [
                {name: "Background", image: srgbImg, opacity: 1.0},
                {name: "MaskBorder", image: borderMask, opacity: 0.85}
            ]
        });
        let reimportedComposite = capability.invoke("hypermedia.radiance.import_psd", {
            input: psdFile
        });
        let reimportedLayer1 = capability.invoke("hypermedia.radiance.import_psd", {
            input: psdFile,
            layer_index: 1
        });
        let compPng = capability.invoke("hypermedia.radiance.encode_rgba_png", {image: reimportedComposite});
        let layerPng = capability.invoke("hypermedia.radiance.encode_rgba_png", {image: reimportedLayer1});
        let asset = {files: [
            {key: "studio.psd", resource: psdFile},
            {key: "composite.png", resource: compPng},
            {key: "layer1.png", resource: layerPng}
        ]};
    "#;
    session.submit(source, &cancel).unwrap();
    let candidate = session.candidate().unwrap();
    assert_eq!(candidate.outputs().len(), 3);
    let psd_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "studio.psd")
        .unwrap()
        .bytes;
    assert_eq!(&psd_bytes[..4], b"8BPS");
}
