use std::{
    f32::consts::{FRAC_PI_2, FRAC_PI_3},
    fmt::{self, Debug},
    fs,
    time::{Duration, Instant},
};

use egui::{Rgba, RichText, Ui};
use glam::{EulerRot, Mat4, Quat, Vec2, Vec3, dvec2, uvec4, vec3, vec4};
use gpecs::{context::Context, executor::gpu::GpuExecutor};
use gpecs_nbody_types::render::{UniformBuffer, Vertex};
use num_traits::ToPrimitive;
use ouroboros::self_referencing;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferAddress, BufferBindingType,
    BufferUsages, ColorTargetState, ColorWrites, CommandEncoder, Device, FragmentState, FrontFace,
    LoadOp, MultisampleState, Operations, PipelineCompilationOptions, PipelineLayout,
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
    uniform_buffer: Buffer,
    staging: StagingBelt,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
    ecs: EcsState,
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
        let uniform_buffer = init_uniform_buffer(device);
        let staging = StagingBelt::new(device.clone(), uniform_buffer.size() * 4);

        let bind_group_layout = init_bind_group_layout(device);
        let bind_group = init_bind_group(device, &bind_group_layout, &uniform_buffer);

        let render_pipeline_layout = init_pipeline_layout(device, &bind_group_layout);
        let render_pipeline =
            init_pipeline(device, format, &shader_module, &render_pipeline_layout);

        let ecs = init_ecs_state(device.clone());

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
            uniform_buffer,
            staging,
            bind_group,
            render_pipeline,
            ecs,
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

        ui.label(RichText::new(format!("Total time: {total_time:?}")).color(Rgba::WHITE));
        ui.label(RichText::new(format!("Delta time: {dt_raw}")).color(Rgba::WHITE));
        ui.label(RichText::new(format!("FPS: {}", 1.0 / dt_raw)).color(Rgba::WHITE));
    }

    pub fn draw(
        &mut self,
        _device: &Device,
        _queue: &Queue,
        render_target: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        let Self {
            width,
            height,
            camera_position,
            camera_rotation,
            ref vertex_buffer,
            ref uniform_buffer,
            ref bind_group,
            ref render_pipeline,
            ref mut staging,
            ref mut ecs,
            ..
        } = *self;

        staging.recall();

        let model = Mat4::from_translation(camera_position);
        let view = Mat4::from_quat(camera_rotation).inverse();
        let z_near = 0.001;
        let z_far = 1000.0;
        let projection =
            Mat4::perspective_rh(FRAC_PI_3, aspect_ratio(width, height), z_near, z_far);
        let data = UniformBuffer {
            model_view_projection: projection * view * model,
            resolution: uvec4(width, height, 0, 0).as_vec4(),
        };

        let uniform_buffer_size = uniform_buffer
            .size()
            .try_into()
            .expect("uniform buffer can't be zero-sized");
        staging
            .write_buffer(encoder, uniform_buffer, 0, uniform_buffer_size)
            .copy_from_slice(bytemuck::bytes_of(&data));

        ecs.with_executor_mut(|executor| executor.execute(encoder));

        let render_pass_desc = RenderPassDescriptor {
            label: Some("`gpecs` n-body simulation example clear render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: render_target,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::default(),
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
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..3);

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

fn init_bind_group_layout(device: &Device) -> BindGroupLayout {
    let bind_group_layout_desc = BindGroupLayoutDescriptor {
        label: Some("`gpecs` n-body simulation example render bind group layout"),
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

fn init_pipeline_layout(device: &Device, layout: &BindGroupLayout) -> PipelineLayout {
    let pipeline_layout_desc = PipelineLayoutDescriptor {
        label: Some("`gpecs` n-body simulation example render pipeline layout"),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    };
    device.create_pipeline_layout(&pipeline_layout_desc)
}

fn init_bind_group(device: &Device, layout: &BindGroupLayout, buffer: &Buffer) -> BindGroup {
    let bind_group_desc = BindGroupDescriptor {
        label: Some("`gpecs` n-body simulation example render bind group"),
        layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
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
                step_mode: VertexStepMode::Instance,
                attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32, 2 => Float32x3],
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
        size: 5.0,
        color: vec3(1.0, 0.0, 0.0),
        padding: 0,
    },
    Vertex {
        position: vec3(-0.5, -0.5, 0.0),
        size: 10.0,
        color: vec3(0.0, 1.0, 0.0),
        padding: 0,
    },
    Vertex {
        position: vec3(0.5, -0.5, 0.0),
        size: 15.0,
        color: vec3(0.0, 0.0, 1.0),
        padding: 0,
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

fn init_uniform_buffer(device: &Device) -> Buffer {
    let data = UniformBuffer {
        model_view_projection: Mat4::IDENTITY,
        resolution: vec4(1.0, 1.0, 0.0, 0.0),
    };
    let buffer_init_desc = BufferInitDescriptor {
        label: Some("`gpecs` n-body simulation example uniform buffer"),
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

#[self_referencing]
pub struct EcsState {
    context: Context,
    #[borrows(mut context)]
    #[covariant]
    executor: GpuExecutor<'this, 'static>,
}

impl Debug for EcsState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.with_executor(|executor| {
            f.debug_struct("EcsState")
                .field("executor", executor)
                .finish_non_exhaustive()
        })
    }
}

fn init_ecs_state(device: Device) -> EcsState {
    let builder = EcsStateBuilder {
        context: Context::new(),
        executor_builder: |context| GpuExecutor::new(context, device),
    };
    builder.build()
}
