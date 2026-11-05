//! Staged, nonpublishing junction generator. This is not a Poet route or a
//! master-quality certificate. Caller must independently bound source/session
//! memory and debit one aggregate operation ledger before invoking this code.
use super::authoring::botanical::{BotanicalLimits, JunctionSource};
use super::botanical_graph_host::BotanicalNode;
use super::fork_adopted_adapter::adopted_single_junction;
use qualia_core_db::specialized_libs::computational_geometry as cg;
use cg::indexed_tetra_surface::{self as tetra, TetraEdgeSlot};
use cg::surface_topology_qualification as qualify;
use cg::{EdgeSlot, HalfEdge};

/// Limits are selected by the host, never by authored source. `max_payload_bytes`
/// bounds accepted initialized Vec capacity here; allocation may transiently exceed
/// it before the capacity is checked. It excludes allocator overhead, source/session
/// storage, stack use, GPU memory or total process memory. All work caps must be
/// reserved from the host's shared operation ledger before admission.
#[derive(Clone, Copy)]
pub struct JunctionLimits {
    pub botanical: BotanicalLimits,
    pub max_samples: usize,
    pub max_cells: usize,
    pub max_grid_axis_checks: usize,
    pub max_vertices: usize,
    pub max_triangles: usize,
    pub max_field_evaluations: usize,
    pub max_extraction_probes: usize,
    pub topology_work: usize,
    pub max_payload_bytes: usize,
}

/// Sample-edge identifiers are local grid provenance, not organ identifiers.
pub struct JunctionScratchResult {
    pub vertices: Vec<cg::Point3>,
    pub triangles: Vec<[u32; 3]>,
    pub sample_edges: Vec<[usize; 2]>,
    pub topology: qualify::SurfaceQualificationReceipt,
}

fn add_bytes<T>(total: usize, n: usize) -> Result<usize, String> {
    n.checked_mul(std::mem::size_of::<T>()).and_then(|v| total.checked_add(v))
        .ok_or_else(|| "Junction payload size overflow".into())
}
// Reserve first, account actual capacity, and refuse before initialization if
// the allocator gave more capacity than the remaining host payload allowance.
// This excludes allocator metadata, source/session storage and process memory.
fn filled<T: Clone>(n: usize, value: T, allocated: &mut usize, cap: usize) -> Result<Vec<T>, String> {
    let mut result = Vec::new();
    result.try_reserve_exact(n).map_err(|_| "Junction allocation refused")?;
    let actual = add_bytes::<T>(*allocated, result.capacity())?;
    if actual > cap { return Err("Junction actual capacity limit exceeded".into()); }
    *allocated = actual;
    result.resize(n, value);
    Ok(result)
}
fn point(p: [f64; 3]) -> cg::Point3 { cg::Point3::new(p[0], p[1], p[2]) }

/// Builds a diagnostic scratch surface from a four-organ, v1 graph. Returned
/// buffers must not be published until the complete edit/submit/replay ledger,
/// prior projection, attribute mapping and source binding are independently
/// validated. Failures may dirty only local scratch; input source is untouched.
pub fn generate_junction(source: &JunctionSource, limits: JunctionLimits) -> Result<JunctionScratchResult, String> {
    source.validate(limits.botanical, limits.max_samples)?;
    let graph = source.graph(limits.botanical)?;
    let b = tetra::tetra_surface_bounds(source.dimensions[0], source.dimensions[1], source.dimensions[2], limits.max_cells)
        .map_err(|e| format!("Grid bounds refused: {e:?}"))?;
    if b.samples > limits.max_samples || b.vertices > limits.max_vertices || b.triangles > limits.max_triangles
        || limits.max_extraction_probes == 0 || limits.topology_work == 0 {
        return Err("Junction host geometry limits exceeded".into());
    }
    let axis_checks = source.dimensions.iter().try_fold(0usize, |sum, n| sum.checked_add(n - 1))
        .ok_or("Grid axis check overflow")?;
    if axis_checks > limits.max_grid_axis_checks { return Err("Grid axis check limit exceeded".into()); }
    // Verify the exact coordinate expression used by both the field sampling
    // and extractor. Finite, positive spacing alone may round distinct indices
    // to the same f64 value at large coordinate magnitudes.
    for axis in 0..3 {
        let mut previous = source.origin_m[axis];
        for i in 1..source.dimensions[axis] {
            let current = source.origin_m[axis] + (i as f64) * source.spacing_m;
            if !current.is_finite() || current <= previous { return Err("Grid sample coordinates collapse".into()); }
            previous = current;
        }
    }    cg::tapered_branch_field::tapered_field_work(b.samples, 3, limits.max_samples, 3, limits.max_field_evaluations)
        .map_err(|e| format!("Field work refused: {e:?}"))?;
    // Reserve worst extractor payload, then separately reserve topology payload
    // against actual extracted counts while all extractor buffers remain live.
    let mut payload = 0usize;
    payload = add_bytes::<cg::Point3>(payload, b.samples)?;
    payload = add_bytes::<f64>(payload, b.samples)?;
    payload = add_bytes::<cg::Point3>(payload, b.vertices)?;
    payload = add_bytes::<[u32; 3]>(payload, b.triangles)?;
    payload = add_bytes::<[usize; 2]>(payload, b.vertices)?;
    payload = add_bytes::<TetraEdgeSlot>(payload, b.edge_slots)?;
    if payload > limits.max_payload_bytes { return Err("Junction payload limit exceeded".into()); }

    let parents = [cg::ROOT_PARENT, 0, 1, 1];
    let mut radii = [0.0; 4];
    let mut scratch = [0u32; 4];
    cg::pipe_model_radii(&parents, graph.tip_radius_m, graph.pipe_exponent, &mut scratch, &mut radii)
        .map_err(|e| format!("Pipe model refused: {e:?}"))?;
    let organs: [super::authoring::botanical::Organ; 4] = graph.organs.try_into()
        .map_err(|_| "Expected four adopted organs")?;
    let mut index = 0;
    let nodes = organs.map(|organ| {
        let radius_m = radii[index];
        index += 1;
        BotanicalNode { key: organ.key, parent_key: organ.parent, position_m: organ.position_m, radius_m }
    });
    let edges = adopted_single_junction(&nodes, source.min_axis_m, source.max_same_direction_cosine)
        .map_err(|e| e.to_owned())?;
    let segments = [edges[0].field, edges[1].field, edges[2].field];

    let mut allocated = 0usize;
    let mut samples = filled(b.samples, cg::Point3::default(), &mut allocated, limits.max_payload_bytes)?;
    let mut scalars = filled(b.samples, 0.0f64, &mut allocated, limits.max_payload_bytes)?;
    let [nx, ny, nz] = source.dimensions;
    for z in 0..nz { for y in 0..ny { for x in 0..nx {
        let p = [source.origin_m[0] + (x as f64)*source.spacing_m,
                 source.origin_m[1] + (y as f64)*source.spacing_m,
                 source.origin_m[2] + (z as f64)*source.spacing_m];
        if p.iter().any(|v| !v.is_finite()) { return Err("Nonfinite sample point".into()); }
        samples[x + nx*y + nx*ny*z] = point(p);
    } } }
    cg::tapered_branch_field::tapered_branch_field_into(&samples, &segments, source.blend_m, source.min_axis_m,
        source.projection_tolerance_m, limits.max_samples, 3, limits.max_field_evaluations, &mut scalars)
        .map_err(|e| format!("Tapered field refused: {e:?}"))?;
    let mut vertices = filled(b.vertices, cg::Point3::default(), &mut allocated, limits.max_payload_bytes)?;
    let mut triangles = filled(b.triangles, [0u32; 3], &mut allocated, limits.max_payload_bytes)?;
    let mut sample_edges = filled(b.vertices, [0usize; 2], &mut allocated, limits.max_payload_bytes)?;
    let mut edge_slots = filled(b.edge_slots, TetraEdgeSlot::default(), &mut allocated, limits.max_payload_bytes)?;
    let (vc, tc) = tetra::indexed_tetra_surface_into(&scalars, source.dimensions, point(source.origin_m),
        cg::Point3::new(source.spacing_m, source.spacing_m, source.spacing_m), 0.0,
        limits.max_cells, limits.max_extraction_probes, source.min_area_sq_m4,
        &mut vertices, &mut triangles, &mut sample_edges, &mut edge_slots)
        .map_err(|e| format!("Junction extraction refused: {e:?}"))?;
    vertices.truncate(vc); triangles.truncate(tc); sample_edges.truncate(vc);
    let vertex_count = u32::try_from(vc).map_err(|_| "Junction vertex count overflow")?;
    let q = qualify::surface_qualification_bounds(vertex_count, tc).map_err(|e| format!("Topology bounds refused: {e:?}"))?;
    // Include actual capacities already allocated when preflighting topology.
    payload = allocated;
    payload = add_bytes::<HalfEdge>(payload, q.half_edges)?;
    payload = add_bytes::<EdgeSlot>(payload, q.edge_slots)?;
    payload = add_bytes::<u32>(payload, q.vertex_starts)?;
    payload = add_bytes::<bool>(payload, q.visited_half_edges)?;
    payload = add_bytes::<u32>(payload, q.face_labels)?;
    payload = add_bytes::<u32>(payload, q.face_queue)?;
    if payload > limits.max_payload_bytes { return Err("Topology payload limit exceeded".into()); }
    let mut half_edges = filled(q.half_edges, HalfEdge::default(), &mut allocated, limits.max_payload_bytes)?;
    let mut topo_slots = filled(q.edge_slots, EdgeSlot::default(), &mut allocated, limits.max_payload_bytes)?;
    let mut starts = filled(q.vertex_starts, 0u32, &mut allocated, limits.max_payload_bytes)?;
    let mut visited = filled(q.visited_half_edges, false, &mut allocated, limits.max_payload_bytes)?;
    let mut labels = filled(q.face_labels, 0u32, &mut allocated, limits.max_payload_bytes)?;
    let mut queue = filled(q.face_queue, 0u32, &mut allocated, limits.max_payload_bytes)?;
    let mut work = limits.topology_work;
    let topology = qualify::qualify_surface_topology_budgeted_into(vertex_count, &triangles, &mut work,
        &mut half_edges, &mut topo_slots, &mut starts, &mut visited, &mut labels, &mut queue)
        .map_err(|e| format!("Junction topology refused: {e:?}"))?;
    Ok(JunctionScratchResult { vertices, triangles, sample_edges, topology })
}
