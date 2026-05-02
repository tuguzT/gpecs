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
use rayon::prelude::*;

use crate::{
    dump::dump_framebuffer_into_file,
    framebuffer::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_SIZE, FRAMEBUFFER_WIDTH},
    setup::{create_entities_with_mixed_components, prepare_entities_with_mixed_components},
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

    log::info!(">> Registering CPU systems...");
    register_cpu_systems(&mut executor, time_delta.clone(), framebuffer.clone());

    log::info!(">> Running CPU systems...");
    for i in (0_u128..).maybe_take(repeat_count) {
        let timestamp = Instant::now();
        executor.execute();

        let elapsed = timestamp.elapsed();
        log::info!(">>! Execution of CPU systems {i} took {elapsed:?}");

        let time_delta = &mut *time_delta.borrow_mut();
        *time_delta = TimeDelta(elapsed.as_secs_f32());

        log::info!(">>> Saving framebuffer state {i} to file...");
        let framebuffer = &*framebuffer.borrow();
        dump_framebuffer_into_file(framebuffer, "cpu", i);
    }

    // Return context from the executor
    executor.into_context()
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
        bundles
            .into_par_iter()
            .for_each(|(_, (position, velocity))| update_position(position, velocity, time_delta));
        let elapsed = timestamp.elapsed();
        log::info!(">>>> `update_position` system took {elapsed:?}");
    });
    executor.add_system(system);

    let system = executor.register_system(move |bundles: BundlesMut<(Data,)>| {
        let time_delta = *time_delta.borrow();
        let timestamp = Instant::now();
        bundles
            .into_par_iter()
            .for_each(|(_, (data,))| update_data(data, time_delta));
        let elapsed = timestamp.elapsed();
        log::info!(">>>> `update_data` system took {elapsed:?}");
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Position, Velocity, Data)>| {
        let timestamp = Instant::now();
        bundles
            .into_par_iter()
            .for_each(|(_, (position, velocity, data))| {
                update_components(position, velocity, data)
            });
        let elapsed = timestamp.elapsed();
        log::info!(">>>> `update_components` system took {elapsed:?}");
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Health,)>| {
        let timestamp = Instant::now();
        bundles
            .into_par_iter()
            .for_each(|(_, (health,))| update_health(health));
        let elapsed = timestamp.elapsed();
        log::info!(">>>> `update_health` system took {elapsed:?}");
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Health, Damage)>| {
        let timestamp = Instant::now();
        bundles
            .into_par_iter()
            .for_each(|(_, (health, damage))| update_damage(health, damage));
        let elapsed = timestamp.elapsed();
        log::info!(">>>> `update_damage` system took {elapsed:?}");
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Sprite, Player, Health)>| {
        let timestamp = Instant::now();
        bundles
            .into_par_iter()
            .for_each(|(_, (sprite, player, health))| update_sprite(sprite, player, health));
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
