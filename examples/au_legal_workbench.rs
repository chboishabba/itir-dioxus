use std::{env, fs, process};

use itir_dioxus::workbench::{
    au_fact_review::replay_legacy_au_workbench_json,
    WorkbenchStageKind,
};

fn main() {
    let path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("usage: au_legal_workbench <persisted-au-workbench.json>");
            process::exit(2);
        }
    };

    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(error) => {
            eprintln!("failed to read {path}: {error}");
            process::exit(2);
        }
    };

    let projection = match replay_legacy_au_workbench_json(&raw, 20, 30) {
        Ok(projection) => projection,
        Err(error) => {
            eprintln!("failed to project AU workbench: {error}");
            process::exit(1);
        }
    };

    println!("world_ref={}", projection.read_model.world_ref);
    println!("source_ref_count={}", projection.persisted_source_refs.len());
    println!("event_ref_count={}", projection.persisted_event_refs.len());
    println!("research_ref_count={}", projection.persisted_research_refs.len());
    println!("graph_ref={}", projection.graph_ir.graph_ref);
    println!("graph_node_count={}", projection.graph_ir.nodes.len());
    println!("graph_edge_count={}", projection.graph_ir.edges.len());
    println!("graph_derived_only={}", projection.graph_ir.derived_only);
    println!("graph_challengeable={}", projection.graph_ir.challengeable);

    for kind in [
        WorkbenchStageKind::Journal,
        WorkbenchStageKind::Timeline,
        WorkbenchStageKind::Handoff,
        WorkbenchStageKind::MatterProof,
        WorkbenchStageKind::Research,
    ] {
        let stage = projection.read_model.stage(kind).expect("stage exists");
        println!(
            "stage={kind:?}|status={}|refs={}",
            stage.availability.label(),
            stage.semantic_refs.len()
        );
        if let Some(reason) = stage.availability.reason() {
            println!("stage_reason={kind:?}|{reason}");
        }
    }

    println!(
        "creates_semantic_authority={}",
        projection.read_model.creates_semantic_authority
    );
    println!(
        "creates_claim_truth={}",
        projection.read_model.creates_claim_truth
    );
    println!("pays_residual={}", projection.read_model.pays_residual);
}
