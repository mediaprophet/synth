use super::*;

#[test]
fn wave12_living_world_botanical_vertebrate_anatomy_fauna_rig_ontology_pin_continuum_and_frustum() {
    assert_eq!(synth::production::assets::OPERATIONS.len(), 148);
    assert_eq!(synth::production::assets::ontology::MORPHISMS.len(), 148);
    let catalogue = synth::production::assets::catalogue();
    let catalogue_ops = catalogue["operations"]
        .as_array()
        .expect("catalogue operations array");
    assert_eq!(catalogue_ops.len(), 148);
    assert_eq!(catalogue["count"], 148);
    for packing in [
        "hypermedia.radiance.pack_sprite_sheet",
        "hypermedia.radiance.pack_nine_slice",
        "hypermedia.surface.pack_pbr_set",
    ] {
        assert!(
            catalogue_ops.iter().any(|op| op["id"] == packing),
            "live catalogue must include packing morphism {packing}"
        );
    }
    assert_eq!(
        synth::production::assets::ontology::ONTOLOGY_NQUINS.len(),
        444
    );
    let catalog = synth::production::assets::OPERATIONS
        .iter()
        .flat_map(|(id, owner, summary)| [*id, *owner, *summary])
        .collect::<Vec<_>>()
        .join("\n");
    for needle in ["CsiFrame", "interpolation=cubic", "UV01"] {
        assert!(
            catalog.contains(needle),
            "host catalog must raise Wave C1 object {needle}"
        );
    }

    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", Vec::new(), ProductionLimits::default())
        .unwrap();
    let source = r##"
        requires [capability("capability.invoke")];
        let botS1 = capability.invoke("hypermedia.manifold.compile_botanical_organ_10d", {
            scale: 1,
            species_id: "corymbia-aparrerinja",
            peel_01: 0.25,
            anthesis_01: 0.75
        });
        let botS4 = capability.invoke("hypermedia.manifold.compile_botanical_organ_10d", {
            scale: 4,
            species_id: "corymbia-aparrerinja",
            peel_01: 0.5,
            anthesis_01: 0.75
        });
        let vertS5 = capability.invoke("hypermedia.manifold.compile_vertebrate_anatomy_10d", {
            archetype: "macropod_saltatorial",
            osteology_vis_01: 1.0,
            myology_vis_01: 0.75,
            integument_vis_01: 0.25,
            explode_01: 0.35
        });
        let faunaRig = capability.invoke("hypermedia.kinetics.rig_biomechanical_archetype", {
            archetype: "macropod_saltatorial",
            speed_m_s: 4.5,
            turn_rate_rad_s: 0.2,
            dt_sec: 0.033
        });
        let bioPin = capability.invoke("hypermedia.discourse.query_anatomical_ontology_pin", {
            scale: 4,
            ray_origin: [0.0, 0.0, 1.0],
            ray_dir: [0.0, 0.0, -1.0],
            max_dist: 10.0
        });
        let continuum3d = capability.invoke("hypermedia.geodesy.step_sector_continuum_3d", {
            sector_id: "jounama_karst",
            steps: 4
        });
        let frustumSync = capability.invoke("hypermedia.layout.synchronize_companion_frustum", {
            rows: 6,
            cols: 10,
            dome_fov_deg: 180.0,
            frame_index: 1
        });

        let summarySvg = capability.invoke("hypermedia.discourse.render_chart_svg", {
            chart_type: "bar",
            title: "Wave 12 Living World Summary",
            width: 480,
            height: 320,
            labels: ["BotVtx", "VertSub", "RigJoints", "BioPins", "FrustumWedges"],
            series: [
                {
                    name: "Counts",
                    values: [
                        botS4.vertex_count,
                        vertS5.submesh_count,
                        faunaRig.joint_count,
                        bioPin.total_canonical_pins,
                        frustumSync.wedge_count
                    ],
                    color: "#22c55e"
                }
            ]
        });

        let asset = {files: [
            {key: "wave12-living-world.svg", resource: summarySvg}
        ]};
    "##;
    session.submit(source, &cancel).unwrap();
    let candidate = session.candidate().unwrap();
    assert_eq!(candidate.outputs().len(), 1);
    let svg_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "wave12-living-world.svg")
        .unwrap()
        .bytes;
    let svg_str = std::str::from_utf8(svg_bytes).unwrap();
    assert!(svg_str.contains("Wave 12 Living World Summary"));
}

#[test]
fn wave13_categorical_domain_crates_end_to_end() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", Vec::new(), ProductionLimits::default())
        .unwrap();
    let source = r##"
        requires [capability("capability.invoke")];
        let sketch = capability.invoke("hypermedia.locus.solve_parametric_sketch", {
            target_length: 15.0
        });
        let pathWarp = capability.invoke("hypermedia.locus.pathfinder_warp_envelope", {
            profile: sketch.profile,
            width: 50.0,
            height: 50.0,
            bend: 20.0,
            corner_radius: 4.0
        });
        let geoVec = capability.invoke("hypermedia.locus.transcode_geospatial_vector", {
            profile: pathWarp.profile,
            lon: 149.13,
            lat: -35.2809,
            zoom: 12
        });
        let assembly = capability.invoke("hypermedia.manifold.sweep_assembly_profile", {
            profile: sketch.profile,
            width: 1.2,
            height: 0.8,
            wall: 0.15,
            length: 2.0,
            caps: true
        });
        let bimIfc = capability.invoke("hypermedia.manifold.compile_bim_ifc4", {
            project_name: "Commons Civic Hall"
        });
        let terrain = capability.invoke("hypermedia.manifold.erode_geomorphic_heightfield", {
            grid_size: 16
        });
        let terrainPng = capability.invoke("hypermedia.radiance.encode_rgba_png", {
            image: terrain.heightfield_image
        });
        let pbr = capability.invoke("hypermedia.surface.bake_pbr_materialx", {
            mesh: bimIfc.mesh,
            resolution: 16,
            samples: 2
        });
        let brush = capability.invoke("hypermedia.surface.expand_procedural_brush", {
            profile: pathWarp.profile,
            stroke_weight: 6.0,
            bristle_count: 8,
            wetness: 0.5
        });
        let rawHdr = capability.invoke("hypermedia.radiance.develop_camera_raw_hdr", {
            image: terrain.heightfield_image,
            width: 16,
            height: 16
        });
        let cdlScopes = capability.invoke("hypermedia.radiance.grade_cdl_scopes", {
            image: rawHdr.image,
            slope_r: 1.05
        });
        let trace = capability.invoke("hypermedia.radiance.trace_vector_contours", {
            image: cdlScopes.image,
            width: 16,
            height: 16,
            target_width: 12
        });
        let kp = capability.invoke("hypermedia.glyph.compose_knuth_plass_paragraph", {
            frame_width: 200.0
        });
        let flexGrid = capability.invoke("hypermedia.layout.solve_flexbox_grid_constraints", {
            container_width: 300.0
        });
        let paged = capability.invoke("hypermedia.layout.typeset_paged_foliation", {
            columns: 2
        });
        let nodegraph = capability.invoke("hypermedia.layout.compile_dataflow_nodegraph", {
            expression: "2 * pi * 5"
        });
        let femLbm = capability.invoke("hypermedia.kinetics.solve_continuum_fem_lbm", {
            lbm_steps: 2
        });
        let garment = capability.invoke("hypermedia.kinetics.drape_orthotropic_garment", {
            substeps: 2,
            iterations: 2
        });
        let blockSim = capability.invoke("hypermedia.kinetics.simulate_causal_block_diagram", {
            image: flexGrid.image,
            constant_input: 3.0
        });
        let cpm = capability.invoke("hypermedia.kinetics.schedule_cpm_evm_montecarlo", {
            trials: 50
        });
        let daw = capability.invoke("hypermedia.acoustics.analyze_ebu_r128_camelot_daw", {
            freq_hz: 440.0
        });
        let csi = capability.invoke("hypermedia.electromagnetics.sense_wifi_csi_vitals_aoa", {
            breathing_hz: 0.25
        });
        let krig = capability.invoke("hypermedia.geodesy.kriging_variogram_morphometry", {
            target_x: 5.0,
            target_y: 5.0
        });
        let audit = capability.invoke("hypermedia.discourse.audit_tex_pdf_ua_matlang", {});
        let chronicle = capability.invoke("hypermedia.chronicle.recalculate_tabular_fiduciary_archive", {
            cell_a1: 200.0,
            cell_b1: 300.0
        });

        let asset = {files: [
            {key: "sketch.svg", resource: sketch.svg_resource},
            {key: "warped.svg", resource: pathWarp.svg_resource},
            {key: "station.wkb", resource: geoVec.wkb_resource},
            {key: "station.kml", resource: geoVec.kml_resource},
            {key: "assembly.10d", resource: assembly.container_10d},
            {key: "civic-hall.ifc", resource: bimIfc.ifc_resource},
            {key: "eroded-terrain.png", resource: terrainPng},
            {key: "material.mtlx", resource: pbr.materialx_resource},
            {key: "material.psd", resource: pbr.psd_resource},
            {key: "brush-stroke.svg", resource: brush.svg_resource},
            {key: "display-p3.icc", resource: rawHdr.icc_resource},
            {key: "grade.cube", resource: cdlScopes.cube_lut_resource},
            {key: "traced-contours.svg", resource: trace.svg_resource},
            {key: "knuth-plass.svg", resource: kp.svg_resource},
            {key: "subset.ttf", resource: kp.font_resource},
            {key: "layout-grid.svg", resource: flexGrid.svg_resource},
            {key: "foliated-page.svg", resource: paged.svg_resource},
            {key: "nodegraph.json", resource: nodegraph.nodegraph_json_resource},
            {key: "slide-morph.svg", resource: nodegraph.morph_svg_resource},
            {key: "structural.inp", resource: femLbm.abaqus_inp_resource},
            {key: "garment.obj", resource: garment.obj_resource},
            {key: "block-trajectory.svg", resource: blockSim.trajectory_svg_resource},
            {key: "schedule-gantt.svg", resource: cpm.gantt_svg_resource},
            {key: "daw-stem.wav", resource: daw.wav_resource},
            {key: "csi-aoa.svg", resource: csi.aoa_svg_resource},
            {key: "kriging-contours.svg", resource: krig.contour_svg_resource},
            {key: "redacted.txt", resource: audit.redacted_txt_resource},
            {key: "equation.svg", resource: audit.equation_svg_resource},
            {key: "archive.docx", resource: chronicle.docx_resource},
            {key: "timeline.otio", resource: chronicle.otio_resource},
            {key: "broadcast.mkv", resource: chronicle.mkv_resource},
            {key: "master.prores", resource: chronicle.prores_resource}
        ]};
    "##;
    session.submit(source, &cancel).unwrap();
    let candidate = session.candidate().unwrap();
    assert_eq!(candidate.outputs().len(), 32);
    let ifc_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "civic-hall.ifc")
        .unwrap()
        .bytes;
    let ifc_str = std::str::from_utf8(ifc_bytes).unwrap();
    assert!(ifc_str.contains("ISO-10303-21;"));
    let docx_bytes = &candidate
        .outputs()
        .iter()
        .find(|r| r.key == "archive.docx")
        .unwrap()
        .bytes;
    assert_eq!(&docx_bytes[..4], b"PK\x03\x04");
}
