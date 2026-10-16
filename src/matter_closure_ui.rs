#![cfg(feature = "production-data")]

use dioxus::prelude::*;

use crate::workbench::matter::GenericMatterWorkspace;

#[component]
pub fn MatterClosureView(model: GenericMatterWorkspace) -> Element {
    let controversy_count = model.controversies.len();
    let open_obligation_count = model
        .controversies
        .iter()
        .flat_map(|controversy| controversy.obligations.iter())
        .filter(|obligation| obligation.is_open)
        .count();
    let investigation_count = model.controversy_investigations.len();
    let evidence_stage_count = model
        .controversy_investigations
        .iter()
        .map(|status| status.evidence_stages.len())
        .sum::<usize>();
    let reassessment_count = model
        .controversy_investigations
        .iter()
        .filter(|status| status.requires_residual_reassessment)
        .count();

    if controversy_count == 0 {
        return rsx! {};
    }

    rsx! {
        section {
            style: "margin-top: 1rem; border: 1px solid #999; border-radius: 0.7rem; padding: 0.9rem;",
            header {
                h2 { "Controversy / proof / investigation" }
                p {
                    "The same persisted Matter object is shown through disagreement, proof-obligation, acquisition and reviewed-evidence coordinates. Investigation and review are continuations of the open residual, not separate workflow worlds."
                }
                div {
                    style: "display:flex; flex-wrap:wrap; gap:.6rem; font-size:.85rem;",
                    span { "{controversy_count} controversies" }
                    span { "{open_obligation_count} open proof obligations" }
                    span { "{investigation_count} investigation queues" }
                    span { "{evidence_stage_count} reviewed-evidence stages" }
                    span { "{reassessment_count} residuals awaiting reassessment" }
                }
                p {
                    style: "font-size:.8rem; opacity:.72;",
                    "evidence supplied ≠ residual resolved · selected route ≠ recommendation · blocked ≠ bad · reviewed ≠ truth"
                }
            }

            for controversy in model.controversies.iter() {
                article {
                    style: "border-top: 1px solid #bbb; margin-top: .9rem; padding-top: .9rem;",
                    div {
                        strong { "Controversy " }
                        code { "{controversy.matter.controversy_ref}" }
                    }
                    div {
                        style: "font-size:.82rem; opacity:.75; margin-top:.2rem;",
                        "Matter {controversy.matter.matter_ref} · root {controversy.matter.root_fibre_ref}"
                    }

                    h3 { style: "margin-bottom:.35rem;", "Proposition fibres" }
                    div {
                        style: "display:grid; grid-template-columns: repeat(auto-fit,minmax(260px,1fr)); gap:.55rem;",
                        for fibre in controversy.fibres.iter() {
                            div {
                                style: "border:1px solid #ccc; border-radius:.5rem; padding:.65rem;",
                                strong { "{fibre.party:?} · {fibre.epistemic_status:?}" }
                                div { code { "{fibre.fibre_ref}" } }
                                div { style: "font-size:.82rem;", "claim: {fibre.claim_ref}" }
                                div { style: "font-size:.82rem;", "normative order: {fibre.normative_order_ref}" }
                                div { style: "font-size:.82rem;", "source: {fibre.source_reference}" }
                                if let Some(reviewed) = fibre.reviewed_evidence_ref.as_ref() {
                                    div { style: "font-size:.82rem;", "reviewed evidence: {reviewed}" }
                                } else {
                                    div { style: "font-size:.82rem;", "review state: candidate / no reviewed-evidence coordinate" }
                                }
                            }
                        }
                    }

                    if !controversy.responses.is_empty() {
                        h3 { style: "margin-bottom:.35rem;", "Typed responses" }
                        for response in controversy.responses.iter() {
                            div {
                                style: "margin-top:.3rem; padding:.45rem .55rem; border-left:3px solid #999;",
                                strong { "{response.mode:?}" }
                                div {
                                    style: "font-size:.82rem;",
                                    "{response.response_fibre_ref} → {response.target_fibre_ref} · {response.response_ref}"
                                }
                            }
                        }
                    }

                    h3 { style: "margin-bottom:.35rem;", "Residuals and next evidence" }
                    for residual in controversy.residuals.iter() {
                        article {
                            style: "border:1px solid #bbb; border-radius:.55rem; padding:.7rem; margin-top:.5rem;",
                            strong { "{residual.kind:?}" }
                            div { code { "{residual.residual_ref}" } }
                            p { "{residual.unresolved_question}" }
                            if let Some(discriminator) = residual.requested_discriminator.as_ref() {
                                div {
                                    strong { "What would discriminate this: " }
                                    "{discriminator}"
                                }
                            }
                            if let Some(query) = residual.target_evidence_query.as_ref() {
                                div {
                                    strong { "Evidence target: " }
                                    "{query}"
                                }
                            }
                        }
                    }

                    h3 { style: "margin-bottom:.35rem;", "Open proof obligations" }
                    for obligation in controversy.obligations.iter() {
                        article {
                            style: if obligation.is_open {
                                "border:1px solid #999;border-radius:.55rem;padding:.7rem;margin-top:.5rem;"
                            } else {
                                "border:1px solid #ddd;border-radius:.55rem;padding:.7rem;margin-top:.5rem;opacity:.72;"
                            },
                            div {
                                strong { "{obligation.kind:?}" }
                                span { if obligation.is_open { " · open" } else { " · closed" } }
                            }
                            div { code { "{obligation.obligation_ref}" } }
                            div { style: "font-size:.82rem;", "required by {obligation.required_by}" }

                            for status in model.controversy_investigations.iter().filter(|status| {
                                status.controversy_ref == controversy.matter.controversy_ref
                                    && status.legal_obligation_ref == obligation.obligation_ref
                            }) {
                                MatterInvestigationStatusView { status: status.clone() }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MatterInvestigationStatusView(
    status: sensiblaw_pg_source_store::ControversyInvestigationStatus,
) -> Element {
    let obligation = &status.queue.obligation;
    let priority = &status.queue.priority;

    rsx! {
        section {
            style: "margin-top:.7rem; padding:.7rem; background: color-mix(in srgb, Canvas 94%, CanvasText 6%); border-radius:.5rem;",
            strong { "Investigation continuation" }
            div {
                style: "font-size:.82rem; margin-top:.25rem;",
                "{obligation.requested_discriminator}"
            }
            div {
                style: "font-size:.82rem;",
                "target: {obligation.target_evidence_query}"
            }
            div {
                style: "display:flex; flex-wrap:wrap; gap:.55rem; margin-top:.45rem; font-size:.82rem;",
                span { "{priority.executable_frontier_route_refs.len()} executable frontier" }
                span { "{priority.blocked_frontier_route_refs.len()} blocked frontier" }
                span { "{status.queue.routes.len()} routes retained" }
                span { "scalar ranking: {priority.scalar_score_used}" }
            }

            details {
                style: "margin-top:.5rem;",
                summary { "Acquisition routes" }
                for route in status.queue.routes.iter() {
                    article {
                        style: "border:1px solid #ccc;border-radius:.45rem;padding:.55rem;margin-top:.4rem;",
                        strong { "{route.route_description}" }
                        div { code { "{route.route_ref}" } }
                        div {
                            style: "font-size:.8rem;",
                            "access={route.access_disposition:?} · independence={route.independence:?} · duplicate={route.duplicate_relation:?}"
                        }
                        div {
                            style: "font-size:.8rem;",
                            "gain {route.information_gain} · dependency {route.dependency_closure_impact} · residual {route.residual_coverage} · provenance {route.provenance_novelty} · cost {route.acquisition_cost}"
                        }
                        if priority.executable_frontier_route_refs.contains(&route.route_ref) {
                            div { style: "font-size:.8rem; font-weight:600;", "Pareto frontier · executable" }
                        } else if priority.blocked_frontier_route_refs.contains(&route.route_ref) {
                            div { style: "font-size:.8rem; font-weight:600;", "Pareto frontier · access blocked/unknown" }
                        } else {
                            div { style: "font-size:.8rem; opacity:.7;", "dominated alternative retained" }
                        }
                    }
                }
            }

            if status.evidence_stages.is_empty() {
                p {
                    style: "font-size:.82rem; margin-bottom:0;",
                    "No reviewed evidence stage yet; the discriminator remains open."
                }
            } else {
                details {
                    style: "margin-top:.5rem;",
                    summary { "Acquisition → review → selective reopening history" }
                    for stage in status.evidence_stages.iter() {
                        article {
                            style: "border-left:3px solid #999;padding:.45rem .6rem;margin-top:.45rem;",
                            div { code { "{stage.stage_ref}" } }
                            div { style: "font-size:.8rem;", "source {stage.acquired_source_revision_ref}" }
                            div { style: "font-size:.8rem;", "review {stage.reviewed_evidence_ref}" }
                            div { style: "font-size:.8rem;", "acquisition receipt {stage.acquisition_receipt_ref}" }
                            div {
                                style: "font-size:.8rem;",
                                "evidence supplied={stage.evidence_supplied} · controversy reopened={stage.controversy_reopened} · reassessment required={stage.requires_residual_reassessment}"
                            }
                            div {
                                style: "font-size:.8rem; font-weight:600;",
                                "residual resolved automatically={stage.residual_resolved}"
                            }
                        }
                    }
                }
            }

            if status.requires_residual_reassessment {
                p {
                    style: "font-size:.82rem; font-weight:600; margin-bottom:0;",
                    "New reviewed evidence reached this controversy; the residual now requires human/legal reassessment."
                }
            }
            p {
                style: "font-size:.78rem; opacity:.7; margin-bottom:0;",
                "creates semantic authority={status.creates_semantic_authority} · creates legal authority={status.creates_legal_authority} · system-resolves residual={status.residual_resolved_by_system}"
            }
        }
    }
}
