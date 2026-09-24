use std::{
    env,
    fs,
    process,
    sync::mpsc,
};

use itir_dioxus::{
    visual::{
        command::{decode_gpu, decode_shell, ShellInput},
        gpu::{
            decode_graph_pick, prepare_graph_gpu_plan, upload_graph_gpu_buffers, GraphRenderer,
        },
        selection::{reduce_selection, SelectionState},
    },
    workbench::au_fact_review::replay_legacy_au_workbench_json,
};

const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;
const COPY_BYTES_PER_ROW: u32 = 256;

fn clip_to_pixel(x: f32, y: f32) -> (u32, u32) {
    let nx = (x * 0.5 + 0.5).clamp(0.0, 1.0);
    let ny = (1.0 - (y * 0.5 + 0.5)).clamp(0.0, 1.0);
    let px = (nx * (WIDTH - 1) as f32).round() as u32;
    let py = (ny * (HEIGHT - 1) as f32).round() as u32;
    (px, py)
}

fn main() {
    if let Err(error) = pollster::block_on(run()) {
        eprintln!("gpu receipt failed: {error}");
        process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let path = env::args()
        .nth(1)
        .ok_or_else(|| "usage: au_legal_gpu_receipt <persisted-au-workbench.json>".to_string())?;
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {path}: {error}"))?;
    let projection = replay_legacy_au_workbench_json(&raw, 20, 30)?;

    if projection.graph_ir.nodes.is_empty() {
        return Err("persisted AU graph has no nodes to draw/pick".into());
    }

    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .map_err(|error| format!("request_adapter failed: {error}"))?;
    let adapter_info = adapter.get_info();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .map_err(|error| format!("request_device failed: {error}"))?;

    let plan = prepare_graph_gpu_plan(&projection.graph_ir)?;
    let buffers = upload_graph_gpu_buffers(&device, &plan);
    let renderer = GraphRenderer::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb);

    let color_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("itir-m10-color-receipt"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let pick_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("itir-m10-pick-receipt"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Uint,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let pick_view = pick_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let provenance_bearing_node = projection
        .graph_ir
        .nodes
        .iter()
        .find(|node| !node.hidden && !node.source_refs.is_empty() && !node.provenance_refs.is_empty())
        .ok_or_else(|| {
            "bounded real GraphIr has no visible node with both source_refs and provenance_refs"
                .to_string()
        })?;
    let selected_vertex = plan
        .node_vertices
        .iter()
        .find(|vertex| vertex.object_id == provenance_bearing_node.id)
        .copied()
        .ok_or_else(|| "provenance-bearing GraphIr node was not uploaded to GPU plan".to_string())?;
    let expected_pick_id = selected_vertex.pick_id;
    let expected_object = selected_vertex.object_id;
    let (pick_x, pick_y) =
        clip_to_pixel(selected_vertex.position[0], selected_vertex.position[1]);

    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("itir-m10-pick-readback"),
        size: u64::from(COPY_BYTES_PER_ROW),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("itir-m10-offscreen-receipt"),
    });

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("itir-m10-color-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        renderer.draw(&mut pass, &buffers);
    }

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("itir-m10-pick-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &pick_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        renderer.draw_pick(&mut pass, &buffers);
    }

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &pick_texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: pick_x,
                y: pick_y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(COPY_BYTES_PER_ROW),
                rows_per_image: Some(1),
            },
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );

    let submission = queue.submit([encoder.finish()]);
    let slice = readback.slice(..4);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: None,
        })
        .map_err(|error| format!("device poll failed: {error}"))?;
    receiver
        .recv()
        .map_err(|error| format!("map callback channel failed: {error}"))?
        .map_err(|error| format!("pick readback map failed: {error}"))?;

    let mapped = slice.get_mapped_range();
    let raw_pick_id = u32::from_ne_bytes(
        mapped[0..4]
            .try_into()
            .map_err(|_| "pick readback returned fewer than four bytes".to_string())?,
    );
    drop(mapped);
    readback.unmap();

    if raw_pick_id != expected_pick_id {
        return Err(format!(
            "GPU pick mismatch: expected integer pick id {expected_pick_id}, got {raw_pick_id}"
        ));
    }

    let gpu_input = decode_graph_pick(Some(raw_pick_id), &plan);
    let gpu_state = reduce_selection(decode_gpu(gpu_input), SelectionState::default());
    let shell_state = reduce_selection(
        decode_shell(ShellInput::Select(expected_object)),
        SelectionState::default(),
    );

    if gpu_state != shell_state {
        return Err(format!(
            "shell/GPU reducer mismatch: shell={shell_state:?}, gpu={gpu_state:?}"
        ));
    }

    let inspection = projection
        .graph_ir
        .inspect_object(expected_object)
        .ok_or_else(|| "selected GPU object cannot be reopened in GraphIr".to_string())?;
    if inspection.source_refs.is_empty() {
        return Err("selected real semantic object reopened without source_refs".into());
    }
    if inspection.provenance_refs.is_empty() {
        return Err("selected real semantic object reopened without provenance_refs".into());
    }

    println!("adapter_name={}", adapter_info.name);
    println!("adapter_backend={:?}", adapter_info.backend);
    println!("world_ref={}", projection.read_model.world_ref);
    println!("graph_ref={}", projection.graph_ir.graph_ref);
    println!("graph_node_count={}", projection.graph_ir.nodes.len());
    println!("graph_edge_count={}", projection.graph_ir.edges.len());
    println!("gpu_draw_submitted=true");
    println!("gpu_pick_pixel={pick_x},{pick_y}");
    println!("gpu_pick_id={raw_pick_id}");
    println!("visual_object_id={}", expected_object.0);
    println!("semantic_ref={}", inspection.semantic_ref);
    println!("kind={}", inspection.kind);
    println!("source_refs={}", inspection.source_refs.join(","));
    println!("provenance_refs={}", inspection.provenance_refs.join(","));
    println!("source_refs_non_empty=true");
    println!("provenance_refs_non_empty=true");
    println!("shell_gpu_reducer_parity=true");
    println!("creates_semantic_authority=false");
    println!("creates_claim_truth=false");
    println!("pays_residual=false");

    Ok(())
}
