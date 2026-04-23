use std::{
    ffi::c_void,
    fs,
    mem::transmute,
    ptr::null,
    time::{Duration, Instant},
};

use gpecs::prelude::*;
use gpecs_ecs_benchmark_types::{
    components::{
        DEFAULT_SEED, Damage, Data, Health, NONE_SPRITE, Player, Position, Sprite, Velocity,
    },
    framebuffer::{Framebuffer, FramebufferDesc},
    utils::{RandomXoshiro128, TimeDelta},
};
use itertools::Itertools;
use renderdoc::{RenderDoc, V141};
use wgpu::util::DeviceExt;

use crate::{
    ENTITY_COUNT, EXEC_COUNT, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_SIZE, FRAMEBUFFER_WIDTH, GPU_PATH,
    save::save_framebuffer_to_file,
    setup::{create_entities_with_mixed_components, prepare_entities_with_mixed_components},
};

pub fn run(context: &mut Context) {
    log::info!("> Running on GPU...");

    let mut rng = RandomXoshiro128::new(DEFAULT_SEED);
    log::info!(">> Creating {ENTITY_COUNT} entities with mixed components...");
    let entities = create_entities_with_mixed_components(context, ENTITY_COUNT);

    log::info!(">> Preparing entities with mixed components...");
    prepare_entities_with_mixed_components(context, &mut rng, &entities);

    let mut time_delta = TimeDelta::default();
    let mut framebuffer = Framebuffer::new(
        FRAMEBUFFER_WIDTH as u32,
        FRAMEBUFFER_HEIGHT as u32,
        vec![NONE_SPRITE; FRAMEBUFFER_SIZE],
    );

    log::info!(">> Initializing GPU resources...");
    let (device, queue) = init_wgpu();
    let mut renderdoc = init_renderdoc();

    let mut executor = GpuExecutor::new(context, device.clone());
    executor
        .register_archetype_of::<(Position, Velocity, Data, Player, Health, Damage, Sprite)>()
        .expect("all the components should be unique");

    let time_delta_slice = [time_delta.0];
    let time_delta_uniform_buffer_desc = wgpu::util::BufferInitDescriptor {
        label: Some("`gpecs` `ecs_benchmark` time delta uniform buffer"),
        contents: bytemuck::cast_slice(&time_delta_slice),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    };
    let time_delta_uniform_buffer = device.create_buffer_init(&time_delta_uniform_buffer_desc);

    let framebuffer_data_storage_buffer_desc = wgpu::util::BufferInitDescriptor {
        label: Some("`gpecs` `ecs_benchmark` framebuffer data storage buffer"),
        contents: bytemuck::cast_slice(framebuffer.buffer()),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    };
    let framebuffer_data_storage_buffer =
        device.create_buffer_init(&framebuffer_data_storage_buffer_desc);

    let framebuffer_desc = [framebuffer.desc()];
    let framebuffer_desc_uniform_buffer_desc = wgpu::util::BufferInitDescriptor {
        label: Some("`gpecs` `ecs_benchmark` framebuffer desc uniform buffer"),
        contents: bytemuck::cast_slice(&framebuffer_desc),
        usage: wgpu::BufferUsages::UNIFORM,
    };
    let framebuffer_desc_uniform_buffer =
        device.create_buffer_init(&framebuffer_desc_uniform_buffer_desc);

    let framebuffer_download_buffer_desc = wgpu::BufferDescriptor {
        label: Some("`gpecs` `ecs_benchmark` framebuffer download buffer"),
        size: framebuffer_data_storage_buffer.size(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    };
    let framebuffer_download_buffer = device.create_buffer(&framebuffer_download_buffer_desc);

    let mut timestamp_query_download_buffer = None;

    log::info!(">> Registering GPU systems...");
    let gpu_systems = register_gpu_systems(&mut executor);
    setup_gpu_systems(
        &mut executor,
        &gpu_systems,
        &time_delta_uniform_buffer,
        &framebuffer_data_storage_buffer,
        &framebuffer_desc_uniform_buffer,
    );

    log::info!(">> Running GPU systems...");
    for i in 0..EXEC_COUNT {
        renderdoc_start_frame_capture(renderdoc.as_mut(), &device);
        let timestamp = Instant::now();

        let mut command_encoder = init_wgpu_command_encoder(&device);
        executor.execute(&mut command_encoder);

        if timestamp_query_download_buffer.is_none() {
            timestamp_query_download_buffer = init_wgpu_timestamp_query_download_buffer(&executor);
        }

        wgpu_copy_into_timestamp_query_download_buffer(
            &executor,
            timestamp_query_download_buffer.as_ref(),
            &mut command_encoder,
        );

        command_encoder.copy_buffer_to_buffer(
            &framebuffer_data_storage_buffer,
            0,
            &framebuffer_download_buffer,
            0,
            framebuffer_data_storage_buffer.size(),
        );

        let command_buffer = command_encoder.finish();
        let submission_index = queue.submit([command_buffer]);

        let timestamp_query_download_slice =
            wgpu_map_whole_buffer(timestamp_query_download_buffer.as_ref());

        let framebuffer_data = framebuffer_download_buffer.slice(..);
        framebuffer_data.map_async(wgpu::MapMode::Read, |_| {});

        let poll_type = wgpu::PollType::Wait {
            submission_index: Some(submission_index),
            timeout: None,
        };
        device.poll(poll_type).expect("device should poll");

        let elapsed = timestamp.elapsed();
        renderdoc_end_frame_capture(renderdoc.as_mut(), &device);

        if let Some(timestamp_query_download_slice) = timestamp_query_download_slice {
            let timestamp_query_view = timestamp_query_download_slice.get_mapped_range();
            let timestamp_query_raw: &[u64] = bytemuck::cast_slice(&*timestamp_query_view);
            let timestamp_period_nanos = queue.get_timestamp_period();
            let mut timestamp_query_result =
                timestamp_query_raw
                    .iter()
                    .tuple_windows()
                    .map(|(first, second)| {
                        let nanos = (second - first) as f32 * timestamp_period_nanos;
                        Duration::from_nanos(nanos as u64)
                    });

            let update_position: Duration = timestamp_query_result.by_ref().take(3).sum();
            log::info!(">>>> `update_position` system took {update_position:?}");

            let update_data: Duration = timestamp_query_result.by_ref().skip(1).take(3).sum();
            log::info!(">>>> `update_data` system took {update_data:?}");

            let update_components: Duration = timestamp_query_result.by_ref().skip(1).take(2).sum();
            log::info!(">>>> `update_components` system took {update_components:?}");

            let update_health: Duration = timestamp_query_result.by_ref().skip(1).take(4).sum();
            log::info!(">>>> `update_health` system took {update_health:?}");

            let update_damage: Duration = timestamp_query_result.by_ref().skip(1).take(4).sum();
            log::info!(">>>> `update_damage` system took {update_damage:?}");

            let update_sprite: Duration = timestamp_query_result.by_ref().skip(1).take(4).sum();
            log::info!(">>>> `update_sprite` system took {update_sprite:?}");

            let render_sprite: Duration = timestamp_query_result.skip(1).sum();
            log::info!(">>>> `render_sprite` system took {render_sprite:?}");
        }
        if let Some(timestamp_query_download_buffer) = timestamp_query_download_buffer.as_ref() {
            timestamp_query_download_buffer.unmap();
        }
        log::info!(">>! Execution of GPU systems {i} took {elapsed:?}");

        time_delta = TimeDelta(elapsed.as_secs_f32());
        let time_delta_slice = [time_delta.0];
        queue.write_buffer(
            &time_delta_uniform_buffer,
            0,
            bytemuck::cast_slice(&time_delta_slice),
        );

        let framebuffer_view = framebuffer_data.get_mapped_range();
        let framebuffer_data = bytemuck::cast_slice(&*framebuffer_view);
        framebuffer.buffer_mut().copy_from_slice(framebuffer_data);

        drop(framebuffer_view);
        framebuffer_download_buffer.unmap();

        log::info!(">>> Saving framebuffer state {i} to file...");
        save_framebuffer_to_file(&framebuffer, GPU_PATH, i);
    }

    let context = executor.into_context(&queue);
    context.destroy_all();
}

#[derive(Debug, Clone, Copy)]
#[expect(unused)]
struct GpuSystems {
    update_position_system: GpuSystemId,
    update_data_system: GpuSystemId,
    update_components_system: GpuSystemId,
    update_health_system: GpuSystemId,
    update_damage_system: GpuSystemId,
    update_sprite_system: GpuSystemId,
    render_sprite_system: GpuSystemId,
}

fn register_gpu_systems(executor: &mut GpuExecutor) -> GpuSystems {
    let shader_module = init_wgpu_shader(executor.device());

    let position_id = executor.register_component::<Position>();
    let velocity_id = executor.register_component::<Velocity>();
    let data_id = executor.register_component::<Data>();
    let health_id = executor.register_component::<Health>();
    let damage_id = executor.register_component::<Damage>();
    let sprite_id = executor.register_component::<Sprite>();
    let player_id = executor.register_component::<Player>();

    let time_delta_uniform_buffer_entry = wgpu::BindGroupLayoutEntry {
        binding: 2,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(
                u64::try_from(size_of::<TimeDelta>())
                    .expect("size of `TimeDelta` should fit in `u64`")
                    .try_into()
                    .expect("size of `TimeDelta` cannot be zero"),
            ),
        },
        count: None,
    };
    let update_position_system_descriptor = GpuSystemDescriptor {
        label: Some("update position"),
        shader_module: shader_module.clone(),
        entry_point: Some("update_position"),
        workgroup_size: 64.try_into().ok(),
        bind_entities: false,
        bind_components: [
            (position_id, GpuComponentAccess::ReadWrite),
            (velocity_id, GpuComponentAccess::ReadOnly),
        ],
        additional_bindings: [time_delta_uniform_buffer_entry],
    };
    let update_position_system = executor
        .register_system(update_position_system_descriptor)
        .expect("archetype components should be unique");
    executor.add_system(update_position_system);

    let time_delta_uniform_buffer_entry = wgpu::BindGroupLayoutEntry {
        binding: 1,
        ..time_delta_uniform_buffer_entry
    };
    let update_data_system_descriptor = GpuSystemDescriptor {
        label: Some("update data"),
        shader_module: shader_module.clone(),
        entry_point: Some("update_data"),
        workgroup_size: 64.try_into().ok(),
        bind_entities: false,
        bind_components: [(data_id, GpuComponentAccess::ReadWrite)],
        additional_bindings: [time_delta_uniform_buffer_entry],
    };
    let update_data_system = executor
        .register_system(update_data_system_descriptor)
        .expect("archetype components should be unique");
    executor.add_system(update_data_system);

    let update_components_system_descriptor = GpuSystemDescriptor {
        label: Some("update components"),
        shader_module: shader_module.clone(),
        entry_point: Some("update_components"),
        workgroup_size: 64.try_into().ok(),
        bind_entities: false,
        bind_components: [
            (position_id, GpuComponentAccess::ReadOnly),
            (velocity_id, GpuComponentAccess::ReadWrite),
            (data_id, GpuComponentAccess::ReadWrite),
        ],
        additional_bindings: [],
    };
    let update_components_system = executor
        .register_system(update_components_system_descriptor)
        .expect("archetype components should be unique");
    executor.add_system(update_components_system);

    let update_health_system_descriptor = GpuSystemDescriptor {
        label: Some("update health"),
        shader_module: shader_module.clone(),
        entry_point: Some("update_health"),
        workgroup_size: 64.try_into().ok(),
        bind_entities: false,
        bind_components: [(health_id, GpuComponentAccess::ReadWrite)],
        additional_bindings: [],
    };
    let update_health_system = executor
        .register_system(update_health_system_descriptor)
        .expect("archetype components should be unique");
    executor.add_system(update_health_system);

    let update_damage_system_descriptor = GpuSystemDescriptor {
        label: Some("update damage"),
        shader_module: shader_module.clone(),
        entry_point: Some("update_damage"),
        workgroup_size: 64.try_into().ok(),
        bind_entities: false,
        bind_components: [
            (health_id, GpuComponentAccess::ReadWrite),
            (damage_id, GpuComponentAccess::ReadOnly),
        ],
        additional_bindings: [],
    };
    let update_damage_system = executor
        .register_system(update_damage_system_descriptor)
        .expect("archetype components should be unique");
    executor.add_system(update_damage_system);

    let update_sprite_system_descriptor = GpuSystemDescriptor {
        label: Some("update sprite"),
        shader_module: shader_module.clone(),
        entry_point: Some("update_sprite"),
        workgroup_size: 64.try_into().ok(),
        bind_entities: false,
        bind_components: [
            (sprite_id, GpuComponentAccess::ReadWrite),
            (player_id, GpuComponentAccess::ReadOnly),
            (health_id, GpuComponentAccess::ReadOnly),
        ],
        additional_bindings: [],
    };
    let update_sprite_system = executor
        .register_system(update_sprite_system_descriptor)
        .expect("archetype components should be unique");
    executor.add_system(update_sprite_system);

    let framebuffer_data_entry = wgpu::BindGroupLayoutEntry {
        binding: 2,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: Some(
                u64::try_from(size_of::<u32>())
                    .expect("size of `u32` should fit in `u64`")
                    .try_into()
                    .expect("size of `u32` cannot be zero"),
            ),
        },
        count: None,
    };
    let framebuffer_desc_entry = wgpu::BindGroupLayoutEntry {
        binding: 3,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(
                u64::try_from(size_of::<FramebufferDesc>())
                    .expect("size of `FramebufferDesc` should fit in `u64`")
                    .try_into()
                    .expect("size of `FramebufferDesc` cannot be zero"),
            ),
        },
        count: None,
    };
    let render_sprite_system_descriptor = GpuSystemDescriptor {
        label: Some("render sprite"),
        shader_module,
        entry_point: Some("render_sprite"),
        workgroup_size: 64.try_into().ok(),
        bind_entities: false,
        bind_components: [
            (position_id, GpuComponentAccess::ReadOnly),
            (sprite_id, GpuComponentAccess::ReadOnly),
        ],
        additional_bindings: [framebuffer_data_entry, framebuffer_desc_entry],
    };
    let render_sprite_system = executor
        .register_system(render_sprite_system_descriptor)
        .expect("archetype components should be unique");
    executor.add_system(render_sprite_system);

    GpuSystems {
        update_position_system,
        update_data_system,
        update_components_system,
        update_health_system,
        update_damage_system,
        update_sprite_system,
        render_sprite_system,
    }
}

fn setup_gpu_systems(
    executor: &mut GpuExecutor,
    systems: &GpuSystems,
    time_delta_uniform_buffer: &wgpu::Buffer,
    framebuffer_data_storage_buffer: &wgpu::Buffer,
    framebuffer_desc_uniform_buffer: &wgpu::Buffer,
) {
    let time_delta_uniform_buffer_entry = wgpu::BindGroupEntry {
        binding: 2,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: time_delta_uniform_buffer,
            offset: 0,
            size: None,
        }),
    };
    let framebuffer_data_entry = wgpu::BindGroupEntry {
        binding: 2,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: framebuffer_data_storage_buffer,
            offset: 0,
            size: None,
        }),
    };
    let framebuffer_desc_entry = wgpu::BindGroupEntry {
        binding: 3,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: framebuffer_desc_uniform_buffer,
            offset: 0,
            size: None,
        }),
    };
    let update_position_system_entries = [time_delta_uniform_buffer_entry.clone()];
    let update_data_system_entries = [wgpu::BindGroupEntry {
        binding: 1,
        ..time_delta_uniform_buffer_entry
    }];
    let render_sprite_system_entries = [framebuffer_data_entry, framebuffer_desc_entry];
    executor.set_additional_bindings([
        (
            systems.update_position_system,
            update_position_system_entries.iter().cloned(),
        ),
        (
            systems.update_data_system,
            update_data_system_entries.iter().cloned(),
        ),
        (
            systems.render_sprite_system,
            render_sprite_system_entries.iter().cloned(),
        ),
    ]);
}

fn init_wgpu() -> (wgpu::Device, wgpu::Queue) {
    let instance_desc = wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    };
    let instance = wgpu::Instance::new(instance_desc);

    let adapter_options = wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    };
    let adapter = pollster::block_on(instance.request_adapter(&adapter_options))
        .expect("failed to create adapter");
    log::debug!("Running on:\n{:#?}", adapter.get_info());
    log::debug!("Adapter features:\n{:#?}", adapter.features());

    let downlevel_capabilities = adapter.get_downlevel_capabilities();
    if !downlevel_capabilities
        .flags
        .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
    {
        panic!("adapter does not support compute shaders, which are required");
    }

    let features = adapter.features();
    if !features.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES) {
        panic!("adapter does not support timestamp queries inside passes, which are required");
    }

    let device_desc = wgpu::DeviceDescriptor {
        label: Some("`gpecs` `ecs_benchmark` device"),
        required_features: wgpu::Features::TIMESTAMP_QUERY
            | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        required_limits: adapter.limits(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&device_desc))
        .expect("failed to create device & queue");
    log::debug!("Limits of the current device:\n{:#?}", device.limits());

    (device, queue)
}

fn init_wgpu_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    const PATH: &str = env!("gpecs_ecs_benchmark_shader.spv");
    log::debug!("Loading shader from {PATH}");

    let data = fs::read(PATH).expect("SPIR-V shader file should exist");
    let shader_desc = wgpu::ShaderModuleDescriptor {
        label: Some("`gpecs` `ecs_benchmark` shader"),
        source: wgpu::util::make_spirv(&data),
    };
    let shader_module = device.create_shader_module(shader_desc);
    let shader_compilation_info = pollster::block_on(shader_module.get_compilation_info());
    log::debug!("Shader compilation info:\n{shader_compilation_info:#?}");

    shader_module
}

fn init_wgpu_timestamp_query_download_buffer(executor: &GpuExecutor) -> Option<wgpu::Buffer> {
    let timestamp_query_resources = executor.timestamp_query_resources()?;
    let timestamp_query_download_buffer_desc = wgpu::BufferDescriptor {
        label: Some("`gpecs` timestamp query download buffer"),
        size: unsafe { timestamp_query_resources.resolve_buffer() }.size(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    };
    executor
        .device()
        .create_buffer(&timestamp_query_download_buffer_desc)
        .into()
}

fn wgpu_copy_into_timestamp_query_download_buffer(
    executor: &GpuExecutor,
    timestamp_query_download_buffer: Option<&wgpu::Buffer>,
    command_encoder: &mut wgpu::CommandEncoder,
) {
    let Some((timestamp_query_resources, timestamp_query_download_buffer)) = executor
        .timestamp_query_resources()
        .zip(timestamp_query_download_buffer)
    else {
        return;
    };

    command_encoder.copy_buffer_to_buffer(
        unsafe { timestamp_query_resources.resolve_buffer() },
        0,
        timestamp_query_download_buffer,
        0,
        timestamp_query_download_buffer.size(),
    );
}

fn wgpu_map_whole_buffer(buffer: Option<&wgpu::Buffer>) -> Option<wgpu::BufferSlice<'_>> {
    let slice = buffer?.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    slice.into()
}

fn init_wgpu_command_encoder(device: &wgpu::Device) -> wgpu::CommandEncoder {
    let command_encoder_desc = wgpu::CommandEncoderDescriptor {
        label: Some("`gpecs` `ecs_benchmark` command encoder"),
    };
    device.create_command_encoder(&command_encoder_desc)
}

fn init_renderdoc() -> Option<RenderDoc<V141>> {
    match RenderDoc::<V141>::new() {
        Ok(renderdoc) => {
            log::info!("RenderDoc version: {:?}", renderdoc.get_api_version());
            Some(renderdoc)
        }
        Err(error) => {
            log::warn!("{error}");
            None
        }
    }
}

fn wgpu_raw_device_window(device: &wgpu::Device) -> (*const c_void, *const c_void) {
    let device_raw = unsafe {
        device
            .as_hal::<wgpu::hal::api::Vulkan>()
            .map(|device| transmute(device.raw_device().handle()))
            .unwrap_or(null::<c_void>())
    };
    let window_raw = null::<c_void>();
    (device_raw, window_raw)
}

fn renderdoc_start_frame_capture(renderdoc: Option<&mut RenderDoc<V141>>, device: &wgpu::Device) {
    let Some(renderdoc) = renderdoc else {
        return;
    };

    log::info!("Starting RenderDoc capture...");
    let (device_raw, window_raw) = wgpu_raw_device_window(device);
    renderdoc.start_frame_capture(device_raw, window_raw);
}

fn renderdoc_end_frame_capture(renderdoc: Option<&mut RenderDoc<V141>>, device: &wgpu::Device) {
    let Some(renderdoc) = renderdoc else {
        return;
    };

    log::info!("Ending RenderDoc capture...");
    let (device_raw, window_raw) = wgpu_raw_device_window(device);
    renderdoc.end_frame_capture(device_raw, window_raw);
}
