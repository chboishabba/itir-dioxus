use dioxus::prelude::*;

#[path = "app_legacy.rs"]
mod legacy;

#[cfg(feature = "production-data")]
pub use legacy::ReviewWorkspaceView;

#[cfg(feature = "production-data")]
#[path = "investigation_ui_v2.rs"]
mod investigation_ui;

#[cfg(feature = "production-data")]
#[path = "controversy_ui.rs"]
mod controversy_ui;

pub fn app() -> Element {
    if let Ok(path) = std::env::var("ITIR_SUBLINEAR_METASYSTEM_PROJECTION") {
        return match crate::metasystem_ui::load_projection(&path) {
            Ok(model) => rsx! {
                document::Title { "ITIR Sublinear Topic Metasystem" }
                crate::metasystem_ui::MetasystemRouter { model }
            },
            Err(error) => rsx! {
                document::Title { "ITIR metasystem projection unavailable" }
                main {
                    style: "font-family:sans-serif;max-width:900px;margin:0 auto;padding:2rem;",
                    h1 { "Metasystem projection unavailable" }
                    p { "{error}" }
                    p { "No topic, proof, worker lane, or cross-project edge is synthesized." }
                }
            },
        };
    }

    // A scoped Matter is the composition root. Specialist selectors remain
    // useful as compatibility/deep-link entry points only when no Matter scope
    // is active; they must not steal focus from the unified world.
    #[cfg(feature = "production-data")]
    if let Ok(scope_path) = std::env::var("SENSIBLAW_MATTER_SCOPE") {
        let loaded = crate::workbench::matter_scope::load_matter_scope_manifest(&scope_path)
            .and_then(crate::workbench::matter::load_generic_matter_workspace);
        return match loaded {
            Ok(model) => rsx! {
                document::Title { "SensibLaw Matter" }
                main {
                    style: "font-family: sans-serif; max-width: 1180px; margin: 0 auto; padding: 2rem;",
                    crate::matter_closure_ui::MatterClosureView { model: model.clone() }
                    crate::matter_ui::GenericMatterWorkspaceView { model }
                }
            },
            Err(error) => rsx! {
                document::Title { "SensibLaw Matter · load error" }
                main {
                    style: "font-family: sans-serif; max-width: 900px; margin: 0 auto; padding: 2rem;",
                    h1 { "Matter could not be opened" }
                    p { "{error}" }
                    p {
                        style: "font-size: 0.85rem; opacity: 0.75;",
                        "The existing canonical world is unchanged. Fix the explicit scope/context or persisted-data receipt and reload."
                    }
                }
            },
        };
    }

    #[cfg(feature = "production-data")]
    if let Ok(controversy_ref) = std::env::var("ITIR_CONTROVERSY_REF") {
        let matter_ref = match std::env::var("ITIR_MATTER_REF") {
            Ok(value) if !value.trim().is_empty() => value,
            _ => {
                return rsx! {
                    document::Title { "ITIR controversy unavailable" }
                    main {
                        style: "font-family:sans-serif;max-width:1000px;margin:0 auto;padding:2rem;",
                        h1 { "Matter controversy unavailable" }
                        p { "ITIR_MATTER_REF is required whenever ITIR_CONTROVERSY_REF is selected." }
                        p { "No controversy is guessed and no first-match Matter is selected." }
                    }
                };
            }
        };
        return match crate::controversy::load_persisted_controversy_workspace(
            &controversy_ref,
            &matter_ref,
        ) {
            Ok(model) => rsx! {
                document::Title { "ITIR Matter Controversy" }
                main {
                    style: "font-family:sans-serif;max-width:1200px;margin:0 auto;padding:1.5rem;",
                    controversy_ui::MatterControversyView { model }
                }
            },
            Err(error) => rsx! {
                document::Title { "ITIR controversy unavailable" }
                main {
                    style: "font-family:sans-serif;max-width:1000px;margin:0 auto;padding:2rem;",
                    h1 { "Matter controversy unavailable" }
                    p { "{error}" }
                    p { "No source, review state, common ground, or adjudicative conclusion is fabricated." }
                }
            },
        };
    }

    #[cfg(feature = "production-data")]
    if let Ok(reference) = std::env::var("ITIR_INV_ACQUISITION_REF") {
        return match crate::workbench::investigation::load_investigation_queue(&reference) {
            Ok(model) => rsx! {
                document::Title { "ITIR Investigation Workbench" }
                main {
                    style: "font-family:sans-serif;max-width:1200px;margin:0 auto;padding:1.5rem;",
                    investigation_ui::InvestigationAcquisitionView { model }
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

    legacy::app()
}
