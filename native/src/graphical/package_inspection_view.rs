//! Derived typed presentation; canonical bytes/state/authority stay in the owner.
use super::super::{checkpoints::ResourceInspection, package::hex_digest};
use dioxus::prelude::*;

#[component]
pub(super) fn M64InspectionView(view: ResourceInspection) -> Element {
    let payload = &view.payload;
    let verification = payload.verification;
    let geographic_origin = payload
        .georaster
        .map(|meta| {
            format!(
                "First-cell-center latitude {:.17}°, longitude {:.17}°",
                meta.origin_lat_deg, meta.origin_lon_deg
            )
        })
        .unwrap_or_default();
    rsx! {
        section { style:"padding:10px;border:1px solid #688b9f;overflow-wrap:anywhere;",
            h3 { "Selected M64 resource" }
            p { "{view.request.filename} · {view.request.key} · saved source revision {view.saved_source_revision}" }
            p { "Resource SHA256 {hex_digest(&view.request.resource_digest)}" }
            p { "Wire version {payload.header.version}; {verification.chunks} chunks; {verification.total_records} records; {verification.record_crc_checked} record CRCs checked; {verification.records_without_checksum} records without embedded checksum." }
            if let Some(meta)=payload.georaster {
                p { "Developmental georaster v3 · source {meta.width} × {meta.height} cells · spacing {meta.cell_size_mm} mm" }
                p { "{geographic_origin}" }
                p { "Frame origin elevation {meta.origin_elevation_mm} mm; epoch {meta.epoch_year}; declared SRID {meta.srid}; declared vertical datum {meta.vertical_datum}." }
                p { "Local east/north grid; columns east; rows {meta.rows:?}; edge policy {meta.edges:?}. Outside source extent is padding, not additional observations." }
                p { "Nodata sentinel {meta.nodata_sentinel_mm:?}; first source sample {payload.first_source_sample:?}. Raw elevation samples are absolute integer mm in the declared vertical datum; origin elevation is independent, not an added offset." }
            } else {
                p { "Legacy v1/v2: source width/height, cell resolution, vertical-datum identifier, epoch, nodata and edge policy are not established by this payload. No georaster metadata was inferred." }
                p { "Stored f32 origin summary {payload.header.spatial_origin:?}; stored base/scale summary {payload.header.elevation_datum:?}. These are not lossless source-frame reconstruction." }
            }
            p { "Structural and listed CRC checks only. Declarations do not establish identifier authority, datum transformation, physical accuracy, rights or accepted model/package review. Inspection uses a bounded UI profile; larger resources require a separately selected host profile." }
        }
    }
}
