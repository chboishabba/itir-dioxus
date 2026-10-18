#![cfg(feature="production-data")]
//! ITIR-INV-1 read model for proof-directed acquisition.
//! Dioxus displays persisted acquisition obligations and Pareto routes under
//! the existing MatterContext. It does not execute searches, grant access,
//! acquire a source, create graph edges, or mutate semantic/review state.

#[path = "investigation_projection.rs"]
pub mod projection;
pub use projection::*;

#[path = "investigation_graph.rs"]
pub mod graph;
pub use graph::*;

use std::collections::BTreeSet;

use sensiblaw_pg_source_store::{
    load_acquisition_queue, load_bound_investigation_graph_projection,
    load_database_config, load_inv_governance_packet,
    AcquisitionGovernancePacket, DurableAcquisitionQueue, InvestigationGraphBinding,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvestigationGraphNode {
    pub semantic_ref: String,
    pub kind: String,
    pub label: String,
    pub source_refs: Vec<String>,
    pub provenance_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvestigationGraphEdge {
    pub semantic_ref: String,
    pub from_ref: String,
    pub to_ref: String,
    pub relation: String,
    pub source_refs: Vec<String>,
    pub provenance_refs: Vec<String>,
    pub challengeable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvestigationGraphProjection {
    pub binding: InvestigationGraphBinding,
    pub projection_ref: String,
    pub document_ref: String,
    pub nodes: Vec<InvestigationGraphNode>,
    pub edges: Vec<InvestigationGraphEdge>,
    pub derived_only: bool,
    pub challengeable: bool,
    pub creates_semantic_authority: bool,
    pub creates_graph_edges: bool,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct InvestigationQueueWorkspace {
    pub queue:DurableAcquisitionQueue,
    pub governance:Option<AcquisitionGovernancePacket>,
    pub graph:Option<InvestigationGraphProjection>,
    pub matter_ref:String,
    pub creates_semantic_authority:bool,
    pub acquisition_executed:bool,
}

#[derive(Debug)]
struct MatterAuthorization {
    matter_ref: String,
    included_refs: BTreeSet<String>,
}

fn authorize(queue:&DurableAcquisitionQueue)->Result<MatterAuthorization,String>{
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
    Ok(MatterAuthorization {
        matter_ref: request.matter_ref,
        included_refs: projection.included_refs.into_iter().collect(),
    })
}

fn load_bound_graph(
    config:&sensiblaw_pg_source_store::DatabaseConfig,
    queue:&DurableAcquisitionQueue,
    authorization:&MatterAuthorization,
)->Result<Option<InvestigationGraphProjection>,String>{
    let Some(bound)=load_bound_investigation_graph_projection(
        config,&queue.obligation.obligation_ref,&authorization.matter_ref,
    ).map_err(|e|e.to_string())? else {
        return Ok(None);
    };
    let binding=bound.binding;
    let persisted=bound.projection;
    if persisted.source_refs.iter().any(|source|
        !authorization.included_refs.contains(source)) {
        return Err("one or more graph source refs are excluded by MatterContext".into());
    }
    if !binding.derived_only||!binding.challengeable||binding.creates_graph_edges
        ||binding.creates_semantic_authority||binding.creates_access_authority
        ||binding.creates_acquisition_state {
        return Err("persisted investigation graph binding crossed its projection firewall".into());
    }

    let nodes=persisted.legal_follow_graph.nodes.into_iter().map(|node|
        InvestigationGraphNode {
            semantic_ref:node.semantic_ref,
            kind:node.semantic_kind,
            label:node.label,
            source_refs:node.source_refs,
            provenance_refs:node.provenance_refs,
        }).collect();
    let edges=persisted.legal_follow_graph.edges.into_iter().map(|edge|
        InvestigationGraphEdge {
            semantic_ref:edge.semantic_ref,
            from_ref:edge.from_ref,
            to_ref:edge.to_ref,
            relation:edge.relation,
            source_refs:edge.source_refs,
            provenance_refs:edge.provenance_refs,
            challengeable:edge.challengeable,
        }).collect();
    Ok(Some(InvestigationGraphProjection {
        projection_ref:persisted.legal_follow_graph.projection_ref,
        document_ref:persisted.legal_follow_graph.document_ref,
        nodes,edges,binding,
        derived_only:true,challengeable:true,
        creates_semantic_authority:false,
        creates_graph_edges:false,
    }))
}

/// Reopen Mabo's legal proof graph through the SAME persisted binding and
/// pnf_follow legal-follow loader used by investigation GraphIr. A legal proof
/// obligation is not an INV relational-acquisition obligation; do not synthesize
/// a comparison or route to make it fit the old acquisition entry point.
/// Absence stays None, and hidden MatterContext sources fail closed.
pub fn load_mabo_legal_proof_graph(
    controversy_ref: &str,
    legal_obligation_ref: &str,
) -> Result<Option<InvestigationGraphProjection>, String> {
    if controversy_ref != sensiblaw_pg_source_store::MABO_CONTROVERSY_REF
        || legal_obligation_ref.trim().is_empty() {
        return Err("not a persisted Mabo legal proof obligation".into());
    }
    let config = load_database_config(None).map_err(|e| e.to_string())?;
    let matter = sensiblaw_pg_source_store::load_legal_controversy_matter(
        &config, controversy_ref,
    ).map_err(|e| e.to_string())?;
    if matter.creates_semantic_authority || matter.creates_legal_authority
        || matter.applicability_promoted || matter.claim_truth_promoted
        || !matter.obligation_refs.iter().any(|r| r == legal_obligation_ref) {
        return Err("Mabo proof graph is not owned by a non-promoting Matter".into());
    }
    let manifest = std::env::var("SENSIBLAW_MATTER_SCOPE")
        .map_err(|_| "SENSIBLAW_MATTER_SCOPE is required".to_owned())?;
    let request = super::matter_scope::load_matter_scope_manifest(&manifest)?;
    let visible = sensiblaw_core::matter_context::project_matter_context(
        &request.context, &request.context_coordinates,
    ).map_err(|e| format!("{e:?}"))?;
    if request.matter_ref != matter.matter_ref
        || visible.canonical_world_mutated || visible.invisibility_means_false {
        return Err("Mabo graph MatterContext cannot authorize this Matter".into());
    }
    let Some(bound) = load_bound_investigation_graph_projection(
        &config, legal_obligation_ref, &matter.matter_ref,
    ).map_err(|e| e.to_string())? else {
        // The graph canvas must not invent topology on a missing binding.
        return Ok(None);
    };
    if bound.binding.obligation_ref != legal_obligation_ref
        || bound.binding.matter_ref != matter.matter_ref
        || !bound.binding.derived_only || !bound.binding.challengeable
        || bound.binding.creates_graph_edges
        || bound.binding.creates_semantic_authority
        || bound.binding.creates_access_authority
        || bound.binding.creates_acquisition_state {
        return Err("Mabo legal proof graph binding crossed authority boundary".into());
    }
    let included = visible.included_refs.into_iter().collect::<BTreeSet<_>>();
    if bound.projection.source_refs.iter().any(|r| !included.contains(r)) {
        return Err("Mabo graph includes a source excluded by MatterContext".into());
    }
    let graph = bound.projection.legal_follow_graph;
    if graph.projection_ref != bound.binding.projection_ref
        || !graph.derived_only || !graph.challengeable
        || graph.nodes.is_empty() || graph.edges.is_empty() {
        return Err("Mabo persisted graph is not a challengeable legal-follow graph".into());
    }
    Ok(Some(InvestigationGraphProjection {
        binding: bound.binding,
        projection_ref: graph.projection_ref,
        document_ref: graph.document_ref,
        nodes: graph.nodes.into_iter().map(|n| InvestigationGraphNode {
            semantic_ref: n.semantic_ref,
            kind: n.semantic_kind,
            label: n.label,
            source_refs: n.source_refs,
            provenance_refs: n.provenance_refs,
        }).collect(),
        edges: graph.edges.into_iter().map(|e| InvestigationGraphEdge {
            semantic_ref: e.semantic_ref,
            from_ref: e.from_ref,
            to_ref: e.to_ref,
            relation: e.relation,
            source_refs: e.source_refs,
            provenance_refs: e.provenance_refs,
            challengeable: e.challengeable,
        }).collect(),
        derived_only: true,
        challengeable: true,
        creates_semantic_authority: false,
        creates_graph_edges: false,
    }))
}

pub fn load_investigation_queue(
    obligation_ref:&str,
)->Result<InvestigationQueueWorkspace,String>{
    let config=load_database_config(None).map_err(|e|e.to_string())?;
    let queue=load_acquisition_queue(&config,obligation_ref)
        .map_err(|e|e.to_string())?
        .ok_or_else(||"unknown investigation acquisition obligation".to_owned())?;
    let authorization=authorize(&queue)?;
    let governance=match std::env::var("ITIR_INV_GOVERNANCE_REF") {
        Ok(packet_ref)=>load_inv_governance_packet(&config,&packet_ref)
            .map_err(|e|e.to_string())?,
        Err(_)=>None,
    };
    if let Some(packet)=governance.as_ref() {
        if packet.obligation_ref!=queue.obligation.obligation_ref
            ||packet.comparison_ref!=queue.obligation.comparison_ref
            ||packet.source_revision_refs!=queue.obligation.source_revision_refs {
            return Err("governance packet does not bind to the visible acquisition queue".into());
        }
    }
    let graph=load_bound_graph(&config,&queue,&authorization)?;
    Ok(InvestigationQueueWorkspace{
        queue,governance,graph,matter_ref:authorization.matter_ref,
        creates_semantic_authority:false,
        acquisition_executed:false,
    })
}
