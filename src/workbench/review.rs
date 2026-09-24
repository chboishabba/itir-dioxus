#![cfg(feature = "production-data")]

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use sensiblaw_pg_source_store::{
    apply_persisted_review_command, load_database_config, load_review_queue,
    ReviewAction, ReviewCommand, ReviewItem, ReviewReceipt,
};
use sensiblaw_reader_model::{project_review_queue, ReviewQueueProjection};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionReviewWorkspace {
    pub queue: ReviewQueueProjection,
    pub candidate_only: bool,
    pub creates_semantic_authority: bool,
    pub applicability_promoted: bool,
    pub claim_truth_promoted: bool,
}

pub fn load_production_review_workspace() -> Result<ProductionReviewWorkspace, String> {
    let config = load_database_config(None).map_err(|error| error.to_string())?;
    let items = load_review_queue(&config).map_err(|error| error.to_string())?;
    let queue = project_review_queue(&items).map_err(|error| format!("{error:?}"))?;

    Ok(ProductionReviewWorkspace {
        queue,
        candidate_only: true,
        creates_semantic_authority: false,
        applicability_promoted: false,
        claim_truth_promoted: false,
    })
}

static REVIEW_COMMAND_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn execute_review_action(
    review_item_ref: &str,
    action: ReviewAction,
    reviewer_ref: &str,
    qualification_ref: Option<String>,
    evidence_request_ref: Option<String>,
) -> Result<(ReviewReceipt, ReviewItem), String> {
    if review_item_ref.trim().is_empty() || reviewer_ref.trim().is_empty() {
        return Err("review item and reviewer refs are required".into());
    }

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis();
    let ordinal = REVIEW_COMMAND_COUNTER.fetch_add(1, Ordering::Relaxed);
    let command = ReviewCommand {
        command_ref: format!(
            "review-command:{review_item_ref}:{reviewer_ref}:{millis}:{ordinal}"
        ),
        review_item_ref: review_item_ref.to_owned(),
        action,
        reviewer_ref: reviewer_ref.to_owned(),
        qualification_ref,
        evidence_request_ref,
    };

    let config = load_database_config(None).map_err(|error| error.to_string())?;
    apply_persisted_review_command(&config, &command)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_projection_shape_is_non_promoting() {
        let model = ProductionReviewWorkspace {
            queue: ReviewQueueProjection {
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
        };
        assert!(model.candidate_only);
        assert!(!model.creates_semantic_authority);
        assert!(!model.applicability_promoted);
        assert!(!model.claim_truth_promoted);
    }
}
