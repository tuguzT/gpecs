use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use glam::Vec3;
use gpecs::prelude::*;
use gpecs_itertools::Itertools as _;
use gpecs_simple_types::{Mass, Position, Tag};
use itertools::Itertools as _;
use num_traits::ToPrimitive;
use rayon::prelude::*;

use crate::setup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StatisticsRecord {
    system: SystemId,
    name: &'static str,
    archetype: ArchetypeId,
    elapsed: Duration,
}

pub fn run(context: &mut Context, entity_count: u32, repeat_count: Option<usize>) -> &mut Context {
    setup::setup(context, entity_count);

    let mut executor = CpuExecutor::new(context);
    let statistics = Rc::new(RefCell::new(Vec::new()));
    register_cpu_systems(&mut executor, statistics.clone());

    log::info!("Starting to execute systems on CPU...");
    for i in (0_u128..).maybe_take(repeat_count) {
        let start = Instant::now();
        executor.execute();
        let elapsed = start.elapsed();

        for record in statistics.borrow_mut().drain(..) {
            let StatisticsRecord {
                system,
                name,
                archetype,
                elapsed,
            } = record;
            log::info!("CPU {system} `{name}` with {archetype} took {elapsed:?}");
        }
        log::info!("Execution of all the CPU systems {i} took {elapsed:?}");
    }

    // Return context from the executor to the caller
    executor.into_context()
}

fn register_cpu_systems(
    executor: &mut CpuExecutor,
    statistics: Rc<RefCell<Vec<StatisticsRecord>>>,
) {
    let update_positions_system = register_update_positions_system(executor, statistics.clone());
    let update_masses_system = register_update_masses_system(executor, statistics.clone());
    let _check_tags_system = register_check_tags_system(executor, statistics);

    executor.add_system(update_positions_system);
    // executor.add_system(check_tags_system);
    executor.add_system(update_masses_system);
}

fn register_update_positions_system(
    executor: &mut CpuExecutor,
    statistics: Rc<RefCell<Vec<StatisticsRecord>>>,
) -> SystemId {
    let system = move |system: SystemId, positions: BundlesMut<(Position,)>| {
        let positions = positions
            .filter(|(_, bundles)| !bundles.is_empty())
            .collect_vec()
            .into_par_iter();
        let mut local_statistics = Vec::with_capacity(positions.len());

        let map = positions.map(|(archetype, positions)| {
            let start = Instant::now();

            positions.into_iter().for_each(|(entity, (position,))| {
                assert!(matches!(entity.index() % 3, 0 | 2));
                // assert_eq!(position.data.x, entity.index() as f32);
                // assert_eq!(position.data.y, -(entity.index() as f32));
                // assert_eq!(position.data.z, 0.0);

                position.data = Vec3 {
                    x: entity.index().to_f32().unwrap(),
                    y: entity.index().to_f32().unwrap() / 2.0,
                    z: -entity.index().to_f32().unwrap() / 2.0,
                };
            });

            StatisticsRecord {
                system,
                name: "update_positions",
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

fn register_update_masses_system(
    executor: &mut CpuExecutor,
    statistics: Rc<RefCell<Vec<StatisticsRecord>>>,
) -> SystemId {
    let system = move |system: SystemId, masses: BundlesMut<(Mass,)>| {
        let masses = masses
            .filter(|(_, bundles)| !bundles.is_empty())
            .collect_vec()
            .into_par_iter();
        let mut local_statistics = Vec::with_capacity(masses.len());

        let map = masses.map(|(archetype, masses)| {
            let start = Instant::now();

            masses.into_iter().for_each(|(entity, (mass,))| {
                assert!(matches!(entity.index() % 3, 1 | 2));
                // assert_eq!(mass.value, entity.index());

                mass.value = entity.index();
            });

            StatisticsRecord {
                system,
                name: "update_masses",
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

fn register_check_tags_system(
    executor: &mut CpuExecutor,
    statistics: Rc<RefCell<Vec<StatisticsRecord>>>,
) -> SystemId {
    let system = move |system: SystemId, tags: Bundles<(Tag,)>| {
        let tags = tags
            .filter(|(_, bundles)| !bundles.is_empty())
            .collect_vec()
            .into_par_iter();
        let mut local_statistics = Vec::with_capacity(tags.len());

        let map = tags.map(|(archetype, tags)| {
            let start = Instant::now();

            tags.into_iter().for_each(|(entity, (tag,))| {
                assert!(matches!(entity.index() % 3, 0 | 1));
                assert_eq!(tag, &Tag);

                // tags_count += 1;
            });

            StatisticsRecord {
                system,
                name: "check_tags",
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
