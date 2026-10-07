#![cfg(feature = "production-data")]

use itir_dioxus::controversy::{
    ControversyPersonaProjection, MatterControversyWorkspace,
};
use sensiblaw_pg_source_store::{DisagreementKind, ResponseMode};

#[test]
fn persona_projections_share_one_matter_and_preserve_typed_dispute() {
    let model = MatterControversyWorkspace::contract_fixture_for_test(
        "matter:1",
        "controversy:1",
        ResponseMode::AdmitOccurrenceDisputeCharacterisation,
        DisagreementKind::Characterisation,
        ["order:indigenous", "order:crown"],
    );

    assert_eq!(model.client.matter_ref(), "matter:1");
    assert_eq!(model.solicitor.matter_ref(), "matter:1");
    assert_eq!(model.court.matter_ref(), "matter:1");
    assert_eq!(model.client.controversy_ref(), model.court.controversy_ref());
    assert_eq!(
        model.solicitor.response_modes,
        vec![ResponseMode::AdmitOccurrenceDisputeCharacterisation]
    );
    assert_eq!(model.court.disagreement_kinds, vec![DisagreementKind::Characterisation]);
    assert!(model.client.normative_order_refs.contains(&"order:indigenous".into()));
    assert!(model.client.normative_order_refs.contains(&"order:crown".into()));
    assert!(!model.court.determines_credibility);
    assert!(!model.court.determines_ultimate_fact);
    assert!(!model.court.assigns_normative_weight);
    assert!(!model.court.enters_final_judgment);
}
