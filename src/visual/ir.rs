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

impl GraphIr {
    pub fn node(&self, id: VisualObjectId) -> Option<&VisualNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn edge(&self, id: VisualObjectId) -> Option<&VisualEdge> {
        self.edges.iter().find(|edge| edge.id == id)
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
}
