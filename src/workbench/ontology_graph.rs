#![cfg(feature="production-data")]
//! WIKI-UI-1 finite-checker/source/repair graph through the same GraphIr and
//! shell↔wgpu selection command vocabulary as the M10 investigation lens.
//! Native source edges are *reported references*, not a new evidence graph.

use sensiblaw_pg_source_store::OntologyDiagnosticRead;
use crate::visual::{
    command::{decode_gpu,decode_shell,DomainCommand,GpuPickInput,ShellInput,VisualObjectId},
    ir::{stable_visual_id,GraphIr,VisualNode,VisualEdge,VisualObjectInspection},
};

fn node(reference:&str,kind:&str,label:&str,x:f32,y:f32,sources:Vec<String>,prov:Vec<String>)->VisualNode {
    VisualNode {
        id:stable_visual_id(reference),semantic_ref:reference.into(),
        kind:kind.into(),label:label.into(),source_refs:sources,
        provenance_refs:prov,hidden:false,x,y,
    }
}
fn edge(reference:&str,from:&str,to:&str,kind:&str,source:&str)->VisualEdge {
    VisualEdge {
        id:stable_visual_id(reference),
        from:stable_visual_id(from),to:stable_visual_id(to),
        semantic_ref:reference.into(),kind:kind.into(),
        source_refs:vec![source.into()],provenance_refs:vec![],
        hidden:false,weight:1.0,
    }
}
/// Graph projection of *one* read-authorized, source-pinned checker packet.
/// An arrow only means "producer reported this witness/candidate".
pub fn ontology_diagnostic_graph(d:&OntologyDiagnosticRead)->GraphIr {
    let p=&d.packet;
    let source=&p.source_revision_ref;
    let checker=format!("wiki1:checker:{}",d.diagnostic_ref);
    let mut nodes=vec![
        node(source,"canonical_source","Pinned Wikidata source",
            -0.65,0.3,vec![source.clone()],vec![p.source_snapshot_digest_ref.clone()]),
        node(&checker,"executed_finite_checker","Finite Lean checker",
            0.0,0.3,vec![source.clone()],
            vec![p.producer_receipt_ref.clone(),p.lean_source_commit.clone()]),
    ];
    let mut edges=vec![edge(&format!("wiki1:checked:{}",d.diagnostic_ref),
        source,&checker,"source_snapshot_checked_in_declared_graph_view",source)];
    for (index,w) in p.witnesses.iter().enumerate() {
        let reference=format!("wiki1:witness:{}:{}",d.diagnostic_ref,w.witness_ref);
        nodes.push(node(&reference,"finite_diagnostic_witness",
            &format!("{} · reported witness",w.rule_ref),
            0.62,0.4-index as f32*0.15,
            w.statement_refs.clone(),w.evidence_refs.clone()));
        edges.push(edge(&format!("wiki1:reported:{reference}"),
            &checker,&reference,"finite_checker_reports_witness_not_world_truth",source));
    }
    for (index,r) in p.repair_candidates.iter().enumerate() {
        let reference=format!("wiki1:advisory-repair:{}:{}",d.diagnostic_ref,r.candidate_ref);
        nodes.push(node(&reference,"advisory_repair",
            "Advisory modeled repair",0.4,-0.4-index as f32*0.15,
            vec![source.clone()],vec![p.producer_receipt_ref.clone()]));
        edges.push(edge(&format!("wiki1:proposal:{reference}"),
            &checker,&reference,"advisory_repair_not_public_edit",source));
    }
    GraphIr {
        graph_ref:format!("graph:wiki1:{}",d.diagnostic_ref),
        derived_only:true,challengeable:true,nodes,edges,
    }
}
pub fn inspect_ontology_graph(graph:&GraphIr,id:VisualObjectId)
    ->Option<VisualObjectInspection> {
    graph.inspect_object(id)
}
pub fn shell_ontology_selection(id:VisualObjectId)->DomainCommand {
    decode_shell(ShellInput::Select(id))
}
pub fn gpu_ontology_selection(id:VisualObjectId)->DomainCommand {
    decode_gpu(GpuPickInput::Hit(id))
}
