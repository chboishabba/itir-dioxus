#![cfg(feature = "production-data")]

use dioxus::prelude::*;
use sensiblaw_pg_source_store::{
    MatterControversyDraft, MatterDisagreementKind, MatterEpistemicStatus,
    MatterPartyRole, MatterResponseMode,
};

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
            if model.entries.is_empty() {
                p { "No persisted controversy is attached to this Matter yet." }
            } else {
                for entry in model.entries.iter() {
                    article {
                        style: "border-top:1px solid #ccc;padding-top:.9rem;margin-top:.9rem;overflow-wrap:anywhere;",
                        div { style: "font-size:.84rem;font-weight:600;", "Stage: {entry.controversy.stage_ref}" }
                        if let Some(previous) = entry.controversy.supersedes_controversy_ref.as_ref() {
                            div { style: "font-size:.8rem;opacity:.72;", "Supersedes persisted snapshot: {previous}" }
                        }
                        details {
                            summary { "Persisted controversy coordinates" }
                            div { strong { "Controversy: " } "{entry.personas.controversy_ref}" }
                            div { strong { "Matter: " } "{entry.personas.matter_ref}" }
                        }
                        match *persona.read() {
                            ControversyPersona::Client => rsx! {
                                h3 { "What is being said, disputed and still needed?" }
                                p { "Each statement below keeps who advances it, its review/status state and the normative order in which it is framed." }
                                for proposition in entry.controversy.propositions.iter() {
                                    PropositionCard {
                                        text: proposition.display_text.clone(),
                                        party: party_label(proposition.party),
                                        status: status_label(proposition.epistemic_status),
                                        normative_order_ref: proposition.normative_order_ref.clone(),
                                        relation_ref: proposition.relation_ref.clone(),
                                        source_ref: proposition.source_ref.clone(),
                                        review_ref: proposition.reviewed_evidence_ref.clone(),
                                    }
                                }
                                if !entry.controversy.responses.is_empty() {
                                    section { style: "margin-top:.8rem;",
                                        h4 { "What the other side / participants say" }
                                        for response in entry.controversy.responses.iter() {
                                            article { style: "border-left:3px solid #999;padding:.45rem .7rem;margin:.45rem 0;",
                                                strong { "{response_mode_label(response.mode)}" }
                                                div { "About: {proposition_text(&entry.controversy, &response.target_proposition_ref)}" }
                                                div { "Response: {proposition_text(&entry.controversy, &response.response_proposition_ref)}" }
                                            }
                                        }
                                    }
                                }
                                if !entry.controversy.residuals.is_empty() {
                                    section { style: "margin-top:.8rem;",
                                        h4 { "What is still unresolved?" }
                                        for residual in entry.controversy.residuals.iter() {
                                            article { style: "border:1px dashed #999;border-radius:.5rem;padding:.65rem;margin:.45rem 0;",
                                                div { "{residual.unresolved_question}" }
                                                div { style: "font-size:.82rem;opacity:.72;", "Needed next: {residual.target_evidence_query}" }
                                            }
                                        }
                                    }
                                }
                                RefGroup { title: "Normative orders in view", refs: entry.personas.client.normative_order_refs.clone() }
                                p {
                                    style: "font-size:.82rem;opacity:.72;",
                                    "Normative-order coordinates are preserved and related; this view does not coerce, rank or translate one order into another."
                                }
                            },
                            ControversyPersona::Solicitor => rsx! {
                                h3 { "What case do I have, and what would reduce the controversy?" }
                                p {
                                    strong { "Working backwards from: " }
                                    "{proposition_text(&entry.controversy, &entry.personas.solicitor.reverse_search.target_proposition_ref)}"
                                }
                                section {
                                    h4 { "Proposition map" }
                                    for proposition in entry.controversy.propositions.iter() {
                                        PropositionCard {
                                            text: proposition.display_text.clone(),
                                            party: party_label(proposition.party),
                                            status: status_label(proposition.epistemic_status),
                                            normative_order_ref: proposition.normative_order_ref.clone(),
                                            relation_ref: proposition.relation_ref.clone(),
                                            source_ref: proposition.source_ref.clone(),
                                            review_ref: proposition.reviewed_evidence_ref.clone(),
                                        }
                                    }
                                }
                                if !entry.controversy.responses.is_empty() {
                                    section { style: "margin-top:.8rem;",
                                        h4 { "Typed responses" }
                                        for response in entry.controversy.responses.iter() {
                                            article { style: "border-left:3px solid #777;padding:.45rem .7rem;margin:.45rem 0;",
                                                strong { "{response_mode_label(response.mode)}" }
                                                div { "Target: {proposition_text(&entry.controversy, &response.target_proposition_ref)}" }
                                                div { "Response: {proposition_text(&entry.controversy, &response.response_proposition_ref)}" }
                                                details { summary { "Coordinates" } div { "{response.response_ref}" } }
                                            }
                                        }
                                    }
                                }
                                if !entry.controversy.residuals.is_empty() {
                                    section { style: "margin-top:.8rem;",
                                        h4 { "Residual controversy → targeted next step" }
                                        for residual in entry.controversy.residuals.iter() {
                                            article { style: "border:1px solid #aaa;border-radius:.5rem;padding:.65rem;margin:.45rem 0;",
                                                strong { "{residual.unresolved_question}" }
                                                div { "Targeted evidence / authority query: {residual.target_evidence_query}" }
                                                div { style: "font-size:.82rem;opacity:.72;", "Requested discriminator: {residual.requested_discriminator_ref}" }
                                                if let Some(comparison_ref) = residual.relational_comparison_ref.as_ref() {
                                                    div { style: "font-size:.82rem;", "REL comparison: {comparison_ref}" }
                                                }
                                                if let Some(obligation_ref) = residual.relational_obligation_ref.as_ref() {
                                                    div { style: "font-size:.82rem;", "REL obligation: {obligation_ref}" }
                                                }
                                            }
                                        }
                                    }
                                }
                                RefGroup { title: "Open proof obligations", refs: entry.personas.solicitor.open_obligation_refs.clone() }
                                RefGroup { title: "Potentially affected propositions", refs: entry.personas.solicitor.reverse_search.potential_reopening_refs.clone() }
                                p {
                                    style: "font-size:.82rem;opacity:.72;",
                                    "Potential reopening is counterfactual only. Acquisition/access authority and actual selective reopening remain downstream INV operations."
                                }
                            },
                            ControversyPersona::Court => rsx! {
                                h3 { "What is common ground and what remains contested?" }
                                p { style: "font-size:.82rem;opacity:.72;", "This is an immutable {entry.controversy.stage_ref} snapshot; prior stages remain separately reopenable." }
                                if !entry.personas.court.common_ground_proposition_refs.is_empty() {
                                    section {
                                        h4 { "Common ground" }
                                        for reference in entry.personas.court.common_ground_proposition_refs.iter() {
                                            div { "✓ {proposition_text(&entry.controversy, reference)}" }
                                        }
                                    }
                                }
                                if !entry.controversy.residuals.is_empty() {
                                    section { style: "margin-top:.8rem;",
                                        h4 { "Unresolved controversy" }
                                        for residual in entry.controversy.residuals.iter() {
                                            article { style: "border-left:3px solid #999;padding:.45rem .7rem;margin:.45rem 0;",
                                                strong { "{disagreement_label(residual.kind)}" }
                                                div { "{residual.unresolved_question}" }
                                                div { style: "font-size:.82rem;opacity:.72;", "Applicant side: {proposition_text(&entry.controversy, &residual.applicant_proposition_ref)}" }
                                                div { style: "font-size:.82rem;opacity:.72;", "Respondent side: {proposition_text(&entry.controversy, &residual.respondent_proposition_ref)}" }
                                            }
                                        }
                                    }
                                }
                                RefGroup { title: "Open evidentiary questions", refs: entry.personas.court.open_evidentiary_obligation_refs.clone() }
                                RefGroup { title: "Open legal questions", refs: entry.personas.court.open_legal_obligation_refs.clone() }
                                details {
                                    summary { "Source, review and normative-order coordinates" }
                                    RefGroup { title: "Sources", refs: entry.personas.court.source_refs.clone() }
                                    RefGroup { title: "Review coordinates", refs: entry.personas.court.reviewed_evidence_refs.clone() }
                                    RefGroup { title: "Normative orders", refs: entry.personas.court.normative_order_refs.clone() }
                                }
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
fn PropositionCard(
    text: String,
    party: &'static str,
    status: &'static str,
    normative_order_ref: String,
    relation_ref: String,
    source_ref: String,
    review_ref: Option<String>,
) -> Element {
    rsx! {
        article { style: "border:1px solid #bbb;border-radius:.55rem;padding:.7rem;margin:.5rem 0;",
            div { style: "font-size:.82rem;opacity:.72;", "{party} · {status}" }
            div { style: "font-weight:600;margin:.2rem 0;", "{text}" }
            div { style: "font-size:.82rem;", "Normative order: {normative_order_ref}" }
            details {
                summary { "Source / review provenance" }
                div { style: "font-size:.82rem;", "Relationship: {relation_ref}" }
                div { style: "font-size:.82rem;", "Source: {source_ref}" }
                if let Some(review_ref) = review_ref.as_ref() {
                    div { style: "font-size:.82rem;", "Reviewed evidence: {review_ref}" }
                } else {
                    div { style: "font-size:.82rem;opacity:.7;", "No reviewed-evidence coordinate yet." }
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

fn proposition_text(controversy: &MatterControversyDraft, proposition_ref: &str) -> String {
    controversy
        .propositions
        .iter()
        .find(|proposition| proposition.proposition_ref == proposition_ref)
        .map(|proposition| proposition.display_text.clone())
        .unwrap_or_else(|| proposition_ref.to_owned())
}

fn party_label(value: MatterPartyRole) -> &'static str {
    match value {
        MatterPartyRole::Applicant => "Applicant",
        MatterPartyRole::Respondent => "Respondent",
        MatterPartyRole::Court => "Court / common-ground record",
        MatterPartyRole::ExternalWitness => "External witness",
    }
}

fn status_label(value: MatterEpistemicStatus) -> &'static str {
    match value {
        MatterEpistemicStatus::Alleged => "alleged",
        MatterEpistemicStatus::Admitted => "admitted / common ground",
        MatterEpistemicStatus::Disputed => "disputed",
        MatterEpistemicStatus::Supported => "supported",
        MatterEpistemicStatus::Proved => "recorded as proved",
        MatterEpistemicStatus::Rejected => "recorded as rejected",
        MatterEpistemicStatus::Unresolved => "unresolved",
    }
}

fn response_mode_label(value: MatterResponseMode) -> &'static str {
    match value {
        MatterResponseMode::DenyOccurrence => "Denies occurrence",
        MatterResponseMode::AdmitOccurrenceDisputeCharacterisation => {
            "Admits occurrence; disputes characterisation"
        }
        MatterResponseMode::AdmitConductAddContext => "Admits conduct; adds context",
        MatterResponseMode::DisputeCausation => "Disputes causation",
        MatterResponseMode::ChallengeEvidenceReliability => "Challenges evidence reliability",
        MatterResponseMode::OfferAlternativeEvent => "Offers alternative event / account",
        MatterResponseMode::AdmitProposition => "Admits proposition",
    }
}

fn disagreement_label(value: MatterDisagreementKind) -> &'static str {
    match value {
        MatterDisagreementKind::Node => "Disputed occurrence / proposition node",
        MatterDisagreementKind::Relation => "Disputed relation",
        MatterDisagreementKind::Evidence => "Disputed evidence / reliability",
        MatterDisagreementKind::Characterisation => "Disputed characterisation",
        MatterDisagreementKind::Causal => "Disputed causation",
        MatterDisagreementKind::LegalConsequence => "Disputed legal consequence",
    }
}
