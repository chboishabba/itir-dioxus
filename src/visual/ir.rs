use super::command::VisualObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualNode {
    pub id: VisualObjectId,
    pub semantic_ref: String,
    pub kind: String,
    pub label: String,
    pub source_refs: Vec<String>,
    pub provenance_refs: Vec<String>,
    pub hidden: bool,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualEdge {
    pub id: VisualObjectId,
    pub from: VisualObjectId,
    pub to: VisualObjectId,
    pub semantic_ref: String,
    pub kind: String,
    pub source_refs: Vec<String>,
    pub provenance_refs: Vec<String>,
    pub hidden: bool,
    pub weight: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphIr {
    pub graph_ref: String,
    pub derived_only: bool,
    pub challengeable: bool,
    pub nodes: Vec<VisualNode>,
    pub edges: Vec<VisualEdge>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartDatum {
    pub id: VisualObjectId,
    pub semantic_ref: String,
    pub label: String,
    pub value: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartIr {
    pub data: Vec<ChartDatum>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VisualisationIr {
    Graph(GraphIr),
    Chart(ChartIr),
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

impl GraphIr {
    pub fn node(&self, id: VisualObjectId) -> Option<&VisualNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn edge(&self, id: VisualObjectId) -> Option<&VisualEdge> {
        self.edges.iter().find(|edge| edge.id == id)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualObjectInspection {
    pub object_id: VisualObjectId,
    pub semantic_ref: String,
    pub kind: String,
    pub source_refs: Vec<String>,
    pub provenance_refs: Vec<String>,
}

impl GraphIr {
    pub fn inspect_object(&self, id: VisualObjectId) -> Option<VisualObjectInspection> {
        if let Some(node) = self.node(id) {
            return Some(VisualObjectInspection {
                object_id: node.id,
                semantic_ref: node.semantic_ref.clone(),
                kind: node.kind.clone(),
                source_refs: node.source_refs.clone(),
                provenance_refs: node.provenance_refs.clone(),
            });
        }
        self.edge(id).map(|edge| VisualObjectInspection {
            object_id: edge.id,
            semantic_ref: edge.semantic_ref.clone(),
            kind: edge.kind.clone(),
            source_refs: edge.source_refs.clone(),
            provenance_refs: edge.provenance_refs.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_and_graph_can_share_selection_identity_without_geometry_collapse() {
        let id = VisualObjectId(7);
        let graph = GraphIr {
            graph_ref: "graph:test".into(),
            derived_only: true,
            challengeable: true,
            nodes: vec![VisualNode {
                id,
                semantic_ref: "semantic:x".into(),
                kind: "proposition".into(),
                label: "X".into(),
                source_refs: vec!["source:x".into()],
                provenance_refs: vec!["receipt:x".into()],
                hidden: false,
                x: 0.0,
                y: 0.0,
            }],
            edges: vec![],
        };
        let chart = ChartIr {
            data: vec![ChartDatum {
                id,
                semantic_ref: "semantic:x".into(),
                label: "X".into(),
                value: 1.0,
            }],
        };
        assert_eq!(graph.nodes[0].id, chart.data[0].id);
        assert_ne!(graph.nodes[0].x, chart.data[0].value);
    }

    #[test]
    fn hidden_visual_node_remains_in_ir_registry() {
        let graph = GraphIr {
            graph_ref: "graph:hidden".into(),
            derived_only: true,
            challengeable: true,
            nodes: vec![VisualNode {
                id: VisualObjectId(9),
                semantic_ref: "semantic:hidden".into(),
                kind: "context".into(),
                label: "Hidden".into(),
                source_refs: vec![],
                provenance_refs: vec![],
                hidden: true,
                x: 0.0,
                y: 0.0,
            }],
            edges: vec![],
        };
        assert!(graph.node(VisualObjectId(9)).is_some());
        assert!(graph.node(VisualObjectId(9)).unwrap().hidden);
    }

    #[test]
    fn selected_object_reopens_semantic_source_and_provenance_refs() {
        let graph = GraphIr {
            graph_ref: "graph:inspect".into(),
            derived_only: true,
            challengeable: true,
            nodes: vec![VisualNode {
                id: VisualObjectId(42),
                semantic_ref: "semantic:authority:42".into(),
                kind: "authority_receipt".into(),
                label: "Authority".into(),
                source_refs: vec!["source:judgment:42".into()],
                provenance_refs: vec!["receipt:authority:42".into()],
                hidden: false,
                x: 0.0,
                y: 0.0,
            }],
            edges: vec![],
        };

        let detail = graph.inspect_object(VisualObjectId(42)).unwrap();
        assert_eq!(detail.semantic_ref, "semantic:authority:42");
        assert_eq!(detail.source_refs, vec!["source:judgment:42"]);
        assert_eq!(detail.provenance_refs, vec!["receipt:authority:42"]);
    }
}
