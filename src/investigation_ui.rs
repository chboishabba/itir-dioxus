#![cfg(feature = "production-data")]

use dioxus::prelude::*;

use crate::workbench::investigation::{
    project_investigation, DisplaySortLens, InvestigationMode, InvestigationQueueWorkspace,
    ProjectedInvestigationRoute,
};

#[component]
pub fn InvestigationAcquisitionView(model: InvestigationQueueWorkspace) -> Element {
    let mut mode = use_signal(|| InvestigationMode::Investigate);
    let mut selected_route_ref = use_signal(|| None::<String>);
    let active_mode = mode();
    let selected = selected_route_ref();
    let projection = project_investigation(
        &model.queue,
        model.governance.as_ref(),
        active_mode,
        DisplaySortLens::StableRouteId,
        selected.as_deref(),
        None,
    );
    let view = match projection {
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
                p {
                    "Unresolved evidence → lawful acquisition alternatives → selective reopening."
                }
                p {
                    style: "font-size:.9rem; opacity:.75;",
                    "Pareto set — no overall ranking · projection only · no acquisition, truth, access, or semantic promotion"
                }
            }

            nav {
                style: "display:flex; gap:.5rem; margin:1rem 0; flex-wrap:wrap;",
                button {
                    type: "button",
                    onclick: move |_| mode.set(InvestigationMode::Investigate),
                    style: mode_button_style(active_mode == InvestigationMode::Investigate),
                    "Investigate"
                }
                button {
                    type: "button",
                    onclick: move |_| mode.set(InvestigationMode::Inspect),
                    style: mode_button_style(active_mode == InvestigationMode::Inspect),
                    "Inspect"
                }
                button {
                    type: "button",
                    onclick: move |_| mode.set(InvestigationMode::Graph),
                    style: mode_button_style(active_mode == InvestigationMode::Graph),
                    "Graph"
                }
            }

            if active_mode == InvestigationMode::Investigate {
                InvestigateView {
                    view: view.clone(),
                    selected_route_ref: selected_route_ref,
                }
            } else if active_mode == InvestigationMode::Inspect {
                InspectView { view: view.clone() }
            } else {
                GraphView { view: view.clone() }
            }

            footer {
                style: "margin-top:1.25rem; padding-top:.75rem; border-top:1px solid #aaa; font-size:.8rem; opacity:.75;",
                p { "Matter: {model.matter_ref}" }
                p {
                    "World routes: {view.world_route_count} · visible: {view.visible_route_count} · hidden by view: {view.hidden_route_count}"
                }
                p {
                    "Selected ≠ preferred · hidden ≠ absent · blocked ≠ dominated · frontier ≠ authorized"
                }
            }
        }
    }
}

#[component]
fn InvestigateView(
    view: crate::workbench::investigation::InvestigationProjection,
    mut selected_route_ref: Signal<Option<String>>,
) -> Element {
    rsx! {
        section {
            article {
                style: "border-left:4px solid #0f5b78; background:#f7fbfd; padding:1rem 1.2rem;",
                h2 { "Unresolved question" }
                p { "{view.unresolved_target}" }
                dl {
                    style: "display:grid; grid-template-columns:max-content minmax(0,1fr); gap:.3rem .75rem;",
                    dt { "Why unresolved" }
                    dd { "Residual {view.residual_obligation_ref} remains unpaid." }
                    dt { "Current source state" }
                    dd { "{view.current_availability}" }
                    dt { "Access constraint" }
                    dd { "{view.access_constraint_ref}" }
                }
            }

            section {
                style: "margin-top:1.25rem;",
                h2 { "Useful next evidence · Pareto frontier" }
                p {
                    "These alternatives survive the declared mixed-orientation evidence trade-offs. Their display order is not a recommendation."
                }
                if view.frontier_executable.is_empty() {
                    p { style: "opacity:.75;", "No frontier route is executable under the current access state." }
                }
                for route in view.frontier_executable.iter() {
                    RouteCard {
                        route: route.clone(),
                        access_label: "frontier · executable now",
                        selected: view.selected_route_ref.as_deref() == Some(route.route_ref.as_str()),
                        on_select: move |reference: String| selected_route_ref.set(Some(reference)),
                    }
                }
            }

            if !view.frontier_blocked.is_empty() {
                section {
                    style: "margin-top:1.25rem;",
                    h2 { "Blocked but Pareto-relevant" }
                    p {
                        "Blocked means the present access/governance state does not permit execution. It does not mean inferior evidence."
                    }
                    for route in view.frontier_blocked.iter() {
                        RouteCard {
                            route: route.clone(),
                            access_label: "frontier · access blocked",
                            selected: view.selected_route_ref.as_deref() == Some(route.route_ref.as_str()),
                            on_select: move |reference: String| selected_route_ref.set(Some(reference)),
                        }
                    }
                }
            }

            if !view.dominated.is_empty() {
                details {
                    style: "margin-top:1.25rem;",
                    summary { "Dominated alternatives · {view.dominated.len()} retained for inspection" }
                    p {
                        "Dominance is an axis-wise Pareto relation, not a truth or evidentiary-worth judgment."
                    }
                    for route in view.dominated.iter() {
                        RouteCard {
                            route: route.clone(),
                            access_label: "dominated on declared acquisition axes",
                            selected: view.selected_route_ref.as_deref() == Some(route.route_ref.as_str()),
                            on_select: move |reference: String| selected_route_ref.set(Some(reference)),
                        }
                    }
                }
            }

            if let Some(governance) = view.governance.as_ref() {
                section {
                    style: "margin-top:1.25rem; border:1px solid #aaa; border-radius:.5rem; padding:1rem;",
                    h2 { "Governance envelope" }
                    p { "Purpose: {governance.purpose_ref}" }
                    p { "Access: {governance.access_state} · privacy: {governance.privacy_state} · AI use: {governance.ai_use_state}" }
                    p { "Service state: {governance.service_change_state} · evidence state: {governance.evidence_state}" }
                    p {
                        style: "font-size:.82rem; opacity:.75;",
                        "This envelope constrains execution. It is not a Pareto coordinate and is not a certification claim."
                    }
                }
            }
        }
    }
}

#[component]
fn RouteCard(
    route: ProjectedInvestigationRoute,
    access_label: &'static str,
    selected: bool,
    on_select: EventHandler<String>,
) -> Element {
    let route_ref = route.route_ref.clone();
    rsx! {
        article {
            style: if selected {
                "border:2px solid #8a4b00; background:#fffaf2; border-radius:.55rem; padding:1rem; margin:.65rem 0;"
            } else {
                "border:1px solid #888; border-radius:.55rem; padding:1rem; margin:.65rem 0;"
            },
            header {
                h3 { "{route.route_description}" }
                p { style: "font-size:.82rem; opacity:.75;", "{access_label}" }
            }
            dl {
                style: "display:grid; grid-template-columns:max-content minmax(0,1fr); gap:.25rem .7rem;",
                dt { "Access" }
                dd { "{route.access_state}" }
                dt { "Information gain" }
                dd { "{route.information_gain}" }
                dt { "Dependency impact" }
                dd { "{route.dependency_closure_impact}" }
                dt { "Residual coverage" }
                dd { "{route.residual_coverage}" }
                dt { "Provenance novelty" }
                dd { "{route.provenance_novelty}" }
                dt { "Acquisition cost" }
                dd { "{route.acquisition_cost}" }
            }
            if !route.potential_reopening_refs.is_empty() {
                details {
                    summary { "{route.potential_reopening_label}" }
                    for reference in route.potential_reopening_refs.iter() {
                        p { style: "overflow-wrap:anywhere;", "{reference}" }
                    }
                    p {
                        style: "font-size:.8rem; opacity:.75;",
                        "No reopening occurs until a genuine acquired source is canonically persisted."
                    }
                }
            }
            button {
                type: "button",
                onclick: move |_| on_select.call(route_ref.clone()),
                "Inspect this route"
            }
        }
    }
}

#[component]
fn InspectView(view: crate::workbench::investigation::InvestigationProjection) -> Element {
    rsx! {
        section {
            h2 { "Exact acquisition coordinates" }
            dl {
                style: "display:grid; grid-template-columns:max-content minmax(0,1fr); gap:.3rem .75rem; overflow-wrap:anywhere;",
                dt { "Obligation" }
                dd { "{view.obligation_ref}" }
                dt { "Comparison" }
                dd { "{view.comparison_ref}" }
                dt { "Residual" }
                dd { "{view.residual_obligation_ref}" }
                dt { "Access constraint" }
                dd { "{view.access_constraint_ref}" }
                dt { "Display lens" }
                dd { "{view.display_sort_label}" }
            }

            h2 { "Route provenance and genealogy" }
            for route in view.frontier_executable.iter()
                .chain(view.frontier_blocked.iter())
                .chain(view.dominated.iter()) {
                article {
                    style: "border-bottom:1px solid #aaa; padding:.75rem 0;",
                    h3 { "{route.route_ref}" }
                    p { style: "overflow-wrap:anywhere;", "Locator: {route.source_locator_ref}" }
                    p { style: "overflow-wrap:anywhere;", "Genealogy: {route.provenance_genealogy_ref}" }
                    p { "Independence: {route.independence_state} · duplicate relation: {route.duplicate_relation}" }
                    p { style: "overflow-wrap:anywhere;", "Axis-estimate receipt: {route.axis_estimation_receipt_ref}" }
                }
            }

            if let Some(governance) = view.governance.as_ref() {
                h2 { "Governance evidence" }
                p { style: "overflow-wrap:anywhere;", "Packet: {governance.packet_ref}" }
                p { "Controls" }
                for reference in governance.control_refs.iter() {
                    p { style: "overflow-wrap:anywhere;", "{reference}" }
                }
                p { "Evidence" }
                for reference in governance.evidence_refs.iter() {
                    p { style: "overflow-wrap:anywhere;", "{reference}" }
                }
            }
        }
    }
}

#[component]
fn GraphView(view: crate::workbench::investigation::InvestigationProjection) -> Element {
    rsx! {
        section {
            h2 { "Graph" }
            article {
                style: "border-left:4px solid #777; background:#f7f7f7; padding:1rem 1.2rem;",
                h3 { "No persisted investigation graph is bound to this view" }
                p {
                    "The graph infrastructure exists, but this acquisition projection will not synthesize nodes or edges from route, journal, or dependency references."
                }
                p {
                    "Bind a real persisted proof/dependency graph to this matter before rendering GraphIr or enabling graph picking."
                }
            }
            if view.frontier_executable.iter()
                .chain(view.frontier_blocked.iter())
                .any(|route| !route.potential_reopening_refs.is_empty()) {
                details {
                    style: "margin-top:1rem;",
                    summary { "Counterfactual reopening coordinates (not a graph)" }
                    for route in view.frontier_executable.iter().chain(view.frontier_blocked.iter()) {
                        if !route.potential_reopening_refs.is_empty() {
                            article {
                                h3 { "{route.route_description}" }
                                for reference in route.potential_reopening_refs.iter() {
                                    p { style: "overflow-wrap:anywhere;", "{reference}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn mode_button_style(active: bool) -> &'static str {
    if active {
        "border:2px solid #0f5b78; background:#e7f5fb; border-radius:.45rem; padding:.55rem .85rem; cursor:pointer;"
    } else {
        "border:1px solid #888; background:#fff; border-radius:.45rem; padding:.55rem .85rem; cursor:pointer;"
    }
}
