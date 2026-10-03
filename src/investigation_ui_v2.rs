#![cfg(feature = "production-data")]

use dioxus::prelude::*;

use crate::visual::command::VisualObjectId;
use crate::workbench::investigation::{
    investigation_graph_ir, project_investigation, DisplaySortLens, InvestigationGraphProjection,
    InvestigationMode, InvestigationProjection, InvestigationQueueWorkspace,
    ProjectedInvestigationRoute,
};

#[component]
pub fn InvestigationAcquisitionView(model: InvestigationQueueWorkspace) -> Element {
    let mut mode = use_signal(|| InvestigationMode::Investigate);
    let mut selected_route_ref = use_signal(|| None::<String>);
    let active_mode = mode();
    let selected = selected_route_ref();
    let view = match project_investigation(
        &model.queue,
        model.governance.as_ref(),
        active_mode,
        DisplaySortLens::StableRouteId,
        selected.as_deref(),
        None,
    ) {
        Ok(view) => view,
        Err(error) => return rsx! {
            article {
                h1 { "Investigation projection unavailable" }
                p { "{error}" }
                p { "The persisted acquisition queue is unchanged." }
            }
        },
    };

    rsx! {
        article {
            header {
                h1 { "Proof-directed investigation" }
                p { "Unresolved evidence → lawful acquisition alternatives → selective reopening." }
                p {
                    style: "font-size:.9rem; opacity:.75;",
                    "Pareto set — no overall ranking · projection only · no acquisition, truth, access, or semantic promotion"
                }
            }

            nav {
                style: "display:flex; gap:.5rem; margin:1rem 0; flex-wrap:wrap;",
                ModeButton { label: "Investigate", active: active_mode == InvestigationMode::Investigate,
                    onclick: move |_| mode.set(InvestigationMode::Investigate) }
                ModeButton { label: "Inspect", active: active_mode == InvestigationMode::Inspect,
                    onclick: move |_| mode.set(InvestigationMode::Inspect) }
                ModeButton { label: "Graph", active: active_mode == InvestigationMode::Graph,
                    onclick: move |_| mode.set(InvestigationMode::Graph) }
            }

            if active_mode == InvestigationMode::Investigate {
                InvestigateView { view: view.clone(), selected_route_ref }
            } else if active_mode == InvestigationMode::Inspect {
                InspectView { view: view.clone() }
            } else {
                InvestigationGraphView { graph: model.graph.clone(), view: view.clone() }
            }

            footer {
                style: "margin-top:1.25rem; padding-top:.75rem; border-top:1px solid #aaa; font-size:.8rem; opacity:.75;",
                p { "Matter: {model.matter_ref}" }
                p { "World routes: {view.world_route_count} · visible: {view.visible_route_count} · hidden by view: {view.hidden_route_count}" }
                p { "Selected ≠ preferred · hidden ≠ absent · blocked ≠ dominated · frontier ≠ authorized" }
            }
        }
    }
}

#[component]
fn ModeButton(label: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            type: "button",
            onclick: move |event| onclick.call(event),
            style: if active {
                "border:2px solid #0f5b78;background:#e7f5fb;border-radius:.45rem;padding:.55rem .85rem;cursor:pointer;"
            } else {
                "border:1px solid #888;background:#fff;border-radius:.45rem;padding:.55rem .85rem;cursor:pointer;"
            },
            "{label}"
        }
    }
}

#[component]
fn InvestigateView(view: InvestigationProjection, mut selected_route_ref: Signal<Option<String>>) -> Element {
    rsx! {
        section {
            article {
                style: "border-left:4px solid #0f5b78;background:#f7fbfd;padding:1rem 1.2rem;",
                h2 { "Unresolved question" }
                p { "{view.unresolved_target}" }
                dl {
                    style: "display:grid;grid-template-columns:max-content minmax(0,1fr);gap:.3rem .75rem;",
                    dt { "Why unresolved" }
                    dd { "Residual {view.residual_obligation_ref} remains unpaid." }
                    dt { "Current source state" }
                    dd { "{view.current_availability}" }
                    dt { "Access constraint" }
                    dd { "{view.access_constraint_ref}" }
                }
            }

            RouteSection {
                title: "Useful next evidence · Pareto frontier",
                explanation: "These alternatives survive the declared mixed-orientation evidence trade-offs. Display order is not a recommendation.",
                routes: view.frontier_executable.clone(),
                selected_route_ref: view.selected_route_ref.clone(),
                status_label: "frontier · executable now",
                on_select: move |reference| selected_route_ref.set(Some(reference)),
            }

            if !view.frontier_blocked.is_empty() {
                RouteSection {
                    title: "Blocked but Pareto-relevant",
                    explanation: "Blocked means current access/governance does not permit execution. It does not mean inferior evidence.",
                    routes: view.frontier_blocked.clone(),
                    selected_route_ref: view.selected_route_ref.clone(),
                    status_label: "frontier · access blocked",
                    on_select: move |reference| selected_route_ref.set(Some(reference)),
                }
            }

            if !view.dominated.is_empty() {
                details {
                    style: "margin-top:1.25rem;",
                    summary { "Dominated alternatives · {view.dominated.len()} retained for inspection" }
                    p { "Dominance is an axis-wise Pareto relation, not a truth or evidentiary-worth judgment." }
                    for route in view.dominated.iter() {
                        RouteCard {
                            route: route.clone(),
                            status_label: "dominated on declared acquisition axes",
                            selected: view.selected_route_ref.as_deref() == Some(route.route_ref.as_str()),
                            on_select: move |reference| selected_route_ref.set(Some(reference)),
                        }
                    }
                }
            }

            if let Some(governance) = view.governance.as_ref() {
                section {
                    style: "margin-top:1.25rem;border:1px solid #aaa;border-radius:.5rem;padding:1rem;",
                    h2 { "Governance envelope" }
                    p { "Purpose: {governance.purpose_ref}" }
                    p { "Access: {governance.access_state} · privacy: {governance.privacy_state} · AI use: {governance.ai_use_state}" }
                    p { "Service: {governance.service_change_state} · evidence: {governance.evidence_state}" }
                    p { style: "font-size:.82rem;opacity:.75;",
                        "This constrains execution. It is not a Pareto coordinate and is not certification." }
                }
            }
        }
    }
}

#[component]
fn RouteSection(
    title: &'static str,
    explanation: &'static str,
    routes: Vec<ProjectedInvestigationRoute>,
    selected_route_ref: Option<String>,
    status_label: &'static str,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        section {
            style: "margin-top:1.25rem;",
            h2 { "{title}" }
            p { "{explanation}" }
            if routes.is_empty() {
                p { style: "opacity:.75;", "No routes in this disposition." }
            }
            for route in routes.iter() {
                RouteCard {
                    route: route.clone(),
                    status_label,
                    selected: selected_route_ref.as_deref() == Some(route.route_ref.as_str()),
                    on_select: move |reference| on_select.call(reference),
                }
            }
        }
    }
}

#[component]
fn RouteCard(
    route: ProjectedInvestigationRoute,
    status_label: &'static str,
    selected: bool,
    on_select: EventHandler<String>,
) -> Element {
    let route_ref = route.route_ref.clone();
    rsx! {
        article {
            style: if selected {
                "border:2px solid #8a4b00;background:#fffaf2;border-radius:.55rem;padding:1rem;margin:.65rem 0;"
            } else {
                "border:1px solid #888;border-radius:.55rem;padding:1rem;margin:.65rem 0;"
            },
            h3 { "{route.route_description}" }
            p { style: "font-size:.82rem;opacity:.75;", "{status_label}" }
            dl {
                style: "display:grid;grid-template-columns:max-content minmax(0,1fr);gap:.25rem .7rem;",
                dt { "Access" } dd { "{route.access_state}" }
                dt { "Information gain" } dd { "{route.information_gain}" }
                dt { "Dependency impact" } dd { "{route.dependency_closure_impact}" }
                dt { "Residual coverage" } dd { "{route.residual_coverage}" }
                dt { "Provenance novelty" } dd { "{route.provenance_novelty}" }
                dt { "Acquisition cost" } dd { "{route.acquisition_cost}" }
            }
            if !route.potential_reopening_refs.is_empty() {
                details {
                    summary { "{route.potential_reopening_label}" }
                    for reference in route.potential_reopening_refs.iter() {
                        p { style: "overflow-wrap:anywhere;", "{reference}" }
                    }
                    p { style: "font-size:.8rem;opacity:.75;",
                        "No reopening occurs until a genuine acquired source is canonically persisted." }
                }
            }
            button { type: "button", onclick: move |_| on_select.call(route_ref.clone()), "Inspect this route" }
        }
    }
}

#[component]
fn InspectView(view: InvestigationProjection) -> Element {
    rsx! {
        section {
            h2 { "Exact acquisition coordinates" }
            dl {
                style: "display:grid;grid-template-columns:max-content minmax(0,1fr);gap:.3rem .75rem;overflow-wrap:anywhere;",
                dt { "Obligation" } dd { "{view.obligation_ref}" }
                dt { "Comparison" } dd { "{view.comparison_ref}" }
                dt { "Residual" } dd { "{view.residual_obligation_ref}" }
                dt { "Access constraint" } dd { "{view.access_constraint_ref}" }
                dt { "Display lens" } dd { "{view.display_sort_label}" }
            }
            h2 { "Route provenance and genealogy" }
            for route in view.frontier_executable.iter().chain(view.frontier_blocked.iter()).chain(view.dominated.iter()) {
                article {
                    style: "border-bottom:1px solid #aaa;padding:.75rem 0;",
                    h3 { "{route.route_ref}" }
                    p { style: "overflow-wrap:anywhere;", "Locator: {route.source_locator_ref}" }
                    p { style: "overflow-wrap:anywhere;", "Genealogy: {route.provenance_genealogy_ref}" }
                    p { "Independence: {route.independence_state} · duplicate relation: {route.duplicate_relation}" }
                    p { style: "overflow-wrap:anywhere;", "Axis estimate: {route.axis_estimation_receipt_ref}" }
                }
            }
            if let Some(governance) = view.governance.as_ref() {
                h2 { "Governance evidence" }
                p { style: "overflow-wrap:anywhere;", "Packet: {governance.packet_ref}" }
                for reference in governance.control_refs.iter() { p { "Control: {reference}" } }
                for reference in governance.evidence_refs.iter() { p { "Evidence: {reference}" } }
            }
        }
    }
}

#[component]
fn InvestigationGraphView(graph: Option<InvestigationGraphProjection>, view: InvestigationProjection) -> Element {
    let mut selected = use_signal(|| None::<VisualObjectId>);
    let Some(persisted) = graph else {
        return rsx! {
            section {
                h2 { "Graph" }
                article {
                    style: "border-left:4px solid #777;background:#f7f7f7;padding:1rem 1.2rem;",
                    h3 { "No persisted investigation graph is bound to this view" }
                    p { "Graph mode will not synthesize topology from journal, route, or dependency references." }
                    p { "Persist and review a legal/proof graph, then bind it to this INV obligation with an explicit binding receipt." }
                }
                CounterfactualCoordinates { view }
            }
        };
    };
    let graph_ir = match investigation_graph_ir(&persisted) {
        Ok(graph) => graph,
        Err(error) => return rsx! {
            section { h2 { "Graph unavailable" } p { "{error}" } }
        },
    };
    let inspection = selected().and_then(|id| graph_ir.inspect_object(id));

    rsx! {
        section {
            header {
                h2 { "Graph · persisted proof/dependency projection" }
                p { "{graph_ir.nodes.len()} nodes · {graph_ir.edges.len()} edges" }
                p { style: "font-size:.82rem;opacity:.75;overflow-wrap:anywhere;",
                    "Projection: {persisted.projection_ref} · binding: {persisted.binding.binding_ref}" }
                p { style: "font-size:.82rem;opacity:.75;",
                    "Derived-only · challengeable · visual geometry is presentation only" }
            }

            div {
                style: "position:relative;min-height:28rem;border:1px solid #aaa;border-radius:.6rem;background:#fafafa;overflow:hidden;margin-top:1rem;",
                for node in graph_ir.nodes.iter() {
                    {
                        let id = node.id;
                        let left = 50.0 + node.x * 44.0;
                        let top = 50.0 - node.y * 42.0;
                        rsx! {
                            button {
                                key: "{node.semantic_ref}",
                                type: "button",
                                style: "position:absolute;left:{left}%;top:{top}%;transform:translate(-50%,-50%);width:11rem;min-height:4.5rem;text-align:left;border:1px solid #666;border-radius:.45rem;background:white;padding:.55rem;cursor:pointer;",
                                onclick: move |_| selected.set(Some(id)),
                                strong { "{node.label}" }
                                div { style: "font-size:.76rem;opacity:.7;", "{node.kind}" }
                            }
                        }
                    }
                }
            }

            details {
                style: "margin-top:1rem;",
                summary { "Persisted graph relations · {graph_ir.edges.len()}" }
                for edge in graph_ir.edges.iter() {
                    p { style: "overflow-wrap:anywhere;", "{edge.semantic_ref}: {edge.kind}" }
                }
            }

            if let Some(item) = inspection {
                article {
                    style: "margin-top:1rem;border-left:4px solid #0f5b78;padding:.75rem 1rem;",
                    h3 { "Selected persisted graph object" }
                    p { style: "overflow-wrap:anywhere;", "{item.semantic_ref}" }
                    p { "Kind: {item.kind}" }
                    for reference in item.source_refs.iter() { p { style: "overflow-wrap:anywhere;", "Source: {reference}" } }
                    for reference in item.provenance_refs.iter() { p { style: "overflow-wrap:anywhere;", "Provenance: {reference}" } }
                    p { style: "font-size:.8rem;opacity:.75;", "Selection changes no review, truth, access, or acquisition state." }
                }
            }

            details {
                style: "margin-top:1rem;",
                summary { "Graph binding evidence" }
                for reference in persisted.binding.binding_evidence_refs.iter() {
                    p { style: "overflow-wrap:anywhere;", "{reference}" }
                }
            }
        }
    }
}

#[component]
fn CounterfactualCoordinates(view: InvestigationProjection) -> Element {
    rsx! {
        details {
            style: "margin-top:1rem;",
            summary { "Counterfactual reopening coordinates (not a graph)" }
            for route in view.frontier_executable.iter().chain(view.frontier_blocked.iter()) {
                if !route.potential_reopening_refs.is_empty() {
                    article {
                        h3 { "{route.route_description}" }
                        for reference in route.potential_reopening_refs.iter() { p { "{reference}" } }
                    }
                }
            }
        }
    }
}
