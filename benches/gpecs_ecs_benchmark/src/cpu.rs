use std::{cell::RefCell, rc::Rc, time::Instant};

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
    dump::{
        CsvRecord, create_csv_writer, dump_csv_header, dump_csv_record, dump_framebuffer_into_file,
    },
    framebuffer::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_SIZE, FRAMEBUFFER_WIDTH},
    setup::{create_entities_with_mixed_components, prepare_entities_with_mixed_components},
    statistics::{StatisticsRecord, log_statistics},
};

pub fn run(context: &mut Context, entity_count: u32, repeat_count: Option<usize>) -> &mut Context {
    log::info!("> Running on CPU...");

    let mut rng = RandomXoshiro128::new(DEFAULT_SEED);
    log::info!(">> Creating {entity_count} entities with mixed components...");
    let entities = create_entities_with_mixed_components(context, entity_count);

    log::info!(">> Preparing entities with mixed components...");
    prepare_entities_with_mixed_components(context, &mut rng, &entities);

    let time_delta = TimeDelta::default();
    let framebuffer = Framebuffer::new(
        u32::try_from(FRAMEBUFFER_WIDTH).unwrap(),
        u32::try_from(FRAMEBUFFER_HEIGHT).unwrap(),
        vec![NONE_SPRITE; FRAMEBUFFER_SIZE],
    );

    let mut executor = CpuExecutor::new(context);

    let time_delta = Rc::new(RefCell::new(time_delta));
    let framebuffer = Rc::new(RefCell::new(framebuffer));
    let statistics = Rc::new(RefCell::new(Vec::new()));

    log::info!(">> Registering CPU systems...");
    register_cpu_systems(
        &mut executor,
        time_delta.clone(),
        framebuffer.clone(),
        statistics.clone(),
    );

    let mut csv_writer = create_csv_writer("cpu", entity_count)
        .expect("csv writer & its file should be created successfully");

    log::info!(">> Running CPU systems...");
    for i in (0_u128..).maybe_take(repeat_count) {
        let start = Instant::now();
        executor.execute();
        let elapsed = start.elapsed();

        let time_delta = &mut *time_delta.borrow_mut();
        *time_delta = TimeDelta(elapsed.as_secs_f32());

        log::info!(">>> Saving framebuffer state {i} to file...");
        let framebuffer = &*framebuffer.borrow();
        dump_framebuffer_into_file(framebuffer, "cpu", i)
            .expect("framebuffer should be saved successfully");

        let statistics = &mut *statistics.borrow_mut();
        log_statistics("CPU", statistics.as_slice(), i, elapsed);

        let csv_record = CsvRecord::new(elapsed, statistics.drain(..));
        if i == 0 {
            dump_csv_header(&csv_record, &mut csv_writer)
                .expect("csv header should be written successfully");
        }
        dump_csv_record(csv_record, &mut csv_writer)
            .expect("csv record should be written successfully");

        csv_writer
            .flush()
            .expect("csv file should be saved successfully");
    }

    // Return context from the executor
    executor.into_context()
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
