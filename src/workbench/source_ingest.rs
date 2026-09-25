#![cfg(feature = "production-data")]

use sensiblaw_pg_source_store::{
    LosslessBulkSourceCompilation, PlainTextSegmentationReceipt,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIngestCoverageReadModel {
    pub source_ref: String,
    pub source_revision_ref: String,
    pub structural_region_count: usize,
    pub semantic_candidate_region_count: usize,
    pub compiled_statement_count: usize,
    pub parser_residual_count: usize,
    pub candidate_pnf_count: usize,
    pub source_region_accounted_count: usize,
    pub source_region_loss_count: usize,
    pub source_coverage_complete: bool,
    pub candidate_only: bool,
    pub creates_semantic_authority: bool,
    pub claim_truth_promoted: bool,
    pub parse_failure_deletes_source: bool,
}

pub fn source_ingest_coverage_from_bulk(
    bulk: &LosslessBulkSourceCompilation,
) -> SourceIngestCoverageReadModel {
    SourceIngestCoverageReadModel {
        source_ref: bulk.source_ref.clone(),
        source_revision_ref: bulk.source_revision_ref.clone(),
        structural_region_count: bulk.exact_region_count,
        semantic_candidate_region_count: bulk.semantic_candidate_region_count,
        compiled_statement_count: bulk.compiled_statement_count,
        parser_residual_count: bulk.residuals.len(),
        candidate_pnf_count: bulk.candidate_pnf_count,
        source_region_accounted_count: bulk.source_region_accounted_count,
        source_region_loss_count: bulk.source_region_loss_count,
        source_coverage_complete: bulk.source_coverage_complete(),
        candidate_only: bulk.candidate_only,
        creates_semantic_authority: bulk.creates_semantic_authority,
        claim_truth_promoted: bulk.claim_truth_promoted,
        parse_failure_deletes_source: bulk.parse_failure_deletes_source,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LongDocumentIngestReadModel {
    pub structural: PlainTextSegmentationReceipt,
    pub coverage: SourceIngestCoverageReadModel,
}

pub fn long_document_ingest_read_model(
    structural: PlainTextSegmentationReceipt,
    bulk: &LosslessBulkSourceCompilation,
) -> Result<LongDocumentIngestReadModel, String> {
    if structural.source_ref != bulk.source_ref
        || structural.source_revision_ref != bulk.source_revision_ref
    {
        return Err("structural and semantic compilation receipts refer to different source revisions".into());
    }

    let coverage = source_ingest_coverage_from_bulk(bulk);
    if coverage.creates_semantic_authority
        || coverage.claim_truth_promoted
        || coverage.parse_failure_deletes_source
    {
        return Err("source ingest crossed non-promotion/lossless boundary".into());
    }

    Ok(LongDocumentIngestReadModel {
        structural,
        coverage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_read_model_keeps_parse_failure_separate_from_source_loss() {
        let bulk = LosslessBulkSourceCompilation {
            source_ref: "book:1".into(),
            source_revision_ref: "revision:1".into(),
            exact_region_count: 100,
            semantic_candidate_region_count: 80,
            compiled_statement_count: 79,
            candidate_pnf_count: 240,
            residuals: vec![sensiblaw_pg_source_store::RegionCompilationResidual {
                region_ref: "sentence:bad".into(),
                source_revision_ref: "revision:1".into(),
                parser_receipt_ref: "parse:bad".into(),
                error_ref: "fixture".into(),
                source_region_preserved: true,
                semantic_authority_created: false,
                claim_truth_promoted: false,
            }],
            transport_or_nonsemantic_region_count: 20,
            source_region_accounted_count: 100,
            source_region_loss_count: 0,
            candidate_only: true,
            creates_semantic_authority: false,
            applicability_promoted: false,
            claim_truth_promoted: false,
            parse_failure_deletes_source: false,
        };

        let model = source_ingest_coverage_from_bulk(&bulk);
        assert_eq!(model.parser_residual_count, 1);
        assert_eq!(model.source_region_loss_count, 0);
        assert!(model.source_coverage_complete);
        assert!(!model.parse_failure_deletes_source);
    }
}
