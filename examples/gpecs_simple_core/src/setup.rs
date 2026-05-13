use glam::Vec3;
use gpecs::prelude::*;
use gpecs_simple_types::{Mass, Position, Tag};
use num_traits::ToPrimitive;

pub fn setup(context: &mut Context, entity_count: u32) {
    log::info!("Filling context with data to process...");
    for i in 0..entity_count {
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
