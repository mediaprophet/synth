//! Asset-neutral production. Producers own domain validation and generation;
//! this module owns bounded source history, resources and replayable HMCs.
//!
//! Producers are trusted Rust implementations, not a sandbox. They must be
//! deterministic and side-effect free, respect the supplied limits/cancellation,
//! and validate their own formats. Resource `kind` is a label, not certification.
pub mod assets;
mod atlas;
mod files;
mod package;
mod source;

pub use files::FilesProducer;
pub use source::ProductionSession;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
pub use vibe::Value as AssetValue;

pub use asset_production_vibe::Resource;

#[derive(Clone, Debug)]
pub struct ProductionLimits {
    pub source: vibe::session::SessionLimits,
    pub evaluation: vibe::Budget,
    pub resources: usize,
    pub resource_bytes: usize,
    pub total_resource_bytes: usize,
    pub package_bytes: usize,
    pub asset_work_units: u64,
}
impl Default for ProductionLimits {
    fn default() -> Self {
        Self {
            source: vibe::session::SessionLimits::default(),
            evaluation: vibe::Budget::default(),
            resources: 512,
            resource_bytes: 16 * 1024 * 1024,
            total_resource_bytes: 64 * 1024 * 1024,
            package_bytes: 80 * 1024 * 1024,
            asset_work_units: 50_000_000,
        }
    }
}

/// HMC bag read limits derived from production budgets.
pub fn bundle_read_limits(
    limits: &ProductionLimits,
) -> Result<qualia_core_db::bundle::BundleReadLimits, String> {
    package::read_limits(limits)
}

pub struct ProductionContext<'a> {
    pub inputs: &'a [Resource],
    pub limits: &'a ProductionLimits,
    pub cancel: &'a AtomicBool,
    pub(crate) assets: &'a assets::AssetWorkspace,
}
impl ProductionContext<'_> {
    /// Resolve an encoded resource returned by a production capability.
    pub fn generated_resource(&self, handle: &AssetValue) -> Result<&Resource, String> {
        self.assets.encoded(handle)
    }
    pub fn check_cancelled(&self) -> Result<(), String> {
        if self.cancel.load(Ordering::Acquire) {
            Err("Production cancelled".into())
        } else {
            Ok(())
        }
    }
}

/// Register versioned domain producers without changing Synth's source/session/container code.
/// `produce` validates the evaluated `asset` binding and returns native output files.
/// It may be called again during strict reopen; external effects are forbidden.
pub trait AssetProducer: Send + Sync {
    fn id(&self) -> &str;
    /// Change this whenever output semantics change; persisted packages require an exact match.
    fn version(&self) -> &str;
    fn produce(
        &self,
        asset: &AssetValue,
        context: &ProductionContext<'_>,
    ) -> Result<Vec<Resource>, String>;
}

#[derive(Default)]
pub struct ProducerRegistry {
    producers: BTreeMap<String, Arc<dyn AssetProducer>>,
}
impl ProducerRegistry {
    pub fn with_builtins() -> Self {
        let mut registry = Self::default();
        registry
            .register(FilesProducer)
            .expect("built-in producer identity");
        registry
    }
    pub fn register(&mut self, producer: impl AssetProducer + 'static) -> Result<(), String> {
        valid_label(producer.id())?;
        valid_label(producer.version())?;
        if self.producers.contains_key(producer.id()) {
            return Err("Producer already registered".into());
        }
        self.producers
            .insert(producer.id().to_owned(), Arc::new(producer));
        Ok(())
    }
    pub fn profiles(&self) -> impl Iterator<Item = (&str, &str)> {
        self.producers.values().map(|p| (p.id(), p.version()))
    }
    pub fn session(
        &self,
        profile: &str,
        inputs: Vec<Resource>,
        limits: ProductionLimits,
    ) -> Result<ProductionSession, String> {
        let producer = self
            .producers
            .get(profile)
            .ok_or_else(|| format!("Unknown producer: {profile}"))?;
        validate_resources(&inputs, &limits)?;
        Ok(ProductionSession::new(producer.clone(), inputs, limits))
    }
    /// Verify container digests, exact producer version and source replay before returning a session.
    pub fn reopen(
        &self,
        bytes: &[u8],
        limits: ProductionLimits,
        cancel: &AtomicBool,
    ) -> Result<ProductionSession, String> {
        package::reopen(self, bytes, limits, cancel)
    }
}

/// Immutable admitted revision. Construction is private so callers cannot bypass production validation.
pub struct Candidate {
    pub(crate) bundle: Vec<u8>,
    pub(crate) outputs: Vec<Resource>,
    pub(crate) revision: u64,
}
impl Candidate {
    pub fn bundle(&self) -> &[u8] {
        &self.bundle
    }
    pub fn outputs(&self) -> &[Resource] {
        &self.outputs
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn sha256(&self) -> String {
        digest(&self.bundle)
    }
}
fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}
fn valid_label(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
    {
        return Err(
            "Labels require 1..64 ASCII letters, digits, hyphens, underscores or dots".into(),
        );
    }
    Ok(())
}
fn valid_key(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 1024
        || value.split('/').any(|p| {
            let stem = p.split('.').next().unwrap_or("").to_ascii_uppercase();
            p.is_empty()
                || p == "."
                || p == ".."
                || p.ends_with(['.', ' '])
                || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                || (stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        })
        || value
            .chars()
            .any(|c| c.is_control() || "\\:<>\"|?*".contains(c))
    {
        return Err(format!("Invalid portable resource key: {value}"));
    }
    Ok(())
}
fn validate_resources(resources: &[Resource], limits: &ProductionLimits) -> Result<(), String> {
    if resources.len() > limits.resources {
        return Err("Resource count exceeds limit".into());
    }
    let mut keys = std::collections::BTreeSet::new();
    let mut total = 0usize;
    for resource in resources {
        valid_key(&resource.key)?;
        valid_label(&resource.kind)?;
        if !keys.insert(resource.key.to_lowercase()) {
            return Err("Duplicate resource key (case-insensitive)".into());
        }
        total = total
            .checked_add(resource.bytes.len())
            .ok_or("Resource size overflow")?;
        if resource.bytes.len() > limits.resource_bytes || total > limits.total_resource_bytes {
            return Err("Resource bytes exceed limit".into());
        }
    }
    for key in &keys {
        let mut prefix = String::new();
        for component in key.split('/').take(key.split('/').count() - 1) {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(component);
            if keys.contains(&prefix) {
                return Err("Resource file conflicts with a directory path".into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
