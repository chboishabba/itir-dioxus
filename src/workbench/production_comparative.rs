#![cfg(feature = "production-data")]

use std::collections::{BTreeMap, BTreeSet};

use postgres::{Client, NoTls};
use sensiblaw_legal_runtime::{
    project_typed_three_way_workbench_comparison, project_typed_workbench_comparison,
    run_pabai_comparative_regression, workbench_overlay_from_explanation,
    ComparativeWorkbenchOverlay,
};
use sensiblaw_pg_source_store::{
    load_database_config, load_persisted_workbench_projection,
};
use sensiblaw_reader_model::{
    ComparativeWorkbenchProjection, PersistedWorkbenchGraph,
    ThreeWayComparativeWorkbenchProjection,
};

use crate::visual::{
    command::VisualObjectId,
    ir::{stable_visual_id, GraphIr, VisualEdge, VisualNode},
};

use super::{
    comparative::{
        comparative_workbench_read_model,
        three_way_comparative_sequence_with_overlays,
        ComparativeExplanationOverlay, ComparativeSelectors,
        ComparativeWorkbenchReadModel, ThreeWayComparativeSequence,
    },
};

fn graph_to_ir(graph: &PersistedWorkbenchGraph) -> Result<GraphIr, String> {
    let mut id_by_ref = BTreeMap::<String, VisualObjectId>::new();
    let mut ref_by_id = BTreeMap::<VisualObjectId, String>::new();
    for node in &graph.nodes {
        let id = stable_visual_id(&node.semantic_ref);
        if let Some(existing) = ref_by_id.insert(id, node.semantic_ref.clone()) {
            if existing != node.semantic_ref {
                return Err(format!(
                    "stable visual ID collision between {existing} and {}",
                    node.semantic_ref
                ));
            }
        }
        id_by_ref.insert(node.semantic_ref.clone(), id);
    }

    let count = graph.nodes.len().max(1) as f32;
    let nodes = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let phase = (index as f32 / count) * std::f32::consts::TAU;
            VisualNode {
                id: id_by_ref[&node.semantic_ref],
                semantic_ref: node.semantic_ref.clone(),
                kind: node.semantic_kind.clone(),
                label: node.label.clone(),
                source_refs: node.source_refs.clone(),
                provenance_refs: node.provenance_refs.clone(),
                hidden: false,
                x: phase.cos() * 0.75,
                y: phase.sin() * 0.75,
            }
        })
        .collect::<Vec<_>>();

    let edges = graph
        .edges
        .iter()
        .map(|edge| {
            let from = id_by_ref.get(&edge.from_ref).copied().ok_or_else(|| {
                format!(
                    "typed edge {} references missing source node {}",
                    edge.semantic_ref, edge.from_ref
                )
            })?;
            let to = id_by_ref.get(&edge.to_ref).copied().ok_or_else(|| {
                format!(
                    "typed edge {} references missing target node {}",
                    edge.semantic_ref, edge.to_ref
                )
            })?;
            Ok(VisualEdge {
                id: stable_visual_id(&edge.semantic_ref),
                from,
                to,
                semantic_ref: edge.semantic_ref.clone(),
                kind: format!(
                    "{}|challengeable={}",
                    edge.relation, edge.challengeable
                ),
                source_refs: edge.source_refs.clone(),
                provenance_refs: edge.provenance_refs.clone(),
                hidden: false,
                weight: 1.0,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(GraphIr {
        graph_ref: graph.projection_ref.clone(),
        derived_only: graph.derived_only,
        challengeable: graph.challengeable,
        nodes,
        edges,
    })
}

fn overlay_from_typed(
    projection: &ComparativeWorkbenchProjection,
) -> ComparativeExplanationOverlay {
    ComparativeExplanationOverlay {
        change_layer_by_semantic_ref: projection.change_layer_by_semantic_ref.clone(),
        explanation_by_semantic_ref: projection.explanation_by_semantic_ref.clone(),
        answer_changing_semantic_refs: projection
            .answer_changing_semantic_refs
            .iter()
            .cloned()
            .collect(),
        unresolved_semantic_refs: projection
            .unresolved_semantic_refs
            .iter()
            .cloned()
            .collect(),
        creates_semantic_authority: false,
        creates_claim_truth: false,
    }
}

fn validate_visual_parity(
    typed: &ComparativeWorkbenchProjection,
    visual: &ComparativeWorkbenchReadModel,
) -> Result<(), String> {
    let typed_shared = typed
        .shared_semantic_refs
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let typed_changed = typed
        .changed_semantic_refs
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let typed_unchanged = typed_shared
        .difference(&typed_changed)
        .cloned()
        .collect::<BTreeSet<_>>();
    let typed_left = typed
        .left_only_semantic_refs
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let typed_right = typed
        .right_only_semantic_refs
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    let comparison = &visual.topology.comparative;
    if comparison.shared_semantic_refs != typed_unchanged
        || comparison.changed_semantic_refs != typed_changed
        || comparison.left_only_semantic_refs != typed_left
        || comparison.right_only_semantic_refs != typed_right
    {
        return Err(
            "Dioxus visual comparison diverged from typed runtime comparative projection".into(),
        );
    }
    Ok(())
}

pub fn comparative_read_model_from_typed(
    projection: &ComparativeWorkbenchProjection,
) -> Result<ComparativeWorkbenchReadModel, String> {
    projection.validate_read_only()?;
    let left = graph_to_ir(&projection.left_graph)?;
    let right = graph_to_ir(&projection.right_graph)?;
    let model = comparative_workbench_read_model(
        projection.comparison_ref.clone(),
        &left,
        &right,
        ComparativeSelectors {
            left_ref: projection.left_world_ref.clone(),
            right_ref: projection.right_world_ref.clone(),
            query_ref: None,
            consumer_ref: None,
            as_at_ref: None,
            scope_ref: None,
        },
        overlay_from_typed(projection),
    );
    validate_visual_parity(projection, &model)?;
    Ok(model)
}

pub fn three_way_read_model_from_typed(
    projection: &ThreeWayComparativeWorkbenchProjection,
) -> Result<ThreeWayComparativeSequence, String> {
    projection.validate_read_only()?;

    let w0 = graph_to_ir(&projection.w0.legal_follow_graph)?;
    let w1 = graph_to_ir(&projection.w1.legal_follow_graph)?;
    let w2 = graph_to_ir(&projection.w2.legal_follow_graph)?;

    let sequence = three_way_comparative_sequence_with_overlays(
        "comparison:typed:w0-w1-w2",
        &w0,
        &w1,
        &w2,
        ComparativeSelectors {
            left_ref: projection.w0.world_ref.clone(),
            right_ref: projection.w2.world_ref.clone(),
            query_ref: None,
            consumer_ref: None,
            as_at_ref: None,
            scope_ref: None,
        },
        overlay_from_typed(&projection.w0_to_w1),
        overlay_from_typed(&projection.w1_to_w2),
    );
    validate_visual_parity(&projection.w0_to_w1, &sequence.w0_to_w1)?;
    validate_visual_parity(&projection.w1_to_w2, &sequence.w1_to_w2)?;
    Ok(sequence)
}

pub fn load_postgres_comparative_workbench(
    comparison_ref: &str,
    left_projection_ref: &str,
    right_projection_ref: &str,
    overlay: Option<&ComparativeWorkbenchOverlay>,
) -> Result<ComparativeWorkbenchReadModel, String> {
    let config = load_database_config(None).map_err(|error| error.to_string())?;
    let mut client =
        Client::connect(config.database_url(), NoTls).map_err(|error| error.to_string())?;

    let left = load_persisted_workbench_projection(
        &mut client,
        left_projection_ref,
        &format!("world:postgres:{left_projection_ref}"),
    )
    .map_err(|error| error.to_string())?;
    let right = load_persisted_workbench_projection(
        &mut client,
        right_projection_ref,
        &format!("world:postgres:{right_projection_ref}"),
    )
    .map_err(|error| error.to_string())?;

    let typed =
        project_typed_workbench_comparison(comparison_ref, &left, &right, overlay)?;
    comparative_read_model_from_typed(&typed)
}

pub fn load_postgres_three_way_comparative_workbench(
    comparison_ref: &str,
    w0_projection_ref: &str,
    w1_projection_ref: &str,
    w2_projection_ref: &str,
    w0_w1_overlay: Option<&ComparativeWorkbenchOverlay>,
    w1_w2_overlay: Option<&ComparativeWorkbenchOverlay>,
) -> Result<ThreeWayComparativeSequence, String> {
    let config = load_database_config(None).map_err(|error| error.to_string())?;
    let mut client =
        Client::connect(config.database_url(), NoTls).map_err(|error| error.to_string())?;

    let load = |client: &mut Client, projection_ref: &str| {
        load_persisted_workbench_projection(
            client,
            projection_ref,
            &format!("world:postgres:{projection_ref}"),
        )
        .map_err(|error| error.to_string())
    };

    let w0 = load(&mut client, w0_projection_ref)?;
    let w1 = load(&mut client, w1_projection_ref)?;
    let w2 = load(&mut client, w2_projection_ref)?;

    let typed = project_typed_three_way_workbench_comparison(
        comparison_ref,
        w0,
        w1,
        w2,
        w0_w1_overlay,
        w1_w2_overlay,
    )?;
    three_way_read_model_from_typed(&typed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sensiblaw_reader_model::{
        PersistedWorkbenchNode, PersistedWorkbenchProjection,
    };

    fn typed(world: &str, receipt: &str) -> PersistedWorkbenchProjection {
        PersistedWorkbenchProjection {
            world_ref: world.into(),
            source_refs: vec!["source:shared".into()],
            event_refs: vec![],
            handoff_refs: vec![],
            research_residual_refs: vec![],
            legal_follow_graph: PersistedWorkbenchGraph {
                projection_ref: format!("projection:{world}"),
                document_ref: "document:shared".into(),
                nodes: vec![PersistedWorkbenchNode {
                    semantic_ref: "semantic:shared".into(),
                    semantic_kind: "proposition".into(),
                    label: "shared".into(),
                    source_refs: vec!["source:shared".into()],
                    provenance_refs: vec![receipt.into()],
                    candidate_only: true,
                }],
                edges: vec![],
                derived_only: true,
                challengeable: true,
            },
            candidate_only: true,
            projection_only: true,
            creates_semantic_authority: false,
            creates_claim_truth: false,
            pays_residual: false,
        }
    }

    #[test]
    fn typed_runtime_projection_is_the_visual_parity_authority() {
        let left = typed("w0", "receipt:old");
        let right = typed("w1", "receipt:new");
        let typed =
            project_typed_workbench_comparison("comparison:typed", &left, &right, None)
                .unwrap();
        let visual = comparative_read_model_from_typed(&typed).unwrap();

        assert!(typed
            .changed_semantic_refs
            .contains(&"semantic:shared".to_string()));
        assert!(visual
            .topology
            .comparative
            .changed_semantic_refs
            .contains("semantic:shared"));
    }
}


pub fn load_postgres_pabai_three_way_workbench(
    w0_projection_ref: &str,
    w1_projection_ref: &str,
    w2_projection_ref: &str,
) -> Result<ThreeWayComparativeSequence, String> {
    let pabai = run_pabai_comparative_regression()?;
    let d_overlay =
        workbench_overlay_from_explanation(&pabai.w0_to_w1_explanation)?;
    let c_overlay =
        workbench_overlay_from_explanation(&pabai.w1_to_w2_explanation)?;
    load_postgres_three_way_comparative_workbench(
        "comparison:pabai:postgres:w0-w1-w2",
        w0_projection_ref,
        w1_projection_ref,
        w2_projection_ref,
        Some(&d_overlay),
        Some(&c_overlay),
    )
}
