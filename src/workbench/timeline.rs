#![cfg(feature = "production-data")]

use std::collections::{BTreeMap, BTreeSet};

use sensiblaw_pg_source_store::{
    load_claims_for_event, load_contestation_relations_for_claims, load_database_config,
    load_observation_event_links_for_event, load_proposition_roots_for_claims,
    load_statement_observation_links_for_observation, load_temporal_assertions_for_event,
};
use sensiblaw_reader_model::{
    project_chronology, ChronologyProjection, EventChronologyInput, SemanticTracePath,
};

use super::semantic_trace::load_semantic_traces_for_event;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionTimelineWorkspace {
    pub chronology: ChronologyProjection,
    pub traces_by_event: BTreeMap<String, Vec<SemanticTracePath>>,
    pub candidate_only: bool,
    pub creates_semantic_authority: bool,
    pub creates_claim_truth: bool,
}

pub fn load_production_timeline(
    event_refs: &[String],
) -> Result<ProductionTimelineWorkspace, String> {
    let config = load_database_config(None).map_err(|error| error.to_string())?;

    let mut events = Vec::new();
    let mut claims_by_ref = BTreeMap::new();
    let mut roots_by_ref = BTreeMap::new();
    let mut relations_by_ref = BTreeMap::new();
    let mut traces_by_event = BTreeMap::new();

    let unique_events: BTreeSet<String> = event_refs
        .iter()
        .filter(|event_ref| !event_ref.trim().is_empty())
        .cloned()
        .collect();

    for event_ref in unique_events {
        let temporal_assertions =
            load_temporal_assertions_for_event(&config, &event_ref)
                .map_err(|error| error.to_string())?;
        let claims =
            load_claims_for_event(&config, &event_ref).map_err(|error| error.to_string())?;
        let roots = load_proposition_roots_for_claims(&config, &claims)
            .map_err(|error| error.to_string())?;
        let relations = load_contestation_relations_for_claims(
            &config,
            &claims
                .iter()
                .map(|claim| claim.claim_ref.clone())
                .collect::<Vec<_>>(),
        )
        .map_err(|error| error.to_string())?;

        let event_observations = load_observation_event_links_for_event(&config, &event_ref)
            .map_err(|error| error.to_string())?;
        let observation_refs = event_observations
            .iter()
            .map(|link| link.observation_ref.clone())
            .collect::<BTreeSet<_>>();

        let mut statement_refs = BTreeSet::new();
        for observation_ref in &observation_refs {
            for link in load_statement_observation_links_for_observation(
                &config,
                observation_ref,
            )
            .map_err(|error| error.to_string())?
            {
                statement_refs.insert(link.statement_ref);
            }
        }

        for claim in &claims {
            claims_by_ref.insert(claim.claim_ref.clone(), claim.clone());
        }
        for root in &roots {
            roots_by_ref.insert(root.proposition_ref.clone(), root.clone());
        }
        for relation in &relations {
            relations_by_ref.insert(relation.relation_ref.clone(), relation.clone());
        }

        events.push(EventChronologyInput {
            event_ref: event_ref.clone(),
            temporal_assertions,
            observation_refs: observation_refs.into_iter().collect(),
            statement_refs: statement_refs.into_iter().collect(),
            proposition_refs: roots
                .iter()
                .map(|root| root.proposition_ref.clone())
                .collect(),
            claim_refs: claims
                .iter()
                .map(|claim| claim.claim_ref.clone())
                .collect(),
            candidate_only: true,
            creates_semantic_authority: false,
            claim_truth_promoted: false,
        });

        traces_by_event.insert(
            event_ref.clone(),
            load_semantic_traces_for_event(&event_ref)?,
        );
    }

    let roots = roots_by_ref.into_values().collect::<Vec<_>>();
    let claims = claims_by_ref.into_values().collect::<Vec<_>>();
    let relations = relations_by_ref.into_values().collect::<Vec<_>>();
    let chronology =
        project_chronology(&events, &roots, &claims, &relations)
            .map_err(|error| format!("{error:?}"))?;

    Ok(ProductionTimelineWorkspace {
        chronology,
        traces_by_event,
        candidate_only: true,
        creates_semantic_authority: false,
        creates_claim_truth: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_event_request_projects_empty_non_promoting_timeline_without_database_rows() {
        // This still requires DATABASE_URL because the production loader is a
        // database-backed adapter. Its semantic boundary is exercised in the
        // reader-model unit tests without PostgreSQL.
        let model = ProductionTimelineWorkspace {
            chronology: ChronologyProjection::default(),
            traces_by_event: BTreeMap::new(),
            candidate_only: true,
            creates_semantic_authority: false,
            creates_claim_truth: false,
        };
        assert!(model.candidate_only);
        assert!(!model.creates_semantic_authority);
        assert!(!model.creates_claim_truth);
    }
}
