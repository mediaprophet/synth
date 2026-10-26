//! Inert unverified source declarations stored in existing HMC resource CBOR metadata.
use serde::{Deserialize, Serialize};
use std::io::Write;

pub const SOURCE_ANNOTATION_BYTES: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceStatus {
    Synthetic,
    Provided,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDeclaration {
    pub status: SourceStatus,
    pub description: String,
    /// A supplied reference, not a verified observation, right or authority.
    pub evidence_ref: Option<String>,
    pub observation_claim: bool,
}
impl SourceDeclaration {
    pub fn validate(&self) -> Result<(), String> {
        let bounded =
            |value: &str| !value.trim().is_empty() && value.len() <= 4096 && !value.contains('\0');
        if !bounded(&self.description) || self.evidence_ref.as_deref().is_some_and(|v| !bounded(v))
        {
            return Err("Raster source description/reference requires one to 4096 bytes".into());
        }
        if self.observation_claim
            && (self.status != SourceStatus::Provided || self.evidence_ref.is_none())
        {
            return Err("An observation claim requires explicit provided-source status and evidence reference; it remains unverified".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceQualification {
    Unverified,
}

/// App-owned resource annotation, not a scientific provenance graph or attestation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAnnotation {
    /// Declared origin payload only; a different current resource SHA is derived
    /// data and is not covered by any observation claim in this annotation.
    pub origin_payload_digest: [u8; 32],
    pub source_declaration: SourceDeclaration,
    pub qualification: SourceQualification,
}

pub(super) fn encode_annotation(
    source: &SourceDeclaration,
    origin_payload_digest: [u8; 32],
) -> Result<Vec<u8>, String> {
    source.validate()?;
    if origin_payload_digest == [0; 32] {
        return Err("Explicit origin payload digest is required for source metadata".into());
    }
    struct Bounded(Vec<u8>);
    impl Write for Bounded {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self
                .0
                .len()
                .checked_add(bytes.len())
                .is_none_or(|n| n > SOURCE_ANNOTATION_BYTES)
            {
                return Err(std::io::Error::other(
                    "Raster source annotation byte budget exceeded",
                ));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut writer = Bounded(Vec::new());
    writer
        .0
        .try_reserve_exact(SOURCE_ANNOTATION_BYTES)
        .map_err(|_| "Source annotation allocation refused")?;
    let annotation = SourceAnnotation {
        origin_payload_digest,
        source_declaration: source.clone(),
        qualification: SourceQualification::Unverified,
    };
    ciborium::ser::into_writer(&annotation, &mut writer)
        .map_err(|e| format!("Resource source metadata encoding refused: {e}"))?;
    Ok(writer.0)
}

/// Byte preservation alone cannot prove that arbitrary metadata remains true
/// after changing samples. Only this known synthetic-origin annotation admits
/// mutation; observational/provided/unknown metadata needs a derivation contract.
pub(super) fn ensure_editable_annotation(meta: Option<&[u8]>) -> Result<(), String> {
    let Some(bytes) = meta else {
        return Ok(());
    };
    if bytes.len() > SOURCE_ANNOTATION_BYTES {
        return Err("Selected raster metadata exceeds the bounded synthetic-origin profile; edit unassessed".into());
    }
    let mut cursor = std::io::Cursor::new(bytes);
    let annotation: SourceAnnotation = ciborium::de::from_reader(&mut cursor)
        .map_err(|_| "Selected raster has opaque/dependent metadata; edit held until a qualified metadata derivation contract")?;
    annotation.source_declaration.validate()?;
    if cursor.position() != bytes.len() as u64
        || annotation.origin_payload_digest == [0; 32]
        || annotation.qualification != SourceQualification::Unverified
        || annotation.source_declaration.status != SourceStatus::Synthetic
        || annotation.source_declaration.observation_claim
    {
        return Err("Selected raster metadata is not a bounded unverified synthetic-origin annotation; provided/unknown/observational source edits require a qualified derivation contract".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn georaster_source_declarations_remain_unverified_bounded_and_distinct() {
        for status in [
            SourceStatus::Synthetic,
            SourceStatus::Provided,
            SourceStatus::Unknown,
        ] {
            let source = SourceDeclaration {
                status,
                description: "explicit fixture declaration".into(),
                evidence_ref: Some("urn:fixture:unverified-source".into()),
                observation_claim: false,
            };
            let bytes = encode_annotation(&source, [7; 32]).unwrap();
            assert!(bytes.len() <= SOURCE_ANNOTATION_BYTES);
            let decoded: SourceAnnotation = ciborium::de::from_reader(bytes.as_slice()).unwrap();
            assert_eq!(decoded.source_declaration, source);
            assert_eq!(decoded.qualification, SourceQualification::Unverified);
            assert_eq!(decoded.origin_payload_digest, [7; 32]);
            assert_eq!(
                ensure_editable_annotation(Some(&bytes)).is_ok(),
                status == SourceStatus::Synthetic
            );
            let mut trailing = bytes;
            trailing.push(0);
            assert!(ensure_editable_annotation(Some(&trailing)).is_err());
        }
        let mut source = SourceDeclaration {
            status: SourceStatus::Provided,
            description: "declared observation; not verified".into(),
            evidence_ref: None,
            observation_claim: true,
        };
        assert!(source.validate().is_err());
        source.evidence_ref = Some("urn:fixture:claim-only".into());
        assert!(source.validate().is_ok());
        let observed = encode_annotation(&source, [7; 32]).unwrap();
        assert!(ensure_editable_annotation(Some(&observed)).is_err());
        assert!(encode_annotation(&source, [0; 32]).is_err());
        source.status = SourceStatus::Synthetic;
        assert!(source.validate().is_err());
        source.observation_claim = false;
        source.description = "d".repeat(4096);
        source.evidence_ref = Some("e".repeat(4096));
        assert!(encode_annotation(&source, [7; 32]).unwrap().len() <= SOURCE_ANNOTATION_BYTES);
        source.description = "x".repeat(4097);
        assert!(encode_annotation(&source, [7; 32]).is_err());
        source.description = "bounded".into();
        source.evidence_ref = Some(" ".into());
        assert!(encode_annotation(&source, [7; 32]).is_err());
        assert!(ensure_editable_annotation(None).is_ok());
        assert!(ensure_editable_annotation(Some(&[0xa1, 0x61, b'x', 1])).is_err());
    }
}
