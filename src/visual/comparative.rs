//! M11 comparative visual projection.
//!
//! This is a projection over two already-existing GraphIr values. It does not
//! merge, rewrite, or adjudicate the underlying semantic worlds.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{
    command::VisualObjectId,
    ir::{GraphIr, VisualEdge, VisualNode},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparativeVisualClass {
    Shared,
    LeftOnly,
    RightOnly,
    Changed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparativeVisualObject {
    pub semantic_ref: String,
    pub class: ComparativeVisualClass,
    pub left_object_id: Option<VisualObjectId>,
    pub right_object_id: Option<VisualObjectId>,
    pub kind: String,
    pub source_refs: Vec<String>,
    pub provenance_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparativeGraphIr {
    pub comparison_ref: String,
    pub left_graph_ref: String,
    pub right_graph_ref: String,
    pub objects: Vec<ComparativeVisualObject>,
    pub shared_semantic_refs: BTreeSet<String>,
    pub left_only_semantic_refs: BTreeSet<String>,
    pub right_only_semantic_refs: BTreeSet<String>,
    pub changed_semantic_refs: BTreeSet<String>,
    pub derived_only: bool,
    pub challengeable: bool,
    pub creates_semantic_authority: bool,
    pub creates_claim_truth: bool,
}

fn node_map(graph: &GraphIr) -> BTreeMap<String, &VisualNode> {
    graph
        .nodes
        .iter()
        .map(|node| (node.semantic_ref.clone(), node))
        .collect()
}

fn edge_map(graph: &GraphIr) -> BTreeMap<String, &VisualEdge> {
    graph
        .edges
        .iter()
        .map(|edge| (edge.semantic_ref.clone(), edge))
        .collect()
}

fn merged_refs(left: &[String], right: &[String]) -> Vec<String> {
    left.iter()
        .chain(right.iter())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn compare_graph_ir(
    comparison_ref: impl Into<String>,
    left: &GraphIr,
    right: &GraphIr,
) -> ComparativeGraphIr {
    let left_nodes = node_map(left);
    let right_nodes = node_map(right);
    let left_edges = edge_map(left);
    let right_edges = edge_map(right);

    let mut semantic_refs = BTreeSet::new();
    semantic_refs.extend(left_nodes.keys().cloned());
    semantic_refs.extend(right_nodes.keys().cloned());
    semantic_refs.extend(left_edges.keys().cloned());
    semantic_refs.extend(right_edges.keys().cloned());

    let mut objects = Vec::new();
    let mut shared_semantic_refs = BTreeSet::new();
    let mut left_only_semantic_refs = BTreeSet::new();
    let mut right_only_semantic_refs = BTreeSet::new();
    let mut changed_semantic_refs = BTreeSet::new();

    for semantic_ref in semantic_refs {
        let left_node = left_nodes.get(&semantic_ref).copied();
        let right_node = right_nodes.get(&semantic_ref).copied();
        let left_edge = left_edges.get(&semantic_ref).copied();
        let right_edge = right_edges.get(&semantic_ref).copied();

        let (class, left_object_id, right_object_id, kind, source_refs, provenance_refs) =
            match (left_node, right_node, left_edge, right_edge) {
                (Some(left_node), Some(right_node), _, _) => {
                    let changed = left_node.kind != right_node.kind
                        || left_node.label != right_node.label
                        || left_node.source_refs != right_node.source_refs
                        || left_node.provenance_refs != right_node.provenance_refs
                        || left_node.hidden != right_node.hidden;
                    (
                        if changed {
                            changed_semantic_refs.insert(semantic_ref.clone());
                            ComparativeVisualClass::Changed
                        } else {
                            shared_semantic_refs.insert(semantic_ref.clone());
                            ComparativeVisualClass::Shared
                        },
                        Some(left_node.id),
                        Some(right_node.id),
                        right_node.kind.clone(),
                        merged_refs(&left_node.source_refs, &right_node.source_refs),
                        merged_refs(
                            &left_node.provenance_refs,
                            &right_node.provenance_refs,
                        ),
                    )
                }
                (Some(left_node), None, _, _) => {
                    left_only_semantic_refs.insert(semantic_ref.clone());
                    (
                        ComparativeVisualClass::LeftOnly,
                        Some(left_node.id),
                        None,
                        left_node.kind.clone(),
                        left_node.source_refs.clone(),
                        left_node.provenance_refs.clone(),
                    )
                }
                (None, Some(right_node), _, _) => {
                    right_only_semantic_refs.insert(semantic_ref.clone());
                    (
                        ComparativeVisualClass::RightOnly,
                        None,
                        Some(right_node.id),
                        right_node.kind.clone(),
                        right_node.source_refs.clone(),
                        right_node.provenance_refs.clone(),
                    )
                }
                (_, _, Some(left_edge), Some(right_edge)) => {
                    let changed = left_edge.from != right_edge.from
                        || left_edge.to != right_edge.to
                        || left_edge.kind != right_edge.kind
                        || left_edge.source_refs != right_edge.source_refs
                        || left_edge.provenance_refs != right_edge.provenance_refs
                        || left_edge.hidden != right_edge.hidden
                        || left_edge.weight != right_edge.weight;
                    (
                        if changed {
                            changed_semantic_refs.insert(semantic_ref.clone());
                            ComparativeVisualClass::Changed
                        } else {
                            shared_semantic_refs.insert(semantic_ref.clone());
                            ComparativeVisualClass::Shared
                        },
                        Some(left_edge.id),
                        Some(right_edge.id),
                        right_edge.kind.clone(),
                        merged_refs(&left_edge.source_refs, &right_edge.source_refs),
                        merged_refs(
                            &left_edge.provenance_refs,
                            &right_edge.provenance_refs,
                        ),
                    )
                }
                (_, _, Some(left_edge), None) => {
                    left_only_semantic_refs.insert(semantic_ref.clone());
                    (
                        ComparativeVisualClass::LeftOnly,
                        Some(left_edge.id),
                        None,
                        left_edge.kind.clone(),
                        left_edge.source_refs.clone(),
                        left_edge.provenance_refs.clone(),
                    )
                }
                (_, _, None, Some(right_edge)) => {
                    right_only_semantic_refs.insert(semantic_ref.clone());
                    (
                        ComparativeVisualClass::RightOnly,
                        None,
                        Some(right_edge.id),
                        right_edge.kind.clone(),
                        right_edge.source_refs.clone(),
                        right_edge.provenance_refs.clone(),
                    )
                }
                _ => continue,
            };

        objects.push(ComparativeVisualObject {
            semantic_ref,
            class,
            left_object_id,
            right_object_id,
            kind,
            source_refs,
            provenance_refs,
        });
    }

    ComparativeGraphIr {
        comparison_ref: comparison_ref.into(),
        left_graph_ref: left.graph_ref.clone(),
        right_graph_ref: right.graph_ref.clone(),
        objects,
        shared_semantic_refs,
        left_only_semantic_refs,
        right_only_semantic_refs,
        changed_semantic_refs,
        derived_only: true,
        challengeable: left.challengeable || right.challengeable,
        creates_semantic_authority: false,
        creates_claim_truth: false,
    }
}


fn canonicalize_graph_pair(left: &GraphIr, right: &GraphIr) -> (GraphIr, GraphIr) {
    let node_semantics = left
        .nodes
        .iter()
        .chain(right.nodes.iter())
        .map(|node| node.semantic_ref.clone())
        .collect::<BTreeSet<_>>();
    let edge_semantics = left
        .edges
        .iter()
        .chain(right.edges.iter())
        .map(|edge| edge.semantic_ref.clone())
        .collect::<BTreeSet<_>>();

    let node_ids = node_semantics
        .into_iter()
        .enumerate()
        .map(|(index, semantic_ref)| (semantic_ref, VisualObjectId(index as u64 + 1)))
        .collect::<BTreeMap<_, _>>();
    let edge_base = node_ids.len() as u64 + 1;
    let edge_ids = edge_semantics
        .into_iter()
        .enumerate()
        .map(|(index, semantic_ref)| {
            (semantic_ref, VisualObjectId(edge_base + index as u64))
        })
        .collect::<BTreeMap<_, _>>();

    let reindex = |graph: &GraphIr| {
        let old_to_new = graph
            .nodes
            .iter()
            .map(|node| (node.id, node_ids[&node.semantic_ref]))
            .collect::<BTreeMap<_, _>>();
        let nodes = graph
            .nodes
            .iter()
            .cloned()
            .map(|mut node| {
                node.id = node_ids[&node.semantic_ref];
                node
            })
            .collect::<Vec<_>>();
        let edges = graph
            .edges
            .iter()
            .cloned()
            .map(|mut edge| {
                edge.id = edge_ids[&edge.semantic_ref];
                edge.from = old_to_new[&edge.from];
                edge.to = old_to_new[&edge.to];
                edge
            })
            .collect::<Vec<_>>();
        GraphIr {
            graph_ref: graph.graph_ref.clone(),
            derived_only: graph.derived_only,
            challengeable: graph.challengeable,
            nodes,
            edges,
        }
    };

    (reindex(left), reindex(right))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparativeProofTopology {
    pub comparison_ref: String,
    pub before: GraphIr,
    pub after: GraphIr,
    pub delta: GraphIr,
    pub comparative: ComparativeGraphIr,
    pub creates_semantic_authority: bool,
    pub creates_claim_truth: bool,
}

fn class_kind(class: ComparativeVisualClass, semantic_kind: &str) -> String {
    let class_ref = match class {
        ComparativeVisualClass::Shared => "shared",
        ComparativeVisualClass::LeftOnly => "left-only",
        ComparativeVisualClass::RightOnly => "right-only",
        ComparativeVisualClass::Changed => "changed",
    };
    format!("comparative:{class_ref}:{semantic_kind}")
}

pub fn comparative_proof_topology(
    comparison_ref: impl Into<String>,
    left: &GraphIr,
    right: &GraphIr,
) -> ComparativeProofTopology {
    let comparison_ref = comparison_ref.into();
    let (left, right) = canonicalize_graph_pair(left, right);
    let comparative = compare_graph_ir(comparison_ref.clone(), &left, &right);

    let mut left_node_by_semantic = BTreeMap::new();
    left_node_by_semantic.extend(
        left.nodes
            .iter()
            .map(|node| (node.semantic_ref.clone(), node.clone())),
    );
    let mut right_node_by_semantic = BTreeMap::new();
    right_node_by_semantic.extend(
        right
            .nodes
            .iter()
            .map(|node| (node.semantic_ref.clone(), node.clone())),
    );

    let mut delta_nodes = Vec::new();
    let mut delta_id_by_semantic = BTreeMap::new();
    for object in &comparative.objects {
        if object.class == ComparativeVisualClass::Shared {
            continue;
        }
        let source_node = right_node_by_semantic
            .get(&object.semantic_ref)
            .or_else(|| left_node_by_semantic.get(&object.semantic_ref));
        let Some(source_node) = source_node else {
            // Edge-only comparison objects are represented when both endpoint
            // nodes survive into the delta projection below.
            continue;
        };
        let mut node = source_node.clone();
        node.kind = class_kind(object.class, &source_node.kind);
        node.source_refs = object.source_refs.clone();
        node.provenance_refs = object.provenance_refs.clone();
        delta_id_by_semantic.insert(object.semantic_ref.clone(), node.id);
        delta_nodes.push(node);
    }

    let left_edge_by_semantic = left
        .edges
        .iter()
        .map(|edge| (edge.semantic_ref.clone(), edge))
        .collect::<BTreeMap<_, _>>();
    let right_edge_by_semantic = right
        .edges
        .iter()
        .map(|edge| (edge.semantic_ref.clone(), edge))
        .collect::<BTreeMap<_, _>>();

    let mut delta_edges = Vec::new();
    for object in &comparative.objects {
        if object.class == ComparativeVisualClass::Shared {
            continue;
        }
        let (source_edge, source_graph) =
            if let Some(edge) = right_edge_by_semantic.get(&object.semantic_ref).copied() {
                (edge, &right)
            } else if let Some(edge) = left_edge_by_semantic.get(&object.semantic_ref).copied() {
                (edge, &left)
            } else {
                continue;
            };

        let source_from_semantic = source_graph
            .node(source_edge.from)
            .map(|node| node.semantic_ref.as_str());
        let source_to_semantic = source_graph
            .node(source_edge.to)
            .map(|node| node.semantic_ref.as_str());

        let (Some(from_semantic), Some(to_semantic)) =
            (source_from_semantic, source_to_semantic)
        else {
            continue;
        };

        for semantic_ref in [from_semantic, to_semantic] {
            if delta_id_by_semantic.contains_key(semantic_ref) {
                continue;
            }
            let context_node = right_node_by_semantic
                .get(semantic_ref)
                .or_else(|| left_node_by_semantic.get(semantic_ref));
            if let Some(context_node) = context_node {
                let mut node = (*context_node).clone();
                node.kind = class_kind(ComparativeVisualClass::Shared, &context_node.kind);
                delta_id_by_semantic.insert(semantic_ref.to_owned(), node.id);
                delta_nodes.push(node);
            }
        }

        let (Some(from), Some(to)) = (
            delta_id_by_semantic.get(from_semantic).copied(),
            delta_id_by_semantic.get(to_semantic).copied(),
        ) else {
            continue;
        };

        let mut edge = source_edge.clone();
        edge.from = from;
        edge.to = to;
        edge.kind = class_kind(object.class, &source_edge.kind);
        edge.source_refs = object.source_refs.clone();
        edge.provenance_refs = object.provenance_refs.clone();
        delta_edges.push(edge);
    }

    let delta = GraphIr {
        graph_ref: format!("{}:delta", comparison_ref),
        derived_only: true,
        challengeable: left.challengeable || right.challengeable,
        nodes: delta_nodes,
        edges: delta_edges,
    };

    ComparativeProofTopology {
        comparison_ref,
        before: left,
        after: right,
        delta,
        comparative,
        creates_semantic_authority: false,
        creates_claim_truth: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: u64, semantic_ref: &str, kind: &str) -> VisualNode {
        VisualNode {
            id: VisualObjectId(id),
            semantic_ref: semantic_ref.into(),
            kind: kind.into(),
            label: semantic_ref.into(),
            source_refs: vec![format!("source:{semantic_ref}")],
            provenance_refs: vec![format!("receipt:{semantic_ref}")],
            hidden: false,
            x: 0.0,
            y: 0.0,
        }
    }

    #[test]
    fn shared_semantic_objects_use_one_canonical_id_space_across_panels() {
        let left = GraphIr {
            graph_ref: "graph:left-id-space".into(),
            derived_only: true,
            challengeable: true,
            nodes: vec![node(90, "semantic:shared", "proposition")],
            edges: vec![],
        };
        let right = GraphIr {
            graph_ref: "graph:right-id-space".into(),
            derived_only: true,
            challengeable: true,
            nodes: vec![node(900, "semantic:shared", "proposition")],
            edges: vec![],
        };

        let topology = comparative_proof_topology("comparison:id-space", &left, &right);
        let before_id = topology
            .before
            .nodes
            .iter()
            .find(|node| node.semantic_ref == "semantic:shared")
            .unwrap()
            .id;
        let after_id = topology
            .after
            .nodes
            .iter()
            .find(|node| node.semantic_ref == "semantic:shared")
            .unwrap()
            .id;

        assert_eq!(before_id, after_id);
        assert_eq!(
            topology.comparative.objects[0].left_object_id,
            topology.comparative.objects[0].right_object_id
        );
    }

    #[test]
    fn changed_edge_keeps_shared_endpoint_context_in_delta_panel() {
        let left_a = node(1, "semantic:a", "proposition");
        let left_b = node(2, "semantic:b", "proposition");
        let right_a = node(10, "semantic:a", "proposition");
        let right_b = node(20, "semantic:b", "proposition");

        let left = GraphIr {
            graph_ref: "graph:left-edge".into(),
            derived_only: true,
            challengeable: true,
            nodes: vec![left_a.clone(), left_b.clone()],
            edges: vec![VisualEdge {
                id: VisualObjectId(3),
                from: left_a.id,
                to: left_b.id,
                semantic_ref: "edge:a-b".into(),
                kind: "supports".into(),
                source_refs: vec![],
                provenance_refs: vec!["receipt:old".into()],
                hidden: false,
                weight: 1.0,
            }],
        };
        let right = GraphIr {
            graph_ref: "graph:right-edge".into(),
            derived_only: true,
            challengeable: true,
            nodes: vec![right_a.clone(), right_b.clone()],
            edges: vec![VisualEdge {
                id: VisualObjectId(30),
                from: right_a.id,
                to: right_b.id,
                semantic_ref: "edge:a-b".into(),
                kind: "supports".into(),
                source_refs: vec![],
                provenance_refs: vec!["receipt:new".into()],
                hidden: false,
                weight: 1.0,
            }],
        };

        let topology = comparative_proof_topology("comparison:edge", &left, &right);

        assert_eq!(topology.delta.edges.len(), 1);
        assert_eq!(topology.delta.nodes.len(), 2);
        assert!(topology
            .delta
            .nodes
            .iter()
            .all(|node| node.kind.starts_with("comparative:shared:")));
        assert!(topology.delta.edges[0].kind.starts_with("comparative:changed:"));
    }

    #[test]
    fn proof_topology_exposes_before_after_and_delta_without_rewriting_inputs() {
        let left = GraphIr {
            graph_ref: "graph:left".into(),
            derived_only: true,
            challengeable: true,
            nodes: vec![
                node(1, "semantic:shared", "proposition"),
                node(2, "semantic:defeater", "defeater"),
            ],
            edges: vec![],
        };
        let right = GraphIr {
            graph_ref: "graph:right".into(),
            derived_only: true,
            challengeable: true,
            nodes: vec![
                node(10, "semantic:shared", "proposition"),
                node(20, "semantic:counter", "counter-defeater"),
            ],
            edges: vec![],
        };

        let left_input = left.clone();
        let right_input = right.clone();
        let topology = comparative_proof_topology("comparison:pabai", &left, &right);

        assert_eq!(left, left_input);
        assert_eq!(right, right_input);
        let before_shared = topology
            .before
            .nodes
            .iter()
            .find(|node| node.semantic_ref == "semantic:shared")
            .unwrap();
        let after_shared = topology
            .after
            .nodes
            .iter()
            .find(|node| node.semantic_ref == "semantic:shared")
            .unwrap();
        assert_eq!(before_shared.id, after_shared.id);
        assert_eq!(topology.delta.nodes.len(), 2);
        assert!(topology
            .delta
            .nodes
            .iter()
            .all(|node| node.kind.starts_with("comparative:")));
        assert!(!topology.creates_semantic_authority);
        assert!(!topology.creates_claim_truth);
    }

    #[test]
    fn comparative_ir_separates_shared_left_right_and_changed_without_merging_worlds() {
        let left = GraphIr {
            graph_ref: "graph:left".into(),
            derived_only: true,
            challengeable: true,
            nodes: vec![
                node(1, "semantic:shared", "proposition"),
                node(2, "semantic:left", "defeater"),
                node(3, "semantic:changed", "authority"),
            ],
            edges: vec![],
        };
        let mut changed_right = node(30, "semantic:changed", "authority");
        changed_right.provenance_refs = vec!["receipt:new".into()];
        let right = GraphIr {
            graph_ref: "graph:right".into(),
            derived_only: true,
            challengeable: true,
            nodes: vec![
                node(10, "semantic:shared", "proposition"),
                node(20, "semantic:right", "counter-defeater"),
                changed_right,
            ],
            edges: vec![],
        };

        let comparison = compare_graph_ir("comparison:test", &left, &right);

        assert!(comparison.shared_semantic_refs.contains("semantic:shared"));
        assert!(comparison.left_only_semantic_refs.contains("semantic:left"));
        assert!(comparison.right_only_semantic_refs.contains("semantic:right"));
        assert!(comparison.changed_semantic_refs.contains("semantic:changed"));
        assert_eq!(comparison.left_graph_ref, "graph:left");
        assert_eq!(comparison.right_graph_ref, "graph:right");
        assert!(!comparison.creates_claim_truth);
    }
}
