use std::{
    f32::consts::{FRAC_PI_2, FRAC_PI_3},
    fs,
    time::{Duration, Instant},
};

use egui::{Rgba, RichText, Ui};
use glam::{EulerRot, Mat4, Quat, Vec2, Vec3, dvec2, vec3};
use gpecs_nbody_types::{CameraBuffer, Vertex};
use num_traits::ToPrimitive;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferAddress, BufferBindingType,
    BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoder, Device, FragmentState,
    FrontFace, LoadOp, MultisampleState, Operations, PipelineCompilationOptions, PipelineLayout,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModule, ShaderModuleDescriptor, ShaderStages, StoreOp, TextureFormat, TextureView,
    VertexBufferLayout, VertexState, VertexStepMode,
    util::{self, BufferInitDescriptor, DeviceExt, StagingBelt},
    vertex_attr_array,
};
use winit::{
    event::{DeviceEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools)]
pub struct State {
    start_time: Instant,
    last_update_time: Option<Instant>,
    total_time: Duration,
    delta_time: Duration,
    width: u32,
    height: u32,
    w_pressed: bool,
    a_pressed: bool,
    s_pressed: bool,
    d_pressed: bool,
    space_pressed: bool,
    shift_pressed: bool,
    mouse_left_pressed: bool,
    mouse_move_delta: Vec2,
    camera_position: Vec3,
    camera_rotation: Quat,
    vertex_buffer: Buffer,
    camera_buffer: Buffer,
    staging: StagingBelt,
    camera_bind_group: BindGroup,
    render_pipeline: RenderPipeline,
}

impl State {
    pub fn new(
        device: &Device,
        width: u32,
        height: u32,
        format: TextureFormat,
        start_time: Instant,
    ) -> Self {
        let shader_module = init_shader(device);

        let vertex_buffer = init_vertex_buffer(device);
        let camera_buffer = init_camera_buffer(device);
        let staging = StagingBelt::new(device.clone(), camera_buffer.size() * 4);

        let camera_layout = init_camera_bind_group_layout(device);
        let camera_bind_group = init_camera_bind_group(device, &camera_layout, &camera_buffer);

        let render_pipeline_layout = init_pipeline_layout(device, &camera_layout);
        let render_pipeline =
            init_pipeline(device, format, &shader_module, &render_pipeline_layout);

        Self {
            start_time,
            last_update_time: None,
            total_time: Duration::ZERO,
            delta_time: Duration::ZERO,
            width,
            height,
            w_pressed: false,
            a_pressed: false,
            s_pressed: false,
            d_pressed: false,
            space_pressed: false,
            shift_pressed: false,
            mouse_left_pressed: false,
            mouse_move_delta: Vec2::ZERO,
            camera_position: Vec3::NEG_Z,
            camera_rotation: Quat::from_axis_angle(Vec3::Z, 0.0),
            vertex_buffer,
            camera_buffer,
            staging,
            camera_bind_group,
            render_pipeline,
        }
    }

    pub fn handle_window_event(&mut self, window: &Window, event: &WindowEvent) {
        let Self {
            width,
            height,
            mouse_left_pressed,
            ..
        } = self;

        let _ = window;
        match event {
            WindowEvent::Resized(size) => {
                *width = size.width;
                *height = size.height;
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                *mouse_left_pressed = state.is_pressed();
            }
            _ => (),
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        let Self {
            w_pressed,
            a_pressed,
            s_pressed,
            d_pressed,
            space_pressed,
            shift_pressed,
            mouse_move_delta,
            ..
        } = self;

        match event {
            DeviceEvent::Key(event) => match event.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => *w_pressed = event.state.is_pressed(),
                PhysicalKey::Code(KeyCode::KeyA) => *a_pressed = event.state.is_pressed(),
                PhysicalKey::Code(KeyCode::KeyS) => *s_pressed = event.state.is_pressed(),
                PhysicalKey::Code(KeyCode::KeyD) => *d_pressed = event.state.is_pressed(),
                PhysicalKey::Code(KeyCode::Space) => *space_pressed = event.state.is_pressed(),
                PhysicalKey::Code(KeyCode::ShiftLeft) => *shift_pressed = event.state.is_pressed(),
                _ => (),
            },
            DeviceEvent::MouseMotion { delta } => {
                let (x, y) = *delta;
                *mouse_move_delta += dvec2(x, y).as_vec2();
            }
            _ => (),
        }
    }

    pub fn update(&mut self, ui: &mut Ui) {
        let Self {
            start_time,
            w_pressed,
            a_pressed,
            s_pressed,
            d_pressed,
            space_pressed,
            shift_pressed,
            mouse_left_pressed,
            ref mut last_update_time,
            ref mut total_time,
            ref mut delta_time,
            ref mut mouse_move_delta,
            ref mut camera_position,
            ref mut camera_rotation,
            ..
        } = *self;

        let now = Instant::now();
        let or = |earlier| now.duration_since(earlier);
        *delta_time = last_update_time.map_or(Duration::from_nanos(1), or);
        *total_time = now.duration_since(start_time);
        *last_update_time = Some(now);

        let dt_raw = delta_time.as_secs_f32();
        if mouse_left_pressed {
            let (mut yaw, mut pitch, _) = camera_rotation.to_euler(EulerRot::YXZ);

            yaw -= mouse_move_delta.x * dt_raw;
            pitch -= mouse_move_delta.y * dt_raw;

            let max_pitch = FRAC_PI_2 - f32::EPSILON;
            let min_pitch = -max_pitch;
            pitch = pitch.clamp(min_pitch, max_pitch);

            *camera_rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        }
        *mouse_move_delta = Vec2::ZERO;

        let forward = *camera_rotation * Vec3::Z;
        let right = *camera_rotation * Vec3::X;
        let up = right.cross(forward);
        *camera_position += bool_to_f32(w_pressed) * dt_raw * forward;
        *camera_position += bool_to_f32(a_pressed) * dt_raw * right;
        *camera_position += bool_to_f32(s_pressed) * dt_raw * -forward;
        *camera_position += bool_to_f32(d_pressed) * dt_raw * -right;
        *camera_position += bool_to_f32(space_pressed) * dt_raw * up;
        *camera_position += bool_to_f32(shift_pressed) * dt_raw * -up;

        let fps = 1.0 / dt_raw;
        let text = format!("Total time: {total_time:?}, delta time: {dt_raw}, FPS: {fps}");
        let color = Rgba::from_gray(1.0 - total_time.as_secs_f32().fract());
        ui.label(RichText::new(text).color(color));
    }

    pub fn draw(
        &mut self,
        _device: &Device,
        _queue: &Queue,
        render_target: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        let Self {
            total_time,
            width,
            height,
            camera_position,
            camera_rotation,
            ref vertex_buffer,
            ref camera_buffer,
            ref camera_bind_group,
            ref render_pipeline,
            ref mut staging,
            ..
        } = *self;

        staging.recall();

        let model = Mat4::from_translation(camera_position);
        let view = Mat4::from_quat(camera_rotation).inverse();
        let projection = Mat4::perspective_rh(FRAC_PI_3, aspect_ratio(width, height), 0.1, 1000.0);
        let data = CameraBuffer {
            model_view_projection: projection * view * model,
        };

        let camera_buffer_size = camera_buffer
            .size()
            .try_into()
            .expect("camera buffer can't be zero-sized");
        staging
            .write_buffer(encoder, camera_buffer, 0, camera_buffer_size)
            .copy_from_slice(bytemuck::bytes_of(&data));

        let gray = total_time.as_secs_f64().fract();
        let clear_color = Color {
            r: gray,
            g: gray,
            b: gray,
            a: 1.0,
        };
        let render_pass_desc = RenderPassDescriptor {
            label: Some("`gpecs` n-body simulation example clear render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: render_target,
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
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..3, 0..1);

        staging.finish();
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

fn init_camera_bind_group_layout(device: &Device) -> BindGroupLayout {
    let bind_group_layout_desc = BindGroupLayoutDescriptor {
        label: Some("`gpecs` n-body simulation example render camera bind group layout"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    };
    device.create_bind_group_layout(&bind_group_layout_desc)
}

fn init_pipeline_layout(device: &Device, camera_layout: &BindGroupLayout) -> PipelineLayout {
    let pipeline_layout_desc = PipelineLayoutDescriptor {
        label: Some("`gpecs` n-body simulation example render pipeline layout"),
        bind_group_layouts: &[Some(camera_layout)],
        immediate_size: 0,
    };
    device.create_pipeline_layout(&pipeline_layout_desc)
}

fn init_camera_bind_group(
    device: &Device,
    camera_layout: &BindGroupLayout,
    camera_buffer: &Buffer,
) -> BindGroup {
    let bind_group_desc = BindGroupDescriptor {
        label: Some("`gpecs` n-body simulation example render camera bind group"),
        layout: camera_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
    };
    device.create_bind_group(&bind_group_desc)
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

fn init_camera_buffer(device: &Device) -> Buffer {
    let data = CameraBuffer {
        model_view_projection: Mat4::IDENTITY,
    };
    let buffer_init_desc = BufferInitDescriptor {
        label: Some("`gpecs` n-body simulation example camera buffer"),
        contents: bytemuck::bytes_of(&data),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    };
    device.create_buffer_init(&buffer_init_desc)
}

fn aspect_ratio(width: u32, height: u32) -> f32 {
    width.to_f32().unwrap_or(1.0) / height.to_f32().unwrap_or(1.0)
}

fn bool_to_f32(bool: bool) -> f32 {
    if bool { 1.0 } else { 0.0 }
}
