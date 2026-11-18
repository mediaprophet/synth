//! Paged MCP views of the 150-morphism catalogue so Grok-sized RPC budgets
//! can list operations without the full ~326 KiB payload.
use super::*;
use qualia_mcp::authoring::ToolError;
use serde::Deserialize;
use serde_json::{json, Value};

pub(super) const DEFAULT_PAGE: usize = 40;
pub(super) const MAX_PAGE: usize = 150;

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct CatalogArg {
    pub category: Option<String>,
    pub id: Option<String>,
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub detail: Option<String>,
    pub q: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Detail {
    Ids,
    Summary,
    Full,
}

impl Detail {
    fn parse(raw: Option<&str>) -> Result<Self, ToolError> {
        match raw.unwrap_or("summary") {
            "ids" => Ok(Self::Ids),
            "summary" => Ok(Self::Summary),
            "full" => Ok(Self::Full),
            _ => Err(ToolError::new(
                "invalid_arguments",
                "detail must be ids, summary, or full",
            )),
        }
    }
}

fn page_bounds(
    total: usize,
    offset: Option<u64>,
    limit: Option<u64>,
) -> Result<(usize, usize, Option<usize>, usize), ToolError> {
    let offset = offset.unwrap_or(0) as usize;
    let limit = limit.unwrap_or(DEFAULT_PAGE as u64) as usize;
    if limit == 0 || limit > MAX_PAGE {
        return Err(ToolError::new(
            "invalid_arguments",
            format!("limit must be 1..={MAX_PAGE}"),
        ));
    }
    let start = offset.min(total);
    let end = (start + limit).min(total);
    let next = if end < total { Some(end) } else { None };
    Ok((start, end, next, limit))
}

fn envelope(
    schema: &str,
    total: usize,
    start: usize,
    end: usize,
    next: Option<usize>,
    limit: usize,
    detail: Detail,
    category: Option<&str>,
    items_key: &str,
    items: Value,
) -> Value {
    json!({
        "schema": schema,
        "count": total,
        "returned": end.saturating_sub(start),
        "offset": start,
        "limit": limit,
        "next_offset": next,
        "detail": match detail {
            Detail::Ids => "ids",
            Detail::Summary => "summary",
            Detail::Full => "full",
        },
        "filter_category": category,
        "hint": "Pass q=xpbd (or nla, ik, navmesh, gis, n3) to search. Pass detail=full and a category or id for one morphism contract. Default page is 40 summaries so the RPC stays inside Grok's result budget. synth_find is the agent entry point.",
        items_key: items,
    })
}

fn query_tokens(parsed: &CatalogArg) -> Vec<String> {
    parsed
        .q
        .as_deref()
        .map(crate::agent_index::parse_tokens)
        .unwrap_or_default()
}

pub(super) fn operations(args: Value) -> Result<Value, ToolError> {
    let parsed: CatalogArg = serde_json::from_value(args)
        .map_err(|_| ToolError::new("invalid_arguments", "Invalid operations filter"))?;
    let detail = Detail::parse(parsed.detail.as_deref())?;
    let morphisms: Vec<_> = crate::production::assets::ontology::MORPHISMS
        .iter()
        .filter(|m| {
            parsed
                .category
                .as_deref()
                .is_none_or(|cat| m.category == cat)
                && parsed.id.as_deref().is_none_or(|id| m.id == id)
                && crate::agent_index::morphism_matches(m, &query_tokens(&parsed))
        })
        .collect();
    let (start, end, next, limit) = page_bounds(morphisms.len(), parsed.offset, parsed.limit)?;
    let page = &morphisms[start..end];
    let items = match detail {
        Detail::Ids => json!(page.iter().map(|m| m.id).collect::<Vec<_>>()),
        Detail::Summary => json!(page
            .iter()
            .map(|m| json!({
                "id": m.id,
                "category": m.category,
                "arguments": m.arguments,
                "summary": m.summary,
            }))
            .collect::<Vec<_>>()),
        Detail::Full => {
            let catalogue = crate::production::assets::catalogue();
            let all = catalogue["operations"].as_array().cloned().unwrap_or_default();
            json!(page
                .iter()
                .filter_map(|m| all.iter().find(|op| op["id"].as_str() == Some(m.id)).cloned())
                .collect::<Vec<_>>())
        }
    };
    Ok(envelope(
        "synth.operations/v2",
        morphisms.len(),
        start,
        end,
        next,
        limit,
        detail,
        parsed.category.as_deref(),
        "operations",
        items,
    ))
}

pub(super) fn morphisms(args: Value) -> Result<Value, ToolError> {
    let parsed: CatalogArg = serde_json::from_value(args)
        .map_err(|_| ToolError::new("invalid_arguments", "Invalid category filter"))?;
    let detail = Detail::parse(parsed.detail.as_deref())?;
    let morphisms: Vec<_> = crate::production::assets::ontology::MORPHISMS
        .iter()
        .filter(|m| {
            parsed
                .category
                .as_deref()
                .is_none_or(|cat| m.category == cat)
                && parsed.id.as_deref().is_none_or(|id| m.id == id)
                && crate::agent_index::morphism_matches(m, &query_tokens(&parsed))
        })
        .collect();
    let (start, end, next, limit) = page_bounds(morphisms.len(), parsed.offset, parsed.limit)?;
    let page = &morphisms[start..end];
    let items = match detail {
        Detail::Ids => json!(page.iter().map(|m| m.id).collect::<Vec<_>>()),
        Detail::Full => json!(page
            .iter()
            .map(|m| crate::production::assets::ontology::morphism_to_json(m))
            .collect::<Vec<_>>()),
        Detail::Summary => json!(page
            .iter()
            .map(|m| json!({
                "id": m.id,
                "category": m.category,
                "domain": m.domain,
                "codomain": m.codomain,
                "arguments": m.arguments,
                "teleology": m.teleology,
            }))
            .collect::<Vec<_>>()),
    };
    Ok(envelope(
        "hypermedia.categorical-morphisms/v2",
        morphisms.len(),
        start,
        end,
        next,
        limit,
        detail,
        parsed.category.as_deref(),
        "morphisms",
        items,
    ))
}

pub(super) fn contracts(args: Value) -> Result<Value, ToolError> {
    let parsed: CatalogArg = serde_json::from_value(args)
        .map_err(|_| ToolError::new("invalid_arguments", "Invalid contracts filter"))?;
    let detail = Detail::parse(parsed.detail.as_deref())?;
    let full = contracts::catalogue();
    let all = full["operations"].as_array().cloned().unwrap_or_default();
    let filtered: Vec<Value> = all
        .into_iter()
        .filter(|op| {
            let id = op["id"].as_str().unwrap_or("");
            parsed.id.as_deref().is_none_or(|want| id == want)
                && parsed.category.as_deref().is_none_or(|cat| {
                    crate::production::assets::ontology::MORPHISMS
                        .iter()
                        .find(|m| m.id == id)
                        .is_some_and(|m| m.category == cat)
                })
                && crate::production::assets::ontology::MORPHISMS
                    .iter()
                    .find(|m| m.id == id)
                    .is_some_and(|m| crate::agent_index::morphism_matches(m, &query_tokens(&parsed)))
        })
        .collect();
    let (start, end, next, limit) = page_bounds(filtered.len(), parsed.offset, parsed.limit)?;
    let page = &filtered[start..end];
    let items = match detail {
        Detail::Ids => json!(page
            .iter()
            .filter_map(|op| op["id"].as_str())
            .collect::<Vec<_>>()),
        Detail::Summary => json!(page
            .iter()
            .map(|op| json!({
                "id": op["id"],
                "execution_host": op["execution_host"],
                "output": op["output"]["type"],
            }))
            .collect::<Vec<_>>()),
        Detail::Full => json!(page),
    };
    let mut body = envelope(
        "synth.operation-contracts/v2",
        filtered.len(),
        start,
        end,
        next,
        limit,
        detail,
        parsed.category.as_deref(),
        "operations",
        items,
    );
    if detail == Detail::Full && parsed.id.is_none() && parsed.offset.unwrap_or(0) == 0 {
        body["output_recipes"] = full["output_recipes"].clone();
        body["production_profiles"] = full["production_profiles"].clone();
        body["fallback"] = full["fallback"].clone();
    }
    Ok(body)
}

pub(super) fn filter_schema() -> Value {
    json!({
        "category": {"type": "string", "maxLength": 64},
        "id": {"type": "string", "maxLength": 128},
        "q": {"type": "string", "maxLength": 128, "description": "Keyword search (xpbd, nla, ik, navmesh, gis, n3). Prefer synth_find."},
        "offset": {"type": "integer", "minimum": 0},
        "limit": {"type": "integer", "minimum": 1, "maximum": MAX_PAGE},
        "detail": {"type": "string", "enum": ["ids", "summary", "full"], "description": "Default summary, 40 rows. Use detail=full with category, id, or q; the unpaged 150-op dump overflows Grok MCP."}
    })
}
