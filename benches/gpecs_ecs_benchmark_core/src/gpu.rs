use std::{
    fs,
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
use gpecs_itertools::Itertools as _;
use wgpu::util::DeviceExt;

use crate::{
    setup::{create_entities_with_mixed_components, prepare_entities_with_mixed_components},
    statistics::StatisticsRecord,
};

#[expect(clippy::too_many_arguments)]
pub fn run<'context, B, E>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    context: &'context mut Context,
    entity_count: u32,
    repeat_count: Option<usize>,
    mut framebuffer: Framebuffer<B>,
    spawn_area_margin: u32,
    mut f: impl FnMut(u128, Duration, Vec<StatisticsRecord>, &Framebuffer<B>) -> Result<(), E>,
) -> Result<&'context mut Context, E>
where
    B: AsRef<[u32]> + AsMut<[u32]> + 'static,
{
    log::info!("> Running on GPU...");

    let mut rng = RandomXoshiro128::new(DEFAULT_SEED);
    log::info!(">> Creating {entity_count} entities with mixed components...");
    let entities = create_entities_with_mixed_components(context, entity_count);

    log::info!(">> Preparing entities with mixed components...");
    framebuffer.buffer_mut().as_mut().fill(NONE_SPRITE);
    prepare_entities_with_mixed_components(
        context,
        &mut rng,
        &entities,
        framebuffer.desc(),
        spawn_area_margin,
    );

    log::info!(">> Initializing GPU resources...");
    let mut executor = GpuExecutor::new(context, device.clone());

    executor
        .register_archetype_of::<(Position, Velocity, Data, Player, Health, Damage, Sprite)>()
        .expect("all the components should be unique");

    let mut time_delta = TimeDelta::default();
    let gpu_system_resources = create_gpu_system_resources(device, time_delta, &framebuffer);
    let gpu_system_additional_entries =
        create_gpu_systems_additional_entries(&gpu_system_resources);

    let framebuffer_download_buffer_desc = wgpu::BufferDescriptor {
        label: Some("`gpecs` `ecs_benchmark` framebuffer download buffer"),
        size: gpu_system_resources.framebuffer_data_storage.size(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    };
    let framebuffer_download_buffer = device.create_buffer(&framebuffer_download_buffer_desc);

    let framebuffer_clear_buffer_desc = wgpu::util::BufferInitDescriptor {
        label: Some("`gpecs` `ecs_benchmark` framebuffer clear buffer"),
        contents: bytemuck::must_cast_slice(framebuffer.buffer().as_ref()),
        usage: wgpu::BufferUsages::COPY_SRC,
    };
    let framebuffer_clear_buffer = device.create_buffer_init(&framebuffer_clear_buffer_desc);

    log::info!(">> Registering GPU systems...");
    let gpu_systems = register_gpu_systems(&mut executor);
    setup_gpu_systems(&mut executor, &gpu_systems, &gpu_system_additional_entries);

    log::info!(">> Running GPU systems...");
    for i in (0..).maybe_take(repeat_count) {
        framebuffer.buffer_mut().as_mut().fill(NONE_SPRITE);

        #[cfg(debug_assertions)]
        unsafe {
            device.start_graphics_debugger_capture();
        }

        let timestamp = Instant::now();

        let mut command_encoder = init_wgpu_command_encoder(device);
        command_encoder.copy_buffer_to_buffer(
            &framebuffer_clear_buffer,
            0,
            &gpu_system_resources.framebuffer_data_storage,
            0,
            framebuffer_clear_buffer.size(),
        );

        executor.execute(&mut command_encoder);

        command_encoder.copy_buffer_to_buffer(
            &gpu_system_resources.framebuffer_data_storage,
            0,
            &framebuffer_download_buffer,
            0,
            framebuffer_download_buffer.size(),
        );
        command_encoder.map_buffer_on_submit(
            &framebuffer_download_buffer,
            wgpu::MapMode::Read,
            ..,
            |_| {},
        );

        let command_buffer = command_encoder.finish();
        let submission_index = queue.submit([command_buffer]);

        let poll_type = wgpu::PollType::Wait {
            submission_index: Some(submission_index),
            timeout: None,
        };
        device
            .poll(poll_type)
            .expect("device should be polled successfully");

        let elapsed = timestamp.elapsed();

        #[cfg(debug_assertions)]
        unsafe {
            device.stop_graphics_debugger_capture();
        }

        time_delta = TimeDelta(elapsed.as_secs_f32());
        write_time_delta(time_delta, queue, &gpu_system_resources);

        download_framebuffer(&mut framebuffer, &framebuffer_download_buffer);
        framebuffer_download_buffer.unmap();

        let statistics = collect_statistics(&executor, queue);
        f(i, elapsed, statistics, &framebuffer)?;
    }

    Ok(executor.into_context(queue))
}

#[derive(Debug)]
struct GpuSystemResources {
    time_delta_uniform: wgpu::Buffer,
    framebuffer_desc_uniform: wgpu::Buffer,
    framebuffer_data_storage: wgpu::Buffer,
}

fn create_gpu_system_resources(
    device: &wgpu::Device,
    time_delta: TimeDelta,
    framebuffer: &Framebuffer<impl AsRef<[u32]>>,
) -> GpuSystemResources {
    let time_delta_slice = [time_delta.0];
    let time_delta_uniform_buffer_desc = wgpu::util::BufferInitDescriptor {
        label: Some("`gpecs` `ecs_benchmark` time delta uniform buffer"),
        contents: bytemuck::must_cast_slice(&time_delta_slice),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    };
    let time_delta_uniform = device.create_buffer_init(&time_delta_uniform_buffer_desc);

    let framebuffer_data_storage_buffer_desc = wgpu::util::BufferInitDescriptor {
        label: Some("`gpecs` `ecs_benchmark` framebuffer data storage buffer"),
        contents: bytemuck::must_cast_slice(framebuffer.buffer().as_ref()),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
    };
    let framebuffer_data_storage = device.create_buffer_init(&framebuffer_data_storage_buffer_desc);

    let framebuffer_desc = [framebuffer.desc()];
    let framebuffer_desc_uniform_buffer_desc = wgpu::util::BufferInitDescriptor {
        label: Some("`gpecs` `ecs_benchmark` framebuffer desc uniform buffer"),
        contents: bytemuck::must_cast_slice(&framebuffer_desc),
        usage: wgpu::BufferUsages::UNIFORM,
    };
    let framebuffer_desc_uniform = device.create_buffer_init(&framebuffer_desc_uniform_buffer_desc);

    GpuSystemResources {
        time_delta_uniform,
        framebuffer_desc_uniform,
        framebuffer_data_storage,
    }
}

#[derive(Debug)]
struct GpuSystemAdditionalEntries<'a> {
    update_position: [wgpu::BindGroupEntry<'a>; 1],
    update_data: [wgpu::BindGroupEntry<'a>; 1],
    render_sprite: [wgpu::BindGroupEntry<'a>; 2],
}

fn create_gpu_systems_additional_entries(
    resources: &GpuSystemResources,
) -> GpuSystemAdditionalEntries<'_> {
    let time_delta_uniform_buffer_entry = wgpu::BindGroupEntry {
        binding: 2,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &resources.time_delta_uniform,
            offset: 0,
            size: None,
        }),
    };
    let framebuffer_data_entry = wgpu::BindGroupEntry {
        binding: 2,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &resources.framebuffer_data_storage,
            offset: 0,
            size: None,
        }),
    };
    let framebuffer_desc_entry = wgpu::BindGroupEntry {
        binding: 3,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &resources.framebuffer_desc_uniform,
            offset: 0,
            size: None,
        }),
    };

    let update_position = [time_delta_uniform_buffer_entry.clone()];
    let update_data = [wgpu::BindGroupEntry {
        binding: 1,
        ..time_delta_uniform_buffer_entry
    }];
    let render_sprite = [framebuffer_data_entry, framebuffer_desc_entry];

    GpuSystemAdditionalEntries {
        update_position,
        update_data,
        render_sprite,
    }
}

#[derive(Debug, Clone, Copy)]
#[expect(unused)]
struct GpuSystems {
    update_position: GpuSystemId,
    update_data: GpuSystemId,
    update_components: GpuSystemId,
    update_health: GpuSystemId,
    update_damage: GpuSystemId,
    update_sprite: GpuSystemId,
    render_sprite: GpuSystemId,
}

#[expect(clippy::too_many_lines)]
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
        label: Some("update_position"),
        shader_module: shader_module.clone(),
        entry_point: Some("update_position"),
        dispatch_strategy: DispatchStrategy::default(),
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
        label: Some("update_data"),
        shader_module: shader_module.clone(),
        entry_point: Some("update_data"),
        dispatch_strategy: DispatchStrategy::default(),
        bind_entities: false,
        bind_components: [(data_id, GpuComponentAccess::ReadWrite)],
        additional_bindings: [time_delta_uniform_buffer_entry],
    };
    let update_data_system = executor
        .register_system(update_data_system_descriptor)
        .expect("archetype components should be unique");
    executor.add_system(update_data_system);

    let update_components_system_descriptor = GpuSystemDescriptor {
        label: Some("update_components"),
        shader_module: shader_module.clone(),
        entry_point: Some("update_components"),
        dispatch_strategy: DispatchStrategy::default(),
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
        label: Some("update_health"),
        shader_module: shader_module.clone(),
        entry_point: Some("update_health"),
        dispatch_strategy: DispatchStrategy::default(),
        bind_entities: false,
        bind_components: [(health_id, GpuComponentAccess::ReadWrite)],
        additional_bindings: [],
    };
    let update_health_system = executor
        .register_system(update_health_system_descriptor)
        .expect("archetype components should be unique");
    executor.add_system(update_health_system);

    let update_damage_system_descriptor = GpuSystemDescriptor {
        label: Some("update_damage"),
        shader_module: shader_module.clone(),
        entry_point: Some("update_damage"),
        dispatch_strategy: DispatchStrategy::default(),
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
        label: Some("update_sprite"),
        shader_module: shader_module.clone(),
        entry_point: Some("update_sprite"),
        dispatch_strategy: DispatchStrategy::default(),
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
        label: Some("render_sprite"),
        shader_module,
        entry_point: Some("render_sprite"),
        dispatch_strategy: DispatchStrategy::default(),
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
        update_position: update_position_system,
        update_data: update_data_system,
        update_components: update_components_system,
        update_health: update_health_system,
        update_damage: update_damage_system,
        update_sprite: update_sprite_system,
        render_sprite: render_sprite_system,
    }
}

fn setup_gpu_systems<'entries>(
    executor: &mut GpuExecutor<'_, 'entries>,
    systems: &GpuSystems,
    additional_entries: &'entries GpuSystemAdditionalEntries<'_>,
) {
    executor.set_additional_entries(systems.update_position, &additional_entries.update_position);
    executor.set_additional_entries(systems.update_data, &additional_entries.update_data);
    executor.set_additional_entries(systems.render_sprite, &additional_entries.render_sprite);
}

fn collect_statistics(executor: &GpuExecutor, queue: &wgpu::Queue) -> Vec<StatisticsRecord> {
    let statistics = executor
        .timestamp_query_statistics(queue)
        .expect("timestamp queries should be enabled")
        .expect("timestamp query statistics should be ready");

    statistics
        .iter()
        .flat_map(|(system, statistics)| {
            let Some(system_shader) = executor.systems().get_system_shader(system) else {
                unreachable!("{system} should exist")
            };

            let label = system_shader.label().expect("GPU system should be labeled");
            let mut statistics: Vec<_> = statistics
                .iter()
                .map(|(archetype, statistics)| StatisticsRecord {
                    system: system.into(),
                    name: label.to_owned().into(),
                    archetype: archetype.into(),
                    elapsed: statistics.duration,
                })
                .collect();
            statistics.sort();
            statistics
        })
        .collect()
}

fn write_time_delta(time_delta: TimeDelta, queue: &wgpu::Queue, resources: &GpuSystemResources) {
    let data = bytemuck::bytes_of(&time_delta);
    queue.write_buffer(&resources.time_delta_uniform, 0, data);
}

fn download_framebuffer<B>(framebuffer: &mut Framebuffer<B>, download_buffer: &wgpu::Buffer)
where
    B: AsMut<[u32]>,
{
    let framebuffer_view = download_buffer.get_mapped_range(..);
    let src = bytemuck::cast_slice(&framebuffer_view);
    let dst = framebuffer.buffer_mut().as_mut();
    dst.copy_from_slice(src);
}

pub fn init_wgpu() -> (wgpu::Device, wgpu::Queue) {
    let instance_desc = wgpu::InstanceDescriptor::new_without_display_handle();
    let instance = wgpu::Instance::new(instance_desc);

    let adapter_options = wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::from_env()
            .unwrap_or(wgpu::PowerPreference::HighPerformance),
        ..Default::default()
    };
    let adapter = pollster::block_on(instance.request_adapter(&adapter_options))
        .expect("failed to create adapter");
    log::debug!("Running on:\n{:#?}", adapter.get_info());
    log::debug!("Adapter features:\n{:#?}", adapter.features());

    let downlevel_capabilities = adapter.get_downlevel_capabilities();
    assert!(
        downlevel_capabilities
            .flags
            .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS),
        "adapter does not support compute shaders, which are required",
    );

    let features = adapter.features();
    assert!(
        features.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES),
        "adapter does not support timestamp queries inside passes, which are required",
    );
    assert!(
        adapter
            .features()
            .contains(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS),
        "adapter does not support mappable primary buffers, whic are required",
    );

    let device_desc = wgpu::DeviceDescriptor {
        label: Some("`gpecs` `ecs_benchmark` device"),
        required_features: wgpu::Features::TIMESTAMP_QUERY
            | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES
            | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
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

fn init_wgpu_command_encoder(device: &wgpu::Device) -> wgpu::CommandEncoder {
    let command_encoder_desc = wgpu::CommandEncoderDescriptor {
        label: Some("`gpecs` `ecs_benchmark` command encoder"),
    };
    device.create_command_encoder(&command_encoder_desc)
}
