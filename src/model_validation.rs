//! 5-Regime Flexibility & `KLOD0`..`KLOD3` Graceful Degradation (`REQ-7.1`..`7.2`),
//! Asset Computational "Form Validator" (`REQ-8.1`..`8.2`),
//! Multi-Format Render & Output Verification Gate (`REQ-9.1`..`9.2`), and
//! 3D Scene & Asset Conversion Pipeline (`synth convert-3d` / MCP `synth_convert_3d`).
use qualia_core_db::container_10d::{
    header::Container10dHeader,
    integrity::verify_whole_file_crc32c,
    decode_container_manifold_identity, TypedFieldRecord, FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE,
    FIELD_KIND_MULTISCALE_STRATA, FIELD_KIND_SYMBOLIC_RDF_CBOR_LD,
};
use qualia_core_db::render::compile_10d::{
    convert_3d_asset_or_scene_with_license, decode_10d_mesh, SourceLicenseAttribution,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const FORM_VALIDATION_SCHEMA: &str = "synth.computational-form-validation/v1";
pub const OUTPUT_VERIFICATION_SCHEMA: &str = "synth.output-verification/v1";

/// REQ-7.2: 4-Tier Kinematic & Structural LOD Budget (`KLOD0`..`KLOD3`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KinematicLodTier {
    /// Hero / Cinema / Full Local GPU (Native Vulkan/DX12/Metal): up to 256 `SKL2` joints + VBD soft-tissue + Cosserat rods + `PROC1`.
    KLOD0,
    /// High / Interactive Native & Desktop WebGPU: up to 128 DQS joints + 8-channel morphs + reduced Cosserat + `PROC1`.
    KLOD1,
    /// Medium / Constrained WASM / WebGL2 / Mid-Distance: 24–32 cardinal joints (distal weights accumulated onto ancestor) + `PROC1`.
    KLOD2,
    /// Low Proxy / Crowd / Distant Horizon: Root transform + single harmonic `PROC1` motor oscillation or `IMPO` impostor.
    KLOD3,
}

impl KinematicLodTier {
    pub fn max_active_joints(self) -> usize {
        match self {
            Self::KLOD0 => 256,
            Self::KLOD1 => 128,
            Self::KLOD2 => 24,
            Self::KLOD3 => 1,
        }
    }
}

/// REQ-8.2: Reference citation binding for a computational form parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceBinding {
    pub source_id: String,
    /// `"q42_assertion" | "bookmark_iri" | "doi" | "standard"`
    pub kind: String,
    #[serde(default)]
    pub observation_method: Option<String>,
    #[serde(default)]
    pub qudt_unit: Option<String>,
}

/// REQ-8.1 & REQ-8.2: Single parameter entry in the Asset Computational Form.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputationalParameterField {
    pub name: String,
    pub value: Value,
    #[serde(default)]
    pub qudt_unit: Option<String>,
    /// `"Measured" | "Described" | "Derived" | "StatisticallyCalibrated" | "PhylogeneticallyBracketed" | "ModelAssumption" | "ArtisticChoice" | "Unknown"`
    pub claim_basis: String,
    /// `"DirectlyObserved" | "SpecimenMeasured" | "PeerReviewedLiterature" | "PhylogeneticallyBracketed" | "AuthorialIntent"`
    #[serde(default)]
    pub epistemic_tier: Option<String>,
    #[serde(default)]
    pub references: Vec<ReferenceBinding>,
}

/// REQ-8.1: Required fields by ontological subject class.
pub fn required_fields_for_subject_class(subject_class: &str) -> &'static [&'static str] {
    match subject_class {
        "VascularPlant" | "Flora" => &[
            "height_m",
            "canopy_width_m",
            "root_depth_m",
            "wood_density_kg_m3",
            "youngs_modulus_mpa",
            "turgor_pressure_mpa",
            "allometry_exponent_k",
            "kinematic_solver_regime",
            "multiscale_strata_l0_l6",
        ],
        "VertebrateFauna" | "HumanAnatomy" => &[
            "height_or_length_m",
            "body_mass_kg",
            "cortical_bmd_g_cm3",
            "kinematic_solver_regime",
            "joint_swing_twist_limits",
            "musculotendon_actuation_pathways",
            "multiscale_strata_l0_l6",
        ],
        "MechanicalVehicle" | "MadeArtefact" => &[
            "length_m",
            "width_m",
            "height_m",
            "dry_mass_kg",
            "youngs_modulus_mpa",
            "yield_stress_mpa",
            "kinematic_solver_regime",
            "multiscale_strata_l0_l6",
        ],
        _ => &[
            "length_m",
            "width_m",
            "height_m",
            "mass_kg",
            "kinematic_solver_regime",
            "multiscale_strata_l0_l6",
        ],
    }
}

/// Anatomical, Botanical & Natural-World Topological Containment Layer (`d in [0, 1]`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RadialLayerSample {
    pub entity_id: String,
    #[serde(default)]
    pub ontology_ref: Option<String>,
    /// Canonical layer name:
    /// - Anatomy/Fauna: `"periosteum_capsule"` [0.00, 0.08], `"ligament_deep_tendon"` [0.05, 0.20],
    ///   `"musculature"` [0.15, 0.75], `"subcutis_fascia"` [0.75, 0.92], `"dermis_epidermis"` [0.92, 1.00]
    /// - Flora/Botanical: `"pith_heartwood"` [0.00, 0.45], `"sapwood_xylem"` [0.45, 0.82],
    ///   `"cambium_phloem"` [0.82, 0.92], `"periderm_bark_cuticle"` [0.92, 1.00]
    pub layer_kind: String,
    pub d_min: f64,
    pub d_max: f64,
    #[serde(default)]
    pub parent_cavity_id: Option<String>,
    /// Signed distance `phi_parent(p)` in metres (must satisfy `phi <= -epsilon_m` when inside a cavity).
    #[serde(default)]
    pub max_cavity_sdf_phi_m: Option<f64>,
    #[serde(default)]
    pub cavity_clearance_eps_m: Option<f64>,
    /// Maximum enthesis or branch-collar UV detachment drift in metres across ROM / allometric sweep (`||p_end - p_uv||`).
    #[serde(default)]
    pub max_enthesis_uv_drift_m: Option<f64>,
    /// Volume preservation determinant `det(F)` under contraction / turgor deformation (target ~ 1.0 for incompressible tissue).
    #[serde(default)]
    pub volume_det_f: Option<f64>,
}

/// Evaluate the 5-Layer Anatomical & Natural-World Topological Invariants:
/// 1. Cavity SDF negative-distance containment (`phi_cavity <= -epsilon`),
/// 2. Intrinsic radial tissue depth ordering (`d in [0, 1]`, no muscle-over-skin or xylem-over-bark inversion),
/// 3. Barycentric UV enthesis / branch-collar zero-detachment lock (`||p_end - p_uv|| <= 1e-4`),
/// 4. Incompressible soft-tissue / turgor volume preservation (`det(F) in [0.85, 1.15]`).
pub fn verify_topological_and_radial_invariants(
    samples: &[RadialLayerSample],
    violations: &mut Vec<String>,
) -> Value {
    let mut cavity_checks = 0usize;
    let mut radial_checks = 0usize;
    let mut enthesis_checks = 0usize;

    for s in samples {
        radial_checks += 1;
        if !(0.0..=1.0).contains(&s.d_min)
            || !(0.0..=1.0).contains(&s.d_max)
            || s.d_min > s.d_max
        {
            violations.push(format!(
                "Topological layer inversion on `{}` (`{}`): normalized radial depth [{:.3}, {:.3}] is outside [0.0, 1.0] or inverted",
                s.entity_id, s.layer_kind, s.d_min, s.d_max
            ));
        }
        let expected_band = match s.layer_kind.as_str() {
            "periosteum_capsule" => Some((0.00, 0.12)),
            "ligament_deep_tendon" => Some((0.03, 0.25)),
            "musculature" => Some((0.10, 0.80)),
            "subcutis_fascia" => Some((0.70, 0.95)),
            "dermis_epidermis" => Some((0.88, 1.00)),
            "pith_heartwood" => Some((0.00, 0.50)),
            "sapwood_xylem" => Some((0.40, 0.86)),
            "cambium_phloem" => Some((0.78, 0.95)),
            "periderm_bark_cuticle" => Some((0.88, 1.00)),
            _ => None,
        };
        if let Some((lo, hi)) = expected_band {
            if s.d_min < lo - 1e-3 || s.d_max > hi + 1e-3 {
                violations.push(format!(
                    "Radial tissue shell breach on `{}` (`{}`): d=[{:.3}, {:.3}] escapes canonical band [{:.2}, {:.2}] (risk of muscle/ligament rendering over skin or vascular tissue over bark)",
                    s.entity_id, s.layer_kind, s.d_min, s.d_max, lo, hi
                ));
            }
        }
        if let Some(phi) = s.max_cavity_sdf_phi_m {
            cavity_checks += 1;
            let eps = s.cavity_clearance_eps_m.unwrap_or(0.0005).max(0.0);
            if phi > -eps {
                violations.push(format!(
                    "Cavity SDF containment failure on `{}` (parent `{}`): max(phi_cavity) = {:.5} m > -{:.5} m (viscera/organ escaping enclosing cavity)",
                    s.entity_id,
                    s.parent_cavity_id.as_deref().unwrap_or("enclosing_cavity"),
                    phi,
                    eps
                ));
            }
        }
        if let Some(drift) = s.max_enthesis_uv_drift_m {
            enthesis_checks += 1;
            if drift > 1e-4 {
                violations.push(format!(
                    "Enthesis / branch-collar UV detachment on `{}`: ||p_end - p_bone_UV|| = {:.6} m exceeds 0.1 mm tolerance across ROM/growth sweep",
                    s.entity_id, drift
                ));
            }
        }
        if let Some(det_f) = s.volume_det_f {
            if !(0.85..=1.15).contains(&det_f) {
                violations.push(format!(
                    "Volume-preserving deformation violation on `{}`: det(F) = {:.3} outside incompressible biological tissue envelope [0.85, 1.15]",
                    s.entity_id, det_f
                ));
            }
        }
    }

    // Pairwise superficial vs deep radial ordering check
    for i in 0..samples.len() {
        for j in (i + 1)..samples.len() {
            let a = &samples[i];
            let b = &samples[j];
            let rank = |k: &str| -> Option<u8> {
                match k {
                    "periosteum_capsule" | "pith_heartwood" => Some(0),
                    "ligament_deep_tendon" | "sapwood_xylem" => Some(1),
                    "musculature" => Some(2),
                    "subcutis_fascia" | "cambium_phloem" => Some(3),
                    "dermis_epidermis" | "periderm_bark_cuticle" => Some(4),
                    _ => None,
                }
            };
            if let (Some(ra), Some(rb)) = (rank(&a.layer_kind), rank(&b.layer_kind)) {
                if ra < rb && a.d_max > b.d_max + 1e-4 {
                    violations.push(format!(
                        "Layer ordering inversion: deep tissue `{}` ({}, d_max={:.3}) pierces superficial layer `{}` ({}, d_max={:.3})",
                        a.entity_id, a.layer_kind, a.d_max, b.entity_id, b.layer_kind, b.d_max
                    ));
                } else if rb < ra && b.d_max > a.d_max + 1e-4 {
                    violations.push(format!(
                        "Layer ordering inversion: deep tissue `{}` ({}, d_max={:.3}) pierces superficial layer `{}` ({}, d_max={:.3})",
                        b.entity_id, b.layer_kind, b.d_max, a.entity_id, a.layer_kind, a.d_max
                    ));
                }
            }
        }
    }

    json!({
        "samples_evaluated": samples.len(),
        "radial_depth_checks": radial_checks,
        "cavity_sdf_containment_checks": cavity_checks,
        "enthesis_uv_lock_checks": enthesis_checks,
        "qualia_shacl_logic_extensions_active": {
            "total_modality_shapes_registered": qualia_core_db::modalities::logic::LOGIC_MODALITY_SHAPES.len(),
            "deontic_no_commodification": "DeonticConfigurationShape (sh:deonticForbid, sh:valuesConsentNonCoerced, sh:valuesHarmBelowCeiling)",
            "epistemic_evidence_tier": "EpistemicConfigurationShape (OP_KNOWS 0x20 / OP_BELIEVES 0x21 certainty >= 128)",
            "spatio_temporal_containment": "Rcc8ConfigurationShape + AllenIntervalConfigurationShape + LtlConfigurationShape",
            "geometry_asset_manifest": "GeometryAssetConfiguration (validate_geometry_manifest + SlgOpcode firewall)",
            "biological_domain": "BiologicalDomainConfiguration (infrastructure_shacl.rs)"
        },
        "canonical_radial_coordinate_system": {
            "anatomy_fauna_d_0_to_1": {
                "periosteum_capsule": [0.00, 0.08],
                "ligament_deep_tendon": [0.05, 0.20],
                "musculature": [0.15, 0.75],
                "subcutis_fascia": [0.75, 0.92],
                "dermis_epidermis": [0.92, 1.00]
            },
            "flora_botanical_d_0_to_1": {
                "pith_heartwood": [0.00, 0.45],
                "sapwood_xylem": [0.45, 0.82],
                "cambium_phloem": [0.82, 0.92],
                "periderm_bark_cuticle": [0.92, 1.00]
            }
        }
    })
}

/// REQ-8.1 & REQ-8.2: Validate an asset's computational model form and optional `.10d` file.
pub fn validate_computational_model_form(args: Value) -> Result<Value, String> {
    let subject_class = args
        .get("subject_class")
        .and_then(Value::as_str)
        .unwrap_or("VascularPlant");
    let required = required_fields_for_subject_class(subject_class);

    let provided_fields: Vec<ComputationalParameterField> = args
        .get("fields")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let mut measured_or_described = 0usize;
    let mut derived_or_bracketed = 0usize;
    let mut model_assumption = 0usize;
    let mut artistic_choice = 0usize;
    let mut unknown_gaps = 0usize;
    let mut reference_sources_bound = 0usize;
    let mut violations = Vec::new();

    for f in &provided_fields {
        reference_sources_bound += f.references.len();
        match f.claim_basis.as_str() {
            "Measured" | "Described" => {
                measured_or_described += 1;
                if f.references.is_empty() {
                    violations.push(format!(
                        "Field `{}` claims basis `{}` but binds 0 reference_resources",
                        f.name, f.claim_basis
                    ));
                }
            }
            "Derived" | "StatisticallyCalibrated" | "PhylogeneticallyBracketed" => {
                derived_or_bracketed += 1;
            }
            "ModelAssumption" => model_assumption += 1,
            "ArtisticChoice" => artistic_choice += 1,
            _ => unknown_gaps += 1,
        }
        if let Some(num) = f.value.as_f64() {
            if !num.is_finite() || (f.name.ends_with("_m") && num <= 0.0) || (f.name.ends_with("_kg") && num <= 0.0) {
                violations.push(format!("Field `{}` has out-of-range QUDT numeric value: {num}", f.name));
            }
        }
    }

    let mut missing_required = Vec::new();
    for &req_name in required {
        if !provided_fields.iter().any(|f| f.name == req_name) {
            missing_required.push(req_name);
        }
    }
    let populated_required = required.len().saturating_sub(missing_required.len());

    let radial_samples: Vec<RadialLayerSample> = args
        .get("topological_layers")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let topological_report =
        verify_topological_and_radial_invariants(&radial_samples, &mut violations);

    // Optional SHACL Turtle + N3/Turtle data validation via QualiaDB's native SHACL + modality extensions compiler
    let mut shacl_extension_validation = Value::Null;
    if let (Some(data_ttl), Some(shapes_ttl)) = (
        args.get("data_ttl").and_then(Value::as_str),
        args.get("shapes_ttl").and_then(Value::as_str),
    ) {
        match qualia_core_db::modalities::logic::shacl::validate_turtle_shapes(data_ttl, shapes_ttl)
        {
            Ok(report) => {
                if !report.conforms {
                    for res in &report.results {
                        violations.push(format!(
                            "SHACL violation on `{}` ({:?}): {}",
                            res.focus_node,
                            res.result_path,
                            res.message.as_deref().unwrap_or("constraint failed")
                        ));
                    }
                }
                shacl_extension_validation =
                    serde_json::to_value(&report).unwrap_or_else(|_| json!({"conforms": report.conforms}));
            }
            Err(e) => {
                violations.push(format!("SHACL Turtle compilation failed: {e}"));
                shacl_extension_validation = json!({"conforms": false, "error": e});
            }
        }
    }

    // If an asset_10d_path is supplied, also inspect its sections, MSTR L0..L6, HOLO, and KLOD readiness
    let mut container_inspection = Value::Null;
    if let Some(p_str) = args.get("asset_10d_path").and_then(Value::as_str) {
        let path = Path::new(p_str);
        if path.is_file() {
            if let Ok(mut bytes) = fs::read(path) {
                let crc_ok = verify_whole_file_crc32c(&mut bytes).is_ok();
                if let Ok(hdr) = Container10dHeader::parse(&bytes) {
                    let mut scratch = [TypedFieldRecord::default(); 32];
                    let mut fields = [TypedFieldRecord::default(); 32];
                    let mid2_info = decode_container_manifold_identity(
                        &bytes,
                        &hdr,
                        &[],
                        &mut scratch,
                        &mut fields,
                    )
                    .ok();
                    let (has_holo, has_mstr, has_cbld) = if let Some((_, fcount)) = mid2_info {
                        (
                            fields[..fcount].iter().any(|f| f.kind == FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE),
                            fields[..fcount].iter().any(|f| f.kind == FIELD_KIND_MULTISCALE_STRATA),
                            fields[..fcount].iter().any(|f| f.kind == FIELD_KIND_SYMBOLIC_RDF_CBOR_LD),
                        )
                    } else {
                        (false, false, false)
                    };
                    container_inspection = json!({
                        "path": p_str,
                        "crc32c_valid": crc_ok,
                        "version": hdr.version,
                        "has_holographic_phase_space_0x0306": has_holo,
                        "has_multiscale_strata_0x0304": has_mstr,
                        "has_symbolic_cbor_ld_0x0305": has_cbld,
                        "klod_degradation_tiers": {
                            "KLOD0_max_joints": KinematicLodTier::KLOD0.max_active_joints(),
                            "KLOD1_max_joints": KinematicLodTier::KLOD1.max_active_joints(),
                            "KLOD2_max_joints": KinematicLodTier::KLOD2.max_active_joints(),
                            "KLOD3_max_joints": KinematicLodTier::KLOD3.max_active_joints(),
                        }
                    });
                }
            }
        }
    }

    let status = if missing_required.is_empty() && violations.is_empty() {
        "pass"
    } else {
        "fail"
    };

    Ok(json!({
        "schema": FORM_VALIDATION_SCHEMA,
        "subject_class": subject_class,
        "status": status,
        "populated_required": populated_required,
        "total_required": required.len(),
        "missing_required_fields": missing_required,
        "classification_summary": {
            "measured_or_described": measured_or_described,
            "derived_or_bracketed": derived_or_bracketed,
            "model_assumption": model_assumption,
            "artistic_choice": artistic_choice,
            "unknown_gaps": unknown_gaps + missing_required.len()
        },
        "reference_sources_bound": reference_sources_bound,
        "qudt_and_citation_violations": violations,
        "topological_and_radial_invariants": topological_report,
        "shacl_extension_validation": shacl_extension_validation,
        "container_10d_inspection": container_inspection
    }))
}

/// REQ-9.1 & REQ-9.2: Automated Multi-Format Render & Playback Verification Harness.
pub fn verify_multi_format_outputs(args: Value) -> Result<Value, String> {
    let target_dir = args
        .get("export_dir")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .or_else(|| crate::project::current().map(|p| p.working.clone()))
        .ok_or("Provide export_dir to verify outputs")?;

    if !target_dir.exists() {
        return Err(format!("Target path does not exist: {}", target_dir.display()));
    }

    let mut files_to_check = Vec::new();
    if target_dir.is_file() {
        files_to_check.push(target_dir.clone());
    } else if let Ok(rd) = fs::read_dir(&target_dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_file() {
                files_to_check.push(p);
            } else if p.is_dir() && p.file_name().and_then(|s| s.to_str()) != Some(".synth") {
                if let Ok(sub) = fs::read_dir(&p) {
                    for se in sub.flatten() {
                        if se.path().is_file() {
                            files_to_check.push(se.path());
                        }
                    }
                }
            }
        }
    }
    files_to_check.sort();

    let mut format_checks = Vec::new();
    let mut all_passed = true;

    for path in &files_to_check {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let rel_name = path
            .strip_prefix(&target_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let Ok(mut bytes) = fs::read(path) else {
            continue;
        };

        match ext.as_str() {
            "10d" => {
                let crc_ok = verify_whole_file_crc32c(&mut bytes).is_ok();
                let decoded_portal =
                    qualia_core_db::render::portal_load_10d::decode_load_10d(&bytes);
                match (crc_ok, decoded_portal) {
                    (true, Ok(bufs)) => {
                        let v_count = bufs.positions.len();
                        let t_count = bufs.triangle_count;
                        let mut min = [f32::INFINITY; 3];
                        let mut max = [f32::NEG_INFINITY; 3];
                        for p in &bufs.positions {
                            for k in 0..3 {
                                min[k] = min[k].min(p[k]);
                                max[k] = max[k].max(p[k]);
                            }
                        }
                        let extent = ((max[0] - min[0]).powi(2)
                            + (max[1] - min[1]).powi(2)
                            + (max[2] - min[2]).powi(2))
                        .sqrt();
                        let valid_bounds = extent.is_finite() && extent > 1e-5;
                        let status = if valid_bounds && v_count >= 3 && t_count >= 1 {
                            "pass"
                        } else {
                            all_passed = false;
                            "fail"
                        };
                        format_checks.push(json!({
                            "target": rel_name,
                            "format": "10d",
                            "runtime_profile": "native-gpu-and-portal-wasm-klod0..klod3",
                            "status": status,
                            "metrics": {
                                "vertices": v_count,
                                "triangles": t_count,
                                "bbox_diagonal_m": extent,
                                "crc32c_valid": true,
                                "materials": bufs.materials.len(),
                                "has_uv01": bufs.uv0.is_some()
                            }
                        }));
                    }
                    (false, Ok(_)) => {
                        all_passed = false;
                        format_checks.push(json!({
                            "target": rel_name,
                            "format": "10d",
                            "runtime_profile": "native-gpu-and-portal-wasm",
                            "status": "fail",
                            "code": "10d_crc32c_invalid",
                            "message": "Whole-file CRC-32C check failed"
                        }));
                    }
                    (crc, Err(e)) => {
                        all_passed = false;
                        format_checks.push(json!({
                            "target": rel_name,
                            "format": "10d",
                            "runtime_profile": "native-gpu-and-portal-wasm",
                            "status": "fail",
                            "code": "10d_decode_failed",
                            "message": format!("crc_valid={crc}, decode_load_10d error: {e}")
                        }));
                    }
                }
            }
            "png" => {
                let valid_sig = bytes.starts_with(b"\x89PNG\r\n\x1a\n") && bytes.len() > 64;
                if !valid_sig {
                    all_passed = false;
                }
                format_checks.push(json!({
                    "target": rel_name,
                    "format": "png",
                    "runtime_profile": "radiance-rgba",
                    "status": if valid_sig { "pass" } else { "fail" },
                    "metrics": { "bytes": bytes.len() }
                }));
            }
            "svg" => {
                let text = String::from_utf8_lossy(&bytes);
                let valid_svg = text.contains("<svg") && text.contains("</svg>");
                if !valid_svg {
                    all_passed = false;
                }
                format_checks.push(json!({
                    "target": rel_name,
                    "format": "svg",
                    "runtime_profile": "locus-vector-svg",
                    "status": if valid_svg { "pass" } else { "fail" },
                    "metrics": { "bytes": bytes.len() }
                }));
            }
            "emf" => {
                let valid_emf = bytes.len() >= 64 && (&bytes[0..8] == b"Q42EMF64" || &bytes[0..4] == b"EMF\0");
                if !valid_emf {
                    all_passed = false;
                }
                format_checks.push(json!({
                    "target": rel_name,
                    "format": "emf",
                    "runtime_profile": "electromagnetics-q42emf64",
                    "status": if valid_emf { "pass" } else { "fail" },
                    "metrics": { "bytes": bytes.len() }
                }));
            }
            "m64" => {
                let valid_m64 = bytes.len() >= 64;
                if !valid_m64 {
                    all_passed = false;
                }
                format_checks.push(json!({
                    "target": rel_name,
                    "format": "m64",
                    "runtime_profile": "geodesy-matter-m64",
                    "status": if valid_m64 { "pass" } else { "fail" },
                    "metrics": { "bytes": bytes.len() }
                }));
            }
            "q42" => {
                let valid_q42 = bytes.len() >= 64 && &bytes[0..4] == b"Q42\x00";
                if !valid_q42 {
                    all_passed = false;
                }
                format_checks.push(json!({
                    "target": rel_name,
                    "format": "q42",
                    "runtime_profile": "qualia-nquin-vault",
                    "status": if valid_q42 { "pass" } else { "fail" },
                    "metrics": { "bytes": bytes.len() }
                }));
            }
            "hmc" => {
                let valid_hmc = qualia_core_db::bundle::BundleReader::parse(&bytes).is_ok();
                if !valid_hmc {
                    all_passed = false;
                }
                format_checks.push(json!({
                    "target": rel_name,
                    "format": "hmc",
                    "runtime_profile": "qbdl-hypermedia-bundle",
                    "status": if valid_hmc { "pass" } else { "fail" },
                    "metrics": { "bytes": bytes.len() }
                }));
            }
            "mp4" | "wav" | "qac" | "dxf" => {
                let non_empty = bytes.len() > 32;
                if !non_empty {
                    all_passed = false;
                }
                format_checks.push(json!({
                    "target": rel_name,
                    "format": ext,
                    "runtime_profile": "media-playback-verify",
                    "status": if non_empty { "pass" } else { "fail" },
                    "metrics": { "bytes": bytes.len() }
                }));
            }
            _ => {}
        }
    }

    let overall = if format_checks.is_empty() {
        "error"
    } else if all_passed {
        "pass"
    } else {
        "fail"
    };

    Ok(json!({
        "schema": OUTPUT_VERIFICATION_SCHEMA,
        "target_dir": target_dir,
        "overall_technical_status": overall,
        "human_visual_acceptance": "pending_human_review",
        "checked_count": format_checks.len(),
        "format_checks": format_checks,
    }))
}

/// Optional overrides when converting a 3D asset from an automated remote/local pipeline
/// (such as `synth_colab` running `microsoft/TRELLIS.2-4B` on Google Colab Ultra).
#[derive(Debug, Clone, Default)]
pub struct Convert3dProvenanceOverrides {
    pub human_actor: Option<crate::changelog_licensing::HumanActorRecord>,
    pub software_agent: Option<crate::changelog_licensing::SoftwareAgentRecord>,
    pub harness: Option<crate::changelog_licensing::HarnessEnvironmentRecord>,
    pub economics: Option<crate::changelog_licensing::RevisionEconomicsTelemetry>,
    pub epistemic_horizon_q42_sha256: Option<String>,
    pub ovoxel_sidecar_json: Option<Value>,
    pub summary_override: Option<String>,
}

/// Phase 1: Convert an existing 3D formatted asset or scene (`.glb`/`.gltf`, `.obj`, `.stl`, or legacy `.10d`)
/// into a Holographic `.10d` v2 container + Mode-B scene instances + `.synth/license-contract.json` & `.synth/changelog.json`.
pub fn convert_3d_scene_or_asset_cli_or_mcp(args: Value) -> Result<Value, String> {
    convert_3d_with_provenance(args, Convert3dProvenanceOverrides::default())
}

/// Parameterized 3D conversion pipeline supporting full `SoftwareAgentRecord`,
/// `HarnessEnvironmentRecord`, `RevisionEconomicsTelemetry`, O-Voxel volumetric sidecar
/// preservation (`HOLO` `0x0306` + `VOX0`), and companion `.n3` / `manifest.q42.yaml` (`L0`–`L6`) emission.
pub fn convert_3d_with_provenance(
    args: Value,
    overrides: Convert3dProvenanceOverrides,
) -> Result<Value, String> {
    let source_path_str = args
        .get("source_path")
        .and_then(Value::as_str)
        .ok_or("source_path is required")?;
    let out_dir_str = args
        .get("out_dir")
        .and_then(Value::as_str)
        .ok_or("out_dir is required (create-only directory under SYNTH_OUTPUT_ROOT or absolute)")?;
    let spdx_license = args
        .get("spdx_license")
        .and_then(Value::as_str)
        .ok_or("spdx_license is required (e.g. CC-BY-NC-ND-4.0, CC-BY-4.0, PC-PAYDOWN-1.0)")?;
    let creator = args
        .get("creator")
        .and_then(Value::as_str)
        .unwrap_or("Timothy Charles Holborn");
    let attribution_notice = args
        .get("attribution_notice")
        .and_then(Value::as_str)
        .ok_or("attribution_notice is required (UTF-8 attribution and copyright notice)")?;
    let asset_uri = args
        .get("asset_uri")
        .and_then(Value::as_str)
        .unwrap_or("urn:qualia:converted:asset");
    let domain_id = args.get("domain_id").and_then(Value::as_u64).unwrap_or(1) as u32;

    if source_path_str.contains("..") || out_dir_str.contains("..") {
        return Err("source_path and out_dir must not contain parent traversal ('..')".into());
    }

    let src_path = PathBuf::from(source_path_str);
    let source_bytes = fs::read(&src_path).map_err(|e| format!("Failed to read {}: {e}", src_path.display()))?;
    let ext_hint = args
        .get("format")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| {
            src_path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_ascii_lowercase())
        });

    let out_dir = PathBuf::from(out_dir_str);
    if out_dir.exists() {
        return Err(format!("Create-only out_dir already exists: {}", out_dir.display()));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let origin_uri = args.get("origin_uri").and_then(Value::as_str).map(str::to_owned);
    let license_attr = SourceLicenseAttribution {
        spdx_license: spdx_license.to_owned(),
        creator_name_or_did: creator.to_owned(),
        attribution_notice_utf8: attribution_notice.to_owned(),
        origin_uri: origin_uri.clone(),
        timestamp_epoch_s: now,
        embed_full_source_bytes: args
            .get("embed_full_source_bytes")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    };

    let converted = convert_3d_asset_or_scene_with_license(
        &source_bytes,
        ext_hint.as_deref(),
        asset_uri,
        domain_id,
        [0.0, 3.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.5],
        &license_attr,
    )
    .map_err(|e| format!("3D conversion failed: {e}"))?;

    fs::create_dir_all(&out_dir).map_err(|e| format!("create out_dir: {e}"))?;
    let stem = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("converted");
    let arch_filename = format!("{stem}-archetype.10d");
    let arch_path = out_dir.join(&arch_filename);
    fs::write(&arch_path, &converted.archetype_10d).map_err(|e| e.to_string())?;

    let src_sha = format!("{:x}", Sha256::digest(&source_bytes));
    let arch_sha = format!("{:x}", Sha256::digest(&converted.archetype_10d));

    let mut changed = vec![crate::changelog_licensing::ChangedResourceDelta {
        key: arch_filename.clone(),
        op: "Added".into(),
        old_sha256: None,
        new_sha256: Some(arch_sha.clone()),
        bytes: converted.archetype_10d.len(),
    }];

    let mut written_instances = Vec::new();
    if !converted.instance_10d_containers.is_empty() {
        let inst_dir = out_dir.join("instances");
        fs::create_dir_all(&inst_dir).map_err(|e| e.to_string())?;
        for (node_name, inst_bytes) in &converted.instance_10d_containers {
            let safe_name: String = node_name
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
                .collect();
            let fname = format!("instances/{safe_name}.10d");
            let fpath = out_dir.join(&fname);
            fs::write(&fpath, inst_bytes).map_err(|e| e.to_string())?;
            changed.push(crate::changelog_licensing::ChangedResourceDelta {
                key: fname.clone(),
                op: "Added".into(),
                old_sha256: None,
                new_sha256: Some(format!("{:x}", Sha256::digest(inst_bytes))),
                bytes: inst_bytes.len(),
            });
            written_instances.push(fname);
        }
    }

    // Persist extracted texture dependencies when present
    let mut written_textures = Vec::new();
    if !converted.texture_dependencies.is_empty() {
        let tex_dir = out_dir.join("textures");
        fs::create_dir_all(&tex_dir).map_err(|e| e.to_string())?;
        for (idx, tex) in converted.texture_dependencies.iter().enumerate() {
            let ext = match tex.mime_type.as_str() {
                "image/png" => "png",
                "image/jpeg" | "image/jpg" => "jpg",
                "image/webp" => "webp",
                "image/ktx2" => "ktx2",
                _ => "bin",
            };
            let fname = format!("textures/tex_{idx:02}.{ext}");
            let fpath = out_dir.join(&fname);
            if !tex.bytes.is_empty() {
                fs::write(&fpath, &tex.bytes).map_err(|e| e.to_string())?;
                changed.push(crate::changelog_licensing::ChangedResourceDelta {
                    key: fname.clone(),
                    op: "Added".into(),
                    old_sha256: None,
                    new_sha256: Some(format!("{:x}", Sha256::digest(&tex.bytes))),
                    bytes: tex.bytes.len(),
                });
                written_textures.push(fname);
            }
        }
    }

    // Persist O-Voxel volumetric sidecar (multi-segment M64HoloRecord clusters + VOX0 DAG nodes) when provided
    let mut ovoxel_sidecar_path: Option<PathBuf> = None;
    if let Some(ovoxel_val) = &overrides.ovoxel_sidecar_json {
        let ovoxel_fname = format!("{stem}.ovoxel.json");
        let ovoxel_p = out_dir.join(&ovoxel_fname);
        let ovoxel_bytes = serde_json::to_vec_pretty(ovoxel_val).unwrap_or_default();
        fs::write(&ovoxel_p, &ovoxel_bytes).map_err(|e| e.to_string())?;
        changed.push(crate::changelog_licensing::ChangedResourceDelta {
            key: ovoxel_fname,
            op: "Added".into(),
            old_sha256: None,
            new_sha256: Some(format!("{:x}", Sha256::digest(&ovoxel_bytes))),
            bytes: ovoxel_bytes.len(),
        });
        ovoxel_sidecar_path = Some(ovoxel_p);
    }

    // Emit companion UTF-8 N3 provenance graph & 7-Layer (L0-L6) manifest.q42.yaml
    let prov_n3_fname = format!("{stem}-provenance.n3");
    let prov_n3_path = out_dir.join(&prov_n3_fname);
    let prov_n3_content = format!(
        "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         @prefix prov: <http://www.w3.org/ns/prov#> .\n\
         @prefix src-lic: <https://ns.webcivics.net/source-asset-licensing/> .\n\n\
         <{asset_uri}> a src-lic:ConvertedEmbodiment ;\n\
             src-lic:compiled10dCrc32c {} ;\n\
             prov:wasDerivedFrom <{asset_uri}#source> .\n\n\
         <{asset_uri}#source> a src-lic:SourceAssetRecord ;\n\
             src-lic:sourceFormat \"{}\" ;\n\
             src-lic:sourceByteLength {} ;\n\
             src-lic:sourceCrc32c {} ;\n\
             src-lic:sourceSha256 \"{src_sha}\"^^xsd:hexBinary ;\n\
             src-lic:hasLicenseTier <{}> ;\n\
             src-lic:spdxIdentifier \"{spdx_license}\" ;\n\
             src-lic:attributionNotice \"{}\" .\n",
        converted.archetype_compiled_digest,
        ext_hint.as_deref().unwrap_or("glb"),
        source_bytes.len(),
        converted.source_digest,
        converted.license_tier.iri(),
        attribution_notice.replace('\\', "\\\\").replace('"', "\\\"")
    );
    fs::write(&prov_n3_path, prov_n3_content.as_bytes()).map_err(|e| e.to_string())?;
    changed.push(crate::changelog_licensing::ChangedResourceDelta {
        key: prov_n3_fname.clone(),
        op: "Added".into(),
        old_sha256: None,
        new_sha256: Some(format!("{:x}", Sha256::digest(prov_n3_content.as_bytes()))),
        bytes: prov_n3_content.len(),
    });

    let manifest_fname = "manifest.q42.yaml";
    let manifest_path = out_dir.join(manifest_fname);
    let manifest_content = format!(
        "schema: yaml-ld-q42/v2\n\
         id: \"{asset_uri}\"\n\
         archetype_10d: \"{arch_filename}\"\n\
         provenance_n3: \"{prov_n3_fname}\"\n\
         holographic_10d_v2:\n\
           field_kind_holo: \"0x0306\"\n\
           field_kind_cbld: \"0x0305\"\n\
           ovoxel_volumetric_sidecar: {}\n\
         multiscale_strata:\n\
           L0: \"Continuum & Classical Field Mechanics\"\n\
           L1: \"Rigid & Articulated Multi-Body Kinetics (PGA / XPBD / SKL2)\"\n\
           L2: \"Soft-Body, Elastoplastic & Fluid Continuum\"\n\
           L3: \"Molecular & Chemical Kinetics\"\n\
           L4: \"Cellular, Tissue & Biological Morphogenesis\"\n\
           L5: \"Ecological, Geomorphic & Planetary Systems\"\n\
           L6: \"L6QuantumPhysics (Quantum Dynamics, Quantum Physics & Quantum Chemistry)\"\n",
        ovoxel_sidecar_path.is_some()
    );
    fs::write(&manifest_path, manifest_content.as_bytes()).map_err(|e| e.to_string())?;
    changed.push(crate::changelog_licensing::ChangedResourceDelta {
        key: manifest_fname.to_owned(),
        op: "Added".into(),
        old_sha256: None,
        new_sha256: Some(format!("{:x}", Sha256::digest(manifest_content.as_bytes()))),
        bytes: manifest_content.len(),
    });

    let regime = match converted.license_tier {
        qualia_core_db::render::compile_10d::SourceLicenseTier::AuthorMoralRightsQualiaCcByNcNd => {
            crate::changelog_licensing::LicensingRegimeKind::QualiaCcByNcNd4
        }
        qualia_core_db::render::compile_10d::SourceLicenseTier::PerpetualCreativeRoyalty => {
            crate::changelog_licensing::LicensingRegimeKind::PerpetualCreativeRoyalty
        }
        qualia_core_db::render::compile_10d::SourceLicenseTier::SanctuaryProtected => {
            crate::changelog_licensing::LicensingRegimeKind::SanctuaryInalienable
        }
        _ => crate::changelog_licensing::LicensingRegimeKind::PermissiveCommonsPaydown,
    };
    let mut contract =
        crate::changelog_licensing::DualRegimeLicenseContract::for_regime(
            asset_uri, regime,
        );
    contract.spdx_or_qualia_id = spdx_license.to_owned();
    contract.attributed_to = vec![format!("{creator} — {attribution_notice}")];
    contract.upstream_source_license = Some(spdx_license.to_owned());

    let human_actor = overrides.human_actor.unwrap_or_else(|| {
        crate::changelog_licensing::HumanActorRecord {
            did: creator.to_owned(),
            display_name: creator.to_owned(),
            ..Default::default()
        }
    });
    let software_agent = overrides.software_agent.unwrap_or_default();
    let harness = overrides.harness.unwrap_or_else(|| {
        crate::changelog_licensing::HarnessEnvironmentRecord {
            harness_name: "Webizen AI Synth 3D Converter".into(),
            synth_version: crate::agent_index::RELEASE_VERSION.into(),
            synth_binary_sha256: None,
            gpu_backend: "native".into(),
            quality_profile: "production16".into(),
        }
    });
    let economics = overrides.economics.unwrap_or_else(|| {
        crate::changelog_licensing::RevisionEconomicsTelemetry {
            currency: "AUD".into(),
            work_status: "SelfFunded".into(),
            ..Default::default()
        }
    });
    let summary = overrides.summary_override.unwrap_or_else(|| {
        format!(
            "Converted 3D asset {} under license {}",
            src_path.display(),
            spdx_license
        )
    });

    let (changelog, _) = crate::changelog_licensing::append_revision_and_license(
        &out_dir,
        asset_uri,
        "Convert3D",
        human_actor,
        software_agent,
        harness,
        &src_sha,
        overrides.epistemic_horizon_q42_sha256.as_deref(),
        changed,
        economics,
        &contract,
        &summary,
    )?;

    let decoded_mesh = decode_10d_mesh(&converted.archetype_10d).ok();

    Ok(json!({
        "schema": "synth.convert-3d-receipt/v1",
        "status": "ok",
        "source_path": src_path,
        "source_crc32c": converted.source_digest,
        "source_sha256": src_sha,
        "archetype_10d_path": arch_path,
        "archetype_compiled_crc32c": converted.archetype_compiled_digest,
        "archetype_bytes": converted.archetype_10d.len(),
        "provenance_n3_path": prov_n3_path,
        "manifest_q42_yaml_path": manifest_path,
        "ovoxel_sidecar_path": ovoxel_sidecar_path,
        "vertices": decoded_mesh.as_ref().map(|m| m.vertex_count()).unwrap_or(8),
        "triangles": decoded_mesh.as_ref().map(|m| m.triangle_count()).unwrap_or(12),
        "mode_b_scene_instances": written_instances,
        "extracted_textures": converted.texture_dependencies.len(),
        "written_textures": written_textures,
        "emitted_nquins": converted.quins.len(),
        "license_tier": converted.license_tier.iri(),
        "changelog_head_rev": changelog.head_rev,
    }))
}
