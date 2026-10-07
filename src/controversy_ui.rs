#![cfg(feature = "production-data")]

use dioxus::prelude::*;

use crate::controversy::MatterControversyWorkspace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Persona {
    Client,
    Solicitor,
    Court,
}

#[component]
pub fn MatterControversyView(model: MatterControversyWorkspace) -> Element {
    let mut persona = use_signal(|| Persona::Solicitor);
    rsx! {
        section {
            header {
                h1 { "Matter controversy" }
                div { style: "font-size:0.85rem;opacity:0.7;overflow-wrap:anywhere;", "{model.matter.matter_ref}" }
                div { style: "font-size:0.8rem;opacity:0.65;overflow-wrap:anywhere;", "{model.matter.controversy_ref}" }
                p { "One persisted controversy, projected for different legal actors without changing the underlying Matter." }
                p { style: "font-size:0.85rem;opacity:0.75;", "Structure/search only · admission ≠ truth · recognition ≠ creation · this view does not decide who should win" }
            }
            nav {
                style: "display:flex;gap:0.5rem;flex-wrap:wrap;margin:1rem 0;",
                button { onclick: move |_| persona.set(Persona::Client), "Client / community" }
                button { onclick: move |_| persona.set(Persona::Solicitor), "Solicitor / counsel" }
                button { onclick: move |_| persona.set(Persona::Court), "Court" }
            }
            match *persona.read() {
                Persona::Client => rsx! { ClientView { model: model.clone() } },
                Persona::Solicitor => rsx! { SolicitorView { model: model.clone() } },
                Persona::Court => rsx! { CourtView { model: model.clone() } },
            }
        }
    }
}

#[component]
fn ClientView(model: MatterControversyWorkspace) -> Element {
    let view = &model.client;
    rsx! {
        section {
            h2 { "What is being said, disputed, and still needed?" }
            p { "Reviewed source-backed propositions stay distinct from candidates, responses, and unresolved questions." }
            h3 { "Normative orders represented" }
            ul { for order in &view.normative_order_refs { li { "{order}" } } }
            h3 { "Reviewed propositions" }
            ul { for reference in &view.reviewed_fibre_refs { li { "{reference}" } } }
            if !view.candidate_fibre_refs.is_empty() {
                h3 { "Still candidate" }
                ul { for reference in &view.candidate_fibre_refs { li { "{reference}" } } }
            }
            h3 { "Other-side responses" }
            ul { for mode in &view.response_modes { li { "{mode:?}" } } }
            h3 { "Still unresolved" }
            ul { for question in &view.unresolved_questions { li { "{question}" } } }
        }
    }
}

#[component]
fn SolicitorView(model: MatterControversyWorkspace) -> Element {
    let view = &model.solicitor;
    rsx! {
        section {
            h2 { "Controversy / case theory" }
            p { "Typed responses preserve whether the dispute concerns occurrence, characterisation, causation, reliability, context, or admission." }
            h3 { "Responses" }
            ul { for mode in &view.response_modes { li { "{mode:?}" } } }
            h3 { "Residual kinds" }
            ul { for kind in &view.residual_kinds { li { "{kind:?}" } } }
            h3 { "Open proof obligations" }
            ul { for reference in &view.open_obligation_refs { li { "{reference}" } } }
            h3 { "What would reduce the controversy?" }
            for (index, query) in view.target_evidence_queries.iter().enumerate() {
                article {
                    style: "border:1px solid #bbb;border-radius:0.5rem;padding:0.7rem;margin-top:0.5rem;",
                    if let Some(discriminator) = view.requested_discriminators.get(index) {
                        div { strong { "Discriminator: " } "{discriminator}" }
                    }
                    div { strong { "Targeted evidence / authority query: " } "{query}" }
                }
            }
        }
    }
}

#[component]
fn CourtView(model: MatterControversyWorkspace) -> Element {
    let view = &model.court;
    rsx! {
        section {
            h2 { "Controversy reconstruction" }
            p { "This projection isolates agreement and dispute types. It does not determine credibility, ultimate facts, normative weight, or final judgment." }
            h3 { "Common ground" }
            ul { for reference in &view.common_ground_refs { li { "{reference}" } } }
            h3 { "Disputed occurrence" }
            ul { for reference in &view.disputed_occurrence_refs { li { "{reference}" } } }
            h3 { "Disputed characterisation / context" }
            ul { for reference in &view.disputed_characterisation_refs { li { "{reference}" } } }
            h3 { "Disputed causation" }
            ul { for reference in &view.disputed_causation_refs { li { "{reference}" } } }
            h3 { "Evidence reliability disputes" }
            ul { for reference in &view.evidence_reliability_refs { li { "{reference}" } } }
            h3 { "Legal consequence disputes" }
            ul { for reference in &view.legal_consequence_refs { li { "{reference}" } } }
            h3 { "Normative-order mismatches" }
            ul { for reference in &view.normative_order_mismatch_refs { li { "{reference}" } } }
            h3 { "Open questions" }
            ul { for reference in &view.open_question_refs { li { "{reference}" } } }
        }
    }
}
