use glam::Vec3;
use gpecs::prelude::*;
use gpecs_simple_types::{Mass, Position, Tag};
use num_traits::ToPrimitive;

pub const ENTITY_COUNT: u32 = if cfg!(debug_assertions) {
    2_400
} else {
    1_200_000
};

pub fn setup(context: &mut Context) {
    log::info!("Filling context with data to process...");
    for i in 0..ENTITY_COUNT {
        let entity = context.spawn();

        let position = Position {
            data: Vec3 {
                x: i.to_f32().unwrap(),
                y: -i.to_f32().unwrap(),
                z: 0.0,
            },
            padding: Default::default(),
        };
        let mass = Mass { value: i };
        match i % 3 {
            0 => {
                context
                    .insert_bundle::<(Tag, Position)>(entity, (Tag, position))
                    .expect("entity should exist & archetype should be valid");
            }
            1 => {
                context
                    .insert_bundle::<(Mass, Tag)>(entity, (mass, Tag))
                    .expect("entity should exist & archetype should be valid");
            }
            _ => {
                context
                    .insert_bundle::<(Position, Mass)>(entity, (position, mass))
                    .expect("entity should exist & archetype should be valid");
            }
        }
    }
}
