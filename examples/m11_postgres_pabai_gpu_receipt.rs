use std::{env, process, sync::mpsc};

use itir_dioxus::{
    visual::{
        command::{decode_gpu, decode_shell, ShellInput, VisualObjectId},
        gpu::{
            decode_graph_pick, prepare_graph_gpu_plan, upload_graph_gpu_buffers,
            GraphGpuPlan, GraphRenderer,
        },
        ir::GraphIr,
        selection::{reduce_selection, SelectionState},
    },
    workbench::{
        comparative::{
            ComparativePresentationChangeLayer, ComparativeWorkbenchReadModel,
        },
        production_comparative::load_postgres_pabai_three_way_workbench,
    },
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
        eprintln!("postgres Pabai comparative gpu receipt failed: {error}");
        process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 3 {
        return Err(
            "usage: m11_postgres_pabai_gpu_receipt <w0-projection-ref> <w1-projection-ref> <w2-projection-ref>"
                .into(),
        );
    }

    let sequence =
        load_postgres_pabai_three_way_workbench(&args[0], &args[1], &args[2])?;

    let d_ref = "coordinate:pabai:comparative:defeater";
    let c_ref = "coordinate:pabai:comparative:counter-defeater";
    validate_typed_answer_change(&sequence.w0_to_w1, d_ref, "D")?;
    validate_typed_answer_change(&sequence.w1_to_w2, c_ref, "C")?;

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

    let w01_before = prepare_graph_gpu_plan(&sequence.w0_to_w1.topology.before)?;
    let w01_after = prepare_graph_gpu_plan(&sequence.w0_to_w1.topology.after)?;
    let w01_delta = prepare_graph_gpu_plan(&sequence.w0_to_w1.topology.delta)?;
    let w12_before = prepare_graph_gpu_plan(&sequence.w1_to_w2.topology.before)?;
    let w12_after = prepare_graph_gpu_plan(&sequence.w1_to_w2.topology.after)?;
    let w12_delta = prepare_graph_gpu_plan(&sequence.w1_to_w2.topology.delta)?;

    for (label, plan) in [
        ("w0-w1-before", &w01_before),
        ("w0-w1-after", &w01_after),
        ("w0-w1-delta", &w01_delta),
        ("w1-w2-before", &w12_before),
        ("w1-w2-after", &w12_after),
        ("w1-w2-delta", &w12_delta),
    ] {
        draw_panel(&device, &queue, &renderer, plan, label)?;
    }

    let d_after_id = visual_node_id(&sequence.w0_to_w1.topology.after, d_ref, "D after")?;
    let d_delta_id = visual_node_id(&sequence.w0_to_w1.topology.delta, d_ref, "D delta")?;
    if d_after_id != d_delta_id {
        return Err("D canonical VisualObjectId differs between After and Delta".into());
    }
    let d_after_state =
        pick_object(&device, &queue, &renderer, &w01_after, d_after_id, "D-after")?;
    let d_delta_state =
        pick_object(&device, &queue, &renderer, &w01_delta, d_delta_id, "D-delta")?;
    let d_shell_state = reduce_selection(
        decode_shell(ShellInput::Select(d_after_id)),
        SelectionState::default(),
    );
    if d_after_state != d_delta_state || d_after_state != d_shell_state {
        return Err(format!(
            "D pick parity failed: after={d_after_state:?}, delta={d_delta_state:?}, shell={d_shell_state:?}"
        ));
    }

    let c_after_id = visual_node_id(&sequence.w1_to_w2.topology.after, c_ref, "C after")?;
    let c_delta_id = visual_node_id(&sequence.w1_to_w2.topology.delta, c_ref, "C delta")?;
    if c_after_id != c_delta_id {
        return Err("C canonical VisualObjectId differs between After and Delta".into());
    }
    let c_after_state =
        pick_object(&device, &queue, &renderer, &w12_after, c_after_id, "C-after")?;
    let c_delta_state =
        pick_object(&device, &queue, &renderer, &w12_delta, c_delta_id, "C-delta")?;
    let c_shell_state = reduce_selection(
        decode_shell(ShellInput::Select(c_after_id)),
        SelectionState::default(),
    );
    if c_after_state != c_delta_state || c_after_state != c_shell_state {
        return Err(format!(
            "C pick parity failed: after={c_after_state:?}, delta={c_delta_state:?}, shell={c_shell_state:?}"
        ));
    }

    println!("carrier=typed-rust");
    println!("database_source=postgres");
    println!("adapter_name={}", adapter_info.name);
    println!("adapter_backend={:?}", adapter_info.backend);
    println!("w0_projection_ref={}", args[0]);
    println!("w1_projection_ref={}", args[1]);
    println!("w2_projection_ref={}", args[2]);
    println!("six_gpu_draws_submitted=true");
    print_annotation_receipt("D", &sequence.w0_to_w1, d_ref)?;
    print_annotation_receipt("C", &sequence.w1_to_w2, c_ref)?;
    println!("D_after_delta_visual_id_parity=true");
    println!("D_gpu_shell_reducer_parity=true");
    println!("C_after_delta_visual_id_parity=true");
    println!("C_gpu_shell_reducer_parity=true");
    println!("creates_semantic_authority=false");
    println!("creates_claim_truth=false");
    println!("predicts_outcome=false");

    Ok(())
}

fn validate_typed_answer_change(
    model: &ComparativeWorkbenchReadModel,
    semantic_ref: &str,
    label: &str,
) -> Result<(), String> {
    let annotation = model
        .explanation_overlay
        .typed_change_annotations
        .get(semantic_ref)
        .ok_or_else(|| format!("{label} typed annotation missing"))?;
    if annotation.layer != ComparativePresentationChangeLayer::Applicability {
        return Err(format!("{label} layer is not Applicability"));
    }
    if !annotation.answer_changing {
        return Err(format!("{label} is not marked answer-changing"));
    }
    if annotation.justification_refs.is_empty() {
        return Err(format!("{label} has no justification receipts"));
    }
    Ok(())
}

fn print_annotation_receipt(
    label: &str,
    model: &ComparativeWorkbenchReadModel,
    semantic_ref: &str,
) -> Result<(), String> {
    let annotation = model
        .explanation_overlay
        .typed_change_annotations
        .get(semantic_ref)
        .ok_or_else(|| format!("{label} typed annotation missing"))?;
    println!("{label}_semantic_ref={semantic_ref}");
    println!("{label}_layer={:?}", annotation.layer);
    println!("{label}_answer_changing={}", annotation.answer_changing);
    println!(
        "{label}_justification_refs={}",
        annotation.justification_refs.join(",")
    );
    if let Some(explanation_ref) = annotation.explanation_ref.as_ref() {
        println!("{label}_explanation_ref={explanation_ref}");
    }
    Ok(())
}

fn visual_node_id(graph: &GraphIr, semantic_ref: &str, label: &str) -> Result<VisualObjectId, String> {
    let node = graph
        .nodes
        .iter()
        .find(|node| node.semantic_ref == semantic_ref)
        .ok_or_else(|| format!("{label} semantic object is not a GPU-pickable node"))?;
    if node.source_refs.is_empty() || node.provenance_refs.is_empty() {
        return Err(format!("{label} lacks source/provenance closure"));
    }
    Ok(node.id)
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
        .ok_or_else(|| format!("{label} plan does not contain answer-changing object"))?;
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
