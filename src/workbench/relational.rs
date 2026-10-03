#![cfg(feature="production-data")]
//! ITIR source-grounded relational comparison: one SLR typed read model;
//! no Dioxus-owned ontology, equivalence heuristic or review mutation.
use std::collections::BTreeSet;
use sensiblaw_pg_source_store::{
    load_database_config,load_relational_comparison,
    DurableRelationalComparison,
};
use crate::visual::{
    command::{decode_shell,decode_gpu,DomainCommand,ShellInput,GpuPickInput,VisualObjectId},
    ir::{GraphIr,VisualNode,VisualEdge,stable_visual_id,VisualObjectInspection},
};

#[derive(Debug,Clone,PartialEq)]
pub struct RelationalWorkspace {
    pub packet:DurableRelationalComparison,
    pub graph:GraphIr,
    pub candidate_only:bool,
    pub creates_semantic_authority:bool,
    pub claim_truth_promoted:bool,
}
fn scoped(pair:&DurableRelationalComparison)->Result<(),String>{
    let manifest=std::env::var("SENSIBLAW_MATTER_SCOPE")
        .map_err(|_|"Matter scope is required to inspect source comparison".to_owned())?;
    let request=super::matter_scope::load_matter_scope_manifest(&manifest)?;
    if request.matter_ref!=pair.consumer.consumer_ref {
        return Err("consumer fibre is outside this Matter scope".into());
    }
    let projection=sensiblaw_core::matter_context::project_matter_context(
        &request.context,&request.context_coordinates,
    ).map_err(|e|format!("{e:?}"))?;
    if !projection.included_refs.iter().any(|s|
        s==&pair.left.source_revision_ref)
        || !projection.included_refs.iter().any(|s|
            s==&pair.right.source_revision_ref)
        || projection.canonical_world_mutated || projection.invisibility_means_false {
        return Err("one or both source revisions are excluded by MatterContext".into());
    }
    Ok(())
}
fn node(reference:&str,kind:&str,label:&str,x:f32,y:f32,
        refs:Vec<String>)->VisualNode {
    VisualNode{
        id:stable_visual_id(reference),semantic_ref:reference.into(),
        kind:kind.into(),label:label.into(),source_refs:refs,
        provenance_refs:vec![],hidden:false,x,y,
    }
}
fn edge(reference:&str,src:&str,dst:&str,kind:&str,ref_source:&str)->VisualEdge{
    VisualEdge{
        id:stable_visual_id(reference),semantic_ref:reference.into(),
        from:stable_visual_id(src),to:stable_visual_id(dst),
        kind:kind.into(),source_refs:vec![ref_source.into()],
        provenance_refs:vec![],hidden:false,weight:1.0,
    }
}
/// Candidate and residual-only graph. Licensing receipts are inspectable
/// and never represented as equivalence of canonical source identities.
pub fn graph_for_relational_pair(pair:&DurableRelationalComparison)->GraphIr{
    let cmp=&pair.comparison;
    let l=&pair.left.source_revision_ref;
    let r=&pair.right.source_revision_ref;
    let root=&cmp.comparison_ref;
    let mut nodes=vec![
        node(l,"native_source","Left original source",-0.8,0.4,vec![l.clone()]),
        node(r,"native_source","Right original source",0.8,0.4,vec![r.clone()]),
        node(root,"candidate_comparison","Consumer-indexed comparison",0.0,0.25,
            vec![l.clone(),r.clone()]),
    ];
    let mut edges=vec![
        edge(&format!("{root}:left"),l,root,
            "observed_structure_not_canonical_identity",l),
        edge(&format!("{root}:right"),r,root,
            "observed_structure_not_canonical_identity",r),
    ];
    for (idx,residual) in cmp.residuals.iter().enumerate(){
        let reference=format!("{root}:residual:{idx}");
        nodes.push(node(&reference,"unpaid_typed_obligation",
            &format!("{:?}",residual.kind),0.0,-0.1-(idx as f32*0.12),
            vec![l.clone(),r.clone()]));
        edges.push(edge(&format!("{reference}:from-comparison"),root,&reference,
            "residual_remains_recoverable",l));
    }
    GraphIr{
        graph_ref:format!("graph:itir:relational:{root}"),
        derived_only:true,challengeable:true,nodes,edges,
    }
}
pub fn open_relational_pair(ref_id:&str)->Result<RelationalWorkspace,String>{
    let cfg=load_database_config(None).map_err(|e|e.to_string())?;
    let packet=load_relational_comparison(&cfg,ref_id)
        .map_err(|e|e.to_string())?
        .ok_or_else(||"unknown relational comparison ref".to_owned())?;
    scoped(&packet)?;
    let graph=graph_for_relational_pair(&packet);
    Ok(RelationalWorkspace{
        packet,graph,candidate_only:true,
        creates_semantic_authority:false,claim_truth_promoted:false,
    })
}
pub fn inspect(w:&RelationalWorkspace,id:VisualObjectId)
    ->Option<VisualObjectInspection>{w.graph.inspect_object(id)}
pub fn select_shell(id:VisualObjectId)->DomainCommand{
    decode_shell(ShellInput::Select(id))
}
pub fn select_gpu(id:VisualObjectId)->DomainCommand{
    decode_gpu(GpuPickInput::Hit(id))
}
