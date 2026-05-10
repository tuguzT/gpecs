use std::time::Instant;

use glam::Vec3;
use gpecs::prelude::*;
use gpecs_itertools::Itertools as _;
use gpecs_simple_types::{Mass, Position, Tag};
use itertools::Itertools as _;
use num_traits::ToPrimitive;
use rayon::prelude::*;

use crate::setup;

pub fn run(context: &mut Context, entity_count: u32, repeat_count: Option<usize>) -> &mut Context {
    setup::setup(context, entity_count);

    let mut executor = CpuExecutor::new(context);
    register_cpu_systems(&mut executor);

    log::info!("Starting to execute systems on CPU...");
    for i in (0_u128..).maybe_take(repeat_count) {
        let start = Instant::now();
        executor.execute();

        let duration = start.elapsed();
        log::info!("Execution of all the CPU systems {i} took {duration:?}");
    }

    // Return context from the executor to the caller
    executor.into_context()
}

fn register_cpu_systems(executor: &mut CpuExecutor) {
    let update_positions_system = register_update_positions_system(executor);
    let update_masses_system = register_update_masses_system(executor);
    let _check_tags_system = register_check_tags_system(executor);

    executor.add_system(update_positions_system);
    // executor.add_system(check_tags_system);
    executor.add_system(update_masses_system);
}

fn register_update_positions_system(executor: &mut CpuExecutor) -> SystemId {
    let system = |system: SystemId, positions: BundlesMut<(Position,)>| {
        // log::info!("Hello from the CPU system working with positions!");

        let positions = positions
            .filter(|(_, bundles)| !bundles.is_empty())
            .collect_vec()
            .into_par_iter();
        positions.for_each(|(archetype, positions)| {
            let start = Instant::now();

            positions.into_iter().for_each(|(entity, (position,))| {
                assert!(matches!(entity.index() % 3, 0 | 2));
                // assert_eq!(position.data.x, entity.index() as f32);
                // assert_eq!(position.data.y, -(entity.index() as f32));
                // assert_eq!(position.data.z, 0.0);

                // log::debug!("{entity} has position of {}", position.data);
                position.data = Vec3 {
                    x: entity.index().to_f32().unwrap(),
                    y: entity.index().to_f32().unwrap() / 2.0,
                    z: -entity.index().to_f32().unwrap() / 2.0,
                };
                log::debug!("{entity} position have been updated to {}", position.data);
            });

            let duration = start.elapsed();
            log::info!("CPU {system} `update_positions` with {archetype} took {duration:?}");
        });
    };
    executor.register_system(system)
}

fn register_update_masses_system(executor: &mut CpuExecutor) -> SystemId {
    let system = |system: SystemId, context: &mut Context| {
        // log::info!("Hello from the CPU system working with masses!");

        let masses = context
            .bundles_mut::<(Mass,)>()
            .expect("archetype of `Mass` should exist")
            .filter(|(_, bundles)| !bundles.is_empty())
            .collect_vec()
            .into_par_iter();
        masses.for_each(|(archetype, masses)| {
            let start = Instant::now();

            masses.into_iter().for_each(|(entity, (mass,))| {
                assert!(matches!(entity.index() % 3, 1 | 2));
                // assert_eq!(mass.value, entity.index());

                // log::debug!("{entity} has mass of {}", mass.value);
                mass.value = entity.index();
                log::debug!("{entity} mass have been updated to {}", mass.value);
            });

            let duration = start.elapsed();
            log::info!("CPU {system} `update_masses` with {archetype} took {duration:?}");
        });
    };
    executor.register_system(system)
}

fn register_check_tags_system(executor: &mut CpuExecutor) -> SystemId {
    let system = |system: SystemId, tags: Bundles<(Tag,)>| {
        // log::info!("Hello from the CPU system working with tags!");

        let tags = tags
            .filter(|(_, bundles)| !bundles.is_empty())
            .collect_vec()
            .into_par_iter();
        tags.for_each(|(archetype, tags)| {
            let start = Instant::now();

            tags.into_iter().for_each(|(entity, (tag,))| {
                assert!(matches!(entity.index() % 3, 0 | 1));
                assert_eq!(tag, &Tag);

                log::debug!("{entity} has {tag:?}");
                // tags_count += 1;
            });

            let duration = start.elapsed();
            log::info!("CPU {system} `check_tags` with {archetype} took {duration:?}");
        });
    };
    executor.register_system(system)
}
