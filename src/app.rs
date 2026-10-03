use dioxus::prelude::*;

#[path = "app_legacy.rs"]
mod legacy;

#[cfg(feature = "production-data")]
#[path = "investigation_ui_v2.rs"]
mod investigation_ui;

pub fn app() -> Element {
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
