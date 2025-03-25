use gpecs::world::registry::WorldRegistry;

#[test]
fn new() {
    let registry = WorldRegistry::new();
    assert_eq!(registry.len(), 0);
}

#[test]
fn one_item() {
    let mut registry = WorldRegistry::new();
    let world_id = registry.create();

    assert_eq!(registry.len(), 1);
    assert_eq!(world_id.index(), 1);
}

#[test]
fn three_items() {
    let mut registry = WorldRegistry::new();
    let world_id1 = registry.create();
    let world_id2 = registry.create();
    let world_id3 = registry.create();

    assert_eq!(registry.len(), 3);
    assert_eq!(world_id1.index(), 1);
    assert_eq!(world_id2.index(), 2);
    assert_eq!(world_id3.index(), 3);
}

#[test]
fn overflow_items() {
    let mut registry = WorldRegistry::new();
    for idx in 0..u16::MAX {
        let world_id = registry.create();
        assert_eq!(world_id.index(), idx + 1);
        assert_eq!(registry.len(), idx + 1);
    }
    assert_eq!(registry.len(), u16::MAX);

    let world_id = registry.create();
    assert_eq!(registry.len(), u16::MAX);
    assert_eq!(world_id.index(), 1);
}
