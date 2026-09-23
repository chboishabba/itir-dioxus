//! wgpu-facing projection boundary.
//!
//! The GPU consumes Visualisation IR and returns pick proposals. It does not own
//! canonical semantic state or mutate the world directly.

use std::collections::BTreeMap;

use wgpu::util::DeviceExt;

use super::{
    command::{GpuPickInput, VisualObjectId},
    ir::GraphIr,
};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphGpuVertex {
    pub position: [f32; 2],
    pub object_id: VisualObjectId,
    pub pick_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphGpuEdgeVertex {
    pub position: [f32; 2],
    pub edge_id: VisualObjectId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphGpuPlan {
    pub node_vertices: Vec<GraphGpuVertex>,
    pub edge_vertices: Vec<GraphGpuEdgeVertex>,
    pub pick_to_object: BTreeMap<u32, VisualObjectId>,
}

pub struct GraphGpuBuffers {
    pub node_buffer: wgpu::Buffer,
    pub edge_buffer: wgpu::Buffer,
    pub node_vertex_count: u32,
    pub edge_vertex_count: u32,
    pub pick_to_object: BTreeMap<u32, VisualObjectId>,
}

pub fn prepare_graph_gpu_plan(graph: &GraphIr) -> Result<GraphGpuPlan, String> {
    let mut pick_to_object = BTreeMap::new();
    let mut object_to_position = BTreeMap::new();
    let mut node_vertices = Vec::new();

    for (index, node) in graph.nodes.iter().filter(|node| !node.hidden).enumerate() {
        let pick_id = u32::try_from(index + 1)
            .map_err(|_| "graph contains too many pickable objects".to_string())?;
        pick_to_object.insert(pick_id, node.id);
        object_to_position.insert(node.id, [node.x, node.y]);
        node_vertices.push(GraphGpuVertex {
            position: [node.x, node.y],
            object_id: node.id,
            pick_id,
        });
    }

    let mut edge_vertices = Vec::new();
    for edge in graph.edges.iter().filter(|edge| !edge.hidden) {
        let Some(from) = object_to_position.get(&edge.from).copied() else {
            continue;
        };
        let Some(to) = object_to_position.get(&edge.to).copied() else {
            continue;
        };
        edge_vertices.push(GraphGpuEdgeVertex {
            position: from,
            edge_id: edge.id,
        });
        edge_vertices.push(GraphGpuEdgeVertex {
            position: to,
            edge_id: edge.id,
        });
    }

    Ok(GraphGpuPlan {
        node_vertices,
        edge_vertices,
        pick_to_object,
    })
}

pub fn upload_graph_gpu_buffers(
    device: &wgpu::Device,
    plan: &GraphGpuPlan,
) -> GraphGpuBuffers {
    let node_bytes = encode_node_vertices(&plan.node_vertices);
    let edge_bytes = encode_edge_vertices(&plan.edge_vertices);

    let node_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("itir-proof-graph-nodes"),
        contents: &node_bytes,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });
    let edge_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("itir-proof-graph-edges"),
        contents: &edge_bytes,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    GraphGpuBuffers {
        node_buffer,
        edge_buffer,
        node_vertex_count: plan.node_vertices.len() as u32,
        edge_vertex_count: plan.edge_vertices.len() as u32,
        pick_to_object: plan.pick_to_object.clone(),
    }
}

pub fn decode_graph_pick(
    raw_pick_id: Option<u32>,
    plan: &GraphGpuPlan,
) -> GpuPickInput {
    raw_pick_id
        .and_then(|pick_id| plan.pick_to_object.get(&pick_id).copied())
        .map(GpuPickInput::Hit)
        .unwrap_or(GpuPickInput::Miss)
}

fn encode_node_vertices(vertices: &[GraphGpuVertex]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vertices.len() * 20);
    for vertex in vertices {
        bytes.extend_from_slice(&vertex.position[0].to_ne_bytes());
        bytes.extend_from_slice(&vertex.position[1].to_ne_bytes());
        bytes.extend_from_slice(&vertex.object_id.0.to_ne_bytes());
        bytes.extend_from_slice(&vertex.pick_id.to_ne_bytes());
    }
    bytes
}

fn encode_edge_vertices(vertices: &[GraphGpuEdgeVertex]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vertices.len() * 16);
    for vertex in vertices {
        bytes.extend_from_slice(&vertex.position[0].to_ne_bytes());
        bytes.extend_from_slice(&vertex.position[1].to_ne_bytes());
        bytes.extend_from_slice(&vertex.edge_id.0.to_ne_bytes());
    }
    bytes
}


const GRAPH_SHADER: &str = r#"
struct VertexOut {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn node_vs(
    @location(0) center: vec2<f32>,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOut {
    var offsets = array<vec2<f32>, 6>(
        vec2<f32>(-0.025, -0.025),
        vec2<f32>( 0.025, -0.025),
        vec2<f32>( 0.025,  0.025),
        vec2<f32>(-0.025, -0.025),
        vec2<f32>( 0.025,  0.025),
        vec2<f32>(-0.025,  0.025),
    );
    var out: VertexOut;
    out.position = vec4<f32>(center + offsets[vertex_index], 0.0, 1.0);
    return out;
}

@vertex
fn edge_vs(@location(0) position: vec2<f32>) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(position, 0.0, 1.0);
    return out;
}

@fragment
fn node_fs() -> @location(0) vec4<f32> {
    return vec4<f32>(0.12, 0.12, 0.12, 1.0);
}

@fragment
fn edge_fs() -> @location(0) vec4<f32> {
    return vec4<f32>(0.45, 0.45, 0.45, 1.0);
}
"#;

pub struct GraphRenderer {
    node_pipeline: wgpu::RenderPipeline,
    edge_pipeline: wgpu::RenderPipeline,
}

impl GraphRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("itir-proof-graph-shader"),
            source: wgpu::ShaderSource::Wgsl(GRAPH_SHADER.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("itir-proof-graph-layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let node_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("itir-proof-graph-node-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("node_vs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 20,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("node_fs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let edge_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("itir-proof-graph-edge-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("edge_vs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 16,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("edge_fs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            node_pipeline,
            edge_pipeline,
        }
    }

    pub fn draw<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        buffers: &'pass GraphGpuBuffers,
    ) {
        if buffers.edge_vertex_count > 0 {
            pass.set_pipeline(&self.edge_pipeline);
            pass.set_vertex_buffer(0, buffers.edge_buffer.slice(..));
            pass.draw(0..buffers.edge_vertex_count, 0..1);
        }

        if buffers.node_vertex_count > 0 {
            pass.set_pipeline(&self.node_pipeline);
            pass.set_vertex_buffer(0, buffers.node_buffer.slice(..));
            pass.draw(0..6, 0..buffers.node_vertex_count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visual::ir::{GraphIr, VisualEdge, VisualNode};

    fn graph() -> GraphIr {
        GraphIr {
            graph_ref: "graph:au".into(),
            derived_only: true,
            challengeable: true,
            nodes: vec![
                VisualNode {
                    id: VisualObjectId(42),
                    semantic_ref: "node:42".into(),
                    kind: "authority".into(),
                    label: "Authority".into(),
                    source_refs: vec!["source:42".into()],
                    provenance_refs: vec!["receipt:42".into()],
                    hidden: false,
                    x: -0.5,
                    y: 0.0,
                },
                VisualNode {
                    id: VisualObjectId(99),
                    semantic_ref: "node:99".into(),
                    kind: "proposition".into(),
                    label: "Proposition".into(),
                    source_refs: vec![],
                    provenance_refs: vec![],
                    hidden: false,
                    x: 0.5,
                    y: 0.0,
                },
            ],
            edges: vec![VisualEdge {
                id: VisualObjectId(7),
                from: VisualObjectId(42),
                to: VisualObjectId(99),
                semantic_ref: "edge:7".into(),
                kind: "supports".into(),
                source_refs: vec![],
                provenance_refs: vec![],
                hidden: false,
                weight: 1.0,
            }],
        }
    }

    #[test]
    fn pick_id_remains_a_proposal_not_state_mutation() {
        assert_eq!(
            decode_pick_id(Some(42)),
            GpuPickInput::Hit(VisualObjectId(42))
        );
    }

    #[test]
    fn graph_ir_prepares_deterministic_gpu_vertices_and_integer_pick_ids() {
        let first = prepare_graph_gpu_plan(&graph()).unwrap();
        let second = prepare_graph_gpu_plan(&graph()).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.node_vertices.len(), 2);
        assert_eq!(first.edge_vertices.len(), 2);
        assert_eq!(decode_graph_pick(Some(1), &first), GpuPickInput::Hit(VisualObjectId(42)));
        assert_eq!(decode_graph_pick(Some(2), &first), GpuPickInput::Hit(VisualObjectId(99)));
        assert_eq!(decode_graph_pick(Some(500), &first), GpuPickInput::Miss);
    }

    #[test]
    fn hidden_nodes_are_not_uploaded_but_remain_in_graph_ir() {
        let mut graph = graph();
        graph.nodes[1].hidden = true;
        let plan = prepare_graph_gpu_plan(&graph).unwrap();
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(plan.node_vertices.len(), 1);
        assert!(graph.node(VisualObjectId(99)).is_some());
    }

    #[test]
    fn vertex_encoding_is_stable_and_non_json() {
        let plan = prepare_graph_gpu_plan(&graph()).unwrap();
        assert_eq!(encode_node_vertices(&plan.node_vertices).len(), 40);
        assert_eq!(encode_edge_vertices(&plan.edge_vertices).len(), 32);
    }
}
