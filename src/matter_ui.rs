#![cfg(feature = "production-data")]

use std::collections::BTreeSet;

use dioxus::prelude::*;
use sensiblaw_core::{
    matter_context::DisclosureBoundary,
    matter_handoff::{
        preview_minimal_handoff, HandoffRecipientProfile,
        MinimalHandoffPreview, MinimalHandoffSelection,
    },
};

use crate::workbench::matter::GenericMatterWorkspace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatterPanel {
    Sources,
    Timeline,
    FactsClaims,
    SuggestedJoins,
    Review,
    LegalProof,
    Research,
    WorkProduct,
    Handoff,
    Acceptance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimelineMode {
    Event,
    Knowledge,
    Work,
}

#[component]
pub fn GenericMatterWorkspaceView(model: GenericMatterWorkspace) -> Element {
    let mut panel = use_signal(|| MatterPanel::Sources);
    let mut timeline_mode = use_signal(|| TimelineMode::Event);
    let mut selected_ref = use_signal(|| None::<String>);

    let projection = &model.projection;
    let source_count = projection.source_traces.len();
    let event_count = projection.event_timeline.entries.len();
    let knowledge_count = projection.knowledge_timeline.len();
    let operational_count = projection.operational_timeline.entries.len();
    let outstanding_count = projection.operational_outstanding.states.len();
    let claim_count = projection
        .event_timeline
        .proposition_views
        .iter()
        .map(|view| view.leaves.len())
        .sum::<usize>();
    let proposal_count = projection.join_proposals.proposals.len();
    let review_count = projection.review_queue.items.len();

    rsx! {
        section {
            style: "margin-top: 1.5rem;",
            header {
                h1 { "Matter" }
                div {
                    style: "font-size: 0.85rem; opacity: 0.75; overflow-wrap: anywhere;",
                    "{projection.matter_ref}"
                }
                p {
                    "One reviewed world, projected through source, event, knowledge, work, review and disclosure dimensions."
                }
                p {
                    style: "font-size: 0.85rem; opacity: 0.7;",
                    "Context projection only · hidden ≠ false · unshared ≠ absent · work activity ≠ evidence"
                }
            }

            section {
                style: "border: 1px solid #bbb; border-radius: 0.7rem; padding: 0.8rem; margin-top: 1rem;",
                strong { "Active MatterContext" }
                div {
                    style: "display: flex; flex-wrap: wrap; gap: 0.6rem; margin-top: 0.5rem; font-size: 0.85rem;",
                    span { "{projection.context_projection.included_refs.len()} visible refs" }
                    span { "{projection.context_projection.exclusions.len()} exclusions" }
                    span { "{source_count} source traces" }
                    span { "{event_count} event entries" }
                    span { "{knowledge_count} knowledge entries" }
                    span { "{operational_count} work events" }
                    span { "{outstanding_count} carryover/unresolved states" }
                }
                if !projection.context_projection.exclusions.is_empty() {
                    details {
                        style: "margin-top: 0.5rem;",
                        summary { "Context exclusions" }
                        ul {
                            for exclusion in projection.context_projection.exclusions.iter() {
                                li {
                                    "{exclusion.semantic_ref} · {exclusion.reason:?}"
                                }
                            }
                        }
                    }
                }
            }

            nav {
                style: "display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 0.5rem; margin-top: 1rem;",
                MatterPanelButton { label: "Sources", count: source_count, target: MatterPanel::Sources, panel }
                MatterPanelButton { label: "Timeline", count: event_count + knowledge_count + operational_count + outstanding_count, target: MatterPanel::Timeline, panel }
                MatterPanelButton { label: "Facts / Claims", count: claim_count, target: MatterPanel::FactsClaims, panel }
                MatterPanelButton { label: "Suggested Joins", count: proposal_count, target: MatterPanel::SuggestedJoins, panel }
                MatterPanelButton { label: "Review", count: review_count, target: MatterPanel::Review, panel }
                MatterPanelButton { label: "Legal / Proof", count: projection.legal_proof_refs.len(), target: MatterPanel::LegalProof, panel }
                MatterPanelButton { label: "Research", count: projection.research_refs.len(), target: MatterPanel::Research, panel }
                MatterPanelButton { label: "Work Product", count: projection.work_product_refs.len(), target: MatterPanel::WorkProduct, panel }
                MatterPanelButton { label: "Handoff", count: projection.handoff_refs.len(), target: MatterPanel::Handoff, panel }
                MatterPanelButton {
                    label: "Acceptance",
                    count: model.acceptance.missing_date_event_refs.len()
                        + model.acceptance.missing_actor_claim_refs.len()
                        + model.acceptance.contradictory_relation_refs.len()
                        + model.acceptance.no_event_refs.len()
                        + model.acceptance.procedural_significance_review_refs.len()
                        + model.acceptance.operational_carryover_refs.len(),
                    target: MatterPanel::Acceptance,
                    panel
                }
            }

            if let Some(reference) = selected_ref.read().as_ref() {
                div {
                    style: "position: sticky; top: 0; z-index: 2; margin-top: 0.8rem; padding: 0.55rem 0.75rem; border: 1px solid #999; border-radius: 0.5rem; background: Canvas;",
                    strong { "Selected semantic coordinate: " }
                    span { "{reference}" }
                    button {
                        style: "margin-left: 0.75rem;",
                        onclick: move |_| selected_ref.set(None),
                        "Clear"
                    }
                }
            }

            match *panel.read() {
                MatterPanel::Sources => rsx! {
                    MatterSourcesPanel {
                        model: model.clone(),
                        selected_ref
                    }
                },
                MatterPanel::Timeline => rsx! {
                    MatterTimelinePanel {
                        model: model.clone(),
                        selected_ref,
                        timeline_mode
                    }
                },
                MatterPanel::FactsClaims => rsx! {
                    MatterClaimsPanel {
                        model: model.clone(),
                        selected_ref
                    }
                },
                MatterPanel::SuggestedJoins => rsx! {
                    MatterSuggestedJoinsPanel {
                        model: model.clone(),
                        selected_ref
                    }
                },
                MatterPanel::Review => rsx! {
                    MatterReviewPanel {
                        model: model.clone(),
                        selected_ref
                    }
                },
                MatterPanel::LegalProof => rsx! {
                    MatterRefListPanel {
                        title: "Legal / Proof",
                        refs: projection.legal_proof_refs.clone(),
                        selected_ref
                    }
                },
                MatterPanel::Research => rsx! {
                    MatterRefListPanel {
                        title: "Research",
                        refs: projection.research_refs.clone(),
                        selected_ref
                    }
                },
                MatterPanel::WorkProduct => rsx! {
                    MatterRefListPanel {
                        title: "Work Product",
                        refs: projection.work_product_refs.clone(),
                        selected_ref
                    }
                },
                MatterPanel::Handoff => rsx! {
                    MatterHandoffPanel {
                        model: model.clone(),
                        selected_ref
                    }
                },
                MatterPanel::Acceptance => rsx! {
                    MatterAcceptancePanel {
                        model: model.clone(),
                        selected_ref
                    }
                },
            }
        }
    }
}

#[component]
fn MatterPanelButton(
    label: &'static str,
    count: usize,
    target: MatterPanel,
    mut panel: Signal<MatterPanel>,
) -> Element {
    rsx! {
        button {
            style: "border: 1px solid #999; border-radius: 0.55rem; padding: 0.65rem; text-align: left; cursor: pointer;",
            onclick: move |_| panel.set(target),
            strong { "{label}" }
            div { style: "font-size: 0.8rem; opacity: 0.7;", "{count} items" }
        }
    }
}

#[component]
fn SemanticRefButton(
    reference: String,
    mut selected_ref: Signal<Option<String>>,
) -> Element {
    let label = reference.clone();
    rsx! {
        button {
            style: "border: 0; background: transparent; padding: 0; text-decoration: underline; cursor: pointer; overflow-wrap: anywhere; text-align: left;",
            onclick: move |_| selected_ref.set(Some(reference.clone())),
            "{label}"
        }
    }
}

#[component]
fn MatterSourcesPanel(
    model: GenericMatterWorkspace,
    selected_ref: Signal<Option<String>>,
) -> Element {
    let traces = &model.projection.source_traces;
    rsx! {
        section {
            style: "margin-top: 1rem;",
            h2 { "Sources" }
            p { "Exact source ancestry remains reversible from statement/observation/event coordinates." }
            if traces.is_empty() {
                p { "No source traces are visible under the active MatterContext." }
            } else {
                for trace in traces.iter() {
                    article {
                        style: "border: 1px solid #bbb; border-radius: 0.6rem; padding: 0.8rem; margin-top: 0.6rem;",
                        div {
                            strong { "Focus: " }
                            SemanticRefButton { reference: trace.focus_ref.clone(), selected_ref }
                        }
                        if let Some(event_ref) = trace.event_ref.as_ref() {
                            div {
                                strong { "Event: " }
                                SemanticRefButton { reference: event_ref.clone(), selected_ref }
                            }
                        }
                        div {
                            strong { "Observation: " }
                            SemanticRefButton { reference: trace.observation_ref.clone(), selected_ref }
                        }
                        div {
                            strong { "Statement: " }
                            SemanticRefButton { reference: trace.statement.statement_ref.clone(), selected_ref }
                        }
                        div {
                            strong { "Source revision: " }
                            SemanticRefButton { reference: trace.statement.source_revision_ref.clone(), selected_ref }
                        }
                        blockquote {
                            style: "margin: 0.7rem 0 0; padding-left: 0.7rem; border-left: 3px solid #aaa; white-space: pre-wrap;",
                            "{trace.statement.literal_text}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MatterTimelinePanel(
    model: GenericMatterWorkspace,
    selected_ref: Signal<Option<String>>,
    mut timeline_mode: Signal<TimelineMode>,
) -> Element {
    rsx! {
        section {
            style: "margin-top: 1rem;",
            h2 { "Timeline" }
            p { "Three linked temporal projections; no universal timeline is manufactured." }
            div {
                style: "display: flex; gap: 0.5rem; flex-wrap: wrap;",
                button { onclick: move |_| timeline_mode.set(TimelineMode::Event), "Event" }
                button { onclick: move |_| timeline_mode.set(TimelineMode::Knowledge), "Source / Knowledge" }
                button { onclick: move |_| timeline_mode.set(TimelineMode::Work), "Work / Activity" }
            }

            match *timeline_mode.read() {
                TimelineMode::Event => rsx! {
                    for entry in model.projection.event_timeline.entries.iter() {
                        article {
                            style: "border: 1px solid #bbb; border-radius: 0.6rem; padding: 0.8rem; margin-top: 0.6rem;",
                            strong { "{entry.display_coordinate}" }
                            span { " · {entry.placement:?}" }
                            div {
                                SemanticRefButton { reference: entry.event_ref.clone(), selected_ref }
                            }
                            if !entry.claim_refs.is_empty() {
                                div {
                                    style: "margin-top: 0.4rem;",
                                    strong { "Claims: " }
                                    for claim_ref in entry.claim_refs.iter() {
                                        SemanticRefButton { reference: claim_ref.clone(), selected_ref }
                                        span { " " }
                                    }
                                }
                            }
                        }
                    }
                },
                TimelineMode::Knowledge => rsx! {
                    for entry in model.projection.knowledge_timeline.iter() {
                        article {
                            style: "border: 1px solid #bbb; border-radius: 0.6rem; padding: 0.8rem; margin-top: 0.6rem;",
                            strong { "{entry.knowledge_time_ref}" }
                            span { " · {entry.knowledge_membership:?}" }
                            div {
                                SemanticRefButton { reference: entry.semantic_ref.clone(), selected_ref }
                            }
                            div {
                                style: "font-size: 0.85rem; opacity: 0.75;",
                                "source revision: "
                                SemanticRefButton { reference: entry.source_revision_ref.clone(), selected_ref }
                            }
                            if let Some(role) = entry.source_role_ref.as_ref() {
                                div { style: "font-size: 0.85rem;", "source role: {role}" }
                            }
                        }
                    }
                },
                TimelineMode::Work => rsx! {
                    if !model.projection.operational_outstanding.states.is_empty() {
                        section {
                            style: "margin-top: 0.7rem;",
                            h3 { "Carryover / interrupted / unresolved" }
                            p {
                                style: "font-size: 0.82rem; opacity: 0.72;",
                                "Operational unresolved ≠ Review pending ≠ Semantic unresolved ≠ User priority."
                            }
                            for state in model.projection.operational_outstanding.states.iter() {
                                article {
                                    style: "border: 1px dashed #999; border-radius: 0.6rem; padding: 0.8rem; margin-top: 0.5rem;",
                                    strong { "{state.label}" }
                                    span { " · {state.kind:?}" }
                                    div {
                                        SemanticRefButton { reference: state.operational_state_ref.clone(), selected_ref }
                                    }
                                    div {
                                        style: "font-size: 0.84rem;",
                                        "subject: "
                                        SemanticRefButton { reference: state.subject_ref.clone(), selected_ref }
                                    }
                                }
                            }
                        }
                    }

                    h3 { style: "margin-top: 1rem;", "Activity events" }
                    for entry in model.projection.operational_timeline.entries.iter() {
                        article {
                            style: "border: 1px solid #bbb; border-radius: 0.6rem; padding: 0.8rem; margin-top: 0.6rem;",
                            strong { "{entry.event.label}" }
                            span { " · {entry.event.start_time_ref}" }
                            div {
                                SemanticRefButton { reference: entry.event.operational_event_ref.clone(), selected_ref }
                            }
                            if !entry.target_refs.is_empty() {
                                div {
                                    style: "margin-top: 0.4rem;",
                                    strong { "Linked semantic/workflow targets: " }
                                    for target in entry.target_refs.iter() {
                                        SemanticRefButton { reference: target.clone(), selected_ref }
                                        span { " " }
                                    }
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn MatterClaimsPanel(
    model: GenericMatterWorkspace,
    selected_ref: Signal<Option<String>>,
) -> Element {
    rsx! {
        section {
            style: "margin-top: 1rem;",
            h2 { "Facts / Claims" }
            p { "Proposition roots retain distinct accounts, denials and qualifications." }
            for view in model.projection.event_timeline.proposition_views.iter() {
                article {
                    style: "border: 1px solid #bbb; border-radius: 0.6rem; padding: 0.8rem; margin-top: 0.6rem;",
                    h3 { "{view.root.label}" }
                    SemanticRefButton { reference: view.root.proposition_ref.clone(), selected_ref }
                    for leaf in view.leaves.iter() {
                        div {
                            style: "margin-top: 0.5rem; padding-left: 0.6rem; border-left: 3px solid #bbb;",
                            strong { "{leaf.kind:?} · {leaf.review_state:?}" }
                            div {
                                SemanticRefButton { reference: leaf.claim_ref.clone(), selected_ref }
                            }
                            if let Some(speaker) = leaf.speaker_ref.as_ref() {
                                div { style: "font-size: 0.82rem;", "speaker: {speaker}" }
                            }
                            for statement_ref in leaf.statement_refs.iter() {
                                SemanticRefButton { reference: statement_ref.clone(), selected_ref }
                                span { " " }
                            }
                        }
                    }
                    if !view.relations.is_empty() {
                        details {
                            summary { "Contestation relations" }
                            ul {
                                for relation in view.relations.iter() {
                                    li {
                                        "{relation.kind:?}: "
                                        SemanticRefButton { reference: relation.from_claim_ref.clone(), selected_ref }
                                        span { " → " }
                                        SemanticRefButton { reference: relation.to_claim_ref.clone(), selected_ref }
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

#[component]
fn MatterSuggestedJoinsPanel(
    model: GenericMatterWorkspace,
    selected_ref: Signal<Option<String>>,
) -> Element {
    rsx! {
        section {
            style: "margin-top: 1rem;",
            h2 { "Suggested Joins" }
            p { "Machine-proposed structure; EventAssembly review is required before event identity exists." }
            for view in model.projection.join_proposals.proposals.iter() {
                article {
                    style: "border: 1px solid #bbb; border-radius: 0.6rem; padding: 0.8rem; margin-top: 0.6rem;",
                    div {
                        strong { "Candidate event join · {view.signal_kind_count} independent signal kinds" }
                    }
                    SemanticRefButton { reference: view.proposal.proposal_ref.clone(), selected_ref }
                    details {
                        summary { "Signal evidence" }
                        ul {
                            for signal in view.proposal.signals.iter() {
                                li { "{signal.kind:?} · {signal.evidence_ref} · detector {signal.detector_ref}" }
                            }
                        }
                    }
                    details {
                        summary { "Observation / statement ancestry" }
                        for observation in view.proposal.observation_refs.iter() {
                            SemanticRefButton { reference: observation.clone(), selected_ref }
                            br {}
                        }
                        for statement in view.proposal.statement_refs.iter() {
                            SemanticRefButton { reference: statement.clone(), selected_ref }
                            br {}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MatterReviewPanel(
    model: GenericMatterWorkspace,
    selected_ref: Signal<Option<String>>,
) -> Element {
    let workspace = crate::workbench::review::ProductionReviewWorkspace {
        queue: model.projection.review_queue.clone(),
        candidate_only: true,
        creates_semantic_authority: false,
        applicability_promoted: false,
        claim_truth_promoted: false,
    };

    rsx! {
        section {
            style: "margin-top: 1rem;",
            h2 { "Review" }
            p { "Select a semantic coordinate for cross-navigation, then use the existing live typed action cards." }
            for item in workspace.queue.items.iter() {
                div {
                    style: "margin-top: 0.35rem;",
                    SemanticRefButton { reference: item.semantic_ref.clone(), selected_ref }
                }
            }
            crate::app::ReviewWorkspaceView { model: workspace }
        }
    }
}

#[component]
fn MatterRefListPanel(
    title: &'static str,
    refs: Vec<String>,
    selected_ref: Signal<Option<String>>,
) -> Element {
    rsx! {
        section {
            style: "margin-top: 1rem;",
            h2 { "{title}" }
            if refs.is_empty() {
                p { "No visible coordinates are currently attached to this Matter projection." }
            } else {
                ul {
                    for reference in refs.iter() {
                        li {
                            SemanticRefButton { reference: reference.clone(), selected_ref }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MatterHandoffPanel(
    model: GenericMatterWorkspace,
    selected_ref: Signal<Option<String>>,
) -> Element {
    let mut recipient_ref = use_signal(|| "recipient:lawyer".to_owned());
    let mut recipient_profile = use_signal(|| HandoffRecipientProfile::Lawyer);
    let mut selected = use_signal(BTreeSet::<String>::new);
    let mut redacted = use_signal(BTreeSet::<String>::new);
    let mut preview = use_signal(|| None::<MinimalHandoffPreview>);
    let mut error = use_signal(|| None::<String>);

    let visible_refs = model.projection.context_projection.included_refs.clone();

    rsx! {
        section {
            style: "margin-top: 1rem;",
            h2 { "Handoff" }
            p {
                "Select a bounded subset already visible under MatterContext, then preview exactly what leaves. Redaction never deletes the canonical Matter."
            }

            label {
                "Recipient ref"
                input {
                    style: "display: block; width: min(100%, 34rem); margin-top: 0.2rem;",
                    value: recipient_ref(),
                    oninput: move |event| recipient_ref.set(event.value())
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; gap: 0.4rem; margin-top: 0.6rem;",
                button { onclick: move |_| recipient_profile.set(HandoffRecipientProfile::Lawyer), "Lawyer" }
                button { onclick: move |_| recipient_profile.set(HandoffRecipientProfile::Clinician), "Clinician" }
                button { onclick: move |_| recipient_profile.set(HandoffRecipientProfile::Advocate), "Advocate" }
                button { onclick: move |_| recipient_profile.set(HandoffRecipientProfile::Regulator), "Regulator" }
                button { onclick: move |_| recipient_profile.set(HandoffRecipientProfile::Researcher), "Researcher" }
            }

            if let Some(reference) = selected_ref.read().as_ref() {
                button {
                    style: "margin-top: 0.6rem;",
                    onclick: {
                        let reference = reference.clone();
                        move |_| {
                            selected.write().insert(reference.clone());
                        }
                    },
                    "Add selected semantic coordinate to handoff"
                }
            }

            details {
                style: "margin-top: 0.8rem;",
                summary { "Visible Matter coordinates" }
                for reference in visible_refs.iter() {
                    {
                        let selected_ref_value = reference.clone();
                        let redact_ref_value = reference.clone();
                        let is_selected = selected.read().contains(reference);
                        let is_redacted = redacted.read().contains(reference);
                        rsx! {
                            div {
                                style: "display: grid; grid-template-columns: 1fr auto auto; gap: 0.5rem; margin-top: 0.35rem;",
                                SemanticRefButton { reference: reference.clone(), selected_ref }
                                button {
                                    onclick: move |_| {
                                        let mut set = selected.write();
                                        if !set.insert(selected_ref_value.clone()) {
                                            set.remove(&selected_ref_value);
                                        }
                                    },
                                    if is_selected { "Selected" } else { "Select" }
                                }
                                button {
                                    disabled: !is_selected,
                                    onclick: move |_| {
                                        let mut set = redacted.write();
                                        if !set.insert(redact_ref_value.clone()) {
                                            set.remove(&redact_ref_value);
                                        }
                                    },
                                    if is_redacted { "Redacted" } else { "Redact" }
                                }
                            }
                        }
                    }
                }
            }

            button {
                style: "margin-top: 0.8rem; padding: 0.45rem 0.8rem;",
                onclick: move |_| {
                    let selection = MinimalHandoffSelection {
                        handoff_ref: format!("handoff:{}:preview", model.projection.matter_ref),
                        matter_ref: model.projection.matter_ref.clone(),
                        recipient_ref: recipient_ref.read().trim().to_owned(),
                        recipient_profile: *recipient_profile.read(),
                        disclosure_boundary: DisclosureBoundary::RecipientScoped,
                        selected_refs: selected.read().iter().cloned().collect(),
                        redaction_refs: redacted.read().iter().cloned().collect(),
                        retention_policy_ref: "retention:bounded".into(),
                        redaction_policy_ref: "redaction:explicit".into(),
                        text_export_policy_ref: "text-export:reviewed-visible-only".into(),
                        local_only: true,
                        do_not_sync: true,
                    };
                    match preview_minimal_handoff(
                        &selection,
                        &model.projection.context_projection,
                    ) {
                        Ok(value) => {
                            preview.set(Some(value));
                            error.set(None);
                        }
                        Err(problem) => {
                            preview.set(None);
                            error.set(Some(format!("{problem:?}")));
                        }
                    }
                },
                "Preview handoff"
            }

            if let Some(problem) = error.read().as_ref() {
                p { "Handoff preview failed: {problem}" }
            }

            if let Some(value) = preview.read().as_ref() {
                article {
                    style: "border: 1px solid #888; border-radius: 0.6rem; padding: 0.8rem; margin-top: 0.8rem;",
                    h3 { "Preview: exactly what leaves" }
                    p { "Recipient: {value.recipient_ref} · {value.recipient_profile:?}" }
                    h4 { "Exported refs" }
                    ul {
                        for reference in value.exported_refs.iter() {
                            li { "{reference}" }
                        }
                    }
                    h4 { "Redacted refs" }
                    ul {
                        for reference in value.redacted_refs.iter() {
                            li { "{reference}" }
                        }
                    }
                    h4 { "Context exclusions retained" }
                    ul {
                        for exclusion in value.visible_exclusions.iter() {
                            li { "{exclusion.semantic_ref} · {exclusion.reason:?}" }
                        }
                    }
                    p {
                        style: "font-size: 0.8rem; opacity: 0.7;",
                        "canonical_world_mutated={value.canonical_world_mutated} · redaction_deletes_source={value.redaction_deletes_canonical_source} · export_creates_authority={value.export_creates_semantic_authority}"
                    }
                }
            }
        }
    }
}


#[component]
fn MatterAcceptancePanel(
    model: GenericMatterWorkspace,
    selected_ref: Signal<Option<String>>,
) -> Element {
    let receipt = &model.acceptance;

    rsx! {
        section {
            style: "margin-top: 1rem;",
            h2 { "M13 Acceptance" }
            p {
                "Executable Mary/SensibLaw operator diagnostics over the current persisted Matter. These are review/product obligations, not new truth statuses."
            }

            div {
                style: "display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 0.5rem;",
                AcceptanceCount { label: "Source reopen paths", count: receipt.source_reopenable_ref_count }
                AcceptanceCount { label: "Event entries", count: receipt.event_entry_count }
                AcceptanceCount { label: "Knowledge entries", count: receipt.knowledge_entry_count }
                AcceptanceCount { label: "Work events", count: receipt.work_event_count }
                AcceptanceCount { label: "Suggested joins", count: receipt.suggested_join_count }
                AcceptanceCount { label: "Review items", count: receipt.review_item_count }
            }

            AcceptanceRefSection {
                title: "Missing date / undated / unknown chronology",
                refs: receipt.missing_date_event_refs.clone(),
                selected_ref
            }
            AcceptanceRefSection {
                title: "Missing actor",
                refs: receipt.missing_actor_claim_refs.clone(),
                selected_ref
            }
            AcceptanceRefSection {
                title: "Contradictory chronology / accounts",
                refs: receipt.contradictory_relation_refs.clone(),
                selected_ref
            }
            AcceptanceRefSection {
                title: "No-event material (explicitly reviewed; absence was not inferred)",
                refs: receipt.no_event_refs.clone(),
                selected_ref
            }
            AcceptanceRefSection {
                title: "Party assertions",
                refs: receipt.party_assertion_refs.clone(),
                selected_ref
            }
            AcceptanceRefSection {
                title: "Procedural outcomes",
                refs: receipt.procedural_outcome_refs.clone(),
                selected_ref
            }
            AcceptanceRefSection {
                title: "Later annotations",
                refs: receipt.later_annotation_refs.clone(),
                selected_ref
            }
            AcceptanceRefSection {
                title: "Procedural significance still open",
                refs: receipt.procedural_significance_review_refs.clone(),
                selected_ref
            }
            AcceptanceRefSection {
                title: "Operational carryover / interrupted / unresolved",
                refs: receipt.operational_carryover_refs.clone(),
                selected_ref
            }

            article {
                style: "border: 1px solid #bbb; border-radius: 0.6rem; padding: 0.8rem; margin-top: 0.9rem;",
                strong { "Non-collapse receipt" }
                ul {
                    li { "missing date ≠ event did not happen: {receipt.missing_date_means_event_did_not_happen}" }
                    li { "missing actor ≠ unknown person finding: {receipt.missing_actor_means_unknown_person}" }
                    li { "no-event material ≠ false: {receipt.no_event_means_false}" }
                    li { "creates semantic authority: {receipt.creates_semantic_authority}" }
                    li { "claim truth promoted: {receipt.claim_truth_promoted}" }
                    li { "canonical world mutated: {receipt.canonical_world_mutated}" }
                }
            }
        }
    }
}

#[component]
fn AcceptanceCount(label: &'static str, count: usize) -> Element {
    rsx! {
        div {
            style: "border: 1px solid #ccc; border-radius: 0.45rem; padding: 0.55rem;",
            strong { "{count}" }
            div { style: "font-size: 0.8rem; opacity: 0.75;", "{label}" }
        }
    }
}

#[component]
fn AcceptanceRefSection(
    title: &'static str,
    refs: Vec<String>,
    selected_ref: Signal<Option<String>>,
) -> Element {
    rsx! {
        details {
            style: "border: 1px solid #ddd; border-radius: 0.5rem; padding: 0.6rem; margin-top: 0.55rem;",
            summary { "{title} · {refs.len()}" }
            if refs.is_empty() {
                p { style: "font-size: 0.85rem; opacity: 0.7;", "No coordinates in this category." }
            } else {
                ul {
                    for reference in refs.iter() {
                        li {
                            SemanticRefButton { reference: reference.clone(), selected_ref }
                        }
                    }
                }
            }
        }
    }
}
