use std::{
    cell::RefCell,
    mem,
    rc::Rc,
    time::{Duration, Instant},
};

use gpecs::prelude::*;
use gpecs_ecs_benchmark_types::{
    components::{
        DEFAULT_SEED, Damage, Data, Health, NONE_SPRITE, Player, Position, Sprite, Velocity,
    },
    framebuffer::Framebuffer,
    systems::{
        render_sprite, update_components, update_damage, update_data, update_health,
        update_position, update_sprite,
    },
    utils::{RandomXoshiro128, TimeDelta},
};
use gpecs_itertools::Itertools as _;
use itertools::Itertools as _;
use rayon::prelude::*;

use crate::{
    setup::{create_entities_with_mixed_components, prepare_entities_with_mixed_components},
    statistics::StatisticsRecord,
};

pub fn run<B, E>(
    context: &mut Context,
    entity_count: u32,
    repeat_count: Option<usize>,
    framebuffer: Framebuffer<B>,
    spawn_area_margin: u32,
    mut f: impl FnMut(u128, Duration, Vec<StatisticsRecord>, &Framebuffer<B>) -> Result<(), E>,
) -> Result<&mut Context, E>
where
    B: AsRef<[u32]> + AsMut<[u32]> + 'static,
{
    if entity_count == 0 || repeat_count == Some(0) {
        return Ok(context);
    }

    log::info!("> Running on CPU...");

    let mut rng = RandomXoshiro128::new(DEFAULT_SEED);
    log::info!(">> Creating {entity_count} entities with mixed components...");
    let entities = create_entities_with_mixed_components(context, entity_count);

    log::info!(">> Preparing entities with mixed components...");
    prepare_entities_with_mixed_components(
        context,
        &mut rng,
        &entities,
        framebuffer.desc(),
        spawn_area_margin,
    );

    let mut executor = CpuExecutor::new(context);

    let time_delta = Rc::new(RefCell::new(TimeDelta::default()));
    let framebuffer = Rc::new(RefCell::new(framebuffer));
    let statistics = Rc::new(RefCell::new(Vec::new()));

    log::info!(">> Registering CPU systems...");
    register_cpu_systems(
        &mut executor,
        time_delta.clone(),
        framebuffer.clone(),
        statistics.clone(),
    );

    log::info!(">> Running CPU systems...");
    for i in (0..).maybe_take(repeat_count) {
        framebuffer
            .borrow_mut()
            .buffer_mut()
            .as_mut()
            .fill(NONE_SPRITE);

        let start = Instant::now();
        executor.execute();
        let elapsed = start.elapsed();

        let time_delta = &mut *time_delta.borrow_mut();
        *time_delta = TimeDelta(elapsed.as_secs_f32());

        let statistics = &mut *statistics.borrow_mut();
        let framebuffer = &*framebuffer.borrow();
        f(i, elapsed, mem::take(statistics), framebuffer)?;
    }

    // Return context from the executor
    Ok(executor.into_context())
}

fn register_cpu_systems<B>(
    executor: &mut CpuExecutor,
    time_delta: Rc<RefCell<TimeDelta>>,
    framebuffer: Rc<RefCell<Framebuffer<B>>>,
    statistics: Rc<RefCell<Vec<StatisticsRecord>>>,
) where
    B: AsMut<[u32]> + 'static,
{
    let system = register_update_position_system(executor, statistics.clone(), time_delta.clone());
    executor.add_system(system);

    let system = register_update_data_system(executor, statistics.clone(), time_delta);
    executor.add_system(system);

    let system = register_update_components_system(executor, statistics.clone());
    executor.add_system(system);

    let system = register_update_health_system(executor, statistics.clone());
    executor.add_system(system);

    let system = register_update_damage_system(executor, statistics.clone());
    executor.add_system(system);

    let system = register_update_sprite_system(executor, statistics.clone());
    executor.add_system(system);

    let system = register_render_sprite_system(executor, statistics, framebuffer);
    executor.add_system(system);
}

fn register_update_position_system(
    executor: &mut CpuExecutor,
    statistics: Rc<RefCell<Vec<StatisticsRecord>>>,
    time_delta: Rc<RefCell<TimeDelta>>,
) -> SystemId {
    let system = move |system: SystemId, bundles: BundlesMut<(Position, Velocity)>| {
        let time_delta = *time_delta.borrow();

        let bundles = bundles
            .filter(|(_, bundles)| !bundles.is_empty())
            .collect_vec()
            .into_par_iter();
        let mut local_statistics = Vec::with_capacity(bundles.len());

        let map = bundles.map(|(archetype, bundles)| {
            let start = Instant::now();

            bundles.into_iter().for_each(|(_, (position, velocity))| {
                update_position(position, velocity, time_delta);
            });

            StatisticsRecord {
                system: system.into(),
                name: "update_position".into(),
                archetype,
                elapsed: start.elapsed(),
            }
        });
        map.collect_into_vec(&mut local_statistics);
        local_statistics.sort();

        statistics.borrow_mut().extend(local_statistics);
    };
    executor.register_system(system)
}

fn register_update_data_system(
    executor: &mut CpuExecutor,
    statistics: Rc<RefCell<Vec<StatisticsRecord>>>,
    time_delta: Rc<RefCell<TimeDelta>>,
) -> SystemId {
    let system = move |system: SystemId, bundles: BundlesMut<(Data,)>| {
        let time_delta = *time_delta.borrow();

        let bundles = bundles
            .filter(|(_, bundles)| !bundles.is_empty())
            .collect_vec()
            .into_par_iter();
        let mut local_statistics = Vec::with_capacity(bundles.len());

        let map = bundles.map(|(archetype, bundles)| {
            let start = Instant::now();

            bundles.into_iter().for_each(|(_, (data,))| {
                update_data(data, time_delta);
            });

            StatisticsRecord {
                system: system.into(),
                name: "update_data".into(),
                archetype,
                elapsed: start.elapsed(),
            }
        });
        map.collect_into_vec(&mut local_statistics);
        local_statistics.sort();

        statistics.borrow_mut().extend(local_statistics);
    };
    executor.register_system(system)
}

fn register_update_components_system(
    executor: &mut CpuExecutor,
    statistics: Rc<RefCell<Vec<StatisticsRecord>>>,
) -> SystemId {
    let system = move |system: SystemId, bundles: BundlesMut<(Position, Velocity, Data)>| {
        let bundles = bundles
            .filter(|(_, bundles)| !bundles.is_empty())
            .collect_vec()
            .into_par_iter();
        let mut local_statistics = Vec::with_capacity(bundles.len());

        let map = bundles.map(|(archetype, bundles)| {
            let start = Instant::now();

            let bundles = bundles.into_iter();
            bundles.for_each(|(_, (position, velocity, data))| {
                update_components(position, velocity, data);
            });

            StatisticsRecord {
                system: system.into(),
                name: "update_components".into(),
                archetype,
                elapsed: start.elapsed(),
            }
        });
        map.collect_into_vec(&mut local_statistics);
        local_statistics.sort();

        statistics.borrow_mut().extend(local_statistics);
    };
    executor.register_system(system)
}

fn register_update_health_system(
    executor: &mut CpuExecutor,
    statistics: Rc<RefCell<Vec<StatisticsRecord>>>,
) -> SystemId {
    let system = move |system: SystemId, bundles: BundlesMut<(Health,)>| {
        let bundles = bundles
            .filter(|(_, bundles)| !bundles.is_empty())
            .collect_vec()
            .into_par_iter();
        let mut local_statistics = Vec::with_capacity(bundles.len());

        let map = bundles.map(|(archetype, bundles)| {
            let start = Instant::now();

            bundles.into_iter().for_each(|(_, (health,))| {
                update_health(health);
            });

            StatisticsRecord {
                system: system.into(),
                name: "update_health".into(),
                archetype,
                elapsed: start.elapsed(),
            }
        });
        map.collect_into_vec(&mut local_statistics);
        local_statistics.sort();

        statistics.borrow_mut().extend(local_statistics);
    };
    executor.register_system(system)
}

fn register_update_damage_system(
    executor: &mut CpuExecutor,
    statistics: Rc<RefCell<Vec<StatisticsRecord>>>,
) -> SystemId {
    let system = move |system: SystemId, bundles: BundlesMut<(Health, Damage)>| {
        let bundles = bundles
            .filter(|(_, bundles)| !bundles.is_empty())
            .collect_vec()
            .into_par_iter();
        let mut local_statistics = Vec::with_capacity(bundles.len());

        let map = bundles.map(|(archetype, bundles)| {
            let start = Instant::now();

            bundles.into_iter().for_each(|(_, (health, damage))| {
                update_damage(health, damage);
            });

            StatisticsRecord {
                system: system.into(),
                name: "update_damage".into(),
                archetype,
                elapsed: start.elapsed(),
            }
        });
        map.collect_into_vec(&mut local_statistics);
        local_statistics.sort();

        statistics.borrow_mut().extend(local_statistics);
    };
    executor.register_system(system)
}

fn register_update_sprite_system(
    executor: &mut CpuExecutor,
    statistics: Rc<RefCell<Vec<StatisticsRecord>>>,
) -> SystemId {
    let system = move |system: SystemId, bundles: BundlesMut<(Sprite, Player, Health)>| {
        let bundles = bundles
            .filter(|(_, bundles)| !bundles.is_empty())
            .collect_vec()
            .into_par_iter();
        let mut local_statistics = Vec::with_capacity(bundles.len());

        let map = bundles.map(|(archetype, bundles)| {
            let start = Instant::now();

            let bundles = bundles.into_iter();
            bundles.for_each(|(_, (sprite, player, health))| {
                update_sprite(sprite, player, health);
            });

            StatisticsRecord {
                system: system.into(),
                name: "update_sprite".into(),
                archetype,
                elapsed: start.elapsed(),
            }
        });
        map.collect_into_vec(&mut local_statistics);
        local_statistics.sort();

        statistics.borrow_mut().extend(local_statistics);
    };
    executor.register_system(system)
}

fn register_render_sprite_system<B>(
    executor: &mut CpuExecutor,
    statistics: Rc<RefCell<Vec<StatisticsRecord>>>,
    framebuffer: Rc<RefCell<Framebuffer<B>>>,
) -> SystemId
where
    B: AsMut<[u32]> + 'static,
{
    let system = move |system: SystemId, bundles: BundlesMut<(Position, Sprite)>| {
        let framebuffer = &mut *framebuffer.borrow_mut();

        let bundles = bundles.filter(|(_, bundles)| !bundles.is_empty());
        let map = bundles.map(|(archetype, bundles)| {
            let start = Instant::now();

            bundles.into_iter().for_each(|(_, (position, sprite))| {
                render_sprite(position, sprite, framebuffer);
            });

            StatisticsRecord {
                system: system.into(),
                name: "render_sprite".into(),
                archetype,
                elapsed: start.elapsed(),
            }
        });
        let mut local_statistics = map.collect_vec();
        local_statistics.sort();

        statistics.borrow_mut().extend(local_statistics);
    };
    executor.register_system(system)
}
