use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use super::{
    au_fact_review::{project_persisted_au_workbench_json, AuFactReviewProjection},
    comparative::{
        comparative_workbench_read_model, three_way_comparative_sequence_with_overlays,
        ComparativeExplanationOverlay, ComparativeSelectors,
        ComparativeWorkbenchReadModel, ThreeWayComparativeSequence,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PersistedComparativeOverlay {
    #[serde(default)]
    pub change_layer_by_semantic_ref: BTreeMap<String, String>,
    #[serde(default)]
    pub explanation_by_semantic_ref: BTreeMap<String, String>,
    #[serde(default)]
    pub answer_changing_semantic_refs: BTreeSet<String>,
    #[serde(default)]
    pub unresolved_semantic_refs: BTreeSet<String>,
    #[serde(default)]
    pub creates_semantic_authority: bool,
    #[serde(default)]
    pub creates_claim_truth: bool,
}

impl PersistedComparativeOverlay {
    pub fn validate(&self) -> Result<(), String> {
        if self.creates_semantic_authority || self.creates_claim_truth {
            return Err("persisted comparative overlay crossed non-promotion boundary".into());
        }
        Ok(())
    }

    pub fn into_read_model_overlay(self) -> Result<ComparativeExplanationOverlay, String> {
        self.validate()?;
        Ok(ComparativeExplanationOverlay {
            change_layer_by_semantic_ref: self.change_layer_by_semantic_ref,
            explanation_by_semantic_ref: self.explanation_by_semantic_ref,
            answer_changing_semantic_refs: self.answer_changing_semantic_refs,
            unresolved_semantic_refs: self.unresolved_semantic_refs,
            creates_semantic_authority: false,
            creates_claim_truth: false,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedComparativeWorkbenchSpecimen {
    pub left: AuFactReviewProjection,
    pub right: AuFactReviewProjection,
    pub read_model: ComparativeWorkbenchReadModel,
    pub source_backed_left: bool,
    pub source_backed_right: bool,
    pub provenance_backed_left: bool,
    pub provenance_backed_right: bool,
    pub candidate_only: bool,
    pub creates_semantic_authority: bool,
    pub creates_claim_truth: bool,
    pub predicts_outcome: bool,
}

fn graph_has_source_refs(projection: &AuFactReviewProjection) -> bool {
    projection
        .graph_ir
        .nodes
        .iter()
        .any(|node| !node.source_refs.is_empty())
        || projection
            .graph_ir
            .edges
            .iter()
            .any(|edge| !edge.source_refs.is_empty())
}

fn graph_has_provenance_refs(projection: &AuFactReviewProjection) -> bool {
    projection
        .graph_ir
        .nodes
        .iter()
        .any(|node| !node.provenance_refs.is_empty())
        || projection
            .graph_ir
            .edges
            .iter()
            .any(|edge| !edge.provenance_refs.is_empty())
}


fn graph_semantic_refs(projection: &AuFactReviewProjection) -> BTreeSet<String> {
    projection
        .graph_ir
        .nodes
        .iter()
        .map(|node| node.semantic_ref.clone())
        .chain(
            projection
                .graph_ir
                .edges
                .iter()
                .map(|edge| edge.semantic_ref.clone()),
        )
        .collect()
}

fn semantic_has_source_and_provenance(
    projection: &AuFactReviewProjection,
    semantic_ref: &str,
) -> bool {
    projection
        .graph_ir
        .nodes
        .iter()
        .find(|node| node.semantic_ref == semantic_ref)
        .is_some_and(|node| !node.source_refs.is_empty() && !node.provenance_refs.is_empty())
        || projection
            .graph_ir
            .edges
            .iter()
            .find(|edge| edge.semantic_ref == semantic_ref)
            .is_some_and(|edge| !edge.source_refs.is_empty() && !edge.provenance_refs.is_empty())
}

fn validate_overlay_against_graphs(
    overlay: &ComparativeExplanationOverlay,
    left: &AuFactReviewProjection,
    right: &AuFactReviewProjection,
) -> Result<(), String> {
    let left_refs = graph_semantic_refs(left);
    let right_refs = graph_semantic_refs(right);
    let available = left_refs
        .union(&right_refs)
        .cloned()
        .collect::<BTreeSet<_>>();

    let referenced = overlay
        .change_layer_by_semantic_ref
        .keys()
        .chain(overlay.explanation_by_semantic_ref.keys())
        .chain(overlay.answer_changing_semantic_refs.iter())
        .chain(overlay.unresolved_semantic_refs.iter())
        .cloned()
        .collect::<BTreeSet<_>>();

    let missing = referenced
        .difference(&available)
        .cloned()
        .collect::<BTreeSet<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "comparative overlay references semantic objects absent from persisted graphs: {missing:?}"
        ));
    }

    for semantic_ref in &overlay.answer_changing_semantic_refs {
        if !semantic_has_source_and_provenance(left, semantic_ref)
            && !semantic_has_source_and_provenance(right, semantic_ref)
        {
            return Err(format!(
                "answer-changing semantic object lacks source/provenance closure: {semantic_ref}"
            ));
        }
    }
    Ok(())
}

pub fn project_persisted_comparative_workbench_json(
    comparison_ref: &str,
    left_raw: &str,
    right_raw: &str,
    overlay_raw: Option<&str>,
    max_nodes: usize,
    max_edges: usize,
) -> Result<PersistedComparativeWorkbenchSpecimen, String> {
    if comparison_ref.trim().is_empty() {
        return Err("persisted comparative workbench requires comparison_ref".into());
    }

    let left = project_persisted_au_workbench_json(left_raw, max_nodes, max_edges)?;
    let right = project_persisted_au_workbench_json(right_raw, max_nodes, max_edges)?;

    if left.graph_ir.nodes.is_empty() || right.graph_ir.nodes.is_empty() {
        return Err("persisted comparative specimen requires non-empty legal-follow graphs".into());
    }

    let overlay = match overlay_raw {
        Some(raw) => serde_json::from_str::<PersistedComparativeOverlay>(raw)
            .map_err(|error| format!("invalid comparative overlay JSON: {error}"))?
            .into_read_model_overlay()?,
        None => ComparativeExplanationOverlay::empty(),
    };

    validate_overlay_against_graphs(&overlay, &left, &right)?;

    let selectors = ComparativeSelectors {
        left_ref: left.read_model.world_ref.clone(),
        right_ref: right.read_model.world_ref.clone(),
        query_ref: None,
        consumer_ref: None,
        as_at_ref: None,
        scope_ref: None,
    };
    let read_model = comparative_workbench_read_model(
        comparison_ref,
        &left.graph_ir,
        &right.graph_ir,
        selectors,
        overlay,
    );

    Ok(PersistedComparativeWorkbenchSpecimen {
        source_backed_left: graph_has_source_refs(&left),
        source_backed_right: graph_has_source_refs(&right),
        provenance_backed_left: graph_has_provenance_refs(&left),
        provenance_backed_right: graph_has_provenance_refs(&right),
        left,
        right,
        read_model,
        candidate_only: true,
        creates_semantic_authority: false,
        creates_claim_truth: false,
        predicts_outcome: false,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedThreeWayComparativeSpecimen {
    pub w0: AuFactReviewProjection,
    pub w1: AuFactReviewProjection,
    pub w2: AuFactReviewProjection,
    pub sequence: ThreeWayComparativeSequence,
    pub candidate_only: bool,
    pub creates_semantic_authority: bool,
    pub creates_claim_truth: bool,
    pub predicts_outcome: bool,
}

pub fn project_persisted_three_way_comparative_json(
    comparison_ref: &str,
    w0_raw: &str,
    w1_raw: &str,
    w2_raw: &str,
    w0_w1_overlay_raw: Option<&str>,
    w1_w2_overlay_raw: Option<&str>,
    max_nodes: usize,
    max_edges: usize,
) -> Result<PersistedThreeWayComparativeSpecimen, String> {
    let w0 = project_persisted_au_workbench_json(w0_raw, max_nodes, max_edges)?;
    let w1 = project_persisted_au_workbench_json(w1_raw, max_nodes, max_edges)?;
    let w2 = project_persisted_au_workbench_json(w2_raw, max_nodes, max_edges)?;

    if [w0.graph_ir.nodes.len(), w1.graph_ir.nodes.len(), w2.graph_ir.nodes.len()]
        .into_iter()
        .any(|count| count == 0)
    {
        return Err("three-way comparative specimen requires three non-empty persisted legal-follow graphs".into());
    }

    let parse_overlay = |raw: Option<&str>| -> Result<PersistedComparativeOverlay, String> {
        match raw {
            Some(raw) => {
                let overlay = serde_json::from_str::<PersistedComparativeOverlay>(raw)
                    .map_err(|error| format!("invalid comparative overlay JSON: {error}"))?;
                overlay.validate()?;
                Ok(overlay)
            }
            None => Ok(PersistedComparativeOverlay::default()),
        }
    };
    let w0_w1 = parse_overlay(w0_w1_overlay_raw)?;
    let w1_w2 = parse_overlay(w1_w2_overlay_raw)?;
    let w0_w1_read_overlay = w0_w1.clone().into_read_model_overlay()?;
    let w1_w2_read_overlay = w1_w2.clone().into_read_model_overlay()?;
    validate_overlay_against_graphs(&w0_w1_read_overlay, &w0, &w1)?;
    validate_overlay_against_graphs(&w1_w2_read_overlay, &w1, &w2)?;

    let sequence = three_way_comparative_sequence_with_overlays(
        comparison_ref,
        &w0.graph_ir,
        &w1.graph_ir,
        &w2.graph_ir,
        ComparativeSelectors {
            left_ref: w0.read_model.world_ref.clone(),
            right_ref: w2.read_model.world_ref.clone(),
            query_ref: None,
            consumer_ref: None,
            as_at_ref: None,
            scope_ref: None,
        },
        w0_w1_read_overlay,
        w1_w2_read_overlay,
    );

    Ok(PersistedThreeWayComparativeSpecimen {
        w0,
        w1,
        w2,
        sequence,
        candidate_only: true,
        creates_semantic_authority: false,
        creates_claim_truth: false,
        predicts_outcome: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn persisted(world: &str, semantic: &str, receipt: &str) -> String {
        json!({
            "workbench": {
                "run": { "fact_run_id": world },
                "sources": [{ "source_id": format!("source:{world}") }],
                "semantic_context": {
                    "legal_follow_graph": {
                        "version": format!("graph:{world}"),
                        "derived_only": true,
                        "challengeable": true,
                        "nodes": [{
                            "id": semantic,
                            "kind": "proposition",
                            "label": semantic,
                            "metadata": {
                                "source_ref": format!("source:{semantic}"),
                                "receipt_ref": receipt
                            }
                        }],
                        "edges": []
                    }
                }
            }
        })
        .to_string()
    }

    #[test]
    fn persisted_pair_uses_real_projection_boundary_not_reconstructed_semantics() {
        let left = persisted("w0", "semantic:shared", "receipt:w0");
        let right = persisted("w1", "semantic:shared", "receipt:w1");
        let specimen = project_persisted_comparative_workbench_json(
            "comparison:persisted",
            &left,
            &right,
            None,
            20,
            30,
        )
        .unwrap();

        assert!(specimen.source_backed_left);
        assert!(specimen.source_backed_right);
        assert!(specimen.provenance_backed_left);
        assert!(specimen.provenance_backed_right);
        let before = specimen.read_model.topology.before.nodes[0].id;
        let after = specimen.read_model.topology.after.nodes[0].id;
        assert_eq!(before, after);
        assert!(!specimen.creates_claim_truth);
    }

    #[test]
    fn overlay_must_reference_persisted_semantic_objects() {
        let left = persisted("w0", "semantic:present", "receipt:w0");
        let right = persisted("w1", "semantic:present", "receipt:w1");
        let overlay = json!({
            "change_layer_by_semantic_ref": {"semantic:missing": "Applicability"},
            "answer_changing_semantic_refs": ["semantic:missing"]
        })
        .to_string();

        assert!(project_persisted_comparative_workbench_json(
            "comparison:missing-overlay-object",
            &left,
            &right,
            Some(&overlay),
            20,
            30,
        )
        .is_err());
    }

    #[test]
    fn answer_changing_overlay_requires_source_and_provenance_closure() {
        let left = json!({
            "workbench": {
                "run": { "fact_run_id": "w0" },
                "semantic_context": {
                    "legal_follow_graph": {
                        "version": "graph:w0",
                        "derived_only": true,
                        "challengeable": true,
                        "nodes": [{
                            "id": "semantic:D",
                            "kind": "defeater",
                            "label": "D",
                            "metadata": {}
                        }],
                        "edges": []
                    }
                }
            }
        }).to_string();
        let right = left.clone();
        let overlay = json!({
            "change_layer_by_semantic_ref": {"semantic:D": "Applicability"},
            "explanation_by_semantic_ref": {"semantic:D": "defeater blocks route"},
            "answer_changing_semantic_refs": ["semantic:D"]
        }).to_string();

        assert!(project_persisted_comparative_workbench_json(
            "comparison:unbacked-answer-change",
            &left,
            &right,
            Some(&overlay),
            20,
            30,
        )
        .is_err());
    }

    #[test]
    fn persisted_overlay_cannot_create_authority_or_truth() {
        let left = persisted("w0", "semantic:D", "receipt:w0");
        let right = persisted("w1", "semantic:D", "receipt:w1");
        let overlay = json!({
            "change_layer_by_semantic_ref": {"semantic:D": "Applicability"},
            "explanation_by_semantic_ref": {"semantic:D": "reviewed defeater blocks route"},
            "answer_changing_semantic_refs": ["semantic:D"],
            "creates_semantic_authority": true
        })
        .to_string();

        assert!(project_persisted_comparative_workbench_json(
            "comparison:bad-overlay",
            &left,
            &right,
            Some(&overlay),
            20,
            30,
        )
        .is_err());
    }
}
