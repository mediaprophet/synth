//! Explicit production output recipes and a bounded native review subprocess.
use super::*;
use std::{io::Read, process::{Command, Stdio}, time::{Duration, Instant}};

pub(super) fn output_recipes() -> Value {
    json!([{
        "id": "rgba_png",
        "resource_key": "image.png",
        "resource_kind": "png",
        "source": "requires [capability(\"capability.invoke\")]; let image = capability.invoke(\"hypermedia.radiance.create_rgba\", {width:8,height:8,color:[51,102,153,255]}); let png = capability.invoke(\"hypermedia.radiance.encode_rgba_png\", {image:image}); let mut asset = {files:[{key:\"image.png\",resource:png}]};"
    }, {
        "id": "sprite_record_png",
        "resource_key": "sheet.png",
        "resource_kind": "png",
        "source": "requires [capability(\"capability.invoke\")]; let tile = capability.invoke(\"hypermedia.radiance.create_rgba\", {width:8,height:8,color:[51,102,153,255]}); let sheet = capability.invoke(\"hypermedia.radiance.pack_sprite_sheet\", {tiles:[tile],columns:1,padding:0}); let mut asset = {files:[{key:\"sheet.png\",resource:sheet.png}]};"
    }, {
        "id": "lossless_sequence",
        "resource_key": "sequence.json",
        "resource_kind": "json",
        "source": "requires [capability(\"capability.invoke\")]; let tile = capability.invoke(\"hypermedia.radiance.create_rgba\", {width:8,height:8,color:[51,102,153,255]}); let sequence = capability.invoke(\"hypermedia.chronicle.capture_deterministic_movie\", {frames:[tile,tile],fps:{numerator:24000,denominator:1001},profile:\"png-sequence\"}); let mut asset = {files:[{key:\"sequence.json\",resource:sequence.manifest_resource}]};"
    }, {
        "id": "emf",
        "resource_key": "field.emf",
        "resource_kind": "emf",
        "container_magic": "Q42EMF64",
        "container_version": 4,
        "source": "requires [capability(\"capability.invoke\")]; let emf = capability.invoke(\"hypermedia.electromagnetics.compile_emf\", {origin:[0.0,0.0,0.0],voxel_size:0.5,grid_dims:[1,1,1],epoch_s:1700000000.0,sources:[{position:[0.0,0.0,0.0],e_amplitude:[150.0,0.0,0.0],wavelength_nm:550.0,phase:0.0}]}); let mut asset = {files:[{key:\"field.emf\",resource:emf}]};"
    }, {
        "id": "qac",
        "resource_key": "signal.qac",
        "resource_kind": "qac",
        "container_magic": "qac\0\0\0\0\0",
        "container_version": 1,
        "source": "requires [capability(\"capability.invoke\")]; let tone = capability.invoke(\"hypermedia.acoustics.synthesize_tone\", {frequency:440.0,sample_rate:48000,frames:256,amplitude:0.3}); let qac = capability.invoke(\"hypermedia.acoustics.encode_aac\", {audio:tone,format:\"qac\"}); let mut asset = {files:[{key:\"signal.qac\",resource:qac}]};"
    }])
}

pub(super) fn catalogue() -> Value {
    let operations: Vec<Value> = crate::production::assets::ontology::MORPHISMS.iter().map(|operation| {
        let (kind, fields, encoder) = match operation.id {
            "hypermedia.manifold.fit_holographic_v2" => ("encoded_resource", json!([]), Value::Null),
            "hypermedia.manifold.seal_sdf_brick_10d" => ("record", json!([
                {"field":"asset","type":"encoded_resource","kind":"10d"},
                {"field":"field","type":"encoded_resource","kind":"sdfb"}
            ]), Value::Null),
            "hypermedia.manifold.seal_leaf_assembly_10d"
            | "hypermedia.manifold.compose_assembly_10d" => ("record", json!([
                {"field":"assembly","type":"encoded_resource","kind":"10d"},
                {"field":"scene","type":"typed_handle","kind":"scene_graph_10d"}
            ]), Value::Null),
            "qualia.manifold.seal_10d_with_cbld_licensing" => ("record", json!([
                {"field":"asset","type":"encoded_resource","kind":"10d"},
                {"field":"license_q42","type":"encoded_resource","kind":"q42"},
                {"field":"manifest_q42_yaml","type":"encoded_resource","kind":"yaml"}
            ]), Value::Null),
            "hypermedia.radiance.compute_scopes_and_icc" => ("record", json!([
                {"field":"image","type":"image_handle","kind":"rgba16"},
                {"field":"scopes","type":"encoded_resource","kind":"json"}
            ]), Value::Null),
            "hypermedia.radiance.develop_raw_hdr_panorama" => ("record", json!([
                {"field":"image","type":"typed_handle","kind":"radiance_lattice"},
                {"field":"metadata","type":"encoded_resource","kind":"q42"}
            ]), Value::Null),
            "hypermedia.surface.paint_3d_texture" => ("record", json!([
                {"field":"maps","type":"image_handle_map","kind":"rgba16"}
            ]), Value::Null),
            "hypermedia.kinetics.bind_skin_and_morphs" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"container_10d","type":"encoded_resource","kind":"10d"},
                {"field":"rig","type":"typed_handle","kind":"skeletal_rig"}
            ]), Value::Null),
            "qualia.glyph.shape_opentype_sfnt" => ("record", json!([
                {"field":"outlines","type":"encoded_resource","kind":"svg"},
                {"field":"metrics","type":"encoded_resource","kind":"json"}
            ]), Value::Null),
            "qualia.governance.validate_domain_shacl_and_tk_bc" => ("record", json!([
                {"field":"provenance_receipt","type":"encoded_resource","kind":"q42"}
            ]), Value::Null),
            "hypermedia.manifold.transfer_attributes"
            | "hypermedia.radiance.composite_aov_graph" => ("typed_handle", json!([]), Value::Null),
            "hypermedia.kinetics.track_planar_and_animate_vector"
            | "hypermedia.kinetics.simulate_control_blocks_and_cpm" => ("encoded_resource", json!([]), Value::Null),
            "hypermedia.kinetics.evaluate_absm_blendspace" => ("record", json!([
                {"field":"clip","type":"typed_handle","kind":"animation_clip"}
            ]), Value::Null),
            "hypermedia.chronicle.evaluate_tabular_workbook_cube" => ("record", json!([
                {"field":"cube","type":"typed_handle","kind":"tabular_cube"},
                {"field":"observations","type":"encoded_resource","kind":"q42","when":"projecting numeric observations"}
            ]), Value::Null),
            "hypermedia.chronicle.encode_video_master" => ("record", json!([
                {"field":"movie","type":"encoded_resource","kind":"mov"},
                {"field":"receipt","type":"encoded_resource","kind":"json"}
            ]), Value::Null),
            "hypermedia.chronicle.decode_video_master" => ("record", json!([
                {"field":"frames","type":"image_handle_list"},
                {"field":"telemetry","type":"encoded_resource","kind":"json"}
            ]), Value::Null),
            "hypermedia.chronicle.reconstruct_multiview_3d" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"field","type":"typed_handle","kind":"field_lattice"}
            ]), Value::Null),
            "hypermedia.kinetics.build_navmesh" => ("record", json!([
                {"field":"navmesh","type":"typed_handle","kind":"field_lattice"}
            ]), Value::Null),
            "hypermedia.kinetics.step_vbd_soft_tissue" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"volume_det_f","type":"typed_handle","kind":"tabular_cube"}
            ]), Value::Null),
            "hypermedia.kinetics.solve_structural_fea_multiphysics" => ("record", json!([
                {"field":"displacement_m","type":"typed_handle","kind":"field_lattice","when":"solving Hex8 elasticity"},
                {"field":"centre_strain_and_stress_pa","type":"typed_handle","kind":"field_lattice","when":"solving Hex8 elasticity"},
                {"field":"vtu","type":"encoded_resource","kind":"vtu","when":"solving Hex8 elasticity"},
                {"field":"temperature_k","type":"typed_handle","kind":"field_lattice","when":"solving Quad4 conduction"}
            ]), Value::Null),
            "hypermedia.electromagnetics.solve_maxwell_yee_3d" => ("record", json!([
                {"field":"fields","type":"typed_handle","kind":"field_lattice"},
                {"field":"emf","type":"encoded_resource","kind":"emf"}
            ]), Value::Null),
            "hypermedia.kinetics.step_cartoon_kinetics" => ("record", json!([
                {"field":"clip","type":"typed_handle","kind":"animation_clip"},
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"deformation_samples","type":"typed_handle","kind":"tabular_cube"}
            ]), Value::Null),
            "hypermedia.kinetics.solve_cosserat_strand" => ("record", json!([
                {"field":"strands","type":"typed_handle","kind":"curve_groom_bundle"},
                {"field":"frames","type":"encoded_resource","kind":"json"}
            ]), Value::Null),
            "hypermedia.manifold.colonize_branching_graph" => ("record", json!([
                {"field":"graph","type":"typed_handle","kind":"branch_graph_10d"}
            ]), Value::Null),
            "hypermedia.geodesy.simulate_ocean_water" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh","when":"water surface is present"},
                {"field":"fields","type":"typed_handle","kind":"field_lattice"}
            ]), Value::Null),
            "hypermedia.cosmos.ephemeris_propagate" => ("record", json!([
                {"field":"trajectory","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"}
            ]), Value::Null),
            "qualia.physics.homogenize_to_continuum" => ("record", json!([]), Value::Null),
            "qualia.accessibility.synthesize_iso17049_tactile_braille" => ("record", json!([
                {"field":"asset","type":"encoded_resource","kind":"10d"},
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"receipt","type":"encoded_resource","kind":"json"}
            ]), Value::Null),
            "qualia.category.solve_adjoint_inverse_morphism" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.kinetics.solve_cfd_flip_smoke" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
            ]), Value::Null),
            "hypermedia.acoustics.master_opus_flac_and_stems" => ("record", json!([
                {"field":"audio","type":"encoded_resource","kind":"flac","when":"op=encode_flac"},
                {"field":"audio","type":"typed_handle","kind":"audio_multichannel","when":"op=time_stretch_wsola"},
                {"field":"receipt","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "qualia.quantum.simulate_openqasm_circuit" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
                {"field":"state","type":"typed_handle","kind":"field_lattice"},
            ]), Value::Null),
            "hypermedia.geodesy.ingest_gis_corpus" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice","when":"GeoTIFF, classic NetCDF, or Zarr v3 ZIP"},
                {"field":"vector_layers","type":"encoded_resource","kind":"json","when":"LAS"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
                {"field":"roundtrip_receipt","type":"encoded_resource","kind":"q42","when":"Zarr v3 ZIP"},
            ]), Value::Null),
            "hypermedia.geodesy.krige_subsurface_strata_m64" => ("record", json!([
                {"field":"estimate","type":"typed_handle","kind":"field_lattice"},
                {"field":"variance","type":"typed_handle","kind":"field_lattice"},
                {"field":"m64","type":"encoded_resource","kind":"m64","when":"a horizon is requested and admissible"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.topology.solve_cellular_sheaf_cohomology" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.narrative.solve_linear_logic_world_grammar" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.logic.solve_cdcl_smt_and_egraph_arena" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.chronicle.transcode_av1_cmaf_mxf" => ("record", json!([
                {"field":"video","type":"encoded_resource","kind":"mp4"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.chronicle.demux_and_parse_mxf_cmaf" => ("record", json!([
                {"field":"timeline","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.radiance.encode_exr_jxl_ktx2" => ("record", json!([
                {"field":"image","type":"encoded_resource","kind":"exr"},
                {"field":"report","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "hypermedia.radiance.import_exr_jxl_ktx2" => ("record", json!([
                {"field":"image","type":"typed_handle","kind":"radiance_lattice"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.manifold.compile_bim_ifc_mep" => ("record", json!([
                {"field":"ifc","type":"encoded_resource","kind":"ifc"},
                {"field":"graph","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.fabrication.unfold_sheet_and_cam_25d" => ("record", json!([
                {"field":"dxf","type":"encoded_resource","kind":"dxf"},
            ]), Value::Null),
            "qualia.privacy.redact_clinical_and_biometric_phi" => ("record", json!([
                {"field":"frames","type":"typed_handle_array","kind":"radiance_lattice","when":"mode=authored_raster_regions"},
                {"field":"image","type":"typed_handle","kind":"radiance_lattice","when":"mode=dicom_roi_raster on a native host"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.accessibility.certify_pse_cvd_and_plain_language" => ("record", json!([
                {"field":"image","type":"typed_handle","kind":"radiance_lattice","when":"simulate_cvd"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.acoustics.synthesize_midi_sf2" => ("record", json!([
                {"field":"audio","type":"typed_handle","kind":"audio_multichannel"},
                {"field":"receipt","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "qualia.acoustics.synthesize_articulatory_speech_and_holosonic" => ("record", json!([
                {"field":"audio","type":"typed_handle","kind":"audio_multichannel"},
                {"field":"qac","type":"encoded_resource","kind":"qac"},
                {"field":"receipt","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "hypermedia.manifold.retopologize_quad" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
            ]), Value::Null),
            "hypermedia.manifold.sculpt_multires" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"displacement_map16","type":"typed_handle","kind":"rgba16"},
            ]), Value::Null),
            "hypermedia.surface.groom_hair_and_pelage" => ("record", json!([
                {"field":"groom","type":"typed_handle","kind":"curve_groom_bundle"},
                {"field":"material","type":"typed_handle","kind":"material_graph"},
            ]), Value::Null),
            "qualia.anatomy.bind_enthesis_and_retinaculum" => ("record", json!([
                {"field":"binding","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "hypermedia.manifold.populate_allometric_vascular_c1" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"report","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "hypermedia.manifold.amplify_terminal_phyllotaxis" => ("record", json!([
                {"field":"assembly","type":"encoded_resource","kind":"10d"},
                {"field":"scene","type":"typed_handle","kind":"scene_graph_10d"},
            ]), Value::Null),
            "hypermedia.surface.stylize_npr_cartoon" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"material","type":"typed_handle","kind":"material_graph"},
                {"field":"materialx","type":"encoded_resource","kind":"mtlx"},
            ]), Value::Null),
            "hypermedia.radiance.render_gpu_hybrid" => ("record", json!([
                {"field":"radiance","type":"typed_handle","kind":"radiance_lattice"},
            ]), Value::Null),
            "qualia.rights.issue_pq_vc_and_agent_state" => ("record", json!([
                {"field":"claims","type":"array","when":"mode=prepare_mldsa65_asset_claims"},
                {"field":"receipt","type":"encoded_resource","kind":"q42","when":"mode=verify_mldsa65_asset_claims"},
            ]), Value::Null),
            "qualia.physics.couple_exergetic_port_hamiltonian_operad" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.category.compose_parametric_optics_crdc" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.kinetics.solve_tribology_ehl_and_wear" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.kinetics.solve_4dvar_fluid_and_contact_adjoint" => ("record", json!([
                {"field":"initial_conditions","type":"typed_handle","kind":"field_lattice"},
                {"field":"final_conditions","type":"typed_handle","kind":"field_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.kinetics.solve_free_surface_3d" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"surface_field","type":"typed_handle","kind":"field_lattice"},
                {"field":"particles","type":"typed_handle","kind":"particle_set"},
            ]), Value::Null),
            "hypermedia.cosmos.synth_planetary_body" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"asset","type":"encoded_resource","kind":"10d"},
                {"field":"radial_height","type":"typed_handle","kind":"field_lattice"},
            ]), Value::Null),
            "hypermedia.cosmos.render_planetary_limb_sky" => ("record", json!([
                {"field":"radiance","type":"typed_handle","kind":"radiance_lattice"},
                {"field":"transmittance","type":"typed_handle","kind":"radiance_lattice"},
            ]), Value::Null),
            "hypermedia.cosmos.synth_planetary_ring" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"asset","type":"encoded_resource","kind":"10d"},
                {"field":"radial_optics","type":"typed_handle","kind":"field_lattice"},
            ]), Value::Null),
            "hypermedia.cosmos.step_space_weather_aurora" => ("record", json!([
                {"field":"aurora","type":"typed_handle","kind":"field_lattice"},
            ]), Value::Null),
            "hypermedia.cosmos.eval_nbody_transfer" => ("record", json!([
                {"field":"trajectory","type":"typed_handle","kind":"tabular_cube"},
            ]), Value::Null),
            "hypermedia.manifold.step_cad_brep" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"bom_port_graph","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.manifold.export_step_ap242_brep" => ("record", json!([
                {"field":"asset","type":"encoded_resource","kind":"step"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.surface.simulate_weathering_and_wear" => ("record", json!([
                {"field":"state","type":"typed_handle","kind":"field_lattice"},
            ]), Value::Null),
            "hypermedia.interchange.transcode_openusd_openvdb_alembic" => ("record", json!([
                {"field":"asset","type":"encoded_resource","kind":"10d"},
                {"field":"field","type":"encoded_resource","kind":"sdfb"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.fabrication.reconstruct_toolpath_to_solid" => ("record", json!([
                {"field":"asset","type":"encoded_resource","kind":"10d"},
                {"field":"field","type":"encoded_resource","kind":"sdfb"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.rights.seal_qecp_and_work_graph" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42","when":"verify mode"},
            ]), Value::Null),
            "hypermedia.fabrication.route_pcb_and_wire_harness" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.infrastructure.solve_microgrid_and_lca_commons" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.control.bridge_iec61131_plc_and_ros2_hil" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.infrastructure.solve_multimodal_transit_and_crowd" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.infrastructure.solve_hydraulic_pipe_and_stormwater" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.chronicle.calibrate_icvfx_led_volume_stage" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.interchange.transcode_eda_spice_touchstone_odb" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.fabrication.certify_dfm_manifold" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "hypermedia.fabrication.synth_tpms_lattice" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "hypermedia.fabrication.orient_nest_tree_support" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "hypermedia.fabrication.slice_and_toolpath" => ("record", json!([
                {"field":"masks","type":"encoded_resource","kind":"zip"},
                {"field":"receipt","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "hypermedia.fabrication.export_print_package" => ("record", json!([
                {"field":"asset","type":"encoded_resource","kind":"3mf","when":"format=3mf_core"},
                {"field":"asset","type":"encoded_resource","kind":"stl","when":"format=stl_binary"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.fabrication.certify_cmm_metrology_gdt" => ("record", json!([
                {"field":"cube","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.surface.transcode_layered_raster" => ("record", json!([
                {"field":"stack","type":"record","kind":"layered_raster_stack","when":"direction=import_stack"},
                {"field":"image","type":"encoded_resource","kind":"psd","when":"export_layered format=psd"},
                {"field":"image","type":"encoded_resource","kind":"psb","when":"export_layered format=psb"},
                {"field":"report","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "qualia.logic.check_cic_dependent_proof_kernel" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.environment.step_fire_and_soil_coupled" => ("record", json!([
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"ground_cover","type":"typed_handle","kind":"field_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.radiance.solve_mueller_polarimetry_and_thin_film" => ("record", json!([
                {"field":"mueller","type":"typed_handle","kind":"field_lattice"},
                {"field":"reflected_stokes","type":"typed_handle","kind":"field_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.radiance.synthesize_cgh_wave_diffraction" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"image","type":"typed_handle","kind":"radiance_lattice"},
                {"field":"phase_only_rad","type":"typed_handle","kind":"field_lattice","when":"phase synthesis requested"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.radiance.bake_refractive_caustics" => ("record", json!([
                {"field":"caustic_map","type":"typed_handle","kind":"radiance_lattice"},
                {"field":"irradiance","type":"typed_handle","kind":"field_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.anatomy.validate_topological_body" => ("record", json!([
                {"field":"report","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "qualia.botany.step_fspm_vascular_and_leaf" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"evidence","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.kinetics.solve_pyro_buoyancy_3d" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
            ]), Value::Null),
            "hypermedia.kinetics.solve_granular_mpm" => ("record", json!([
                {"field":"heightfield","type":"typed_handle","kind":"field_lattice"},
                {"field":"mean_stress_pressure_pa","type":"typed_handle","kind":"field_lattice"},
                {"field":"particles","type":"typed_handle","kind":"particle_set"},
            ]), Value::Null),
            "hypermedia.cosmos.raymarch_relativistic_disk" => ("record", json!([
                {"field":"spectral_radiance","type":"typed_handle","kind":"field_lattice"},
                {"field":"ray_geometry","type":"typed_handle","kind":"field_lattice"},
            ]), Value::Null),
            "hypermedia.fabrication.split_joinery_assembly" => ("record", json!([
                {"field":"parts","type":"typed_handle_array","kind":"attributed_mesh"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.anvil.attest_and_route_np64" => ("record", json!([
                {"field":"plan","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.sensing.simulate_hardware_sensor_twin" => ("record", json!([
                {"field":"observations","type":"typed_handle","kind":"field_lattice"},
                {"field":"point_cloud","type":"typed_handle","kind":"field_lattice","when":"hits > 0"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.storage.offload_zns_csd_and_qubo_commons" => ("record", json!([
                {"field":"plan","type":"encoded_resource","kind":"json","when":"mode=sequential_zone_plan"},
                {"field":"report","type":"encoded_resource","kind":"json","when":"mode=offline_qubo_anneal"},
                {"field":"matrix","type":"encoded_resource","kind":"bin","when":"mode=compile_qubo_constraint_fixture"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.provenance.encode_creation_tree_payload" => ("record", json!([
                {"field":"anchor","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.provenance.decode_stego_and_verify_lineage" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.physics.solve_piezoelectric_multiferroic_coupling" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.fabrication.simulate_molding_casting_and_composites" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.kinetics.solve_compliant_soft_robotics_origami" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.geodesy.solve_glaciology_and_permafrost" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.geodesy.simulate_seismology_and_tsunami_coupling" => ("record", json!([
                {"field":"water","type":"typed_handle","kind":"field_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.environment.solve_limnology_estuarine_biogeochemistry" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.infrastructure.solve_geotechnical_slope_and_tunnel" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.radiance.raytrace_multi_element_optical_lens" => ("record", json!([
                {"field":"image","type":"typed_handle","kind":"radiance_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.provenance.verify_microstructure_puf_fuzzy" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42","when":"compare_registered_optical_capture"},
            ]), Value::Null),
            "qualia.accessibility.synthesize_sigml_sign_language_avatar" => ("record", json!([
                {"field":"clip","type":"typed_handle","kind":"animation_clip"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.interchange.transcode_musicxml_and_mei_scores" => ("record", json!([
                {"field":"score","type":"encoded_resource","kind":"musicxml|mei"},
                {"field":"q42","type":"encoded_resource","kind":"q42"},
                {"field":"receipt","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "hypermedia.manifold.fracture_voronoi_3d" => ("record", json!([
                {"field":"scene","type":"typed_handle","kind":"scene_graph_10d"},
                {"field":"assembly","type":"encoded_resource","kind":"10d"},
            ]), Value::Null),
            "hypermedia.layout.paginate_foliation_and_pdf_ua" => ("record", json!([
                {"field":"pdf","type":"encoded_resource","kind":"pdf"},
                {"field":"report","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "qualia.physics.solve_boltzmann_radiation_and_bateman" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.environment.solve_atmospheric_photochemistry_dispersion" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.acoustics.solve_inverse_rir_prb_and_bss" => ("record", json!([
                {"field":"audio","type":"typed_handle","kind":"audio_multichannel"},
                {"field":"qac","type":"encoded_resource","kind":"qac"},
                {"field":"q42","type":"encoded_resource","kind":"q42"},
                {"field":"receipt","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "hypermedia.geodesy.solve_elastic_fwi_adjoint_tomography" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"receivers","type":"typed_handle","kind":"field_lattice"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.medical.ingest_and_reconstruct_dicom" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.matter.solve_m64_thermochem_electro" => ("record", json!([
                {"field":"telemetry","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "hypermedia.fabrication.plan_cnc_subtractive_multiaxis" => ("record", json!([
                {"field":"residual_field","type":"typed_handle","kind":"field_lattice"},
                {"field":"preview","type":"encoded_resource","kind":"svg"},
                {"field":"receipt","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "qualia.anatomy.synthesize_invertebrate_and_ergonomic_10d" => ("record", json!([
                {"field":"scene","type":"typed_handle","kind":"scene_graph_10d"},
                {"field":"assembly","type":"encoded_resource","kind":"10d"},
                {"field":"rig","type":"typed_handle","kind":"skeletal_rig"},
                {"field":"container_10d","type":"encoded_resource","kind":"10d"},
                {"field":"report","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "qualia.sensing.bridge_etalon_nfc_radio_and_fusion" => ("record", json!([
                {"field":"report","type":"encoded_resource","kind":"json","when":"mode=synthetic_thermal_fusion"},
                {"field":"fixture","type":"encoded_resource","kind":"bin","when":"mode=encode_nfc1_fixture"},
                {"field":"receipt","type":"encoded_resource","kind":"q42","when":"mode=synthetic_thermal_fusion or verify_pinned_nfc1_fixture_receipt"},
            ]), Value::Null),
            "hypermedia.radiance.solve_adaptive_optics_and_nonlinear_nlse" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.ai.interpret_p64_sae_and_renyi_dp_federation" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.crypto.attest_vdf_roughtime_and_pq_threshold" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42","when":"check or verify mode"},
            ]), Value::Null),
            "hypermedia.radiance.solve_adjoint_prb_inverse_rendering" => ("record", json!([
                {"field":"fitted_luminance","type":"typed_handle","kind":"field_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.quantum.derive_hellmann_feynman_force_field" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.layout.decompile_tagged_pdf_ua" => ("record", json!([
                {"field":"document","type":"record","kind":"tagged_document"},
                {"field":"page","type":"record","kind":"page_layout","when":"source includes a retained page recipe"},
                {"field":"reflow_args","type":"record","kind":"pagination_arguments","when":"source includes a page recipe and font matches its digest"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.quantum.solve_dft_scf" => ("record", json!([
                {"field":"density_e_per_bohr3","type":"typed_handle","kind":"field_lattice"},
                {"field":"orbital_per_sqrt_bohr","type":"typed_handle","kind":"field_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.electromagnetics.solve_vlasov_maxwell_pic_plasma" => ("record", json!([
                {"field":"fields","type":"typed_handle","kind":"field_lattice"},
                {"field":"particles","type":"typed_handle","kind":"field_lattice"},
                {"field":"emf","type":"encoded_resource","kind":"emf"},
            ]), Value::Null),
            "qualia.quantum.solve_tdgl_cryogenic_superconductivity" => ("record", json!([
                {"field":"order_parameter","type":"typed_handle","kind":"field_lattice"},
                {"field":"flux_and_vortices","type":"typed_handle","kind":"field_lattice"},
                {"field":"current_x","type":"typed_handle","kind":"field_lattice"},
                {"field":"current_y","type":"typed_handle","kind":"field_lattice"},
                {"field":"link_x_phase_rad","type":"typed_handle","kind":"field_lattice"},
                {"field":"link_y_phase_rad","type":"typed_handle","kind":"field_lattice"},
            ]), Value::Null),
            "hypermedia.manifold.build_cluster_dag_and_impostor" => ("record", json!([
                {"field":"asset","type":"encoded_resource","kind":"10d"},
                {"field":"meshlet_positions_f32x3_le","type":"encoded_resource","kind":"bin"},
                {"field":"meshlet_triangle_indices_u16_le","type":"encoded_resource","kind":"bin"},
            ]), Value::Null),
            "hypermedia.electromagnetics.trace_rf_multipath_and_radar" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.chemistry.solve_reaction_network" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.quantum.evaluate_bio_coherence" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.vision.extract_biometrics_and_pose468" => ("record", json!([
                {"field":"clip","type":"typed_handle","kind":"animation_clip"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.chemistry.invert_flux_balance_and_rates" => ("record", json!([
                {"field":"cube","type":"typed_handle","kind":"tabular_cube","when":"op=directed_incidence_max_flow"},
                {"field":"forward_args","type":"record","kind":"normalized_exchange_arguments","when":"op=inverse_normalized_exchange"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.chemistry.step_molecular_dynamics" => ("record", json!([
                {"field":"trajectory","type":"typed_handle","kind":"field_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.electromagnetics.solve_adjoint_fdtd_inverse_design" => ("record", json!([
                {"field":"probe_history","type":"typed_handle","kind":"field_lattice"},
            ]), Value::Null),
            "qualia.medical.segment_volume_to_10d" => ("record", json!([
                {"field":"asset","type":"encoded_resource","kind":"10d"},
                {"field":"field","type":"encoded_resource","kind":"sdfb"},
                {"field":"report","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "qualia.medical.analyze_histopathology_and_radiomics" => ("record", json!([
                {"field":"labels","type":"typed_handle","kind":"field_lattice"},
                {"field":"report","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "qualia.crypto.prove_zk_deontic_pq_and_sanctuary" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.bio.solve_pbpk_pd_adme_clearance" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.biosphere.step_funga_and_trophic_web" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
                {"field":"flux_q42","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.anatomy.simulate_pathology_regime" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.neuro.solve_hodgkin_huxley_connectome_gtop" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.anatomy.synthesize_organ_system_10d" => ("record", json!([
                {"field":"scene","type":"typed_handle","kind":"scene_graph_10d"},
                {"field":"assembly","type":"encoded_resource","kind":"10d"},
                {"field":"hmc","type":"encoded_resource","kind":"hmc"},
                {"field":"report","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "qualia.wellfare.evaluate_clinical_burden_and_vault" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.anatomy.incise_resect_and_suture" => ("record", json!([
                {"field":"scene","type":"typed_handle","kind":"scene_graph_10d"},
                {"field":"assembly","type":"encoded_resource","kind":"10d"},
                {"field":"hmc","type":"encoded_resource","kind":"hmc"},
                {"field":"report","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "qualia.poet.bind_pulse_aura_hypermedia" => ("record", json!([
                {"field":"bound_session","type":"record","kind":"poet_event_binding"},
                {"field":"motion_trigger_receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.medical.export_dicom_seg_rtstruct" => ("record", json!([
                {"field":"seg","type":"encoded_resource","kind":"dcm"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.biology.fold_macromolecule_and_cell_sheet" => ("record", json!([
                {"field":"bead_accessibility","type":"typed_handle","kind":"field_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.bio.solve_grn_epigenetic_morphogenesis" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.fabrication.embed_digital_mesh_stego" => ("record", json!([
                {"field":"asset","type":"encoded_resource","kind":"10d","when":"prepare or embed mode"},
                {"field":"field","type":"encoded_resource","kind":"sdfb","when":"prepare or embed mode"},
                {"field":"creation_graph","type":"encoded_resource","kind":"q42","when":"prepare mode"},
                {"field":"receipt","type":"encoded_resource","kind":"q42","when":"embed or verify mode"},
                {"field":"lineage_receipt","type":"encoded_resource","kind":"q42","when":"embed or verify mode"},
            ]), Value::Null),
            "qualia.biosphere.step_funga_microbial_trophic_coupling" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
                {"field":"flux_q42","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.fabrication.embed_physical_print_stego" => ("record", json!([
                {"field":"asset","type":"encoded_resource","kind":"10d","when":"prepare mode"},
                {"field":"print_archive","type":"encoded_resource","kind":"3mf"},
                {"field":"finish_profile","type":"encoded_resource","kind":"json"},
                {"field":"creation_graph","type":"encoded_resource","kind":"q42","when":"prepare mode"},
                {"field":"receipt","type":"encoded_resource","kind":"q42","when":"embed or verify mode"},
                {"field":"lineage_receipt","type":"encoded_resource","kind":"q42","when":"embed or verify mode"},
            ]), Value::Null),
            "qualia.bio.simulate_immunology_wound_and_seir_network" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.crypto.evaluate_zk_mpc_and_vole_psi" => ("record", json!([
                {"field":"intersection_cardinality","type":"scalar","kind":"u64"},
                {"field":"verified_aggregate_sum","type":"scalar","kind":"u64"},
                {"field":"spdz_mac_verified","type":"scalar","kind":"bool"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.electromagnetics.reconstruct_sar_isar_tomography" => ("record", json!([
                {"field":"field","type":"typed_handle","kind":"field_lattice"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.manifold.measure_semantic_sections" => ("record", json!([
                {"field":"svg","type":"encoded_resource","kind":"svg"},
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.accessibility.compile_scene_focus_graph" => ("record", json!([
                {"field":"focus_graph","type":"encoded_resource","kind":"json"},
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.container.plan_hmc_semantic_working_set" => ("record", json!([
                {"field":"plan","type":"encoded_resource","kind":"json"},
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.provenance.rebase_hmc_review_anchors" => ("record", json!([
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.accessibility.audit_affordance_equivalence" => ("record", json!([
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.manifold.verify_semantic_lod_transition" => ("record", json!([
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.chronicle.bind_timed_semantic_cues" => ("record", json!([
                {"field":"vtt","type":"encoded_resource","kind":"vtt"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.discourse.bind_evidence_fragments" => ("record", json!([
                {"field":"annotation","type":"encoded_resource","kind":"json"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.radiance.plan_semantic_viewpoints" => ("record", json!([
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.kinetics.retarget_semantic_clip" => ("record", json!([
                {"field":"clip","type":"typed_handle","kind":"animation_clip"},
                {"field":"report","type":"encoded_resource","kind":"json"},
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.geospatial.synthesize_corridor_and_isogloss" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.inference.compile_conditioning_and_q42_journal" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.network.sync_solid_pod_and_qdnf_peer" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.inference.transcode_p64_ternary_and_sieve" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.geometry.advanced_cg_arrangements_and_meshing" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.twin.evaluate_ethogram_and_entity_affordance" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.graph.prove_equivalence_scrub_and_rdf_star" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.governance.evaluate_semantic_instruments_and_tk_bc" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.cooperative.evaluate_nine_stage_lifecycle_and_abac" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.anatomy.seal_cardiac_m64_emf_and_work_graph" => ("record", json!([
                {"field":"asset","type":"encoded_resource","kind":"10d"},
                {"field":"matter","type":"encoded_resource","kind":"m64"},
                {"field":"emf","type":"encoded_resource","kind":"emf"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.wellfair.evaluate_personal_health_and_guardianship" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.economics.solve_macro_market_portfolio_and_engineering" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.wellfair.enact_safeguards_dead_mans_and_accountability" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.kinetics.solve_homeostatic_graph_flux" => ("record", json!([
                {"field":"nodes","type":"list","kind":"node_flux"},
                {"field":"edges","type":"list","kind":"edge_flux"},
                {"field":"motors","type":"list","kind":"pga_motor"},
            ]), Value::Null),
            "hypermedia.cosmos.deep_space_rf_link" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.geodesy.export_cog_zarr_netcdf4" => ("record", json!([
                {"field":"archive","type":"encoded_resource","kind":"zarr.zip"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"}
            ]), Value::Null),
            "qualia.category.compute_pushout_pullback_and_kan_extension" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"}
            ]), Value::Null),
            "hypermedia.radiance.path_trace_scene" => ("record", json!([
                {"field":"radiance","type":"typed_handle","kind":"radiance_lattice"},
                {"field":"aovs","type":"typed_handle","kind":"field_lattice"},
                {"field":"primal_pixel_luminance","type":"typed_handle","kind":"field_lattice"}
            ]), Value::Null),
            "hypermedia.kinetics.synthesize_laban_choreography" => ("record", json!([
                {"field":"clip","type":"typed_handle","kind":"animation_clip"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "qualia.compute.inspect_and_dequantize_p64" => ("record", json!([
                {"field":"table","type":"typed_handle","kind":"tabular_cube"},
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
                {"field":"safetensors","type":"encoded_resource","kind":"safetensors"},
            ]), Value::Null),
            "qualia.interchange.transcode_scientific_cae_bio_and_qudt" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"}
            ]), Value::Null),
            "qualia.compute.dispatch_wgsl_forge" => ("record", json!([
                {"field":"output","type":"typed_handle","kind":"field_lattice"},
                {"field":"receipt","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "qualia.manifold.compile_ssm1_and_eont_affordances" => ("record", json!([
                {"field":"container_10d","type":"encoded_resource","kind":"10d"},
                {"field":"basis","type":"typed_handle","kind":"field_lattice"},
            ]), Value::Null),
            "qualia.topology.compute_persistent_homology_ddg" => ("record", json!([
                {"field":"barcode","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.acoustics.room_rir_and_hoa_render" => ("record", json!([
                {"field":"hoa","type":"encoded_resource","kind":"qac"},
                {"field":"binaural","type":"typed_handle","kind":"audio_multichannel","when":"an HRIR is supplied"},
                {"field":"primal_etc","type":"typed_handle","kind":"field_lattice"}
            ]), Value::Null),
            "hypermedia.surface.node_graph_eval" => ("record", json!([
                {"field":"materialx","type":"encoded_resource","kind":"mtlx"},
                {"field":"graph","type":"encoded_resource","kind":"json"},
            ]), Value::Null),
            "hypermedia.geodesy.synthesize_volumetric_cloud" => ("record", json!([
                {"field":"density","type":"typed_handle","kind":"field_lattice"},
                {"field":"phase_lut","type":"typed_handle","kind":"field_lattice"},
                {"field":"preview","type":"typed_handle","kind":"radiance_lattice"},
            ]), Value::Null),
            "hypermedia.manifold.scatter_biome_instances" => ("record", json!([
                {"field":"scene","type":"typed_handle","kind":"scene_graph_10d"},
                {"field":"assembly","type":"encoded_resource","kind":"10d"},
            ]), Value::Null),
            "qualia.linguistics.analyze_spatiotemporal_semiotics" => ("record", json!([
                {"field":"drift_graph","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.geodesy.synthesize_ocean_spectrum" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"fields","type":"typed_handle","kind":"field_lattice"},
            ]), Value::Null),
            "qualia.compute.compile_p64_surrogate_cartridge" => ("record", json!([
                {"field":"p64","type":"encoded_resource","kind":"p64"},
                {"field":"p64_extension","type":"encoded_resource","kind":"p64ext"},
                {"field":"calibration_receipt","type":"encoded_resource","kind":"q42"},
            ]), Value::Null),
            "hypermedia.acoustics.solve_continuum_wave_3d" => ("record", json!([
                {"field":"fields","type":"typed_handle","kind":"field_lattice"},
                {"field":"audio","type":"typed_handle","kind":"audio_multichannel"},
            ]), Value::Null),
            "hypermedia.kinetics.sew_and_drape_garment" => ("record", json!([
                {"field":"mesh","type":"typed_handle","kind":"attributed_mesh"},
                {"field":"strain","type":"typed_handle","kind":"tabular_cube"},
            ]), Value::Null),
            "qualia.vision.process_and_track" => ("record", json!([
                {"field":"tracks","type":"encoded_resource","kind":"json"},
                {"field":"evidence","type":"encoded_resource","kind":"q42"}
            ]), Value::Null),
            "hypermedia.locus.draft_cad_and_trace_vector" => ("record", json!([
                {"field":"profiles","type":"profile_handle_list"}
            ]), Value::Null),
            "hypermedia.kinetics.solve_shallow_water_25d" => ("record", json!([
                {"field":"water","type":"typed_handle","kind":"field_lattice"}
            ]), Value::Null),
            "qualia.rights.evaluate_agreement_and_pool" => ("record", json!([
                {"field":"attestation_receipt","type":"encoded_resource","kind":"q42"}
            ]), Value::Null),
            "hypermedia.geodesy.gp_spatial_analyze" => ("record", json!([
                {"field":"directions","type":"typed_handle","kind":"field_lattice","when":"solving D8 drainage"},
                {"field":"basins","type":"typed_handle","kind":"field_lattice","when":"solving watersheds"},
                {"field":"visibility","type":"typed_handle","kind":"field_lattice","when":"calculating a viewshed"},
                {"field":"hillshade","type":"typed_handle","kind":"field_lattice","when":"calculating hillshade"}
            ]), Value::Null),
            "hypermedia.geodesy.solve_geomorph_stage"
            | "hypermedia.manifold.accumulate_snow_lakes" => ("record", json!([]), Value::Null),
            "qualia.knowledge.query_q42_sparql_and_n3"
            | "qualia.knowledge.solve_causal_temporal_rcc8" => ("record", json!([]), Value::Null),
            "hypermedia.manifold.configure_interactive_cutaway" => ("record", json!([
                {"field":"descriptor","type":"encoded_resource","kind":"json"}
            ]), Value::Null),
            "hypermedia.manifold.bake_gaussian_surfels_10d" => ("record", json!([
                {"field":"asset","type":"encoded_resource","kind":"10d"},
                {"field":"turntable_preview","type":"typed_handle","kind":"radiance_lattice"}
            ]), Value::Null),
            "qualia.science.validate_c02_passivity_and_oracles"
            | "qualia.geometry.repair_polygon_soup_and_ddg_poisson" => ("record", json!([
                {"field":"certificate","type":"encoded_resource","kind":"q42"}
            ]), Value::Null),
            "qualia.q42.encode_spatial_predicates_and_situated_place_time" => ("record", json!([
                {"field":"q42","type":"encoded_resource","kind":"q42","when":"packing a coordinate or link"},
                {"field":"substrate","type":"encoded_resource","kind":"subq","when":"composing a model substrate"}
            ]), Value::Null),
            "qualia.solvers.evaluate_graph_opt_active_learning_and_special_fns" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
                {"field":"cube","type":"typed_handle","kind":"tabular_cube"}
            ]), Value::Null),
            "qualia.geospatial.evaluate_gis_gp_terrain_and_datacubes" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"},
                {"field":"field","type":"typed_handle","kind":"field_lattice","when":"op=decode_zarr_v3_zip"},
                {"field":"asset","type":"encoded_resource","kind":"10d","when":"sealing a datacube raster"}
            ]), Value::Null),
            "hypermedia.acoustics.bridge_qac_ultrasound_haptic_and_bioacoustics" => ("operation_specific", json!([
                {"field":"qac","type":"encoded_resource","kind":"qac","when":"writing ultrasound or haptic measurements"},
                {"field":"q42","type":"encoded_resource","kind":"q42","when":"detecting vocalizations"},
                {"field":"pcm_f32le","type":"encoded_resource","kind":"f32le","when":"rendering a tensor voice"}
            ]), Value::Null),
            "qualia.vision.super_resolve_and_compile_10d_observations" => ("record", json!([
                {"field":"voxels","type":"encoded_resource","kind":"f32le","when":"resampling an isotropic volume"},
                {"field":"image","type":"image_handle","when":"resolving an RGB raster"}
            ]), Value::Null),
            "hypermedia.kinetics.compile_skl2_rig" => ("record", json!([
                {"field":"container_10d","type":"encoded_resource","kind":"10d"},
                {"field":"rig","type":"typed_handle","kind":"skeletal_rig"}
            ]), Value::Null),
            "hypermedia.kinetics.simulate_vehicle_aero_hydro_6dof" => ("record", json!([
                {"field":"clip","type":"typed_handle","kind":"animation_clip"},
                {"field":"telemetry","type":"typed_handle","kind":"tabular_cube"}
            ]), Value::Null),
            "hypermedia.kinetics.solve_multibody_dca_ik"
            | "hypermedia.kinetics.bake_aeroelastic_wind" => ("record", json!([
                {"field":"clip","type":"typed_handle","kind":"animation_clip"}
            ]), Value::Null),
            "hypermedia.kinetics.fit_and_verify_articulation_sweep" => ("record", json!([
                {"field":"model","type":"encoded_resource","kind":"q42"},
                {"field":"receipt","type":"encoded_resource","kind":"json"}
            ]), Value::Null),
            "qualia.network.dispatch_qdnf_ilp_ebpf_and_os_services" => ("record", json!([
                {"field":"asset","type":"encoded_resource","kind":"10d","when":"sealing a GeoTIFF"},
                {"field":"receipt","type":"encoded_resource","kind":"q42","when":"sealing a GeoTIFF"}
            ]), Value::Null),
            "qualia.vibe.decompile_merge_and_legislate_cof" => ("operation_specific", json!([
                {"field":"source","type":"encoded_resource","kind":"vibe","when":"merging without conflicts"},
                {"field":"rgba8","type":"encoded_resource","kind":"rgba8","when":"transcoding KTX2"}
            ]), Value::Null),
            "hypermedia.chronicle.interchange_aaf_omf_mxf_and_yin_midi" => ("record", json!([
                {"field":"midi","type":"encoded_resource","kind":"mid","when":"transcribing monophonic audio"},
                {"field":"receipt","type":"encoded_resource","kind":"q42","when":"transcribing monophonic audio"}
            ]), Value::Null),
            "qualia.sensing.optimize_csi_privshield_and_bvp"
            | "hypermedia.kinetics.solve_explicit_dynamic_fem_and_buckling" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"}
            ]), Value::Null),
            "qualia.geospatial.erode_multires_rivers_and_ce_droplets" => ("operation_specific", json!([
                {"field":"maps","type":"typed_handle","kind":"field_lattice","when":"processing a river or droplet raster"},
                {"field":"asset","type":"encoded_resource","kind":"10d","when":"sealing an authored raster plane"}
            ]), Value::Null),
            "qualia.cooperative.reconcile_landed_cost_retainage_and_web_gis" => ("operation_specific", json!([
                {"field":"journal_draft","type":"encoded_resource","kind":"json","when":"allocating landed costs"},
                {"field":"receipt","type":"encoded_resource","kind":"q42","when":"allocating landed costs"}
            ]), Value::Null),
            "hypermedia.vector.compute_rolling_ball_concave_hull_and_offset" => ("operation_specific", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42","when":"evaluating signed Bézier metrics"}
            ]), Value::Null),
            "qualia.discourse.ingest_office_smartart_xlsb_ods_and_formcalc" => ("operation_specific", json!([
                {"field":"cube","type":"typed_handle","kind":"tabular_cube","when":"importing a worksheet"}
            ]), Value::Null),
            "hypermedia.image.segment_slic_grabcut_sky_and_roto_matte" => ("record", json!([
                {"field":"labels","type":"typed_handle","kind":"field_lattice","when":"segmenting superpixels"},
                {"field":"mask","type":"typed_handle","kind":"field_lattice","when":"filtering a mask"}
            ]), Value::Null),
            "hypermedia.kinetics.filter_gpu_vbd_contact_and_seam_links" => ("record", json!([
                {"field":"topology.gtop","type":"encoded_resource","kind":"gtop","when":"extracting seam links"},
                {"field":"receipt","type":"encoded_resource","kind":"q42","when":"extracting seam links"},
                {"field":"candidates","type":"encoded_resource","kind":"json","when":"filtering spatial contact on the native GPU"}
            ]), Value::Null),
            "hypermedia.image.roundtrip_psd_smart_filters_and_multi_app_brushes" => ("record", json!([
                {"field":"asset","type":"encoded_resource","when":"preserving a PSD or PSB"},
                {"field":"block","type":"encoded_resource","kind":"bin","when":"extracting a layer-info block"}
            ]), Value::Null),
            "qualia.container.pack_and_diff_hmc" => ("record", json!([
                {"field":"hmc","type":"encoded_resource","kind":"hmc"},
                {"field":"delta","type":"encoded_resource","kind":"q42","when":"baseline supplied"}
            ]), Value::Null),
            "qualia.math.solve_symbolic_cas_and_ode" => ("record", json!([
                {"field":"model","type":"encoded_resource","kind":"q42","when":"expression result"}
            ]), Value::Null),
            "qualia.statistics.evaluate_inference_and_econometrics"
            | "qualia.knowledge.evaluate_epistemic_jural_and_blackboard"
            | "hypermedia.physics.homogenize_multiphase_and_cartoon_kinetics" => ("record", json!([
                {"field":"receipt","type":"encoded_resource","kind":"q42"}
            ]), Value::Null),
            "qualia.epistemics.audit_research_and_grounding" => ("record", json!([
                {"field":"report","type":"encoded_resource","kind":"q42"}
            ]), Value::Null),
            "hypermedia.live.author_dmx_hbbtv_and_portal_world" => ("record", json!([
                {"field":"hmc","type":"encoded_resource","kind":"hmc"},
                {"field":"manifest","type":"encoded_resource","kind":"json"}
            ]), Value::Null),
            "qualia.topology.harbour_gtop_geometry_graphs" => ("record", json!([
                {"field":"gtop","type":"encoded_resource","kind":"gtop"},
                {"field":"cbld","type":"encoded_resource","kind":"cbld"}
            ]), Value::Null),
            "hypermedia.manifold.compile_10d" => ("encoded_resource", json!([]), Value::Null),
            "hypermedia.manifold.compile_10d_pbr" => ("encoded_resource", json!([{"field":"textures","type":"array","items":{"digest":"sha256","mime_type":"image MIME","resource":"encoded_resource"},"packaging":"automatically retained by files/v2 as textures/sha256/<digest>"}]), Value::Null),
            "hypermedia.manifold.sweep_assembly_profile" => ("record", json!([{"field":"container_10d","type":"encoded_resource","kind":"10d"}]), Value::Null),
            "hypermedia.surface.unwrap_slim" => ("record", json!([{"field":"container_10d","type":"encoded_resource","kind":"10d"},{"field":"mesh","type":"mesh_handle"},{"field":"source_vertices","type":"array","when":"mode=atlas"},{"field":"uvs","type":"array"}]), Value::Null),
            "hypermedia.chronicle.capture_deterministic_movie" => ("profile_dependent", json!([{"field":"manifest_resource","type":"encoded_resource","kind":"json","when":"profile=png-sequence","packaging":"leaf PNG/WAV dependencies automatically retained by files/v2"},{"field":"mp4","type":"encoded_resource","when":"profile=preview"}]), Value::Null),
            "hypermedia.surface.pack_pbr_set" => ("record", json!([{"field":"colour","type":"encoded_resource","kind":"png"},{"field":"normal","type":"encoded_resource","kind":"png"},{"field":"orm","type":"encoded_resource","kind":"png"}]), Value::Null),
            "hypermedia.surface.synthesize_maps" => ("record", json!([{"field":"colour","type":"rgba_handle"},{"field":"normal","type":"rgba_handle"},{"field":"orm","type":"rgba_handle"}]), json!({"operation":"hypermedia.surface.pack_pbr_set","arguments":{"colour":"$result.colour","normal":"$result.normal","orm":"$result.orm"}})),
            "hypermedia.radiance.pack_sprite_sheet" => ("record", json!([{"field":"png","type":"encoded_resource","kind":"png"},{"field":"image","type":"rgba_handle"},{"field":"frames","type":"array"}]), Value::Null),
            "hypermedia.surface.bake_pbr_materialx" => ("record", json!([{"field":"materialx_resource","type":"encoded_resource"},{"field":"psd_resource","type":"encoded_resource"},{"field":"ao_image","type":"rgba_handle"},{"field":"normal_image","type":"rgba_handle"}]), Value::Null),
            "hypermedia.radiance.create_rgba" | "hypermedia.layout.render_ui_sheet" => ("rgba_handle", json!([]), json!({"operation":"hypermedia.radiance.encode_rgba_png","arguments":{"image":"$result"}})),
            "hypermedia.radiance.encode_rgba_png" | "hypermedia.radiance.encode_rgb_png" | "hypermedia.locus.project_svg" | "hypermedia.acoustics.encode_wav" | "hypermedia.acoustics.encode_aac" | "hypermedia.electromagnetics.compile_emf" => ("encoded_resource", json!([]), Value::Null),
            _ => ("operation_specific", json!([]), Value::Null),
        };
        let attribute_edit = matches!(operation.id,
            "hypermedia.manifold.boolean_3d" | "hypermedia.manifold.simplify" | "hypermedia.manifold.radial_euler"
            | "hypermedia.manifold.bevel" | "hypermedia.manifold.inset" | "hypermedia.manifold.solidify"
            | "hypermedia.manifold.bisect_plane" | "hypermedia.manifold.subdivide_catmull_clark"
            | "hypermedia.manifold.evaluate_field" | "hypermedia.manifold.realize_instances" | "hypermedia.manifold.mesh_to_sdf_brick"
            | "hypermedia.manifold.sweep_assembly_profile" | "hypermedia.surface.unwrap_slim");
        let extensions = if attribute_edit {json!({"attribute_policy":{"type":"string","enum":["preserve","discard"],"default":"preserve",
            "semantics":"Preserve refuses geometry-only edits of attributed meshes unless a supported remap transports them. Discard explicitly creates a geometry revision requiring surface rebind/bake; source remains intact.",
            "unwrap_exception":"Untextured slim/atlas preserves the supported remapped mesh; atlas exposes source_vertices. Textured rebaking is required."}})} else {json!({})};
        json!({"id": operation.id, "execution_host":"synth.production", "domain": operation.domain,
            "codomain": operation.codomain, "arguments": operation.arguments,
            "input_extensions": extensions,
            "output":{"type":kind,"resource_fields":fields,"encoder":encoder},
            "handle_lifetime":"one source evaluation", "metadata_provider_call":"schema hash only; does not execute production"})
    }).collect();
    let recipes = output_recipes();
    json!({"schema":"synth.operation-contracts/v1", "count":operations.len(), "operations": operations,
        "fallback":"operation_specific means consult the codomain and implementation contract; no namespace-based export inference is permitted",
        "production_profiles":[{"id":"preview","resource_mib":16,"aggregate_mib":64,"package_mib":80,"logical_work_units":50000000},{"id":"production16","resource_mib":128,"aggregate_mib":2048,"package_mib":2304,"logical_work_units":2000000000,"raster_max_dimension":2048,"precision":"RGBA16 UNORM with sRGB/linear PNG metadata"},{"id":"png-sequence","operation":"hypermedia.chronicle.capture_deterministic_movie","frames":64,"raster_max_dimension":2048,"precision":"congruent RGBA8 or RGBA16 frames","timing":"exact rational manifest, optional sample-aligned PCM WAV"}],
        "resource_type":{"type":"encoded_resource", "export_binding":{"key":"output.ext","resource":"$encoded"}},
        "output_recipes": recipes,
        "recipes": recipes})
}

fn preview_command(executable: &Path, asset: &Path, destination: &Path, package: &Path, asset_key: &str) -> Command {
    let mut command = Command::new(executable);
    command.arg("preview").arg("--asset").arg(asset).arg("--out").arg(destination)
        .arg("--hmc").arg(package).arg("--asset-key").arg(asset_key);
    command
}

pub(super) fn native_preview(asset: &Path, destination: &Path, package: &Path, asset_key: &str) -> Result<(), ToolError> {
    let executable = std::env::current_exe().map_err(|_| ToolError::new("preview_failed", "Cannot locate Synth executable"))?;
    let mut child = preview_command(&executable,asset,destination,package,asset_key)
        .stdout(Stdio::null()).stderr(Stdio::piped()).spawn()
        .map_err(|e| ToolError::new("preview_failed", format!("Cannot start native review: {e}")))?;
    let stderr = child.stderr.take().unwrap();
    // Drain stderr even after the diagnostic cap to avoid blocking a renderer subprocess.
    let reader = std::thread::spawn(move || {
        let mut retained = Vec::new();
        let mut stream = stderr;
        let mut buffer = [0u8; 4096];
        while let Ok(count) = stream.read(&mut buffer) {
            if count == 0 { break; }
            let keep = count.min(2048usize.saturating_sub(retained.len()));
            retained.extend_from_slice(&buffer[..keep]);
        }
        String::from_utf8_lossy(&retained).into_owned()
    });
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let diagnostic = reader.join().unwrap_or_default();
                return if status.success() { Ok(()) } else {
                    Err(ToolError::new("preview_failed", format!("Native review exited {status}; {diagnostic}")))
                };
            }
            Ok(None) if start.elapsed() < Duration::from_secs(180) => std::thread::sleep(Duration::from_millis(50)),
            outcome => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                return Err(ToolError::new("preview_failed", if outcome.is_err() { "Native review process could not be observed" } else { "Native review exceeded the 180 second time budget; completed exports remain available" }));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_preview_forwards_canonical_package_and_asset_key_without_shell() {
        let command = preview_command(Path::new("synth.exe"),Path::new("export/mesh.10d"),Path::new("review"),Path::new("asset.hmc"),"outputs/mesh.10d");
        let args:Vec<_> = command.get_args().map(|v|v.to_string_lossy().into_owned()).collect();
        assert_eq!(args, ["preview","--asset","export/mesh.10d","--out","review","--hmc","asset.hmc","--asset-key","outputs/mesh.10d"]);
    }

    #[test]
    fn output_recipes_declare_emf_v4_and_qac_v1_formats() {
        let recipes = output_recipes();
        let list = recipes.as_array().expect("output_recipes returns array");
        let emf = list.iter().find(|r| r["id"] == "emf").expect("emf recipe");
        assert_eq!(emf["resource_key"], "field.emf");
        assert_eq!(emf["resource_kind"], "emf");
        assert_eq!(emf["container_magic"], "Q42EMF64");
        assert_eq!(emf["container_version"], 4);

        let qac = list.iter().find(|r| r["id"] == "qac").expect("qac recipe");
        assert_eq!(qac["resource_key"], "signal.qac");
        assert_eq!(qac["resource_kind"], "qac");
        assert_eq!(qac["container_magic"], "qac\0\0\0\0\0");
        assert_eq!(qac["container_version"], 1);

        let cat = catalogue();
        let compile_emf = cat["operations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|op| op["id"] == "hypermedia.electromagnetics.compile_emf")
            .unwrap();
        assert_eq!(compile_emf["output"]["type"], "encoded_resource");

        let encode_aac = cat["operations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|op| op["id"] == "hypermedia.acoustics.encode_aac")
            .unwrap();
        assert_eq!(encode_aac["output"]["type"], "encoded_resource");
        assert_eq!(cat["output_recipes"], recipes);
        assert_eq!(cat["recipes"], recipes);
    }
}
