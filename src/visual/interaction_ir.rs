//! Intent-level interaction IR mirrored from DASHI.
//!
//! This module is deliberately above physical gestures and below semantic
//! reducers. DOM selectors, pointer coordinates and GPU hit mechanics are not
//! canonical interaction identity.

use serde::{Deserialize, Serialize};

use super::command::{DomainCommand, VisualObjectId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetKind {
    Semantic,
    Source,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    Activate,
    Focus,
    Select,
    Expand,
    Collapse,
    Follow,
    OpenSource,
    Zoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationKind {
    Visible,
    Hidden,
    Focused,
    Selected,
    Expanded,
    Collapsed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoomLevel {
    In,
    Out,
    Fit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionTarget {
    pub object_id: VisualObjectId,
    pub target_kind: TargetKind,
    pub semantic_ref: String,
    pub source_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionIntent {
    pub action: ActionKind,
    pub target: InteractionTarget,
    pub zoom: Option<ZoomLevel>,
}

pub fn intent_to_domain_command(intent: &InteractionIntent) -> Option<DomainCommand> {
    let id = intent.target.object_id;
    match intent.action {
        ActionKind::Activate | ActionKind::Select => Some(DomainCommand::SelectObject(id)),
        ActionKind::Focus => Some(DomainCommand::FocusProvenance(id)),
        ActionKind::Follow => Some(DomainCommand::FollowTarget(id)),
        ActionKind::OpenSource => Some(DomainCommand::OpenSource(id)),
        ActionKind::Expand | ActionKind::Collapse | ActionKind::Zoom => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> InteractionTarget {
        InteractionTarget {
            object_id: VisualObjectId(42),
            target_kind: TargetKind::Semantic,
            semantic_ref: "semantic:au:node:42".into(),
            source_ref: Some("source:au:judgment".into()),
        }
    }

    #[test]
    fn intent_ir_maps_only_semantic_actions_into_domain_commands() {
        let select = InteractionIntent {
            action: ActionKind::Select,
            target: target(),
            zoom: None,
        };
        let source = InteractionIntent {
            action: ActionKind::OpenSource,
            target: target(),
            zoom: None,
        };
        let zoom = InteractionIntent {
            action: ActionKind::Zoom,
            target: target(),
            zoom: Some(ZoomLevel::Fit),
        };

        assert_eq!(
            intent_to_domain_command(&select),
            Some(DomainCommand::SelectObject(VisualObjectId(42)))
        );
        assert_eq!(
            intent_to_domain_command(&source),
            Some(DomainCommand::OpenSource(VisualObjectId(42)))
        );
        assert_eq!(intent_to_domain_command(&zoom), None);
    }

    #[test]
    fn physical_gesture_is_not_part_of_intent_identity() {
        let a = InteractionIntent {
            action: ActionKind::Select,
            target: target(),
            zoom: None,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }
}
