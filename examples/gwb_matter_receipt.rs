#![cfg(feature = "production-data")]

use itir_dioxus::workbench::gwb_matter::load_gwb_matter_workspace;

fn main() -> Result<(), String> {
    let manifest_path = std::env::args()
        .nth(1)
        .ok_or_else(|| "usage: gwb_matter_receipt <reviewed-manifest.json>".to_owned())?;

    let model = load_gwb_matter_workspace(&manifest_path)?;

    println!("matter_ref={}", model.matter_ref);
    println!("handoff_ref={}", model.handoff_ref);
    println!("source_family_count={}", model.source_family_refs.len());
    println!("source_statement_count={}", model.source_statement_count);
    println!("event_count={}", model.event_refs.len());
    println!(
        "chronology_entry_count={}",
        model.timeline.chronology.entries.len()
    );
    println!(
        "proposition_view_count={}",
        model.timeline.chronology.proposition_views.len()
    );
    println!("review_item_count={}", model.review.queue.items.len());
    println!(
        "research_review_item_count={}",
        model.research_review_item_refs.len()
    );
    println!("candidate_only={}", model.candidate_only);
    println!(
        "creates_semantic_authority={}",
        model.creates_semantic_authority
    );
    println!("applicability_promoted={}", model.applicability_promoted);
    println!("claim_truth_promoted={}", model.claim_truth_promoted);

    if !model.candidate_only
        || model.creates_semantic_authority
        || model.applicability_promoted
        || model.claim_truth_promoted
    {
        return Err("GWB matter projection crossed non-promotion boundary".into());
    }

    if model.event_refs.is_empty() {
        return Err("GWB matter projection has no reviewed events".into());
    }
    if model.timeline.chronology.entries.is_empty() {
        return Err("GWB matter projection has no chronology entries".into());
    }

    Ok(())
}
