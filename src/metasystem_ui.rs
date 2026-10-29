use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use dioxus::router::prelude::*;

use crate::components::{
    CrossProjectNodeGraph, MultiMetricSankey, ProjectEdge, ProjectNode, SankeyFlow, TopicExplorer,
    TopicNode,
};

#[derive(Clone, Routable, PartialEq)]
pub enum MetasystemRoute {
    #[route("/topics")]
    Topics {},
    #[route("/flow")]
    Flow {},
    #[route("/projects")]
    Projects {},
    #[route("/")]
    Home {},
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MetasystemProjection {
    pub topics: Vec<TopicNode>,
    pub flows: Vec<SankeyFlow>,
    pub project_nodes: Vec<ProjectNode>,
    pub project_edges: Vec<ProjectEdge>,
}

impl Default for MetasystemProjection {
    fn default() -> Self {
        Self { topics: vec![], flows: vec![], project_nodes: vec![], project_edges: vec![] }
    }
}

pub fn load_projection(path: impl AsRef<std::path::Path>) -> Result<MetasystemProjection, String> {
    let text = std::fs::read_to_string(path.as_ref())
        .map_err(|error| format!("metasystem projection read failed: {error}"))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("metasystem projection parse failed: {error}"))
}

#[component]
pub fn MetasystemRouter(model: MetasystemProjection) -> Element {
    use_context_provider(|| model);
    rsx! { Router::<MetasystemRoute> {} }
}

#[component]
fn Home() -> Element {
    rsx! {
        main {
            h1 { "Sublinear Topic Metasystem" }
            nav {
                Link { to: MetasystemRoute::Topics {}, "Topics" }
                " · "
                Link { to: MetasystemRoute::Flow {}, "Flow" }
                " · "
                Link { to: MetasystemRoute::Projects {}, "Projects" }
            }
        }
    }
}

#[component]
fn Topics() -> Element {
    let model = use_context::<MetasystemProjection>();
    rsx! { main { TopicExplorer { topics: model.topics } } }
}

#[component]
fn Flow() -> Element {
    let model = use_context::<MetasystemProjection>();
    rsx! { main { MultiMetricSankey { flows: model.flows } } }
}

#[component]
fn Projects() -> Element {
    let model = use_context::<MetasystemProjection>();
    rsx! { main { CrossProjectNodeGraph { nodes: model.project_nodes, edges: model.project_edges } } }
}
