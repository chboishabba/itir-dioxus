#![cfg(feature = "production-data")]

use sensiblaw_core::{
    matter_context::DisclosureBoundary,
    matter_handoff::{
        preview_minimal_handoff, HandoffRecipientProfile, MinimalHandoffSelection,
    },
};
use sensiblaw_pg_source_store::ReviewAction;
use itir_dioxus::workbench::{
    chat_source::load_conversation_source_workspace,
    gwb_matter::load_gwb_matter_workspace,
    matter::load_generic_matter_workspace,
    matter_scope::load_matter_scope_manifest,
    review::execute_review_action,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FixtureKind {
    Gwb,
    Operator,
}

fn fixture_kind(value: &str) -> Result<FixtureKind, String> {
    match value {
        "gwb" => Ok(FixtureKind::Gwb),
        "operator" => Ok(FixtureKind::Operator),
        other => Err(format!("unknown fixture kind: {other}; expected gwb or operator")),
    }
}

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

fn require_nonzero(name: &str, value: usize) -> Result<(), String> {
    if value == 0 {
        Err(format!("M13 empirical capstone missing required population: {name}=0"))
    } else {
        Ok(())
    }
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let kind = fixture_kind(&args.next().ok_or_else(|| {
        "usage: m13_cohesive_matter_acceptance_receipt <gwb|operator> <matter-scope.json> <review_item_ref|-> <reviewer_ref|-> <recipient_ref> <profile> <selected_csv> [redacted_csv] [fixture_arg]".to_owned()
    })?)?;
    let scope_path = args.next().ok_or_else(|| "missing matter scope path".to_owned())?;
    let review_item_ref = args.next().ok_or_else(|| "missing review item ref or -".to_owned())?;
    let reviewer_ref = args.next().ok_or_else(|| "missing reviewer ref or -".to_owned())?;
    let recipient_ref = args.next().ok_or_else(|| "missing recipient_ref".to_owned())?;
    let recipient_profile = profile(
        &args.next().ok_or_else(|| "missing recipient profile".to_owned())?,
    )?;
    let selected_refs = csv(args.next());
    let redaction_refs = csv(args.next());
    let fixture_arg = args.next();

    require_nonzero("handoff_selected_refs", selected_refs.len())?;

    let request = load_matter_scope_manifest(&scope_path)?;
    let before = load_generic_matter_workspace(request.clone())?;
    let projection = &before.projection;
    let acceptance = &before.acceptance;

    let claim_count = projection
        .event_timeline
        .proposition_views
        .iter()
        .map(|view| view.leaves.len())
        .sum::<usize>();

    require_nonzero("source_traces", projection.source_traces.len())?;
    require_nonzero("event_timeline", projection.event_timeline.entries.len())?;
    require_nonzero("knowledge_timeline", projection.knowledge_timeline.len())?;
    require_nonzero("claims", claim_count)?;
    require_nonzero("auto_proposals", projection.join_proposals.proposals.len())?;
    require_nonzero("review_items", projection.review_queue.items.len())?;
    require_nonzero(
        "source_reopenable_refs",
        acceptance.source_reopenable_ref_count,
    )?;
    require_nonzero("event_entries", acceptance.event_entry_count)?;

    if projection.canonical_world_mutated
        || projection.creates_semantic_authority
        || projection.claim_truth_promoted
        || acceptance.canonical_world_mutated
        || acceptance.creates_semantic_authority
        || acceptance.claim_truth_promoted
    {
        return Err("M13 empirical capstone crossed semantic boundary".into());
    }

    let selection = MinimalHandoffSelection {
        handoff_ref: format!("handoff:{}:m13-capstone", projection.matter_ref),
        matter_ref: projection.matter_ref.clone(),
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
        &projection.context_projection,
    )
    .map_err(|error| format!("{error:?}"))?;

    require_nonzero("handoff_exported_refs", preview.exported_refs.len())?;
    if preview.canonical_world_mutated
        || preview.redaction_deletes_canonical_source
        || preview.export_creates_semantic_authority
        || preview.claim_truth_promoted
    {
        return Err("M13 handoff preview crossed semantic boundary".into());
    }

    let mut review_mutation_persisted = false;
    let mut review_reload_matches = false;

    match kind {
        FixtureKind::Gwb => {
            let manifest_path = fixture_arg.ok_or_else(|| {
                "gwb fixture requires final argument <gwb-capstone-manifest.json>".to_owned()
            })?;
            let gwb = load_gwb_matter_workspace(&manifest_path)?;
            require_nonzero("gwb_source_statements", gwb.source_statement_count)?;
            if gwb.source_family_refs.len() < 2 {
                return Err(format!(
                    "GWB fixture is not heterogeneous: source_family_count={}",
                    gwb.source_family_refs.len()
                ));
            }
            require_nonzero("gwb_events", gwb.event_refs.len())?;
            println!("gwb_source_family_count={}", gwb.source_family_refs.len());
            println!("gwb_source_statement_count={}", gwb.source_statement_count);
        }
        FixtureKind::Operator => {
            require_nonzero(
                "operational_timeline",
                projection.operational_timeline.entries.len(),
            )?;
            require_nonzero(
                "context_exclusions",
                projection.context_projection.exclusions.len(),
            )?;
            require_nonzero("redacted_refs", preview.redacted_refs.len())?;

            if acceptance.missing_date_event_refs.is_empty()
                && acceptance.missing_actor_claim_refs.is_empty()
            {
                return Err(
                    "operator fixture must exercise missing-date or missing-actor acceptance debt"
                        .into(),
                );
            }
            require_nonzero("contradictions", acceptance.contradictory_relation_refs.len())?;
            require_nonzero("no_event_material", acceptance.no_event_refs.len())?;
            require_nonzero(
                "operational_carryover",
                acceptance.operational_carryover_refs.len(),
            )?;

            if review_item_ref == "-" || reviewer_ref == "-" {
                return Err(
                    "operator fixture requires review_item_ref and reviewer_ref for persisted mutation"
                        .into(),
                );
            }

            let before_item = projection
                .review_queue
                .items
                .iter()
                .find(|item| item.review_item_ref == review_item_ref)
                .ok_or_else(|| {
                    format!("review item not present in Matter projection: {review_item_ref}")
                })?;
            if !before_item.available_actions.contains(&ReviewAction::Accept) {
                return Err(format!(
                    "review item does not offer Accept: {review_item_ref}"
                ));
            }

            let (review_receipt, mutated_item) = execute_review_action(
                &review_item_ref,
                ReviewAction::Accept,
                &reviewer_ref,
                None,
                None,
            )?;
            if !review_receipt.candidate_only
                || review_receipt.creates_semantic_authority
                || review_receipt.applicability_promoted
                || review_receipt.claim_truth_promoted
            {
                return Err("persisted review action crossed semantic boundary".into());
            }
            review_mutation_persisted = true;

            let reloaded = load_generic_matter_workspace(request)?;
            let reloaded_item = reloaded
                .projection
                .review_queue
                .items
                .iter()
                .find(|item| item.review_item_ref == review_item_ref)
                .ok_or_else(|| {
                    format!("review item disappeared after reload: {review_item_ref}")
                })?;
            if reloaded_item.current_status != mutated_item.current_status {
                return Err(format!(
                    "review reload mismatch for {review_item_ref}: persisted={:?} reloaded={:?}",
                    mutated_item.current_status, reloaded_item.current_status
                ));
            }
            review_reload_matches = true;

            let conversation_ref = fixture_arg.ok_or_else(|| {
                "operator fixture requires final argument <conversation_ref>".to_owned()
            })?;
            let chat = load_conversation_source_workspace(&conversation_ref)?;
            require_nonzero("chat_messages", chat.messages.len())?;
            require_nonzero("chat_statements", chat.statement_count)?;
            if chat.creates_semantic_authority || chat.claim_truth_promoted {
                return Err("CHAT projection crossed semantic boundary".into());
            }
            println!("chat_message_count={}", chat.messages.len());
            println!("chat_statement_count={}", chat.statement_count);
        }
    }

    println!("fixture_kind={kind:?}");
    println!("matter_ref={}", projection.matter_ref);
    println!("source_trace_count={}", projection.source_traces.len());
    println!("event_timeline_count={}", projection.event_timeline.entries.len());
    println!("knowledge_timeline_count={}", projection.knowledge_timeline.len());
    println!(
        "operational_timeline_count={}",
        projection.operational_timeline.entries.len()
    );
    println!("claim_count={claim_count}");
    println!("auto_proposal_count={}", projection.join_proposals.proposals.len());
    println!("review_item_count={}", projection.review_queue.items.len());
    println!(
        "context_exclusion_count={}",
        projection.context_projection.exclusions.len()
    );
    println!("handoff_exported_ref_count={}", preview.exported_refs.len());
    println!("handoff_redacted_ref_count={}", preview.redacted_refs.len());
    println!("review_mutation_persisted={review_mutation_persisted}");
    println!("review_reload_matches={review_reload_matches}");
    println!("canonical_world_mutated={}", projection.canonical_world_mutated);
    println!(
        "creates_semantic_authority={}",
        projection.creates_semantic_authority
    );
    println!("claim_truth_promoted={}", projection.claim_truth_promoted);

    Ok(())
}
