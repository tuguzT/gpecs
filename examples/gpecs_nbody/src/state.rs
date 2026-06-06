use std::{
    fs,
    time::{Duration, Instant},
};

use egui::{Rgba, RichText, Ui};
use glam::vec3;
use gpecs_nbody_types::Vertex;
use wgpu::{
    BlendState, Buffer, BufferAddress, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoder, Device, FragmentState, FrontFace, LoadOp, MultisampleState, Operations,
    PipelineCompilationOptions, PipelineLayout, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, StoreOp,
    TextureFormat, TextureView, VertexBufferLayout, VertexState, VertexStepMode,
    util::{self, BufferInitDescriptor, DeviceExt},
    vertex_attr_array,
};

#[derive(Debug)]
pub struct State {
    start_time: Instant,
    last_update_time: Instant,
    total_time: Duration,
    delta_time: Duration,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
}

impl State {
    pub fn new(device: &Device, format: TextureFormat, start_time: Instant) -> Self {
        let shader_module = init_shader(device);
        let render_pipeline_layout = init_pipeline_layout(device);
        let render_pipeline =
            init_pipeline(device, format, &shader_module, &render_pipeline_layout);
        let vertex_buffer = init_vertex_buffer(device);

        Self {
            start_time,
            last_update_time: start_time,
            total_time: Duration::ZERO,
            delta_time: Duration::ZERO,
            render_pipeline,
            vertex_buffer,
        }
    }

    pub fn update(&mut self, ui: &mut Ui) {
        let Self {
            start_time,
            ref mut last_update_time,
            ref mut total_time,
            ref mut delta_time,
            ..
        } = *self;

        let now = Instant::now();
        *total_time = now.duration_since(start_time);
        *delta_time = now.duration_since(*last_update_time);
        *last_update_time = now;

        let dt_raw = delta_time.as_secs_f64();
        let fps = 1.0 / dt_raw;
        let text = format!("Total time: {total_time:?}, delta time: {dt_raw}, FPS: {fps}");
        let color = Rgba::from_gray(1.0 - total_time.as_secs_f32().fract());
        ui.label(RichText::new(text).color(color));
    }

    pub fn draw(
        &self,
        _device: &Device,
        _queue: &Queue,
        view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        let Self {
            total_time,
            render_pipeline,
            vertex_buffer,
            ..
        } = self;

        let clear_color = Color {
            r: total_time.as_secs_f64().fract(),
            g: total_time.as_secs_f64().fract(),
            b: total_time.as_secs_f64().fract(),
            a: 1.0,
        };
        let render_pass_desc = RenderPassDescriptor {
            label: Some("`gpecs` n-body simulation example clear render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(clear_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        };
        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..3, 0..1);
    }
}

fn init_shader(device: &Device) -> ShaderModule {
    const PATH: &str = env!("gpecs_nbody_shader.spv");
    log::info!("Loading shader from {PATH}...");

    let data = fs::read(PATH).expect("SPIR-V shader file should exist");
    let shader_desc = ShaderModuleDescriptor {
        label: Some("`gpecs` n-body simulation example shader"),
        source: util::make_spirv(&data),
    };
    device.create_shader_module(shader_desc)
}

fn init_pipeline_layout(device: &Device) -> PipelineLayout {
    let pipeline_layout_desc = PipelineLayoutDescriptor {
        label: Some("`gpecs` n-body simulation example render pipeline layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    };
    device.create_pipeline_layout(&pipeline_layout_desc)
}

fn init_pipeline(
    device: &Device,
    format: TextureFormat,
    shader_module: &ShaderModule,
    pipeline_layout: &PipelineLayout,
) -> RenderPipeline {
    let render_pipeline_desc = RenderPipelineDescriptor {
        label: Some("`gpecs` n-body simulation example render pipeline"),
        layout: Some(pipeline_layout),
        vertex: VertexState {
            module: shader_module,
            entry_point: Some("vertex"),
            compilation_options: PipelineCompilationOptions::default(),
            buffers: &[VertexBufferLayout {
                array_stride: size_of::<Vertex>() as BufferAddress,
                step_mode: VertexStepMode::Vertex,
                attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x3],
            }],
        },
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState::default(),
        fragment: Some(FragmentState {
            module: shader_module,
            entry_point: Some("fragment"),
            compilation_options: PipelineCompilationOptions::default(),
            targets: &[Some(ColorTargetState {
                format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    };
    device.create_render_pipeline(&render_pipeline_desc)
}

const TRIANGLE_VERTICES: [Vertex; 3] = [
    Vertex {
        position: vec3(0.0, 0.5, 0.0),
        color: vec3(1.0, 0.0, 0.0),
    },
    Vertex {
        position: vec3(-0.5, -0.5, 0.0),
        color: vec3(0.0, 1.0, 0.0),
    },
    Vertex {
        position: vec3(0.5, -0.5, 0.0),
        color: vec3(0.0, 0.0, 1.0),
    },
];

fn init_vertex_buffer(device: &Device) -> Buffer {
    let buffer_init_desc = BufferInitDescriptor {
        label: Some("`gpecs` n-body simulation example vertex buffer"),
        contents: bytemuck::must_cast_slice(&TRIANGLE_VERTICES),
        usage: BufferUsages::VERTEX,
    };
    device.create_buffer_init(&buffer_init_desc)
}
