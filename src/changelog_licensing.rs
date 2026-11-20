//! Asset Changelog, Multi-Actor SVN-Style Revisions, Provenance Signatures, Economics & Dual-Regime
//! Licensing Contracts (`REQ-6.1`..`6.4`).
//!
//! Reuses canonical concepts from `qualia-cooperative-core` (`AgentType::NaturalPerson`,
//! `AgentType::SoftwareAgent`, `WorkStatus`, `merge_contributions`, 9-stage contribution lifecycle)
//! and `qualia-core-db` (`permissive-commons.n3`, `social-permissive-commons.n3`,
//! `source-asset-licensing.n3`, finite obligation pool `S + W + H <= T`).
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const CHANGELOG_SCHEMA: &str = "synth.asset.changelog/v1";
pub const CHANGELOG_SIGN_CONTEXT: &str = "synth.asset.changelog.rev.v1";
pub const LICENSE_CONTRACT_SCHEMA: &str = "synth.asset.license-contract/v1";

/// Human actor (`AgentType::NaturalPerson`) retaining moral and legal authorship/stewardship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanActorRecord {
    pub did: String,
    pub display_name: String,
    /// `Steward | Contributor | Reviewer`
    pub role: String,
    #[serde(default)]
    pub organization: Option<String>,
    #[serde(default)]
    pub jurisdiction: Option<String>,
    /// `Unknown | Unpaid | SelfFunded | PartiallyCompensated | Compensated | Disputed`
    pub work_status: String,
}

impl Default for HumanActorRecord {
    fn default() -> Self {
        Self {
            did: "did:q42:person:timothy-charles-holborn".into(),
            display_name: "Timothy Charles Holborn".into(),
            role: "Steward".into(),
            organization: Some("Webizen / Qualia".into()),
            jurisdiction: Some("AU".into()),
            work_status: "SelfFunded".into(),
        }
    }
}

/// Software Agent disclosed in `JudgementProvenance::informed_by` (`AgentType::SoftwareAgent`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareAgentRecord {
    pub agent_id: String,
    pub model_name: String,
    #[serde(default)]
    pub model_version: Option<String>,
    /// `authoring | research | review | conversion`
    pub role: String,
    pub within_validated_scope: bool,
}

impl Default for SoftwareAgentRecord {
    fn default() -> Self {
        Self {
            agent_id: "agent-synth-mcp".into(),
            model_name: "Antigravity Agent".into(),
            model_version: Some("2026-11".into()),
            role: "authoring".into(),
            within_validated_scope: true,
        }
    }
}

/// Harness & Execution Environment record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessEnvironmentRecord {
    pub harness_name: String,
    pub synth_version: String,
    #[serde(default)]
    pub synth_binary_sha256: Option<String>,
    pub gpu_backend: String,
    pub quality_profile: String,
}

/// Unit-safe Economics, Compute & Token Telemetry per revision (`REQ-6.2`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RevisionEconomicsTelemetry {
    pub human_effort_minutes: u32,
    pub capital_advanced_cents: i64,
    pub reimbursable_expense_cents: i64,
    pub currency: String,
    pub work_status: String,
    #[serde(default)]
    pub input_tokens: Option<u64>,
    #[serde(default)]
    pub output_tokens: Option<u64>,
    #[serde(default)]
    pub reasoning_tokens: Option<u64>,
    #[serde(default)]
    pub local_inference: bool,
    pub wall_time_ms: u64,
    pub asset_work_units: u64,
    pub peak_ram_bytes: u64,
    #[serde(default)]
    pub estimated_energy_joules: Option<f64>,
}

/// Per-resource delta entry in an SVN-style revision (`REQ-6.3`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedResourceDelta {
    pub key: String,
    /// `Added | Modified | Deleted`
    pub op: String,
    #[serde(default)]
    pub old_sha256: Option<String>,
    #[serde(default)]
    pub new_sha256: Option<String>,
    pub bytes: usize,
}

/// Single SVN-style revision entry (`rev: r1, r2, ...`) in `synth/changelog.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogRevisionEntry {
    pub rev_number: u64,
    pub revision_id: String,
    pub predecessor_ids: Vec<String>,
    /// `Checkout | Commit | Branch | Merge | Review | LicenceUpdate | Convert3D`
    pub action: String,
    pub created_at_epoch_s: u64,
    pub user: HumanActorRecord,
    pub agent: SoftwareAgentRecord,
    pub harness: HarnessEnvironmentRecord,
    pub source_vibe_sha256: String,
    pub outputs_merkle_root: String,
    #[serde(default)]
    pub hmc_sha256: Option<String>,
    #[serde(default)]
    pub epistemic_horizon_q42_sha256: Option<String>,
    pub changed_resources: Vec<ChangedResourceDelta>,
    pub economics: RevisionEconomicsTelemetry,
    pub license_contract_digest: String,
    /// Domain-separated commitment digest (`synth.asset.changelog.rev.v1`).
    pub revision_commitment_sha256: String,
    /// Distinguishes `"host_build_attestation"` from `"human_review_adoption"`.
    pub attestation_kind: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetChangelog {
    pub schema: String,
    pub asset_id: String,
    pub head_rev: u64,
    pub total_effort_minutes: u64,
    pub revisions: Vec<ChangelogRevisionEntry>,
}

/// Dual-Regime Licensing & Obligation Contract (`REQ-6.4`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LicensingRegimeKind {
    /// Regime A: Finite cost-recovery obligation pool (`pc:Work` -> `pc:Discharged` true commons).
    PermissiveCommonsPaydown,
    /// Regime B: Non-extinguishing ongoing royalty splits (`PerpetualCreativeRoyalty`).
    PerpetualCreativeRoyalty,
    /// Sovereign CC-BY-NC-ND-4.0 (Copyright Timothy Charles Holborn).
    SovereignCcByNcNd4,
    /// Inalienable biosphere / sanctuary protection (`SPC`).
    SanctuaryInalienable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualRegimeLicenseContract {
    pub schema: String,
    pub asset_id: String,
    pub spdx_or_qualia_id: String,
    pub regime: LicensingRegimeKind,
    /// Permanent moral attribution (`pc:attributedTo`, never extinguishes).
    pub attributed_to: Vec<String>,
    pub travels_with_artifact: bool,
    /// Contract lifecycle state: `"draft_proposed" | "human_adopted_release" | "valued_obligation" | "discharged_commons"`
    pub contract_stage: String,
    /// Finite obligation pool target `T = productionCost * roiMultiplier` (in minor currency units / cents).
    pub target_pool_t_cents: u64,
    /// SHACL-capped ROI multiplier in `[1.0, 10.0]` (`pc:RoiMultiplierCapShape`).
    pub roi_multiplier: f64,
    /// Verified compensation paid to contributors (`S`).
    pub settled_s_cents: u64,
    /// Authorized non-cash discharge (`W`).
    pub discharged_w_cents: u64,
    /// Active holds (`H`). Invariant: `S + W + H <= T`.
    pub holds_h_cents: u64,
    pub currency: String,
    /// Qualifying human-rights / civic uses permitted at no charge (`OP_PERMIT`, never recorded as payment/discharge).
    pub human_rights_no_charge_permitted: bool,
    /// Whether economic obligation extinguishes into true commons when `S + W >= T`.
    pub extinguishes_on_cost_recovery: bool,
    /// For Regime B (`PerpetualCreativeRoyalty`): derivative / streaming revenue share in basis points (100 bps = 1%).
    pub ongoing_royalty_bps: u16,
    /// Source asset provenance license tier when converted from an existing 3D file.
    #[serde(default)]
    pub upstream_source_license: Option<String>,
}

impl DualRegimeLicenseContract {
    pub fn default_sovereign_or_commons(asset_id: &str, regime: LicensingRegimeKind) -> Self {
        let (spdx, extinguishes, royalty_bps) = match regime {
            LicensingRegimeKind::PermissiveCommonsPaydown => ("PC-PAYDOWN-1.0", true, 0),
            LicensingRegimeKind::PerpetualCreativeRoyalty => ("PERPETUAL-ROYALTY-1.0", false, 1000),
            LicensingRegimeKind::SovereignCcByNcNd4 => ("CC-BY-NC-ND-4.0", false, 0),
            LicensingRegimeKind::SanctuaryInalienable => ("SPC-SANCTUARY-1.0", false, 0),
        };
        Self {
            schema: LICENSE_CONTRACT_SCHEMA.into(),
            asset_id: asset_id.into(),
            spdx_or_qualia_id: spdx.into(),
            regime,
            attributed_to: vec![
                "Timothy Charles Holborn <timothy.holborn@gmail.com> (https://www.linkedin.com/in/ubiquitous/)"
                    .into(),
            ],
            travels_with_artifact: true,
            contract_stage: "draft_proposed".into(),
            target_pool_t_cents: 0,
            roi_multiplier: 1.0,
            settled_s_cents: 0,
            discharged_w_cents: 0,
            holds_h_cents: 0,
            currency: "AUD".into(),
            human_rights_no_charge_permitted: true,
            extinguishes_on_cost_recovery: extinguishes,
            ongoing_royalty_bps: royalty_bps,
            upstream_source_license: None,
        }
    }

    /// Enforce `pc:RoiMultiplierCapShape` (`1.0 <= roi_multiplier <= 10.0`) and `S + W + H <= T`.
    pub fn validate(&self) -> Result<(), String> {
        if !self.roi_multiplier.is_finite()
            || self.roi_multiplier < 1.0
            || self.roi_multiplier > 10.0
        {
            return Err(format!(
                "SHACL violation (pc:RoiMultiplierCapShape): roi_multiplier {} outside [1.0, 10.0]",
                self.roi_multiplier
            ));
        }
        if !self.travels_with_artifact {
            return Err(
                "SHACL violation (spc:StickyProhibitionShape): travels_with_artifact must be true"
                    .into(),
            );
        }
        if self.attributed_to.is_empty() {
            return Err("SHACL violation: pc:attributedTo must name at least one creator".into());
        }
        let committed = self
            .settled_s_cents
            .saturating_add(self.discharged_w_cents)
            .saturating_add(self.holds_h_cents);
        if self.target_pool_t_cents > 0 && committed > self.target_pool_t_cents {
            return Err(format!(
                "Obligation pool invariant violated: S + W + H ({committed}) > T ({})",
                self.target_pool_t_cents
            ));
        }
        Ok(())
    }

    pub fn remaining_recovery_cents(&self) -> u64 {
        self.target_pool_t_cents.saturating_sub(
            self.settled_s_cents
                .saturating_add(self.discharged_w_cents)
                .saturating_add(self.holds_h_cents),
        )
    }

    pub fn is_commons_discharged(&self) -> bool {
        self.extinguishes_on_cost_recovery
            && self.target_pool_t_cents > 0
            && self
                .settled_s_cents
                .saturating_add(self.discharged_w_cents)
                >= self.target_pool_t_cents
    }

    pub fn digest_sha256(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        format!("{:x}", Sha256::digest(&bytes))
    }
}

pub fn compute_revision_commitment(entry: &ChangelogRevisionEntry) -> String {
    let mut hasher = Sha256::new();
    hasher.update(CHANGELOG_SIGN_CONTEXT.as_bytes());
    hasher.update(b"\0");
    hasher.update(entry.rev_number.to_le_bytes());
    hasher.update(entry.predecessor_ids.join(",").as_bytes());
    hasher.update(entry.user.did.as_bytes());
    hasher.update(entry.agent.agent_id.as_bytes());
    hasher.update(entry.harness.harness_name.as_bytes());
    hasher.update(entry.created_at_epoch_s.to_le_bytes());
    hasher.update(entry.outputs_merkle_root.as_bytes());
    hasher.update(entry.license_contract_digest.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Append or merge a revision into a project's `.synth/changelog.json` and `.synth/license-contract.json`.
pub fn append_revision_and_license(
    export_or_project_dir: &Path,
    asset_id: &str,
    action: &str,
    user: HumanActorRecord,
    agent: SoftwareAgentRecord,
    harness: HarnessEnvironmentRecord,
    source_vibe_sha256: &str,
    hmc_sha256: Option<&str>,
    changed_resources: Vec<ChangedResourceDelta>,
    economics: RevisionEconomicsTelemetry,
    contract: &DualRegimeLicenseContract,
    summary: &str,
) -> Result<(AssetChangelog, DualRegimeLicenseContract), String> {
    contract.validate()?;
    let synth_dir = export_or_project_dir.join(".synth");
    fs::create_dir_all(&synth_dir).map_err(|e| format!(".synth dir: {e}"))?;

    let changelog_path = synth_dir.join("changelog.json");
    let license_path = synth_dir.join("license-contract.json");

    let mut changelog: AssetChangelog = if changelog_path.is_file() {
        let raw = fs::read_to_string(&changelog_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&raw).map_err(|e| format!("changelog.json: {e}"))?
    } else {
        AssetChangelog {
            schema: CHANGELOG_SCHEMA.into(),
            asset_id: asset_id.into(),
            head_rev: 0,
            total_effort_minutes: 0,
            revisions: Vec::new(),
        }
    };

    let next_rev = changelog.head_rev + 1;
    let predecessor_ids = changelog
        .revisions
        .last()
        .map(|r| vec![r.revision_id.clone()])
        .unwrap_or_default();

    let mut merkle = Sha256::new();
    for res in &changed_resources {
        merkle.update(res.key.as_bytes());
        if let Some(ref h) = res.new_sha256 {
            merkle.update(h.as_bytes());
        }
    }
    let outputs_merkle_root = format!("{:x}", merkle.finalize());
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let license_digest = contract.digest_sha256();

    let mut entry = ChangelogRevisionEntry {
        rev_number: next_rev,
        revision_id: format!("r{next_rev}"),
        predecessor_ids,
        action: action.into(),
        created_at_epoch_s: now,
        user,
        agent,
        harness,
        source_vibe_sha256: source_vibe_sha256.into(),
        outputs_merkle_root,
        hmc_sha256: hmc_sha256.map(str::to_owned),
        epistemic_horizon_q42_sha256: None,
        changed_resources,
        economics,
        license_contract_digest: license_digest,
        revision_commitment_sha256: String::new(),
        attestation_kind: "host_build_attestation".into(),
        summary: summary.into(),
    };
    entry.revision_commitment_sha256 = compute_revision_commitment(&entry);

    changelog.head_rev = next_rev;
    changelog.total_effort_minutes += entry.economics.human_effort_minutes as u64;
    changelog.revisions.push(entry);

    fs::write(
        &changelog_path,
        serde_json::to_vec_pretty(&changelog).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        &license_path,
        serde_json::to_vec_pretty(contract).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    Ok((changelog, contract.clone()))
}

/// Inspect or update an asset's changelog and dual-regime license contract via MCP (`synth_changelog_license`).
pub fn dispatch_changelog_license_mcp(args: Value) -> Result<Value, String> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("inspect");
    let target_dir = args
        .get("project_dir")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .or_else(|| crate::project::current().map(|p| p.root.clone()))
        .or_else(|| {
            std::env::var("SYNTH_OUTPUT_ROOT")
                .ok()
                .map(|s| PathBuf::from(s.trim()))
        })
        .ok_or("Specify project_dir or configure SYNTH_OUTPUT_ROOT")?;

    let synth_dir = target_dir.join(".synth");
    let changelog_path = synth_dir.join("changelog.json");
    let license_path = synth_dir.join("license-contract.json");

    match action {
        "inspect" => {
            let changelog: Value = if changelog_path.is_file() {
                serde_json::from_str(&fs::read_to_string(&changelog_path).map_err(|e| e.to_string())?)
                    .unwrap_or(Value::Null)
            } else {
                Value::Null
            };
            let contract: Option<DualRegimeLicenseContract> = if license_path.is_file() {
                serde_json::from_str(&fs::read_to_string(&license_path).map_err(|e| e.to_string())?)
                    .ok()
            } else {
                None
            };
            let pool_status = contract.as_ref().map(|c| {
                json!({
                    "remaining_recovery_cents": c.remaining_recovery_cents(),
                    "is_commons_discharged": c.is_commons_discharged(),
                    "shacl_valid": c.validate().is_ok(),
                })
            });
            Ok(json!({
                "schema": "synth.changelog-license-report/v1",
                "project_dir": target_dir,
                "changelog": changelog,
                "license_contract": contract,
                "obligation_pool_status": pool_status,
            }))
        }
        "set_license" => {
            let asset_id = args
                .get("asset_id")
                .and_then(Value::as_str)
                .unwrap_or("urn:qualia:asset:work");
            let regime = match args.get("regime").and_then(Value::as_str).unwrap_or("commons_paydown") {
                "perpetual_royalty" => LicensingRegimeKind::PerpetualCreativeRoyalty,
                "sovereign_cc_by_nc_nd" => LicensingRegimeKind::SovereignCcByNcNd4,
                "sanctuary" => LicensingRegimeKind::SanctuaryInalienable,
                _ => LicensingRegimeKind::PermissiveCommonsPaydown,
            };
            let mut contract = DualRegimeLicenseContract::default_sovereign_or_commons(asset_id, regime);
            if let Some(t) = args.get("target_pool_t_cents").and_then(Value::as_u64) {
                contract.target_pool_t_cents = t;
            }
            if let Some(roi) = args.get("roi_multiplier").and_then(Value::as_f64) {
                contract.roi_multiplier = roi;
            }
            if let Some(s) = args.get("settled_s_cents").and_then(Value::as_u64) {
                contract.settled_s_cents = s;
            }
            if let Some(up) = args.get("upstream_source_license").and_then(Value::as_str) {
                contract.upstream_source_license = Some(up.to_owned());
            }
            contract.validate()?;
            fs::create_dir_all(&synth_dir).map_err(|e| e.to_string())?;
            fs::write(
                &license_path,
                serde_json::to_vec_pretty(&contract).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            Ok(json!({
                "status": "ok",
                "license_contract": contract,
                "remaining_recovery_cents": contract.remaining_recovery_cents(),
                "is_commons_discharged": contract.is_commons_discharged(),
            }))
        }
        other => Err(format!("Unknown action `{other}`; expected `inspect` or `set_license`")),
    }
}
