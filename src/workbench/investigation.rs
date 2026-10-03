#![cfg(feature="production-data")]
//! ITIR-INV-1 read model for proof-directed acquisition.
//! Dioxus displays persisted acquisition obligations and Pareto routes under
//! the existing MatterContext. It does not execute searches, grant access,
//! acquire a source, or mutate semantic/review state.

use sensiblaw_pg_source_store::{
    load_acquisition_queue,load_database_config,
    DurableAcquisitionQueue,
};

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct InvestigationQueueWorkspace {
    pub queue:DurableAcquisitionQueue,
    pub matter_ref:String,
    pub creates_semantic_authority:bool,
    pub acquisition_executed:bool,
}

fn authorize(queue:&DurableAcquisitionQueue)->Result<String,String>{
    let manifest=std::env::var("SENSIBLAW_MATTER_SCOPE")
        .map_err(|_|"SENSIBLAW_MATTER_SCOPE is required".to_owned())?;
    let request=super::matter_scope::load_matter_scope_manifest(&manifest)?;
    let projection=sensiblaw_core::matter_context::project_matter_context(
        &request.context,&request.context_coordinates,
    ).map_err(|e|format!("{e:?}"))?;
    if queue.obligation.source_revision_refs.iter().any(|source|
        !projection.included_refs.iter().any(|visible|visible==source)){
        return Err("one or more acquisition-parent source revisions are excluded by MatterContext".into());
    }
    if projection.canonical_world_mutated||projection.invisibility_means_false{
        return Err("MatterContext crossed its disclosure/semantic firewall".into());
    }
    Ok(request.matter_ref)
}

pub fn load_investigation_queue(
    obligation_ref:&str,
)->Result<InvestigationQueueWorkspace,String>{
    let config=load_database_config(None).map_err(|e|e.to_string())?;
    let queue=load_acquisition_queue(&config,obligation_ref)
        .map_err(|e|e.to_string())?
        .ok_or_else(||"unknown investigation acquisition obligation".to_owned())?;
    let matter_ref=authorize(&queue)?;
    Ok(InvestigationQueueWorkspace{
        queue,matter_ref,
        creates_semantic_authority:false,
        acquisition_executed:false,
    })
}
