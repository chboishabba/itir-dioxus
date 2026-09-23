use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::visual::{
    command::VisualObjectId,
    ir::{GraphIr, VisualEdge, VisualNode},
};

use super::{au_legal_bearing_read_model, AuLegalBearingInput, UnifiedWorkbenchReadModel};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegalFollowGraphNode {
    pub id: String,
    pub kind: String,
    pub label: String,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegalFollowGraphEdge {
    pub source: String,
    pub target: String,
    pub kind: String,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LegalFollowGraph {
    pub version: Option<String>,
    pub derived_only: Option<bool>,
    pub challengeable: Option<bool>,
    #[serde(default)]
    pub nodes: Vec<LegalFollowGraphNode>,
    #[serde(default)]
    pub edges: Vec<LegalFollowGraphEdge>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuFactReviewProjectionInput {
    pub world_ref: String,
    pub source_refs: Vec<String>,
    pub event_refs: Vec<String>,
    pub handoff_refs: Vec<String>,
    pub research_residual_refs: Vec<String>,
    pub legal_follow_graph: LegalFollowGraph,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuFactReviewProjection {
    pub read_model: UnifiedWorkbenchReadModel,
    pub graph_ir: GraphIr,
    pub persisted_source_refs: Vec<String>,
    pub persisted_event_refs: Vec<String>,
    pub persisted_research_refs: Vec<String>,
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
        .map(edge_semantic_ref)
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

pub fn project_persisted_au_workbench_json(
    raw: &str,
    max_nodes: usize,
    max_edges: usize,
) -> Result<AuFactReviewProjection, String> {
    let document: Value =
        serde_json::from_str(raw).map_err(|error| format!("invalid persisted workbench JSON: {error}"))?;
    let workbench = document.get("workbench").unwrap_or(&document);

    let graph_value = workbench
        .get("legal_follow_graph")
        .or_else(|| workbench.get("semantic_context")?.get("legal_follow_graph"))
        .or_else(|| document.get("semantic_context")?.get("legal_follow_graph"))
        .ok_or_else(|| "persisted AU workbench has no legal_follow_graph".to_string())?;

    let legal_follow_graph: LegalFollowGraph = serde_json::from_value(graph_value.clone())
        .map_err(|error| format!("invalid persisted legal_follow_graph: {error}"))?;

    let source_refs = collect_refs(
        workbench
            .get("sources")
            .or_else(|| document.get("sources")),
        &["source_id", "source_ref"],
    );
    let event_refs = collect_refs(
        workbench
            .get("events")
            .or_else(|| document.get("events")),
        &["event_id"],
    );
    let handoff_refs = collect_operator_view_refs(workbench, "professional_handoff");
    let research_residual_refs = collect_research_refs(workbench);

    let world_ref = workbench
        .get("run")
        .and_then(|run| {
            string_field(run, "fact_run_id")
                .or_else(|| string_field(run, "run_id"))
                .or_else(|| string_field(run, "semantic_run_id"))
        })
        .or_else(|| {
            document.get("run").and_then(|run| {
                string_field(run, "fact_run_id")
                    .or_else(|| string_field(run, "run_id"))
                    .or_else(|| string_field(run, "semantic_run_id"))
            })
        })
        .map(|run| format!("world:au:{run}"))
        .unwrap_or_else(|| "world:au:persisted-fact-review".into());

    let input = AuFactReviewProjectionInput {
        world_ref,
        source_refs: source_refs.clone(),
        event_refs: event_refs.clone(),
        handoff_refs,
        research_residual_refs: research_residual_refs.clone(),
        legal_follow_graph: legal_follow_graph.clone(),
    };

    let read_model = project_au_fact_review(input);
    let graph_ir = legal_follow_graph_to_ir(&legal_follow_graph, max_nodes, max_edges)?;

    Ok(AuFactReviewProjection {
        read_model,
        graph_ir,
        persisted_source_refs: source_refs,
        persisted_event_refs: event_refs,
        persisted_research_refs: research_residual_refs,
    })
}

pub fn legal_follow_graph_to_ir(
    graph: &LegalFollowGraph,
    max_nodes: usize,
    max_edges: usize,
) -> Result<GraphIr, String> {
    if graph.nodes.is_empty() {
        return Ok(GraphIr {
            graph_ref: graph
                .version
                .clone()
                .unwrap_or_else(|| "au.legal_follow_graph".into()),
            derived_only: graph.derived_only.unwrap_or(true),
            challengeable: graph.challengeable.unwrap_or(true),
            nodes: vec![],
            edges: vec![],
        });
    }

    // A bounded proof view must retain a usable neighbourhood.  Taking the
    // first N source nodes can produce a visually populated but edgeless
    // graph when producer ordering puts event nodes before relation endpoints.
    // Prefer source-order edges that fit the node budget, then fill remaining
    // capacity from source-order nodes.  This is a projection choice only;
    // it neither infers nor promotes an omitted relationship.
    let known_node_ids = graph
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut retained_id_values = BTreeSet::<String>::new();
    let mut retained_edge_indexes = Vec::new();
    for (index, edge) in graph.edges.iter().enumerate() {
        if retained_edge_indexes.len() >= max_edges
            || !known_node_ids.contains(edge.source.as_str())
            || !known_node_ids.contains(edge.target.as_str())
        {
            continue;
        }
        let required_nodes = [edge.source.as_str(), edge.target.as_str()]
            .into_iter()
            .filter(|node_ref| !retained_id_values.contains(*node_ref))
            .collect::<BTreeSet<_>>()
            .len();
        if retained_id_values.len() + required_nodes > max_nodes {
            continue;
        }
        retained_id_values.insert(edge.source.clone());
        retained_id_values.insert(edge.target.clone());
        retained_edge_indexes.push(index);
    }
    for node in &graph.nodes {
        if retained_id_values.len() >= max_nodes {
            break;
        }
        retained_id_values.insert(node.id.clone());
    }
    let retained_nodes = graph
        .nodes
        .iter()
        .filter(|node| retained_id_values.contains(&node.id))
        .collect::<Vec<_>>();
    let retained_ids = retained_nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();

    let mut visual_id_by_semantic_ref = BTreeMap::new();
    let mut semantic_ref_by_visual_id = BTreeMap::new();

    for node in &retained_nodes {
        let visual_id = stable_visual_id(&node.id);
        if let Some(existing) = semantic_ref_by_visual_id.insert(visual_id, node.id.clone()) {
            if existing != node.id {
                return Err(format!(
                    "stable visual ID collision between {existing} and {}",
                    node.id
                ));
            }
        }
        visual_id_by_semantic_ref.insert(node.id.clone(), visual_id);
    }

    let count = retained_nodes.len().max(1) as f32;
    let visual_nodes = retained_nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let phase = (index as f32 / count) * std::f32::consts::TAU;
            VisualNode {
                id: visual_id_by_semantic_ref[&node.id],
                semantic_ref: node.id.clone(),
                kind: node.kind.clone(),
                label: node.label.clone(),
                source_refs: metadata_refs(&node.metadata, SOURCE_REF_KEYS),
                provenance_refs: metadata_refs(&node.metadata, PROVENANCE_REF_KEYS),
                hidden: false,
                x: phase.cos() * 0.75,
                y: phase.sin() * 0.75,
            }
        })
        .collect::<Vec<_>>();

    let mut visual_edges = Vec::new();
    for index in retained_edge_indexes {
        let edge = &graph.edges[index];
        if !retained_ids.contains(edge.source.as_str()) || !retained_ids.contains(edge.target.as_str()) {
            continue;
        }
        let semantic_ref = edge_semantic_ref(edge);
        visual_edges.push(VisualEdge {
            id: stable_visual_id(&semantic_ref),
            from: visual_id_by_semantic_ref[&edge.source],
            to: visual_id_by_semantic_ref[&edge.target],
            semantic_ref,
            kind: edge.kind.clone(),
            source_refs: metadata_refs(&edge.metadata, SOURCE_REF_KEYS),
            provenance_refs: metadata_refs(&edge.metadata, PROVENANCE_REF_KEYS),
            hidden: false,
            weight: edge_weight(edge),
        });
    }

    Ok(GraphIr {
        graph_ref: graph
            .version
            .clone()
            .unwrap_or_else(|| "au.legal_follow_graph".into()),
        derived_only: graph.derived_only.unwrap_or(true),
        challengeable: graph.challengeable.unwrap_or(true),
        nodes: visual_nodes,
        edges: visual_edges,
    })
}

const SOURCE_REF_KEYS: &[&str] = &[
    "source_ref",
    "source_refs",
    "source_id",
    "source_ids",
    "canonical_ref",
    "content_refs",
    "span_refs",
];

const PROVENANCE_REF_KEYS: &[&str] = &[
    "provenance_ref",
    "provenance_refs",
    "receipt_ref",
    "receipt_refs",
    "promoted_record_ref",
    "support_phi_ids",
    "record_ref",
    "fact_node_ref",
    "claim_node_ref",
];

fn metadata_refs(metadata: &BTreeMap<String, Value>, keys: &[&str]) -> Vec<String> {
    let mut out = BTreeSet::new();
    for key in keys {
        if let Some(value) = metadata.get(*key) {
            collect_strings(value, &mut out);
        }
    }
    out.into_iter().collect()
}

fn collect_strings(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::String(value) if !value.trim().is_empty() => {
            out.insert(value.clone());
        }
        Value::Array(values) => {
            for value in values {
                collect_strings(value, out);
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                collect_strings(value, out);
            }
        }
        _ => {}
    }
}

fn edge_semantic_ref(edge: &LegalFollowGraphEdge) -> String {
    format!("edge:{}:{}->{}", edge.kind, edge.source, edge.target)
}

fn edge_weight(edge: &LegalFollowGraphEdge) -> f32 {
    edge.metadata
        .get("weight")
        .and_then(Value::as_f64)
        .map(|weight| weight as f32)
        .unwrap_or(1.0)
}

pub fn stable_visual_id(semantic_ref: &str) -> VisualObjectId {
    // Deterministic FNV-1a. This is a visual identity projection only, never a
    // canonical semantic digest or authority identifier.
    let mut hash = 0xcbf29ce484222325u64;
    for byte in semantic_ref.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    VisualObjectId(hash)
}

fn collect_refs(rows: Option<&Value>, keys: &[&str]) -> Vec<String> {
    let mut refs = BTreeSet::new();
    let Some(Value::Array(rows)) = rows else {
        return vec![];
    };
    for row in rows {
        for key in keys {
            if let Some(value) = row.get(*key).and_then(Value::as_str) {
                if !value.trim().is_empty() {
                    refs.insert(value.to_string());
                }
            }
        }
    }
    refs.into_iter().collect()
}

fn collect_operator_view_refs(workbench: &Value, view_name: &str) -> Vec<String> {
    let items = workbench
        .get("operator_views")
        .and_then(|views| views.get(view_name))
        .and_then(|view| view.get("items"))
        .and_then(Value::as_array);
    let mut refs = BTreeSet::new();
    for item in items.into_iter().flatten() {
        for key in ["fact_id", "event_id", "source_id", "claim_id"] {
            if let Some(value) = item.get(key).and_then(Value::as_str) {
                refs.insert(value.to_string());
            }
        }
    }
    refs.into_iter().collect()
}

fn collect_research_refs(workbench: &Value) -> Vec<String> {
    let mut refs = BTreeSet::new();

    if let Some(rows) = workbench.get("review_queue").and_then(Value::as_array) {
        for row in rows {
            for key in ["fact_id", "claim_id", "event_id"] {
                if let Some(value) = row.get(key).and_then(Value::as_str) {
                    refs.insert(value.to_string());
                }
            }
        }
    }

    if let Some(views) = workbench.get("operator_views") {
        for view_name in ["authority_follow", "legal_follow_graph", "contested_items"] {
            if let Some(queue) = views
                .get(view_name)
                .and_then(|view| view.get("queue").or_else(|| view.get("items")))
                .and_then(Value::as_array)
            {
                for row in queue {
                    for key in ["fact_id", "claim_id", "edge_id", "event_id", "target_ref"] {
                        if let Some(value) = row.get(key).and_then(Value::as_str) {
                            refs.insert(value.to_string());
                        }
                    }
                }
            }
        }
    }

    refs.into_iter().collect()
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workbench::{StageAvailability, WorkbenchStageKind};

    fn graph_fixture() -> LegalFollowGraph {
        LegalFollowGraph {
            version: Some("au.legal_follow_graph.v1".into()),
            derived_only: Some(true),
            challengeable: Some(true),
            nodes: vec![
                LegalFollowGraphNode {
                    id: "node:authority:1".into(),
                    kind: "authority".into(),
                    label: "Authority".into(),
                    metadata: BTreeMap::from([
                        ("source_ref".into(), Value::String("source:judgment".into())),
                        ("receipt_ref".into(), Value::String("receipt:authority:1".into())),
                    ]),
                },
                LegalFollowGraphNode {
                    id: "node:proposition:1".into(),
                    kind: "proposition".into(),
                    label: "Proposition".into(),
                    metadata: BTreeMap::new(),
                },
            ],
            edges: vec![LegalFollowGraphEdge {
                source: "node:authority:1".into(),
                target: "node:proposition:1".into(),
                kind: "supports".into(),
                metadata: BTreeMap::new(),
            }],
        }
    }

    #[test]
    fn persisted_legal_follow_graph_makes_proof_stage_available() {
        let model = project_au_fact_review(AuFactReviewProjectionInput {
            world_ref: "world:au:fixture".into(),
            source_refs: vec!["source:au:judgment".into()],
            event_refs: vec!["event:au:hearing".into()],
            handoff_refs: vec![],
            research_residual_refs: vec!["residual:authority-follow".into()],
            legal_follow_graph: graph_fixture(),
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

    #[test]
    fn legal_graph_projects_to_stable_provenance_bearing_visual_ir() {
        let graph = graph_fixture();
        let first = legal_follow_graph_to_ir(&graph, 20, 30).unwrap();
        let second = legal_follow_graph_to_ir(&graph, 20, 30).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.nodes.len(), 2);
        assert_eq!(first.edges.len(), 1);
        assert_eq!(first.nodes[0].source_refs, vec!["source:judgment"]);
        assert_eq!(first.nodes[0].provenance_refs, vec!["receipt:authority:1"]);
        assert!(first.derived_only);
        assert!(first.challengeable);
    }

    #[test]
    fn bounded_graph_projection_prefers_a_connected_neighbourhood() {
        let mut graph = graph_fixture();
        graph.nodes.insert(
            0,
            LegalFollowGraphNode {
                id: "node:unconnected:1".into(),
                kind: "event".into(),
                label: "Unconnected first node".into(),
                metadata: BTreeMap::new(),
            },
        );
        graph.nodes.insert(
            1,
            LegalFollowGraphNode {
                id: "node:unconnected:2".into(),
                kind: "event".into(),
                label: "Unconnected second node".into(),
                metadata: BTreeMap::new(),
            },
        );

        let projection = legal_follow_graph_to_ir(&graph, 2, 1).unwrap();

        assert_eq!(projection.nodes.len(), 2);
        assert_eq!(projection.edges.len(), 1);
        assert_eq!(
            projection.edges[0].semantic_ref,
            "edge:supports:node:authority:1->node:proposition:1"
        );
    }

    #[test]
    fn real_shape_json_adapter_reads_semantic_context_graph_without_fabrication() {
        let raw = serde_json::json!({
            "run": { "fact_run_id": "factrun:au-real" },
            "sources": [{ "source_id": "src:1" }],
            "events": [{ "event_id": "event:1" }],
            "review_queue": [{ "fact_id": "fact:needs-review" }],
            "operator_views": {
                "professional_handoff": { "items": [] },
                "authority_follow": { "queue": [{ "fact_id": "fact:authority-follow" }] }
            },
            "semantic_context": {
                "legal_follow_graph": graph_fixture()
            }
        })
        .to_string();

        let projection = project_persisted_au_workbench_json(&raw, 20, 30).unwrap();

        assert_eq!(projection.persisted_source_refs, vec!["src:1"]);
        assert_eq!(projection.persisted_event_refs, vec!["event:1"]);
        assert_eq!(
            projection
                .read_model
                .stage(WorkbenchStageKind::MatterProof)
                .unwrap()
                .availability,
            StageAvailability::Available
        );
        assert!(projection
            .persisted_research_refs
            .contains(&"fact:needs-review".to_string()));
        assert!(projection
            .persisted_research_refs
            .contains(&"fact:authority-follow".to_string()));
    }

    #[test]
    fn visual_ids_are_stable_but_not_semantic_authority() {
        assert_eq!(
            stable_visual_id("node:authority:1"),
            stable_visual_id("node:authority:1")
        );
        assert_ne!(
            stable_visual_id("node:authority:1"),
            stable_visual_id("node:authority:2")
        );
    }
}
