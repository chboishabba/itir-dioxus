#![cfg(feature = "production-data")]

use crate::controversy::MatterControversyWorkspace;

/// Validate the minimal persisted shape required before a controversy workspace
/// can be emitted as a live acceptance receipt.
///
/// This is deliberately stricter than the ordinary read projection: an empty
/// collection may be a valid display state, but it cannot pay an acceptance
/// claim that says the persisted Matter retains fibres, typed responses,
/// residuals and proof obligations.
pub fn validate_controversy_acceptance_workspace(
    workspace: &MatterControversyWorkspace,
) -> Result<(), String> {
    if workspace.fibres.is_empty() {
        return Err("acceptance requires at least one persisted proposition fibre".into());
    }
    if workspace.responses.is_empty() {
        return Err("acceptance requires at least one persisted typed response".into());
    }
    if workspace.residuals.is_empty() {
        return Err("acceptance requires at least one persisted controversy residual".into());
    }
    if workspace.obligations.is_empty() {
        return Err("acceptance requires at least one persisted proof obligation".into());
    }
    if workspace.solicitor.open_obligation_refs.is_empty() {
        return Err("acceptance lost the open proof obligation in the solicitor projection".into());
    }
    if workspace.solicitor.requested_discriminators.is_empty()
        || workspace.solicitor.target_evidence_queries.is_empty()
    {
        return Err("acceptance lost discriminator/acquisition intent".into());
    }
    if workspace.client.creates_semantic_authority
        || workspace.client.claim_truth_promoted
        || workspace.solicitor.creates_semantic_authority
        || workspace.solicitor.claim_truth_promoted
        || workspace.court.determines_credibility
        || workspace.court.determines_ultimate_fact
        || workspace.court.assigns_normative_weight
        || workspace.court.enters_final_judgment
    {
        return Err("acceptance crossed the no-adjudication/no-authority boundary".into());
    }
    Ok(())
}
