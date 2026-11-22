use super::*;
use std::collections::BTreeMap;

/// General file production from VibeScript. Each file declares exactly one of
/// UTF-8 `text`, integer `bytes`, or an `input` resource key to retain intact.
/// This producer does not claim to validate arbitrary native media formats.
pub struct FilesProducer;
impl AssetProducer for FilesProducer {
    fn id(&self) -> &str {
        "files"
    }
    fn version(&self) -> &str {
        "2"
    }
    fn produce(
        &self,
        asset: &AssetValue,
        context: &ProductionContext<'_>,
    ) -> Result<Vec<Resource>, String> {
        produce_files(asset, context, true)
    }
}

/// Historical files/v1 interpretation, retained only for strict package reopening.
pub(super) struct LegacyFilesProducer;
impl AssetProducer for LegacyFilesProducer {
    fn id(&self) -> &str { "files" }
    fn version(&self) -> &str { "1" }
    fn produce(&self, asset: &AssetValue, context: &ProductionContext<'_>) -> Result<Vec<Resource>, String> {
        produce_files(asset, context, false)
    }
}

fn produce_files(asset: &AssetValue, context: &ProductionContext<'_>, collect_dependencies: bool) -> Result<Vec<Resource>, String> {
        let AssetValue::Record(asset) = asset else {
            return Err("asset must be a record".into());
        };
        if asset.len() != 1 {
            return Err("files asset accepts only the files field".into());
        }
        let Some(AssetValue::List(files)) = asset.get("files") else {
            return Err("asset.files must be a list".into());
        };
        if files.len() > context.limits.resources {
            return Err("Too many output files".into());
        }
        let mut outputs = Vec::new();
        let mut dependencies = BTreeMap::new();
        let mut total = 0usize;
        for file in files {
            context.check_cancelled()?;
            let AssetValue::Record(fields) = file else {
                return Err("Each file must be a record".into());
            };
            if let Some(handle) = fields.get("resource") {
                if fields.len() != 2 {
                    return Err("Generated file requires exactly key and resource".into());
                }
                let key = string(fields, "key")?;
                valid_key(key)?;
                let resource = context.generated_resource(handle)?;
                total = total
                    .checked_add(resource.bytes.len())
                    .ok_or("Output size overflow")?;
                if resource.bytes.len() > context.limits.resource_bytes
                    || total > context.limits.total_resource_bytes
                {
                    return Err("Output bytes exceed limit".into());
                }
                outputs.push(Resource {
                    key: key.into(),
                    kind: resource.kind.clone(),
                    bytes: resource.bytes.clone(),
                });
                if collect_dependencies { texture_dependencies(handle, context, &mut dependencies)?; }
                if collect_dependencies { encoded_dependencies(handle, context, &mut dependencies)?; }
                continue;
            }
            if fields.len() != 3 {
                return Err(
                    "Each file requires key, kind and exactly one of text/bytes/input".into(),
                );
            }
            let key = string(fields, "key")?;
            let kind = string(fields, "kind")?;
            valid_key(key)?;
            valid_label(kind)?;
            let content = match (fields.get("text"), fields.get("bytes"), fields.get("input")) {
                (Some(AssetValue::String(text)), None, None) => Content::Text(text),
                (None, Some(AssetValue::List(bytes)), None) => Content::Bytes(bytes),
                (None, None, Some(AssetValue::String(key))) => {
                    let input = context
                        .inputs
                        .iter()
                        .find(|r| &r.key == key)
                        .ok_or_else(|| format!("Missing input: {key}"))?;
                    if input.kind != kind {
                        return Err("Retained input kind must be preserved".into());
                    }
                    Content::Input(&input.bytes)
                }
                _ => return Err("File content must be text, byte list, or an input key".into()),
            };
            let len = match &content {
                Content::Text(v) => v.len(),
                Content::Bytes(v) => v.len(),
                Content::Input(v) => v.len(),
            };
            total = total.checked_add(len).ok_or("Output size overflow")?;
            if len > context.limits.resource_bytes || total > context.limits.total_resource_bytes {
                return Err("Output bytes exceed limit".into());
            }
            let bytes = match content {
                Content::Text(v) => v.as_bytes().to_vec(),
                Content::Input(v) => v.to_vec(),
                Content::Bytes(values) => values
                    .iter()
                    .map(|value| match value {
                        AssetValue::I64(n) => {
                            u8::try_from(*n).map_err(|_| "Byte outside 0..255".to_owned())
                        }
                        AssetValue::U64(n) => {
                            u8::try_from(*n).map_err(|_| "Byte outside 0..255".to_owned())
                        }
                        _ => Err("Bytes require integers in 0..255".into()),
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            };
            outputs.push(Resource {
                key: key.to_owned(),
                kind: kind.to_owned(),
                bytes,
            });
        }
        for (key, dependency) in dependencies {
            if let Some(existing) = outputs.iter().find(|r| r.key.eq_ignore_ascii_case(&key)) {
                if existing.kind != dependency.kind || existing.bytes != dependency.bytes {
                    return Err("Texture dependency conflicts with an explicit output".into());
                }
                continue;
            }
            total = total.checked_add(dependency.bytes.len()).ok_or("Output size overflow")?;
            if outputs.len() >= context.limits.resources || total > context.limits.total_resource_bytes {
                return Err("Texture dependencies exceed output budget".into());
            }
            outputs.push(dependency);
        }
        Ok(outputs)
}

enum Content<'a> {
    Text(&'a str),
    Bytes(&'a [AssetValue]),
    Input(&'a [u8]),
}
fn string<'a>(fields: &'a BTreeMap<String, AssetValue>, key: &str) -> Result<&'a str, String> {
    match fields.get(key) {
        Some(AssetValue::String(v)) => Ok(v),
        _ => Err(format!("File {key} must be a string")),
    }
}

/// Texture bytes remain original encoded payloads keyed by the MAT1 SHA-256 digest.
/// A shared digest is admitted once; conflicting metadata never silently wins.
fn texture_dependencies(
    handle: &AssetValue,
    context: &ProductionContext<'_>,
    dependencies: &mut BTreeMap<String, Resource>,
) -> Result<(), String> {
    let AssetValue::Record(handle) = handle else { return Ok(()); };
    let Some(textures) = handle.get("textures") else { return Ok(()); };
    let AssetValue::List(textures) = textures else { return Err("Texture dependencies must be a list".into()); };
    if textures.len() > context.limits.resources { return Err("Texture dependency count exceeds budget".into()); }
    for texture in textures {
        let AssetValue::Record(texture) = texture else { return Err("Texture dependency must be a record".into()); };
        let digest = string(texture, "digest")?;
        let mime_type = string(texture, "mime_type")?;
        let expected_kind = match mime_type {
            "image/png" => "png", "image/jpeg" => "jpeg", "image/ktx2" => "ktx2", "image/webp" => "webp",
            _ => return Err("Unsupported texture dependency MIME type".into()),
        };
        let resource = context.generated_resource(texture.get("resource").ok_or("Missing texture resource")?)?;
        if resource.kind != expected_kind || super::digest(&resource.bytes) != digest {
            return Err("Texture dependency digest or encoded kind mismatch".into());
        }
        let key = format!("textures/sha256/{digest}");
        if let Some(existing) = dependencies.get(&key) {
            if existing.kind != mime_type || existing.bytes != resource.bytes {
                return Err("Conflicting metadata or bytes for shared texture digest".into());
            }
            continue;
        }
        let retained_bytes: usize = dependencies.values().map(|r| r.bytes.len()).sum();
        if dependencies.len() >= context.limits.resources || retained_bytes.saturating_add(resource.bytes.len()) > context.limits.total_resource_bytes {
            return Err("Texture dependency bytes exceed budget".into());
        }
        dependencies.insert(key.clone(), Resource { key, kind: mime_type.into(), bytes: resource.bytes.clone() });
    }
    Ok(())
}

/// Explicit leaf resources of an encoded delivery manifest (for example a PNG sequence).
fn encoded_dependencies(handle:&AssetValue,context:&ProductionContext<'_>,dependencies:&mut BTreeMap<String,Resource>)->Result<(),String> {
    let AssetValue::Record(handle)=handle else{return Ok(());};
    let Some(items)=handle.get("dependencies") else{return Ok(());};
    let AssetValue::List(items)=items else{return Err("Encoded dependencies must be a list".into());};
    if items.len()>context.limits.resources{return Err("Encoded dependency count exceeds budget".into());}
    for item in items {
        let AssetValue::Record(item)=item else{return Err("Encoded dependency must be a record".into());};
        let key=string(item,"key")?;valid_key(key)?;
        let handle=item.get("resource").ok_or("Missing encoded dependency resource")?;
        if matches!(handle,AssetValue::Record(r) if r.contains_key("dependencies")){return Err("Nested manifest dependencies require explicit flattening".into());}
        let resource=context.generated_resource(handle)?;
        if let Some(existing)=dependencies.get(key) {
            if existing.kind!=resource.kind||existing.bytes!=resource.bytes{return Err("Conflicting encoded dependency".into());}
            continue;
        }
        let bytes:usize=dependencies.values().map(|r|r.bytes.len()).sum();
        if dependencies.len()>=context.limits.resources||bytes.saturating_add(resource.bytes.len())>context.limits.total_resource_bytes{return Err("Encoded dependencies exceed budget".into());}
        dependencies.insert(key.into(),Resource{key:key.into(),kind:resource.kind.clone(),bytes:resource.bytes.clone()});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::Digest;

    #[test]
    fn selected_encoded_asset_retains_verified_shared_texture_once_and_rejects_tampering() {
        let limits = ProductionLimits::default();
        let tool_limits = assets::tool_limits(&limits);
        let cancel = AtomicBool::new(false);
        let mut host = asset_production_vibe::AssetHost {
            workspace: assets::AssetWorkspace::default(), inputs: &[], limits: &tool_limits, cancel: &cancel, work: 0,
        };
        // This is a resource-transport fixture; image decoding is tested by the import owner.
        let texture_bytes = b"original encoded texture payload".to_vec();
        let digest = super::super::digest(&texture_bytes);
        let texture = host.admit_encoded("png", texture_bytes.clone()).unwrap();
        let profile = host
            .invoke(
                "hypermedia.locus.construct",
                &AssetValue::Record(BTreeMap::from([
                    ("shape".into(), AssetValue::String("circle".into())),
                    ("radius".into(), AssetValue::F64(4.0)),
                ])),
            )
            .unwrap();
        let mesh = host
            .invoke(
                "hypermedia.manifold.extrude",
                &AssetValue::Record(BTreeMap::from([
                    ("profile".into(), profile),
                    (
                        "path".into(),
                        AssetValue::List(vec![
                            AssetValue::List(vec![AssetValue::F64(0.0), AssetValue::F64(0.0), AssetValue::F64(0.0)]),
                            AssetValue::List(vec![AssetValue::F64(0.0), AssetValue::F64(0.0), AssetValue::F64(4.0)]),
                        ]),
                    ),
                ])),
            )
            .unwrap();
        let mut encoded = host
            .invoke(
                "hypermedia.manifold.compile_10d",
                &AssetValue::Record(BTreeMap::from([("mesh".into(), mesh)])),
            )
            .unwrap();
        let dependency = AssetValue::Record(BTreeMap::from([
            ("digest".into(), AssetValue::String(digest.clone())),
            ("mime_type".into(), AssetValue::String("image/png".into())),
            ("resource".into(), texture),
        ]));
        if let AssetValue::Record(record) = &mut encoded {
            record.insert("textures".into(), AssetValue::List(vec![dependency.clone(), dependency]));
        }
        let asset = AssetValue::Record(BTreeMap::from([("files".into(), AssetValue::List(vec![
            AssetValue::Record(BTreeMap::from([("key".into(), AssetValue::String("mesh.10d".into())), ("resource".into(), encoded.clone())]))
        ]))]));
        let context = ProductionContext {inputs:&[],limits:&limits,cancel:&cancel,assets:&host.workspace};
        let outputs = FilesProducer.produce(&asset, &context).unwrap();
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[1].key, format!("textures/sha256/{digest}"));
        assert_eq!(outputs[1].kind, "image/png");
        assert_eq!(outputs[1].bytes, texture_bytes);
        validate_resources(&outputs, &limits).unwrap();
        let bundle = super::super::package::build(&FilesProducer, &["fixture".into()], &[], &outputs, &limits).unwrap();
        let reader = qualia_core_db::bundle::BundleReader::parse(&bundle).unwrap();
        let raw_digest: [u8; 32] = sha2::Sha256::digest(&texture_bytes).into();
        let resolved = qualia_core_db::render::asset_package::resolve_hmc_texture_resource(&reader, &raw_digest).unwrap();
        assert_eq!(resolved.mime_type, "image/png");
        assert_eq!(resolved.bytes, texture_bytes);
        if let AssetValue::Record(record) = &mut encoded {
            if let Some(AssetValue::List(textures)) = record.get_mut("textures") {
                if let AssetValue::Record(texture) = &mut textures[0] {
                    texture.insert("digest".into(), AssetValue::String("0".repeat(64)));
                }
            }
        }
        assert!(texture_dependencies(&encoded, &context, &mut BTreeMap::new()).unwrap_err().contains("digest"));
    }
}
