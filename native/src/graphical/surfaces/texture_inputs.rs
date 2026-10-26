//! Bounded immutable snapshot adapters; all wire formats and image codecs are Core-owned.
use super::resources::SurfaceTextureLimits;
use qualia_core_db::{
    bundle::{BundleReader, BundleWriter},
    render::{
        asset_package::{resolve_hmc_texture_resource, texture_hmc_key},
        assets::TextureDependency,
        texture_decode::{self, TextureColorSpace, TextureDecodeLimits},
    },
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
pub struct DetailInput {
    pub rgba: Vec<u8>,
    pub source: TextureDependency,
    pub width: u32,
    pub height: u32,
    pub colour_space: TextureColorSpace,
}
fn checked(bytes: &[u8], limits: SurfaceTextureLimits) -> Result<BundleReader<'_>, String> {
    if bytes.len() > limits.package_bytes {
        return Err("Surface snapshot exceeds package budget".into());
    }
    let reader = BundleReader::parse(bytes).map_err(|e| e.to_string())?;
    if reader.flags() != 0
        || reader
            .entries()
            .iter()
            .any(|e| !reader.verify_entry(&e.key))
    {
        return Err(
            "Retained HMC has unsupported flags or invalid entry SHA; no rehash/merge".into(),
        );
    }
    Ok(reader)
}
pub fn length(
    bytes: &[u8],
    limits: SurfaceTextureLimits,
    digest: [u8; 32],
) -> Result<usize, String> {
    let reader = checked(bytes, limits)?;
    let r = resolve_hmc_texture_resource(&reader, &digest).map_err(|e| e.to_string())?;
    if r.bytes.len() > limits.encoded_bytes {
        return Err("Source texture exceeds encoded budget".into());
    }
    Ok(r.bytes.len())
}
pub fn retained(
    bytes: &[u8],
    limits: SurfaceTextureLimits,
) -> Result<Vec<qualia_core_db::poet_host::invoke::studio_store::checkpoint::ResourceData>, String>
{
    let reader = checked(bytes, limits)?;
    if bytes
        .len()
        .checked_mul(4)
        .is_none_or(|n| n > limits.working_bytes)
    {
        return Err("Retained input snapshot copying exceeds working admission".into());
    }
    if reader
        .entry(qualia_core_db::poet_host::invoke::studio_store::checkpoint::SOURCE_RESOURCE_KEY)
        .is_some_and(|e| e.meta.is_some())
    {
        return Err("Bound source-entry metadata requires explicit source-carrier continuity; snapshot-only compile refuses rather than dropping it".into());
    }
    reader
        .entries()
        .iter()
        .filter(|e| {
            e.key
                != qualia_core_db::poet_host::invoke::studio_store::checkpoint::SOURCE_RESOURCE_KEY
        })
        .map(|e| {
            Ok(
                qualia_core_db::poet_host::invoke::studio_store::checkpoint::ResourceData {
                    key: e.key.clone(),
                    kind: e.kind.clone(),
                    bytes: reader
                        .get(&e.key)
                        .ok_or("Retained payload absent")?
                        .to_vec(),
                    meta: e.meta.clone(),
                },
            )
        })
        .collect()
}
pub fn dependency(
    bytes: &[u8],
    limits: SurfaceTextureLimits,
    digest: [u8; 32],
) -> Result<TextureDependency, String> {
    if bytes.len() > limits.package_bytes {
        return Err("Surface snapshot exceeds package budget".into());
    }
    let reader = checked(bytes, limits)?;
    let r = resolve_hmc_texture_resource(&reader, &digest).map_err(|e| e.to_string())?;
    if r.bytes.len() > limits.encoded_bytes {
        return Err("Original texture exceeds encoded budget".into());
    }
    Ok(TextureDependency {
        digest,
        mime_type: r.mime_type.into(),
        bytes: r.bytes.to_vec(),
    })
}
pub fn decode(
    bytes: &[u8],
    limits: SurfaceTextureLimits,
    digest: [u8; 32],
    side: u32,
) -> Result<DetailInput, String> {
    decode_any(bytes, limits, digest, Some(side))
}
pub fn decode_any(
    bytes: &[u8],
    limits: SurfaceTextureLimits,
    digest: [u8; 32],
    side: Option<u32>,
) -> Result<DetailInput, String> {
    decode_inner(bytes, limits, digest, side, false)
}
pub fn decode_mask(
    bytes: &[u8],
    limits: SurfaceTextureLimits,
    digest: [u8; 32],
    side: u32,
) -> Result<DetailInput, String> {
    decode_inner(bytes, limits, digest, Some(side), true)
}
fn decode_inner(
    bytes: &[u8],
    limits: SurfaceTextureLimits,
    digest: [u8; 32],
    side: Option<u32>,
    allow_linear: bool,
) -> Result<DetailInput, String> {
    if bytes.len() > limits.package_bytes {
        return Err("Surface snapshot exceeds package budget".into());
    }
    let reader = checked(bytes, limits)?;
    let r = resolve_hmc_texture_resource(&reader, &digest).map_err(|e| e.to_string())?;
    let bound = TextureDecodeLimits {
        max_encoded_bytes: limits.encoded_bytes,
        max_width: limits.width,
        max_height: limits.height,
        max_decoded_bytes: limits.decoded_bytes,
    };
    let (info, scratch) =
        texture_decode::inspect_hmc_texture_requirements(&r, bound).map_err(|e| e.to_string())?;
    if side.is_some_and(|side| info.width != side || info.height != side)
        || (!allow_linear && info.color_space == TextureColorSpace::Linear)
    {
        return Err(
            "Paint base must match the authored square extent and explicit sRGB base-colour role"
                .into(),
        );
    }
    let metadata = reader.entries().iter().try_fold(0usize, |n, e| {
        n.checked_add(e.meta.as_ref().map_or(0, Vec::len))
            .ok_or("Decode metadata size overflow")
    })?;
    let work = info
        .rgba8_bytes
        .checked_add(scratch)
        .and_then(|n| n.checked_add(r.bytes.len()))
        .and_then(|n| n.checked_add(bytes.len()))
        .and_then(|n| metadata.checked_mul(2).and_then(|m| n.checked_add(m)))
        .ok_or("Detail decode size overflow")?;
    if work > limits.working_bytes {
        return Err("Detail decode working budget exceeded".into());
    }
    let mut rgba = vec![0; info.rgba8_bytes];
    let mut scratch = vec![0; scratch];
    let actual = texture_decode::decode_hmc_texture_rgba8_into(&r, bound, &mut rgba, &mut scratch)
        .map_err(|e| e.to_string())?;
    if actual != info {
        return Err("Detail decoder changed admitted metadata".into());
    }
    // Unknown transfer metadata is interpreted under this explicit authored
    // base-colour/sRGB recipe. It does not acquire embedded metadata evidence.
    Ok(DetailInput {
        rgba,
        width: info.width,
        height: info.height,
        colour_space: if allow_linear {
            info.color_space
        } else {
            TextureColorSpace::Srgb
        },
        source: TextureDependency {
            digest,
            mime_type: r.mime_type.into(),
            bytes: r.bytes.to_vec(),
        },
    })
}
pub fn merge(
    bytes: &[u8],
    limits: SurfaceTextureLimits,
    deps: &[TextureDependency],
) -> Result<Vec<u8>, String> {
    if bytes.len() > limits.package_bytes || deps.len() > limits.bindings {
        return Err("Texture merge input budget exceeded".into());
    }
    let reader = checked(bytes, limits)?;
    let mut pending = BTreeMap::new();
    let mut total = bytes.len();
    let mut encoded = 0usize;
    for d in deps {
        if d.bytes.len() > limits.encoded_bytes
            || d.mime_type.len() > 128
            || !d.mime_type.starts_with("image/")
            || d.mime_type.contains('\0')
        {
            return Err("Texture merge dependency violates image budget/kind".into());
        }
        if <[u8; 32]>::from(Sha256::digest(&d.bytes)) != d.digest {
            return Err("Texture merge digest does not match bytes".into());
        }
        let key = texture_hmc_key(&d.digest);
        if let Some(old) = reader.entry(&key) {
            if old.kind != d.mime_type
                || reader.get(&key) != Some(d.bytes.as_slice())
                || !reader.verify_entry(&key)
            {
                return Err("Texture merge conflicts with retained resource; preserved".into());
            }
            continue;
        }
        if let Some(old) = pending.insert(key.clone(), d) {
            if old.bytes != d.bytes || old.mime_type != d.mime_type {
                return Err("Conflicting generated texture declarations".into());
            }
            continue;
        }
        total = total
            .checked_add(d.bytes.len())
            .and_then(|n| n.checked_add(key.len() + d.mime_type.len() + 512))
            .ok_or("Texture package size overflow")?;
        encoded = encoded
            .checked_add(d.bytes.len())
            .ok_or("Texture merge size overflow")?;
        if total > limits.package_bytes {
            return Err("Texture package output budget exceeded before allocation".into());
        }
    }
    let metadata = reader.entries().iter().try_fold(0usize, |n, e| {
        n.checked_add(e.meta.as_ref().map_or(0, Vec::len))
            .ok_or("Texture metadata size overflow")
    })?;
    // Original Arc, retained original-input copies (at most encoded_bytes),
    // generated dependencies, writer payloads, build Vec, decoded index/meta,
    // and the final Arc copy have overlapping lifetimes. This is conservative
    // declared admission, not an instrumented peak measurement.
    let peak = bytes
        .len()
        .checked_add(limits.encoded_bytes)
        .and_then(|n| n.checked_add(encoded))
        .and_then(|n| total.checked_mul(2).and_then(|v| n.checked_add(v)))
        .and_then(|n| metadata.checked_mul(4).and_then(|m| n.checked_add(m)));
    if peak.is_none_or(|n| n > limits.working_bytes) {
        return Err("Texture merge live-copy working admission exceeded".into());
    }
    let mut writer = BundleWriter::new();
    for e in reader.entries() {
        writer
            .add_file(
                &e.key,
                &e.kind,
                reader
                    .get(&e.key)
                    .ok_or("Retained resource absent")?
                    .to_vec(),
                e.meta.clone(),
            )
            .map_err(|e| e.to_string())?;
    }
    for (key, d) in pending {
        writer
            .add_file(key, &d.mime_type, d.bytes.clone(), None)
            .map_err(|e| e.to_string())?;
    }
    let out = writer.build().map_err(|e| e.to_string())?;
    if out.len() > limits.package_bytes {
        return Err("Texture package final size exceeds budget".into());
    }
    Ok(out)
}
