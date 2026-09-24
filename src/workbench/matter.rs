#![cfg(feature = "production-data")]

use std::collections::BTreeSet;

use sensiblaw_core::matter_context::{
    ContextProjectionCoordinate, MatterContext,
};
use sensiblaw_pg_source_store::{
    load_database_config, load_operational_outstanding_for_date,
};
use sensiblaw_reader_model::{
    project_matter_workspace, project_operational_outstanding,
    KnowledgeTimelineEntry, MatterWorkspaceInput, MatterWorkspaceProjection,
    OperationalTimelineProjection,
};

use super::{
    event_discovery::load_production_event_discovery_workspace,
    operational_timeline::load_production_operational_timeline,
    review::load_production_review_workspace,
    timeline::load_production_timeline,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericMatterRequest {
    pub matter_ref: String,
    pub event_refs: Vec<String>,
    pub operational_dates: Vec<String>,
    pub context: MatterContext,
    pub context_coordinates: Vec<ContextProjectionCoordinate>,
    pub knowledge_timeline: Vec<KnowledgeTimelineEntry>,
    pub legal_proof_refs: Vec<String>,
    pub research_refs: Vec<String>,
    pub work_product_refs: Vec<String>,
    pub handoff_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericMatterWorkspace {
    pub projection: MatterWorkspaceProjection,
    pub event_refs: Vec<String>,
    pub operational_dates: Vec<String>,
}

pub fn load_generic_matter_workspace(
    request: GenericMatterRequest,
) -> Result<GenericMatterWorkspace, String> {
    if request.matter_ref.trim().is_empty() {
        return Err("matter ref is required".into());
    }
    if request.context.matter_ref != request.matter_ref {
        return Err("MatterContext matter_ref does not match request".into());
    }

    let mut event_refs = request
        .event_refs
        .iter()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    event_refs.sort();
    event_refs.dedup();

    let timeline = load_production_timeline(&event_refs)?;

    let source_traces = timeline
        .traces_by_event
        .values()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();

    let join_proposals = load_production_event_discovery_workspace()?.projection;
    let review_queue = load_production_review_workspace()?.queue;
    let config = load_database_config(None).map_err(|error| error.to_string())?;

    let mut operational_dates = request
        .operational_dates
        .iter()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    operational_dates.sort();
    operational_dates.dedup();

    let mut operational_entries = Vec::new();
    let mut operational_targets = BTreeSet::new();
    for state_date in &operational_dates {
        let workspace = load_production_operational_timeline(state_date)?;
        for entry in workspace.timeline.entries {
            operational_targets.extend(entry.target_refs.iter().cloned());
            operational_entries.push(entry);
        }
    }
    operational_entries.sort_by(|left, right| {
        left.event
            .state_date
            .cmp(&right.event.state_date)
            .then_with(|| left.event.start_time_ref.cmp(&right.event.start_time_ref))
            .then_with(|| {
                left.event
                    .operational_event_ref
                    .cmp(&right.event.operational_event_ref)
            })
    });
    operational_entries.dedup_by(|left, right| {
        left.event.operational_event_ref == right.event.operational_event_ref
    });

    let operational_timeline = OperationalTimelineProjection {
        entries: operational_entries,
        linked_target_count: operational_targets.len(),
        creates_semantic_authority: false,
        pays_evidence: false,
        claim_truth_promoted: false,
    };

    let mut operational_outstanding_states = Vec::new();
    for state_date in &operational_dates {
        operational_outstanding_states.extend(
            load_operational_outstanding_for_date(&config, state_date)
                .map_err(|error| error.to_string())?,
        );
    }
    let operational_outstanding =
        project_operational_outstanding(&operational_outstanding_states)
            .map_err(|error| format!("{error:?}"))?;

    let input = MatterWorkspaceInput {
        matter_ref: request.matter_ref.clone(),
        context: request.context,
        context_coordinates: request.context_coordinates,
        source_traces,
        event_timeline: timeline.chronology,
        knowledge_timeline: request.knowledge_timeline,
        operational_timeline,
        operational_outstanding,
        join_proposals,
        review_queue,
        legal_proof_refs: request.legal_proof_refs,
        research_refs: request.research_refs,
        work_product_refs: request.work_product_refs,
        handoff_refs: request.handoff_refs,
    };

    let projection =
        project_matter_workspace(&input).map_err(|error| format!("{error:?}"))?;

    if projection.creates_semantic_authority
        || projection.claim_truth_promoted
        || projection.canonical_world_mutated
    {
        return Err("generic Matter projection crossed semantic boundary".into());
    }

    Ok(GenericMatterWorkspace {
        projection,
        event_refs,
        operational_dates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sensiblaw_core::matter_context::{
        DisclosureBoundary, MatterConsumerRole, MatterPurpose,
    };

    #[test]
    fn request_rejects_context_for_a_different_matter_before_database_access() {
        let request = GenericMatterRequest {
            matter_ref: "matter:a".into(),
            event_refs: vec![],
            operational_dates: vec![],
            context: MatterContext {
                matter_ref: "matter:b".into(),
                purpose_ref: MatterPurpose::PersonalReview,
                active_consumer_role: MatterConsumerRole::ProtectedSubject,
                disclosure_boundary: DisclosureBoundary::MatterInternal,
                knowledge_time_cut_ref: None,
                sealed_refs: vec![],
                minimum_necessary: true,
                purpose_limited: true,
                access_logged: true,
                revocable: true,
                mutates_canonical_world: false,
                creates_semantic_authority: false,
                claim_truth_promoted: false,
            },
            context_coordinates: vec![],
            knowledge_timeline: vec![],
            legal_proof_refs: vec![],
            research_refs: vec![],
            work_product_refs: vec![],
            handoff_refs: vec![],
        };

        assert_eq!(
            load_generic_matter_workspace(request),
            Err("MatterContext matter_ref does not match request".into())
        );
    }
}
