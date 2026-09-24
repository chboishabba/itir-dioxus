#![cfg(feature = "production-data")]

use sensiblaw_pg_source_store::{
    load_observation_event_links_for_event, load_source_statement,
    load_statement_observation_links_for_observation, StatementObservationDisposition,
};
use sensiblaw_reader_model::{
    ParseTraceCoordinate, SemanticTracePath, StatementTraceCoordinate, TraceReviewState,
};

fn review_state(disposition: StatementObservationDisposition) -> TraceReviewState {
    match disposition {
        StatementObservationDisposition::Candidate => TraceReviewState::Unreviewed,
        StatementObservationDisposition::ParseReviewed => TraceReviewState::ParseReviewed,
        StatementObservationDisposition::SemanticallyAdmitted => {
            TraceReviewState::SemanticallyAdmitted
        }
        StatementObservationDisposition::Rejected => TraceReviewState::Rejected,
        StatementObservationDisposition::Abstained => TraceReviewState::Abstained,
        StatementObservationDisposition::Qualified => TraceReviewState::Qualified,
    }
}

pub fn load_semantic_traces_for_event(event_ref: &str) -> Result<Vec<SemanticTracePath>, String> {
    if event_ref.trim().is_empty() {
        return Err("semantic trace requires a non-empty event ref".into());
    }

    let config = sensiblaw_pg_source_store::load_database_config(None)
        .map_err(|error| error.to_string())?;
    let event_links = load_observation_event_links_for_event(&config, event_ref)
        .map_err(|error| error.to_string())?;

    let mut traces = Vec::new();
    for event_link in event_links {
        let statement_links = load_statement_observation_links_for_observation(
            &config,
            &event_link.observation_ref,
        )
        .map_err(|error| error.to_string())?;

        for statement_link in statement_links {
            let statement = load_source_statement(&config, &statement_link.statement_ref)
                .map_err(|error| error.to_string())?
                .ok_or_else(|| {
                    format!(
                        "statement {} disappeared while traversing event {}",
                        statement_link.statement_ref, event_ref
                    )
                })?;

            let trace = SemanticTracePath {
                focus_ref: event_ref.to_owned(),
                statement: StatementTraceCoordinate {
                    statement_ref: statement.statement_ref,
                    document_ref: statement.document_ref,
                    source_revision_ref: statement.source_revision_ref,
                    span_ref: statement.exact_span_ref,
                    literal_text: statement.literal_text,
                },
                parse: Some(ParseTraceCoordinate {
                    candidate_pnf_ref: statement_link.candidate_pnf_ref,
                    parser_receipt_ref: statement_link.parser_receipt_ref,
                    review_ref: statement_link.parse_review_ref,
                    admission_receipt_ref: statement_link.admission_receipt_ref,
                    review_state: review_state(statement_link.disposition),
                }),
                observation_ref: event_link.observation_ref.clone(),
                event_ref: Some(event_link.event_ref.clone()),
                claim_refs: Vec::new(),
                downstream_use_refs: Vec::new(),
                candidate_only: true,
                creates_semantic_authority: false,
                applicability_promoted: false,
                claim_truth_promoted: false,
            };
            trace.validate().map_err(|error| format!("{error:?}"))?;
            traces.push(trace);
        }
    }

    traces.sort_by(|left, right| {
        left.observation_ref
            .cmp(&right.observation_ref)
            .then_with(|| left.statement.statement_ref.cmp(&right.statement.statement_ref))
    });
    Ok(traces)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disposition_projection_does_not_promote_semantics() {
        assert_eq!(
            review_state(StatementObservationDisposition::Candidate),
            TraceReviewState::Unreviewed
        );
        assert_eq!(
            review_state(StatementObservationDisposition::SemanticallyAdmitted),
            TraceReviewState::SemanticallyAdmitted
        );
    }
}
