#![cfg(feature = "production-data")]

use std::{fs, path::PathBuf};

use itir_dioxus::workbench::matter_scope::load_matter_scope_manifest;

fn temp_manifest(name: &str, body: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("itir-dioxus-{name}-{}.json", std::process::id()));
    fs::write(&path, body).expect("write test matter scope");
    path
}

#[test]
fn matter_scope_owns_controversy_coordinates_without_a_second_route_selector() {
    let path = temp_manifest(
        "unified-matter-controversy",
        r#"{
          "schema": "sensiblaw.matter-scope.v0_1",
          "matter_ref": "matter:smith",
          "event_refs": [],
          "operational_dates": [],
          "controversy_refs": ["controversy:smith:s22"],
          "context": {
            "purpose": "legal-advocacy",
            "role": "lawyer",
            "disclosure_boundary": "role-scoped",
            "sealed_refs": []
          }
        }"#,
    );

    let request = load_matter_scope_manifest(&path).expect("valid scoped Matter");
    let _ = fs::remove_file(path);

    assert_eq!(request.matter_ref, "matter:smith");
    assert_eq!(request.controversy_refs, vec!["controversy:smith:s22"]);
    assert_eq!(request.context.matter_ref, "matter:smith");
}

#[test]
fn old_scope_manifests_remain_valid_without_controversy_coordinates() {
    let path = temp_manifest(
        "backward-compatible-matter",
        r#"{
          "schema": "sensiblaw.matter-scope.v0_1",
          "matter_ref": "matter:legacy",
          "context": {
            "purpose": "legal-research",
            "role": "lawyer",
            "disclosure_boundary": "matter-internal",
            "sealed_refs": []
          }
        }"#,
    );

    let request = load_matter_scope_manifest(&path).expect("old scope remains valid");
    let _ = fs::remove_file(path);

    assert!(request.controversy_refs.is_empty());
}
