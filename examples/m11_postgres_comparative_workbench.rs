use std::{env, process};

use itir_dioxus::workbench::production_comparative::{
    discover_postgres_comparative_candidates, load_postgres_comparative_workbench,
    load_postgres_pabai_three_way_workbench,
    load_postgres_three_way_comparative_workbench, probe_postgres_pabai_triples,
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
        [flag] if flag == "--discover" => {
            print_discovery(200)?;
        }
        [flag, limit] if flag == "--discover" => {
            let limit = limit
                .parse::<i64>()
                .map_err(|error| format!("invalid discovery limit: {error}"))?;
            print_discovery(limit)?;
        }
        [flag] if flag == "--probe-pabai" => {
            print_pabai_probes(200)?;
        }
        [flag, limit] if flag == "--probe-pabai" => {
            let limit = limit
                .parse::<i64>()
                .map_err(|error| format!("invalid Pabai probe limit: {error}"))?;
            print_pabai_probes(limit)?;
        }
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
                "usage: m11_postgres_comparative_workbench --discover [limit] | --probe-pabai [limit] | [--pabai] <w0-projection-ref> <w1-projection-ref> [w2-projection-ref]"
                    .into(),
            );
        }
    }
    Ok(())
}



fn print_discovery(limit: i64) -> Result<(), String> {
    let receipt = discover_postgres_comparative_candidates(limit)?;
    println!("carrier=typed-rust");
    println!("database_source=postgres");
    println!("pair_candidate_count={}", receipt.pair_candidates.len());
    println!("triple_candidate_count={}", receipt.triple_candidates.len());

    for pair in &receipt.pair_candidates {
        println!(
            "pair_candidate\tdocument_ref={}\tbefore={}\tafter={}\tshared={}\tchanged={}\tleft_only={}\tright_only={}\thas_semantic_delta={}",
            pair.document_ref,
            pair.before_projection_ref,
            pair.after_projection_ref,
            pair.shared_semantic_ref_count,
            pair.changed_semantic_ref_count,
            pair.left_only_semantic_ref_count,
            pair.right_only_semantic_ref_count,
            pair.has_semantic_delta,
        );
    }

    for triple in &receipt.triple_candidates {
        println!(
            "triple_candidate\tdocument_ref={}\tw0={}\tw1={}\tw2={}\tw0_w1_delta={}\tw1_w2_delta={}\tboth_transitions_nontrivial={}",
            triple.document_ref,
            triple.w0_projection_ref,
            triple.w1_projection_ref,
            triple.w2_projection_ref,
            triple.w0_w1_changed_semantic_ref_count,
            triple.w1_w2_changed_semantic_ref_count,
            triple.both_transitions_nontrivial,
        );
    }
    Ok(())
}

fn print_pabai_probes(limit: i64) -> Result<(), String> {
    let probes = probe_postgres_pabai_triples(limit)?;
    println!("carrier=typed-rust");
    println!("database_source=postgres");
    println!("pabai_probe_count={}", probes.len());
    for probe in probes {
        println!(
            "pabai_probe\tdocument_ref={}\tw0={}\tw1={}\tw2={}\texact_typed_overlay_weld={}\tfailure_reason={}",
            probe.document_ref,
            probe.w0_projection_ref,
            probe.w1_projection_ref,
            probe.w2_projection_ref,
            probe.exact_typed_overlay_weld,
            probe.failure_reason.as_deref().unwrap_or("none"),
        );
    }
    Ok(())
}

fn print_typed_annotations(
    prefix: &str,
    model: &itir_dioxus::workbench::comparative::ComparativeWorkbenchReadModel,
) {
    println!(
        "{prefix}_typed_annotation_count={}",
        model.explanation_overlay.typed_change_annotations.len()
    );
    for (semantic_ref, annotation) in &model.explanation_overlay.typed_change_annotations {
        println!("{prefix}_annotation_semantic_ref={semantic_ref}");
        println!("{prefix}_annotation_layer={:?}", annotation.layer);
        println!(
            "{prefix}_annotation_answer_changing={}",
            annotation.answer_changing
        );
        println!(
            "{prefix}_annotation_justification_refs={}",
            annotation.justification_refs.join(",")
        );
        if let Some(explanation_ref) = annotation.explanation_ref.as_ref() {
            println!("{prefix}_annotation_explanation_ref={explanation_ref}");
        }
    }
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
    print_typed_annotations("pair", model);
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
    print_typed_annotations("w0_w1", &sequence.w0_to_w1);
    print_typed_annotations("w1_w2", &sequence.w1_to_w2);
    println!("creates_semantic_authority={}", sequence.creates_semantic_authority);
    println!("creates_claim_truth={}", sequence.creates_claim_truth);
    println!("predicts_outcome={}", sequence.predicts_outcome);
}
