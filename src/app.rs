use dioxus::prelude::*;

#[cfg(feature = "production-data")]
use sensiblaw_reader_model::{
    ChronologyPlacementKind, PropositionContestationView, SemanticTracePath, TraceReviewState,
};
#[cfg(feature = "production-data")]
use sensiblaw_pg_source_store::{ReviewAction, ReviewItem};

use crate::workbench::{
    comparative::ComparativeWorkbenchReadModel, wave5_personal_handoff_read_model,
    StageAvailability, UnifiedWorkbenchReadModel, WorkbenchStageKind,
};

pub fn app() -> Element {
    // WIKI-UI-1 is an *explicit* diagnostic selector, not a crawl or
    // unattended Wikidata editing interface.
    #[cfg(feature="production-data")]
    if let Ok(reference)=std::env::var("ITIR_WIKI_DIAGNOSTIC_REF") {
        return match crate::workbench::ontology::load_ontology_case(&reference) {
            Ok(diagnostic)=>rsx! {
                document::Title { "ITIR Wikidata Ontology Diagnostics" }
                main {
                    style:"font-family:sans-serif;max-width:1200px;margin:0 auto;padding:1.5rem;",
                    WikiOntologyDiagnosticView { diagnostic }
                }
            },
            Err(error)=>rsx! {
                document::Title { "Ontology diagnostic unavailable" }
                main {
                    style:"font-family:sans-serif;padding:1.5rem;",
                    h1 {"Ontology diagnostic unavailable"}
                    p {"{error}"}
                    p {"No source, witness or proposed edit is exposed without its current MatterContext scope."}
                }
            },
        }
    }
    // Explicit pair selector: the UI never scans and auto-merges identical
    // phrases across chat/transcripts. Scope controls observer visibility.
    #[cfg(feature = "production-data")]
    if let (Ok(left), Ok(right)) = (
        std::env::var("ITIR_MIXED_LEFT_REV"),
        std::env::var("ITIR_MIXED_RIGHT_REV"),
    ) {
        use sensiblaw_pg_source_store::ContextVisibility;
        let scope=std::env::var("ITIR_MIXED_OBSERVER_SCOPE")
            .unwrap_or_else(|_| "unavailable".into());
        let visibility=match scope.as_str() {
            "available"=>ContextVisibility::Available,
            "excluded"=>ContextVisibility::ExcludedByScope,
            "redacted"=>ContextVisibility::Redacted,
            _=>ContextVisibility::Unavailable,
        };
        let chat_message_ref=std::env::var("ITIR_MIXED_CHAT_MESSAGE_REF").ok();
        let loaded=crate::workbench::mixed_source::load_mixed_source_workspace(
            &left,&right,chat_message_ref.as_deref(),visibility,&scope,
        );
        return match loaded {
            Ok(model)=>rsx! {
                document::Title { "ITIR Mixed-Source Review" }
                main {
                    style:"font-family: sans-serif; max-width: 1220px; margin: 0 auto; padding: 1.5rem;",
                    MixedSourceDualLensView { model }
                }
            },
            Err(error)=>rsx! {
                document::Title { "ITIR mixed-source data unavailable" }
                main {
                    style:"font-family: sans-serif; max-width: 1000px; margin: 0 auto; padding: 2rem;",
                    h1 {"Mixed-source review unavailable"}
                    p {"{error}"}
                    p {"No semantic relationship or operational activity is inferred from unavailable source records."}
                }
            },
        };
    }

    #[cfg(feature = "production-data")]
    if let Ok(scope_path) = std::env::var("SENSIBLAW_MATTER_SCOPE") {
        let loaded = crate::workbench::matter_scope::load_matter_scope_manifest(&scope_path)
            .and_then(crate::workbench::matter::load_generic_matter_workspace);
        return match loaded {
            Ok(model) => rsx! {
                document::Title { "SensibLaw Matter" }
                main {
                    style: "font-family: sans-serif; max-width: 1180px; margin: 0 auto; padding: 2rem;",
                    crate::matter_ui::GenericMatterWorkspaceView { model }
                }
            },
            Err(error) => rsx! {
                document::Title { "SensibLaw Matter · load error" }
                main {
                    style: "font-family: sans-serif; max-width: 900px; margin: 0 auto; padding: 2rem;",
                    h1 { "Matter could not be opened" }
                    p { "{error}" }
                    p {
                        style: "font-size: 0.85rem; opacity: 0.75;",
                        "The existing canonical world is unchanged. Fix the explicit scope/context or persisted-data receipt and reload."
                    }
                }
            },
        };
    }

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
                        let observation_count = entry.observation_refs.len();
                        let statement_count = entry.statement_refs.len();
                        let claim_count = entry.claim_refs.len();
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
                                    div { "{observation_count} observations" }
                                    div { "{statement_count} statements" }
                                    div { "{claim_count} claims" }
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
                                        let statement_count = leaf.statement_refs.len();
                                        let observation_count = leaf.observation_refs.len();
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
                                                    "{statement_count} statements · {observation_count} observations"
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


#[cfg(feature = "production-data")]
#[component]
pub fn ReviewWorkspaceView(
    model: crate::workbench::review::ProductionReviewWorkspace,
) -> Element {
    rsx! {
        section {
            style: "margin-top: 2rem;",
            header {
                h2 { "Review" }
                p {
                    "Portable review queue · parse, observation, claim, event, chronology, authority, research, legal treatment and handoff."
                }
                p {
                    style: "font-size: 0.9rem; opacity: 0.75;",
                    "Actions run through the typed SLR reducer and persist a ReviewReceipt. Review state never creates semantic authority, applicability, or claim truth."
                }
            }

            if model.queue.items.is_empty() {
                p { "No persisted review items are currently queued." }
            } else {
                for item in model.queue.items.iter() {
                    LiveReviewItemCard { item: item.clone() }
                }
            }
        }
    }
}

#[cfg(feature = "production-data")]
#[component]
fn LiveReviewItemCard(item: ReviewItem) -> Element {
    let mut live_item = use_signal(|| item);
    let mut reviewer_ref = use_signal(|| "operator:local".to_owned());
    let mut qualification_ref = use_signal(String::new);
    let mut evidence_request_ref = use_signal(String::new);
    let mut action_result = use_signal(|| None::<String>);

    let snapshot = live_item.read().clone();
    let kind = format!("{:?}", snapshot.item_kind);
    let status = format!("{:?}", snapshot.current_status);
    let provenance_count = snapshot.provenance_refs.len();
    let source_count = snapshot.source_refs.len();
    let consumer_count = snapshot.affected_consumer_refs.len();

    rsx! {
        article {
            style: "border: 1px solid #aaa; border-radius: 0.6rem; padding: 1rem; margin-top: 0.75rem;",
            header {
                strong { "{kind}" }
                span { " · {status}" }
                div {
                    style: "font-size: 0.8rem; opacity: 0.7; overflow-wrap: anywhere;",
                    "{snapshot.review_item_ref}"
                }
            }

            p { "{snapshot.reason}" }

            dl {
                style: "display: grid; grid-template-columns: max-content minmax(0, 1fr); gap: 0.25rem 0.75rem;",
                dt { "Semantic object" }
                dd { "{snapshot.semantic_ref}" }
                dt { "Provenance" }
                dd { "{provenance_count} refs" }
                dt { "Sources" }
                dd { "{source_count} refs" }
                dt { "Affected consumers" }
                dd { "{consumer_count} refs" }
            }

            if !snapshot.source_refs.is_empty() {
                details {
                    style: "margin-top: 0.6rem;",
                    summary { "Source refs" }
                    ul {
                        for source_ref in snapshot.source_refs.iter() {
                            li { "{source_ref}" }
                        }
                    }
                }
            }

            if !snapshot.provenance_refs.is_empty() {
                details {
                    style: "margin-top: 0.4rem;",
                    summary { "Provenance refs" }
                    ul {
                        for provenance_ref in snapshot.provenance_refs.iter() {
                            li { "{provenance_ref}" }
                        }
                    }
                }
            }

            if !snapshot.affected_consumer_refs.is_empty() {
                details {
                    style: "margin-top: 0.4rem;",
                    summary { "Affected consumers" }
                    ul {
                        for consumer_ref in snapshot.affected_consumer_refs.iter() {
                            li { "{consumer_ref}" }
                        }
                    }
                }
            }

            section {
                style: "margin-top: 0.8rem; border-top: 1px solid #ddd; padding-top: 0.8rem;",
                strong { "Live typed actions" }

                label {
                    style: "display: block; margin-top: 0.6rem; font-size: 0.85rem;",
                    "Reviewer ref"
                    input {
                        style: "display: block; width: min(100%, 32rem); margin-top: 0.2rem;",
                        value: reviewer_ref(),
                        oninput: move |event| reviewer_ref.set(event.value())
                    }
                }

                if snapshot.available_actions.contains(&ReviewAction::Qualify) {
                    label {
                        style: "display: block; margin-top: 0.6rem; font-size: 0.85rem;",
                        "Qualification ref / note"
                        input {
                            style: "display: block; width: min(100%, 40rem); margin-top: 0.2rem;",
                            placeholder: "required for Qualify",
                            oninput: move |event| qualification_ref.set(event.value())
                        }
                    }
                }

                if snapshot.available_actions.contains(&ReviewAction::RequestEvidence) {
                    label {
                        style: "display: block; margin-top: 0.6rem; font-size: 0.85rem;",
                        "Evidence request ref / note"
                        input {
                            style: "display: block; width: min(100%, 40rem); margin-top: 0.2rem;",
                            placeholder: "required for Request Evidence",
                            oninput: move |event| evidence_request_ref.set(event.value())
                        }
                    }
                }

                div {
                    style: "display: flex; flex-wrap: wrap; gap: 0.4rem; margin-top: 0.7rem;",
                    for action in snapshot.available_actions.iter().copied() {
                        {
                            let action_label = format!("{:?}", action);
                            let result_action_label = action_label.clone();
                            let review_item_ref = snapshot.review_item_ref.clone();
                            rsx! {
                                button {
                                    style: "border: 1px solid #888; border-radius: 999px; padding: 0.35rem 0.7rem; cursor: pointer;",
                                    onclick: move |_| {
                                        let reviewer = reviewer_ref.read().trim().to_owned();
                                        let qualification = qualification_ref.read().trim().to_owned();
                                        let evidence = evidence_request_ref.read().trim().to_owned();

                                        let qualification = if action == ReviewAction::Qualify {
                                            if qualification.is_empty() {
                                                action_result.set(Some(
                                                    "Qualify requires an explicit qualification ref / note.".into()
                                                ));
                                                return;
                                            }
                                            Some(qualification)
                                        } else {
                                            None
                                        };

                                        let evidence_request = if action == ReviewAction::RequestEvidence {
                                            if evidence.is_empty() {
                                                action_result.set(Some(
                                                    "RequestEvidence requires an explicit evidence-request ref / note.".into()
                                                ));
                                                return;
                                            }
                                            Some(evidence)
                                        } else {
                                            None
                                        };

                                        match crate::workbench::review::execute_review_action(
                                            &review_item_ref,
                                            action,
                                            &reviewer,
                                            qualification,
                                            evidence_request,
                                        ) {
                                            Ok((receipt, persisted)) => {
                                                let effect = format!("{:?}", receipt.effect);
                                                let new_status = format!("{:?}", persisted.current_status);
                                                live_item.set(persisted);
                                                action_result.set(Some(format!(
                                                    "Persisted {result_action_label}: effect={effect}; status={new_status}"
                                                )));
                                            }
                                            Err(error) => {
                                                action_result.set(Some(format!(
                                                    "Review action failed: {error}"
                                                )));
                                            }
                                        }
                                    },
                                    "{action_label}"
                                }
                            }
                        }
                    }
                }

                if let Some(result) = action_result.read().as_ref() {
                    p {
                        style: "font-size: 0.85rem; margin-top: 0.6rem;",
                        "{result}"
                    }
                }

                p {
                    style: "font-size: 0.78rem; opacity: 0.7;",
                    "OpenSource and FollowAuthority are receipted navigation requests and preserve review status. RequestEvidence moves to NeedsEvidence; none of these actions promotes claim truth."
                }
            }
        }
    }
}


#[cfg(feature = "production-data")]
#[component]
pub fn GwbMatterWorkspaceView(
    model: crate::workbench::gwb_matter::GwbMatterWorkspace,
) -> Element {
    let event_count = model.event_refs.len();
    let source_family_count = model.source_family_refs.len();
    let research_count = model.research_review_item_refs.len();
    let claim_count = model
        .timeline
        .chronology
        .proposition_views
        .iter()
        .map(|view| view.leaves.len())
        .sum::<usize>();
    let review_count = model.review.queue.items.len();

    rsx! {
        section {
            style: "margin-top: 2rem;",
            header {
                h1 { "Matter: George W. Bush corpus" }
                div {
                    style: "font-size: 0.85rem; opacity: 0.75; overflow-wrap: anywhere;",
                    "{model.matter_ref}"
                }
                p {
                    "One matter-scoped chronology over heterogeneous reviewed sources; accounts remain separately inspectable."
                }
                p {
                    style: "font-size: 0.9rem; opacity: 0.75;",
                    "Read projection only · no source, event, claim, applicability, or truth promotion"
                }
            }

            nav {
                style: "display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 0.6rem; margin-top: 1rem;",
                GwbMatterNavCard { title: "Sources", count: source_family_count }
                GwbMatterNavCard { title: "Timeline", count: event_count }
                GwbMatterNavCard { title: "Facts / Claims", count: claim_count }
                GwbMatterNavCard { title: "Review", count: review_count }
                GwbMatterNavCard { title: "Research", count: research_count }
            }

            section {
                style: "margin-top: 2rem;",
                h2 { "Sources" }
                p {
                    "{model.source_statement_count} reviewed statement selections across {source_family_count} source families"
                }
                div {
                    style: "font-size: 0.8rem; opacity: 0.75; overflow-wrap: anywhere;",
                    "Handoff: {model.handoff_ref}"
                }
                ul {
                    for family in model.source_family_refs.iter() {
                        li { "{family}" }
                    }
                }
            }

            TimelineWorkspaceView { model: model.timeline.clone() }

            ReviewWorkspaceView { model: model.review.clone() }

            section {
                style: "margin-top: 2rem;",
                h2 { "Research" }
                if model.research_review_item_refs.is_empty() {
                    p {
                        "No GWB research-acquisition / authority-follow review item is currently scoped to this matter."
                    }
                } else {
                    ul {
                        for review_ref in model.research_review_item_refs.iter() {
                            li { "{review_ref}" }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "production-data")]
#[component]
fn GwbMatterNavCard(title: &'static str, count: usize) -> Element {
    rsx! {
        div {
            style: "border: 1px solid #aaa; border-radius: 0.6rem; padding: 0.7rem;",
            strong { "{title}" }
            div {
                style: "font-size: 0.85rem; margin-top: 0.25rem;",
                "{count} items"
            }
        }
    }
}


#[cfg(feature = "production-data")]
#[component]
pub fn EventDiscoveryWorkspaceView(
    model: crate::workbench::event_discovery::ProductionEventDiscoveryWorkspace,
) -> Element {
    rsx! {
        section {
            style: "margin-top: 2rem;",
            header {
                h2 { "Suggested Event Joins" }
                p {
                    "Automatic candidate discovery over source-bound observations. Suggestions require EventAssembly review before any observation → event identity is materialised."
                }
                p {
                    style: "font-size: 0.9rem; opacity: 0.75;",
                    "Same QID alone is insufficient · candidate-only · no semantic authority or truth promotion"
                }
            }

            if model.projection.proposals.is_empty() {
                p { "No candidate event joins currently meet the configured discovery threshold." }
            } else {
                for view in model.projection.proposals.iter() {
                    article {
                        style: "border: 1px solid #aaa; border-radius: 0.6rem; padding: 1rem; margin-top: 0.75rem;",
                        header {
                            strong { "Candidate event join" }
                            span { " · {view.signal_kind_count} signal kinds" }
                            div {
                                style: "font-size: 0.8rem; opacity: 0.7; overflow-wrap: anywhere;",
                                "{view.proposal.proposal_ref}"
                            }
                        }

                        div {
                            style: "display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 0.5rem; margin-top: 0.75rem;",
                            div { "{view.observation_count} observations" }
                            div { "{view.source_family_count} source families" }
                            div { "review required" }
                        }

                        details {
                            style: "margin-top: 0.75rem;",
                            summary { "Source observations" }
                            ul {
                                for observation_ref in view.proposal.observation_refs.iter() {
                                    li { "{observation_ref}" }
                                }
                            }
                        }

                        details {
                            style: "margin-top: 0.4rem;",
                            summary { "Why this was suggested" }
                            ul {
                                for signal in view.proposal.signals.iter() {
                                    {
                                        let kind = format!("{:?}", signal.kind);
                                        rsx! {
                                            li {
                                                strong { "{kind}" }
                                                span { " · {signal.evidence_ref}" }
                                                div {
                                                    style: "font-size: 0.8rem; opacity: 0.7;",
                                                    "detector: {signal.detector_ref}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if !view.proposal.statement_refs.is_empty() {
                            details {
                                style: "margin-top: 0.4rem;",
                                summary { "Statement ancestry" }
                                ul {
                                    for statement_ref in view.proposal.statement_refs.iter() {
                                        li { "{statement_ref}" }
                                    }
                                }
                            }
                        }

                        p {
                            style: "font-size: 0.85rem; opacity: 0.75;",
                            "This proposal is not an event. Accept/reject/qualify it through the ordinary Review workspace."
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "production-data")]
#[component]
pub fn OperationalTimelineWorkspaceView(
    model: crate::workbench::operational_timeline::ProductionOperationalTimelineWorkspace,
) -> Element {
    rsx! {
        section {
            style: "margin-top: 2rem;",
            header {
                h2 { "Work / Activity Timeline" }
                p {
                    "Producer-owned StatiBaker operational history: what the operator/system was doing, kept distinct from world/matter events."
                }
                p {
                    style: "font-size: 0.9rem; opacity: 0.75;",
                    "OperationalEvent ≠ SemanticEvent · opened source ≠ evidence payment · tool use ≠ endorsement"
                }
                div {
                    style: "font-size: 0.85rem; opacity: 0.7;",
                    "State date: {model.state_date}"
                }
            }

            if model.timeline.entries.is_empty() {
                p { "No imported operational events are available for this state date." }
            } else {
                for entry in model.timeline.entries.iter() {
                    {
                        let kind = format!("{:?}", entry.event.kind);
                        let link_count = entry.links.len();
                        rsx! {
                            article {
                                style: "border: 1px solid #aaa; border-radius: 0.6rem; padding: 1rem; margin-top: 0.75rem;",
                                header {
                                    strong { "{entry.event.label}" }
                                    span { " · {kind}" }
                                    div {
                                        style: "font-size: 0.8rem; opacity: 0.7; overflow-wrap: anywhere;",
                                        "{entry.event.operational_event_ref}"
                                    }
                                }

                                dl {
                                    style: "display: grid; grid-template-columns: max-content minmax(0, 1fr); gap: 0.25rem 0.75rem; margin-top: 0.75rem;",
                                    dt { "Start" }
                                    dd { "{entry.event.start_time_ref}" }
                                    dt { "End" }
                                    dd { "{entry.event.end_time_ref}" }
                                    dt { "Producer event" }
                                    dd { "{entry.event.producer_event_ref}" }
                                    if let Some(app_ref) = entry.event.primary_app_ref.as_ref() {
                                        dt { "App" }
                                        dd { "{app_ref}" }
                                    }
                                    dt { "Semantic links" }
                                    dd { "{link_count}" }
                                }

                                if !entry.links.is_empty() {
                                    details {
                                        style: "margin-top: 0.75rem;",
                                        summary { "Reviewed semantic/workflow links" }
                                        ul {
                                            for link in entry.links.iter() {
                                                {
                                                    let relation = format!("{:?}", link.relation_kind);
                                                    let target_kind = format!("{:?}", link.target_kind);
                                                    rsx! {
                                                        li {
                                                            strong { "{relation}" }
                                                            span { " · {target_kind}: {link.target_ref}" }
                                                            div {
                                                                style: "font-size: 0.8rem; opacity: 0.7;",
                                                                "receipt: {link.relationship_receipt_ref}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                details {
                                    style: "margin-top: 0.4rem;",
                                    summary { "Operational provenance" }
                                    ul {
                                        for provenance_ref in entry.event.provenance_refs.iter() {
                                            li { "{provenance_ref}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


#[cfg(feature = "production-data")]
#[component]
pub fn ConversationSourceWorkspaceView(
    model: crate::workbench::chat_source::ConversationSourceWorkspace,
) -> Element {
    rsx! {
        section {
            style: "margin-top: 2rem;",
            header {
                h2 { "Conversation / Source" }
                div {
                    style: "font-size: 0.85rem; opacity: 0.7; overflow-wrap: anywhere;",
                    "{model.conversation_ref}"
                }
                p {
                    "Archived message chronology with original node/branch identity. Message identity remains distinct from exact statement subspans and from described world events."
                }
                p {
                    style: "font-size: 0.9rem; opacity: 0.75;",
                    "{model.messages.len()} messages · {model.statement_count} persisted statement spans · {model.inactive_branch_message_count} inactive-branch messages"
                }
            }

            if model.messages.is_empty() {
                p { "No exact-coordinate archived messages are persisted for this conversation." }
            } else {
                for view in model.messages.iter() {
                    {
                        let role = format!("{:?}", view.source.role);
                        let branch = format!("{:?}", view.source.branch_membership);
                        let kind = format!("{:?}", view.source.content_kind);
                        let statement_count = view.statements.len();
                        rsx! {
                            article {
                                style: "border: 1px solid #aaa; border-radius: 0.6rem; padding: 1rem; margin-top: 0.75rem;",
                                header {
                                    strong { "{role}" }
                                    span { " · {branch} · {kind}" }
                                    div {
                                        style: "font-size: 0.8rem; opacity: 0.7; overflow-wrap: anywhere;",
                                        "message {view.source.message_ref}"
                                    }
                                }

                                dl {
                                    style: "display: grid; grid-template-columns: max-content minmax(0, 1fr); gap: 0.25rem 0.75rem; margin-top: 0.6rem;",
                                    dt { "Message time" }
                                    dd { "{view.source.message_time_ref}" }
                                    dt { "Node" }
                                    dd { "{view.source.node_ref}" }
                                    if let Some(parent) = view.source.parent_node_ref.as_ref() {
                                        dt { "Parent" }
                                        dd { "{parent}" }
                                    }
                                    dt { "Statements" }
                                    dd { "{statement_count}" }
                                }

                                details {
                                    style: "margin-top: 0.7rem;",
                                    summary { "Exact archived message text" }
                                    pre {
                                        style: "white-space: pre-wrap; overflow-wrap: anywhere;",
                                        "{view.source.literal_text}"
                                    }
                                }

                                if view.statements.is_empty() {
                                    p {
                                        style: "font-size: 0.85rem; opacity: 0.7;",
                                        "No M12 statement subspans have been materialised from this message."
                                    }
                                } else {
                                    details {
                                        style: "margin-top: 0.6rem;",
                                        summary { "M12 statement subspans" }
                                        for statement in view.statements.iter() {
                                            article {
                                                style: "border-left: 3px solid #aaa; padding-left: 0.7rem; margin-top: 0.6rem;",
                                                div {
                                                    style: "font-size: 0.8rem; opacity: 0.7; overflow-wrap: anywhere;",
                                                    "{statement.statement_ref}"
                                                }
                                                div {
                                                    style: "font-size: 0.8rem; opacity: 0.7; overflow-wrap: anywhere;",
                                                    "span {statement.exact_span_ref}"
                                                }
                                                p { "{statement.literal_text}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            p {
                style: "font-size: 0.82rem; opacity: 0.7; margin-top: 1rem;",
                "Inactive assistant generations are preserved as conversation/decision history; they are not automatically independent evidence about the world."
            }
        }
    }
}

#[cfg(feature = "production-data")]
#[component]
pub fn MixedSourceDualLensView(
    model: crate::workbench::mixed_source::MixedSourceWorkspace,
) -> Element {
    use crate::visual::command::{DomainCommand, VisualObjectId};
    use sensiblaw_pg_source_store::{
        ContextVisibility, GenealogyStatus, SemanticComparison,
    };
    let mut selected = use_signal(|| None::<VisualObjectId>);
    let mut operational_lens = use_signal(|| false);
    let comparison = &model.comparison;
    let semantic_label=match comparison.semantic_comparison {
        SemanticComparison::SharedCandidateFingerprint =>
            "Shared PNF candidate fingerprints — subject identity unreviewed",
        SemanticComparison::NoSharedCandidateFingerprint =>
            "No matching stored L2 fingerprints — not proof of different subjects",
        SemanticComparison::InsufficientPnf =>
            "PNF comparison unavailable or insufficient",
    };
    let genealogy_label=match comparison.genealogy {
        GenealogyStatus::ExactNativeTextCorrespondence =>
            "Native text correspondence; copying remains a separate question",
        GenealogyStatus::SourceBackreferenceUnverified =>
            "Qualified source backreference; derivation not independently certified",
        GenealogyStatus::NoRecordedLineage =>
            "No recorded provenance lineage; absence is not disproof",
    };
    let observer_label=match comparison.operational_visibility {
        ContextVisibility::Available=>"Operational observations available",
        ContextVisibility::NotObserved=>"No linked operational events observed in this scope",
        ContextVisibility::Unavailable=>"Operational records unavailable",
        ContextVisibility::ExcludedByScope=>"Operational records excluded by this scope",
        ContextVisibility::Redacted=>"Operational records redacted",
    };
    let graph=if operational_lens() { &model.operational_graph } else { &model.graph };
    let inspected=selected().and_then(|id| model.inspect(id,operational_lens()));
    rsx! {
        section {
            header {
                h1 {"Mixed-source investigation"}
                p {"Two lenses, separate source authorities, one typed selection vocabulary."}
                p {
                    style:"font-size: 0.875rem; opacity: 0.8;",
                    "Scope: {model.scope_label} · Candidate-only · Review pending · No truth promotion"
                }
            }
            div {
                style:"display: grid; grid-template-columns: repeat(auto-fit, minmax(310px, 1fr)); gap: 1rem;",
                article {
                    style:"border: 1px solid #777; border-radius: 0.7rem; padding: 1rem;",
                    h2 {"Source A"}
                    p {style:"overflow-wrap: anywhere;", "{comparison.left_source_revision_ref}"}
                    p {"Persisted candidate statements: {comparison.left_statement_refs.len()}"}
                    pre {
                        style:"white-space: pre-wrap; overflow-wrap: anywhere; border-radius: .35rem; padding: .65rem; background: #222; color: #eee;",
                        "{comparison.left_source_excerpt}"
                    }
                }
                article {
                    style:"border: 1px solid #777; border-radius: 0.7rem; padding: 1rem;",
                    h2 {"Source B"}
                    p {style:"overflow-wrap: anywhere;", "{comparison.right_source_revision_ref}"}
                    p {"Persisted candidate statements: {comparison.right_statement_refs.len()}"}
                    pre {
                        style:"white-space: pre-wrap; overflow-wrap: anywhere; border-radius: .35rem; padding: .65rem; background: #222; color: #eee;",
                        "{comparison.right_source_excerpt}"
                    }
                }
            }
            section {
                style:"margin-top: 1rem; border: 1px solid #888; border-radius: 0.7rem; padding: 1rem;",
                h2 {"Context and provenance"}
                p {"{semantic_label}"}
                p {"{genealogy_label}"}
                p {"{observer_label}"}
                p {
                    "Independent witnesses established: not assessed. "
                    "Identical text, common candidate factors and temporal proximity are not corroboration."
                }
                if comparison.semantic_review_pending {
                    p {"Semantic relation remains unresolved until reviewed."}
                }
            }
            MixedSourceRelationReviewPanel { model: model.clone() }
            section {
                style:"margin-top: 1rem;",
                h2 {"Source relationship lenses"}
                button {
                    type:"button",
                    onclick:move |_| { operational_lens.set(false); selected.set(None); },
                    "PNF candidate comparison"
                }
                button {
                    type:"button",
                    style:"margin-left: .5rem;",
                    onclick:move |_| { operational_lens.set(true); selected.set(None); },
                    "Operational history"
                }
                p {
                    style:"font-size: 0.875rem; opacity: 0.8;",
                    "Graph nodes are source-linked proposals. Selecting an object does not change review state."
                }
                div {
                    style:"display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: .5rem;",
                    for node in graph.nodes.iter() {
                        button {
                            key:"{node.semantic_ref}",
                            type:"button",
                            style:"text-align: left; border: 1px solid #888; border-radius: .4rem; padding: .7rem; background: transparent; color: inherit;",
                            onclick: {
                                let id=node.id;
                                move |_| {
                                    let command=crate::visual::command::decode_shell(
                                        crate::visual::command::ShellInput::Select(id)
                                    );
                                    if let DomainCommand::SelectObject(value) = command {
                                        selected.set(Some(value));
                                    }
                                }
                            },
                            strong {"{node.label}"}
                            div {style:"font-size: .8rem; opacity: .8;", "{node.kind}"}
                        }
                    }
                }
                if let Some(inspection)=inspected {
                    article {
                        style:"border-left: 4px solid #4787c9; padding: .7rem 1rem; margin-top: 1rem;",
                        h3 {"Selected source object"}
                        p {style:"overflow-wrap: anywhere;", "{inspection.semantic_ref}"}
                        h4 {"Source refs"}
                        for source_ref in inspection.source_refs.iter() {
                            p {style:"overflow-wrap: anywhere;", "{source_ref}"}
                        }
                        h4 {"Provenance refs"}
                        if inspection.provenance_refs.is_empty() {
                            p {"No additional attached provenance in this read projection."}
                        }
                        for provenance_ref in inspection.provenance_refs.iter() {
                            p {style:"overflow-wrap: anywhere;", "{provenance_ref}"}
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "production-data")]
#[component]
fn MixedSourceRelationReviewPanel(
    model: crate::workbench::mixed_source::MixedSourceWorkspace,
) -> Element {
    use sensiblaw_pg_source_store::{CorrespondenceAxis,ReviewAction};
    use crate::workbench::correspondence_review::CorrespondenceReviewWorkspace;

    let mut live_review=use_signal(||None::<CorrespondenceReviewWorkspace>);
    let mut review_result=use_signal(||None::<String>);
    let mut reviewer_ref=use_signal(||String::new());
    let mut qualification_ref=use_signal(String::new);
    let mut evidence_request_ref=use_signal(String::new);

    // The workspace's *viewer* scope is not automatically a permission to
    // review or disclose. Review admission requires an explicit consumer scope.
    let consumer_scope=std::env::var("ITIR_MIXED_CONSUMER_SCOPE")
        .ok().filter(|s|!s.trim().is_empty());
    let chat_message_ref=std::env::var("ITIR_MIXED_CHAT_MESSAGE_REF").ok();
    let matter_scope_loaded=std::env::var("SENSIBLAW_MATTER_SCOPE").is_ok();

    let mut candidates=Vec::new();
    for witness in &model.comparison.shared_entity_candidate_refs {
        candidates.push((
            CorrespondenceAxis::SameSubject,witness.clone(),
            "Explore same-subject relation (PNF entity candidate)".to_owned(),
        ));
    }
    for witness in &model.comparison.shared_event_candidate_refs {
        candidates.push((
            CorrespondenceAxis::SameEvent,witness.clone(),
            "Explore same-event relation (PNF event candidate)".to_owned(),
        ));
    }
    for witness in &model.comparison.native_join_refs {
        candidates.push((
            CorrespondenceAxis::Quotation,witness.clone(),
            "Review possible quotation (native source backreference)".to_owned(),
        ));
        candidates.push((
            CorrespondenceAxis::SourceDependency,witness.clone(),
            "Review possible source dependency (native source backreference)".to_owned(),
        ));
    }

    let snapshot=live_review.read().clone();
    rsx! {
        section {
            style:"margin-top: 1.1rem; padding: 1rem; border: 1px solid #777; border-radius: .65rem;",
            h2 {"Review a proposed correspondence · S29"}
            p {
                "Review is about this proposed relationship, not the truth of either source. "
                "A candidate must have a stored PNF fingerprint or native source-join witness."
            }
            if consumer_scope.is_none() || !matter_scope_loaded {
                p {
                    style:"font-weight: 600;",
                    "Read-only: set ITIR_MIXED_CONSUMER_SCOPE to the actual Matter ref "
                    "and SENSIBLAW_MATTER_SCOPE to the existing scope manifest. "
                    "Both sources must be visible under that MatterContext."
                }
            } else if candidates.is_empty() {
                p {"No witnessed relation candidates are available to admit for review."}
            } else {
                div {
                    style:"display: flex; flex-wrap: wrap; gap: .5rem;",
                    for (axis,evidence,label) in candidates.iter().cloned() {
                        {
                            let left=model.comparison.left_source_revision_ref.clone();
                            let right=model.comparison.right_source_revision_ref.clone();
                            let chat_ref=chat_message_ref.clone();
                            let scope=consumer_scope.clone().unwrap_or_default();
                            rsx! {
                                button {
                                    key:"{axis:?}:{evidence}",
                                    type:"button",
                                    style:"padding: .55rem .7rem; border: 1px solid #777; border-radius: .35rem; background: transparent; color: inherit; text-align: left;",
                                    onclick:move |_| {
                                        match crate::workbench::correspondence_review::propose_review(
                                            &left,&right,chat_ref.as_deref(),axis,&evidence,&scope
                                        ) {
                                            Ok(reopened)=>{
                                                review_result.set(Some(
                                                    format!("Reopened S29 item: {}",reopened.relation.review_item_ref)
                                                ));
                                                live_review.set(Some(reopened));
                                            },
                                            Err(error)=>review_result.set(Some(format!("Review proposal unavailable: {error}"))),
                                        }
                                    },
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }
            if let Some(value)=review_result.read().as_ref() {
                p {style:"overflow-wrap: anywhere;", "{value}"}
            }
            if let Some(current)=snapshot {
                article {
                    style:"margin-top: 1rem; border-left: 3px solid #4787c9; padding: .75rem;",
                    h3 {"Persisted correspondence review item"}
                    p {style:"overflow-wrap: anywhere;", "{current.relation.relation_ref}"}
                    p {"Axis: {current.relation.axis:?} · S29 status: {current.item.current_status:?}"}
                    p {style:"overflow-wrap: anywhere;", "Evidence: {current.relation.evidence_ref}"}
                    p {style:"overflow-wrap: anywhere;", "Consumer scope: {current.relation.consumer_scope_ref}"}
                    p {"Correspondence review does not create semantic authority, claim truth or evidentiary independence."}
                    label {
                        "Reviewer identity"
                        input {
                            style:"display: block; width: min(100%, 28rem); margin-top: .3rem;",
                            placeholder:"required reviewer ref",
                            value:reviewer_ref(),
                            oninput:move |e| reviewer_ref.set(e.value()),
                        }
                    }
                    label {
                        "Qualification ref (for Qualify)"
                        input {
                            style:"display: block; width: min(100%, 28rem); margin-top: .3rem;",
                            value:qualification_ref(),
                            oninput:move |e| qualification_ref.set(e.value()),
                        }
                    }
                    label {
                        "Requested evidence ref (for RequestEvidence)"
                        input {
                            style:"display: block; width: min(100%, 28rem); margin-top: .3rem;",
                            value:evidence_request_ref(),
                            oninput:move |e| evidence_request_ref.set(e.value()),
                        }
                    }
                    details {
                        style:"margin: .75rem 0;",
                        summary {"Attributable S29 review receipt history ({current.history.len()})"}
                        p {
                            style:"font-size: .85rem; opacity: .7;",
                            "Receipts are listed by command reference, not claimed wall-clock order."
                        }
                        for receipt in current.history.iter() {
                            article {
                                key:"{receipt.command_ref}",
                                style:"border-bottom: 1px solid #555; padding: .45rem;",
                                p {"Reviewer: {receipt.reviewer_ref} · Action: {receipt.action_ref}"}
                                p {"Effect: {receipt.effect_ref}"}
                                if let Some(effect_value)=receipt.effect_value_ref.as_ref() {
                                    p {style:"overflow-wrap: anywhere;", "Value: {effect_value}"}
                                }
                                p {style:"overflow-wrap: anywhere;", "Receipt: {receipt.command_ref}"}
                            }
                        }
                    }
                    div {
                        style:"display: flex; flex-wrap: wrap; gap: .5rem; margin-top: .8rem;",
                        for action in current.item.available_actions.iter().copied() {
                            if action != ReviewAction::OpenSource && action != ReviewAction::FollowAuthority {
                                {
                                    let relation_ref=current.relation.relation_ref.clone();
                                    rsx! {
                                        button {
                                            key:"{action:?}",
                                            type:"button",
                                            onclick:move |_| {
                                                let reviewer=reviewer_ref.read().trim().to_owned();
                                                let qualification=qualification_ref.read().trim().to_owned();
                                                let evidence_request=evidence_request_ref.read().trim().to_owned();
                                                let qualification=(action==ReviewAction::Qualify)
                                                    .then_some(qualification);
                                                let evidence_request=(action==ReviewAction::RequestEvidence)
                                                    .then_some(evidence_request);
                                                match crate::workbench::correspondence_review::execute_relation_review(
                                                    &relation_ref,action,&reviewer,qualification,evidence_request,
                                                ) {
                                                    Ok((receipt,reopened))=>{
                                                        review_result.set(Some(format!(
                                                            "S29 persisted: {} ({:?}); current status {:?}",
                                                            receipt.command_ref,receipt.action,reopened.item.current_status,
                                                        )));
                                                        live_review.set(Some(reopened));
                                                    },
                                                    Err(error)=>review_result.set(Some(format!(
                                                        "S29 action rejected: {error}"
                                                    ))),
                                                }
                                            },
                                            "{action:?}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature="production-data")]
#[component]
fn WikiOntologyDiagnosticView(
    diagnostic:sensiblaw_pg_source_store::OntologyDiagnosticRead,
)->Element {
    use sensiblaw_pg_source_store::ReviewAction;
    use crate::visual::command::{DomainCommand,ShellInput,VisualObjectId,decode_shell};
    let mut selected_graph_object=use_signal(||None::<VisualObjectId>);
    let mut current=use_signal(||diagnostic);
    let mut reviewer_ref=use_signal(String::new);
    let mut qualification_ref=use_signal(String::new);
    let mut evidence_request_ref=use_signal(String::new);
    let mut feedback=use_signal(||None::<String>);
    let data=current.read().clone();
    let graph=crate::workbench::ontology_graph::ontology_diagnostic_graph(&data);
    let inspected=selected_graph_object().and_then(|id|
        crate::workbench::ontology_graph::inspect_ontology_graph(&graph,id));
    let p=&data.packet;
    let view=format!("{:?}",p.graph_view);
    let checker=format!("{:?}",p.checker_kind);
    let disposition=format!("{:?}",p.disposition);
    let status=format!("{:?}",data.s29_status);
    rsx! {
        article {
            header {
                h1 {"Wikidata ontology · finite diagnostic"}
                p {"Graph view: {view} · Checker: {checker} · Outcome: {disposition}"}
                p {
                    style:"opacity:.8;",
                    "Candidate-only. A modeled checker verdict is not a community edit, claim truth or global Wikidata consistency proof."
                }
            }
            section {
                style:"padding:.8rem;border:1px solid #888;border-radius:.5rem;",
                h2 {"Source and execution receipts"}
                p {style:"overflow-wrap:anywhere;", "Native source revision: {p.source_revision_ref}"}
                p {style:"overflow-wrap:anywhere;", "Snapshot digest: {p.source_snapshot_digest_ref}"}
                p {style:"overflow-wrap:anywhere;", "Wikidata entity: {p.wikidata_entity_ref}"}
                p {style:"overflow-wrap:anywhere;", "Graph slice: {p.graph_slice_ref:?}"}
                p {style:"overflow-wrap:anywhere;", "Lean owner: {p.lean_owner_ref}"}
                p {style:"overflow-wrap:anywhere;", "Lean source commit: {p.lean_source_commit}"}
                p {style:"overflow-wrap:anywhere;", "Producer run: {p.producer_run_ref}"}
                p {style:"overflow-wrap:anywhere;", "Execution receipt: {p.producer_receipt_ref}"}
                p {style:"overflow-wrap:anywhere;", "Output digest: {p.producer_output_digest_ref}"}
                p {"Lean kernel check: {p.lean_kernel_checked} · Original author: {p.original_author_ref}"}
                p {style:"overflow-wrap:anywhere;", "DASHI integration: {p.integration_ref}"}
                p {"Reviewer scope: {data.consumer_scope_ref}"}
            }
            section {
                style:"margin-top:1rem;padding:.8rem;border:1px solid #888;border-radius:.5rem;",
                h2 {"Checker/source/witness graph"}
                p {
                    "Typed relationships are producer-reported review objects, not admitted semantic relations. "
                    "Dioxus and wgpu use the same GraphIr/DomainCommand IDs."
                }
                div {
                    style:"display:grid;grid-template-columns:repeat(auto-fit,minmax(250px,1fr));gap:.5rem;",
                    for vertex in graph.nodes.iter() {
                        {
                            let id=vertex.id;
                            rsx! {
                                button {
                                    key:"{vertex.semantic_ref}",
                                    type:"button",
                                    style:"text-align:left;padding:.7rem;border:1px solid #777;border-radius:.4rem;",
                                    onclick:move |_| {
                                        if let DomainCommand::SelectObject(selected)=
                                            decode_shell(ShellInput::Select(id)) {
                                            selected_graph_object.set(Some(selected));
                                        }
                                    },
                                    strong {"{vertex.label}"}
                                    p {style:"overflow-wrap:anywhere;font-size:.8rem;", "{vertex.kind}"}
                                }
                            }
                        }
                    }
                }
                if let Some(selection)=inspected {
                    article {
                        style:"margin-top:.7rem;border-left:4px solid #4787c9;padding:.6rem;",
                        h3 {"Selected graph object"}
                        p {style:"overflow-wrap:anywhere;", "{selection.semantic_ref}"}
                        for source in selection.source_refs.iter() {
                            p {style:"overflow-wrap:anywhere;", "Source: {source}"}
                        }
                        for provenance in selection.provenance_refs.iter() {
                            p {style:"overflow-wrap:anywhere;", "Provenance: {provenance}"}
                        }
                    }
                }
            }
            section {
                style:"margin-top:1rem;",
                h2 {"Rule witnesses · {p.witnesses.len()}"}
                for witness in p.witnesses.iter() {
                    article {
                        key:"{witness.witness_ref}",
                        style:"padding:.8rem;margin-top:.5rem;border:1px solid #888;border-radius:.5rem;",
                        strong {"{witness.rule_ref}"}
                        p {"{witness.explanation}"}
                        p {style:"overflow-wrap:anywhere;", "Witness: {witness.witness_ref}"}
                        h4 {"Exact source statements"}
                        for reference in witness.statement_refs.iter() {
                            p {style:"overflow-wrap:anywhere;", "{reference}"}
                        }
                        if !witness.native_statements.is_empty() {
                            details {
                                summary {"Producer-native statement bundles (qualifiers, ranks, references)"}
                                for statement in witness.native_statements.iter() {
                                    article {
                                        key:"{statement.statement_ref}",
                                        style:"padding:.65rem;border:1px solid #777;border-radius:.4rem;",
                                        p {style:"overflow-wrap:anywhere;", "GUID: {statement.statement_ref}"}
                                        p {style:"overflow-wrap:anywhere;", "Subject: {statement.subject_ref}"}
                                        p {style:"overflow-wrap:anywhere;", "Property: {statement.property_ref}"}
                                        p {style:"overflow-wrap:anywhere;", "Value: {statement.value_ref}"}
                                        p {"Rank: {statement.rank_ref}"}
                                        p {style:"overflow-wrap:anywhere;", "Statement revision: {statement.statement_revision_ref}"}
                                        for qualifier in statement.qualifiers.iter() {
                                            p {style:"overflow-wrap:anywhere;", "Qualifier {qualifier.property_ref}: {qualifier.value_ref}"}
                                        }
                                        for native_reference in statement.reference_refs.iter() {
                                            p {style:"overflow-wrap:anywhere;", "Reference: {native_reference}"}
                                        }
                                    }
                                }
                            }
                        }
                        h4 {"Additional evidence refs"}
                        for evidence in witness.evidence_refs.iter() {
                            p {style:"overflow-wrap:anywhere;", "{evidence}"}
                        }
                    }
                }
                if p.witnesses.is_empty() {
                    p {"No source-backed violation witness in this finite output; this does not certify the world."}
                }
            }
            section {
                h2 {"Unpaid obligations · {p.missing_obligations.len()}"}
                for missing in p.missing_obligations.iter() {
                    p {"{missing}"}
                }
            }
            section {
                h2 {"Advisory repair candidates · {p.repair_candidates.len()}"}
                for candidate in p.repair_candidates.iter() {
                    article {
                        key:"{candidate.candidate_ref}",
                        style:"margin-top:.5rem;padding:.8rem;border:1px solid #888;border-radius:.5rem;",
                        h3 {"{candidate.candidate_ref}"}
                        p {"{candidate.proposed_edit_description}"}
                        p {"{candidate.rationale}"}
                        p {"Modeled verdict: {candidate.modeled_verdict_ref}"}
                        p {style:"overflow-wrap:anywhere;", "Before: {candidate.modeled_before_ref}"}
                        p {style:"overflow-wrap:anywhere;", "After: {candidate.modeled_after_ref}"}
                        p {"Wikidata modification: false · Public edit authority: false"}
                    }
                }
            }
            section {
                style:"margin-top:1rem;padding:1rem;border:1px solid #888;border-radius:.5rem;",
                h2 {"S29 review · {status}"}
                p {"Review concerns the finite diagnostic; accepting it does not authorize a repair or public edit."}
                label {
                    "Reviewer identity"
                    input {
                        value:reviewer_ref(),
                        oninput:move |event|reviewer_ref.set(event.value()),
                    }
                }
                label {
                    "Qualification reference"
                    input {
                        value:qualification_ref(),
                        oninput:move |event|qualification_ref.set(event.value()),
                    }
                }
                label {
                    "Evidence request reference"
                    input {
                        value:evidence_request_ref(),
                        oninput:move |event|evidence_request_ref.set(event.value()),
                    }
                }
                div {
                    style:"display:flex;flex-wrap:wrap;gap:.5rem;margin-top:.7rem;",
                    for action in [
                        ReviewAction::Accept,ReviewAction::Reject,
                        ReviewAction::Abstain,ReviewAction::Qualify,
                        ReviewAction::RequestEvidence,ReviewAction::Supersede,
                    ] {
                        {
                            let diagnostic_ref=data.diagnostic_ref.clone();
                            rsx! {
                                button {
                                    key:"{action:?}",
                                    type:"button",
                                    onclick:move |_| {
                                        let reviewer=reviewer_ref.read().trim().to_owned();
                                        let qual=qualification_ref.read().trim().to_owned();
                                        let request=evidence_request_ref.read().trim().to_owned();
                                        match crate::workbench::ontology::review_ontology_case(
                                            &diagnostic_ref,action,&reviewer,
                                            (action==ReviewAction::Qualify).then_some(qual),
                                            (action==ReviewAction::RequestEvidence).then_some(request),
                                        ) {
                                            Ok((receipt,reopened))=>{
                                                feedback.set(Some(format!(
                                                    "Persisted S29 receipt {} for {:?}",
                                                    receipt.command_ref,receipt.action,
                                                )));
                                                current.set(reopened);
                                            },
                                            Err(error)=>feedback.set(Some(format!(
                                                "S29 action rejected: {error}"
                                            ))),
                                        }
                                    },
                                    "{action:?}"
                                }
                            }
                        }
                    }
                }
                if let Some(message)=feedback.read().as_ref() {
                    p {style:"overflow-wrap:anywhere;", "{message}"}
                }
            }
        }
    }
}
