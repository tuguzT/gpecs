use gpecs::{
    archetype::{
        erased::error::{MissingComponentError, TooFewComponentsError},
        error::IncompatibleBundleValueError,
        storage::ArchetypeStorage,
    },
    bundle::{NewBundle, erased::error::DowncastErrorKind},
    context::Components,
    entity::registry::EntityRegistry,
    world::registry::WorldRegistry,
};

use crate::common::{Name, Position, Tag};

#[test]
fn storage_tag() {
    let mut components = Components::new();
    let mut storage = ArchetypeStorage::register::<(Tag,), _, _>(&mut components)
        .expect("creation of storage for tag archetype should succeed");
    assert_eq!(storage.entities(), []);

    let component_ids = <(Tag,)>::register_components(&mut components);
    itertools::assert_equal(storage.archetype().component_ids(), component_ids);

    let storage_from_ids = ArchetypeStorage::new(&components.as_view(), component_ids)
        .expect("tag component should be already registered");
    itertools::assert_equal(
        storage_from_ids.archetype().component_ids(),
        storage.archetype().component_ids(),
    );

    let mut worlds = WorldRegistry::new();
    let world = worlds.spawn();

    let mut entities = EntityRegistry::new();
    let entity = entities.spawn(world, ());

    let value = storage
        .insert_bundle::<(Tag,), _>(&components.as_view(), entity, (Tag,))
        .expect("archetype storage should store tag");
    assert_eq!(value, None);
    assert_eq!(storage.entities(), [entity]);

    let refs = storage
        .get_bundle::<(Tag,), _>(&components.as_view(), entity)
        .expect("components by given entity should exist");
    assert_eq!(refs, Some((&Tag,)));
    assert_eq!(storage.entities(), [entity]);

    let value = storage
        .remove_bundle::<(Tag,), _>(&components.as_view(), entity)
        .expect("components by given entity should exist");
    assert_eq!(value, Some((Tag,)));
    assert_eq!(storage.entities(), []);
}

#[test]
fn storage_tuple() {
    let mut components = Components::new();

    let error = ArchetypeStorage::register::<(Position, Position), _, _>(&mut components)
        .expect_err("creation of storage for bundle `(Position, Position)` should fail");
    assert_eq!(
        error.component_id(),
        components.register_component::<Position>(),
    );

    let mut storage = ArchetypeStorage::register::<(Position, Name), _, _>(&mut components)
        .expect("creation of storage for bundle `(Position, Name)` should succeed");
    assert_eq!(storage.entities(), []);

    let component_ids = <(Position, Name)>::register_components(&mut components);
    itertools::assert_equal(storage.archetype().component_ids(), component_ids);

    let storage_from_ids = ArchetypeStorage::new(&components.as_view(), component_ids)
        .expect("`Position` and `Name` components should be already registered");
    itertools::assert_equal(
        storage_from_ids.archetype().component_ids(),
        storage.archetype().component_ids(),
    );

    let mut worlds = WorldRegistry::new();
    let world = worlds.spawn();

    let mut entities = EntityRegistry::new();
    let entity = entities.spawn(world, ());

    let slices = storage
        .bundles::<(Position,), _>(&components.as_view())
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(slices, ([].as_slice(), ([].as_slice(),)));

    let error = storage
        .bundles::<(Position, Name, Tag), _>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert!(matches!(
        error,
        DowncastErrorKind::ComponentNotRegistered(_),
    ));

    components.register_component::<Tag>();
    let error = storage
        .bundles::<(Position, Name, Tag), _>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let slices = storage
        .bundles::<(Name, Position), _>(&components.as_view())
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(slices, ([].as_slice(), ([].as_slice(), [].as_slice())));

    let slices = storage
        .bundles_mut::<(Position,), _>(&components.as_view())
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(slices, ([].as_slice(), ([].as_mut_slice(),)));

    let error = storage
        .bundles_mut::<(Position, Name, Tag), _>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let slices = storage
        .bundles_mut::<(Name, Position), _>(&components.as_view())
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(
        slices,
        ([].as_slice(), ([].as_mut_slice(), [].as_mut_slice())),
    );

    let mut position = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        padding: Default::default(),
    };
    let IncompatibleBundleValueError { value, source, .. } = storage
        .insert_bundle::<(Position,), _>(&components.as_view(), entity, (position,))
        .expect_err("insertion of just `Position` should fail");
    assert_eq!(value, (position,));
    assert_eq!(source, TooFewComponentsError::new().into());

    let mut name = Name {
        value: "Hello, World!".to_owned(),
    };
    let tag = Tag;
    let IncompatibleBundleValueError { value, source, .. } = storage
        .insert_bundle::<(Position, Name, Tag), _>(
            &components.as_view(),
            entity,
            (position, name.clone(), tag),
        )
        .expect_err("insertion of `Position`, `Name` and `Tag` should fail");
    assert_eq!(value, (position, name.clone(), tag));
    assert_eq!(
        source,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let value = storage
        .insert_bundle::<(Name, Position), _>(
            &components.as_view(),
            entity,
            (name.clone(), position),
        )
        .expect("insertion of `Name` and `Position` should succeed");
    assert_eq!(value, None);
    assert_eq!(storage.entities(), [entity]);
    assert!(storage.contains(entity));

    let refs = storage
        .get_bundle::<(Position,), _>(&components.as_view(), entity)
        .expect("retrieval of just `Position` should succeed");
    assert_eq!(refs, Some((&position,)));
    assert_eq!(storage.entities(), [entity]);
    assert!(storage.contains(entity));

    let error = storage
        .get_bundle::<(Position, Name, Tag), _>(&components.as_view(), entity)
        .expect_err("retrieval of `Position`, `Name` and `Tag` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let refs = storage
        .get_bundle::<(Name, Position), _>(&components.as_view(), entity)
        .expect("retrieval of `Name` and `Position` should succeed");
    assert_eq!(refs, Some((&name, &position)));
    assert_eq!(storage.entities(), [entity]);
    assert!(storage.contains(entity));

    let refs_mut = storage
        .get_bundle_mut::<(Position,), _>(&components.as_view(), entity)
        .expect("retrieval of just `Position` should succeed");
    assert_eq!(refs_mut, Some((&mut position,)));
    assert_eq!(storage.entities(), [entity]);
    assert!(storage.contains(entity));

    let error = storage
        .get_bundle_mut::<(Position, Name, Tag), _>(&components.as_view(), entity)
        .expect_err("retrieval of `Position`, `Name` and `Tag` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let refs_mut = storage
        .get_bundle_mut::<(Name, Position), _>(&components.as_view(), entity)
        .expect("retrieval of `Name` and `Position` should succeed");
    assert_eq!(refs_mut, Some((&mut name, &mut position)));
    assert_eq!(storage.entities(), [entity]);
    assert!(storage.contains(entity));

    let slices = storage
        .bundles::<(Position,), _>(&components.as_view())
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(slices, ([entity].as_slice(), ([position].as_slice(),)));

    let error = storage
        .bundles::<(Position, Name, Tag), _>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let slices = storage
        .bundles::<(Name, Position), _>(&components.as_view())
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(
        slices,
        (
            [entity].as_slice(),
            ([name.clone()].as_slice(), [position].as_slice()),
        ),
    );

    let slices = storage
        .bundles_mut::<(Position,), _>(&components.as_view())
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(slices, ([entity].as_slice(), ([position].as_mut_slice(),)));

    let error = storage
        .bundles_mut::<(Position, Name, Tag), _>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let slices = storage
        .bundles_mut::<(Name, Position), _>(&components.as_view())
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(
        slices,
        (
            [entity].as_slice(),
            ([name.clone()].as_mut_slice(), [position].as_mut_slice()),
        ),
    );

    let error = storage
        .remove_bundle::<(Position,), _>(&components.as_view(), entity)
        .expect_err("removal of just `Position` should fail");
    assert_eq!(error, TooFewComponentsError::new().into());

    let error = storage
        .remove_bundle::<(Position, Name, Tag), _>(&components.as_view(), entity)
        .expect_err("removal of `Position`, `Name` and `Tag` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let value = storage
        .remove_bundle::<(Name, Position), _>(&components.as_view(), entity)
        .expect("removal of `Name` and `Position` should succeed");
    assert_eq!(value, Some((name.clone(), position)));
    assert_eq!(storage.entities(), []);
    assert!(!storage.contains(entity));

    let value = storage
        .insert_bundle::<(Name, Position), _>(
            &components.as_view(),
            entity,
            (name.clone(), position),
        )
        .expect("insertion of `Name` and `Position` should succeed");
    assert_eq!(value, None);
    assert_eq!(storage.entities(), [entity]);
    assert!(storage.contains(entity));
}
