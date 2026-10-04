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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewAuthorityProjectionInput {
    pub case_ref: String,
    pub matter_ref: String,
    pub reviewed_evidence_ref: String,
    pub review_receipt_ref: String,
    pub consumer_ref: String,
    pub requirement_ref: String,
    pub evidence_role_ref: String,
    pub normative_order_ref: String,
    pub proposition_ref: String,
    pub source_revision_ref: String,
    pub exact_span_ref: String,
    pub candidate_only: bool,
    pub creates_semantic_authority: bool,
    pub applicability_promoted: bool,
    pub claim_truth_promoted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewedEvidenceAuthorityProjection {
    pub case_ref: String,
    pub reviewed_evidence_ref: String,
    pub review_receipt_ref: String,
    pub consumer_ref: String,
    pub requirement_ref: String,
    pub evidence_role_ref: String,
    pub normative_order_ref: String,
    pub proposition_ref: String,
    pub source_revision_ref: String,
    pub exact_span_ref: String,
    pub candidate_only: bool,
    pub creates_semantic_authority: bool,
    pub applicability_promoted: bool,
    pub claim_truth_promoted: bool,
    pub persistence_creates_legal_authority: bool,
}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

pub fn project_review_authority_context(
    expected_matter_ref: &str,
    input: ReviewAuthorityProjectionInput,
) -> Result<ReviewedEvidenceAuthorityProjection, String> {
    if !nonempty(expected_matter_ref)
        || !nonempty(&input.case_ref)
        || !nonempty(&input.matter_ref)
        || !nonempty(&input.reviewed_evidence_ref)
        || !nonempty(&input.review_receipt_ref)
        || !nonempty(&input.consumer_ref)
        || !nonempty(&input.requirement_ref)
        || !nonempty(&input.evidence_role_ref)
        || !nonempty(&input.normative_order_ref)
        || !nonempty(&input.proposition_ref)
        || !nonempty(&input.source_revision_ref)
        || !nonempty(&input.exact_span_ref)
    {
        return Err("reviewed evidence projection requires complete persisted authority coordinates".into());
    }
    if input.matter_ref != expected_matter_ref {
        return Err("reviewed evidence case does not belong to the visible MatterContext".into());
    }
    if !input.candidate_only
        || input.creates_semantic_authority
        || input.applicability_promoted
        || input.claim_truth_promoted
    {
        return Err("reviewed evidence crossed its non-promotion boundary".into());
    }

    Ok(ReviewedEvidenceAuthorityProjection {
        case_ref: input.case_ref,
        reviewed_evidence_ref: input.reviewed_evidence_ref,
        review_receipt_ref: input.review_receipt_ref,
        consumer_ref: input.consumer_ref,
        requirement_ref: input.requirement_ref,
        evidence_role_ref: input.evidence_role_ref,
        normative_order_ref: input.normative_order_ref,
        proposition_ref: input.proposition_ref,
        source_revision_ref: input.source_revision_ref,
        exact_span_ref: input.exact_span_ref,
        candidate_only: true,
        creates_semantic_authority: false,
        applicability_promoted: false,
        claim_truth_promoted: false,
        persistence_creates_legal_authority: false,
    })
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
