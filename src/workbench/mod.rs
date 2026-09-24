#[cfg(feature = "production-data")]
pub mod production_comparative;
#[cfg(feature = "production-data")]
pub mod semantic_trace;
#[cfg(feature = "production-data")]
pub mod timeline;
#[cfg(feature = "production-data")]
pub mod review;
pub mod persisted_comparative;
pub mod comparative;
pub mod au_fact_review;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkbenchStageKind {
    Journal,
    Timeline,
    Handoff,
    MatterProof,
    Research,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageAvailability {
    Available,
    Blocked { reason: String },
    Unavailable { reason: String },
}

impl StageAvailability {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Blocked { .. } => "blocked",
            Self::Unavailable { .. } => "unavailable",
        }
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Available => None,
            Self::Blocked { reason } | Self::Unavailable { reason } => Some(reason),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkbenchStage {
    pub kind: WorkbenchStageKind,
    pub availability: StageAvailability,
    pub semantic_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnifiedWorkbenchReadModel {
    pub world_ref: String,
    pub stages: Vec<WorkbenchStage>,
    pub candidate_only: bool,
    pub creates_semantic_authority: bool,
    pub creates_claim_truth: bool,
    pub pays_residual: bool,
}

impl UnifiedWorkbenchReadModel {
    pub fn stage(&self, kind: WorkbenchStageKind) -> Option<&WorkbenchStage> {
        self.stages.iter().find(|stage| stage.kind == kind)
    }
}

pub fn wave5_personal_handoff_read_model() -> UnifiedWorkbenchReadModel {
    UnifiedWorkbenchReadModel {
        world_ref: "world:wave5:professional-handoff".into(),
        stages: vec![
            WorkbenchStage {
                kind: WorkbenchStageKind::Journal,
                availability: StageAvailability::Available,
                semantic_refs: vec![
                    "fact:f5b42c5d8feb0ec6".into(),
                    "src:aae9767bfc03c41a".into(),
                    "statement:0b109faaec28961a".into(),
                    "fact:e0667b55061037f4".into(),
                    "src:816003715d9b4ab0".into(),
                    "statement:9d022f16a280c923".into(),
                    "fact:2499c1b666ccd133".into(),
                    "src:5e78e41d87374808".into(),
                    "statement:6a76d26b680c03e9".into(),
                ],
            },
            WorkbenchStage {
                kind: WorkbenchStageKind::Timeline,
                availability: StageAvailability::Unavailable {
                    reason: "no-assembled-event-coordinate".into(),
                },
                semantic_refs: vec![],
            },
            WorkbenchStage {
                kind: WorkbenchStageKind::Handoff,
                availability: StageAvailability::Blocked {
                    reason: "reviewed-therapist-note-awaits-real-share-scope-receipt".into(),
                },
                semantic_refs: vec!["coordinate:wave5:therapist-note".into()],
            },
            WorkbenchStage {
                kind: WorkbenchStageKind::MatterProof,
                availability: StageAvailability::Unavailable {
                    reason: "no-attached-legal-proof-graph".into(),
                },
                semantic_refs: vec![],
            },
            WorkbenchStage {
                kind: WorkbenchStageKind::Research,
                availability: StageAvailability::Available,
                semantic_refs: vec![
                    "coordinate:wave5:clinic-letter".into(),
                    "coordinate:wave5:user-journal-account".into(),
                    "coordinate:wave5:therapist-note".into(),
                ],
            },
        ],
        candidate_only: true,
        creates_semantic_authority: false,
        creates_claim_truth: false,
        pays_residual: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuLegalBearingInput {
    pub world_ref: String,
    pub journal_or_source_refs: Vec<String>,
    pub timeline_event_refs: Vec<String>,
    pub handoff_refs: Vec<String>,
    pub legal_node_refs: Vec<String>,
    pub legal_edge_refs: Vec<String>,
    pub research_residual_refs: Vec<String>,
}

pub fn au_legal_bearing_read_model(input: AuLegalBearingInput) -> UnifiedWorkbenchReadModel {
    let timeline = if input.timeline_event_refs.is_empty() {
        StageAvailability::Unavailable {
            reason: "no-persisted-timeline-event".into(),
        }
    } else {
        StageAvailability::Available
    };

    let proof = if input.legal_node_refs.is_empty() {
        StageAvailability::Unavailable {
            reason: "no-persisted-legal-follow-graph".into(),
        }
    } else {
        StageAvailability::Available
    };

    let handoff = if input.handoff_refs.is_empty() {
        StageAvailability::Unavailable {
            reason: "no-persisted-handoff-fibre".into(),
        }
    } else {
        StageAvailability::Available
    };

    let research = if input.research_residual_refs.is_empty() {
        StageAvailability::Blocked {
            reason: "no-live-research-residual".into(),
        }
    } else {
        StageAvailability::Available
    };

    let mut proof_refs = input.legal_node_refs;
    proof_refs.extend(input.legal_edge_refs);

    UnifiedWorkbenchReadModel {
        world_ref: input.world_ref,
        stages: vec![
            WorkbenchStage {
                kind: WorkbenchStageKind::Journal,
                availability: StageAvailability::Available,
                semantic_refs: input.journal_or_source_refs,
            },
            WorkbenchStage {
                kind: WorkbenchStageKind::Timeline,
                availability: timeline,
                semantic_refs: input.timeline_event_refs,
            },
            WorkbenchStage {
                kind: WorkbenchStageKind::Handoff,
                availability: handoff,
                semantic_refs: input.handoff_refs,
            },
            WorkbenchStage {
                kind: WorkbenchStageKind::MatterProof,
                availability: proof,
                semantic_refs: proof_refs,
            },
            WorkbenchStage {
                kind: WorkbenchStageKind::Research,
                availability: research,
                semantic_refs: input.research_residual_refs,
            },
        ],
        candidate_only: true,
        creates_semantic_authority: false,
        creates_claim_truth: false,
        pays_residual: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave5_does_not_fabricate_timeline_or_proof() {
        let model = wave5_personal_handoff_read_model();
        assert!(matches!(
            model.stage(WorkbenchStageKind::Timeline).unwrap().availability,
            StageAvailability::Unavailable { .. }
        ));
        assert!(matches!(
            model.stage(WorkbenchStageKind::MatterProof).unwrap().availability,
            StageAvailability::Unavailable { .. }
        ));
        assert!(!model.creates_semantic_authority);
        assert!(!model.creates_claim_truth);
        assert!(!model.pays_residual);
    }

    #[test]
    fn au_legal_graph_makes_proof_available_only_when_persisted_nodes_exist() {
        let absent = au_legal_bearing_read_model(AuLegalBearingInput {
            world_ref: "world:au".into(),
            journal_or_source_refs: vec!["source:au:1".into()],
            timeline_event_refs: vec![],
            handoff_refs: vec![],
            legal_node_refs: vec![],
            legal_edge_refs: vec![],
            research_residual_refs: vec![],
        });
        assert!(matches!(
            absent.stage(WorkbenchStageKind::MatterProof).unwrap().availability,
            StageAvailability::Unavailable { .. }
        ));

        let present = au_legal_bearing_read_model(AuLegalBearingInput {
            world_ref: "world:au".into(),
            journal_or_source_refs: vec!["source:au:1".into()],
            timeline_event_refs: vec!["event:au:1".into()],
            handoff_refs: vec![],
            legal_node_refs: vec!["legal-node:1".into()],
            legal_edge_refs: vec!["legal-edge:1".into()],
            research_residual_refs: vec!["residual:1".into()],
        });
        assert_eq!(
            present.stage(WorkbenchStageKind::MatterProof).unwrap().availability,
            StageAvailability::Available
        );
        assert_eq!(
            present.stage(WorkbenchStageKind::Timeline).unwrap().availability,
            StageAvailability::Available
        );
    }
}
