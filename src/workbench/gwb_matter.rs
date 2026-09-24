#![cfg(feature = "production-data")]

use std::collections::BTreeSet;

use sensiblaw_pg_source_store::{
    load_gwb_chronology_capstone_manifest, GwbReviewItemKind,
};
use sensiblaw_reader_model::project_review_queue;

use super::{
    review::{load_production_review_workspace, ProductionReviewWorkspace},
    timeline::{load_production_timeline, ProductionTimelineWorkspace},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GwbMatterWorkspace {
    pub matter_ref: String,
    pub handoff_ref: String,
    pub source_family_refs: Vec<String>,
    pub source_statement_count: usize,
    pub event_refs: Vec<String>,
    pub research_review_item_refs: Vec<String>,
    pub timeline: ProductionTimelineWorkspace,
    pub review: ProductionReviewWorkspace,
    pub candidate_only: bool,
    pub creates_semantic_authority: bool,
    pub applicability_promoted: bool,
    pub claim_truth_promoted: bool,
}

pub fn load_gwb_matter_workspace(
    manifest_path: &str,
) -> Result<GwbMatterWorkspace, String> {
    let manifest = load_gwb_chronology_capstone_manifest(manifest_path)
        .map_err(|error| error.to_string())?;

    let mut source_family_refs = manifest
        .statements
        .iter()
        .map(|statement| statement.source_family_ref.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    source_family_refs.sort();

    let mut event_refs = manifest
        .event_joins
        .iter()
        .map(|event| event.event_ref.clone())
        .collect::<Vec<_>>();
    event_refs.sort();
    event_refs.dedup();

    let timeline = load_production_timeline(&event_refs)?;

    let full_review = load_production_review_workspace()?;
    let claim_refs = timeline
        .chronology
        .proposition_views
        .iter()
        .flat_map(|view| view.leaves.iter().map(|leaf| leaf.claim_ref.clone()))
        .collect::<BTreeSet<_>>();
    let event_set = event_refs.iter().cloned().collect::<BTreeSet<_>>();

    let filtered_items = full_review
        .queue
        .items
        .iter()
        .filter(|item| {
            event_set.contains(&item.semantic_ref)
                || claim_refs.contains(&item.semantic_ref)
                || item
                    .affected_consumer_refs
                    .iter()
                    .any(|consumer| consumer.contains(&manifest.matter_ref))
        })
        .cloned()
        .collect::<Vec<_>>();
    let review_queue =
        project_review_queue(&filtered_items).map_err(|error| format!("{error:?}"))?;
    let review = ProductionReviewWorkspace {
        queue: review_queue,
        candidate_only: true,
        creates_semantic_authority: false,
        applicability_promoted: false,
        claim_truth_promoted: false,
    };

    let research_review_item_refs = manifest
        .review_items
        .iter()
        .filter(|item| {
            matches!(
                item.item_kind,
                GwbReviewItemKind::ResearchAcquisition
                    | GwbReviewItemKind::AuthorityFollow
            )
        })
        .map(|item| item.review_item_ref.clone())
        .collect::<Vec<_>>();

    Ok(GwbMatterWorkspace {
        matter_ref: manifest.matter_ref,
        handoff_ref: manifest.handoff_ref,
        source_family_refs,
        source_statement_count: manifest.statements.len(),
        event_refs,
        research_review_item_refs,
        timeline,
        review,
        candidate_only: true,
        creates_semantic_authority: false,
        applicability_promoted: false,
        claim_truth_promoted: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matter_projection_flags_are_non_promoting() {
        let workspace = GwbMatterWorkspace {
            matter_ref: "matter:gwb:test".into(),
            handoff_ref: "handoff:gwb:test".into(),
            source_family_refs: vec!["book".into(), "wikipedia".into()],
            source_statement_count: 2,
            event_refs: vec!["event:gwb:test".into()],
            research_review_item_refs: vec![],
            timeline: ProductionTimelineWorkspace {
                chronology: Default::default(),
                traces_by_event: Default::default(),
                candidate_only: true,
                creates_semantic_authority: false,
                creates_claim_truth: false,
            },
            review: ProductionReviewWorkspace {
                queue: sensiblaw_reader_model::ReviewQueueProjection {
                    items: vec![],
                    count_by_kind: Default::default(),
                    count_by_status: Default::default(),
                    candidate_only: true,
                    creates_semantic_authority: false,
                    applicability_promoted: false,
                    claim_truth_promoted: false,
                },
                candidate_only: true,
                creates_semantic_authority: false,
                applicability_promoted: false,
                claim_truth_promoted: false,
            },
            candidate_only: true,
            creates_semantic_authority: false,
            applicability_promoted: false,
            claim_truth_promoted: false,
        };
        assert!(workspace.candidate_only);
        assert!(!workspace.creates_semantic_authority);
        assert!(!workspace.applicability_promoted);
        assert!(!workspace.claim_truth_promoted);
    }
}
