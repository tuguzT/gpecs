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
    let check_positions_system = executor.register_system(update_positions);
    let check_masses_system = executor.register_system(update_masses);
    // let check_tags_system = executor.register_system(check_tags);

    // Setup order of systems' execution
    executor.add_system(check_positions_system);
    // executor.add_system(check_tags_system);
    executor.add_system(check_masses_system);

    log::info!("Starting to execute systems on CPU...");
    for i in (0_u128..).maybe_take(repeat_count) {
        let start = Instant::now();
        executor.execute();

        let duration = start.elapsed();
        log::info!("Execution of all the CPU systems {i} took {duration:?}");
    }

    // Return context from the executor
    executor.into_context()
}

fn update_positions(positions: BundlesMut<(Position,)>) {
    // log::info!("Hello from the CPU system working with positions!");

    let positions = positions.collect_vec().into_par_iter();
    positions.for_each(|positions| {
        let archetype_id = positions.archetype_id();
        let start = Instant::now();

        let positions = positions.into_meta().into_par_iter();
        positions.for_each(|(entity, (position,))| {
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
        log::info!("CPU system `update_positions` with {archetype_id} took {duration:?}");
    });
}

fn update_masses(context: &mut Context) {
    // log::info!("Hello from the CPU system working with masses!");

    let masses = context
        .bundles_mut::<(Mass,)>()
        .expect("archetype of `Mass` should exist")
        .collect_vec()
        .into_par_iter();
    masses.for_each(|masses| {
        let archetype_id = masses.archetype_id();
        let start = Instant::now();

        let masses = masses.into_meta().into_par_iter();
        masses.for_each(|(entity, (mass,))| {
            assert!(matches!(entity.index() % 3, 1 | 2));
            // assert_eq!(mass.value, entity.index());

            // log::debug!("{entity} has mass of {}", mass.value);
            mass.value = entity.index();
            log::debug!("{entity} mass have been updated to {}", mass.value);
        });

        let duration = start.elapsed();
        log::info!("CPU system `update_masses` with {archetype_id} took {duration:?}");
    });
}

fn _check_tags(tags: Bundles<(Tag,)>) {
    // log::info!("Hello from the CPU system working with tags!");

    let tags = tags.collect_vec().into_par_iter();
    tags.for_each(|tags| {
        let archetype_id = tags.archetype_id();
        let start = Instant::now();

        let tags = tags.into_meta().into_par_iter();
        tags.for_each(|(entity, (tag,))| {
            assert!(matches!(entity.index() % 3, 0 | 1));
            assert_eq!(tag, &Tag);

            log::debug!("{entity} has {tag:?}");
            // tags_count += 1;
        });

        let duration = start.elapsed();
        log::info!("CPU system `check_tags` with {archetype_id} took {duration:?}");
    });
}
