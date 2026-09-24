#![cfg(feature = "production-data")]

use itir_dioxus::workbench::{
    matter::load_generic_matter_workspace,
    matter_scope::load_matter_scope_manifest,
};

fn main() -> Result<(), String> {
    let scope_path = std::env::args()
        .nth(1)
        .ok_or_else(|| "usage: m13_mary_acceptance_receipt <matter-scope.json>".to_owned())?;

    let request = load_matter_scope_manifest(&scope_path)?;
    let model = load_generic_matter_workspace(request)?;
    let receipt = &model.acceptance;

    println!("matter_ref={}", receipt.matter_ref);
    println!(
        "source_reopenable_ref_count={}",
        receipt.source_reopenable_ref_count
    );
    println!("event_entry_count={}", receipt.event_entry_count);
    println!("knowledge_entry_count={}", receipt.knowledge_entry_count);
    println!("work_event_count={}", receipt.work_event_count);
    println!("suggested_join_count={}", receipt.suggested_join_count);
    println!("review_item_count={}", receipt.review_item_count);
    println!(
        "context_exclusion_count={}",
        receipt.context_exclusion_count
    );
    println!(
        "missing_date_event_count={}",
        receipt.missing_date_event_refs.len()
    );
    println!(
        "missing_actor_claim_count={}",
        receipt.missing_actor_claim_refs.len()
    );
    println!(
        "contradictory_relation_count={}",
        receipt.contradictory_relation_refs.len()
    );
    println!("no_event_count={}", receipt.no_event_refs.len());
    println!(
        "party_assertion_count={}",
        receipt.party_assertion_refs.len()
    );
    println!(
        "procedural_outcome_count={}",
        receipt.procedural_outcome_refs.len()
    );
    println!(
        "later_annotation_count={}",
        receipt.later_annotation_refs.len()
    );
    println!(
        "procedural_significance_open_count={}",
        receipt.procedural_significance_review_refs.len()
    );
    println!(
        "operational_carryover_count={}",
        receipt.operational_carryover_refs.len()
    );
    println!(
        "missing_date_means_event_did_not_happen={}",
        receipt.missing_date_means_event_did_not_happen
    );
    println!(
        "missing_actor_means_unknown_person={}",
        receipt.missing_actor_means_unknown_person
    );
    println!("no_event_means_false={}", receipt.no_event_means_false);
    println!(
        "creates_semantic_authority={}",
        receipt.creates_semantic_authority
    );
    println!("claim_truth_promoted={}", receipt.claim_truth_promoted);
    println!(
        "canonical_world_mutated={}",
        receipt.canonical_world_mutated
    );

    if receipt.missing_date_means_event_did_not_happen
        || receipt.missing_actor_means_unknown_person
        || receipt.no_event_means_false
        || receipt.creates_semantic_authority
        || receipt.claim_truth_promoted
        || receipt.canonical_world_mutated
    {
        return Err("M13 acceptance receipt crossed semantic boundary".into());
    }

    for reference in &receipt.missing_date_event_refs {
        println!("missing_date={reference}");
    }
    for reference in &receipt.missing_actor_claim_refs {
        println!("missing_actor={reference}");
    }
    for reference in &receipt.contradictory_relation_refs {
        println!("contradiction={reference}");
    }
    for reference in &receipt.no_event_refs {
        println!("no_event={reference}");
    }
    for reference in &receipt.party_assertion_refs {
        println!("party_assertion={reference}");
    }
    for reference in &receipt.procedural_outcome_refs {
        println!("procedural_outcome={reference}");
    }
    for reference in &receipt.later_annotation_refs {
        println!("later_annotation={reference}");
    }
    for reference in &receipt.procedural_significance_review_refs {
        println!("procedural_significance_open={reference}");
    }
    for reference in &receipt.operational_carryover_refs {
        println!("operational_carryover={reference}");
    }

    Ok(())
}
