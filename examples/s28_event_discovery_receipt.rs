#![cfg(feature = "production-data")]

use itir_dioxus::workbench::event_discovery::load_production_event_discovery_workspace;

fn main() -> Result<(), String> {
    let model = load_production_event_discovery_workspace()?;

    println!("proposal_count={}", model.projection.proposals.len());
    println!("candidate_only={}", model.candidate_only);
    println!("creates_event_identity={}", model.creates_event_identity);
    println!("creates_semantic_authority={}", model.creates_semantic_authority);
    println!("claim_truth_promoted={}", model.claim_truth_promoted);

    if !model.candidate_only
        || model.creates_event_identity
        || model.creates_semantic_authority
        || model.claim_truth_promoted
    {
        return Err("event discovery crossed candidate-only boundary".into());
    }

    for view in &model.projection.proposals {
        if !view.requires_review || view.creates_event_identity {
            return Err(format!(
                "proposal {} bypasses EventAssembly review",
                view.proposal.proposal_ref
            ));
        }
        println!(
            "proposal={} observations={} source_families={} signal_kinds={}",
            view.proposal.proposal_ref,
            view.observation_count,
            view.source_family_count,
            view.signal_kind_count,
        );
    }

    Ok(())
}
