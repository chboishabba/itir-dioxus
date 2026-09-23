use std::{env, process, sync::mpsc};

use itir_dioxus::{
    visual::{
        command::{decode_gpu, decode_shell, ShellInput, VisualObjectId},
        gpu::{
            decode_graph_pick, prepare_graph_gpu_plan, upload_graph_gpu_buffers,
            GraphGpuPlan, GraphRenderer,
        },
        selection::{reduce_selection, SelectionState},
    },
    workbench::production_comparative::load_postgres_comparative_workbench,
};

const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;
const COPY_BYTES_PER_ROW: u32 = 256;

fn clip_to_pixel(x: f32, y: f32) -> (u32, u32) {
    let nx = (x * 0.5 + 0.5).clamp(0.0, 1.0);
    let ny = (1.0 - (y * 0.5 + 0.5)).clamp(0.0, 1.0);
    (
        (nx * (WIDTH - 1) as f32).round() as u32,
        (ny * (HEIGHT - 1) as f32).round() as u32,
    )
}

fn main() {
    if let Err(error) = pollster::block_on(run()) {
        eprintln!("postgres comparative gpu receipt failed: {error}");
        process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err(
            "usage: m11_postgres_comparative_gpu_receipt <before-projection-ref> <after-projection-ref>"
                .into(),
        );
    }

    let model = load_postgres_comparative_workbench(
        "comparison:postgres:gpu",
        &args[0],
        &args[1],
        None,
    )?;

    let shared = model
        .topology
        .comparative
        .shared_semantic_refs
        .iter()
        .find_map(|semantic_ref| {
            let before = model
                .topology
                .before
                .nodes
                .iter()
                .find(|node| {
                    node.semantic_ref == *semantic_ref
                        && !node.source_refs.is_empty()
                        && !node.provenance_refs.is_empty()
                })?;
            let after = model
                .topology
                .after
                .nodes
                .iter()
                .find(|node| {
                    node.semantic_ref == *semantic_ref
                        && !node.source_refs.is_empty()
                        && !node.provenance_refs.is_empty()
                })?;
            (before.id == after.id).then_some((
                semantic_ref.clone(),
                before.id,
                before.source_refs.clone(),
                before.provenance_refs.clone(),
            ))
        })
        .ok_or_else(|| {
            "typed comparative pair has no shared provenance-bearing node".to_string()
        })?;

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
    let renderer = GraphRenderer::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb);

    let before_plan = prepare_graph_gpu_plan(&model.topology.before)?;
    let after_plan = prepare_graph_gpu_plan(&model.topology.after)?;
    let delta_plan = prepare_graph_gpu_plan(&model.topology.delta)?;

    draw_panel(&device, &queue, &renderer, &before_plan, "before")?;
    draw_panel(&device, &queue, &renderer, &after_plan, "after")?;
    draw_panel(&device, &queue, &renderer, &delta_plan, "delta")?;

    let before_state =
        pick_object(&device, &queue, &renderer, &before_plan, shared.1, "before")?;
    let after_state =
        pick_object(&device, &queue, &renderer, &after_plan, shared.1, "after")?;
    let shell_state = reduce_selection(
        decode_shell(ShellInput::Select(shared.1)),
        SelectionState::default(),
    );
    if before_state != after_state || before_state != shell_state {
        return Err(format!(
            "typed comparative pick parity failed: before={before_state:?}, after={after_state:?}, shell={shell_state:?}"
        ));
    }

    println!("carrier=typed-rust");
    println!("database_source=postgres");
    println!("adapter_name={}", adapter_info.name);
    println!("adapter_backend={:?}", adapter_info.backend);
    println!("left_world_ref={}", model.selectors.left_ref);
    println!("right_world_ref={}", model.selectors.right_ref);
    println!("before_draw_submitted=true");
    println!("after_draw_submitted=true");
    println!("delta_draw_submitted=true");
    println!("shared_semantic_ref={}", shared.0);
    println!("shared_visual_object_id={}", shared.1.0);
    println!("source_refs={}", shared.2.join(","));
    println!("provenance_refs={}", shared.3.join(","));
    println!("source_refs_non_empty=true");
    println!("provenance_refs_non_empty=true");
    println!("before_after_gpu_pick_parity=true");
    println!("shell_gpu_reducer_parity=true");
    println!("delta_node_count={}", model.topology.delta.nodes.len());
    println!("delta_edge_count={}", model.topology.delta.edges.len());
    println!("creates_semantic_authority=false");
    println!("creates_claim_truth=false");
    println!("predicts_outcome=false");

    Ok(())
}

fn draw_panel(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &GraphRenderer,
    plan: &GraphGpuPlan,
    label: &str,
) -> Result<(), String> {
    let buffers = upload_graph_gpu_buffers(device, plan);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
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
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(label),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
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
    queue.submit([encoder.finish()]);
    Ok(())
}

fn pick_object(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &GraphRenderer,
    plan: &GraphGpuPlan,
    object: VisualObjectId,
    label: &str,
) -> Result<SelectionState, String> {
    let buffers = upload_graph_gpu_buffers(device, plan);
    let vertex = plan
        .node_vertices
        .iter()
        .find(|vertex| vertex.object_id == object)
        .ok_or_else(|| format!("{label} plan does not contain shared object"))?;
    let (pick_x, pick_y) = clip_to_pixel(vertex.position[0], vertex.position[1]);

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
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
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: u64::from(COPY_BYTES_PER_ROW),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(label),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
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
            texture: &texture,
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
    let pick_id = u32::from_ne_bytes(
        mapped[0..4]
            .try_into()
            .map_err(|_| "pick readback returned fewer than four bytes".to_string())?,
    );
    drop(mapped);
    readback.unmap();

    Ok(reduce_selection(
        decode_gpu(decode_graph_pick(Some(pick_id), plan)),
        SelectionState::default(),
    ))
}
