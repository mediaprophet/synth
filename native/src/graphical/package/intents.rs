//! Strict command intent schema; this never parses HMC or grants filesystem authority.
use super::*;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum ResourceIntent {
    Add {
        filename: String,
        key: String,
        kind: String,
    },
    Replace {
        filename: String,
        key: String,
        expected_resource_sha256: String,
    },
    Remove {
        key: String,
        expected_resource_sha256: String,
    },
}

pub(crate) fn resource_plan(intents: Vec<ResourceIntent>) -> Result<Vec<LibraryEdit>, String> {
    if intents.is_empty() || intents.len() > MAX_RESOURCE_EDITS {
        return Err("Resource plan requires 1 to 64 edits".into());
    }
    let mut plan = Vec::with_capacity(intents.len());
    for intent in intents {
        plan.push(match intent {
            ResourceIntent::Add {
                filename,
                key,
                kind,
            } => LibraryEdit::Import(ImportRequest {
                filename,
                key,
                kind,
                expected_resource: None,
            }),
            ResourceIntent::Replace {
                filename,
                key,
                expected_resource_sha256,
            } => LibraryEdit::Import(ImportRequest {
                filename,
                key,
                kind: String::new(),
                expected_resource: Some(parse_digest(&expected_resource_sha256)?),
            }),
            ResourceIntent::Remove {
                key,
                expected_resource_sha256,
            } => LibraryEdit::Remove {
                key,
                expected_resource: parse_digest(&expected_resource_sha256)?,
            },
        });
    }
    super::plan::validate_requests(&plan)?;
    Ok(plan)
}

pub fn parse_resource_plan_json(text: &str) -> Result<Vec<LibraryEdit>, String> {
    if text.len() > 64 * 1024 {
        return Err("Resource plan text exceeds the 64 KiB command budget".into());
    }
    resource_plan(
        serde_json::from_str(text).map_err(|error| format!("Resource command plan: {error}"))?,
    )
}

pub(crate) fn resource_plan_schema() -> Value {
    let common =
        json!({"op":{"type":"string"},"key":{"type":"string","minLength":1,"maxLength":4096}});
    let mut add = common.clone();
    add["op"] = json!({"const":"add"});
    add["filename"] = json!({"type":"string","minLength":1,"maxLength":240});
    add["kind"] = json!({"type":"string","minLength":1,"maxLength":4096});
    let mut replace = common.clone();
    replace["op"] = json!({"const":"replace"});
    replace["filename"] = add["filename"].clone();
    replace["expected_resource_sha256"] = json!({"type":"string","pattern":"^[0-9a-fA-F]{64}$"});
    let mut remove = common;
    remove["op"] = json!({"const":"remove"});
    remove["expected_resource_sha256"] = replace["expected_resource_sha256"].clone();
    json!({"type":"array","minItems":1,"maxItems":64,"items":{"oneOf":[
        {"type":"object","properties":add,"required":["op","filename","key","kind"],"additionalProperties":false},
        {"type":"object","properties":replace,"required":["op","filename","key","expected_resource_sha256"],"additionalProperties":false},
        {"type":"object","properties":remove,"required":["op","key","expected_resource_sha256"],"additionalProperties":false}
    ]}})
}
