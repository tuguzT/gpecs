use gpecs::{
    entity::{Entity, EntityEpoch, registry::EntityRegistry},
    world::registry::WorldRegistry,
};

#[test]
fn new() {
    let entities = EntityRegistry::<()>::new();
    assert_eq!(entities.len(), 0);
    assert_eq!(entities.capacity(), 0);
}

#[test]
fn with_capacity() {
    let entities = EntityRegistry::<()>::with_capacity(10);
    assert_eq!(entities.len(), 0);
    assert!(entities.capacity() >= 10);
}

#[test]
fn one_item_spawn() {
    let mut worlds = WorldRegistry::new();
    let world = worlds.spawn();

    let mut entities = EntityRegistry::new();
    let entity = entities.spawn(world, ());

    assert_eq!(entities.len(), 1);
    assert!(entities.capacity() >= 1);

    assert_eq!(entity.index(), 0);
    assert_eq!(entity.epoch().into_u16(), 0);

    assert!(entities.contains(entity));
    assert_eq!(entities.get_epoch(0).map(EntityEpoch::into_u16), Some(0));
}

#[test]
fn one_item_reuse() {
    let mut worlds = WorldRegistry::new();
    let world = worlds.spawn();

    let mut entities = EntityRegistry::new();
    let entity = entities.spawn(world, ());

    entities.despawn(entity);
    let entity = entities.spawn(world, ());

    assert_eq!(entities.len(), 1);
    assert!(entities.capacity() >= 1);

    assert_eq!(entity.index(), 0);
    assert_eq!(entity.epoch().into_u16(), 1);

    assert!(entities.contains(entity));
    assert_eq!(entities.get_epoch(0).map(EntityEpoch::into_u16), Some(1));
}

#[test]
fn one_item_invalidate() {
    let mut worlds = WorldRegistry::new();
    let world = worlds.spawn();

    let mut entities = EntityRegistry::new();
    let entity = entities.spawn(world, ());
    entities.despawn(entity);

    let entity = entities.spawn(world, ());
    assert_eq!(entity, Entity::new(0, 1.into(), world));

    assert_eq!(
        entities.invalidate_epoch(Entity::new(0, 0.into(), world)),
        None,
    );
    assert_eq!(
        entities.invalidate_epoch(entity),
        Some(Entity::new(0, 2.into(), world)),
    );
    assert_eq!(entities.invalidate_epoch(entity), None);

    assert!(!entities.contains(Entity::new(0, 0.into(), world)));
    assert!(!entities.contains(Entity::new(0, 1.into(), world)));
    assert!(entities.contains(Entity::new(0, 2.into(), world)));
}
