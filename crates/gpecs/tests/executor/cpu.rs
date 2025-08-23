use gpecs::prelude::*;
use num_traits::ToPrimitive;

use crate::common::Position;

#[test]
fn execute_simple() {
    let mut context = Context::new();
    let mut executor = CpuExecutor::new(&mut context);

    let hello_system = executor.register_system(|| println!("Hello from the hello system!"));
    let panic_system = executor.register_system(|| unreachable!("Hello from the panic system!"));
    let create_positions_system = executor.register_system(create_positions);
    let update_positions_system = executor.register_system(update_positions);
    let validate_positions_system = executor.register_system(validate_positions);

    let added = executor.add_system(panic_system);
    assert!(added, "system {panic_system:?} should not be scheduled yet");

    let added = executor.add_system(hello_system);
    assert!(added, "system {hello_system:?} should not be scheduled yet");

    let added = executor.add_system(panic_system);
    assert!(!added, "system {panic_system:?} should be scheduled before");

    let added = executor.add_system(create_positions_system);
    assert!(
        added,
        "system {create_positions_system:?} should not be scheduled yet"
    );

    let added = executor.add_system(update_positions_system);
    assert!(
        added,
        "system {update_positions_system:?} should not be scheduled yet"
    );

    let added = executor.add_system(validate_positions_system);
    assert!(
        added,
        "system {validate_positions_system:?} should not be scheduled yet"
    );

    let removed = executor.remove_system(panic_system);
    assert!(
        removed,
        "system {panic_system:?} should be scheduled before"
    );

    let removed = executor.remove_system(panic_system);
    assert!(
        !removed,
        "system {panic_system:?} should not be scheduled yet"
    );

    executor.execute();
}

fn create_positions(context: &mut Context) {
    println!("Hello from the `create_positions` system!");
    for i in 0..10 {
        let entity = context.spawn();
        assert_eq!(entity.index(), i);

        let position = Position {
            x: i.to_f32().unwrap(),
            y: -i.to_f32().unwrap(),
            z: 0.0,
        };
        context.insert_bundle(entity, (position,)).unwrap();
    }
}

fn update_positions(positions: BundlesMut<(Position,)>) {
    println!("Hello from the `update_positions` system!");
    for (_, (position,)) in positions {
        position.x *= 2.0;
        position.y /= 2.0;
        position.z += 1.0;
    }
}

fn validate_positions(positions: Bundles<(Position,)>) {
    println!("Hello from the `validate_positions` system!");
    for (entity, (position,)) in positions {
        assert_eq!(position.x, entity.index().to_f32().unwrap() * 2.0);
        assert_eq!(position.y, -entity.index().to_f32().unwrap() / 2.0);
        assert_eq!(position.z, 1.0);
    }
}
