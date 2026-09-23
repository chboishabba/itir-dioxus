use serde::{Deserialize, Serialize};

use super::{au_legal_bearing_read_model, AuLegalBearingInput, UnifiedWorkbenchReadModel};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegalFollowGraphNode {
    pub id: String,
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegalFollowGraphEdge {
    pub source: String,
    pub target: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LegalFollowGraph {
    pub version: Option<String>,
    pub derived_only: Option<bool>,
    pub challengeable: Option<bool>,
    pub nodes: Vec<LegalFollowGraphNode>,
    pub edges: Vec<LegalFollowGraphEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuFactReviewProjectionInput {
    pub world_ref: String,
    pub source_refs: Vec<String>,
    pub event_refs: Vec<String>,
    pub handoff_refs: Vec<String>,
    pub research_residual_refs: Vec<String>,
    pub legal_follow_graph: LegalFollowGraph,
}

pub fn project_au_fact_review(input: AuFactReviewProjectionInput) -> UnifiedWorkbenchReadModel {
    let node_refs = input
        .legal_follow_graph
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect();

    let edge_refs = input
        .legal_follow_graph
        .edges
        .iter()
        .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
        .collect();

    au_legal_bearing_read_model(AuLegalBearingInput {
        world_ref: input.world_ref,
        journal_or_source_refs: input.source_refs,
        timeline_event_refs: input.event_refs,
        handoff_refs: input.handoff_refs,
        legal_node_refs: node_refs,
        legal_edge_refs: edge_refs,
        research_residual_refs: input.research_residual_refs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workbench::{StageAvailability, WorkbenchStageKind};

    #[test]
    fn persisted_legal_follow_graph_makes_proof_stage_available() {
        let model = project_au_fact_review(AuFactReviewProjectionInput {
            world_ref: "world:au:fixture".into(),
            source_refs: vec!["source:au:judgment".into()],
            event_refs: vec!["event:au:hearing".into()],
            handoff_refs: vec![],
            research_residual_refs: vec!["residual:authority-follow".into()],
            legal_follow_graph: LegalFollowGraph {
                version: Some("1".into()),
                derived_only: Some(true),
                challengeable: Some(true),
                nodes: vec![
                    LegalFollowGraphNode {
                        id: "node:authority:1".into(),
                        kind: "authority".into(),
                        label: "Authority".into(),
                    },
                    LegalFollowGraphNode {
                        id: "node:proposition:1".into(),
                        kind: "proposition".into(),
                        label: "Proposition".into(),
                    },
                ],
                edges: vec![LegalFollowGraphEdge {
                    source: "node:authority:1".into(),
                    target: "node:proposition:1".into(),
                    kind: "supports".into(),
                }],
            },
        });

        assert_eq!(
            model
                .stage(WorkbenchStageKind::MatterProof)
                .unwrap()
                .availability,
            StageAvailability::Available
        );
        assert!(!model.creates_semantic_authority);
        assert!(!model.creates_claim_truth);
        assert!(!model.pays_residual);
    }

    #[test]
    fn empty_graph_does_not_fabricate_proof_availability() {
        let model = project_au_fact_review(AuFactReviewProjectionInput {
            world_ref: "world:au:empty".into(),
            source_refs: vec!["source:au:1".into()],
            event_refs: vec![],
            handoff_refs: vec![],
            research_residual_refs: vec![],
            legal_follow_graph: LegalFollowGraph::default(),
        });

        assert!(matches!(
            model
                .stage(WorkbenchStageKind::MatterProof)
                .unwrap()
                .availability,
            StageAvailability::Unavailable { .. }
        ));
    }
}
