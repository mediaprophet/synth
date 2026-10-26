//! Small native explicit constant-grid source form; no inferred survey values.
use crate::graphical::{
    checkpoints::{GeoRasterCreateRequest, SourceDeclaration, SourceStatus},
    controller::EditorSnapshot,
    jobs::{self, Action, Context, HostHandle},
};
use dioxus::prelude::*;
use qualia_core_db::q42::m64_matter::{M64GeoRasterMetadata, M64RasterEdges, M64RasterRows};

const FIELDS: [(&str, &str); 17] = [
    ("New resource key", "synthetic/grid.m64"),
    ("Width (cells)", "2"),
    ("Height (cells)", "2"),
    ("Cell size (mm)", "1000"),
    ("First-cell latitude (degrees)", "0"),
    ("First-cell longitude (degrees)", "0"),
    ("Origin elevation (absolute mm)", "0"),
    ("Declared epoch year", "0"),
    ("Declared horizontal SRID", "0"),
    ("Declared vertical datum code", "0"),
    ("LUT bank epoch", "0"),
    ("Constant sample (absolute mm)", "0"),
    ("Rows: north or south", "south"),
    ("Edges: reject-partial or repeat-nearest", "repeat-nearest"),
    ("Nodata: none or signed integer mm", "none"),
    ("Source declaration", "synthetic constant grid; unqualified"),
    (
        "Unverified source reference",
        "urn:poet:source:synthetic-constant-grid",
    ),
];

fn request(
    filename: String,
    package_digest: [u8; 32],
    values: &[String],
) -> Result<GeoRasterCreateRequest, String> {
    if values.len() != FIELDS.len() || values.iter().any(|v| v.len() > 4096) {
        return Err("Raster form exceeds native text bounds".into());
    }
    fn number<T: std::str::FromStr>(values: &[String], index: usize) -> Result<T, String> {
        values[index]
            .parse()
            .map_err(|_| format!("Invalid {}", FIELDS[index].0))
    }
    // This form intentionally authors an explicit synthetic constant grid.
    // Real survey values can use the typed source-array API, with provenance
    // qualification separate. No invented observation is written as fact.
    let width: u64 = number(values, 1)?;
    let height: u64 = number(values, 2)?;
    let count = width
        .checked_mul(height)
        .filter(|n| *n > 0 && *n <= 4096)
        .ok_or("This native constant-grid form supports 1–4096 source cells")?
        as usize;
    let rows = match values[12].as_str() {
        "north" => M64RasterRows::North,
        "south" => M64RasterRows::South,
        _ => return Err("Rows must be north or south".into()),
    };
    let edges = match values[13].as_str() {
        "reject-partial" => M64RasterEdges::RejectPartial,
        "repeat-nearest" => M64RasterEdges::RepeatNearest,
        _ => return Err("Choose reject-partial or repeat-nearest edges".into()),
    };
    let nodata_sentinel_mm = if values[14] == "none" {
        None
    } else {
        Some(number(values, 14)?)
    };
    let metadata = M64GeoRasterMetadata {
        width,
        height,
        cell_size_mm: number(values, 3)?,
        origin_lat_deg: number(values, 4)?,
        origin_lon_deg: number(values, 5)?,
        origin_elevation_mm: number(values, 6)?,
        epoch_year: number(values, 7)?,
        srid: number(values, 8)?,
        vertical_datum: number(values, 9)?,
        rows,
        edges,
        nodata_sentinel_mm,
    };
    let sample: i32 = number(values, 11)?;
    let request = GeoRasterCreateRequest {
        filename,
        package_digest,
        key: values[0].clone(),
        metadata,
        lut_bank_epoch: number(values, 10)?,
        samples_mm: vec![sample; count],
        source: SourceDeclaration {
            status: SourceStatus::Synthetic,
            description: values[15].clone(),
            evidence_ref: Some(values[16].clone()),
            observation_claim: false,
        },
    };
    request.validate_bounds()?;
    Ok(request)
}

#[component]
pub(super) fn GeoRasterCreateView(
    host: HostHandle,
    current: EditorSnapshot,
    filename: String,
    package_digest: [u8; 32],
    busy: Signal<bool>,
    status: Signal<String>,
    on_saved: EventHandler<()>,
) -> Element {
    let mut values = use_signal(|| {
        FIELDS
            .iter()
            .map(|(_, default)| (*default).to_string())
            .collect::<Vec<_>>()
    });
    let save = move |_| {
        if busy() || !current.publication_is_current() {
            return;
        }
        let request = match request(filename.clone(), package_digest, &values()) {
            Ok(request) => request,
            Err(error) => {
                status.set(error);
                return;
            }
        };
        busy.set(true);
        let host = host.clone();
        let guard = host.clone();
        let current = current.clone();
        let origin = Context::from_snapshot(&current);
        spawn(async move {
            match jobs::run(host, Action::CreateGeoRaster { request, expected: current.source.clone() }, origin).await {
                Ok(reply) if jobs::completion_is_current(&guard, &current, &reply) => {
                    if reply.operation_succeeded { on_saved.call(()); }
                    status.set(reply.message);
                }
                Ok(_) => status.set("Raster producer context changed; refresh source/inventory. Started writes may require reconciliation.".into()),
                Err(error) => status.set(error),
            }
            busy.set(false);
        });
    };
    rsx! {
        fieldset { disabled: busy(),
            legend { "Produce explicit synthetic M64 v3 source" }
            p { "Source status is synthetic; qualification is unverified. Description and source reference below are retained in HMC resource metadata. Every frame value is an authored declaration; zero reference codes/epoch are unqualified. No legacy metadata or real terrain values are inferred. Use a new resource key." }
            for (index, (label, _)) in FIELDS.iter().enumerate() {
                label { key: "{index}", "{label} "
                    input { value: "{values()[index]}", aria_label: "{label}",
                        oninput: move |event| { let value = event.value(); if value.len() <= 4096 { values.write()[index] = value; } } }
                }
            }
            button { onclick: save, "Compile and add explicit v3 grid to this HMC" }
            p { "This form creates 1–4096 constant source cells through the canonical producer. The typed API supports explicit sample arrays under caller budgets. Origin and absolute samples remain independent; complete HMC/package review and scientific acceptance stay separate." }
        }
    }
}
