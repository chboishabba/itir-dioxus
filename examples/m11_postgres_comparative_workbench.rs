use std::{env, process};

use itir_dioxus::workbench::production_comparative::{
    load_postgres_comparative_workbench,
    load_postgres_pabai_three_way_workbench,
    load_postgres_three_way_comparative_workbench,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("postgres comparative workbench failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [left, right] => {
            let model = load_postgres_comparative_workbench(
                "comparison:postgres:pair",
                left,
                right,
                None,
            )?;
            print_pair(&model);
        }
        [w0, w1, w2] => {
            let sequence = load_postgres_three_way_comparative_workbench(
                "comparison:postgres:three-way",
                w0,
                w1,
                w2,
                None,
                None,
            )?;
            print_three_way(&sequence);
        }
        [flag, w0, w1, w2] if flag == "--pabai" => {
            let sequence =
                load_postgres_pabai_three_way_workbench(w0, w1, w2)?;
            print_three_way(&sequence);
        }
        _ => {
            return Err(
                "usage: m11_postgres_comparative_workbench [--pabai] <w0-projection-ref> <w1-projection-ref> [w2-projection-ref]"
                    .into(),
            );
        }
    }
    Ok(())
}

fn print_pair(model: &itir_dioxus::workbench::comparative::ComparativeWorkbenchReadModel) {
    println!("carrier=typed-rust");
    println!("left_world_ref={}", model.selectors.left_ref);
    println!("right_world_ref={}", model.selectors.right_ref);
    println!("before_graph_ref={}", model.topology.before.graph_ref);
    println!("after_graph_ref={}", model.topology.after.graph_ref);
    println!("delta_graph_ref={}", model.topology.delta.graph_ref);
    println!(
        "changed_semantic_refs={}",
        model.topology.comparative.changed_semantic_refs.len()
    );
    println!(
        "answer_changing_semantic_refs={}",
        model
            .explanation_overlay
            .answer_changing_semantic_refs
            .len()
    );
    println!("creates_semantic_authority={}", model.creates_semantic_authority);
    println!("creates_claim_truth={}", model.creates_claim_truth);
    println!("predicts_outcome={}", model.predicts_outcome);
}

fn print_three_way(
    sequence: &itir_dioxus::workbench::comparative::ThreeWayComparativeSequence,
) {
    println!("carrier=typed-rust");
    println!(
        "w0_w1_changed_semantic_refs={}",
        sequence
            .w0_to_w1
            .topology
            .comparative
            .changed_semantic_refs
            .len()
    );
    println!(
        "w1_w2_changed_semantic_refs={}",
        sequence
            .w1_to_w2
            .topology
            .comparative
            .changed_semantic_refs
            .len()
    );
    println!(
        "w0_w1_answer_changing_semantic_refs={}",
        sequence
            .w0_to_w1
            .explanation_overlay
            .answer_changing_semantic_refs
            .len()
    );
    println!(
        "w1_w2_answer_changing_semantic_refs={}",
        sequence
            .w1_to_w2
            .explanation_overlay
            .answer_changing_semantic_refs
            .len()
    );
    println!("creates_semantic_authority={}", sequence.creates_semantic_authority);
    println!("creates_claim_truth={}", sequence.creates_claim_truth);
    println!("predicts_outcome={}", sequence.predicts_outcome);
}
