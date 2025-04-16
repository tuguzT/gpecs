use gpecs::{
    archetype::error::MissingComponentError, context::error::EntityNotFoundError, prelude::*,
};

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
        .insert_bundle_exact::<(Position, Mass, Tag)>(entity, (position, mass, tag))
        .expect("entity should exist & not contain `Position` component yet");

    itertools::assert_equal(
        context
            .bundles::<(Position, Mass, Tag)>()
            .expect("archetype of `Position`, `Mass` and `Tag` should exist")
            .into_iter()
            .map(|(entity, _)| entity),
        [entity],
    );

    let (position_mut,) = context
        .get_bundle_mut::<(Position,)>(entity)
        .expect("entity should exist & contain `Position` component");
    assert_eq!(position_mut.x, 1.0);
    assert_eq!(position_mut.y, 2.0);
    assert_eq!(position_mut.z, 3.0);
    *position_mut = Position {
        x: 4.0,
        y: 5.0,
        z: 6.0,
    };

    let (&tag, position) = context
        .get_bundle::<(Tag, Position)>(entity)
        .expect("entity should exist & contain `Tag` and `Position` components");
    assert_eq!(tag, Tag);
    assert_eq!(position.x, 4.0);
    assert_eq!(position.y, 5.0);
    assert_eq!(position.z, 6.0);

    let position = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    context
        .insert_bundle::<(Position,)>(entity, (position,))
        .expect("entity should exist & archetype of `Position` should be valid");

    let (position,) = context
        .remove_bundle_exact::<(Position,)>(entity)
        .expect("entity should exist & contain `Position` component");
    assert!(context.contains(entity));
    assert_eq!(position.x, 1.0);
    assert_eq!(position.y, 2.0);
    assert_eq!(position.z, 3.0);

    itertools::assert_equal(
        context
            .bundles::<(Position, Mass, Tag)>()
            .expect("archetype of `Position`, `Mass` and `Tag` should exist")
            .into_iter()
            .map(|(entity, _)| entity),
        [],
    );
    itertools::assert_equal(
        context
            .bundles::<(Tag, Mass)>()
            .expect("archetype of `Position`, `Mass` and `Tag` should exist")
            .into_iter()
            .map(|(entity, _)| entity),
        [entity],
    );

    context
        .remove_bundle::<(Mass, Tag, Position)>(entity)
        .expect("entity should exist");
    itertools::assert_equal(
        context
            .bundles::<(Tag, Mass)>()
            .expect("archetype of `Position`, `Mass` and `Tag` should exist")
            .into_iter()
            .map(|(entity, _)| entity),
        [],
    );
    assert!(context.contains(entity));

    let error = context
        .get_bundle::<(Position, Mass, Tag)>(entity)
        .expect_err("entity should not have `Position`, `Mass` and `Tag` components");
    assert_eq!(
        error,
        MissingComponentError::new(context.components_mut().register_component::<Tag>()).into(),
    );

    context.despawn(entity);
    assert!(!context.contains(entity));

    let error = context
        .get_bundle::<(Position, Mass, Tag)>(entity)
        .expect_err("entity should not exist");
    assert_eq!(error, EntityNotFoundError::new(entity).into());
}
