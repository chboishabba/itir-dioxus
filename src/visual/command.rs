use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VisualObjectId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainCommand {
    SelectObject(VisualObjectId),
    ClearSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellInput {
    Select(VisualObjectId),
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPickInput {
    Hit(VisualObjectId),
    Miss,
}

pub fn decode_shell(input: ShellInput) -> DomainCommand {
    match input {
        ShellInput::Select(id) => DomainCommand::SelectObject(id),
        ShellInput::Clear => DomainCommand::ClearSelection,
    }
}

pub fn decode_gpu(input: GpuPickInput) -> DomainCommand {
    match input {
        GpuPickInput::Hit(id) => DomainCommand::SelectObject(id),
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
}
