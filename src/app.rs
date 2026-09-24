use dioxus::prelude::*;

#[cfg(feature = "production-data")]
use sensiblaw_reader_model::{SemanticTracePath, TraceReviewState};

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
                                            "justification:"
                                            for justification_ref in annotation.justification_refs.iter() {
                                                span { " {justification_ref}" }
                                            }
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


#[cfg(feature = "production-data")]
#[component]
pub fn SemanticTraceInspectorView(traces: Vec<SemanticTracePath>) -> Element {
    rsx! {
        section {
            style: "margin-top: 2rem;",
            header {
                h2 { "Source / Statement Trace" }
                p {
                    "Read-only provenance traversal · event → observation → parse → statement → exact source"
                }
                p {
                    style: "font-size: 0.9rem; opacity: 0.75;",
                    "This view creates no evidence payment, semantic authority, applicability, or claim truth."
                }
            }

            if traces.is_empty() {
                p { "No persisted semantic trace is available for the selected event." }
            } else {
                for trace in traces.iter() {
                    SemanticTraceCard { trace: trace.clone() }
                }
            }
        }
    }
}

#[cfg(feature = "production-data")]
#[component]
fn SemanticTraceCard(trace: SemanticTracePath) -> Element {
    let review_label = trace
        .parse
        .as_ref()
        .map(|parse| match parse.review_state {
            TraceReviewState::Unreviewed => "unreviewed",
            TraceReviewState::ParseReviewed => "parse reviewed",
            TraceReviewState::SemanticallyAdmitted => "semantically admitted",
            TraceReviewState::Rejected => "rejected",
            TraceReviewState::Abstained => "abstained",
            TraceReviewState::Qualified => "qualified",
        })
        .unwrap_or("no parse coordinate");

    rsx! {
        article {
            style: "border: 1px solid #aaa; border-radius: 0.6rem; padding: 1rem; margin-top: 0.75rem;",
            dl {
                style: "display: grid; grid-template-columns: max-content minmax(0, 1fr); gap: 0.3rem 0.8rem;",
                dt { "Event" }
                dd { "{trace.event_ref.as_deref().unwrap_or("—")}" }
                dt { "Observation" }
                dd { "{trace.observation_ref}" }
                if let Some(parse) = trace.parse.as_ref() {
                    dt { "PNF candidate" }
                    dd { "{parse.candidate_pnf_ref}" }
                    dt { "Review" }
                    dd { "{review_label}" }
                    if let Some(review_ref) = parse.review_ref.as_ref() {
                        dt { "Review receipt" }
                        dd { "{review_ref}" }
                    }
                    if let Some(admission_ref) = parse.admission_receipt_ref.as_ref() {
                        dt { "Admission receipt" }
                        dd { "{admission_ref}" }
                    }
                }
                dt { "Statement" }
                dd { "{trace.statement.statement_ref}" }
                dt { "Exact span" }
                dd { "{trace.statement.span_ref}" }
                dt { "Source revision" }
                dd { "{trace.statement.source_revision_ref}" }
            }
            blockquote {
                style: "margin: 1rem 0 0; padding: 0.75rem; border-left: 3px solid #aaa; white-space: pre-wrap;",
                "{trace.statement.literal_text}"
            }
            if !trace.claim_refs.is_empty() {
                div {
                    style: "margin-top: 0.75rem; font-size: 0.9rem;",
                    strong { "Claims: " }
                    for claim_ref in trace.claim_refs.iter() {
                        span { "{claim_ref} " }
                    }
                }
            }
            if !trace.downstream_use_refs.is_empty() {
                div {
                    style: "margin-top: 0.5rem; font-size: 0.9rem;",
                    strong { "Downstream uses: " }
                    for use_ref in trace.downstream_use_refs.iter() {
                        span { "{use_ref} " }
                    }
                }
            }
        }
    }
}
