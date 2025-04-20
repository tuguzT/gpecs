use glam::Vec3;
use gpecs::prelude::*;

use self::common::{Mass, Position, Tag};

mod common;

fn main() {
    let mut context = Context::new();
    for i in 0..24 {
        let entity = context.spawn();

        let position = Position {
            data: Vec3 {
                x: i as f32,
                y: -(i as f32),
                z: 0.0,
            },
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

    let mut executor = CpuExecutor::new(&mut context);

    let position_system = executor.register_system(|positions: BundlesMut<(Position,)>| {
        println!("Hello from the system working with positions!");

        let mut positions_count = 0;
        for (entity, (position,)) in positions {
            assert!(matches!(entity.index() % 3, 0 | 2));
            assert_eq!(position.data.x, entity.index() as f32);
            assert_eq!(position.data.y, -(entity.index() as f32));
            assert_eq!(position.data.z, 0.0);
            positions_count += 1;
        }
        assert_eq!(positions_count, 16);
    });
    let mass_system = executor.register_system(|context: &mut Context| {
        println!("Hello from the system working with masses!");

        let mut masses_count = 0;
        let masses = context
            .bundles_mut::<(Mass,)>()
            .expect("archetype of `Mass` should exist");
        for (entity, (mass,)) in masses {
            assert!(matches!(entity.index() % 3, 1 | 2));
            assert_eq!(mass.value, entity.index());
            masses_count += 1;
        }
        assert_eq!(masses_count, 16);
    });
    let tag_system = executor.register_system(|tags: Bundles<(Tag,)>| {
        println!("Hello from the system working with tags!");

        let mut tags_count = 0;
        for (_, (tag,)) in tags {
            assert_eq!(tag, &Tag);
            tags_count += 1;
        }
        assert_eq!(tags_count, 16);
    });

    executor.add_system(position_system);
    executor.add_system(mass_system);
    executor.add_system(tag_system);
    executor.execute();
}
