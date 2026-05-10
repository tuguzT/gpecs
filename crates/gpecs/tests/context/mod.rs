use gpecs::{
    archetype::storage::error::EntityNotFoundError, context::error::EntityHasNoDataError,
    prelude::*,
};

use crate::common::{Mass, Position, Tag};

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
    assert_eq!(entity.epoch().into_u16(), 0);
    assert_eq!(entity.world(), WorldId::default());
    assert!(context.contains(entity));

    let position = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        padding: Default::default(),
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
            .flat_map(|(_, bundles)| bundles)
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
        padding: Default::default(),
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
        padding: Default::default(),
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
            .flat_map(|(_, bundles)| bundles)
            .map(|(entity, _)| entity),
        [],
    );
    itertools::assert_equal(
        context
            .bundles::<(Tag, Mass)>()
            .expect("archetype of `Position`, `Mass` and `Tag` should exist")
            .into_iter()
            .flat_map(|(_, bundles)| bundles)
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
            .flat_map(|(_, bundles)| bundles)
            .map(|(entity, _)| entity),
        [],
    );
    assert!(context.contains(entity));

    let error = context
        .get_bundle::<(Position, Mass, Tag)>(entity)
        .expect_err("entity should not have `Position`, `Mass` and `Tag` components");
    assert_eq!(error, EntityHasNoDataError::new(entity).into());

    context.despawn(entity);
    assert!(!context.contains(entity));

    let error = context
        .get_bundle::<(Position, Mass, Tag)>(entity)
        .expect_err("entity should not exist");
    assert_eq!(error, EntityNotFoundError::new(entity).into());
}

#[test]
fn many_entities() {
    let mut context = Context::new();

    for i in 0..12 {
        let entity = context.spawn();
        if i % 2 == 0 {
            let position = Position {
                x: i as f32,
                y: -(i as f32),
                z: 0.0,
                padding: Default::default(),
            };
            context
                .insert_bundle(entity, (Tag, position))
                .expect("entity should exist & archetype of `Tag` and `Position` should be valid");
        } else {
            context
                .insert_bundle(entity, (Mass { value: i }, Tag))
                .expect("entity should exist & archetype of `Mass` and `Tag` should be valid");
        }
    }

    let mut positions_count = 0;
    let positions = context
        .bundles_mut::<(Position,)>()
        .expect("archetype of `Position` should exist");
    for (entity, (position,)) in positions.flat_map(|(_, bundles)| bundles) {
        assert_eq!(entity.index() % 2, 0);
        assert_eq!(position.x, entity.index() as f32);
        assert_eq!(position.y, -(entity.index() as f32));
        assert_eq!(position.z, 0.0);
        positions_count += 1;
    }
    assert_eq!(positions_count, 6);

    let mut masses_count = 0;
    let masses = context
        .bundles_mut::<(Mass,)>()
        .expect("archetype of `Mass` should exist");
    for (entity, (mass,)) in masses.flat_map(|(_, bundles)| bundles) {
        assert_eq!(entity.index() % 2, 1);
        assert_eq!(mass.value, entity.index());
        masses_count += 1;
    }
    assert_eq!(masses_count, 6);

    let mut tags_count = 0;
    let tags = context
        .bundles_mut::<(Tag,)>()
        .expect("archetype of `Tag` should exist");
    for (_, (tag,)) in tags.flat_map(|(_, bundles)| bundles) {
        assert_eq!(tag, &Tag);
        tags_count += 1;
    }
    assert_eq!(tags_count, 12);

    let entities: Vec<_> = context
        .entities()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    for entity in entities {
        context
            .remove_bundle::<(Tag,)>(entity)
            .expect("entity should exist")
    }

    assert_eq!(
        context
            .bundles_mut::<(Position,)>()
            .expect("archetype of `Position` should exist")
            .into_iter()
            .flat_map(|(_, bundles)| bundles)
            .count(),
        6,
    );
    assert_eq!(
        context
            .bundles_mut::<(Mass,)>()
            .expect("archetype of `Mass` should exist")
            .into_iter()
            .flat_map(|(_, bundles)| bundles)
            .count(),
        6,
    );
    assert_eq!(
        context
            .bundles_mut::<(Tag,)>()
            .expect("archetype of `Tag` should exist")
            .into_iter()
            .flat_map(|(_, bundles)| bundles)
            .count(),
        0,
    );
}
