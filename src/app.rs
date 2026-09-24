use dioxus::prelude::*;

#[cfg(feature = "production-data")]
use sensiblaw_reader_model::{
    ChronologyPlacementKind, PropositionContestationView, SemanticTracePath, TraceReviewState,
};

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
    let event_label = trace.event_ref.as_deref().unwrap_or("—");
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
                dd { "{event_label}" }
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


#[cfg(feature = "production-data")]
fn chronology_placement_label(kind: ChronologyPlacementKind) -> &'static str {
    match kind {
        ChronologyPlacementKind::Exact => "Dated",
        ChronologyPlacementKind::Approximate => "Approximate",
        ChronologyPlacementKind::RelativeOnly => "Relative",
        ChronologyPlacementKind::Undated => "Undated",
        ChronologyPlacementKind::Unknown => "Unknown",
    }
}

#[cfg(feature = "production-data")]
#[component]
pub fn TimelineWorkspaceView(
    model: crate::workbench::timeline::ProductionTimelineWorkspace,
) -> Element {
    rsx! {
        section {
            style: "margin-top: 2rem;",
            header {
                h2 { "Timeline" }
                p {
                    "Reviewed chronology projection · exact, approximate, relative, undated and unknown remain distinct."
                }
                p {
                    style: "font-size: 0.9rem; opacity: 0.75;",
                    "Contestation is shown from typed claim relations; no event date or narrative is inferred by this view."
                }
            }

            if model.chronology.entries.is_empty() {
                p { "No persisted chronology entries are available for this matter selection." }
            } else {
                for entry in model.chronology.entries.iter() {
                    {
                        let placement_label = chronology_placement_label(entry.placement);
                        let contested_label = if entry.has_contestation() {
                            " · contested"
                        } else {
                            ""
                        };
                        let traces = model
                            .traces_by_event
                            .get(&entry.event_ref)
                            .cloned()
                            .unwrap_or_default();
                        rsx! {
                            details {
                                style: "border: 1px solid #aaa; border-radius: 0.6rem; margin-top: 0.75rem; padding: 0.8rem;",
                                summary {
                                    style: "cursor: pointer;",
                                    strong { "{entry.display_coordinate}" }
                                    span { " · {placement_label}{contested_label}" }
                                    div {
                                        style: "font-size: 0.8rem; opacity: 0.7; overflow-wrap: anywhere;",
                                        "{entry.event_ref}"
                                    }
                                }

                                div {
                                    style: "display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 0.5rem; margin-top: 0.8rem;",
                                    div { "{entry.observation_refs.len()} observations" }
                                    div { "{entry.statement_refs.len()} statements" }
                                    div { "{entry.claim_refs.len()} claims" }
                                }

                                if let Some(relative_ref) = entry.relative_event_ref.as_ref() {
                                    p {
                                        style: "font-size: 0.9rem;",
                                        "Relative event: {relative_ref}"
                                    }
                                }

                                if !entry.claim_refs.is_empty() {
                                    div {
                                        style: "margin-top: 0.75rem;",
                                        strong { "Claim leaves" }
                                        ul {
                                            for claim_ref in entry.claim_refs.iter() {
                                                li { "{claim_ref}" }
                                            }
                                        }
                                    }
                                }

                                if !entry.contestation_relation_refs.is_empty() {
                                    div {
                                        style: "margin-top: 0.75rem;",
                                        strong { "Contestation relations" }
                                        ul {
                                            for relation_ref in entry.contestation_relation_refs.iter() {
                                                li { "{relation_ref}" }
                                            }
                                        }
                                    }
                                }

                                if traces.is_empty() {
                                    p {
                                        style: "font-size: 0.9rem; opacity: 0.75;",
                                        "No persisted source trace is available for this event."
                                    }
                                } else {
                                    section {
                                        style: "margin-top: 1rem;",
                                        h4 { "Source trace" }
                                        for trace in traces.iter() {
                                            SemanticTraceCard { trace: trace.clone() }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            FactsClaimsWorkspaceView {
                views: model.chronology.proposition_views.clone()
            }
        }
    }
}

#[cfg(feature = "production-data")]
#[component]
fn FactsClaimsWorkspaceView(views: Vec<PropositionContestationView>) -> Element {
    rsx! {
        section {
            style: "margin-top: 2rem;",
            header {
                h2 { "Facts / Claims" }
                p {
                    "One proposition root may retain multiple distinct accounts, denials and qualifications."
                }
            }

            if views.is_empty() {
                p { "No proposition roots are linked to the selected timeline events." }
            } else {
                for view in views.iter() {
                    article {
                        style: "border: 1px solid #aaa; border-radius: 0.6rem; padding: 1rem; margin-top: 0.75rem;",
                        h3 { "{view.root.label}" }
                        div {
                            style: "font-size: 0.8rem; opacity: 0.7; overflow-wrap: anywhere;",
                            "{view.root.proposition_ref}"
                        }

                        h4 { "Accounts" }
                        if view.leaves.is_empty() {
                            p { "No claim leaves are attached." }
                        } else {
                            ul {
                                for leaf in view.leaves.iter() {
                                    {
                                        let kind = format!("{:?}", leaf.kind);
                                        let review = format!("{:?}", leaf.review_state);
                                        let speaker = leaf.speaker_ref.as_deref().unwrap_or("speaker unknown");
                                        rsx! {
                                            li {
                                                strong { "{kind}" }
                                                span { " · {review} · {speaker}" }
                                                div {
                                                    style: "font-size: 0.8rem; opacity: 0.75; overflow-wrap: anywhere;",
                                                    "{leaf.claim_ref}"
                                                }
                                                div {
                                                    style: "font-size: 0.8rem;",
                                                    "{leaf.statement_refs.len()} statements · {leaf.observation_refs.len()} observations"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        h4 { "Relations" }
                        if view.relations.is_empty() {
                            p { "No typed contestation relation is persisted." }
                        } else {
                            ul {
                                for relation in view.relations.iter() {
                                    {
                                        let kind = format!("{:?}", relation.kind);
                                        rsx! {
                                            li {
                                                strong { "{kind}" }
                                                span {
                                                    " · {relation.from_claim_ref} → {relation.to_claim_ref}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if !view.orphan_relation_refs.is_empty() {
                            div {
                                style: "font-size: 0.85rem; opacity: 0.75;",
                                strong { "Unresolved relation references: " }
                                for relation_ref in view.orphan_relation_refs.iter() {
                                    span { "{relation_ref} " }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
