#![cfg(feature = "production-data")]

use itir_dioxus::workbench::{
    matter::load_generic_matter_workspace,
    matter_scope::load_matter_scope_manifest,
};

fn main() -> Result<(), String> {
    let scope_path = std::env::args()
        .nth(1)
        .ok_or_else(|| "usage: s30_generic_matter_receipt <matter-scope.json>".to_owned())?;

    let request = load_matter_scope_manifest(&scope_path)?;
    let model = load_generic_matter_workspace(request)?;
    let projection = &model.projection;

    println!("matter_ref={}", projection.matter_ref);
    println!("visible_ref_count={}", projection.context_projection.included_refs.len());
    println!("context_exclusion_count={}", projection.context_projection.exclusions.len());
    println!("source_trace_count={}", projection.source_traces.len());
    println!("event_timeline_count={}", projection.event_timeline.entries.len());
    println!("knowledge_timeline_count={}", projection.knowledge_timeline.len());
    println!("operational_timeline_count={}", projection.operational_timeline.entries.len());
    println!("suggested_join_count={}", projection.join_proposals.proposals.len());
    println!("review_item_count={}", projection.review_queue.items.len());
    println!("legal_proof_ref_count={}", projection.legal_proof_refs.len());
    println!("research_ref_count={}", projection.research_refs.len());
    println!("work_product_ref_count={}", projection.work_product_refs.len());
    println!("handoff_ref_count={}", projection.handoff_refs.len());
    println!("canonical_world_mutated={}", projection.canonical_world_mutated);
    println!("creates_semantic_authority={}", projection.creates_semantic_authority);
    println!("claim_truth_promoted={}", projection.claim_truth_promoted);

    if projection.canonical_world_mutated
        || projection.creates_semantic_authority
        || projection.claim_truth_promoted
    {
        return Err("Matter projection crossed semantic boundary".into());
    }

    Ok(())
}
