use glam::Vec2;
use gpecs::{
    bundle::erased::error::DowncastErrorKind, context::error::IncompatibleBundleError, prelude::*,
};
use gpecs_ecs_benchmark_types::{
    components::{
        Damage, Data, Health, Player, PlayerType, Position, SPAWN_SPRITE, Sprite, StatusEffect,
        Velocity,
    },
    utils::RandomXoshiro128,
};
use num_traits::ToPrimitive;

use crate::framebuffer::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, SPAWN_AREA_MARGIN};

pub fn create_entities_with_mixed_components(context: &mut Context, count: u32) -> Vec<Entity> {
    let entities_capacity = count.try_into().unwrap_or_default();
    let mut entities = Vec::with_capacity(entities_capacity);

    context
        .register_archetype_of::<(Position, Velocity, Data, Player, Health, Damage, Sprite)>()
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

#[expect(
    clippy::nonminimal_bool,
    reason = "preserve exactly from the reference impl"
)]
pub fn prepare_entities_with_mixed_components(
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
        r#type: PlayerType::default(),
        padding: Default::default(),
    };
    let health = Health::default();
    let damage = Damage::default();
    let sprite = Sprite::default();
    context
        .insert_bundle_exact(entity, (player, health, damage, sprite))
        .expect("entity should be present & should not have these components");

    match context.get_bundle::<(Position,)>(entity) {
        Err(IncompatibleBundleError::Downcast(DowncastErrorKind::MissingComponent(_))) => context
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
            0..=3 => PlayerType::NPC,
            4..=30 => PlayerType::Hero,
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
            PlayerType::Hero => player.rng.range(5..15).cast_signed(),
            PlayerType::Monster => player.rng.range(4..12).cast_signed(),
            PlayerType::NPC => player.rng.range(6..12).cast_signed(),
        },
        status: StatusEffect::default(),
        padding: Default::default(),
    };

    *damage = Damage {
        attack: match player.r#type {
            PlayerType::Hero => player.rng.range(4..10).cast_signed(),
            PlayerType::Monster => player.rng.range(3..9).cast_signed(),
            PlayerType::NPC => 0,
        },
        defense: match player.r#type {
            PlayerType::Hero => player.rng.range(2..6).cast_signed(),
            PlayerType::Monster => player.rng.range(2..8).cast_signed(),
            PlayerType::NPC => player.rng.range(3..8).cast_signed(),
        },
    };

    *sprite = Sprite {
        character: SPAWN_SPRITE,
    };

    let framebuffer_width = u32::try_from(FRAMEBUFFER_WIDTH).unwrap();
    let framebuffer_height = u32::try_from(FRAMEBUFFER_HEIGHT).unwrap();
    let spawn_area_margin_float = SPAWN_AREA_MARGIN.to_f32().unwrap();
    let x_rng = player.rng.range(0..framebuffer_width + SPAWN_AREA_MARGIN);
    let y_rng = player.rng.range(0..framebuffer_height + SPAWN_AREA_MARGIN);
    *position = Position {
        data: Vec2 {
            x: x_rng.to_f32().unwrap() - spawn_area_margin_float,
            y: y_rng.to_f32().unwrap() - SPAWN_AREA_MARGIN.to_f32().unwrap(),
        },
    };
}
