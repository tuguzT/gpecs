use gpecs::entity::{registry::EntityRegistry, Entity};

#[test]
fn new() {
    let registry = EntityRegistry::new();
    assert_eq!(registry.len(), 0);
    assert_eq!(registry.capacity(), 0);
}

#[test]
fn with_capacity() {
    let registry = EntityRegistry::with_capacity(10);
    assert_eq!(registry.len(), 0);
    assert!(registry.capacity() >= 10);
}

#[test]
fn one_item_spawn() {
    let mut registry = EntityRegistry::new();
    let entity = registry.spawn();

    assert_eq!(registry.len(), 1);
    assert!(registry.capacity() >= 1);

    assert_eq!(entity.sparse_index(), 0);
    assert_eq!(entity.epoch(), 0);

    assert!(registry.contains(entity));
    assert_eq!(registry.get_epoch(0), Some(0));
}

#[test]
fn one_item_reuse() {
    let mut registry = EntityRegistry::new();

    let entity = registry.spawn();
    registry.despawn(entity);
    let entity = registry.spawn();

    assert_eq!(registry.len(), 1);
    assert!(registry.capacity() >= 1);

    assert_eq!(entity.sparse_index(), 0);
    assert_eq!(entity.epoch(), 1);

    assert!(registry.contains(entity));
    assert_eq!(registry.get_epoch(0), Some(1));
}

#[test]
fn one_item_invalidate() {
    let mut registry = EntityRegistry::new();

    let entity = registry.spawn();
    registry.despawn(entity);
    let entity = registry.spawn();
    assert_eq!(entity, Entity::new(0, 1));

    assert_eq!(registry.invalidate_epoch(Entity::new(0, 0)), None);
    assert_eq!(registry.invalidate_epoch(entity), Some(Entity::new(0, 2)));
    assert_eq!(registry.invalidate_epoch(entity), None);

    assert!(!registry.contains(Entity::new(0, 0)));
    assert!(!registry.contains(Entity::new(0, 1)));
    assert!(registry.contains(Entity::new(0, 2)));
}
