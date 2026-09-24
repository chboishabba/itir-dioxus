#![cfg(feature = "production-data")]

use sensiblaw_reader_model::{
    project_work_product_coverage, ForwardCoverageStatus,
    MatterOmissionJudgment, MatterWorkProductExpectation,
    ReverseCoverageStatus, WorkProductCoverageJudgment,
    WorkProductCoverageProjection, WorkProductMatterMatch,
    WorkProductPropositionOccurrence,
};

use super::matter::GenericMatterWorkspace;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkProductCoverageWorkspace {
    pub projection: WorkProductCoverageProjection,
    pub forward_supported_count: usize,
    pub forward_qualified_count: usize,
    pub forward_contradicted_count: usize,
    pub forward_unreviewed_count: usize,
    pub forward_unsupported_count: usize,
    pub reverse_represented_count: usize,
    pub reverse_possibly_omitted_count: usize,
    pub reverse_excluded_by_scope_count: usize,
    pub reverse_not_expected_count: usize,
    pub reverse_unreviewed_count: usize,
}

impl WorkProductCoverageWorkspace {
    #[must_use]
    pub fn forward_for_occurrence(
        &self,
        occurrence_ref: &str,
    ) -> Option<&WorkProductCoverageJudgment> {
        self.projection
            .forward
            .iter()
            .find(|item| item.occurrence_ref == occurrence_ref)
    }

    #[must_use]
    pub fn reverse_for_proposition(
        &self,
        proposition_ref: &str,
    ) -> Option<&MatterOmissionJudgment> {
        self.projection
            .reverse
            .iter()
            .find(|item| item.matter_proposition_ref == proposition_ref)
    }
}

pub fn project_generic_work_product_coverage(
    matter: &GenericMatterWorkspace,
    occurrences: &[WorkProductPropositionOccurrence],
    matches: &[WorkProductMatterMatch],
    expectations: &[MatterWorkProductExpectation],
) -> Result<WorkProductCoverageWorkspace, String> {
    let projection = project_work_product_coverage(
        &matter.projection,
        occurrences,
        matches,
        expectations,
    )
    .map_err(|error| format!("{error:?}"))?;

    if projection.creates_semantic_authority
        || projection.applicability_promoted
        || projection.claim_truth_promoted
        || projection.canonical_world_mutated
        || projection.supported_implies_legal_sufficiency
        || projection.contradicted_implies_false
        || projection.unsupported_implies_false
        || projection.omission_implies_should_include
        || projection.redaction_deletes_canonical_source
    {
        return Err("work-product coverage crossed semantic/product boundary".into());
    }

    let mut workspace = WorkProductCoverageWorkspace {
        projection,
        forward_supported_count: 0,
        forward_qualified_count: 0,
        forward_contradicted_count: 0,
        forward_unreviewed_count: 0,
        forward_unsupported_count: 0,
        reverse_represented_count: 0,
        reverse_possibly_omitted_count: 0,
        reverse_excluded_by_scope_count: 0,
        reverse_not_expected_count: 0,
        reverse_unreviewed_count: 0,
    };

    for item in &workspace.projection.forward {
        match item.status {
            ForwardCoverageStatus::Supported => workspace.forward_supported_count += 1,
            ForwardCoverageStatus::Qualified => workspace.forward_qualified_count += 1,
            ForwardCoverageStatus::Contradicted => {
                workspace.forward_contradicted_count += 1
            }
            ForwardCoverageStatus::Unreviewed => workspace.forward_unreviewed_count += 1,
            ForwardCoverageStatus::Unsupported => workspace.forward_unsupported_count += 1,
        }
    }

    for item in &workspace.projection.reverse {
        match item.status {
            ReverseCoverageStatus::Represented => workspace.reverse_represented_count += 1,
            ReverseCoverageStatus::PossiblyOmitted => {
                workspace.reverse_possibly_omitted_count += 1
            }
            ReverseCoverageStatus::ExcludedByScope => {
                workspace.reverse_excluded_by_scope_count += 1
            }
            ReverseCoverageStatus::NotExpectedForProduct => {
                workspace.reverse_not_expected_count += 1
            }
            ReverseCoverageStatus::UnreviewedForOmission => {
                workspace.reverse_unreviewed_count += 1
            }
        }
    }

    Ok(workspace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_enums_remain_directionally_distinct() {
        assert_ne!(
            format!("{:?}", ForwardCoverageStatus::Unsupported),
            format!("{:?}", ReverseCoverageStatus::PossiblyOmitted),
        );
    }
}
