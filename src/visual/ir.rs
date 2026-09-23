use super::command::VisualObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualNode {
    pub id: VisualObjectId,
    pub semantic_ref: String,
    pub label: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualEdge {
    pub from: VisualObjectId,
    pub to: VisualObjectId,
    pub semantic_ref: String,
    pub weight: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphIr {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_and_graph_can_share_selection_identity_without_geometry_collapse() {
        let id = VisualObjectId(7);
        let graph = GraphIr {
            nodes: vec![VisualNode {
                id,
                semantic_ref: "semantic:x".into(),
                label: "X".into(),
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
    }
}
