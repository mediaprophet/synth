//! Staged junction generation accounting. Not yet wired into a Poet route.
//! Structural units count bounded visits/probes, not elapsed time or instructions.
use crate::graphical::authoring::botanical::JunctionSource;
use crate::graphical::geometry_host::GeometryHost;
use crate::graphical::junction_generate::{generate_junction, JunctionLimits, JunctionScratchResult};
use qualia_core_db::specialized_libs::computational_geometry as cg;
use vibe::Value;

fn add(a: usize, b: usize) -> Result<usize, String> {
    a.checked_add(b).ok_or_else(|| "Junction work overflow".into())
}
fn mul(a: usize, b: usize) -> Result<usize, String> {
    a.checked_mul(b).ok_or_else(|| "Junction work overflow".into())
}

/// Bounded source scan for the junction's primitive-only Vibe record. This is
/// performed before `JunctionSource::read` clones or parses the graph. Charge
/// as it visits so even an early refusal consumes the operation's work cap.
/// Logical bytes are not heap bytes and do not include BTreeMap allocator cost.
fn scan_value(host: &mut GeometryHost, value: &Value, depth: usize,
    nodes: &mut usize, logical_bytes: &mut usize, max_nodes: usize, max_bytes: usize)
    -> Result<(), String> {
    if depth > 12 || *nodes >= max_nodes { return Err("Junction source depth or item limit exceeded".into()); }
    host.reserve_junction_work(1, 0, 0)?;
    *nodes += 1;
    *logical_bytes = add(*logical_bytes, std::mem::size_of::<Value>())?;
    if *logical_bytes > max_bytes { return Err("Junction source logical size exceeded".into()); }
    match value {
        Value::Null | Value::Bool(_) | Value::I64(_) | Value::U64(_) | Value::F64(_) => {},
        Value::String(s) => {
            host.reserve_junction_work(s.len(), 0, 0)?;
            *logical_bytes = add(*logical_bytes, s.len())?;
        }
        Value::List(values) => {
            for child in values { scan_value(host, child, depth + 1, nodes, logical_bytes, max_nodes, max_bytes)?; }
        }
        Value::Record(values) => {
            for (key, child) in values {
                host.reserve_junction_work(key.len(), 0, 0)?;
                *logical_bytes = add(*logical_bytes, key.len())?;
                if *logical_bytes > max_bytes { return Err("Junction source logical size exceeded".into()); }
                scan_value(host, child, depth + 1, nodes, logical_bytes, max_nodes, max_bytes)?;
            }
        }
        _ => return Err("Junction source contains unsupported value".into()),
    }
    if *logical_bytes > max_bytes { return Err("Junction source logical size exceeded".into()); }
    Ok(())
}

/// All limits must be host selected. `clone_payload` is a conservative logical
/// reservation for codec-owned copies; it is not a bound on allocator metadata.
/// Vibe's full session/environment cloning still needs separate accounting.
pub(crate) fn read_accounted(host: &mut GeometryHost, value: &Value,
    limits: JunctionLimits, max_nodes: usize, max_logical_bytes: usize,
    clone_payload: usize) -> Result<JunctionSource, String> {
    let mut nodes = 0;
    let mut bytes = 0;
    scan_value(host, value, 0, &mut nodes, &mut bytes, max_nodes, max_logical_bytes)?;
    if clone_payload < mul(bytes, 3)? { return Err("Junction clone reservation is too small".into()); }
    // Validation and graph parsing run twice in read; reserve two source scans
    // and the host-selected copy allowance before invoking the codec.
    host.reserve_junction_work(mul(add(nodes, bytes)?, 2)?, 0, clone_payload)?;
    JunctionSource::read(value, limits.botanical, limits.max_samples)
}

/// Reserve the host's logical source envelope before codec serialization.
/// The codec still owns its allocations; the subsequent size check cannot
/// certify allocator metadata or transient over-capacity.
pub(crate) fn serialize_accounted(host: &mut GeometryHost, source: &JunctionSource,
    limits: JunctionLimits, max_source_bytes: usize) -> Result<String, String> {
    if max_source_bytes == 0 { return Err("Junction source size cap unavailable".into()); }
    host.reserve_junction_work(mul(max_source_bytes, 4)?, 0, mul(max_source_bytes, 3)?)?;
    let encoded = source.source_expression(limits.botanical, limits.max_samples)?;
    if encoded.len() > max_source_bytes { return Err("Serialized junction source exceeds host cap".into()); }
    Ok(encoded)
}

/// Charge each generation from Core's checked grid bound, the field owner's
/// sample-segment evaluation bound, and host-selected probe budgets. Full
/// maximum Vec payload is charged rather than guessing actual capacities.
/// This does not account source/session serialization or total process memory.
pub(crate) fn generate_accounted(host: &mut GeometryHost, source: &JunctionSource,
    mut limits: JunctionLimits) -> Result<JunctionScratchResult, String> {
    let dims = source.dimensions;
    let bounds = cg::indexed_tetra_surface::tetra_surface_bounds(dims[0], dims[1], dims[2], limits.max_cells)
        .map_err(|e| format!("Junction grid refused: {e:?}"))?;
    if bounds.samples > limits.max_samples || bounds.vertices > limits.max_vertices
        || bounds.triangles > limits.max_triangles || limits.max_extraction_probes == 0
        || limits.topology_work == 0 || limits.max_payload_bytes == 0 {
        return Err("Junction host geometry limits exceeded".into());
    }
    let field = cg::tapered_branch_field::tapered_field_work(bounds.samples, 3,
        limits.max_samples, 3, limits.max_field_evaluations)
        .map_err(|e| format!("Junction field refused: {e:?}"))?;
    let axis = dims.iter().try_fold(0usize, |n, d| add(n, d - 1))?;
    if axis > limits.max_grid_axis_checks { return Err("Junction axis checks exceed limit".into()); }
    // Fixed extractor loop visits, excluding the separately charged probes:
    // samples, two slot clears, eight corners and six tetrahedra per cell,
    // and three directed-edge validation visits per output triangle.
    let fixed = add(add(bounds.samples, mul(2, bounds.edge_slots)?)?,
        add(mul(bounds.cells, 98)?, mul(bounds.triangles, 3)?)?)?;
    // Input scans and sample construction; generation also performs a bounded
    // four-organ pipe and graph check. This is a structural accounting model.
    // Two graph reads, four organ records, bounded 128-byte identifiers,
    // and bounded attractor lists. This is a deliberately coarse structural
    // allowance; review it again if the source owner changes.
    let graph_scans = add(16_384, mul(limits.botanical.max_attractors, 64)?)?;
    let mut work = add(field, add(mul(bounds.samples, 2)?, graph_scans)?)?;
    for part in [axis, fixed, limits.max_extraction_probes, limits.topology_work] {
        work = add(work, part)?;
    }
    // Bound simultaneous extractor and topology payload using Core owners at
    // worst output counts. The generator refuses any allocator over-capacity.
    let q = cg::surface_topology_qualification::surface_qualification_bounds(
        u32::try_from(bounds.vertices).map_err(|_| "Junction vertex count overflow")?, bounds.triangles)
        .map_err(|e| format!("Junction topology bounds refused: {e:?}"))?;
    use std::mem::size_of;
    let elements = [
        (bounds.samples, size_of::<cg::Point3>()), (bounds.samples, size_of::<f64>()),
        (bounds.vertices, size_of::<cg::Point3>()), (bounds.vertices, size_of::<[usize; 2]>()),
        (bounds.triangles, size_of::<[u32; 3]>()),
        (bounds.edge_slots, size_of::<cg::indexed_tetra_surface::TetraEdgeSlot>()),
        (q.half_edges, size_of::<cg::HalfEdge>()),
        (q.edge_slots, size_of::<cg::EdgeSlot>()),
        (q.vertex_starts, size_of::<u32>()), (q.visited_half_edges, size_of::<bool>()),
        (q.face_labels, size_of::<u32>()), (q.face_queue, size_of::<u32>()),
    ];
    let payload = elements.into_iter().try_fold(0usize, |sum, (count, bytes)| add(sum, mul(count, bytes)?))?;
    if payload > limits.max_payload_bytes { return Err("Junction payload bound exceeds host limit".into()); }
    // Every initialized element is at least a byte; charging payload bytes
    // covers all twelve initialization passes in these structural units.
    work = add(work, payload)?;
    host.reserve_junction_work(work, 4, payload)?;
    limits.max_payload_bytes = payload;
    generate_junction(source, limits)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_item_limit_refuses_before_codec_parse_or_clone() {
        let grants = ["capability.invoke", "ComputationalGeometry.space_colonize_3d",
            "ComputationalGeometry.pipe_model_radii", "ComputationalGeometry.extrude_profile"]
            .into_iter().map(str::to_owned).collect();
        let mut host = GeometryHost::new(grants, 10_000);
        let limits = host.junction_limits().unwrap();
        host.begin_junction_operation().unwrap();
        let oversized = Value::List(vec![Value::Null, Value::Null]);
        let error = read_accounted(&mut host, &oversized, limits, 1, 4096, 12288).unwrap_err();
        assert!(error.contains("item limit"));
    }
}
