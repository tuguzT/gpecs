use std::{
    fs,
    time::{Duration, Instant},
};

use gpecs::{executor::gpu::AdditionalEntries, prelude::*};
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
    if entity_count == 0 || repeat_count == Some(0) {
        return Ok(context);
    }

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
    let gpu_system_resources = GpuSystemsResources::new(device, time_delta, &framebuffer);

    let framebuffer_download_buffer_desc = wgpu::BufferDescriptor {
        label: Some("`gpecs` `ecs_benchmark` framebuffer download buffer"),
        size: gpu_system_resources.framebuffer_data.size(),
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
    setup_gpu_systems(&mut executor, &gpu_systems, &gpu_system_resources);

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
            &gpu_system_resources.framebuffer_data,
            0,
            framebuffer_clear_buffer.size(),
        );

        executor.execute(&mut command_encoder);

        command_encoder.copy_buffer_to_buffer(
            &gpu_system_resources.framebuffer_data,
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
struct GpuSystemsResources {
    time_delta: wgpu::Buffer,
    framebuffer_data: wgpu::Buffer,
    framebuffer_desc: wgpu::Buffer,
}

impl GpuSystemsResources {
    fn new(
        device: &wgpu::Device,
        time_delta: TimeDelta,
        framebuffer: &Framebuffer<impl AsRef<[u32]>>,
    ) -> Self {
        Self {
            time_delta: create_time_delta_buffer(device, time_delta),
            framebuffer_data: create_framebuffer_data_buffer(device, framebuffer),
            framebuffer_desc: create_framebuffer_desc_buffer(device, framebuffer.desc()),
        }
    }
}

fn create_time_delta_buffer(device: &wgpu::Device, time_delta: TimeDelta) -> wgpu::Buffer {
    let desc = wgpu::util::BufferInitDescriptor {
        label: Some("`gpecs` `ecs_benchmark` time delta uniform buffer"),
        contents: bytemuck::bytes_of(&time_delta),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    };
    device.create_buffer_init(&desc)
}

fn create_framebuffer_data_buffer(
    device: &wgpu::Device,
    framebuffer: &Framebuffer<impl AsRef<[u32]>>,
) -> wgpu::Buffer {
    let desc = wgpu::util::BufferInitDescriptor {
        label: Some("`gpecs` `ecs_benchmark` framebuffer data storage buffer"),
        contents: bytemuck::must_cast_slice(framebuffer.buffer().as_ref()),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
    };
    device.create_buffer_init(&desc)
}

fn create_framebuffer_desc_buffer(
    device: &wgpu::Device,
    framebuffer_desc: FramebufferDesc,
) -> wgpu::Buffer {
    let desc = wgpu::util::BufferInitDescriptor {
        label: Some("`gpecs` `ecs_benchmark` framebuffer desc uniform buffer"),
        contents: bytemuck::bytes_of(&framebuffer_desc),
        usage: wgpu::BufferUsages::UNIFORM,
    };
    device.create_buffer_init(&desc)
}

#[derive(Debug)]
enum GpuSystemsAdditionalEntries {
    UpdatePosition {
        time_delta: wgpu::Buffer,
    },
    UpdateData {
        time_delta: wgpu::Buffer,
    },
    RenderSprite {
        framebuffer_data: wgpu::Buffer,
        framebuffer_desc: wgpu::Buffer,
    },
}

impl GpuSystemsAdditionalEntries {
    fn update_position(resources: &GpuSystemsResources) -> Self {
        let GpuSystemsResources { time_delta, .. } = resources;

        let time_delta = time_delta.clone();
        Self::UpdatePosition { time_delta }
    }

    fn update_data(resources: &GpuSystemsResources) -> Self {
        let GpuSystemsResources { time_delta, .. } = resources;

        let time_delta = time_delta.clone();
        Self::UpdateData { time_delta }
    }

    fn render_sprite(resources: &GpuSystemsResources) -> Self {
        let GpuSystemsResources {
            framebuffer_data,
            framebuffer_desc,
            ..
        } = resources;

        Self::RenderSprite {
            framebuffer_data: framebuffer_data.clone(),
            framebuffer_desc: framebuffer_desc.clone(),
        }
    }
}

impl AdditionalEntries for GpuSystemsAdditionalEntries {
    type Output<'a>
        = GpuSystemsAdditionalEntriesOutput<'a>
    where
        Self: 'a;

    fn additional_entries(&self) -> Self::Output<'_> {
        match self {
            Self::UpdatePosition { time_delta } => {
                let time_delta_entry = update_position_time_delta_entry(time_delta);
                let entries = [time_delta_entry];
                GpuSystemsAdditionalEntriesOutput::UpdatePosition { entries }
            }
            Self::UpdateData { time_delta } => {
                let time_delta_entry = update_data_time_delta_entry(time_delta);
                let entries = [time_delta_entry];
                GpuSystemsAdditionalEntriesOutput::UpdateData { entries }
            }
            Self::RenderSprite {
                framebuffer_data,
                framebuffer_desc,
            } => {
                let framebuffer_data_entry = render_sprite_framebuffer_data_entry(framebuffer_data);
                let framebuffer_desc_entry = render_sprite_framebuffer_desc_entry(framebuffer_desc);
                let entries = [framebuffer_data_entry, framebuffer_desc_entry];
                GpuSystemsAdditionalEntriesOutput::RenderSprite { entries }
            }
        }
    }
}

#[derive(Debug)]
enum GpuSystemsAdditionalEntriesOutput<'a> {
    UpdatePosition {
        entries: [wgpu::BindGroupEntry<'a>; 1],
    },
    UpdateData {
        entries: [wgpu::BindGroupEntry<'a>; 1],
    },
    RenderSprite {
        entries: [wgpu::BindGroupEntry<'a>; 2],
    },
}

impl<'a> AsRef<[wgpu::BindGroupEntry<'a>]> for GpuSystemsAdditionalEntriesOutput<'a> {
    fn as_ref(&self) -> &[wgpu::BindGroupEntry<'a>] {
        match self {
            Self::UpdatePosition { entries } | Self::UpdateData { entries } => entries,
            Self::RenderSprite { entries } => entries,
        }
    }
}

const UPDATE_POSITION_TIME_DELTA_BINDING: u32 = 2;
const UPDATE_DATA_TIME_DELTA_BINDING: u32 = 1;
const RENDER_SPRITE_FRAMEBUFFER_DATA_BINDING: u32 = 2;
const RENDER_SPRITE_FRAMEBUFFER_DESC_BINDING: u32 = 3;

fn update_position_time_delta_entry(time_delta: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding: UPDATE_POSITION_TIME_DELTA_BINDING,
        resource: time_delta.as_entire_binding(),
    }
}

fn update_data_time_delta_entry(time_delta: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding: UPDATE_DATA_TIME_DELTA_BINDING,
        resource: time_delta.as_entire_binding(),
    }
}

fn render_sprite_framebuffer_data_entry(
    framebuffer_data: &wgpu::Buffer,
) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding: RENDER_SPRITE_FRAMEBUFFER_DATA_BINDING,
        resource: framebuffer_data.as_entire_binding(),
    }
}

fn render_sprite_framebuffer_desc_entry(
    framebuffer_desc: &wgpu::Buffer,
) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding: RENDER_SPRITE_FRAMEBUFFER_DESC_BINDING,
        resource: framebuffer_desc.as_entire_binding(),
    }
}

fn time_delta_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    let size_of_as_u64 = size_of::<TimeDelta>()
        .try_into()
        .expect("size of `TimeDelta` should fit in `u64`");
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(size_of_as_u64),
        },
        count: None,
    }
}

fn update_position_time_delta_layout_entry() -> wgpu::BindGroupLayoutEntry {
    time_delta_layout_entry(UPDATE_POSITION_TIME_DELTA_BINDING)
}

fn update_data_time_delta_layout_entry() -> wgpu::BindGroupLayoutEntry {
    time_delta_layout_entry(UPDATE_DATA_TIME_DELTA_BINDING)
}

fn framebuffer_data_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    let size_of_as_u64 = size_of::<u32>()
        .try_into()
        .expect("size of `u32` should fit in `u64`");
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(size_of_as_u64),
        },
        count: None,
    }
}

fn render_sprite_framebuffer_data_layout_entry() -> wgpu::BindGroupLayoutEntry {
    framebuffer_data_layout_entry(RENDER_SPRITE_FRAMEBUFFER_DATA_BINDING)
}

fn framebuffer_desc_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    let size_of_as_u64 = size_of::<FramebufferDesc>()
        .try_into()
        .expect("size of `FramebufferDesc` should fit in `u64`");
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(size_of_as_u64),
        },
        count: None,
    }
}

fn render_sprite_framebuffer_desc_layout_entry() -> wgpu::BindGroupLayoutEntry {
    framebuffer_desc_layout_entry(RENDER_SPRITE_FRAMEBUFFER_DESC_BINDING)
}

fn register_update_position_system<T>(
    executor: &mut GpuExecutor<T, impl Sized>,
    shader_module: wgpu::ShaderModule,
) -> GpuSystemId
where
    T: AsRef<Context> + AsMut<Context> + ?Sized,
{
    let position_id = executor.register_component::<Position>();
    let velocity_id = executor.register_component::<Velocity>();

    let descriptor = GpuSystemDescriptor {
        label: Some("update_position"),
        shader_module,
        entry_point: Some("update_position"),
        dispatch_strategy: DispatchStrategy::default(),
        bind_entities: false,
        bind_components: [
            (position_id, GpuComponentAccess::ReadWrite),
            (velocity_id, GpuComponentAccess::ReadOnly),
        ],
        additional_bindings: [update_position_time_delta_layout_entry()],
    };
    executor
        .register_system(descriptor)
        .expect("archetype components should be unique")
}

fn register_update_data_system<T>(
    executor: &mut GpuExecutor<T, impl Sized>,
    shader_module: wgpu::ShaderModule,
) -> GpuSystemId
where
    T: AsRef<Context> + AsMut<Context> + ?Sized,
{
    let data_id = executor.register_component::<Data>();

    let descriptor = GpuSystemDescriptor {
        label: Some("update_data"),
        shader_module,
        entry_point: Some("update_data"),
        dispatch_strategy: DispatchStrategy::default(),
        bind_entities: false,
        bind_components: [(data_id, GpuComponentAccess::ReadWrite)],
        additional_bindings: [update_data_time_delta_layout_entry()],
    };
    executor
        .register_system(descriptor)
        .expect("archetype components should be unique")
}

fn register_update_components_system<T>(
    executor: &mut GpuExecutor<T, impl Sized>,
    shader_module: wgpu::ShaderModule,
) -> GpuSystemId
where
    T: AsRef<Context> + AsMut<Context> + ?Sized,
{
    let position_id = executor.register_component::<Position>();
    let velocity_id = executor.register_component::<Velocity>();
    let data_id = executor.register_component::<Data>();

    let descriptor = GpuSystemDescriptor {
        label: Some("update_components"),
        shader_module,
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
    executor
        .register_system(descriptor)
        .expect("archetype components should be unique")
}

fn register_update_health_system<T>(
    executor: &mut GpuExecutor<T, impl Sized>,
    shader_module: wgpu::ShaderModule,
) -> GpuSystemId
where
    T: AsRef<Context> + AsMut<Context> + ?Sized,
{
    let health_id = executor.register_component::<Health>();

    let descriptor = GpuSystemDescriptor {
        label: Some("update_health"),
        shader_module,
        entry_point: Some("update_health"),
        dispatch_strategy: DispatchStrategy::default(),
        bind_entities: false,
        bind_components: [(health_id, GpuComponentAccess::ReadWrite)],
        additional_bindings: [],
    };
    executor
        .register_system(descriptor)
        .expect("archetype components should be unique")
}

fn register_update_damage_system<T>(
    executor: &mut GpuExecutor<T, impl Sized>,
    shader_module: wgpu::ShaderModule,
) -> GpuSystemId
where
    T: AsRef<Context> + AsMut<Context> + ?Sized,
{
    let health_id = executor.register_component::<Health>();
    let damage_id = executor.register_component::<Damage>();

    let descriptor = GpuSystemDescriptor {
        label: Some("update_damage"),
        shader_module,
        entry_point: Some("update_damage"),
        dispatch_strategy: DispatchStrategy::default(),
        bind_entities: false,
        bind_components: [
            (health_id, GpuComponentAccess::ReadWrite),
            (damage_id, GpuComponentAccess::ReadOnly),
        ],
        additional_bindings: [],
    };
    executor
        .register_system(descriptor)
        .expect("archetype components should be unique")
}

fn register_update_sprite_system<T>(
    executor: &mut GpuExecutor<T, impl Sized>,
    shader_module: wgpu::ShaderModule,
) -> GpuSystemId
where
    T: AsRef<Context> + AsMut<Context> + ?Sized,
{
    let sprite_id = executor.register_component::<Sprite>();
    let player_id = executor.register_component::<Player>();
    let health_id = executor.register_component::<Health>();

    let descriptor = GpuSystemDescriptor {
        label: Some("update_sprite"),
        shader_module,
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
    executor
        .register_system(descriptor)
        .expect("archetype components should be unique")
}

fn register_render_sprite_system<T>(
    executor: &mut GpuExecutor<T, impl Sized>,
    shader_module: wgpu::ShaderModule,
) -> GpuSystemId
where
    T: AsRef<Context> + AsMut<Context> + ?Sized,
{
    let position_id = executor.register_component::<Position>();
    let sprite_id = executor.register_component::<Sprite>();

    let descriptor = GpuSystemDescriptor {
        label: Some("render_sprite"),
        shader_module,
        entry_point: Some("render_sprite"),
        dispatch_strategy: DispatchStrategy::default(),
        bind_entities: false,
        bind_components: [
            (position_id, GpuComponentAccess::ReadOnly),
            (sprite_id, GpuComponentAccess::ReadOnly),
        ],
        additional_bindings: [
            render_sprite_framebuffer_data_layout_entry(),
            render_sprite_framebuffer_desc_layout_entry(),
        ],
    };
    executor
        .register_system(descriptor)
        .expect("archetype components should be unique")
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

fn register_gpu_systems<T>(executor: &mut GpuExecutor<T, impl Sized>) -> GpuSystems
where
    T: AsRef<Context> + AsMut<Context> + ?Sized,
{
    let shader_module = init_wgpu_shader(executor.device());

    let update_position = register_update_position_system(executor, shader_module.clone());
    executor.add_system(update_position);

    let update_data = register_update_data_system(executor, shader_module.clone());
    executor.add_system(update_data);

    let update_components = register_update_components_system(executor, shader_module.clone());
    executor.add_system(update_components);

    let update_health = register_update_health_system(executor, shader_module.clone());
    executor.add_system(update_health);

    let update_damage = register_update_damage_system(executor, shader_module.clone());
    executor.add_system(update_damage);

    let update_sprite = register_update_sprite_system(executor, shader_module.clone());
    executor.add_system(update_sprite);

    let render_sprite = register_render_sprite_system(executor, shader_module);
    executor.add_system(render_sprite);

    GpuSystems {
        update_position,
        update_data,
        update_components,
        update_health,
        update_damage,
        update_sprite,
        render_sprite,
    }
}

fn setup_gpu_systems<T>(
    executor: &mut GpuExecutor<T, GpuSystemsAdditionalEntries>,
    systems: &GpuSystems,
    resources: &GpuSystemsResources,
) where
    T: AsRef<Context> + ?Sized,
{
    executor.set_additional_entries(
        systems.update_position,
        GpuSystemsAdditionalEntries::update_position(resources),
    );
    executor.set_additional_entries(
        systems.update_data,
        GpuSystemsAdditionalEntries::update_data(resources),
    );
    executor.set_additional_entries(
        systems.render_sprite,
        GpuSystemsAdditionalEntries::render_sprite(resources),
    );
}

fn collect_statistics(
    executor: &GpuExecutor<impl ?Sized, impl Sized>,
    queue: &wgpu::Queue,
) -> Vec<StatisticsRecord> {
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

fn write_time_delta(time_delta: TimeDelta, queue: &wgpu::Queue, resources: &GpuSystemsResources) {
    let data = bytemuck::bytes_of(&time_delta);
    queue.write_buffer(&resources.time_delta, 0, data);
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
        memory_hints: wgpu::MemoryHints::Manual {
            // just use the minimal possible one
            suballocated_device_memory_block_size: 0..0,
        },
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
