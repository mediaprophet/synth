//! Staged, bounded adapter for exactly one adopted three-arm junction.
//!
//! This is a source-graph check and field input adapter, not a surface,
//! clearance, provenance, UV or biological qualification. The returned field
//! segments have deterministic source order. Never pass a colonization proposal
//! suffix here; the caller must supply exactly the accepted adopted graph.
use super::botanical_graph_host::BotanicalNode;
use qualia_core_db::specialized_libs::computational_geometry as cg;

/// Source endpoints remain identifiers, distinct from extractor sample-edge IDs.
#[derive(Debug)]
pub struct JunctionEdge<'a> {
    pub from_key: &'a str,
    pub to_key: &'a str,
    pub field: cg::tapered_branch_field::TaperedFieldSegment,
}

/// A host-selected angular screen. It rejects arms pointing in nearly the same
/// direction; opposite directions are permitted. This is not a clearance proof.
/// Nodes must be root, center, and two children of center in source order.
/// Output is center-to-root, center-to-first-child, center-to-second-child.
pub fn adopted_single_junction<'a>(
    nodes: &'a [BotanicalNode],
    min_axis_m: f64,
    max_same_direction_cosine: f64,
) -> Result<[JunctionEdge<'a>; 3], &'static str> {
    if nodes.len() != 4 || !min_axis_m.is_finite() || min_axis_m <= 0.0
        || !max_same_direction_cosine.is_finite()
        || !(0.0..1.0).contains(&max_same_direction_cosine) {
        return Err("Invalid junction graph or host-selected thresholds");
    }
    for (i, node) in nodes.iter().enumerate() {
        if node.key.is_empty() || nodes[..i].iter().any(|prior| prior.key == node.key)
            || !node.radius_m.is_finite() || node.radius_m <= 0.0
            || node.position_m.iter().any(|v| !v.is_finite() || v.abs() > 1.0e6) {
            return Err("Invalid adopted junction node");
        }
    }
    if nodes[0].parent_key.is_some()
        || nodes[1].parent_key.as_deref() != Some(nodes[0].key.as_str())
        || nodes[2].parent_key.as_deref() != Some(nodes[1].key.as_str())
        || nodes[3].parent_key.as_deref() != Some(nodes[1].key.as_str()) {
        return Err("Expected one adopted junction with three incident edges");
    }
    let center = &nodes[1];
    let others = [&nodes[0], &nodes[2], &nodes[3]];
    let mut unit = [[0.0; 3]; 3];
    for (i, other) in others.iter().enumerate() {
        let d = [other.position_m[0] - center.position_m[0],
                 other.position_m[1] - center.position_m[1],
                 other.position_m[2] - center.position_m[2]];
        let length = d[0].hypot(d[1]).hypot(d[2]);
        if !length.is_finite() || length < min_axis_m || length < f64::MIN_POSITIVE {
            return Err("Degenerate junction axis");
        }
        unit[i] = [d[0] / length, d[1] / length, d[2] / length];
    }
    for i in 0..3 {
        for j in (i + 1)..3 {
            let dot = unit[i][0] * unit[j][0] + unit[i][1] * unit[j][1] + unit[i][2] * unit[j][2];
            if !dot.is_finite() || dot >= max_same_direction_cosine {
                return Err("Junction arms too nearly coincident");
            }
        }
    }
    let edge = |other: &'a BotanicalNode| JunctionEdge {
        from_key: center.key.as_str(),
        to_key: other.key.as_str(),
        field: cg::tapered_branch_field::TaperedFieldSegment {
            a: cg::Point3::new(center.position_m[0], center.position_m[1], center.position_m[2]),
            b: cg::Point3::new(other.position_m[0], other.position_m[1], other.position_m[2]),
            radius_a: center.radius_m,
            radius_b: other.radius_m,
        },
    };
    Ok([edge(others[0]), edge(others[1]), edge(others[2])])
}
#[cfg(test)]
mod tests {
    use super::*;
    fn node(key: &str, parent: Option<&str>, p: [f64; 3]) -> BotanicalNode {
        BotanicalNode { key: key.into(), parent_key: parent.map(str::to_owned), position_m: p, radius_m: 0.1 }
    }
    fn graph() -> Vec<BotanicalNode> {
        vec![node("root", None, [-1.0, 0.0, 0.0]), node("center", Some("root"), [0.0, 0.0, 0.0]),
            node("a", Some("center"), [1.0, 0.0, 0.0]), node("b", Some("center"), [0.0, 1.0, 0.0])]
    }
    #[test]
    fn source_order_and_opposite_trunk_are_retained() {
        let nodes = graph();
        let edges = adopted_single_junction(&nodes, 1e-6, 0.99).unwrap();
        assert_eq!((edges[0].from_key, edges[0].to_key), ("center", "root"));
        assert_eq!((edges[1].from_key, edges[1].to_key), ("center", "a"));
        assert_eq!((edges[2].from_key, edges[2].to_key), ("center", "b"));
        assert_eq!(edges[0].field.radius_a, nodes[1].radius_m);
    }
    #[test]
    fn refuses_extra_proposal_or_unrelated_edge() {
        let mut nodes = graph();
        nodes.push(node("proposal", Some("a"), [2.0, 0.0, 0.0]));
        assert!(adopted_single_junction(&nodes, 1e-6, 0.99).is_err());
        nodes.pop(); nodes[3].parent_key = Some("a".into());
        assert!(adopted_single_junction(&nodes, 1e-6, 0.99).is_err());
    }
    #[test]
    fn refuses_duplicate_identity_degenerate_and_same_direction() {
        let mut nodes = graph(); nodes[3].key = "a".into();
        assert!(adopted_single_junction(&nodes, 1e-6, 0.99).is_err());
        nodes = graph(); nodes[3].position_m = nodes[1].position_m;
        assert!(adopted_single_junction(&nodes, 1e-6, 0.99).is_err());
        nodes = graph(); nodes[3].position_m = [2.0, 0.0, 0.0];
        assert!(adopted_single_junction(&nodes, 1e-6, 0.99).is_err());
    }
}
