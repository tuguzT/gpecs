use gpecs::{
    archetype::{
        error::{
            ExclusiveComponentError, IncompatibleBundleError, IncompatibleBundleValueError,
            TooFewComponentsError,
        },
        storage::ArchetypeStorage,
    },
    bundle::Bundle,
    component::{registry::ComponentRegistry, Component},
    entity::registry::EntityRegistry,
    world::registry::WorldRegistry,
};

#[derive(Debug, PartialEq, Clone, Copy)]
struct Tag;

impl Component for Tag {}

#[test]
fn storage_tag() {
    let mut components = ComponentRegistry::new();
    let mut storage = ArchetypeStorage::of::<(Tag,)>(&mut components)
        .expect("creation of storage for tag archetype should succeed");
    assert_eq!(storage.entities(), []);

    let component_ids = <(Tag,)>::register_components(&mut components);
    assert!(storage.component_ids().eq(component_ids));

    let storage_from_ids = ArchetypeStorage::new(&components, component_ids)
        .expect("tag component should be already registered");
    assert!(storage_from_ids.component_ids().eq(storage.component_ids()));

    let mut worlds = WorldRegistry::new();
    let world = worlds.spawn();

    let mut entities = EntityRegistry::new();
    let entity = entities.spawn(world, ());

    let value = storage
        .insert::<(Tag,)>(&mut components, entity, (Tag,))
        .expect("archetype storage should store tag");
    assert_eq!(value, None);
    assert_eq!(storage.entities(), [entity]);

    let refs = storage
        .get::<(Tag,)>(&mut components, entity)
        .expect("components by given entity should exist");
    assert_eq!(refs, Some((&Tag,)));
    assert_eq!(storage.entities(), [entity]);

    let value = storage
        .remove::<(Tag,)>(&mut components, entity)
        .expect("components by given entity should exist");
    assert_eq!(value, Some((Tag,)));
    assert_eq!(storage.entities(), []);
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, PartialEq, Clone)]
struct Name {
    value: String,
}

impl Component for Position {}
impl Component for Name {}

#[test]
fn storage_tuple() {
    let mut components = ComponentRegistry::new();

    let error = ArchetypeStorage::of::<(Position, Position)>(&mut components)
        .expect_err("creation of storage for bundle `(Position, Position)` should fail");
    assert_eq!(
        error.component_id(),
        components.register_component::<Position>(),
    );

    let mut storage = ArchetypeStorage::of::<(Position, Name)>(&mut components)
        .expect("creation of storage for bundle `(Position, Name)` should succeed");
    assert_eq!(storage.entities(), []);

    let component_ids = <(Position, Name)>::register_components(&mut components);
    assert!(storage.component_ids().eq(component_ids));

    let storage_from_ids = ArchetypeStorage::new(&components, component_ids)
        .expect("`Position` and `Name` components should be already registered");
    assert!(storage_from_ids.component_ids().eq(storage.component_ids()));

    let mut worlds = WorldRegistry::new();
    let world = worlds.spawn();

    let mut entities = EntityRegistry::new();
    let entity = entities.spawn(world, ());

    let slices = storage
        .components::<(Position,)>(&mut components)
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(slices, ([].as_slice(), ([].as_slice(),)));

    let error = storage
        .components::<(Position, Name, Tag)>(&mut components)
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert!(matches!(
        error,
        IncompatibleBundleError::ComponentNotRegistered(_),
    ));

    components.register_component::<Tag>();
    let error = storage
        .components::<(Position, Name, Tag)>(&mut components)
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Tag>()).into(),
    );

    let slices = storage
        .components::<(Name, Position)>(&mut components)
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(slices, ([].as_slice(), ([].as_slice(), [].as_slice())));

    let slices = storage
        .components_mut::<(Position,)>(&mut components)
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(slices, ([].as_slice(), ([].as_mut_slice(),)));

    let error = storage
        .components_mut::<(Position, Name, Tag)>(&mut components)
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Tag>()).into(),
    );

    let slices = storage
        .components_mut::<(Name, Position)>(&mut components)
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(
        slices,
        ([].as_slice(), ([].as_mut_slice(), [].as_mut_slice())),
    );

    let mut position = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let IncompatibleBundleValueError { value, reason, .. } = storage
        .insert::<(Position,)>(&mut components, entity, (position,))
        .expect_err("insertion of just `Position` should fail");
    assert_eq!(value, (position,));
    assert_eq!(reason, TooFewComponentsError::new().into());

    let mut name = Name {
        value: "Hello, World!".to_owned(),
    };
    let tag = Tag;
    let IncompatibleBundleValueError { value, reason, .. } = storage
        .insert::<(Position, Name, Tag)>(&mut components, entity, (position, name.clone(), tag))
        .expect_err("insertion of `Position`, `Name` and `Tag` should fail");
    assert_eq!(value, (position, name.clone(), tag));
    assert_eq!(
        reason,
        ExclusiveComponentError::new(components.register_component::<Tag>()).into(),
    );

    let value = storage
        .insert::<(Name, Position)>(&mut components, entity, (name.clone(), position))
        .expect("insertion of `Name` and `Position` should succeed");
    assert_eq!(value, None);
    assert_eq!(storage.entities(), [entity]);
    assert!(storage.contains(entity));

    let refs = storage
        .get::<(Position,)>(&mut components, entity)
        .expect("retrieval of just `Position` should succeed");
    assert_eq!(refs, Some((&position,)));
    assert_eq!(storage.entities(), [entity]);
    assert!(storage.contains(entity));

    let error = storage
        .get::<(Position, Name, Tag)>(&mut components, entity)
        .expect_err("retrieval of `Position`, `Name` and `Tag` should fail");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Tag>()).into(),
    );

    let refs = storage
        .get::<(Name, Position)>(&mut components, entity)
        .expect("retrieval of `Name` and `Position` should succeed");
    assert_eq!(refs, Some((&name, &position)));
    assert_eq!(storage.entities(), [entity]);
    assert!(storage.contains(entity));

    let refs_mut = storage
        .get_mut::<(Position,)>(&mut components, entity)
        .expect("retrieval of just `Position` should succeed");
    assert_eq!(refs_mut, Some((&mut position,)));
    assert_eq!(storage.entities(), [entity]);
    assert!(storage.contains(entity));

    let error = storage
        .get_mut::<(Position, Name, Tag)>(&mut components, entity)
        .expect_err("retrieval of `Position`, `Name` and `Tag` should fail");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Tag>()).into(),
    );

    let refs_mut = storage
        .get_mut::<(Name, Position)>(&mut components, entity)
        .expect("retrieval of `Name` and `Position` should succeed");
    assert_eq!(refs_mut, Some((&mut name, &mut position)));
    assert_eq!(storage.entities(), [entity]);
    assert!(storage.contains(entity));

    let slices = storage
        .components::<(Position,)>(&mut components)
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(slices, ([entity].as_slice(), ([position].as_slice(),)));

    let error = storage
        .components::<(Position, Name, Tag)>(&mut components)
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Tag>()).into(),
    );

    let slices = storage
        .components::<(Name, Position)>(&mut components)
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(
        slices,
        (
            [entity].as_slice(),
            ([name.clone()].as_slice(), [position].as_slice()),
        ),
    );

    let slices = storage
        .components_mut::<(Position,)>(&mut components)
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(slices, ([entity].as_slice(), ([position].as_mut_slice(),)));

    let error = storage
        .components_mut::<(Position, Name, Tag)>(&mut components)
        .expect_err("retrieval of slice of `(Position, Name, Tag)` should fail");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Tag>()).into(),
    );

    let slices = storage
        .components_mut::<(Name, Position)>(&mut components)
        .expect("retrieval of slice of `(Name, Position)` should succeed");
    assert_eq!(
        slices,
        (
            [entity].as_slice(),
            ([name.clone()].as_mut_slice(), [position].as_mut_slice()),
        ),
    );

    let error = storage
        .remove::<(Position,)>(&mut components, entity)
        .expect_err("removal of just `Position` should fail");
    assert_eq!(error, TooFewComponentsError::new().into());

    let error = storage
        .remove::<(Position, Name, Tag)>(&mut components, entity)
        .expect_err("removal of `Position`, `Name` and `Tag` should fail");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Tag>()).into(),
    );

    let value = storage
        .remove::<(Name, Position)>(&mut components, entity)
        .expect("removal of `Name` and `Position` should succeed");
    assert_eq!(value, Some((name.clone(), position)));
    assert_eq!(storage.entities(), []);
    assert!(!storage.contains(entity));

    let value = storage
        .insert::<(Name, Position)>(&mut components, entity, (name.clone(), position))
        .expect("insertion of `Name` and `Position` should succeed");
    assert_eq!(value, None);
    assert_eq!(storage.entities(), [entity]);
    assert!(storage.contains(entity));
}
