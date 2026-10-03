#![cfg(feature = "production-data")]

use std::collections::BTreeSet;

use itir_dioxus::workbench::investigation::{
    investigation_graph_ir, load_investigation_queue,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let obligation_ref = std::env::var("ITIR_INV_ACQUISITION_REF")?;
    let workspace = load_investigation_queue(&obligation_ref)
        .map_err(|error| format!("load investigation queue: {error}"))?;
    let persisted = workspace
        .graph
        .as_ref()
        .ok_or("no persisted investigation graph is bound to this obligation")?;
    let graph = investigation_graph_ir(persisted)
        .map_err(|error| format!("project investigation GraphIr: {error}"))?;

    if graph.nodes.len() != persisted.nodes.len() || graph.edges.len() != persisted.edges.len() {
        return Err("GraphIr node/edge cardinality differs from persisted graph".into());
    }
    if !graph.derived_only || !graph.challengeable {
        return Err("GraphIr crossed the derived/challengeable boundary".into());
    }
    if persisted.creates_semantic_authority || persisted.creates_graph_edges {
        return Err("typed investigation graph crossed the projection firewall".into());
    }

    let persisted_node_refs = persisted
        .nodes
        .iter()
        .map(|node| node.semantic_ref.as_str())
        .collect::<BTreeSet<_>>();
    let visual_node_refs = graph
        .nodes
        .iter()
        .map(|node| node.semantic_ref.as_str())
        .collect::<BTreeSet<_>>();
    if persisted_node_refs != visual_node_refs {
        return Err("GraphIr changed persisted node identity".into());
    }
    let persisted_edge_refs = persisted
        .edges
        .iter()
        .map(|edge| edge.semantic_ref.as_str())
        .collect::<BTreeSet<_>>();
    let visual_edge_refs = graph
        .edges
        .iter()
        .map(|edge| edge.semantic_ref.as_str())
        .collect::<BTreeSet<_>>();
    if persisted_edge_refs != visual_edge_refs {
        return Err("GraphIr changed persisted edge identity".into());
    }

    println!("matter_ref={}", workspace.matter_ref);
    println!("obligation_ref={}", workspace.queue.obligation.obligation_ref);
    println!("binding_ref={}", persisted.binding.binding_ref);
    println!("projection_ref={}", persisted.projection_ref);
    println!("graph_nodes={}", graph.nodes.len());
    println!("graph_edges={}", graph.edges.len());
    println!("graph_created_by_ui=false");
    println!("graph_edges_created_by_ui=false");
    println!("semantic_authority_created=false");
    println!("acquisition_executed=false");
    Ok(())
}
