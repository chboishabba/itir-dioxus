#![cfg(feature = "production-data")]

use std::collections::BTreeSet;

use itir_dioxus::workbench::investigation::{
    project_investigation, DisplaySortLens, InvestigationMode,
};
use sensiblaw_pg_source_store::{
    AccessDisposition, AcquisitionObligation, AcquisitionPriorityReceipt,
    AcquisitionRouteCandidate, DurableAcquisitionQueue, DuplicateRelation,
    EvidenceIndependence, RecordAvailability,
};

fn queue() -> DurableAcquisitionQueue {
    let obligation = AcquisitionObligation {
        obligation_ref: "acquisition:visible-ui".into(),
        comparison_ref: "comparison:visible-ui".into(),
        residual_obligation_ref: "residual:missing-primary".into(),
        source_revision_refs: vec!["revision:a".into(), "revision:b".into()],
        target_description: "locate primary source".into(),
        current_availability: RecordAvailability::NotLocated,
        authority_or_access_constraint_ref: "access:lawful-only".into(),
        dependency_target_refs: vec!["assessment:direct".into()],
        candidate_only: true,
        creates_semantic_authority: false,
        claim_truth_promoted: false,
    };
    let route = |id: &str, access, gain, cost| AcquisitionRouteCandidate {
        route_ref: id.into(),
        obligation_ref: obligation.obligation_ref.clone(),
        route_description: format!("route {id}"),
        source_locator_ref: format!("locator:{id}"),
        access_disposition: access,
        authority_receipt_ref: None,
        provenance_genealogy_ref: format!("genealogy:{id}"),
        independence: EvidenceIndependence::Unknown,
        independence_receipt_ref: None,
        duplicate_relation: DuplicateRelation::Unknown,
        information_gain: gain,
        dependency_closure_impact: 5,
        residual_coverage: 5,
        provenance_novelty: 5,
        acquisition_cost: cost,
        axis_estimation_receipt_ref: "estimate:reviewed".into(),
        candidate_only: true,
        creates_acquisition_authority: false,
    };
    DurableAcquisitionQueue {
        obligation,
        routes: vec![
            route("blocked", AccessDisposition::RequiresAuthorization, 20, 20),
            route("public", AccessDisposition::Public, 10, 5),
            route("dominated", AccessDisposition::Public, 9, 6),
        ],
        priority: AcquisitionPriorityReceipt {
            obligation_ref: "acquisition:visible-ui".into(),
            frontier_route_refs: vec!["blocked".into(), "public".into()],
            executable_frontier_route_refs: vec!["public".into()],
            blocked_frontier_route_refs: vec!["blocked".into()],
            scalar_score_used: false,
            creates_semantic_authority: false,
            creates_screening_or_acquisition_decision: false,
        },
    }
}

#[test]
fn investigate_inspect_and_graph_modes_preserve_the_same_frontier() {
    let q = queue();
    let investigate = project_investigation(
        &q, None, InvestigationMode::Investigate,
        DisplaySortLens::StableRouteId, Some("public"), None,
    ).unwrap();
    let inspect = project_investigation(
        &q, None, InvestigationMode::Inspect,
        DisplaySortLens::InformationGainDescending, Some("public"), None,
    ).unwrap();
    let graph = project_investigation(
        &q, None, InvestigationMode::Graph,
        DisplaySortLens::AcquisitionCostAscending, Some("public"), None,
    ).unwrap();

    assert_eq!(investigate.frontier_refs(), inspect.frontier_refs());
    assert_eq!(investigate.frontier_refs(), graph.frontier_refs());
    assert_eq!(investigate.selected_route_ref.as_deref(), Some("public"));
    assert!(!investigate.selection_creates_preference);
    assert!(!graph.sort_changes_frontier);
    assert!(!graph.acquisition_executed);
    assert!(!graph.creates_semantic_authority);
}

#[test]
fn filtering_a_route_hides_it_without_deleting_it_from_the_world() {
    let q = queue();
    let visible = BTreeSet::from(["public".to_owned(), "dominated".to_owned()]);
    let view = project_investigation(
        &q, None, InvestigationMode::Investigate,
        DisplaySortLens::StableRouteId, None, Some(&visible),
    ).unwrap();

    assert_eq!(view.world_route_count, 3);
    assert_eq!(view.visible_route_count, 2);
    assert_eq!(view.hidden_route_count, 1);
    assert!(!view.hidden_means_absent);
}
