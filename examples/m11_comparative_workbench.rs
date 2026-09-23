use std::{env, fs, process};

use itir_dioxus::workbench::persisted_comparative::{
    project_persisted_comparative_workbench_json,
    project_persisted_three_way_comparative_json,
};

fn read(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) => {
            eprintln!("failed to read {path}: {error}");
            process::exit(2);
        }
    }
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 && args.len() != 3 && args.len() != 5 {
        eprintln!(
            "usage: m11_comparative_workbench <before.json> <after.json> [counter-after.json [D-overlay.json C-overlay.json]]"
        );
        process::exit(2);
    }

    let w0 = read(&args[0]);
    let w1 = read(&args[1]);

    if args.len() == 2 {
        let specimen = match project_persisted_comparative_workbench_json(
            "comparison:empirical:pair",
            &w0,
            &w1,
            None,
            1000,
            2000,
        ) {
            Ok(specimen) => specimen,
            Err(error) => {
                eprintln!("comparative projection failed: {error}");
                process::exit(1);
            }
        };

        println!("mode=pair");
        println!("left_world_ref={}", specimen.left.read_model.world_ref);
        println!("right_world_ref={}", specimen.right.read_model.world_ref);
        println!(
            "before_nodes={}",
            specimen.read_model.topology.before.nodes.len()
        );
        println!(
            "after_nodes={}",
            specimen.read_model.topology.after.nodes.len()
        );
        println!(
            "delta_nodes={}",
            specimen.read_model.topology.delta.nodes.len()
        );
        println!(
            "delta_edges={}",
            specimen.read_model.topology.delta.edges.len()
        );
        println!(
            "shared_semantic_refs={}",
            specimen
                .read_model
                .topology
                .comparative
                .shared_semantic_refs
                .len()
        );
        println!(
            "changed_semantic_refs={}",
            specimen
                .read_model
                .topology
                .comparative
                .changed_semantic_refs
                .len()
        );
        println!(
            "left_only_semantic_refs={}",
            specimen
                .read_model
                .topology
                .comparative
                .left_only_semantic_refs
                .len()
        );
        println!(
            "right_only_semantic_refs={}",
            specimen
                .read_model
                .topology
                .comparative
                .right_only_semantic_refs
                .len()
        );
        println!("source_backed_left={}", specimen.source_backed_left);
        println!("source_backed_right={}", specimen.source_backed_right);
        println!(
            "provenance_backed_left={}",
            specimen.provenance_backed_left
        );
        println!(
            "provenance_backed_right={}",
            specimen.provenance_backed_right
        );
        println!("candidate_only={}", specimen.candidate_only);
        println!(
            "creates_semantic_authority={}",
            specimen.creates_semantic_authority
        );
        println!("creates_claim_truth={}", specimen.creates_claim_truth);
        println!("predicts_outcome={}", specimen.predicts_outcome);
        return;
    }

    let w2 = read(&args[2]);
    let d_overlay = (args.len() == 5).then(|| read(&args[3]));
    let c_overlay = (args.len() == 5).then(|| read(&args[4]));
    let specimen = match project_persisted_three_way_comparative_json(
        "comparison:empirical:three-way",
        &w0,
        &w1,
        &w2,
        d_overlay.as_deref(),
        c_overlay.as_deref(),
        1000,
        2000,
    ) {
        Ok(specimen) => specimen,
        Err(error) => {
            eprintln!("three-way comparative projection failed: {error}");
            process::exit(1);
        }
    };

    println!("mode=three-way");
    println!("w0_world_ref={}", specimen.w0.read_model.world_ref);
    println!("w1_world_ref={}", specimen.w1.read_model.world_ref);
    println!("w2_world_ref={}", specimen.w2.read_model.world_ref);
    println!(
        "w0_w1_delta_nodes={}",
        specimen.sequence.w0_to_w1.topology.delta.nodes.len()
    );
    println!(
        "w1_w2_delta_nodes={}",
        specimen.sequence.w1_to_w2.topology.delta.nodes.len()
    );
    println!(
        "w0_w1_changed_semantic_refs={}",
        specimen
            .sequence
            .w0_to_w1
            .topology
            .comparative
            .changed_semantic_refs
            .len()
    );
    println!(
        "w1_w2_changed_semantic_refs={}",
        specimen
            .sequence
            .w1_to_w2
            .topology
            .comparative
            .changed_semantic_refs
            .len()
    );
    println!(
        "w0_w1_answer_changing_refs={}",
        specimen
            .sequence
            .w0_to_w1
            .explanation_overlay
            .answer_changing_semantic_refs
            .len()
    );
    println!(
        "w1_w2_answer_changing_refs={}",
        specimen
            .sequence
            .w1_to_w2
            .explanation_overlay
            .answer_changing_semantic_refs
            .len()
    );
    println!(
        "w0_w1_typed_explanations={}",
        specimen
            .sequence
            .w0_to_w1
            .explanation_overlay
            .explanation_by_semantic_ref
            .len()
    );
    println!(
        "w1_w2_typed_explanations={}",
        specimen
            .sequence
            .w1_to_w2
            .explanation_overlay
            .explanation_by_semantic_ref
            .len()
    );
    println!("candidate_only={}", specimen.candidate_only);
    println!(
        "creates_semantic_authority={}",
        specimen.creates_semantic_authority
    );
    println!("creates_claim_truth={}", specimen.creates_claim_truth);
    println!("predicts_outcome={}", specimen.predicts_outcome);
}
