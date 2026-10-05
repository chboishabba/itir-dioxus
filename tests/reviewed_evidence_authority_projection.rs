#![cfg(feature = "production-data")]

use itir_dioxus::workbench::investigation::{
    project_review_authority_context, ReviewAuthorityProjectionInput,
};

fn input() -> ReviewAuthorityProjectionInput {
    ReviewAuthorityProjectionInput {
        case_ref: "inv-case:mabo:1".into(),
        matter_ref: "matter:mabo".into(),
        reviewed_evidence_ref: "reviewed-evidence:mabo:radical-title".into(),
        review_receipt_ref: "review:mabo:radical-title".into(),
        consumer_ref: "consumer:mabo:legal-follow".into(),
        requirement_ref: "requirement:mabo:radical-title".into(),
        evidence_role_ref: "evidence-role:primary-authority-support".into(),
        normative_order_ref: "normative-order:crown-municipal-australia".into(),
        proposition_ref: "proposition:mabo:radical-title".into(),
        source_revision_ref: "source-revision:mabo:brennan".into(),
        exact_span_ref: "span:mabo:brennan:radical-title".into(),
        candidate_only: true,
        creates_semantic_authority: false,
        applicability_promoted: false,
        claim_truth_promoted: false,
    }
}

#[test]
fn reviewed_evidence_projection_preserves_normative_order_without_promotion() {
    let projection = project_review_authority_context("matter:mabo", input()).unwrap();

    assert_eq!(
        projection.normative_order_ref,
        "normative-order:crown-municipal-australia"
    );
    assert_eq!(projection.reviewed_evidence_ref, "reviewed-evidence:mabo:radical-title");
    assert!(projection.candidate_only);
    assert!(!projection.creates_semantic_authority);
    assert!(!projection.applicability_promoted);
    assert!(!projection.claim_truth_promoted);
    assert!(!projection.persistence_creates_legal_authority);
}

#[test]
fn reviewed_evidence_projection_fails_closed_on_normative_order_or_authority_drift() {
    let mut blank_order = input();
    blank_order.normative_order_ref = "   ".into();
    assert!(project_review_authority_context("matter:mabo", blank_order).is_err());

    let mut promoted = input();
    promoted.claim_truth_promoted = true;
    assert!(project_review_authority_context("matter:mabo", promoted).is_err());

    assert!(project_review_authority_context("matter:other", input()).is_err());
}
