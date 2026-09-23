use super::command::{DomainCommand, VisualObjectId};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SelectionState {
    pub selected: Option<VisualObjectId>,
}

pub fn reduce_selection(command: DomainCommand, state: SelectionState) -> SelectionState {
    match command {
        DomainCommand::SelectObject(id) => SelectionState { selected: Some(id) },
        DomainCommand::ClearSelection => SelectionState { selected: None },
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
}
