use std::{cell::RefCell, fs::File, io::Write, rc::Rc};

use gpecs::{context::error::IncompatibleBundleError, prelude::*};
use gpecs_ecs_benchmark::{
    components::{
        Damage, Data, Health, Player, PlayerType, Position, Sprite, Velocity, NONE_SPRITE,
        SPAWN_SPRITE,
    },
    framebuffer::Framebuffer,
    systems::{
        render_sprite, update_components, update_damage, update_data, update_health,
        update_position, update_sprite,
    },
    utils::{RandomXoshiro128, TimeDelta},
};

const ENTITY_COUNT: usize = 1000;
const FRAMEBUFFER_WIDTH: usize = 320;
const FRAMEBUFFER_HEIGHT: usize = 240;
const FRAMEBUFFER_SIZE: usize = FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT;
const SPAWN_AREA_MARGIN: u32 = 100;

fn main() {
    let mut rng = rand::rng();
    let mut context = Context::new();

    let entities = create_entities_with_mixed_components(&mut context, ENTITY_COUNT);
    prepare_entities_with_mixed_components(&mut context, &mut rng, &entities);

    let time_delta = TimeDelta(0.0);
    let framebuffer = Framebuffer::new(
        FRAMEBUFFER_WIDTH as u32,
        FRAMEBUFFER_HEIGHT as u32,
        vec![NONE_SPRITE; FRAMEBUFFER_SIZE],
    );

    let mut executor = CpuExecutor::new(&mut context);

    let time_delta = Rc::new(RefCell::new(time_delta));
    let framebuffer = Rc::new(RefCell::new(framebuffer));
    register_cpu_systems(&mut executor, time_delta, framebuffer.clone());

    executor.execute();

    save_framebuffer_to_file(&*framebuffer.borrow());
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
    rng: &mut impl rand::Rng,
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
            .expect(""),
        Err(error) => unreachable!("unexpected error occurred: {error}"),
        Ok(_) => {}
    }
}

fn init_components(
    context: &mut Context,
    entity: Entity,
    rng: &mut impl rand::Rng,
    player_type: Option<PlayerType>,
) {
    let (position, player, health, damage, sprite) = context
        .get_bundle_mut::<(Position, Player, Health, Damage, Sprite)>(entity)
        .expect("entity should be present & have all these components");

    let mut rng = RandomXoshiro128::new(rng.next_u32());
    let r#type = player_type.unwrap_or_else(|| {
        let rate = rng.range(1..100);
        match rate {
            ..=3 => PlayerType::NPC,
            ..=30 => PlayerType::Hero,
            _ => PlayerType::Monster,
        }
    });
    *player = Player { rng, r#type };

    *health = Health {
        hp: 0,
        max_hp: match player.r#type {
            PlayerType::Hero => player.rng.range(5..15) as i32,
            PlayerType::Monster => player.rng.range(4..12) as i32,
            PlayerType::NPC => player.rng.range(6..12) as i32,
        },
        status: Default::default(),
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
        for (_, (position, velocity)) in bundles {
            update_position(position, velocity, *time_delta_clone.borrow());
        }
    });
    executor.add_system(system);

    let system = executor.register_system(move |bundles: BundlesMut<(Data,)>| {
        for (_, (data,)) in bundles {
            update_data(data, *time_delta.borrow());
        }
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Position, Velocity, Data)>| {
        for (_, (position, velocity, data)) in bundles {
            update_components(position, velocity, data);
        }
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Health,)>| {
        for (_, (health,)) in bundles {
            update_health(health);
        }
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Health, Damage)>| {
        for (_, (health, damage)) in bundles {
            update_damage(health, damage);
        }
    });
    executor.add_system(system);

    let system = executor.register_system(|bundles: BundlesMut<(Sprite, Player, Health)>| {
        for (_, (sprite, player, health)) in bundles {
            update_sprite(sprite, player, health);
        }
    });
    executor.add_system(system);

    let system = executor.register_system(move |bundles: BundlesMut<(Position, Sprite)>| {
        for (_, (position, sprite)) in bundles {
            render_sprite(position, sprite, &mut *framebuffer.borrow_mut());
        }
    });
    executor.add_system(system);
}

fn save_framebuffer_to_file<B>(framebuffer: &Framebuffer<B>)
where
    B: AsRef<[u32]>,
{
    let mut framebuffer_file =
        File::create("framebuffer.txt").expect("failed to create framebuffer file");
    for chunk in framebuffer.buffer().chunks_exact(FRAMEBUFFER_WIDTH) {
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
