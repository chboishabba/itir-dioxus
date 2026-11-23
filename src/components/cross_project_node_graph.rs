use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ProjectNode {
    pub node_ref: String,
    pub project_ref: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub receipt_ref: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ProjectEdge {
    pub edge_ref: String,
    pub source_ref: String,
    pub target_ref: String,
    pub relation_ref: String,
}

pub fn valid_edges(nodes: &[ProjectNode], edges: &[ProjectEdge]) -> Vec<ProjectEdge> {
    let ids = nodes.iter().map(|node| node.node_ref.as_str()).collect::<std::collections::BTreeSet<_>>();
    edges.iter()
        .filter(|edge| ids.contains(edge.source_ref.as_str()) && ids.contains(edge.target_ref.as_str()))
        .cloned()
        .collect()
}

#[component]
pub fn CrossProjectNodeGraph(nodes: Vec<ProjectNode>, edges: Vec<ProjectEdge>) -> Element {
    let edges = valid_edges(&nodes, &edges);
    rsx! {
        section { class: "cross-project-node-graph",
            h2 { "Cross-project node graph" }
            p { "ITIR-suite · SensibLaw · StatiBaker · SeaMeInIt. Only supplied edges are rendered." }
            svg {
                view_box: "0 0 1000 620",
                role: "img",
                aria_label: "Cross-project graph canvas",
                for edge in edges {
                    if let (Some(source), Some(target)) = (
                        nodes.iter().find(|node| node.node_ref == edge.source_ref),
                        nodes.iter().find(|node| node.node_ref == edge.target_ref),
                    ) {
                        line {
                            key: "{edge.edge_ref}",
                            x1: "{source.x}", y1: "{source.y}",
                            x2: "{target.x}", y2: "{target.y}",
                            stroke: "currentColor", stroke_width: "1.5",
                        }
                    }
                }
                for node in nodes {
                    g { key: "{node.node_ref}",
                        circle { cx: "{node.x}", cy: "{node.y}", r: "14", fill: "none", stroke: "currentColor" }
                        text { x: "{node.x + 20.0}", y: "{node.y + 4.0}", "{node.project_ref}: {node.label}" }
                        if let Some(receipt) = &node.receipt_ref {
                            title { "{node.node_ref} · {receipt}" }
                        }
                    }
                }
            }
        }
    }
}
