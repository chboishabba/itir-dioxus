#![cfg(feature = "production-data")]
//! M10.2 dual-lens inspection of SCALE-2 persisted source families.
//! One typed SLR read path; GPU and Dioxus consume exactly the same GraphIr
//! and select through the existing DomainCommand decoder. No UI-derived
//! semantic equality, copying, review or claim truth is introduced.

use sensiblaw_pg_source_store::{
    load_database_config, load_mixed_source_comparison, ContextVisibility,
    GenealogyStatus, MixedSourceComparison, SemanticComparison,
};
use crate::visual::{
    command::{decode_gpu, decode_shell, DomainCommand, GpuPickInput, ShellInput, VisualObjectId},
    ir::{stable_visual_id, GraphIr, VisualEdge, VisualNode, VisualObjectInspection},
};

#[derive(Debug, Clone, PartialEq)]
pub struct MixedSourceWorkspace {
    pub comparison: MixedSourceComparison,
    pub graph: GraphIr,
    pub operational_graph: GraphIr,
    pub scope_label: String,
    pub candidate_only: bool,
    pub creates_semantic_authority: bool,
    pub creates_claim_truth: bool,
}

fn node(reference: &str, kind: &str, label: &str, x: f32, y: f32, sources: Vec<String>) -> VisualNode {
    VisualNode {
        id: stable_visual_id(reference), semantic_ref: reference.into(),
        kind: kind.into(), label: label.into(), source_refs: sources,
        provenance_refs: vec![], hidden: false, x, y,
    }
}
fn edge(reference: &str, from: &str, to: &str, kind: &str, sources: Vec<String>) -> VisualEdge {
    VisualEdge {
        id: stable_visual_id(reference), from: stable_visual_id(from),
        to: stable_visual_id(to), semantic_ref: reference.into(), kind: kind.into(),
        source_refs: sources, provenance_refs: vec![], hidden: false,
        weight: 1.0,
    }
}

/// Emit a graph of **possible** PNF matches and **recorded** source joins.
/// Never draw a "quoted in" edge based on textual equality or L2 fingerprints.
pub fn mixed_source_graph(comparison: &MixedSourceComparison) -> GraphIr {
    let left=&comparison.left_source_revision_ref;
    let right=&comparison.right_source_revision_ref;
    let mut nodes=vec![
        node(left,"native_source","Source A", -0.7,0.2,vec![left.clone()]),
        node(right,"native_source","Source B",0.7,0.2,vec![right.clone()]),
    ];
    let mut edges=vec![];
    let semantic = comparison.shared_entity_candidate_refs.iter().map(|r|("entity",r))
        .chain(comparison.shared_proposition_candidate_refs.iter().map(|r|("proposition",r)))
        .chain(comparison.shared_event_candidate_refs.iter().map(|r|("event",r)));
    for (i,(kind,fingerprint)) in semantic.enumerate() {
        // Read-model-only comparison node, not a canonical matched entity.
        let reference=format!("m10:shared-candidate:{kind}:{fingerprint}");
        let y=-0.2-(i as f32*0.11);
        nodes.push(node(&reference,&format!("pnf_candidate_{kind}"),
            &format!("{kind} candidate"),0.0,y,vec![left.clone(),right.clone()]));
        for (side,src) in [("left",left),("right",right)] {
            edges.push(edge(&format!("m10:context:{side}:{reference}"),src,&reference,
                "pnf_candidate_overlap_not_subject_identity",vec![src.clone()]));
        }
    }
    for (i,join_ref) in comparison.native_join_refs.iter().enumerate() {
        let reference=format!("m10:backref:{i}:{join_ref}");
        let y=0.55-(i as f32*0.09);
        nodes.push(node(&reference,"source_backreference","Qualified source join",
            0.0,y,vec![left.clone(),right.clone()]));
        for (side,src) in [("left",left),("right",right)] {
            edges.push(edge(&format!("m10:join:{side}:{reference}"),src,&reference,
                "source_backreference_not_independent_witness",
                vec![src.clone()]));
        }
    }
    GraphIr {
        graph_ref:format!("graph:m10:mixed:{left}:{right}"),
        derived_only:true,challengeable:true,nodes,edges,
    }
}

pub fn operational_context_graph(comparison: &MixedSourceComparison) -> GraphIr {
    let left=&comparison.left_source_revision_ref;
    let right=&comparison.right_source_revision_ref;
    let mut nodes=vec![
        node(left,"native_source","Source A",-0.7,0.5,vec![left.clone()]),
        node(right,"native_source","Source B",0.7,0.5,vec![right.clone()]),
    ];
    let mut edges=vec![];
    // Use the *reviewed* producer/source target that SLR actually returned.
    // These edges denote activity context, not proposition support.
    if matches!(comparison.operational_visibility,ContextVisibility::Available) {
        for (i,link) in comparison.operational_links.iter().enumerate() {
            let reference=format!("m10:observer:{}",link.operational_event_ref);
            if !nodes.iter().any(|n|n.semantic_ref==reference) {
                nodes.push(VisualNode {
                    id:stable_visual_id(&reference),
                    semantic_ref:reference.clone(),
                    kind:"statibaker_observation".into(),
                    label:link.label.clone(),
                    source_refs:vec![link.source_revision_ref.clone()],
                    provenance_refs:link.provenance_refs.clone(),
                    hidden:false,x:0.0,y:0.0-(i as f32*0.13),
                });
            }
            edges.push(VisualEdge {
                id:stable_visual_id(&format!("m10:operational-link:{}",link.link_ref)),
                from:stable_visual_id(&reference),
                to:stable_visual_id(&link.source_revision_ref),
                semantic_ref:link.link_ref.clone(),
                kind:format!("operational_association_not_semantic_evidence:{:?}",link.relation_kind),
                source_refs:vec![link.source_revision_ref.clone()],
                provenance_refs:vec![link.relationship_receipt_ref.clone()],
                hidden:false,weight:1.0,
            });
        }
    }
    GraphIr {
        graph_ref:format!("graph:m10:operational:{left}:{right}"),
        derived_only:true,challengeable:true,nodes,edges,
    }
}

impl MixedSourceWorkspace {
    pub fn from_comparison(comparison: MixedSourceComparison, scope_label: String) -> Self {
        let graph=mixed_source_graph(&comparison);
        let operational_graph=operational_context_graph(&comparison);
        Self {
            comparison,graph,operational_graph,scope_label,
            candidate_only:true,creates_semantic_authority:false,
            creates_claim_truth:false,
        }
    }

    pub fn inspect(&self, object_id: VisualObjectId, operational_lens: bool)
        -> Option<VisualObjectInspection> {
        let graph=if operational_lens {&self.operational_graph} else {&self.graph};
        graph.inspect_object(object_id)
    }
    pub fn shell_select(&self, id: VisualObjectId) -> DomainCommand {
        decode_shell(ShellInput::Select(id))
    }
    pub fn gpu_select(&self, id: VisualObjectId) -> DomainCommand {
        decode_gpu(GpuPickInput::Hit(id))
    }
}

pub fn load_mixed_source_workspace(
    left_revision: &str,
    right_revision: &str,
    chat_message_ref: Option<&str>,
    observer_scope: ContextVisibility,
    scope_label: &str,
) -> Result<MixedSourceWorkspace,String> {
    let config=load_database_config(None).map_err(|e|e.to_string())?;
    let comparison=load_mixed_source_comparison(
        &config,left_revision,right_revision,chat_message_ref,observer_scope
    ).map_err(|e|e.to_string())?;
    Ok(MixedSourceWorkspace::from_comparison(comparison,scope_label.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn comparison() -> MixedSourceComparison {
        MixedSourceComparison {
            left_source_revision_ref:"source:transcript:1".into(),
            right_source_revision_ref:"source:chat:1".into(),
            left_statement_refs:vec![],right_statement_refs:vec![],
            left_source_excerpt:"a".into(),right_source_excerpt:"b".into(),
            shared_entity_candidate_refs:vec!["candidate:paper".into()],
            shared_proposition_candidate_refs:vec![],shared_event_candidate_refs:vec![],
            semantic_comparison:SemanticComparison::SharedCandidateFingerprint,
            genealogy:GenealogyStatus::NoRecordedLineage,
            native_join_refs:vec![],operational_context_refs:vec![],
            operational_links:vec![],
            operational_visibility:ContextVisibility::ExcludedByScope,
            semantic_review_pending:true,independent_witnesses_established:None,
            creates_semantic_authority:false,pays_evidence:false,claim_truth_promoted:false,
        }
    }
    #[test]
    fn shared_pnf_does_not_draw_copying_or_corrob_edge() {
        let model=MixedSourceWorkspace::from_comparison(comparison(),"private".into());
        assert_eq!(model.graph.edges.len(),2);
        assert!(model.graph.edges.iter().all(|e|
            e.kind=="pnf_candidate_overlap_not_subject_identity"));
        assert_eq!(model.operational_graph.nodes.len(),2);
        assert_eq!(model.shell_select(model.graph.nodes[0].id),
                   model.gpu_select(model.graph.nodes[0].id));
        assert!(!model.creates_claim_truth);
    }
}
