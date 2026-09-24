#![cfg(feature = "production-data")]

use itir_dioxus::workbench::operational_timeline::load_production_operational_timeline;

fn main() -> Result<(), String> {
    let state_date = std::env::args()
        .nth(1)
        .ok_or_else(|| "usage: s28_operational_timeline_receipt <YYYY-MM-DD>".to_owned())?;
    let model = load_production_operational_timeline(&state_date)?;

    println!("state_date={}", model.state_date);
    println!("operational_event_count={}", model.timeline.entries.len());
    println!("linked_target_count={}", model.timeline.linked_target_count);
    println!(
        "creates_semantic_authority={}",
        model.creates_semantic_authority
    );
    println!("pays_evidence={}", model.pays_evidence);
    println!("claim_truth_promoted={}", model.claim_truth_promoted);

    if model.creates_semantic_authority
        || model.pays_evidence
        || model.claim_truth_promoted
    {
        return Err("operational timeline crossed semantic boundary".into());
    }

    for entry in &model.timeline.entries {
        println!(
            "operational_event={} producer={} start={} end={} targets={}",
            entry.event.operational_event_ref,
            entry.event.producer_event_ref,
            entry.event.start_time_ref,
            entry.event.end_time_ref,
            entry.target_refs.len(),
        );
        if entry.event.creates_semantic_authority
            || entry.event.pays_evidence
            || entry.event.claim_truth_promoted
        {
            return Err(format!(
                "operational event {} was semantically promoted",
                entry.event.operational_event_ref
            ));
        }
    }

    Ok(())
}
