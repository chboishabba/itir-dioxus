#![cfg(feature = "production-data")]

use itir_dioxus::workbench::semantic_trace::load_semantic_traces_for_event;

fn main() -> Result<(), String> {
    let event_ref = std::env::args()
        .nth(1)
        .ok_or_else(|| "usage: m12_semantic_trace_receipt <event_ref>".to_string())?;

    let traces = load_semantic_traces_for_event(&event_ref)?;
    if traces.is_empty() {
        return Err(format!("no persisted semantic trace for {event_ref}"));
    }

    println!("event_ref={event_ref}");
    println!("trace_count={}", traces.len());
    for (index, trace) in traces.iter().enumerate() {
        trace.validate().map_err(|error| format!("{error:?}"))?;
        println!("trace[{index}].reverse={:?}", trace.reverse_source_chain());
        println!("trace[{index}].forward={:?}", trace.forward_downstream_chain());
        println!(
            "trace[{index}].non_promotion=authority:{} applicability:{} truth:{} payment:{}",
            trace.creates_semantic_authority,
            trace.applicability_promoted,
            trace.claim_truth_promoted,
            trace.creates_evidence_payment(),
        );
    }

    Ok(())
}
