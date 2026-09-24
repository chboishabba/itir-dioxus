#![cfg(feature = "production-data")]

use itir_dioxus::workbench::review::execute_review_action;
use sensiblaw_pg_source_store::ReviewAction;

fn parse_action(value: &str) -> Result<ReviewAction, String> {
    match value {
        "accept" => Ok(ReviewAction::Accept),
        "reject" => Ok(ReviewAction::Reject),
        "abstain" => Ok(ReviewAction::Abstain),
        "qualify" => Ok(ReviewAction::Qualify),
        "supersede" => Ok(ReviewAction::Supersede),
        "request-evidence" => Ok(ReviewAction::RequestEvidence),
        "open-source" => Ok(ReviewAction::OpenSource),
        "follow-authority" => Ok(ReviewAction::FollowAuthority),
        other => Err(format!("unknown review action: {other}")),
    }
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let review_item_ref = args.next().ok_or_else(|| {
        "usage: s29_live_review_action_receipt <review_item_ref> <action> <reviewer_ref> [qualification_or_evidence_ref]".to_owned()
    })?;
    let action = parse_action(
        &args.next().ok_or_else(|| "missing action".to_owned())?,
    )?;
    let reviewer_ref = args
        .next()
        .ok_or_else(|| "missing reviewer_ref".to_owned())?;
    let extra = args.next();

    let qualification_ref = if action == ReviewAction::Qualify {
        Some(
            extra
                .clone()
                .ok_or_else(|| "qualify requires qualification ref".to_owned())?,
        )
    } else {
        None
    };
    let evidence_request_ref = if action == ReviewAction::RequestEvidence {
        Some(
            extra
                .ok_or_else(|| "request-evidence requires evidence request ref".to_owned())?,
        )
    } else {
        None
    };

    let (receipt, item) = execute_review_action(
        &review_item_ref,
        action,
        &reviewer_ref,
        qualification_ref,
        evidence_request_ref,
    )?;

    println!("command_ref={}", receipt.command_ref);
    println!("review_item_ref={}", receipt.review_item_ref);
    println!("semantic_ref={}", receipt.semantic_ref);
    println!("action={:?}", receipt.action);
    println!("effect={:?}", receipt.effect);
    println!("resulting_status={:?}", item.current_status);
    println!("candidate_only={}", receipt.candidate_only);
    println!(
        "creates_semantic_authority={}",
        receipt.creates_semantic_authority
    );
    println!("applicability_promoted={}", receipt.applicability_promoted);
    println!("claim_truth_promoted={}", receipt.claim_truth_promoted);

    if !receipt.candidate_only
        || receipt.creates_semantic_authority
        || receipt.applicability_promoted
        || receipt.claim_truth_promoted
    {
        return Err("persisted review command crossed semantic boundary".into());
    }

    Ok(())
}
