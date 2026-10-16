#![cfg(feature = "production-data")]

use std::collections::BTreeSet;

use sensiblaw_core::matter_context::{
    project_matter_context, ContextProjectionCoordinate, MatterContext,
};
use sensiblaw_pg_source_store::{
    load_controversy_investigation_status, load_database_config,
    load_operational_outstanding_for_date, ControversyInvestigationStatus,
};
use sensiblaw_reader_model::{
    project_matter_acceptance, project_matter_workspace, project_operational_outstanding,
    KnowledgeTimelineEntry, MatterAcceptanceInput, MatterAcceptanceReceipt,
    MatterAcceptanceRoleCoordinate, MatterWorkspaceInput, MatterWorkspaceProjection,
    OperationalTimelineProjection,
};

use crate::controversy::{
    load_persisted_controversy_workspace, MatterControversyWorkspace,
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
    pub controversy_refs: Vec<String>,
    pub context: MatterContext,
    pub context_coordinates: Vec<ContextProjectionCoordinate>,
    pub knowledge_timeline: Vec<KnowledgeTimelineEntry>,
    pub legal_proof_refs: Vec<String>,
    pub research_refs: Vec<String>,
    pub work_product_refs: Vec<String>,
    pub handoff_refs: Vec<String>,
    pub no_event_refs: Vec<String>,
    pub acceptance_roles: Vec<MatterAcceptanceRoleCoordinate>,
    pub procedural_significance_review_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericMatterWorkspace {
    pub projection: MatterWorkspaceProjection,
    pub acceptance: MatterAcceptanceReceipt,
    pub event_refs: Vec<String>,
    pub operational_dates: Vec<String>,
    /// Persisted controversy objects are part of the Matter world, not a
    /// second application state.
    pub controversies: Vec<MatterControversyWorkspace>,
    /// Investigation/review/reopening state is projected back onto the exact
    /// open proof obligation that owns it.
    pub controversy_investigations: Vec<ControversyInvestigationStatus>,
}

/// Apply the ordinary MatterContext firewall to explicitly requested
/// controversy coordinates before any persisted controversy row is opened.
///
/// Naming a controversy is navigation state, not authorization. Hidden or
/// excluded coordinates are simply omitted from this projection; they are not
/// treated as absent or false in the canonical world.
pub fn visible_controversy_refs(
    requested_refs: &[String],
    context: &MatterContext,
    coordinates: &[ContextProjectionCoordinate],
) -> Result<Vec<String>, String> {
    let projection =
        project_matter_context(context, coordinates).map_err(|error| format!("{error:?}"))?;
    let visible = projection
        .included_refs
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut refs = requested_refs
        .iter()
        .filter(|value| !value.trim().is_empty())
        .filter(|value| visible.contains(value.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    refs.sort();
    refs.dedup();
    Ok(refs)
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

    let controversy_refs = visible_controversy_refs(
        &request.controversy_refs,
        &request.context,
        &request.context_coordinates,
    )?;

    let config = load_database_config(None).map_err(|error| error.to_string())?;

    let mut controversies = Vec::new();
    let mut controversy_investigations = Vec::new();
    for controversy_ref in &controversy_refs {
        let workspace =
            load_persisted_controversy_workspace(controversy_ref, &request.matter_ref)?;
        for obligation in workspace
            .obligations
            .iter()
            .filter(|obligation| obligation.is_open)
        {
            if let Some(status) = load_controversy_investigation_status(
                &config,
                controversy_ref,
                &obligation.obligation_ref,
            )
            .map_err(|error| error.to_string())?
            {
                if status.matter_ref != request.matter_ref
                    || status.controversy_ref != *controversy_ref
                    || status.legal_obligation_ref != obligation.obligation_ref
                    || status.creates_semantic_authority
                    || status.creates_legal_authority
                    || status.residual_resolved_by_system
                {
                    return Err(
                        "controversy investigation projection crossed Matter/authority boundary"
                            .into(),
                    );
                }
                controversy_investigations.push(status);
            }
        }
        controversies.push(workspace);
    }
    controversy_investigations.sort_by(|left, right| {
        left.controversy_ref
            .cmp(&right.controversy_ref)
            .then_with(|| left.legal_obligation_ref.cmp(&right.legal_obligation_ref))
    });

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
    let operational_outstanding = project_operational_outstanding(&operational_outstanding_states)
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

    let acceptance = project_matter_acceptance(&MatterAcceptanceInput {
        workspace: projection.clone(),
        no_event_refs: request.no_event_refs,
        role_coordinates: request.acceptance_roles,
        procedural_significance_review_refs: request.procedural_significance_review_refs,
    })
    .map_err(|error| format!("{error:?}"))?;

    if acceptance.creates_semantic_authority
        || acceptance.claim_truth_promoted
        || acceptance.canonical_world_mutated
    {
        return Err("Matter acceptance receipt crossed semantic boundary".into());
    }

    Ok(GenericMatterWorkspace {
        projection,
        acceptance,
        event_refs,
        operational_dates,
        controversies,
        controversy_investigations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sensiblaw_core::matter_context::{
        DisclosureBoundary, KnowledgeCutMembership, MatterConsumerRole, MatterPurpose,
    };

    fn test_context() -> MatterContext {
        MatterContext {
            matter_ref: "matter:a".into(),
            purpose_ref: MatterPurpose::LegalAdvocacy,
            active_consumer_role: MatterConsumerRole::Lawyer,
            disclosure_boundary: DisclosureBoundary::RoleScoped,
            knowledge_time_cut_ref: Some("cut:now".into()),
            sealed_refs: vec![],
            minimum_necessary: true,
            purpose_limited: true,
            access_logged: true,
            revocable: true,
            mutates_canonical_world: false,
            creates_semantic_authority: false,
            claim_truth_promoted: false,
        }
    }

    fn controversy_coordinate(reference: &str) -> ContextProjectionCoordinate {
        ContextProjectionCoordinate {
            semantic_ref: reference.into(),
            matter_ref: "matter:a".into(),
            allowed_roles: vec![MatterConsumerRole::Lawyer],
            allowed_purposes: vec![MatterPurpose::LegalAdvocacy],
            knowledge_membership: KnowledgeCutMembership::KnownAtCut,
            explicitly_selected: true,
            sealed: false,
        }
    }

    #[test]
    fn named_but_hidden_controversy_is_not_opened() {
        let requested = vec!["controversy:visible".into(), "controversy:hidden".into()];
        let mut hidden = controversy_coordinate("controversy:hidden");
        hidden.explicitly_selected = false;
        let refs = visible_controversy_refs(
            &requested,
            &test_context(),
            &[controversy_coordinate("controversy:visible"), hidden],
        )
        .expect("context projection is valid");

        assert_eq!(refs, vec!["controversy:visible"]);
    }

    #[test]
    fn request_rejects_context_for_a_different_matter_before_database_access() {
        let request = GenericMatterRequest {
            matter_ref: "matter:a".into(),
            event_refs: vec![],
            operational_dates: vec![],
            controversy_refs: vec![],
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
            no_event_refs: vec![],
            acceptance_roles: vec![],
            procedural_significance_review_refs: vec![],
        };

        assert_eq!(
            load_generic_matter_workspace(request),
            Err("MatterContext matter_ref does not match request".into())
        );
    }
}
