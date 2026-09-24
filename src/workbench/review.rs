#![cfg(feature = "production-data")]

use sensiblaw_pg_source_store::{load_database_config, load_review_queue};
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
