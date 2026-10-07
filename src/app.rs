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
