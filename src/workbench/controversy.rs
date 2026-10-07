#![cfg(feature = "production-data")]

use sensiblaw_pg_source_store::{
    load_database_config, load_matter_controversies_for_matter,
    project_matter_personas, project_matter_reverse_proof_search,
    MatterControversyDraft, MatterPersonaProjections, MatterProceduralGoal,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatterControversyWorkspaceEntry {
    pub controversy: MatterControversyDraft,
    pub personas: MatterPersonaProjections,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatterControversyWorkspace {
    pub matter_ref: String,
    pub entries: Vec<MatterControversyWorkspaceEntry>,
    pub creates_semantic_authority: bool,
    pub claim_truth_promoted: bool,
    pub canonical_world_mutated: bool,
}

pub fn load_matter_controversy_workspace(
    matter_ref: &str,
) -> Result<MatterControversyWorkspace, String> {
    if matter_ref.trim().is_empty() {
        return Err("matter ref is required for controversy projection".into());
    }
    let config = load_database_config(None).map_err(|error| error.to_string())?;
    let persisted = load_matter_controversies_for_matter(&config, matter_ref)
        .map_err(|error| error.to_string())?;
    let mut entries = Vec::with_capacity(persisted.len());
    for packet in persisted {
        if packet.controversy.matter_ref != matter_ref
            || !packet.derived_only
            || packet.creates_semantic_authority
            || packet.creates_legal_authority
            || packet.applicability_promoted
            || packet.claim_truth_promoted
        {
            return Err("persisted controversy crossed Matter/non-promotion boundary".into());
        }
        let reverse = project_matter_reverse_proof_search(
            &packet.controversy,
            MatterProceduralGoal::PrepareForAdjudication,
        )
        .map_err(|error| error.to_string())?;
        let personas = project_matter_personas(&packet.controversy, &reverse)
            .map_err(|error| error.to_string())?;
        if personas.matter_ref != matter_ref
            || personas.controversy_ref != packet.controversy.controversy_ref
            || personas.creates_semantic_authority
            || personas.claim_truth_promoted
            || personas.canonical_world_mutated
        {
            return Err("controversy persona projection crossed Matter boundary".into());
        }
        entries.push(MatterControversyWorkspaceEntry {
            controversy: packet.controversy,
            personas,
        });
    }
    Ok(MatterControversyWorkspace {
        matter_ref: matter_ref.to_owned(),
        entries,
        creates_semantic_authority: false,
        claim_truth_promoted: false,
        canonical_world_mutated: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_type_is_read_only_and_same_matter_scoped() {
        let workspace = MatterControversyWorkspace {
            matter_ref: "matter:test".into(),
            entries: vec![],
            creates_semantic_authority: false,
            claim_truth_promoted: false,
            canonical_world_mutated: false,
        };
        assert_eq!(workspace.matter_ref, "matter:test");
        assert!(!workspace.creates_semantic_authority);
        assert!(!workspace.claim_truth_promoted);
        assert!(!workspace.canonical_world_mutated);
    }
}