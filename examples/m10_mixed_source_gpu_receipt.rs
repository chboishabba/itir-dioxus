//! M10.3 optional physical GPU acceptance for the SAME SLR typed source
//! comparison shown by Dioxus. No GPU inference or semantic promotion.
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
    workbench::mixed_source::load_mixed_source_workspace,
};
use sensiblaw_pg_source_store::ContextVisibility;

const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;
const COPY_BYTES_PER_ROW: u32 = 256;

fn clip_to_pixel(x: f32, y: f32) -> (u32, u32) {
    let nx = (x * 0.5 + 0.5).clamp(0.0, 1.0);
    let ny = (1.0 - (y * 0.5 + 0.5)).clamp(0.0, 1.0);
    ((nx * (WIDTH - 1) as f32).round() as u32,
     (ny * (HEIGHT - 1) as f32).round() as u32)
}

fn main() {
    if let Err(error)=pollster::block_on(run()) {
        eprintln!("mixed source gpu receipt failed: {error}");
        process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let args=env::args().skip(1).collect::<Vec<_>>();
    if args.len()!=2 {
        return Err("usage: m10_mixed_source_gpu_receipt <left-source-revision> <right-source-revision>".into());
    }
    // Unprivileged GPU receipt must not consult private StatiBaker history.
    let model=load_mixed_source_workspace(
        &args[0],&args[1],None,ContextVisibility::ExcludedByScope,"excluded",
    )?;
    let selected_ref=model.comparison.left_source_revision_ref.clone();
    let selected_id=model.graph.nodes.iter()
        .find(|node|node.semantic_ref==selected_ref)
        .ok_or_else(||"selected source absent from PNF graph".to_owned())?
        .id;
    if model.operational_graph.nodes.iter()
        .all(|node|node.id!=selected_id)
    {
        return Err("source semantic identity changed between lenses".into());
    }
    let instance=wgpu::Instance::default();
    let adapter=instance.request_adapter(&wgpu::RequestAdapterOptions::default())
        .await.map_err(|error|format!("request_adapter failed: {error}"))?;
    let adapter_info=adapter.get_info();
    let (device,queue)=adapter.request_device(&wgpu::DeviceDescriptor::default())
        .await.map_err(|error|format!("request_device failed: {error}"))?;
    let renderer=GraphRenderer::new(&device,wgpu::TextureFormat::Bgra8UnormSrgb);
    let semantic_plan=prepare_graph_gpu_plan(&model.graph)?;
    let operational_plan=prepare_graph_gpu_plan(&model.operational_graph)?;
    draw_panel(&device,&queue,&renderer,&semantic_plan,"pnf")?;
    draw_panel(&device,&queue,&renderer,&operational_plan,"operational")?;
    let semantic_pick=pick_object(
        &device,&queue,&renderer,&semantic_plan,selected_id,"pnf",
    )?;
    let operational_pick=pick_object(
        &device,&queue,&renderer,&operational_plan,selected_id,"operational",
    )?;
    let shell=reduce_selection(
        decode_shell(ShellInput::Select(selected_id)),
        SelectionState::default(),
    );
    if shell!=semantic_pick || shell!=operational_pick {
        return Err("mixed-source shell / both GPU lens pick parity failed".into());
    }
    if model.comparison.independent_witnesses_established.is_some()
        || model.creates_semantic_authority || model.creates_claim_truth
    {
        return Err("mixed-source UI receipt crossed evidentiary authority".into());
    }
    println!("carrier=typed-rust");
    println!("adapter_name={}",adapter_info.name);
    println!("adapter_backend={:?}",adapter_info.backend);
    println!("left_source_revision_ref={}",args[0]);
    println!("right_source_revision_ref={}",args[1]);
    println!("semantic_comparison={:?}",model.comparison.semantic_comparison);
    println!("genealogy={:?}",model.comparison.genealogy);
    println!("operational_visibility={:?}",model.comparison.operational_visibility);
    println!("semantic_graph_nodes={}",semantic_plan.node_vertices.len());
    println!("operational_graph_nodes={}",operational_plan.node_vertices.len());
    println!("real_wgpu_draw_submitted=true");
    println!("real_wgpu_r32uint_pick_readback=true");
    println!("source_selection_shell_both_gpu_lenses_identical=true");
    println!("independent_witness_count=not_established");
    println!("creates_semantic_authority=false");
    println!("claim_truth_promoted=false");
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
