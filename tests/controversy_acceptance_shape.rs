#![cfg(feature = "production-data")]

use itir_dioxus::{
    controversy::MatterControversyWorkspace,
    controversy_acceptance::validate_controversy_acceptance_workspace,
};
use sensiblaw_pg_source_store::{DisagreementKind, ResponseMode};

fn fixture() -> MatterControversyWorkspace {
    MatterControversyWorkspace::contract_fixture_for_test(
        "matter:mabo",
        "controversy:mabo",
        ResponseMode::AdmitOccurrenceDisputeCharacterisation,
        DisagreementKind::LegalConsequence,
        ["order:applicant", "order:respondent"],
    )
}

#[test]
fn acceptance_requires_persisted_fibres() {
    let mut workspace = fixture();
    workspace.fibres.clear();
    let error = validate_controversy_acceptance_workspace(&workspace).unwrap_err();
    assert!(error.contains("fibre"));
}

#[test]
fn acceptance_requires_persisted_responses() {
    let mut workspace = fixture();
    workspace.responses.clear();
    let error = validate_controversy_acceptance_workspace(&workspace).unwrap_err();
    assert!(error.contains("response"));
}
