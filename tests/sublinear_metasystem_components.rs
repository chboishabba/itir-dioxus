use itir_dioxus::components::{
    metric_value, ordered_deliverables, valid_edges, DeliverableReceipt, ProjectEdge, ProjectNode,
    SankeyFlow, SankeyMetric,
};
use itir_dioxus::metasystem_ui::{load_projection, MetasystemProjection};

#[test]
fn topic_timeline_is_stably_sequence_ordered() {
    let rows = vec![
        DeliverableReceipt {
            label: "proof".into(),
            status: "verified".into(),
            pr_ref: Some("#2".into()),
            proof_hash: Some("bbb".into()),
            sequence: 2,
        },
        DeliverableReceipt {
            label: "implementation".into(),
            status: "merged".into(),
            pr_ref: Some("#1".into()),
            proof_hash: Some("aaa".into()),
            sequence: 1,
        },
    ];
    let ordered = ordered_deliverables(&rows);
    assert_eq!(ordered[0].label, "implementation");
    assert_eq!(ordered[1].label, "proof");
}

#[test]
fn sankey_switches_between_supplied_metrics_without_reclassification() {
    let flow = SankeyFlow {
        chat_ref: "chat:c1".into(),
        cluster_ref: "pnf:k1".into(),
        lane_ref: "lane:w1".into(),
        continuous_alpha: 0.8,
        perplexity: 1.7,
        frontier_blockers: 4.0,
        task_priority: 0.6,
    };
    assert_eq!(metric_value(&flow, SankeyMetric::ContinuousAlpha), 0.8);
    assert_eq!(metric_value(&flow, SankeyMetric::Perplexity), 1.7);
    assert_eq!(metric_value(&flow, SankeyMetric::FrontierBlockers), 4.0);
    assert_eq!(metric_value(&flow, SankeyMetric::TaskPriority), 0.6);
}

#[test]
fn cross_project_graph_drops_edges_without_supplied_endpoints() {
    let nodes = vec![
        ProjectNode {
            node_ref: "itir:topic".into(),
            project_ref: "ITIR-suite".into(),
            label: "Topic".into(),
            x: 10.0,
            y: 20.0,
            receipt_ref: None,
        },
        ProjectNode {
            node_ref: "slr:proof".into(),
            project_ref: "SensibLaw".into(),
            label: "Proof".into(),
            x: 50.0,
            y: 80.0,
            receipt_ref: Some("sha256:abc".into()),
        },
    ];
    let edges = vec![
        ProjectEdge {
            edge_ref: "e1".into(),
            source_ref: "itir:topic".into(),
            target_ref: "slr:proof".into(),
            relation_ref: "consumes".into(),
        },
        ProjectEdge {
            edge_ref: "e2".into(),
            source_ref: "itir:topic".into(),
            target_ref: "missing".into(),
            relation_ref: "guessed".into(),
        },
    ];
    let valid = valid_edges(&nodes, &edges);
    assert_eq!(valid.len(), 1);
    assert_eq!(valid[0].edge_ref, "e1");
}

#[test]
fn projection_loader_preserves_explicit_empty_model(tmp_path: &std::path::Path) {
    let path = tmp_path.join("projection.json");
    std::fs::write(
        &path,
        r#"{"topics":[],"flows":[],"project_nodes":[],"project_edges":[]}"#,
    )
    .unwrap();
    let model = load_projection(&path).unwrap();
    assert_eq!(model, MetasystemProjection::default());
}
