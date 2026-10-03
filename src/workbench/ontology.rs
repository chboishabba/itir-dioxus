#![cfg(feature="production-data")]
//! WIKI-UI-1 — Dioxus reads source-pinned Lean finite-checker diagnostics.
//! Matter visibility is checked BEFORE the diagnostic is returned to UI.
//! Nothing in the GUI runs Lean, proves completeness, or authorizes edits.

use std::sync::atomic::{AtomicU64,Ordering};
use std::time::{SystemTime,UNIX_EPOCH};
use sensiblaw_pg_source_store::{
    apply_ontology_diagnostic_review, load_database_config,
    load_ontology_diagnostic, OntologyDiagnosticRead,
    ReviewAction, ReviewCommand, ReviewReceipt,
};
static REVIEW_ORDINAL:AtomicU64=AtomicU64::new(0);

/// Keep the same S30 visibility contract as the mixed-source review lane.
/// Merely knowing an opaque diagnostic ID grants no ability to read it.
fn authorized_scope(packet:&OntologyDiagnosticRead)->Result<(),String> {
    let manifest=std::env::var("SENSIBLAW_MATTER_SCOPE")
        .map_err(|_|"SENSIBLAW_MATTER_SCOPE is required".to_owned())?;
    let request=super::matter_scope::load_matter_scope_manifest(manifest)?;
    if request.matter_ref!=packet.consumer_scope_ref {
        return Err("diagnostic belongs to a different consumer scope".into());
    }
    let view=sensiblaw_core::matter_context::project_matter_context(
        &request.context,&request.context_coordinates,
    ).map_err(|e|format!("{e:?}"))?;
    let visible=view.included_refs.iter().collect::<std::collections::BTreeSet<_>>();
    if !visible.contains(&packet.packet.source_revision_ref) {
        return Err("ontology source revision excluded from MatterContext".into());
    }
    // A witness may point to a separate, private statement source. It
    // cannot be copied into a visible read model unless it is authorized.
    for witness in &packet.packet.witnesses {
        for source_ref in &witness.statement_refs {
            if !visible.contains(source_ref) {
                return Err("diagnostic witness excluded by MatterContext".into());
            }
        }
    }
    if view.canonical_world_mutated || view.invisibility_means_false {
        return Err("MatterContext projection crossed its scope firewall".into());
    }
    Ok(())
}
pub fn load_ontology_case(reference:&str)->Result<OntologyDiagnosticRead,String> {
    let config=load_database_config(None).map_err(|e|e.to_string())?;
    let packet=load_ontology_diagnostic(&config,reference)
        .map_err(|e|e.to_string())?
        .ok_or_else(||"unknown ontology diagnostic".to_owned())?;
    authorized_scope(&packet)?;
    Ok(packet)
}
pub fn review_ontology_case(
    reference:&str,action:ReviewAction,reviewer:&str,
    qualification_ref:Option<String>,evidence_request_ref:Option<String>,
)->Result<(ReviewReceipt,OntologyDiagnosticRead),String> {
    if reviewer.trim().is_empty() {return Err("reviewer identity required".into());}
    let original=load_ontology_case(reference)?;
    let config=load_database_config(None).map_err(|e|e.to_string())?;
    let now=SystemTime::now().duration_since(UNIX_EPOCH)
        .map_err(|e|e.to_string())?.as_nanos();
    let ordinal=REVIEW_ORDINAL.fetch_add(1,Ordering::Relaxed);
    let command=ReviewCommand {
        command_ref:format!("s29:ontology-command:{reference}:{now}:{ordinal}"),
        review_item_ref:original.review_item_ref.clone(),
        action,reviewer_ref:reviewer.into(),
        qualification_ref,evidence_request_ref,
    };
    // Re-evaluate scope for each write, never use stale Dioxus state.
    authorized_scope(&original)?;
    let (receipt,updated)=apply_ontology_diagnostic_review(
        &config,reference,&command,
    ).map_err(|e|e.to_string())?;
    authorized_scope(&updated)?;
    Ok((receipt,updated))
}
