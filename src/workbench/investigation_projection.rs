#![cfg(feature = "production-data")]

use std::collections::{BTreeMap, BTreeSet};

use sensiblaw_pg_source_store::{
    route_frontier_dispositions, AcquisitionGovernancePacket, AcquisitionRouteCandidate,
    DurableAcquisitionQueue, RouteFrontierDisposition,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvestigationMode {
    Investigate,
    Inspect,
    Graph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplaySortLens {
    StableRouteId,
    InformationGainDescending,
    DependencyImpactDescending,
    ResidualCoverageDescending,
    ProvenanceNoveltyDescending,
    AcquisitionCostAscending,
}

impl DisplaySortLens {
    pub fn label(self) -> &'static str {
        match self {
            Self::StableRouteId => "stable route ID (nonsemantic)",
            Self::InformationGainDescending => "information gain (display lens only)",
            Self::DependencyImpactDescending => "dependency impact (display lens only)",
            Self::ResidualCoverageDescending => "residual coverage (display lens only)",
            Self::ProvenanceNoveltyDescending => "provenance novelty (display lens only)",
            Self::AcquisitionCostAscending => "acquisition cost (display lens only)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedInvestigationRoute {
    pub route_ref: String,
    pub route_description: String,
    pub source_locator_ref: String,
    pub access_state: String,
    pub provenance_genealogy_ref: String,
    pub independence_state: String,
    pub duplicate_relation: String,
    pub information_gain: u32,
    pub dependency_closure_impact: u32,
    pub residual_coverage: u32,
    pub provenance_novelty: u32,
    pub acquisition_cost: u32,
    pub axis_estimation_receipt_ref: String,
    pub disposition: RouteFrontierDisposition,
    pub potential_reopening_refs: Vec<String>,
    pub potential_reopening_label: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceProjection {
    pub packet_ref: String,
    pub purpose_ref: String,
    pub access_state: String,
    pub privacy_state: String,
    pub ai_use_state: String,
    pub service_change_state: String,
    pub evidence_state: String,
    pub control_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub certification_claim: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvestigationProjection {
    pub mode: InvestigationMode,
    pub display_sort_lens: DisplaySortLens,
    pub display_sort_label: &'static str,
    pub obligation_ref: String,
    pub comparison_ref: String,
    pub residual_obligation_ref: String,
    pub unresolved_target: String,
    pub current_availability: String,
    pub access_constraint_ref: String,
    pub frontier_executable: Vec<ProjectedInvestigationRoute>,
    pub frontier_blocked: Vec<ProjectedInvestigationRoute>,
    pub dominated: Vec<ProjectedInvestigationRoute>,
    pub selected_route_ref: Option<String>,
    pub governance: Option<GovernanceProjection>,
    pub world_route_count: usize,
    pub visible_route_count: usize,
    pub hidden_route_count: usize,
    pub no_overall_ranking: bool,
    pub selection_creates_preference: bool,
    pub hidden_means_absent: bool,
    pub sort_changes_frontier: bool,
    pub creates_semantic_authority: bool,
    pub acquisition_executed: bool,
}

impl InvestigationProjection {
    pub fn frontier_refs(&self) -> Vec<String> {
        let mut refs = self
            .frontier_executable
            .iter()
            .chain(self.frontier_blocked.iter())
            .map(|route| route.route_ref.clone())
            .collect::<Vec<_>>();
        refs.sort();
        refs
    }
}

fn project_governance(packet: &AcquisitionGovernancePacket) -> GovernanceProjection {
    GovernanceProjection {
        packet_ref: packet.packet_ref.clone(),
        purpose_ref: packet.purpose_ref.clone(),
        access_state: format!("{:?}", packet.access_state),
        privacy_state: format!("{:?}", packet.privacy_state),
        ai_use_state: format!("{:?}", packet.ai_use_state),
        service_change_state: packet.service_evidence_state.service_change_state.clone(),
        evidence_state: packet.service_evidence_state.evidence_state.clone(),
        control_refs: packet.control_refs.clone(),
        evidence_refs: packet.evidence_refs.clone(),
        certification_claim: packet.certification_claim,
    }
}

fn project_route(
    route: &AcquisitionRouteCandidate,
    disposition: RouteFrontierDisposition,
    potential_reopening_refs: &[String],
) -> ProjectedInvestigationRoute {
    ProjectedInvestigationRoute {
        route_ref: route.route_ref.clone(),
        route_description: route.route_description.clone(),
        source_locator_ref: route.source_locator_ref.clone(),
        access_state: format!("{:?}", route.access_disposition),
        provenance_genealogy_ref: route.provenance_genealogy_ref.clone(),
        independence_state: format!("{:?}", route.independence),
        duplicate_relation: format!("{:?}", route.duplicate_relation),
        information_gain: route.information_gain,
        dependency_closure_impact: route.dependency_closure_impact,
        residual_coverage: route.residual_coverage,
        provenance_novelty: route.provenance_novelty,
        acquisition_cost: route.acquisition_cost,
        axis_estimation_receipt_ref: route.axis_estimation_receipt_ref.clone(),
        disposition,
        potential_reopening_refs: potential_reopening_refs.to_vec(),
        potential_reopening_label: "could reopen (counterfactual only)",
    }
}

fn sort_routes(routes: &mut [ProjectedInvestigationRoute], lens: DisplaySortLens) {
    routes.sort_by(|left, right| {
        let primary = match lens {
            DisplaySortLens::StableRouteId => std::cmp::Ordering::Equal,
            DisplaySortLens::InformationGainDescending => right.information_gain.cmp(&left.information_gain),
            DisplaySortLens::DependencyImpactDescending => right.dependency_closure_impact.cmp(&left.dependency_closure_impact),
            DisplaySortLens::ResidualCoverageDescending => right.residual_coverage.cmp(&left.residual_coverage),
            DisplaySortLens::ProvenanceNoveltyDescending => right.provenance_novelty.cmp(&left.provenance_novelty),
            DisplaySortLens::AcquisitionCostAscending => left.acquisition_cost.cmp(&right.acquisition_cost),
        };
        primary.then_with(|| left.route_ref.cmp(&right.route_ref))
    });
}

pub fn project_investigation(
    queue: &DurableAcquisitionQueue,
    governance: Option<&AcquisitionGovernancePacket>,
    mode: InvestigationMode,
    sort_lens: DisplaySortLens,
    selected_route_ref: Option<&str>,
    visible_route_refs: Option<&BTreeSet<String>>,
) -> Result<InvestigationProjection, String> {
    if queue.priority.scalar_score_used
        || queue.priority.creates_semantic_authority
        || queue.priority.creates_screening_or_acquisition_decision
    {
        return Err("persisted acquisition priority crossed projection boundary".into());
    }
    if let Some(packet) = governance {
        if packet.obligation_ref != queue.obligation.obligation_ref
            || packet.comparison_ref != queue.obligation.comparison_ref
            || packet.source_revision_refs != queue.obligation.source_revision_refs
            || packet.creates_semantic_authority
            || packet.creates_access_authority
            || packet.creates_priority
            || packet.certification_claim
        {
            return Err("governance packet does not match visible lower-authority queue".into());
        }
    }

    let dispositions = route_frontier_dispositions(&queue.obligation, &queue.routes)
        .map_err(|error| error.to_string())?;
    let disposition_by_ref = dispositions
        .into_iter()
        .map(|row| (row.route_ref, row.disposition))
        .collect::<BTreeMap<_, _>>();

    let world_route_count = queue.routes.len();
    let visible_routes = queue
        .routes
        .iter()
        .filter(|route| {
            visible_route_refs
                .map(|visible| visible.contains(&route.route_ref))
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();
    let visible_route_count = visible_routes.len();
    let hidden_route_count = world_route_count.saturating_sub(visible_route_count);

    let mut frontier_executable = Vec::new();
    let mut frontier_blocked = Vec::new();
    let mut dominated = Vec::new();
    for route in visible_routes {
        let disposition = disposition_by_ref
            .get(&route.route_ref)
            .cloned()
            .ok_or_else(|| format!("missing disposition for {}", route.route_ref))?;
        let projected = project_route(
            route,
            disposition.clone(),
            &queue.obligation.dependency_target_refs,
        );
        match disposition {
            RouteFrontierDisposition::FrontierExecutable => frontier_executable.push(projected),
            RouteFrontierDisposition::FrontierBlocked => frontier_blocked.push(projected),
            RouteFrontierDisposition::Dominated { .. } => dominated.push(projected),
        }
    }
    sort_routes(&mut frontier_executable, sort_lens);
    sort_routes(&mut frontier_blocked, sort_lens);
    sort_routes(&mut dominated, sort_lens);

    let selected = selected_route_ref
        .filter(|selected| {
            frontier_executable
                .iter()
                .chain(frontier_blocked.iter())
                .chain(dominated.iter())
                .any(|route| route.route_ref == *selected)
        })
        .map(str::to_owned);

    Ok(InvestigationProjection {
        mode,
        display_sort_lens: sort_lens,
        display_sort_label: sort_lens.label(),
        obligation_ref: queue.obligation.obligation_ref.clone(),
        comparison_ref: queue.obligation.comparison_ref.clone(),
        residual_obligation_ref: queue.obligation.residual_obligation_ref.clone(),
        unresolved_target: queue.obligation.target_description.clone(),
        current_availability: format!("{:?}", queue.obligation.current_availability),
        access_constraint_ref: queue.obligation.authority_or_access_constraint_ref.clone(),
        frontier_executable,
        frontier_blocked,
        dominated,
        selected_route_ref: selected,
        governance: governance.map(project_governance),
        world_route_count,
        visible_route_count,
        hidden_route_count,
        no_overall_ranking: true,
        selection_creates_preference: false,
        hidden_means_absent: false,
        sort_changes_frontier: false,
        creates_semantic_authority: false,
        acquisition_executed: false,
    })
}

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
        assert!(!gain.sort_changes_frontier);
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
        let visible = BTreeSet::from(["cheap".to_owned(), "dominated".to_owned()]);
        let view = project_investigation(&q, None, InvestigationMode::Investigate,
            DisplaySortLens::StableRouteId, None, Some(&visible)).unwrap();
        assert_eq!(view.world_route_count, 3);
        assert_eq!(view.visible_route_count, 2);
        assert_eq!(view.hidden_route_count, 1);
        assert!(!view.hidden_means_absent);
    }
}
