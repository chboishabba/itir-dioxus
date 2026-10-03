#![cfg(feature = "production-data")]

use std::collections::BTreeSet;

use crate::visual::ir::{stable_visual_id, GraphIr, VisualEdge, VisualNode};

use super::InvestigationGraphProjection;

/// Deterministic visual geometry over an already-persisted graph.
///
/// This function creates no semantic edge. Every returned VisualEdge is a
/// projection of one persisted edge with the same semantic/source/provenance
/// coordinates. x/y are display geometry only.
pub fn investigation_graph_ir(
    persisted: &InvestigationGraphProjection,
) -> Result<GraphIr, String> {
    if !persisted.derived_only
        || !persisted.challengeable
        || persisted.creates_semantic_authority
        || persisted.creates_graph_edges
    {
        return Err("investigation graph crossed the projection-only boundary".into());
    }

    let node_refs = persisted
        .nodes
        .iter()
        .map(|node| node.semantic_ref.as_str())
        .collect::<BTreeSet<_>>();
    if persisted.edges.iter().any(|edge| {
        !node_refs.contains(edge.from_ref.as_str()) || !node_refs.contains(edge.to_ref.as_str())
    }) {
        return Err("persisted investigation graph contains a dangling visual edge".into());
    }

    let count = persisted.nodes.len().max(1);
    let nodes = persisted
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let columns = 4usize;
            let col = index % columns;
            let row = index / columns;
            let row_count = count.div_ceil(columns).max(1);
            let x = if columns == 1 {
                0.0
            } else {
                -0.78 + (col as f32 * (1.56 / (columns - 1) as f32))
            };
            let y = if row_count == 1 {
                0.0
            } else {
                0.72 - (row as f32 * (1.44 / (row_count - 1) as f32))
            };
            VisualNode {
                id: stable_visual_id(&node.semantic_ref),
                semantic_ref: node.semantic_ref.clone(),
                kind: node.kind.clone(),
                label: node.label.clone(),
                source_refs: node.source_refs.clone(),
                provenance_refs: node.provenance_refs.clone(),
                hidden: false,
                x,
                y,
            }
        })
        .collect();

    let edges = persisted
        .edges
        .iter()
        .map(|edge| VisualEdge {
            id: stable_visual_id(&edge.semantic_ref),
            from: stable_visual_id(&edge.from_ref),
            to: stable_visual_id(&edge.to_ref),
            semantic_ref: edge.semantic_ref.clone(),
            kind: edge.relation.clone(),
            source_refs: edge.source_refs.clone(),
            provenance_refs: edge.provenance_refs.clone(),
            hidden: false,
            weight: 1.0,
        })
        .collect();

    Ok(GraphIr {
        graph_ref: format!("inv-graph:{}", persisted.projection_ref),
        derived_only: true,
        challengeable: true,
        nodes,
        edges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sensiblaw_pg_source_store::InvestigationGraphBinding;
    use crate::workbench::investigation::{InvestigationGraphEdge, InvestigationGraphNode};

    fn projection() -> InvestigationGraphProjection {
        InvestigationGraphProjection {
            binding: InvestigationGraphBinding {
                schema: "itir.inv1.graph-binding.v1".into(),
                binding_ref: "binding:1".into(),
                obligation_ref: "acquisition:1".into(),
                projection_ref: "projection:1".into(),
                matter_ref: "matter:1".into(),
                binding_evidence_refs: vec!["receipt:binding".into()],
                derived_only: true,
                challengeable: true,
                creates_graph_edges: false,
                creates_semantic_authority: false,
                creates_access_authority: false,
                creates_acquisition_state: false,
            },
            projection_ref: "projection:1".into(),
            document_ref: "document:1".into(),
            nodes: vec![
                InvestigationGraphNode {
                    semantic_ref: "node:a".into(),
                    kind: "source".into(),
                    label: "A".into(),
                    source_refs: vec!["revision:a".into()],
                    provenance_refs: vec!["receipt:a".into()],
                },
                InvestigationGraphNode {
                    semantic_ref: "node:b".into(),
                    kind: "residual".into(),
                    label: "B".into(),
                    source_refs: vec!["revision:b".into()],
                    provenance_refs: vec!["receipt:b".into()],
                },
            ],
            edges: vec![InvestigationGraphEdge {
                semantic_ref: "edge:a-b".into(),
                from_ref: "node:a".into(),
                to_ref: "node:b".into(),
                relation: "supports-review".into(),
                source_refs: vec!["revision:a".into()],
                provenance_refs: vec!["receipt:edge".into()],
                challengeable: true,
            }],
            derived_only: true,
            challengeable: true,
            creates_semantic_authority: false,
            creates_graph_edges: false,
        }
    }

    #[test]
    fn graph_ir_preserves_only_persisted_nodes_and_edges() {
        let persisted = projection();
        let graph = investigation_graph_ir(&persisted).unwrap();
        assert_eq!(graph.nodes.len(), persisted.nodes.len());
        assert_eq!(graph.edges.len(), persisted.edges.len());
        assert_eq!(graph.edges[0].semantic_ref, "edge:a-b");
        assert!(graph.derived_only);
        assert!(graph.challengeable);
    }

    #[test]
    fn dangling_persisted_edge_is_not_visualized() {
        let mut persisted = projection();
        persisted.edges[0].to_ref = "node:missing".into();
        assert!(investigation_graph_ir(&persisted).is_err());
    }
}
