use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VisualObjectId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainCommand {
    SelectObject(VisualObjectId),
    FocusProvenance(VisualObjectId),
    FollowTarget(VisualObjectId),
    OpenSource(VisualObjectId),
    ClearSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellInput {
    Select(VisualObjectId),
    FocusProvenance(VisualObjectId),
    FollowTarget(VisualObjectId),
    OpenSource(VisualObjectId),
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPickInput {
    Hit(VisualObjectId),
    FocusProvenance(VisualObjectId),
    FollowTarget(VisualObjectId),
    OpenSource(VisualObjectId),
    Miss,
}

pub fn decode_shell(input: ShellInput) -> DomainCommand {
    match input {
        ShellInput::Select(id) => DomainCommand::SelectObject(id),
        ShellInput::FocusProvenance(id) => DomainCommand::FocusProvenance(id),
        ShellInput::FollowTarget(id) => DomainCommand::FollowTarget(id),
        ShellInput::OpenSource(id) => DomainCommand::OpenSource(id),
        ShellInput::Clear => DomainCommand::ClearSelection,
    }
}

pub fn decode_gpu(input: GpuPickInput) -> DomainCommand {
    match input {
        GpuPickInput::Hit(id) => DomainCommand::SelectObject(id),
        GpuPickInput::FocusProvenance(id) => DomainCommand::FocusProvenance(id),
        GpuPickInput::FollowTarget(id) => DomainCommand::FollowTarget(id),
        GpuPickInput::OpenSource(id) => DomainCommand::OpenSource(id),
        GpuPickInput::Miss => DomainCommand::ClearSelection,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_and_gpu_pick_decode_to_same_command() {
        let id = VisualObjectId(42);
        assert_eq!(
            decode_shell(ShellInput::Select(id)),
            decode_gpu(GpuPickInput::Hit(id))
        );
    }

    #[test]
    fn richer_interaction_intents_preserve_shell_gpu_parity() {
        let id = VisualObjectId(7);
        assert_eq!(
            decode_shell(ShellInput::FocusProvenance(id)),
            decode_gpu(GpuPickInput::FocusProvenance(id))
        );
        assert_eq!(
            decode_shell(ShellInput::FollowTarget(id)),
            decode_gpu(GpuPickInput::FollowTarget(id))
        );
        assert_eq!(
            decode_shell(ShellInput::OpenSource(id)),
            decode_gpu(GpuPickInput::OpenSource(id))
        );
    }
}
