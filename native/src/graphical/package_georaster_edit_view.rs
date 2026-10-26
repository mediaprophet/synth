//! Native controls for an inspected persisted elevation-only developmental v3 grid.
use crate::graphical::{
    checkpoints::{GeoRasterEditRequest, ResourceInspection},
    controller::EditorSnapshot,
    jobs::{self, Action, Context, HostHandle},
};
use dioxus::prelude::*;

#[component]
pub(super) fn GeoRasterEditView(
    host: HostHandle,
    current: EditorSnapshot,
    view: ResourceInspection,
    busy: Signal<bool>,
    status: Signal<String>,
    on_saved: EventHandler<()>,
) -> Element {
    let meta = view.payload.georaster;
    let mut spacing =
        use_signal(move || meta.map(|m| m.cell_size_mm.to_string()).unwrap_or_default());
    let mut origin = use_signal(move || {
        meta.map(|m| m.origin_elevation_mm.to_string())
            .unwrap_or_default()
    });
    let mut x = use_signal(|| "0".to_string());
    let mut y = use_signal(|| "0".to_string());
    let first = view.payload.first_source_sample.map(|sample| match sample {
        qualia_core_db::q42::m64_matter::M64RasterSample::ElevationMm(v)
        | qualia_core_db::q42::m64_matter::M64RasterSample::Nodata(v) => v,
    });
    let mut sample = use_signal(move || first.map(|v| v.to_string()).unwrap_or_default());
    let save = move |_| {
        if busy() || !current.publication_is_current() {
            return;
        }
        let request = (|| -> Result<GeoRasterEditRequest, String> {
            let request = GeoRasterEditRequest {
                resource: view.request.clone(),
                cell_size_mm: spacing()
                    .parse()
                    .map_err(|_| "Cell size requires positive integer mm")?,
                origin_elevation_mm: origin()
                    .parse()
                    .map_err(|_| "Origin requires signed integer mm")?,
                sample_x: x()
                    .parse()
                    .map_err(|_| "Column requires a nonnegative integer")?,
                sample_y: y()
                    .parse()
                    .map_err(|_| "Row requires a nonnegative integer")?,
                sample_mm: sample()
                    .parse()
                    .map_err(|_| "Sample requires signed integer mm")?,
            };
            request.validate_bounds()?;
            Ok(request)
        })();
        let request = match request {
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
            match jobs::run(host, Action::EditGeoRaster { request, expected: current.source.clone() }, origin).await {
                Ok(reply) if jobs::completion_is_current(&guard, &current, &reply) => {
                    if reply.operation_succeeded { on_saved.call(()); }
                    status.set(reply.message);
                }
                Ok(_) => status.set("Raster edit context changed; refresh source and inventory before another command. A started write may require reconciliation.".into()),
                Err(error) => status.set(error),
            }
            busy.set(false);
        });
    };
    rsx! {
        if let Some(meta) = meta {
            fieldset { disabled: busy(),
                legend { "Edit persisted M64 v3 elevation raster" }
                p { "Source extent {meta.width} × {meta.height}. Columns and rows are zero based. Samples are absolute integer mm; origin is independent. Datum, epoch, row direction and nodata declaration are retained." }
                label { "Cell size (mm) " input { value: "{spacing}", aria_label: "Raster cell size millimetres", oninput: move |e| spacing.set(e.value()) } }
                label { "Origin elevation (mm) " input { value: "{origin}", aria_label: "Raster origin elevation millimetres", oninput: move |e| origin.set(e.value()) } }
                label { "Column " input { value: "{x}", aria_label: "Raster source column", oninput: move |e| x.set(e.value()) } }
                label { "Row " input { value: "{y}", aria_label: "Raster source row", oninput: move |e| y.set(e.value()) } }
                label { "Absolute sample (mm) " input { value: "{sample}", aria_label: "Raster absolute sample millimetres", oninput: move |e| sample.set(e.value()) } }
                p { "Persisted nodata sentinel: {meta.nodata_sentinel_mm:?}. Enter that integer only to mark this selected sample nodata." }
                button { onclick: save, "Compile and save guarded raster edit" }
                p { "Opt-in developmental v3 only. Opaque, provided, unknown or observational source metadata holds editing until a qualified derivation contract exists. Synthetic origin declarations retain their original payload SHA; changed samples are derived data. Save retains other HMC resources; package review and scientific acceptance remain separate. Refresh inventory after saving." }
            }
        } else {
            p { "Legacy v1/v2 raster metadata is absent. Automatic upgrade/edit is refused until explicit source metadata is supplied through a qualified producer." }
        }
    }
}
