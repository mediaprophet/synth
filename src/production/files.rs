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
        "1"
    }
    fn produce(
        &self,
        asset: &AssetValue,
        context: &ProductionContext<'_>,
    ) -> Result<Vec<Resource>, String> {
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
        Ok(outputs)
    }
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
