#![cfg(feature = "production-data")]

use sensiblaw_core::{
    matter_context::DisclosureBoundary,
    matter_handoff::{
        preview_minimal_handoff, HandoffRecipientProfile, MinimalHandoffSelection,
    },
};
use itir_dioxus::workbench::{
    matter::load_generic_matter_workspace,
    matter_scope::load_matter_scope_manifest,
};

fn profile(value: &str) -> Result<HandoffRecipientProfile, String> {
    match value {
        "lawyer" => Ok(HandoffRecipientProfile::Lawyer),
        "clinician" => Ok(HandoffRecipientProfile::Clinician),
        "advocate" => Ok(HandoffRecipientProfile::Advocate),
        "regulator" => Ok(HandoffRecipientProfile::Regulator),
        "public-official" => Ok(HandoffRecipientProfile::PublicOfficial),
        "researcher" => Ok(HandoffRecipientProfile::Researcher),
        "other" => Ok(HandoffRecipientProfile::Other),
        other => Err(format!("unknown recipient profile: {other}")),
    }
}

fn csv(value: Option<String>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let scope_path = args.next().ok_or_else(|| {
        "usage: m13_handoff_preview_receipt <matter-scope.json> <recipient_ref> <profile> <selected_csv> [redacted_csv]".to_owned()
    })?;
    let recipient_ref = args.next().ok_or_else(|| "missing recipient_ref".to_owned())?;
    let recipient_profile = profile(
        &args.next().ok_or_else(|| "missing recipient profile".to_owned())?,
    )?;
    let selected_refs = csv(args.next());
    let redaction_refs = csv(args.next());

    let request = load_matter_scope_manifest(&scope_path)?;
    let model = load_generic_matter_workspace(request)?;

    let selection = MinimalHandoffSelection {
        handoff_ref: format!("handoff:{}:receipt", model.projection.matter_ref),
        matter_ref: model.projection.matter_ref.clone(),
        recipient_ref,
        recipient_profile,
        disclosure_boundary: DisclosureBoundary::RecipientScoped,
        selected_refs,
        redaction_refs,
        retention_policy_ref: "retention:bounded".into(),
        redaction_policy_ref: "redaction:explicit".into(),
        text_export_policy_ref: "text-export:reviewed-visible-only".into(),
        local_only: true,
        do_not_sync: true,
    };
    let preview = preview_minimal_handoff(
        &selection,
        &model.projection.context_projection,
    )
    .map_err(|error| format!("{error:?}"))?;

    println!("handoff_ref={}", preview.handoff_ref);
    println!("matter_ref={}", preview.matter_ref);
    println!("recipient_ref={}", preview.recipient_ref);
    println!("recipient_profile={:?}", preview.recipient_profile);
    println!("exported_ref_count={}", preview.exported_refs.len());
    println!("redacted_ref_count={}", preview.redacted_refs.len());
    println!("visible_exclusion_count={}", preview.visible_exclusions.len());
    println!("canonical_world_mutated={}", preview.canonical_world_mutated);
    println!(
        "redaction_deletes_canonical_source={}",
        preview.redaction_deletes_canonical_source
    );
    println!(
        "export_creates_semantic_authority={}",
        preview.export_creates_semantic_authority
    );
    println!("claim_truth_promoted={}", preview.claim_truth_promoted);

    if preview.canonical_world_mutated
        || preview.redaction_deletes_canonical_source
        || preview.export_creates_semantic_authority
        || preview.claim_truth_promoted
    {
        return Err("handoff preview crossed Matter boundary".into());
    }

    for reference in &preview.exported_refs {
        println!("export={reference}");
    }
    for reference in &preview.redacted_refs {
        println!("redacted={reference}");
    }
    for exclusion in &preview.visible_exclusions {
        println!("excluded={} reason={:?}", exclusion.semantic_ref, exclusion.reason);
    }

    Ok(())
}
