use gpecs::world::registry::WorldRegistry;

#[test]
fn new() {
    let worlds = WorldRegistry::new();
    assert_eq!(worlds.len(), 0);
}

#[test]
fn one_item() {
    let mut worlds = WorldRegistry::new();
    let world = worlds.spawn();

    assert_eq!(worlds.len(), 1);
    assert_eq!(world.index(), 1);
}

#[test]
fn three_items() {
    let mut worlds = WorldRegistry::new();
    let world1 = worlds.spawn();
    let world2 = worlds.spawn();
    let world3 = worlds.spawn();

    assert_eq!(worlds.len(), 3);
    assert_eq!(world1.index(), 1);
    assert_eq!(world2.index(), 2);
    assert_eq!(world3.index(), 3);
}

#[test]
fn overflow_items() {
    let mut worlds = WorldRegistry::new();
    for idx in 0..u16::MAX {
        let world = worlds.spawn();
        assert_eq!(world.index(), idx + 1);
        assert_eq!(worlds.len(), idx + 1);
    }
    assert_eq!(worlds.len(), u16::MAX);

    let world = worlds.spawn();
    assert_eq!(worlds.len(), u16::MAX);
    assert_eq!(world.index(), 1);
}
