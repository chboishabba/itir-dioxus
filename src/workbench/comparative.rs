use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::visual::{
    command::{DomainCommand, VisualObjectId},
    comparative::{comparative_proof_topology, ComparativeProofTopology},
    ir::{GraphIr, VisualObjectInspection},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparativePanel {
    Before,
    After,
    Delta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparativeSelectors {
    pub left_ref: String,
    pub right_ref: String,
    pub query_ref: Option<String>,
    pub consumer_ref: Option<String>,
    pub as_at_ref: Option<String>,
    pub scope_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparativeWorkbenchReadModel {
    pub topology: ComparativeProofTopology,
    pub selectors: ComparativeSelectors,
    pub answer_changing_semantic_refs: BTreeSet<String>,
    pub unresolved_semantic_refs: BTreeSet<String>,
    pub candidate_only: bool,
    pub creates_semantic_authority: bool,
    pub creates_claim_truth: bool,
    pub predicts_outcome: bool,
}

impl ComparativeWorkbenchReadModel {
    pub fn select_command(
        &self,
        _panel: ComparativePanel,
        object_id: VisualObjectId,
    ) -> DomainCommand {
        // Deliberately reuse the existing command language. Because the topology
        // canonicalizes semantic object IDs, selecting the same semantic object
        // in any panel emits the same domain command.
        DomainCommand::SelectObject(object_id)
    }

    pub fn inspect(
        &self,
        panel: ComparativePanel,
        object_id: VisualObjectId,
    ) -> Option<VisualObjectInspection> {
        match panel {
            ComparativePanel::Before => self.topology.before.inspect_object(object_id),
            ComparativePanel::After => self.topology.after.inspect_object(object_id),
            ComparativePanel::Delta => self.topology.delta.inspect_object(object_id),
        }
    }
}

pub fn comparative_workbench_read_model(
    comparison_ref: impl Into<String>,
    left: &GraphIr,
    right: &GraphIr,
    selectors: ComparativeSelectors,
    answer_changing_semantic_refs: BTreeSet<String>,
    unresolved_semantic_refs: BTreeSet<String>,
) -> ComparativeWorkbenchReadModel {
    ComparativeWorkbenchReadModel {
        topology: comparative_proof_topology(comparison_ref, left, right),
        selectors,
        answer_changing_semantic_refs,
        unresolved_semantic_refs,
        candidate_only: true,
        creates_semantic_authority: false,
        creates_claim_truth: false,
        predicts_outcome: false,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreeWayComparativeSequence {
    pub w0_to_w1: ComparativeWorkbenchReadModel,
    pub w1_to_w2: ComparativeWorkbenchReadModel,
    pub candidate_only: bool,
    pub creates_semantic_authority: bool,
    pub creates_claim_truth: bool,
    pub predicts_outcome: bool,
}

pub fn three_way_comparative_sequence(
    comparison_ref: &str,
    w0: &GraphIr,
    w1: &GraphIr,
    w2: &GraphIr,
    selectors: ComparativeSelectors,
    w0_w1_answer_changing: BTreeSet<String>,
    w1_w2_answer_changing: BTreeSet<String>,
) -> ThreeWayComparativeSequence {
    ThreeWayComparativeSequence {
        w0_to_w1: comparative_workbench_read_model(
            format!("{comparison_ref}:w0-w1"),
            w0,
            w1,
            selectors.clone(),
            w0_w1_answer_changing,
            BTreeSet::new(),
        ),
        w1_to_w2: comparative_workbench_read_model(
            format!("{comparison_ref}:w1-w2"),
            w1,
            w2,
            selectors,
            w1_w2_answer_changing,
            BTreeSet::new(),
        ),
        candidate_only: true,
        creates_semantic_authority: false,
        creates_claim_truth: false,
        predicts_outcome: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visual::ir::VisualNode;

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

    fn graph(reference: &str, nodes: Vec<VisualNode>) -> GraphIr {
        GraphIr {
            graph_ref: reference.into(),
            derived_only: true,
            challengeable: true,
            nodes,
            edges: vec![],
        }
    }

    #[test]
    fn same_semantic_selection_uses_same_existing_domain_command_in_all_panels() {
        let left = graph(
            "graph:left",
            vec![
                node(71, "semantic:shared", "proposition"),
                node(72, "semantic:left-only", "defeater"),
            ],
        );
        let right = graph(
            "graph:right",
            vec![
                node(701, "semantic:shared", "proposition"),
                node(702, "semantic:right-only", "counter-defeater"),
            ],
        );
        let model = comparative_workbench_read_model(
            "comparison:test",
            &left,
            &right,
            ComparativeSelectors {
                left_ref: "left".into(),
                right_ref: "right".into(),
                query_ref: Some("query:test".into()),
                consumer_ref: Some("consumer:test".into()),
                as_at_ref: None,
                scope_ref: None,
            },
            BTreeSet::new(),
            BTreeSet::new(),
        );

        let before_id = model
            .topology
            .before
            .nodes
            .iter()
            .find(|node| node.semantic_ref == "semantic:shared")
            .unwrap()
            .id;
        let after_id = model
            .topology
            .after
            .nodes
            .iter()
            .find(|node| node.semantic_ref == "semantic:shared")
            .unwrap()
            .id;
        assert_eq!(before_id, after_id);
        assert_eq!(
            model.select_command(ComparativePanel::Before, before_id),
            model.select_command(ComparativePanel::After, after_id)
        );
    }

    #[test]
    fn three_way_pabai_shape_preserves_D_then_C_without_prediction() {
        let w0 = graph(
            "graph:pabai:w0",
            vec![node(1, "semantic:support", "support")],
        );
        let w1 = graph(
            "graph:pabai:w1",
            vec![
                node(10, "semantic:support", "support"),
                node(11, "semantic:D", "defeater"),
            ],
        );
        let w2 = graph(
            "graph:pabai:w2",
            vec![
                node(20, "semantic:support", "support"),
                node(21, "semantic:D", "defeater"),
                node(22, "semantic:C", "counter-defeater"),
            ],
        );
        let sequence = three_way_comparative_sequence(
            "comparison:pabai",
            &w0,
            &w1,
            &w2,
            ComparativeSelectors {
                left_ref: "w0".into(),
                right_ref: "w2".into(),
                query_ref: Some("query:pabai:duty-route".into()),
                consumer_ref: Some("consumer:pabai-climate-duty".into()),
                as_at_ref: None,
                scope_ref: None,
            },
            BTreeSet::from(["semantic:D".into()]),
            BTreeSet::from(["semantic:C".into()]),
        );

        assert!(sequence
            .w0_to_w1
            .answer_changing_semantic_refs
            .contains("semantic:D"));
        assert!(sequence
            .w1_to_w2
            .answer_changing_semantic_refs
            .contains("semantic:C"));
        assert!(!sequence.predicts_outcome);
        assert!(!sequence.creates_claim_truth);
    }
}
