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
    assert!(validate_controversy_acceptance_workspace(&workspace).is_err());
}

#[test]
fn acceptance_requires_persisted_responses() {
    let mut workspace = fixture();
    workspace.responses.clear();
    assert!(validate_controversy_acceptance_workspace(&workspace).is_err());
}
