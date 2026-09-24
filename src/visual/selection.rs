use super::command::{DomainCommand, VisualObjectId};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SelectionState {
    pub selected: Option<VisualObjectId>,
    pub provenance_focus: Option<VisualObjectId>,
    pub follow_target: Option<VisualObjectId>,
    pub open_source: Option<VisualObjectId>,
}

pub fn reduce_selection(command: DomainCommand, state: SelectionState) -> SelectionState {
    match command {
        DomainCommand::SelectObject(id) => SelectionState {
            selected: Some(id),
            ..state
        },
        DomainCommand::FocusProvenance(id) => SelectionState {
            provenance_focus: Some(id),
            ..state
        },
        DomainCommand::FollowTarget(id) => SelectionState {
            follow_target: Some(id),
            ..state
        },
        DomainCommand::OpenSource(id) => SelectionState {
            open_source: Some(id),
            ..state
        },
        DomainCommand::ClearSelection => SelectionState::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visual::command::{decode_gpu, decode_shell, GpuPickInput, ShellInput};

    #[test]
    fn shell_and_gpu_inputs_produce_identical_selection_state() {
        let id = VisualObjectId(42);
        let start = SelectionState::default();
        let shell = reduce_selection(decode_shell(ShellInput::Select(id)), start);
        let gpu = reduce_selection(decode_gpu(GpuPickInput::Hit(id)), start);
        assert_eq!(shell, gpu);
    }

    #[test]
    fn shell_and_gpu_provenance_follow_and_source_intents_share_reducer() {
        let id = VisualObjectId(7);
        let start = SelectionState::default();

        assert_eq!(
            reduce_selection(decode_shell(ShellInput::FocusProvenance(id)), start),
            reduce_selection(decode_gpu(GpuPickInput::FocusProvenance(id)), start)
        );
        assert_eq!(
            reduce_selection(decode_shell(ShellInput::FollowTarget(id)), start),
            reduce_selection(decode_gpu(GpuPickInput::FollowTarget(id)), start)
        );
        assert_eq!(
            reduce_selection(decode_shell(ShellInput::OpenSource(id)), start),
            reduce_selection(decode_gpu(GpuPickInput::OpenSource(id)), start)
        );
    }
}
