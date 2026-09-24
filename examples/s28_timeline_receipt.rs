#![cfg(feature = "production-data")]

use itir_dioxus::workbench::timeline::load_production_timeline;

fn main() -> Result<(), String> {
    let event_refs = std::env::args().skip(1).collect::<Vec<_>>();
    if event_refs.is_empty() {
        return Err("usage: s28_timeline_receipt <event_ref> [event_ref ...]".into());
    }

    let model = load_production_timeline(&event_refs)?;
    println!("event_request_count={}", event_refs.len());
    println!("chronology_entry_count={}", model.chronology.entries.len());
    println!("proposition_view_count={}", model.chronology.proposition_views.len());
    println!("candidate_only={}", model.candidate_only);
    println!("creates_semantic_authority={}", model.creates_semantic_authority);
    println!("creates_claim_truth={}", model.creates_claim_truth);

    for entry in &model.chronology.entries {
        println!(
            "entry event={} placement={:?} coordinate={} observations={} statements={} claims={} relations={}",
            entry.event_ref,
            entry.placement,
            entry.display_coordinate,
            entry.observation_refs.len(),
            entry.statement_refs.len(),
            entry.claim_refs.len(),
            entry.contestation_relation_refs.len(),
        );
        if let Some(relative) = &entry.relative_event_ref {
            println!("  relative_event={relative}");
        }
        let traces = model
            .traces_by_event
            .get(&entry.event_ref)
            .cloned()
            .unwrap_or_default();
        println!("  source_trace_count={}", traces.len());
        for trace in traces {
            trace.validate().map_err(|error| format!("{error:?}"))?;
            println!("  reverse={:?}", trace.reverse_source_chain());
        }
    }

    for view in &model.chronology.proposition_views {
        println!(
            "proposition={} leaves={} relations={} orphan_relations={}",
            view.root.proposition_ref,
            view.leaves.len(),
            view.relations.len(),
            view.orphan_relation_refs.len(),
        );
    }

    Ok(())
}
