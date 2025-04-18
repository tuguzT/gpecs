use gpecs::world::registry::{WorldId, WorldRegistry};

#[test]
fn new() {
    let worlds = WorldRegistry::new();
    assert_eq!(worlds.len(), 1);
    assert_eq!(worlds.world_ids().last(), Some(WorldId::new()));
}

#[test]
fn one_item() {
    let mut worlds = WorldRegistry::new();
    let world = worlds.spawn();

    assert_eq!(worlds.len(), 2);
    assert_eq!(world.into_u16(), 1);
    assert!(worlds.world_ids().eq([WorldId::new(), world]));
}

#[test]
fn three_items() {
    let mut worlds = WorldRegistry::new();
    let world1 = worlds.spawn();
    let world2 = worlds.spawn();
    let world3 = worlds.spawn();

    assert_eq!(worlds.len(), 4);
    assert_eq!(world1.into_u16(), 1);
    assert_eq!(world2.into_u16(), 2);
    assert_eq!(world3.into_u16(), 3);
    assert!(worlds
        .world_ids()
        .eq([WorldId::new(), world1, world2, world3]));
}

#[test]
#[cfg_attr(miri, ignore)]
fn overflow_items() {
    let mut worlds = WorldRegistry::new();
    for idx in 0..u16::MAX - 1 {
        let world = worlds.spawn();
        assert_eq!(world.into_u16(), idx + 1);
        assert_eq!(worlds.len(), idx + 2);
    }

    let world = worlds.spawn();
    assert_eq!(world.into_u16(), u16::MAX);
    assert_eq!(worlds.len(), u16::MAX);

    let world = worlds.spawn();
    assert_eq!(worlds.len(), u16::MAX);
    assert_eq!(world.into_u16(), 0);
}
