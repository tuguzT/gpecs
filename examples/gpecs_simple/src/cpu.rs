use std::time::Instant;

use glam::Vec3;
use gpecs::prelude::*;
use gpecs_simple_types::{Mass, Position, Tag};
use num_traits::ToPrimitive;

use crate::{ITER_COUNT, setup::setup};

pub fn run(context: &mut Context) {
    setup(context);

    let mut executor = CpuExecutor::new(context);
    let check_positions_system = executor.register_system(update_positions);
    let check_masses_system = executor.register_system(update_masses);
    // let check_tags_system = executor.register_system(check_tags);

    // Setup order of systems' execution
    executor.add_system(check_positions_system);
    // executor.add_system(check_tags_system);
    executor.add_system(check_masses_system);

    log::info!("Starting to execute systems on CPU...");
    for i in 0..ITER_COUNT {
        let start = Instant::now();
        executor.execute();
        let duration = start.elapsed();
        log::info!("CPU system execution {i} took {duration:?}");
    }

    // Return context from the executor
    let context = executor.into_context();
    context.destroy_all();
}

fn update_positions(positions: BundlesMut<(Position,)>) {
    // log::info!("Hello from the CPU system working with positions!");

    // let mut positions_count = 0;
    let start = Instant::now();
    for (entity, (position,)) in positions {
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

        // positions_count += 1;
    }
    // assert_eq!(positions_count, ENTITY_COUNT / 3 * 2);
    let duration = start.elapsed();
    log::info!("CPU system working with positions took {duration:?}");
}

fn update_masses(context: &mut Context) {
    // log::info!("Hello from the CPU system working with masses!");

    // let mut masses_count = 0;
    let start = Instant::now();
    let masses = context
        .bundles_mut::<(Mass,)>()
        .expect("archetype of `Mass` should exist");
    for (entity, (mass,)) in masses {
        assert!(matches!(entity.index() % 3, 1 | 2));
        // assert_eq!(mass.value, entity.index());

        // log::debug!("{entity} has mass of {}", mass.value);
        mass.value = entity.index();
        log::debug!("{entity} mass have been updated to {}", mass.value);

        // masses_count += 1;
    }
    // assert_eq!(masses_count, ENTITY_COUNT / 3 * 2);
    let duration = start.elapsed();
    log::info!("CPU system working with masses took {duration:?}");
}

fn _check_tags(tags: Bundles<(Tag,)>) {
    // log::info!("Hello from the CPU system working with tags!");

    // let mut tags_count = 0;
    let start = Instant::now();
    for (entity, (tag,)) in tags {
        assert!(matches!(entity.index() % 3, 0 | 1));
        // assert_eq!(tag, &Tag);

        log::debug!("{entity} has {tag:?}");
        // tags_count += 1;
    }
    // assert_eq!(tags_count, ENTITY_COUNT / 3 * 2);
    let duration = start.elapsed();
    log::info!("CPU system working with tags took {duration:?}");
}
