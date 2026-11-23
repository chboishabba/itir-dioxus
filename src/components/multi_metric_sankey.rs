use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SankeyMetric {
    ContinuousAlpha,
    Perplexity,
    FrontierBlockers,
    TaskPriority,
}

impl SankeyMetric {
    pub fn label(self) -> &'static str {
        match self {
            Self::ContinuousAlpha => "Continuous Alpha",
            Self::Perplexity => "Perplexity",
            Self::FrontierBlockers => "Frontier Blockers",
            Self::TaskPriority => "Task Priority",
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SankeyFlow {
    pub chat_ref: String,
    pub cluster_ref: String,
    pub lane_ref: String,
    pub continuous_alpha: f64,
    pub perplexity: f64,
    pub frontier_blockers: f64,
    pub task_priority: f64,
}

pub fn metric_value(flow: &SankeyFlow, metric: SankeyMetric) -> f64 {
    match metric {
        SankeyMetric::ContinuousAlpha => flow.continuous_alpha,
        SankeyMetric::Perplexity => flow.perplexity,
        SankeyMetric::FrontierBlockers => flow.frontier_blockers,
        SankeyMetric::TaskPriority => flow.task_priority,
    }
}

#[component]
pub fn MultiMetricSankey(flows: Vec<SankeyFlow>) -> Element {
    let mut metric = use_signal(|| SankeyMetric::ContinuousAlpha);
    let active = metric();
    rsx! {
        section { class: "multi-metric-sankey",
            h2 { "Chat → PNF cluster → worker lane" }
            nav { aria_label: "Sankey metric",
                for candidate in [
                    SankeyMetric::ContinuousAlpha,
                    SankeyMetric::Perplexity,
                    SankeyMetric::FrontierBlockers,
                    SankeyMetric::TaskPriority,
                ] {
                    button {
                        onclick: move |_| metric.set(candidate),
                        aria_pressed: candidate == active,
                        "{candidate.label()}"
                    }
                }
            }
            div { class: "sankey-rows",
                for flow in flows {
                    div { class: "sankey-row", key: "{flow.chat_ref}-{flow.cluster_ref}-{flow.lane_ref}",
                        span { class: "sankey-node", "{flow.chat_ref}" }
                        div {
                            class: "sankey-link",
                            style: "height:{2.0 + metric_value(&flow, active).max(0.0) * 6.0}px;",
                        }
                        span { class: "sankey-node", "{flow.cluster_ref}" }
                        div {
                            class: "sankey-link",
                            style: "height:{2.0 + metric_value(&flow, active).max(0.0) * 6.0}px;",
                        }
                        span { class: "sankey-node", "{flow.lane_ref}" }
                        code { "{active.label()}: {metric_value(&flow, active):.3}" }
                    }
                }
            }
        }
    }
}
