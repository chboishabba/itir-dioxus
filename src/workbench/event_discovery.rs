#![cfg(feature = "production-data")]

use sensiblaw_pg_source_store::{
    load_database_config, load_pending_event_join_proposals,
};
use sensiblaw_reader_model::{
    project_event_join_proposals, EventDiscoveryProjection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionEventDiscoveryWorkspace {
    pub projection: EventDiscoveryProjection,
    pub candidate_only: bool,
    pub creates_event_identity: bool,
    pub creates_semantic_authority: bool,
    pub claim_truth_promoted: bool,
}

pub fn load_production_event_discovery_workspace(
) -> Result<ProductionEventDiscoveryWorkspace, String> {
    let config = load_database_config(None).map_err(|error| error.to_string())?;
    let proposals =
        load_pending_event_join_proposals(&config).map_err(|error| error.to_string())?;
    let projection =
        project_event_join_proposals(&proposals).map_err(|error| format!("{error:?}"))?;

    Ok(ProductionEventDiscoveryWorkspace {
        projection,
        candidate_only: true,
        creates_event_identity: false,
        creates_semantic_authority: false,
        claim_truth_promoted: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_discovery_projection_shape_is_non_promoting() {
        let model = ProductionEventDiscoveryWorkspace {
            projection: EventDiscoveryProjection::default(),
            candidate_only: true,
            creates_event_identity: false,
            creates_semantic_authority: false,
            claim_truth_promoted: false,
        };
        assert!(model.candidate_only);
        assert!(!model.creates_event_identity);
        assert!(!model.creates_semantic_authority);
        assert!(!model.claim_truth_promoted);
    }
}
