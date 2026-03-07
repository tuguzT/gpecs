use std::{
    cell::RefCell,
    ffi::c_void,
    fs::{self, File},
    io::Write,
    mem::transmute,
    path::Path,
    ptr::null,
    rc::Rc,
    time::{Duration, Instant},
};

use gpecs::{context::error::IncompatibleBundleError, prelude::*};
use gpecs_ecs_benchmark_types::{
    components::{
        DEFAULT_SEED, Damage, Data, Health, NONE_SPRITE, Player, PlayerType, Position,
        SPAWN_SPRITE, Sprite, Velocity,
    },
    framebuffer::{Framebuffer, FramebufferDesc},
    systems::{
        render_sprite, update_components, update_damage, update_data, update_health,
        update_position, update_sprite,
    },
    utils::{RandomXoshiro128, TimeDelta},
};
use itertools::Itertools;
use renderdoc::{RenderDoc, V141};
use wgpu::util::DeviceExt;

const ENTITY_COUNT: usize = 1_000_000;
const EXEC_COUNT: usize = 10;

const CPU_PATH: &str = "cpu";
const GPU_PATH: &str = "gpu";

const FRAMEBUFFER_WIDTH: usize = 320;
const FRAMEBUFFER_HEIGHT: usize = 240;
const FRAMEBUFFER_SIZE: usize = FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT;
const SPAWN_AREA_MARGIN: u32 = 100;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let mut context = Context::new();
    run_cpu(&mut context);
    run_gpu(&mut context);
}

fn run_cpu(context: &mut Context) {
    log::info!("> Running on CPU...");

    let mut rng = RandomXoshiro128::new(DEFAULT_SEED);
    log::info!(">> Creating {ENTITY_COUNT} entities with mixed components...");
    let entities = create_entities_with_mixed_components(context, ENTITY_COUNT);

    log::info!(">> Preparing entities with mixed components...");
    prepare_entities_with_mixed_components(context, &mut rng, &entities);

    let time_delta = TimeDelta::default();
    let framebuffer = Framebuffer::new(
        FRAMEBUFFER_WIDTH as u32,
        FRAMEBUFFER_HEIGHT as u32,
        vec![NONE_SPRITE; FRAMEBUFFER_SIZE],
    );

    let mut executor = CpuExecutor::new(context);

    let time_delta = Rc::new(RefCell::new(time_delta));
    let framebuffer = Rc::new(RefCell::new(framebuffer));

    log::info!(">> Registering CPU systems...");
    register_cpu_systems(&mut executor, time_delta.clone(), framebuffer.clone());

    log::info!(">> Running CPU systems...");
    for i in 0..EXEC_COUNT {
        let timestamp = Instant::now();
        executor.execute();

        let elapsed = timestamp.elapsed();
        log::info!(">>! Execution of CPU systems {i} took {elapsed:?}");

        let time_delta = &mut *time_delta.borrow_mut();
        *time_delta = TimeDelta(elapsed.as_secs_f32());

        log::info!(">>> Saving framebuffer state {i} to file...");
        let framebuffer = &*framebuffer.borrow();
        save_framebuffer_to_file(framebuffer, CPU_PATH, i);
    }

    context.clear();
}

fn run_gpu(context: &mut Context) {
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
        .register_archetype::<(Position, Velocity, Data, Player, Health, Damage, Sprite)>()
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
    register_gpu_systems(
        &mut executor,
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
        queue.submit([command_buffer]);

        let timestamp_query_download_slice =
            wgpu_map_whole_buffer(timestamp_query_download_buffer.as_ref());

        let framebuffer_data = framebuffer_download_buffer.slice(..);
        framebuffer_data.map_async(wgpu::MapMode::Read, |_| {});

        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("device should poll");

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

    context.clear();
}

fn create_entities_with_mixed_components(context: &mut Context, count: usize) -> Vec<Entity> {
    let mut entities = Vec::with_capacity(count);
    context
        .register_archetype::<(Position, Velocity, Data, Player, Health, Damage, Sprite)>()
        .expect("all the components should be unique");

    let mut j = 0;
    for i in 0..count {
        let entity = create_entity(context);
        entities.push(entity);

        if count < 100 || (i >= 2 * count / 4 && i <= 3 * count / 4) {
            if count < 100 || (j % 10) == 0 {
                if (i % 7) == 0 {
                    remove_component_one(context, entity);
                }
                if (i % 11) == 0 {
                    remove_component_two(context, entity);
                }
                if (i % 13) == 0 {
                    remove_component_three(context, entity);
                }

                // if (i % 17) == 0 {
                //     context.despawn(entity);
                // }
            }
            j += 1;
        }
    }
    entities
}

fn prepare_entities_with_mixed_components(
    context: &mut Context,
    rng: &mut RandomXoshiro128,
    entities: &[Entity],
) {
    let mut j = 0;
    let count = entities.len();
    for (i, entity) in entities.iter().copied().enumerate() {
        if (count < 100 && i == 0) || count >= 100 || i >= count / 8 {
            if (count < 100 && i == 0) || count >= 100 || (j % 2) == 0 {
                if i == 0 {
                    add_components(context, entity);
                    init_components(context, entity, rng, Some(PlayerType::Hero));
                } else if (i % 6) == 0 {
                    add_components(context, entity);
                    init_components(context, entity, rng, None);
                } else if (i % 4) == 0 {
                    add_components(context, entity);
                    init_components(context, entity, rng, Some(PlayerType::Hero));
                } else if (i % 2) == 0 {
                    add_components(context, entity);
                    init_components(context, entity, rng, Some(PlayerType::Monster));
                }
            }
            j += 1;
        }
    }
}

fn create_entity(context: &mut Context) -> Entity {
    let entity = context.spawn();

    let position = Position::default();
    let velocity = Velocity::default();
    let data = Data {
        rng: RandomXoshiro128::new(0),
        thingy: 0,
        dingy: 0.0,
        mingy: 0,
        seed: 0,
        numgy: 0,
        padding: Default::default(),
    };
    context
        .insert_bundle_exact(entity, (position, velocity, data))
        .expect("entity should be present & should not have these components");

    entity
}

fn remove_component_one(context: &mut Context, entity: Entity) {
    context
        .remove_bundle_exact::<(Position,)>(entity)
        .expect("entity should be present & have `Position` component");
}

fn remove_component_two(context: &mut Context, entity: Entity) {
    context
        .remove_bundle_exact::<(Velocity,)>(entity)
        .expect("entity should be present & have `Velocity` component");
}

fn remove_component_three(context: &mut Context, entity: Entity) {
    context
        .remove_bundle_exact::<(Data,)>(entity)
        .expect("entity should be present & have `Data` component");
}

fn add_components(context: &mut Context, entity: Entity) {
    let player = Player {
        rng: RandomXoshiro128::new(0),
        r#type: Default::default(),
        padding: Default::default(),
    };
    let health = Health::default();
    let damage = Damage::default();
    let sprite = Sprite::default();
    context
        .insert_bundle_exact(entity, (player, health, damage, sprite))
        .expect("entity should be present & should not have these components");

    match context.get_bundle::<(Position,)>(entity) {
        Err(IncompatibleBundleError::MissingComponent(_)) => context
            .insert_bundle_exact(entity, (Position::default(),))
            .expect("entity should be present & should not have `Position` component"),
        Err(error) => unreachable!("unexpected error occurred: {error}"),
        Ok(_) => {}
    }
}

fn init_components(
    context: &mut Context,
    entity: Entity,
    rng: &mut RandomXoshiro128,
    player_type: Option<PlayerType>,
) {
    let (position, player, health, damage, sprite) = context
        .get_bundle_mut::<(Position, Player, Health, Damage, Sprite)>(entity)
        .expect("entity should be present & have all these components");

    let mut rng = RandomXoshiro128::new(rng.generate());
    let r#type = player_type.unwrap_or_else(|| {
        let rate = rng.range(1..100);
        match rate {
            ..=3 => PlayerType::NPC,
            ..=30 => PlayerType::Hero,
            _ => PlayerType::Monster,
        }
    });
    *player = Player {
        rng,
        r#type,
        padding: Default::default(),
    };

    *health = Health {
        hp: 0,
        max_hp: match player.r#type {
            PlayerType::Hero => player.rng.range(5..15) as i32,
            PlayerType::Monster => player.rng.range(4..12) as i32,
            PlayerType::NPC => player.rng.range(6..12) as i32,
        },
        status: Default::default(),
        padding: Default::default(),
    };

    *damage = Damage {
        attack: match player.r#type {
            PlayerType::Hero => player.rng.range(4..10) as i32,
            PlayerType::Monster => player.rng.range(3..9) as i32,
            PlayerType::NPC => 0,
        },
        defense: match player.r#type {
            PlayerType::Hero => player.rng.range(2..6) as i32,
            PlayerType::Monster => player.rng.range(2..8) as i32,
            PlayerType::NPC => player.rng.range(3..8) as i32,
        },
    };

    *sprite = Sprite {
        character: SPAWN_SPRITE,
    };

    *position = Position {
        x: player
            .rng
            .range(0..FRAMEBUFFER_WIDTH as u32 + SPAWN_AREA_MARGIN) as f32
            - SPAWN_AREA_MARGIN as f32,
        y: player
            .rng
            .range(0..FRAMEBUFFER_HEIGHT as u32 + SPAWN_AREA_MARGIN) as f32
            - SPAWN_AREA_MARGIN as f32,
    };
}

fn register_cpu_systems<B>(
    executor: &mut CpuExecutor,
    time_delta: Rc<RefCell<TimeDelta>>,
    framebuffer: Rc<RefCell<Framebuffer<B>>>,
) where
    B: AsMut<[u32]> + 'static,
{
    let time_delta_clone = Rc::clone(&time_delta);
    let system = executor.register_system(move |bundles: BundlesMut<(Position, Velocity)>| {
        let time_delta = *time_delta_clone.borrow();
        let timestamp = Instant::now();
        for (_, (position, velocity)) in bundles {
            update_position(position, velocity, time_delta);
        }
        let elapsed = timestamp.elapsed();
        log::info!(">>>> `update_position` system took {elapsed:?}",);
    });
    executor.add_system(system);

    let system = executor.register_system(move |bundles: BundlesMut<(Data,)>| {
        let time_delta = *time_delta.borrow();
        let timestamp = Instant::now();
        for (_, (data,)) in bundles {
            update_data(data, time_delta);
        }
        let elapsed = timestamp.elapsed();
        log::info!(">>>> `update_data` system took {elapsed:?}");
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Position, Velocity, Data)>| {
        let timestamp = Instant::now();
        for (_, (position, velocity, data)) in bundles {
            update_components(position, velocity, data);
        }
        let elapsed = timestamp.elapsed();
        log::info!(">>>> `update_components` system took {elapsed:?}");
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Health,)>| {
        let timestamp = Instant::now();
        for (_, (health,)) in bundles {
            update_health(health);
        }
        let elapsed = timestamp.elapsed();
        log::info!(">>>> `update_health` system took {elapsed:?}");
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Health, Damage)>| {
        let timestamp = Instant::now();
        for (_, (health, damage)) in bundles {
            update_damage(health, damage);
        }
        let elapsed = timestamp.elapsed();
        log::info!(">>>> `update_damage` system took {elapsed:?}");
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Sprite, Player, Health)>| {
        let timestamp = Instant::now();
        for (_, (sprite, player, health)) in bundles {
            update_sprite(sprite, player, health);
        }
        let elapsed = timestamp.elapsed();
        log::info!(">>>> `update_sprite` system took {elapsed:?}");
    });
    executor.add_system(system);

    let system = executor.register_system(move |bundles: BundlesMut<(Position, Sprite)>| {
        let framebuffer = &mut *framebuffer.borrow_mut();
        let timestamp = Instant::now();
        for (_, (position, sprite)) in bundles {
            render_sprite(position, sprite, framebuffer);
        }
        let elapsed = timestamp.elapsed();
        log::info!(">>>> `render_sprite` system took {elapsed:?}");
    });
    executor.add_system(system);
}

fn register_gpu_systems(
    executor: &mut GpuExecutor,
    time_delta_uniform_buffer: &wgpu::Buffer,
    framebuffer_data_storage_buffer: &wgpu::Buffer,
    framebuffer_desc_uniform_buffer: &wgpu::Buffer,
) {
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
            update_position_system,
            update_position_system_entries.iter().cloned(),
        ),
        (
            update_data_system,
            update_data_system_entries.iter().cloned(),
        ),
        (
            render_sprite_system,
            render_sprite_system_entries.iter().cloned(),
        ),
    ]);
}

fn init_wgpu() -> (wgpu::Device, wgpu::Queue) {
    let instance_desc = wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    };
    let instance = wgpu::Instance::new(&instance_desc);

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

fn save_framebuffer_to_file<B>(framebuffer: &Framebuffer<B>, path: &str, index: usize)
where
    B: AsRef<[u32]>,
{
    let path = format!("./dump/{path}/{index}.txt");
    let path = Path::new(&path);

    let prefix = path.parent().expect("failed to get parent directory");
    fs::create_dir_all(prefix).expect("failed to create parent directory");

    let mut framebuffer_file = File::create(path).expect("failed to create framebuffer file");
    for chunk in framebuffer
        .buffer()
        .as_ref()
        .chunks_exact(FRAMEBUFFER_WIDTH)
    {
        for &char in chunk {
            let char = u8::try_from(char).expect("failed to convert character to `u8`");
            assert!(char.is_ascii(), "character should be ASCII");
            framebuffer_file
                .write_all(&[char])
                .expect("failed to write character to file");
        }
        framebuffer_file
            .write_all(b"\n")
            .expect("failed to write newline to file");
    }
}
