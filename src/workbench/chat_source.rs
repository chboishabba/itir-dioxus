#![cfg(feature = "production-data")]

use sensiblaw_pg_source_store::{
    load_chat_messages_for_conversation, load_database_config,
    load_source_statements_for_document, PersistedChatMessageSource,
    PersistedSourceStatement,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationMessageView {
    pub source: PersistedChatMessageSource,
    pub statements: Vec<PersistedSourceStatement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationSourceWorkspace {
    pub conversation_ref: String,
    pub messages: Vec<ConversationMessageView>,
    pub statement_count: usize,
    pub inactive_branch_message_count: usize,
    pub creates_semantic_authority: bool,
    pub claim_truth_promoted: bool,
}

pub fn load_conversation_source_workspace(
    conversation_ref: &str,
) -> Result<ConversationSourceWorkspace, String> {
    if conversation_ref.trim().is_empty() {
        return Err("conversation ref is required".into());
    }
    let config = load_database_config(None).map_err(|error| error.to_string())?;
    let sources =
        load_chat_messages_for_conversation(&config, conversation_ref)
            .map_err(|error| error.to_string())?;

    let mut messages = Vec::with_capacity(sources.len());
    let mut statement_count = 0usize;
    let mut inactive_branch_message_count = 0usize;
    for source in sources {
        if matches!(
            source.branch_membership,
            sensiblaw_pg_source_store::ChatBranchMembership::Inactive
        ) {
            inactive_branch_message_count += 1;
        }
        let statements =
            load_source_statements_for_document(&config, &source.document_ref)
                .map_err(|error| error.to_string())?;
        statement_count += statements.len();
        messages.push(ConversationMessageView { source, statements });
    }

    Ok(ConversationSourceWorkspace {
        conversation_ref: conversation_ref.to_owned(),
        messages,
        statement_count,
        inactive_branch_message_count,
        creates_semantic_authority: false,
        claim_truth_promoted: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_shape_keeps_message_and_statement_counts_separate() {
        let workspace = ConversationSourceWorkspace {
            conversation_ref: "conversation:test".into(),
            messages: vec![],
            statement_count: 0,
            inactive_branch_message_count: 0,
            creates_semantic_authority: false,
            claim_truth_promoted: false,
        };
        assert_eq!(workspace.messages.len(), 0);
        assert_eq!(workspace.statement_count, 0);
        assert!(!workspace.creates_semantic_authority);
        assert!(!workspace.claim_truth_promoted);
    }
}
