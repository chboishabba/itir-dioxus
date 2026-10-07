#![cfg(feature = "production-data")]

use dioxus::prelude::*;

use crate::workbench::controversy::MatterControversyWorkspace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControversyPersona {
    Client,
    Solicitor,
    Court,
}

#[component]
pub fn MatterControversyWorkspaceView(model: MatterControversyWorkspace) -> Element {
    let mut persona = use_signal(|| ControversyPersona::Client);
    rsx! {
        section {
            style: "margin-top:1.25rem;border:1px solid #999;border-radius:.8rem;padding:1rem;",
            h2 { "Controversy" }
            p {
                "One persisted Matter, projected differently for affected people, legal operators and the court."
            }
            p {
                style: "font-size:.84rem;opacity:.74;",
                "Reconstruction only · typed disagreement ≠ Boolean negation · recognition ≠ creation · no credibility, ultimate-fact, normative-weight or merits determination"
            }
            div {
                style: "display:flex;gap:.5rem;flex-wrap:wrap;margin:.8rem 0;",
                button { onclick: move |_| persona.set(ControversyPersona::Client), "Client / community" }
                button { onclick: move |_| persona.set(ControversyPersona::Solicitor), "Solicitor / counsel" }
                button { onclick: move |_| persona.set(ControversyPersona::Court), "Court / associate" }
            }
            if model.projections.is_empty() {
                p { "No persisted controversy is attached to this Matter yet." }
            } else {
                for projection in model.projections.iter() {
                    article {
                        style: "border-top:1px solid #ccc;padding-top:.9rem;margin-top:.9rem;overflow-wrap:anywhere;",
                        div { strong { "Controversy: " } "{projection.controversy_ref}" }
                        div { strong { "Matter: " } "{projection.matter_ref}" }
                        match *persona.read() {
                            ControversyPersona::Client => rsx! {
                                h3 { "What is being said, disputed and still needed?" }
                                RefGroup { title: "Propositions", refs: projection.client.proposition_refs.clone() }
                                RefGroup { title: "Reviewed", refs: projection.client.reviewed_proposition_refs.clone() }
                                RefGroup { title: "Still candidate", refs: projection.client.candidate_proposition_refs.clone() }
                                RefGroup { title: "Disputed", refs: projection.client.disputed_proposition_refs.clone() }
                                RefGroup { title: "Responses from other participants", refs: projection.client.response_refs.clone() }
                                RefGroup { title: "Still unresolved", refs: projection.client.unresolved_residual_refs.clone() }
                                RefGroup { title: "Source coordinates", refs: projection.client.source_refs.clone() }
                                RefGroup { title: "Review coordinates", refs: projection.client.reviewed_evidence_refs.clone() }
                                RefGroup { title: "Normative orders in view", refs: projection.client.normative_order_refs.clone() }
                                p {
                                    style: "font-size:.82rem;opacity:.72;",
                                    "Normative-order coordinates are preserved and related; this view does not coerce, rank or translate one order into another."
                                }
                            },
                            ControversyPersona::Solicitor => rsx! {
                                h3 { "What case do I have, and what would reduce the controversy?" }
                                RefGroup { title: "Common ground", refs: projection.solicitor.common_ground_proposition_refs.clone() }
                                RefGroup { title: "Typed responses", refs: projection.solicitor.typed_response_refs.clone() }
                                RefGroup { title: "Residual controversy", refs: projection.solicitor.residual_refs.clone() }
                                RefGroup { title: "Open proof obligations", refs: projection.solicitor.open_obligation_refs.clone() }
                                RefGroup { title: "Targeted evidence / authority queries", refs: projection.solicitor.reverse_search.target_evidence_queries.clone() }
                                RefGroup { title: "Potentially affected propositions", refs: projection.solicitor.reverse_search.potential_reopening_refs.clone() }
                                RefGroup { title: "Normative orders", refs: projection.solicitor.normative_order_refs.clone() }
                                p {
                                    style: "font-size:.82rem;opacity:.72;",
                                    "Potential reopening is counterfactual only. Acquisition/access authority and actual selective reopening remain downstream INV operations."
                                }
                            },
                            ControversyPersona::Court => rsx! {
                                h3 { "What is common ground and what remains contested?" }
                                RefGroup { title: "Common ground", refs: projection.court.common_ground_proposition_refs.clone() }
                                RefGroup { title: "Disputed occurrence", refs: projection.court.disputed_occurrence_refs.clone() }
                                RefGroup { title: "Disputed characterisation", refs: projection.court.disputed_characterisation_refs.clone() }
                                RefGroup { title: "Disputed causation", refs: projection.court.disputed_causation_refs.clone() }
                                RefGroup { title: "Disputed evidence / reliability", refs: projection.court.disputed_evidence_refs.clone() }
                                RefGroup { title: "Legal consequence", refs: projection.court.legal_consequence_refs.clone() }
                                RefGroup { title: "Open evidentiary questions", refs: projection.court.open_evidentiary_obligation_refs.clone() }
                                RefGroup { title: "Open legal questions", refs: projection.court.open_legal_obligation_refs.clone() }
                                RefGroup { title: "Sources", refs: projection.court.source_refs.clone() }
                                RefGroup { title: "Review coordinates", refs: projection.court.reviewed_evidence_refs.clone() }
                                RefGroup { title: "Normative orders", refs: projection.court.normative_order_refs.clone() }
                                p {
                                    style: "font-size:.82rem;font-weight:600;",
                                    "No winner, credibility score, ultimate-fact determination, normative-weight assignment or final judgment is produced by this projection."
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RefGroup(title: &'static str, refs: Vec<String>) -> Element {
    rsx! {
        section { style: "margin-top:.65rem;",
            strong { "{title}" }
            if refs.is_empty() {
                div { style: "font-size:.84rem;opacity:.65;", "None in the persisted projection." }
            } else {
                ul {
                    for reference in refs.iter() {
                        li { style: "font-size:.86rem;", "{reference}" }
                    }
                }
            }
        }
    }
}
