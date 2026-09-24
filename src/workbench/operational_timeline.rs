#![cfg(feature = "production-data")]

use sensiblaw_pg_source_store::{
    load_database_config, load_operational_events_for_date,
    load_operational_semantic_links_for_event,
};
use sensiblaw_reader_model::{
    project_operational_timeline, OperationalTimelineProjection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionOperationalTimelineWorkspace {
    pub state_date: String,
    pub timeline: OperationalTimelineProjection,
    pub creates_semantic_authority: bool,
    pub pays_evidence: bool,
    pub claim_truth_promoted: bool,
}

pub fn load_production_operational_timeline(
    state_date: &str,
) -> Result<ProductionOperationalTimelineWorkspace, String> {
    if state_date.trim().is_empty() {
        return Err("state date is required".into());
    }

    let config = load_database_config(None).map_err(|error| error.to_string())?;
    let events =
        load_operational_events_for_date(&config, state_date)
            .map_err(|error| error.to_string())?;

    let mut links = Vec::new();
    for event in &events {
        links.extend(
            load_operational_semantic_links_for_event(
                &config,
                &event.operational_event_ref,
            )
            .map_err(|error| error.to_string())?,
        );
    }

    let timeline =
        project_operational_timeline(&events, &links)
            .map_err(|error| format!("{error:?}"))?;

    Ok(ProductionOperationalTimelineWorkspace {
        state_date: state_date.into(),
        timeline,
        creates_semantic_authority: false,
        pays_evidence: false,
        claim_truth_promoted: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_operational_projection_shape_is_non_promoting() {
        let model = ProductionOperationalTimelineWorkspace {
            state_date: "2026-09-24".into(),
            timeline: OperationalTimelineProjection::default(),
            creates_semantic_authority: false,
            pays_evidence: false,
            claim_truth_promoted: false,
        };
        assert!(!model.creates_semantic_authority);
        assert!(!model.pays_evidence);
        assert!(!model.claim_truth_promoted);
    }
}
