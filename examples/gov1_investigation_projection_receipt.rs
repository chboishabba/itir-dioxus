#![cfg(feature = "production-data")]

use itir_dioxus::workbench::investigation::{
    load_investigation_queue, project_investigation, DisplaySortLens, InvestigationMode,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let obligation_ref = std::env::var("ITIR_INV_ACQUISITION_REF")?;
    let workspace = load_investigation_queue(&obligation_ref)
        .map_err(|error| format!("load investigation queue: {error}"))?;

    let stable = project_investigation(
        &workspace.queue,
        workspace.governance.as_ref(),
        InvestigationMode::Investigate,
        DisplaySortLens::StableRouteId,
        None,
        None,
    )
    .map_err(|error| format!("stable projection: {error}"))?;
    let sorted = project_investigation(
        &workspace.queue,
        workspace.governance.as_ref(),
        InvestigationMode::Investigate,
        DisplaySortLens::InformationGainDescending,
        stable.frontier_refs().first().map(String::as_str),
        None,
    )
    .map_err(|error| format!("sorted projection: {error}"))?;

    if stable.frontier_refs() != sorted.frontier_refs() {
        return Err("display sort changed Pareto frontier membership".into());
    }
    if !stable.no_overall_ranking
        || stable.selection_creates_preference
        || stable.hidden_means_absent
        || stable.sort_changes_frontier
        || stable.creates_semantic_authority
        || stable.acquisition_executed
    {
        return Err("investigation projection crossed its UI authority firewall".into());
    }
    if stable
        .governance
        .as_ref()
        .is_some_and(|governance| governance.certification_claim)
    {
        return Err("governance projection exposed a certification claim".into());
    }

    println!("matter_ref={}", workspace.matter_ref);
    println!("obligation_ref={}", stable.obligation_ref);
    println!("world_routes={}", stable.world_route_count);
    println!("visible_routes={}", stable.visible_route_count);
    println!("hidden_routes={}", stable.hidden_route_count);
    println!("frontier_executable={}", stable.frontier_executable.len());
    println!("frontier_blocked={}", stable.frontier_blocked.len());
    println!("dominated={}", stable.dominated.len());
    println!("no_overall_ranking=true");
    println!("sort_changes_frontier=false");
    println!("selection_creates_preference=false");
    println!("hidden_means_absent=false");
    println!("semantic_authority_created=false");
    println!("acquisition_executed=false");
    Ok(())
}
