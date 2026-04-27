#![cfg(feature = "alloc")]

use std::mem::MaybeUninit;

use gpecs_archetype::{
    bundle::{
        Bundle,
        erased::{self, traits::MustDrop},
    },
    erased::error::{IncompatibleArchetypeError, MissingComponentError, TooFewComponentsError},
    storage::{self, error::IncompatibleBundleValueError},
};
use gpecs_entity::registry::EntityRegistry;
use gpecs_soa_erased::{ptr::slice::CoreSliceItemPtrs, storage::BoxedAlignedUninitStorage};
use gpecs_world::registry::WorldRegistry;

use crate::common::{Components, ErasedDropMeta, Name, Position, Tag};

type ArchetypeStorage = storage::ArchetypeStorage<ErasedBundle<ErasedDropMeta>>;
type ErasedBundle<Meta> = erased::ErasedBundle<Meta, MustDrop, Storage, SlicePtrs>;

type Storage = BoxedAlignedUninitStorage;
type SlicePtrs = CoreSliceItemPtrs<MaybeUninit<u8>>;

#[test]
fn storage_tag() {
    let mut components = Components::new();
    let mut storage = ArchetypeStorage::register::<(Tag,), _, _>(&mut components)
        .expect("creation of storage for tag archetype should succeed");
    assert_eq!(storage.as_entities(), []);

    let component_ids = <(Tag,)>::register_components(&mut components)
        .expect("archetype of only `Tag` should have unique components");
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
        .insert_bundle::<(Tag,)>(&components.as_view(), entity, (Tag,))
        .expect("archetype storage should store tag");
    assert_eq!(value, None);
    assert_eq!(storage.as_entities(), [entity]);

    let refs = storage
        .get_bundle::<(Tag,)>(&components.as_view(), entity)
        .expect("components by given entity should exist");
    assert_eq!(refs, Some((&Tag,)));
    assert_eq!(storage.as_entities(), [entity]);

    let value = storage
        .remove_bundle::<(Tag,)>(&components.as_view(), entity)
        .expect("components by given entity should exist");
    assert_eq!(value, Some((Tag,)));
    assert_eq!(storage.as_entities(), []);
}

#[test]
fn storage_tag_erased() {
    let mut components = Components::new();
    let mut storage = ArchetypeStorage::register::<(Tag,), _, _>(&mut components)
        .expect("creation of storage for tag archetype should succeed");
    assert_eq!(storage.as_entities(), []);

    let component_ids = <(Tag,)>::register_components(&mut components)
        .expect("archetype of only `Tag` should have unique components");
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

    {
        let bundle = ErasedBundle::from_bundle(&mut components, (Tag,))
            .expect("bundle should be created successfully");
        let bundle = storage
            .insert(entity, bundle)
            .expect("archetype storage should store tag");
        assert!(bundle.is_none());
    }
    assert_eq!(storage.as_entities(), [entity]);

    let refs = storage
        .get_bundle::<(Tag,)>(&components.as_view(), entity)
        .expect("components by given entity should exist");
    assert_eq!(refs, Some((&Tag,)));
    assert_eq!(storage.as_entities(), [entity]);

    let bundle = storage
        .remove(entity)
        .expect("components by given entity should exist");
    let value = bundle
        .downcast::<(Tag,)>(&components.as_view())
        .expect("bundle should contain only `Tag` component");
    assert_eq!(value, (Tag,));
    assert_eq!(storage.as_entities(), []);
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
    assert_eq!(storage.as_entities(), []);

    let component_ids = <(Position, Name)>::register_components(&mut components)
        .expect("archetype of `Position` & `Name` should have unique components");
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

    let (positions,) = storage
        .as_bundles::<(Position,)>(&components.as_view())
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(positions, []);

    let error = storage
        .as_bundles::<(Position, Name, Tag)>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert!(matches!(
        error,
        IncompatibleArchetypeError::ComponentNotRegistered(_),
    ));

    components.register_component::<Tag>();
    let error = storage
        .as_bundles::<(Position, Name, Tag)>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let (names, positions) = storage
        .as_bundles::<(Name, Position)>(&components.as_view())
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(names, []);
    assert_eq!(positions, []);

    let (positions,) = storage
        .as_mut_bundles::<(Position,)>(&components.as_view())
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(positions, []);

    let error = storage
        .as_mut_bundles::<(Position, Name, Tag)>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let (names, positions) = storage
        .as_mut_bundles::<(Name, Position)>(&components.as_view())
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(names, []);
    assert_eq!(positions, []);

    let mut position = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        padding: Default::default(),
    };
    let IncompatibleBundleValueError { value, source, .. } = storage
        .insert_bundle::<(Position,)>(&components.as_view(), entity, (position,))
        .expect_err("insertion of just `Position` should fail");
    assert_eq!(value, (position,));
    assert_eq!(source, TooFewComponentsError::new().into());

    let mut name = Name {
        value: "Hello, World!".to_owned(),
    };
    let tag = Tag;
    let IncompatibleBundleValueError { value, source, .. } = storage
        .insert_bundle(&components.as_view(), entity, (position, name.clone(), tag))
        .expect_err("insertion of `Position`, `Name` and `Tag` should fail");
    assert_eq!(value, (position, name.clone(), tag));
    assert_eq!(
        source,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let value = storage
        .insert_bundle::<(Name, Position)>(&components.as_view(), entity, (name.clone(), position))
        .expect("insertion of `Name` and `Position` should succeed");
    assert_eq!(value, None);
    assert_eq!(storage.as_entities(), [entity]);
    assert!(storage.contains(entity));

    let refs = storage
        .get_bundle::<(Position,)>(&components.as_view(), entity)
        .expect("retrieval of just `Position` should succeed");
    assert_eq!(refs, Some((&position,)));
    assert_eq!(storage.as_entities(), [entity]);
    assert!(storage.contains(entity));

    let error = storage
        .get_bundle::<(Position, Name, Tag)>(&components.as_view(), entity)
        .expect_err("retrieval of `Position`, `Name` and `Tag` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let refs = storage
        .get_bundle::<(Name, Position)>(&components.as_view(), entity)
        .expect("retrieval of `Name` and `Position` should succeed");
    assert_eq!(refs, Some((&name, &position)));
    assert_eq!(storage.as_entities(), [entity]);
    assert!(storage.contains(entity));

    let refs_mut = storage
        .get_bundle_mut::<(Position,)>(&components.as_view(), entity)
        .expect("retrieval of just `Position` should succeed");
    assert_eq!(refs_mut, Some((&mut position,)));
    assert_eq!(storage.as_entities(), [entity]);
    assert!(storage.contains(entity));

    let error = storage
        .get_bundle_mut::<(Position, Name, Tag)>(&components.as_view(), entity)
        .expect_err("retrieval of `Position`, `Name` and `Tag` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let refs_mut = storage
        .get_bundle_mut::<(Name, Position)>(&components.as_view(), entity)
        .expect("retrieval of `Name` and `Position` should succeed");
    assert_eq!(refs_mut, Some((&mut name, &mut position)));
    assert_eq!(storage.as_entities(), [entity]);
    assert!(storage.contains(entity));

    let (entities, (positions,), _) = storage
        .as_bundles_with_archetype::<(Position,)>(&components.as_view())
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(entities, [entity]);
    assert_eq!(positions, [position]);

    let error = storage
        .as_bundles::<(Position, Name, Tag)>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let (entities, (names, positions), _) = storage
        .as_bundles_with_archetype::<(Name, Position)>(&components.as_view())
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(entities, [entity]);
    assert_eq!(names, [name.clone()]);
    assert_eq!(positions, [position]);

    let (entities, (positions,), _) = storage
        .as_mut_bundles_with_archetype::<(Position,)>(&components.as_view())
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(entities, [entity]);
    assert_eq!(positions, [position]);

    let error = storage
        .as_mut_bundles::<(Position, Name, Tag)>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let (entities, (names, positions), _) = storage
        .as_mut_bundles_with_archetype::<(Name, Position)>(&components.as_view())
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(entities, [entity]);
    assert_eq!(names, [name.clone()]);
    assert_eq!(positions, [position]);

    let error = storage
        .remove_bundle::<(Position,)>(&components.as_view(), entity)
        .expect_err("removal of just `Position` should fail");
    assert_eq!(error, TooFewComponentsError::new().into());

    let error = storage
        .remove_bundle::<(Position, Name, Tag)>(&components.as_view(), entity)
        .expect_err("removal of `Position`, `Name` and `Tag` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let value = storage
        .remove_bundle::<(Name, Position)>(&components.as_view(), entity)
        .expect("removal of `Name` and `Position` should succeed");
    assert_eq!(value, Some((name.clone(), position)));
    assert_eq!(storage.as_entities(), []);
    assert!(!storage.contains(entity));

    let value = storage
        .insert_bundle(&components.as_view(), entity, (name.clone(), position))
        .expect("insertion of `Name` and `Position` should succeed");
    assert_eq!(value, None);
    assert_eq!(storage.as_entities(), [entity]);
    assert!(storage.contains(entity));
}

#[test]
fn storage_tuple_erased() {
    let mut components = Components::new();

    let error = ArchetypeStorage::register::<(Position, Position), _, _>(&mut components)
        .expect_err("creation of storage for bundle `(Position, Position)` should fail");
    assert_eq!(
        error.component_id(),
        components.register_component::<Position>(),
    );

    let mut storage = ArchetypeStorage::register::<(Position, Name), _, _>(&mut components)
        .expect("creation of storage for bundle `(Position, Name)` should succeed");
    assert_eq!(storage.as_entities(), []);

    let component_ids = <(Position, Name)>::register_components(&mut components)
        .expect("archetype of `Position` & `Name` should have unique components");
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

    let (positions,) = storage
        .as_bundles::<(Position,)>(&components.as_view())
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(positions, []);

    let error = storage
        .as_bundles::<(Position, Name, Tag)>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert!(matches!(
        error,
        IncompatibleArchetypeError::ComponentNotRegistered(_),
    ));

    components.register_component::<Tag>();
    let error = storage
        .as_bundles::<(Position, Name, Tag)>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let (names, positions) = storage
        .as_bundles::<(Name, Position)>(&components.as_view())
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(names, []);
    assert_eq!(positions, []);

    let (positions,) = storage
        .as_mut_bundles::<(Position,)>(&components.as_view())
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(positions, []);

    let error = storage
        .as_mut_bundles::<(Position, Name, Tag)>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let (names, positions) = storage
        .as_mut_bundles::<(Name, Position)>(&components.as_view())
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(names, []);
    assert_eq!(positions, []);

    let mut position = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        padding: Default::default(),
    };
    let bundle = ErasedBundle::from_bundle(&mut components, (position,))
        .expect("bundle should be created successfully");
    let error = storage
        .insert(entity, bundle)
        .expect_err("insertion of just `Position` should fail");
    assert_eq!(error, TooFewComponentsError::new().into());

    let mut name = Name {
        value: "Hello, World!".to_owned(),
    };
    let tag = Tag;
    let bundle = ErasedBundle::from_bundle(&mut components, (position, name.clone(), tag))
        .expect("bundle should be created successfully");
    let error = storage
        .insert(entity, bundle)
        .expect_err("insertion of `Position`, `Name` and `Tag` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    {
        let bundle = ErasedBundle::from_bundle(&mut components, (name.clone(), position))
            .expect("bundle should be created successfully");
        let bundle = storage
            .insert(entity, bundle)
            .expect("insertion of `Name` and `Position` should succeed");
        assert!(bundle.is_none());
    }
    assert_eq!(storage.as_entities(), [entity]);
    assert!(storage.contains(entity));

    let refs = storage
        .get_bundle::<(Position,)>(&components.as_view(), entity)
        .expect("retrieval of just `Position` should succeed");
    assert_eq!(refs, Some((&position,)));
    assert_eq!(storage.as_entities(), [entity]);
    assert!(storage.contains(entity));

    let error = storage
        .get_bundle::<(Position, Name, Tag)>(&components.as_view(), entity)
        .expect_err("retrieval of `Position`, `Name` and `Tag` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let refs = storage
        .get_bundle::<(Name, Position)>(&components.as_view(), entity)
        .expect("retrieval of `Name` and `Position` should succeed");
    assert_eq!(refs, Some((&name, &position)));
    assert_eq!(storage.as_entities(), [entity]);
    assert!(storage.contains(entity));

    let refs_mut = storage
        .get_bundle_mut::<(Position,)>(&components.as_view(), entity)
        .expect("retrieval of just `Position` should succeed");
    assert_eq!(refs_mut, Some((&mut position,)));
    assert_eq!(storage.as_entities(), [entity]);
    assert!(storage.contains(entity));

    let error = storage
        .get_bundle_mut::<(Position, Name, Tag)>(&components.as_view(), entity)
        .expect_err("retrieval of `Position`, `Name` and `Tag` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let refs_mut = storage
        .get_bundle_mut::<(Name, Position)>(&components.as_view(), entity)
        .expect("retrieval of `Name` and `Position` should succeed");
    assert_eq!(refs_mut, Some((&mut name, &mut position)));
    assert_eq!(storage.as_entities(), [entity]);
    assert!(storage.contains(entity));

    let (entities, (positions,), _) = storage
        .as_bundles_with_archetype::<(Position,)>(&components.as_view())
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(entities, [entity]);
    assert_eq!(positions, [position]);

    let error = storage
        .as_bundles::<(Position, Name, Tag)>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let (entities, (names, positions), _) = storage
        .as_bundles_with_archetype::<(Name, Position)>(&components.as_view())
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(entities, [entity]);
    assert_eq!(names, [name.clone()]);
    assert_eq!(positions, [position]);

    let (entities, (positions,), _) = storage
        .as_mut_bundles_with_archetype::<(Position,)>(&components.as_view())
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(entities, [entity]);
    assert_eq!(positions, [position]);

    let error = storage
        .as_mut_bundles::<(Position, Name, Tag)>(&components.as_view())
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let (entities, (names, positions), _) = storage
        .as_mut_bundles_with_archetype::<(Name, Position)>(&components.as_view())
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(entities, [entity]);
    assert_eq!(names, [name.clone()]);
    assert_eq!(positions, [position]);

    let error = storage
        .remove_bundle::<(Position,)>(&components.as_view(), entity)
        .expect_err("removal of just `Position` should fail");
    assert_eq!(error, TooFewComponentsError::new().into());

    let error = storage
        .remove_bundle::<(Position, Name, Tag)>(&components.as_view(), entity)
        .expect_err("removal of `Position`, `Name` and `Tag` should fail");
    assert_eq!(
        error,
        MissingComponentError::new(components.register_component::<Tag>()).into(),
    );

    let bundle = storage
        .remove(entity)
        .expect("components by given entity should exist");
    let bundle = bundle
        .downcast::<(Name, Position)>(&components.as_view())
        .expect("bundle should contain `Name` and `Position` components");
    assert_eq!(bundle, (name.clone(), position));
    assert_eq!(storage.as_entities(), []);
    assert!(!storage.contains(entity));

    {
        let bundle = ErasedBundle::from_bundle(&mut components, (name.clone(), position))
            .expect("bundle should be created successfully");
        let bundle = storage
            .insert(entity, bundle)
            .expect("insertion of `Name` and `Position` should succeed");
        assert!(bundle.is_none());
    }
    assert_eq!(storage.as_entities(), [entity]);
    assert!(storage.contains(entity));
}
