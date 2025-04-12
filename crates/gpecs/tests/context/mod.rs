use gpecs::prelude::*;

#[test]
fn empty() {
    let context = Context::new();

    let (worlds, entities, components, archetypes) = context.as_parts();
    assert_eq!(worlds.len(), 1);
    assert_eq!(entities.len(), 0);
    assert_eq!(components.len(), 0);
    assert_eq!(archetypes.len(), 0);
}

#[test]
fn one_entity() {
    let mut context = Context::new();

    let entity = context.spawn();
    assert_eq!(entity.index(), 0);
    assert_eq!(entity.epoch(), 0);
    assert_eq!(entity.world(), WorldId::default());
    assert!(context.contains(entity));

    context.despawn(entity);
    assert!(!context.contains(entity));
}
