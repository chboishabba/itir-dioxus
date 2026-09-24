#![cfg(feature = "production-data")]

use itir_dioxus::workbench::chat_source::load_conversation_source_workspace;

fn main() -> Result<(), String> {
    let conversation_ref = std::env::args()
        .nth(1)
        .ok_or_else(|| "usage: s28_chat_source_receipt <conversation_ref>".to_owned())?;

    let model = load_conversation_source_workspace(&conversation_ref)?;

    println!("conversation_ref={}", model.conversation_ref);
    println!("message_count={}", model.messages.len());
    println!("statement_count={}", model.statement_count);
    println!(
        "inactive_branch_message_count={}",
        model.inactive_branch_message_count
    );
    println!(
        "creates_semantic_authority={}",
        model.creates_semantic_authority
    );
    println!("claim_truth_promoted={}", model.claim_truth_promoted);

    if model.creates_semantic_authority || model.claim_truth_promoted {
        return Err("conversation source projection crossed semantic boundary".into());
    }

    for message in &model.messages {
        println!(
            "message={} node={} parent={} branch={:?} role={:?} kind={:?} time={} statements={}",
            message.source.message_ref,
            message.source.node_ref,
            message.source.parent_node_ref.as_deref().unwrap_or(""),
            message.source.branch_membership,
            message.source.role,
            message.source.content_kind,
            message.source.message_time_ref,
            message.statements.len(),
        );
    }

    Ok(())
}
