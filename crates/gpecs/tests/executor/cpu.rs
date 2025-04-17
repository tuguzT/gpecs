use gpecs::prelude::*;

use crate::common::{Mass, Position, Tag};

#[test]
fn execute_simple() {
    let mut context = Context::new();
    let mut executor = CpuExecutor::new(&mut context);

    let system1 = executor.register_system(|| println!("Hello from the simple system!"));
    let system2 = executor.register_system(|| unreachable!());

    let added = executor.add_system(system2);
    assert!(added, "system {system2:?} should not be scheduled yet");

    let added = executor.add_system(system1);
    assert!(added, "system {system1:?} should not be scheduled yet");

    let added = executor.add_system(system2);
    assert!(!added, "system {system2:?} should be scheduled before");

    let removed = executor.remove_system(system2);
    assert!(removed, "system {system2:?} should be scheduled before");

    let removed = executor.remove_system(system2);
    assert!(!removed, "system {system2:?} should not be scheduled yet");

    executor.execute();
}

#[test]
fn execute_bundles() {
    let mut context = Context::new();
    for i in 0..12 {
        let entity = context.spawn();
        if i % 2 == 0 {
            let x = i as f32;
            let y = -(i as f32);
            let z = 0.0;
            context
                .insert_bundle(entity, (Tag, Position { x, y, z }))
                .expect("entity should exist & archetype of `Tag` and `Position` should be valid");
        } else {
            context
                .insert_bundle(entity, (Mass { value: i }, Tag))
                .expect("entity should exist & archetype of `Mass` and `Tag` should be valid");
        }
    }

    let mut executor = CpuExecutor::new(&mut context);

    let position_system = executor.register_system(|positions: BundlesMut<(Position,)>| {
        println!("Hello from the system working with positions!");

        let mut positions_count = 0;
        for (entity, (position,)) in positions {
            assert_eq!(entity.index() % 2, 0);
            assert_eq!(position.x, entity.index() as f32);
            assert_eq!(position.y, -(entity.index() as f32));
            assert_eq!(position.z, 0.0);
            positions_count += 1;
        }
        assert_eq!(positions_count, 6);
    });
    let mass_system = executor.register_system(|context: &mut Context| {
        println!("Hello from the system working with masses!");

        let mut masses_count = 0;
        let masses = context
            .bundles_mut::<(Mass,)>()
            .expect("archetype of `Mass` should exist");
        for (entity, (mass,)) in masses {
            assert_eq!(entity.index() % 2, 1);
            assert_eq!(mass.value, entity.index());
            masses_count += 1;
        }
        assert_eq!(masses_count, 6);
    });
    let tag_system = executor.register_system(|tags: Bundles<(Tag,)>| {
        println!("Hello from the system working with tags!");

        let mut tags_count = 0;
        for (_, (tag,)) in tags {
            assert_eq!(tag, &Tag);
            tags_count += 1;
        }
        assert_eq!(tags_count, 12);
    });

    executor.add_system(position_system);
    executor.add_system(mass_system);
    executor.add_system(tag_system);
    executor.execute();
}
