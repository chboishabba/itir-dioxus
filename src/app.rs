use dioxus::prelude::*;

use crate::workbench::{
    comparative::ComparativeWorkbenchReadModel, wave5_personal_handoff_read_model,
    StageAvailability, UnifiedWorkbenchReadModel, WorkbenchStageKind,
};

pub fn app() -> Element {
    let model = wave5_personal_handoff_read_model();

    rsx! {
        document::Title { "ITIR Unified Workbench" }
        main {
            style: "font-family: sans-serif; max-width: 1100px; margin: 0 auto; padding: 2rem;",
            header {
                h1 { "ITIR Unified Workbench" }
                p { "Same canonical world, progressive operator projections." }
                p {
                    style: "font-size: 0.9rem; opacity: 0.75;",
                    "Derived-only · no authority, truth, or payment promotion"
                }
            }
            WorkbenchStageStrip { model }
        }
    }
}

#[component]
fn WorkbenchStageStrip(model: UnifiedWorkbenchReadModel) -> Element {
    rsx! {
        section {
            h2 { "Progression" }
            div {
                style: "display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 0.75rem;",
                for stage in model.stages.iter() {
                    WorkbenchStageCard {
                        kind: stage.kind,
                        availability: stage.availability.clone(),
                        semantic_ref_count: stage.semantic_refs.len()
                    }
                }
            }
        }
    }
}

#[component]
fn WorkbenchStageCard(
    kind: WorkbenchStageKind,
    availability: StageAvailability,
    semantic_ref_count: usize,
) -> Element {
    let label = match kind {
        WorkbenchStageKind::Journal => "Journal",
        WorkbenchStageKind::Timeline => "Timeline",
        WorkbenchStageKind::Handoff => "Handoff",
        WorkbenchStageKind::MatterProof => "Matter / Proof",
        WorkbenchStageKind::Research => "Research",
    };

    let reason = availability.reason().unwrap_or("ready");
    let status = availability.label();

    rsx! {
        article {
            style: "border: 1px solid #aaa; border-radius: 0.6rem; padding: 0.8rem;",
            strong { "{label}" }
            div { style: "margin-top: 0.5rem;", "{status}" }
            div { style: "font-size: 0.8rem; opacity: 0.75;", "{reason}" }
            div { style: "font-size: 0.8rem; margin-top: 0.5rem;", "{semantic_ref_count} refs" }
        }
    }
}


#[component]
pub fn ComparativeWorkbenchView(model: ComparativeWorkbenchReadModel) -> Element {
    let before_nodes = model.topology.before.nodes.len();
    let before_edges = model.topology.before.edges.len();
    let after_nodes = model.topology.after.nodes.len();
    let after_edges = model.topology.after.edges.len();
    let delta_nodes = model.topology.delta.nodes.len();
    let delta_edges = model.topology.delta.edges.len();

    rsx! {
        section {
            style: "margin-top: 2rem;",
            header {
                h2 { "Comparative Workbench" }
                p {
                    "Before / After / Δ · same canonical semantic object identity"
                }
                p {
                    style: "font-size: 0.9rem; opacity: 0.75;",
                    "Candidate-only · no winner selection · no outcome prediction · no truth promotion"
                }
            }

            dl {
                style: "display: grid; grid-template-columns: max-content 1fr; gap: 0.25rem 0.75rem;",
                dt { "Left" }
                dd { "{model.selectors.left_ref}" }
                dt { "Right" }
                dd { "{model.selectors.right_ref}" }
                if let Some(query_ref) = model.selectors.query_ref.as_ref() {
                    dt { "Query" }
                    dd { "{query_ref}" }
                }
                if let Some(consumer_ref) = model.selectors.consumer_ref.as_ref() {
                    dt { "Consumer" }
                    dd { "{consumer_ref}" }
                }
                if let Some(as_at_ref) = model.selectors.as_at_ref.as_ref() {
                    dt { "As-at" }
                    dd { "{as_at_ref}" }
                }
                if let Some(scope_ref) = model.selectors.scope_ref.as_ref() {
                    dt { "Scope" }
                    dd { "{scope_ref}" }
                }
            }

            div {
                style: "display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 0.75rem; margin-top: 1rem;",
                ComparativePanelSummary {
                    title: "Before",
                    graph_ref: model.topology.before.graph_ref.clone(),
                    node_count: before_nodes,
                    edge_count: before_edges,
                }
                ComparativePanelSummary {
                    title: "After",
                    graph_ref: model.topology.after.graph_ref.clone(),
                    node_count: after_nodes,
                    edge_count: after_edges,
                }
                ComparativePanelSummary {
                    title: "Δ",
                    graph_ref: model.topology.delta.graph_ref.clone(),
                    node_count: delta_nodes,
                    edge_count: delta_edges,
                }
            }

            section {
                style: "margin-top: 1rem;",
                h3 { "Answer-changing distinctions" }
                if model
                    .explanation_overlay
                    .answer_changing_semantic_refs
                    .is_empty()
                {
                    p { "No typed answer-changing semantic object is declared for this comparison." }
                } else {
                    ul {
                        for semantic_ref in model
                            .explanation_overlay
                            .answer_changing_semantic_refs
                            .iter()
                        {
                            li {
                                strong { "{semantic_ref}" }
                                if let Some(annotation) = model
                                    .explanation_overlay
                                    .typed_change_annotations
                                    .get(semantic_ref)
                                {
                                    span { " · layer={annotation.layer:?}" }
                                    if let Some(reason) = annotation.explanation_ref.as_ref() {
                                        div {
                                            style: "font-size: 0.9rem; opacity: 0.8;",
                                            "{reason}"
                                        }
                                    }
                                    if !annotation.justification_refs.is_empty() {
                                        div {
                                            style: "font-size: 0.8rem; opacity: 0.7;",
                                            "justification: {annotation.justification_refs.join(", ")}"
                                        }
                                    }
                                } else {
                                    if let Some(layer) = model
                                        .explanation_overlay
                                        .change_layer_by_semantic_ref
                                        .get(semantic_ref)
                                    {
                                        span { " · layer={layer}" }
                                    }
                                    if let Some(reason) = model
                                        .explanation_overlay
                                        .explanation_by_semantic_ref
                                        .get(semantic_ref)
                                    {
                                        div {
                                            style: "font-size: 0.9rem; opacity: 0.8;",
                                            "{reason}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !model.explanation_overlay.unresolved_semantic_refs.is_empty() {
                section {
                    style: "margin-top: 1rem;",
                    h3 { "Unresolved comparative items" }
                    ul {
                        for semantic_ref in model
                            .explanation_overlay
                            .unresolved_semantic_refs
                            .iter()
                        {
                            li { "{semantic_ref}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ComparativePanelSummary(
    title: &'static str,
    graph_ref: String,
    node_count: usize,
    edge_count: usize,
) -> Element {
    rsx! {
        article {
            style: "border: 1px solid #aaa; border-radius: 0.6rem; padding: 0.8rem;",
            strong { "{title}" }
            div {
                style: "font-size: 0.8rem; margin-top: 0.4rem; overflow-wrap: anywhere;",
                "{graph_ref}"
            }
            div { "{node_count} nodes · {edge_count} edges" }
        }
    }
}
