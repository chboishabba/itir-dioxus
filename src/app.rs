use dioxus::prelude::*;

#[path = "app_legacy.rs"]
mod legacy;

#[cfg(feature = "production-data")]
pub use legacy::ReviewWorkspaceView;

#[cfg(feature = "production-data")]
#[path = "investigation_ui_v2.rs"]
mod investigation_ui;

pub fn app() -> Element {
    #[cfg(feature = "production-data")]
    if let Ok(reference) = std::env::var("ITIR_INV_ACQUISITION_REF") {
        return match crate::workbench::investigation::load_investigation_queue(&reference) {
            Ok(model) => {
                let reviewed = model.reviewed_evidence.clone();
                rsx! {
                    document::Title { "ITIR Investigation Workbench" }
                    main {
                        style: "font-family:sans-serif;max-width:1200px;margin:0 auto;padding:1.5rem;",
                        if let Some(reviewed) = reviewed {
                            section {
                                style: "border-left:4px solid #6b4f9c;background:#faf8ff;padding:.9rem 1.1rem;margin-bottom:1rem;overflow-wrap:anywhere;",
                                h2 { "Persisted reviewed-evidence authority context" }
                                p { "Normative order: {reviewed.normative_order_ref}" }
                                p { "Evidence role: {reviewed.evidence_role_ref}" }
                                p { "Consumer: {reviewed.consumer_ref} · requirement: {reviewed.requirement_ref}" }
                                p { "Reviewed evidence: {reviewed.reviewed_evidence_ref}" }
                                p { "Review receipt: {reviewed.review_receipt_ref}" }
                                p { "Proposition: {reviewed.proposition_ref}" }
                                p { "Source revision: {reviewed.source_revision_ref} · exact span: {reviewed.exact_span_ref}" }
                                p {
                                    style: "font-size:.82rem;opacity:.76;",
                                    "Persisted review ≠ claim truth ≠ applicability ≠ legal authority. Normative-order coordinates are displayed, not coerced or ranked."
                                }
                            }
                        }
                        investigation_ui::InvestigationAcquisitionView { model }
                    }
                }
            },
            Err(error) => rsx! {
                document::Title { "ITIR investigation acquisition unavailable" }
                main {
                    style: "font-family:sans-serif;max-width:1000px;margin:0 auto;padding:2rem;",
                    h1 { "Investigation acquisition unavailable" }
                    p { "{error}" }
                    p { "No missing source is fabricated and no blocked route is treated as executable." }
                }
            },
        };
    }

    #[cfg(feature = "production-data")]
    if let Ok(scope_path) = std::env::var("SENSIBLAW_MATTER_SCOPE") {
        return match crate::workbench::matter_scope::load_matter_scope_manifest(&scope_path)
            .and_then(crate::workbench::matter::load_generic_matter_workspace)
        {
            Ok(model) => {
                let controversy = model.controversy.clone();
                rsx! {
                    document::Title { "SensibLaw Matter" }
                    main {
                        style: "font-family:sans-serif;max-width:1200px;margin:0 auto;padding:1.5rem;",
                        crate::controversy_ui::MatterControversyWorkspaceView { model: controversy }
                        crate::matter_ui::GenericMatterWorkspaceView { model }
                    }
                }
            },
            Err(error) => rsx! {
                document::Title { "SensibLaw Matter unavailable" }
                main {
                    style: "font-family:sans-serif;max-width:1000px;margin:0 auto;padding:2rem;",
                    h1 { "Matter unavailable" }
                    p { "{error}" }
                    p { "No controversy, review state or adjudicative conclusion is fabricated when persisted Matter state cannot be reopened." }
                }
            },
        };
    }

    legacy::app()
}