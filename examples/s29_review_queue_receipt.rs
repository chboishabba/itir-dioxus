#![cfg(feature = "production-data")]

use itir_dioxus::workbench::review::load_production_review_workspace;

fn main() -> Result<(), String> {
    let model = load_production_review_workspace()?;
    println!("review_item_count={}", model.queue.items.len());
    println!("candidate_only={}", model.candidate_only);
    println!("creates_semantic_authority={}", model.creates_semantic_authority);
    println!("applicability_promoted={}", model.applicability_promoted);
    println!("claim_truth_promoted={}", model.claim_truth_promoted);

    for item in &model.queue.items {
        println!(
            "item={} kind={:?} status={:?} actions={} provenance={} sources={} consumers={}",
            item.review_item_ref,
            item.item_kind,
            item.current_status,
            item.available_actions.len(),
            item.provenance_refs.len(),
            item.source_refs.len(),
            item.affected_consumer_refs.len(),
        );
    }

    Ok(())
}
