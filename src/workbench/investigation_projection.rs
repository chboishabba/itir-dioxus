#![cfg(feature = "production-data")]

#[cfg(test)]
mod tests {
    use super::*;
    use sensiblaw_pg_source_store::{
        AccessDisposition, AcquisitionObligation, AcquisitionPriorityReceipt,
        AcquisitionRouteCandidate, DurableAcquisitionQueue, DuplicateRelation,
        EvidenceIndependence, RecordAvailability,
    };

    fn queue() -> DurableAcquisitionQueue {
        let obligation = AcquisitionObligation {
            obligation_ref: "acquisition:1".into(),
            comparison_ref: "comparison:1".into(),
            residual_obligation_ref: "residual:1".into(),
            source_revision_refs: vec!["revision:a".into(), "revision:b".into()],
            target_description: "obtain source".into(),
            current_availability: RecordAvailability::NotLocated,
            authority_or_access_constraint_ref: "access:policy".into(),
            dependency_target_refs: vec!["assessment:a".into(), "assessment:b".into()],
            candidate_only: true,
            creates_semantic_authority: false,
            claim_truth_promoted: false,
        };
        let route = |id: &str, access, gain, cost| AcquisitionRouteCandidate {
            route_ref: id.into(), obligation_ref: obligation.obligation_ref.clone(),
            route_description: id.into(), source_locator_ref: format!("locator:{id}"),
            access_disposition: access, authority_receipt_ref: None,
            provenance_genealogy_ref: format!("genealogy:{id}"),
            independence: EvidenceIndependence::Unknown, independence_receipt_ref: None,
            duplicate_relation: DuplicateRelation::Unknown,
            information_gain: gain, dependency_closure_impact: 5,
            residual_coverage: 5, provenance_novelty: 5, acquisition_cost: cost,
            axis_estimation_receipt_ref: "estimate:1".into(), candidate_only: true,
            creates_acquisition_authority: false,
        };
        DurableAcquisitionQueue {
            obligation,
            routes: vec![
                route("blocked", AccessDisposition::RequiresAuthorization, 20, 20),
                route("cheap", AccessDisposition::Public, 10, 5),
                route("dominated", AccessDisposition::Public, 9, 6),
            ],
            priority: AcquisitionPriorityReceipt {
                obligation_ref: "acquisition:1".into(),
                frontier_route_refs: vec!["blocked".into(), "cheap".into()],
                executable_frontier_route_refs: vec!["cheap".into()],
                blocked_frontier_route_refs: vec!["blocked".into()],
                scalar_score_used: false,
                creates_semantic_authority: false,
                creates_screening_or_acquisition_decision: false,
            },
        }
    }

    #[test]
    fn sort_lens_cannot_change_frontier_membership_or_create_winner() {
        let q = queue();
        let stable = project_investigation(&q, None, InvestigationMode::Investigate,
            DisplaySortLens::StableRouteId, None, None).unwrap();
        let gain = project_investigation(&q, None, InvestigationMode::Investigate,
            DisplaySortLens::InformationGainDescending, Some("cheap"), None).unwrap();
        assert_eq!(stable.frontier_refs(), gain.frontier_refs());
        assert_eq!(gain.selected_route_ref.as_deref(), Some("cheap"));
        assert!(gain.no_overall_ranking);
        assert!(!gain.selection_creates_preference);
    }

    #[test]
    fn blocked_frontier_and_executable_dominated_are_separate_collections() {
        let q = queue();
        let view = project_investigation(&q, None, InvestigationMode::Investigate,
            DisplaySortLens::StableRouteId, None, None).unwrap();
        assert_eq!(view.frontier_blocked.iter().map(|r| r.route_ref.as_str()).collect::<Vec<_>>(), vec!["blocked"]);
        assert_eq!(view.frontier_executable.iter().map(|r| r.route_ref.as_str()).collect::<Vec<_>>(), vec!["cheap"]);
        assert_eq!(view.dominated.iter().map(|r| r.route_ref.as_str()).collect::<Vec<_>>(), vec!["dominated"]);
    }

    #[test]
    fn hidden_route_is_not_treated_as_absent() {
        let q = queue();
        let visible = std::collections::BTreeSet::from(["cheap".to_owned(), "dominated".to_owned()]);
        let view = project_investigation(&q, None, InvestigationMode::Investigate,
            DisplaySortLens::StableRouteId, None, Some(&visible)).unwrap();
        assert_eq!(view.world_route_count, 3);
        assert_eq!(view.visible_route_count, 2);
        assert_eq!(view.hidden_route_count, 1);
        assert!(!view.hidden_means_absent);
    }
}
