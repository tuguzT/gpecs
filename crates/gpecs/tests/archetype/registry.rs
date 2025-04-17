use gpecs::{
    archetype::{
        error::{
            AlreadyHasComponentError, IncompatibleBundleError, InsertBundleExactError,
            MissingComponentError,
        },
        registry::ArchetypeRegistry,
    },
    bundle::Bundle,
    component::registry::ComponentRegistry,
    entity::registry::EntityRegistry,
};

use crate::common::{Mass, Position, Tag};

#[test]
fn new() {
    let archetypes = ArchetypeRegistry::new();
    assert_eq!(archetypes.len(), 0);
    assert!(archetypes.archetype_ids().is_empty());
}

#[test]
fn register_archetype() {
    let mut components = ComponentRegistry::new();
    let mut archetypes = ArchetypeRegistry::new();
    assert_eq!(archetypes.len(), 0);

    let id = archetypes
        .register_archetype::<(Position, Mass, Tag)>(&mut components)
        .expect("archetype of `Position`, `Mass` and `Tag` should contain unique component ids");
    assert!(archetypes.len() > 1);
    assert_eq!(
        archetypes
            .archetype_id::<(Position, Mass, Tag)>(&components)
            .expect("archetype of `Position`, `Mass` and `Tag` should contain unique component ids")
            .expect("archetype of `Position`, `Mass` and `Tag` should be already registered"),
        id,
    );
    assert!(archetypes.archetype_ids().any(|item| item == id));

    let same_id = archetypes
        .register_archetype::<(Mass, Tag, Position)>(&mut components)
        .expect("archetype of `Position`, `Mass` and `Tag` should contain unique component ids");
    assert_eq!(same_id, id);

    let component_ids = <(Mass, Tag, Position)>::register_components(&mut components);
    let same_id = archetypes
        .register_archetype_from(&components, component_ids)
        .expect("archetype of `Position`, `Mass` and `Tag` should contain unique component ids");
    assert_eq!(same_id, id);
    assert_eq!(
        archetypes
            .archetype_id_from(component_ids)
            .expect("archetype of `Position`, `Mass` and `Tag` should contain unique component ids")
            .expect("archetype of `Position`, `Mass` and `Tag` should be already registered"),
        id,
    );

    dbg!(&archetypes);

    let component_ids = <(Position,)>::register_components(&mut components);
    assert_ne!(
        archetypes
            .archetype_id_from(component_ids)
            .expect("archetype of only `Position` should contain unique component ids")
            .expect("archetype of only `Position` should be already registered"),
        id,
    );

    let component_ids = <(Mass,)>::register_components(&mut components);
    assert_ne!(
        archetypes
            .archetype_id_from(component_ids)
            .expect("archetype of only `Mass` should contain unique component ids")
            .expect("archetype of only `Mass` should be already registered"),
        id,
    );

    let component_ids = <(Tag,)>::register_components(&mut components);
    assert_ne!(
        archetypes
            .archetype_id_from(component_ids)
            .expect("archetype of only `Tag` should contain unique component ids")
            .expect("archetype of only `Tag` should be already registered"),
        id,
    );

    let component_ids = <(Mass, Tag)>::register_components(&mut components);
    assert_ne!(
        archetypes
            .archetype_id_from(component_ids)
            .expect("archetype of `Tag` and `Mass` should contain unique component ids")
            .expect("archetype of `Tag` and `Mass` should be already registered"),
        id,
    );

    let new_id = archetypes
        .register_archetype::<(Mass, Tag)>(&mut components)
        .expect("archetype of `Tag` and `Mass` should contain unique component ids");
    assert_ne!(new_id, id);
    assert_eq!(
        archetypes
            .archetype_id::<(Mass, Tag)>(&components)
            .expect("archetype of `Tag` and `Mass` should contain unique component ids")
            .expect("archetype of `Tag` and `Mass` should be already registered"),
        new_id,
    );
    assert!(archetypes.archetype_ids().any(|item| item == new_id));
    let id = new_id;

    let component_ids = <(Mass, Tag)>::register_components(&mut components);
    let same_id = archetypes
        .register_archetype_from(&components, component_ids)
        .expect("archetype of `Tag` and `Mass` should contain unique component ids");
    assert_eq!(same_id, id);
    assert_eq!(
        archetypes
            .archetype_id_from(component_ids)
            .expect("archetype of `Tag` and `Mass` should contain unique component ids")
            .expect("archetype of `Tag` and `Mass` should be already registered"),
        same_id,
    );
}

#[test]
fn exchange_components() {
    let mut entities = EntityRegistry::new();
    let mut components = ComponentRegistry::new();
    let mut archetypes = ArchetypeRegistry::new();

    let archetype = archetypes
        .register_archetype::<(Position, Mass, Tag)>(&mut components)
        .expect("archetype of `Position`, `Mass` and `Tag` should contain unique component ids");
    let archetype_subset = archetypes
        .register_archetype::<(Position, Mass)>(&mut components)
        .expect("archetype of `Position` and `Mass` should contain unique component ids");

    let entity = entities.spawn(Default::default(), ());

    let mut position = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let mass = Mass { value: 42 };
    let mut tag = Tag;

    let storage = archetypes
        .get_archetype_info(archetype_subset)
        .expect("archetype should exist")
        .storage();
    assert!(!storage.contains(entity));

    let error = archetypes
        .get_bundle::<(Mass, Position)>(&components, entity)
        .expect_err("entity should not have `Mass` and `Position` components yet");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Mass>()).into(),
    );

    archetypes
        .insert_bundle_exact::<(Position, Mass)>(&mut components, entity, (position, mass))
        .expect("entity should not have `Position` and `Mass` components yet");

    let InsertBundleExactError { value, kind, .. } = archetypes
        .insert_bundle_exact::<(Mass, Position)>(&mut components, entity, (mass, position))
        .expect_err("entity should already have `Position` and `Mass` components");
    assert_eq!(
        kind,
        AlreadyHasComponentError::new(components.register_component::<Mass>()).into(),
    );
    assert_eq!(value, (mass, position));

    let mut mass = Mass { value: 1024 };
    archetypes
        .insert_bundle::<(Mass,)>(&mut components, entity, (mass,))
        .expect("archetype of only `Mass` should contain unique component ids");

    let storage = archetypes
        .get_archetype_info(archetype_subset)
        .expect("archetype should exist")
        .storage();
    assert!(storage.contains(entity));

    let refs = archetypes
        .get_bundle_mut::<(Mass, Position)>(&components, entity)
        .expect("entity should have `Mass` and `Position` components");
    assert_eq!(refs, (&mut mass, &mut position));

    let error = archetypes
        .get_bundle::<(Mass, Tag, Position)>(&components, entity)
        .expect_err("entity should not have `Tag` component yet");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    archetypes
        .insert_bundle_exact::<(Tag,)>(&mut components, entity, (tag,))
        .expect("entity should not have `Tag` component yet");

    let InsertBundleExactError { value, kind, .. } = archetypes
        .insert_bundle_exact::<(Tag,)>(&mut components, entity, (tag,))
        .expect_err("entity already has `Tag` component");
    assert_eq!(
        kind,
        AlreadyHasComponentError::new(components.register_component::<Tag>()).into(),
    );
    assert_eq!(value, (tag,));

    let mut position = Position {
        x: -1.0,
        y: -2.0,
        z: -3.0,
    };
    let mut mass = Mass {
        value: u32::MAX - 1024,
    };
    archetypes
        .insert_bundle::<(Mass, Position)>(&mut components, entity, (mass, position))
        .expect("archetype of `Mass` and `Position` should contain unique component ids");

    let storage = archetypes
        .get_archetype_info(archetype_subset)
        .expect("archetype should exist")
        .storage();
    assert!(!storage.contains(entity));

    let storage = archetypes
        .get_archetype_info(archetype)
        .expect("archetype should exist")
        .storage();
    assert!(storage.contains(entity));

    let refs = archetypes
        .get_bundle_mut::<(Mass, Tag, Position)>(&components, entity)
        .expect("entity should have `Mass`, `Tag` and `Position` components");
    assert_eq!(refs, (&mut mass, &mut tag, &mut position));

    let (tag,) = archetypes
        .remove_bundle_exact::<(Tag,)>(&mut components, entity)
        .expect("entity should have `Tag` component");
    assert_eq!(tag, Tag);

    let error = archetypes
        .remove_bundle_exact::<(Tag,)>(&mut components, entity)
        .expect_err("entity should not have `Tag` component");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let storage = archetypes
        .get_archetype_info(archetype_subset)
        .expect("archetype should exist")
        .storage();
    assert!(storage.contains(entity));

    let storage = archetypes
        .get_archetype_info(archetype)
        .expect("archetype should exist")
        .storage();
    assert!(!storage.contains(entity));

    let error = archetypes
        .get_bundle_mut::<(Mass, Tag, Position)>(&components, entity)
        .expect_err("entity should not have `Tag` component");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let refs = archetypes
        .get_bundle::<(Mass, Position)>(&components, entity)
        .expect("entity should have `Mass` and `Position` components");
    assert_eq!(refs, (&mass, &position));

    let error = archetypes
        .remove_bundle_exact::<(Mass, Tag, Position)>(&mut components, entity)
        .expect_err("entity should not have `Tag` component");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let value = archetypes
        .remove_bundle_exact::<(Mass, Position)>(&mut components, entity)
        .expect("entity should have `Mass` and `Position` components");
    assert_eq!(value, (mass, position));

    let error = archetypes
        .remove_bundle_exact::<(Mass, Position)>(&mut components, entity)
        .expect_err("entity should not have `Mass` and `Position` components");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Mass>()).into(),
    );

    let storage = archetypes
        .get_archetype_info(archetype_subset)
        .expect("archetype should exist")
        .storage();
    assert!(!storage.contains(entity));

    let storage = archetypes
        .get_archetype_info(archetype)
        .expect("archetype should exist")
        .storage();
    assert!(!storage.contains(entity));

    let error = archetypes
        .remove_bundle_exact::<(Position, Mass)>(&mut components, entity)
        .expect_err("entity should not have `Position` and `Mass` component");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Position>()).into(),
    );
}

#[test]
fn exchange_components_empty_registry() {
    let mut entities = EntityRegistry::new();
    let mut components = ComponentRegistry::new();
    let mut archetypes = ArchetypeRegistry::new();

    let entity = entities.spawn(Default::default(), ());

    let error = archetypes
        .get_bundle::<(Mass, Tag)>(&components, entity)
        .expect_err("entity should not have `Mass` and `Tag` components yet");
    assert!(matches!(
        error,
        IncompatibleBundleError::ComponentNotRegistered(_),
    ));

    let tag = Tag;
    archetypes
        .insert_bundle::<(Tag,)>(&mut components, entity, (tag,))
        .expect("archetype of only `Tag` should contain unique component ids");

    let (&tag,) = archetypes
        .get_bundle::<(Tag,)>(&components, entity)
        .expect("entity should have `Tag` component");
    assert_eq!(tag, Tag);

    let error = archetypes
        .get_bundle::<(Mass, Tag)>(&components, entity)
        .expect_err("entity should not have `Mass` and `Tag` components yet");
    assert!(matches!(
        error,
        IncompatibleBundleError::ComponentNotRegistered(_),
    ));

    let mass = Mass { value: 42 };
    let InsertBundleExactError { value, kind, .. } = archetypes
        .insert_bundle_exact::<(Mass, Tag)>(&mut components, entity, (mass, tag))
        .expect_err("entity already has `Tag` component");
    assert_eq!(
        kind,
        AlreadyHasComponentError::new(components.register_component::<Tag>()).into(),
    );
    assert_eq!(value, (mass, tag));

    let component_ids = <(Tag,)>::register_components(&mut components);
    let archetype = archetypes
        .register_archetype_from(&components, component_ids)
        .expect("archetype of only `Tag` should contain unique component ids");
    let storage = archetypes
        .get_archetype_info(archetype)
        .expect("archetype should exist")
        .storage();
    assert!(storage.contains(entity));

    archetypes
        .remove_bundle::<(Tag, Position)>(&mut components, entity)
        .expect("entity should have `Tag` component, existence of `Position` is not important");

    let error = archetypes
        .remove_bundle_exact::<(Tag,)>(&mut components, entity)
        .expect_err("entity should not have `Tag` component");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let storage = archetypes
        .get_archetype_info(archetype)
        .expect("archetype should exist")
        .storage();
    assert!(!storage.contains(entity));
}

#[test]
fn components() {
    let mut entities = EntityRegistry::new();
    let mut components = ComponentRegistry::new();
    let mut archetypes = ArchetypeRegistry::new();

    let entity1 = entities.spawn(Default::default(), ());
    let position = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    archetypes
        .insert_bundle::<(Position, Tag)>(&mut components, entity1, (position, Tag))
        .expect("archetype of `Position` and `Tag` should contain unique component ids");

    let entity2 = entities.spawn(Default::default(), ());
    let mass = Mass { value: 42 };
    archetypes
        .insert_bundle::<(Mass, Tag)>(&mut components, entity2, (mass, Tag))
        .expect("archetype of `Mass` and `Tag` should contain unique component ids");

    let positions = archetypes
        .bundles_mut::<(Position,)>(&components)
        .expect("archetype of just `Position` should exist & contain unique component ids");
    for (entity, (position,)) in positions {
        assert_eq!(entity, entity1);
        position.x -= 1.0;
        position.y -= 2.0;
        position.z -= 3.0;
    }

    let (position,) = archetypes
        .get_bundle::<(Position,)>(&components, entity1)
        .expect("entity should have `Position` component");
    assert_eq!(position.x, 0.0);
    assert_eq!(position.y, 0.0);
    assert_eq!(position.z, 0.0);

    let masses = archetypes
        .bundles::<(Mass,)>(&components)
        .expect("archetype of just `Mass` should exist & contain unique component ids");
    for (entity, (mass,)) in masses {
        assert_eq!(entity, entity2);
        assert_eq!(mass.value, 42);
    }

    let tags = archetypes
        .bundles::<(Tag,)>(&components)
        .expect("archetype of just `Tag` should exist & contain unique component ids");
    for (entity, (&tag,)) in tags.clone() {
        assert!(entity == entity1 || entity == entity2);
        assert_eq!(tag, Tag);
    }
    itertools::assert_equal(
        tags.clone().into_iter().map(|(entity, _)| entity),
        [entity1, entity2].into_iter(),
    );
    itertools::assert_equal(
        tags.into_iter().rev().map(|(entity, _)| entity),
        [entity1, entity2].into_iter().rev(),
    );

    let positions_with_masses = archetypes
        .bundles_mut::<(Position, Mass)>(&components)
        .expect("archetype of `Position` and `Mass` should exist & contain unique component ids");
    assert_eq!(positions_with_masses.into_iter().count(), 0);
}
