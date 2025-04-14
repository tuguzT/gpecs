use gpecs::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Mass {
    value: u16,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Tag;

impl Component for Position {}
impl Component for Mass {}
impl Component for Tag {}

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

    let position = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let mass = Mass { value: 42 };
    let tag = Tag;
    context
        .try_insert_bundle(&(), entity, (position, mass, tag))
        .expect("entity should exist")
        .expect("entity should not contain `Position` component yet");

    let (position_mut,) = context
        .try_get_bundle_mut::<(Position,)>(&(), entity)
        .expect("entity should exist")
        .expect("entity should contain `Position` component");
    assert_eq!(position_mut.x, 1.0);
    assert_eq!(position_mut.y, 2.0);
    assert_eq!(position_mut.z, 3.0);
    *position_mut = Position {
        x: 4.0,
        y: 5.0,
        z: 6.0,
    };

    let (&tag, position) = context
        .try_get_bundle::<(Tag, Position)>(&(), entity)
        .expect("entity should exist")
        .expect("entity should contain `Tag` and `Position` components");
    assert_eq!(tag, Tag);
    assert_eq!(position.x, 4.0);
    assert_eq!(position.y, 5.0);
    assert_eq!(position.z, 6.0);

    let (position,) = context
        .try_remove_bundle::<(Position,)>(&(), entity)
        .expect("entity should exist")
        .expect("entity should contain `Position` component");
    assert_eq!(position.x, 4.0);
    assert_eq!(position.y, 5.0);
    assert_eq!(position.z, 6.0);

    let (mass, tag) = context
        .try_remove_bundle::<(Mass, Tag)>(&(), entity)
        .expect("entity should exist")
        .expect("entity should contain `Mass` and `Tag` components");
    assert_eq!(mass, Mass { value: 42 });
    assert_eq!(tag, Tag);

    context.despawn(entity);
    assert!(!context.contains(entity));
    assert!(context
        .try_get_bundle::<(Position, Mass, Tag)>(&(), entity)
        .is_none());
}
