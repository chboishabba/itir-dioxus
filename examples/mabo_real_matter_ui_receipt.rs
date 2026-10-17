#![cfg(feature = "production-data")]

use itir_dioxus::controversy::{
    load_persisted_controversy_workspace, ControversyPersonaProjection,
};

fn env_ref(name: &str) -> String {
    std::env::var(name)
        .unwrap_or_else(|_| panic!("{name} must identify the already-persisted Mabo coordinate"))
}

fn main() -> Result<(), String> {
    let matter_ref = env_ref("ITIR_MABO_MATTER_REF");
    let controversy_ref = env_ref("ITIR_MABO_CONTROVERSY_REF");
    let workspace = load_persisted_controversy_workspace(&controversy_ref, &matter_ref)?;

    for (name, projected_matter, projected_controversy) in [
        (
            "client",
            workspace.client.matter_ref(),
            workspace.client.controversy_ref(),
        ),
        (
            "solicitor",
            workspace.solicitor.matter_ref(),
            workspace.solicitor.controversy_ref(),
        ),
        (
            "court",
            workspace.court.matter_ref(),
            workspace.court.controversy_ref(),
        ),
    ] {
        if projected_matter != matter_ref || projected_controversy != controversy_ref {
            return Err(format!("{name} persona left the canonical persisted Matter"));
        }
    }

    if workspace.client.creates_semantic_authority
        || workspace.client.claim_truth_promoted
        || workspace.solicitor.creates_semantic_authority
        || workspace.solicitor.claim_truth_promoted
        || workspace.court.determines_credibility
        || workspace.court.determines_ultimate_fact
        || workspace.court.assigns_normative_weight
        || workspace.court.enters_final_judgment
    {
        return Err("persona projection crossed the no-adjudication/no-authority boundary".into());
    }
    if workspace.residuals.is_empty() || workspace.obligations.is_empty() {
        return Err("Mabo Matter has no persisted controversy residual/proof obligation".into());
    }
    if workspace.solicitor.open_obligation_refs.is_empty() {
        return Err("Mabo solicitor projection lost the open proof obligation".into());
    }
    if workspace.solicitor.requested_discriminators.is_empty()
        || workspace.solicitor.target_evidence_queries.is_empty()
    {
        return Err("Mabo solicitor projection lost discriminator/acquisition intent".into());
    }

    println!("matter_ref={matter_ref}");
    println!("controversy_ref={controversy_ref}");
    println!("fibres={}", workspace.fibres.len());
    println!("responses={}", workspace.responses.len());
    println!("residuals={}", workspace.residuals.len());
    println!("open_obligations={}", workspace.solicitor.open_obligation_refs.len());
    println!("client_same_matter=true");
    println!("solicitor_same_matter=true");
    println!("court_same_matter=true");
    println!("determines_credibility=false");
    println!("determines_ultimate_fact=false");
    println!("assigns_normative_weight=false");
    println!("enters_final_judgment=false");
    Ok(())
}
