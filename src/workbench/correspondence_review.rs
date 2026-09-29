#![cfg(feature = "production-data")]
//! M10.4: typed S29 commands on *reviewed relation proposals*.
//! No UI mutation of L2, source genealogy, S30 Matter, or canonical world.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use sensiblaw_pg_source_store::{
    apply_correspondence_review, load_correspondence_review,
    load_correspondence_review_history, CorrespondenceReviewActionRecord,
    load_database_config, load_review_item, propose_correspondence_review,
    CorrespondenceAxis, CorrespondenceReviewProposal, ReviewAction,
    ReviewCommand, ReviewItem, ReviewReceipt,
};
static COMMAND_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrespondenceReviewWorkspace {
    pub relation: CorrespondenceReviewProposal,
    pub item: ReviewItem,
    pub history: Vec<CorrespondenceReviewActionRecord>,
    pub candidate_only: bool,
    pub creates_semantic_authority: bool,
    pub claim_truth_promoted: bool,
}

fn load_pair(config: &sensiblaw_pg_source_store::DatabaseConfig,
             relation: CorrespondenceReviewProposal)
    -> Result<CorrespondenceReviewWorkspace,String> {
    let item=load_review_item(config,&relation.review_item_ref)
        .map_err(|e|e.to_string())?
        .ok_or_else(||"review item missing after relation reopen".to_owned())?;
    if item.semantic_ref!=relation.relation_ref
        || item.item_kind!=sensiblaw_core::review_workstation::ReviewItemKind::SourceCorrespondence
        || item.creates_semantic_authority || item.claim_truth_promoted
    {return Err("invalid S29 relation review type/authority".into());}
    let history=load_correspondence_review_history(config,&relation.relation_ref)
        .map_err(|e|e.to_string())?;
    Ok(CorrespondenceReviewWorkspace {
        relation,item,history,candidate_only:true,
        creates_semantic_authority:false,claim_truth_promoted:false,
    })
}

pub fn propose_review(
    left:&str,right:&str,chat_message_ref:Option<&str>,
    axis:CorrespondenceAxis,evidence_ref:&str,scope:&str,
) -> Result<CorrespondenceReviewWorkspace,String> {
    let config=load_database_config(None).map_err(|e|e.to_string())?;
    let relation=propose_correspondence_review(
        &config,left,right,chat_message_ref,axis,evidence_ref,scope,
    ).map_err(|e|e.to_string())?;
    load_pair(&config,relation)
}

/// One S29 command, with the existing validator/transaction/status receipt.
/// The target is checked against the immutable relation item before any
/// write. Returning the reopened item prevents optimistic UI-only status.
pub fn execute_relation_review(
    relation_ref:&str,action:ReviewAction,reviewer_ref:&str,
    qualification_ref:Option<String>,evidence_request_ref:Option<String>,
) -> Result<(ReviewReceipt,CorrespondenceReviewWorkspace),String> {
    if relation_ref.trim().is_empty() || reviewer_ref.trim().is_empty() {
        return Err("relation and reviewer identities are required".into());
    }
    let config=load_database_config(None).map_err(|e|e.to_string())?;
    let original=load_correspondence_review(&config,relation_ref)
        .map_err(|e|e.to_string())?
        .ok_or_else(||"relation is not a persisted S29 review item".to_owned())?;
    let now=SystemTime::now().duration_since(UNIX_EPOCH)
        .map_err(|e|e.to_string())?.as_nanos();
    let ordinal=COMMAND_COUNTER.fetch_add(1,Ordering::Relaxed);
    let cmd=ReviewCommand {
        command_ref:format!("s29:relation-command:{relation_ref}:{now}:{ordinal}"),
        review_item_ref:original.review_item_ref,
        action,reviewer_ref:reviewer_ref.into(),
        qualification_ref,evidence_request_ref,
    };
    let (receipt,updated)=apply_correspondence_review(
        &config,relation_ref,&cmd,
    ).map_err(|e|e.to_string())?;
    Ok((receipt,load_pair(&config,updated)?))
}
