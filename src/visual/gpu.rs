//! wgpu-facing projection boundary.
//!
//! The GPU consumes Visualisation IR and returns pick proposals. It does not own
//! canonical semantic state or mutate the world directly.

use super::command::{GpuPickInput, VisualObjectId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuSurfaceDescriptor {
    pub format: wgpu::TextureFormat,
}

impl Default for GpuSurfaceDescriptor {
    fn default() -> Self {
        Self {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
        }
    }
}

pub fn decode_pick_id(raw_pick_id: Option<u64>) -> GpuPickInput {
    match raw_pick_id {
        Some(id) => GpuPickInput::Hit(VisualObjectId(id)),
        None => GpuPickInput::Miss,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_id_remains_a_proposal_not_state_mutation() {
        assert_eq!(
            decode_pick_id(Some(42)),
            GpuPickInput::Hit(VisualObjectId(42))
        );
    }
}
