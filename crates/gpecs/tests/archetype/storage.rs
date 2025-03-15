use gpecs::{
    archetype::{
        error::{ExclusiveComponentError, IncompatibleBundleValueError, TooFewComponentsError},
        storage::ArchetypeStorage,
    },
    component::{registry::ComponentRegistry, Component},
    entity::registry::EntityRegistry,
};

#[test]
fn storage_unit() {
    let mut components = ComponentRegistry::new();
    let mut storage = ArchetypeStorage::of::<()>(&mut components, ())
        .expect("creation of storage for empty archetype should succeed");
    assert_eq!(storage.entities(), []);

    let mut entities = EntityRegistry::new();
    let entity = entities.spawn();

    let value = storage
        .insert::<()>(&mut components, &(), entity, ())
        .expect("archetype storage should store unit");
    assert_eq!(value, None);
    assert_eq!(storage.entities(), [entity]);

    let refs = storage
        .get::<()>(&mut components, &(), entity)
        .expect("components by given entity should exist");
    assert_eq!(refs, Some(&()));
    assert_eq!(storage.entities(), [entity]);

    let value = storage
        .remove::<()>(&mut components, &(), entity)
        .expect("components by given entity should exist");
    assert_eq!(value, Some(()));
    assert_eq!(storage.entities(), []);
}

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

impl Component for Position {}
impl Component for Mass {}

#[test]
fn storage_tuple() {
    let mut components = ComponentRegistry::new();

    let error = ArchetypeStorage::of::<(Position, Position)>(&mut components, ())
        .expect_err("creation of storage for bundle `(Position, Position)` should fail");
    assert_eq!(
        error.component_id(),
        components.register_component::<Position>(),
    );

    let mut storage = ArchetypeStorage::of::<(Position, Mass)>(&mut components, ())
        .expect("creation of storage for bundle `(Position, Mass)` should succeed");
    assert_eq!(storage.entities(), []);

    let mut entities = EntityRegistry::new();
    let entity = entities.spawn();

    let slices = storage
        .components::<(Position,)>(&mut components, &())
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(slices, ([].as_slice(),));

    let error = storage
        .components::<(Position, Mass, ())>(&mut components, &())
        .expect_err("retrieval of slice of `(Position, Mass, ())` should fail");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<()>()).into(),
    );

    let slices = storage
        .components::<(Mass, Position)>(&mut components, &())
        .expect("retrieval of slice of `(Mass, Position)` should succeed");
    assert_eq!(slices, ([].as_slice(), [].as_slice()));

    let mut position = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let IncompatibleBundleValueError { value, reason, .. } = storage
        .insert::<(Position,)>(&mut components, &(), entity, (position,))
        .expect_err("insertion of just `Position` should fail");
    assert_eq!(value, (position,));
    assert_eq!(reason, TooFewComponentsError::new().into());

    let mut mass = Mass { value: 4 };
    let IncompatibleBundleValueError { value, reason, .. } = storage
        .insert::<(Position, Mass, ())>(&mut components, &(), entity, (position, mass, ()))
        .expect_err("insertion of `Position`, `Mass` and `()` should fail");
    assert_eq!(value, (position, mass, ()));
    assert_eq!(
        reason,
        ExclusiveComponentError::new(components.register_component::<()>()).into(),
    );

    let value = storage
        .insert::<(Mass, Position)>(&mut components, &(), entity, (mass, position))
        .expect("insertion of `Mass` and `Position` should succeed");
    assert_eq!(value, None);
    assert_eq!(storage.entities(), [entity]);

    let refs = storage
        .get::<(Position,)>(&mut components, &(), entity)
        .expect("retrieval of just `Position` should succeed");
    assert_eq!(refs, Some((&position,)));
    assert_eq!(storage.entities(), [entity]);

    let error = storage
        .get::<(Position, Mass, ())>(&mut components, &(), entity)
        .expect_err("retrieval of `Position`, `Mass` and `()` should fail");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<()>()).into(),
    );

    let refs = storage
        .get::<(Mass, Position)>(&mut components, &(), entity)
        .expect("retrieval of `Mass` and `Position` should succeed");
    assert_eq!(refs, Some((&mass, &position)));
    assert_eq!(storage.entities(), [entity]);

    let refs_mut = storage
        .get_mut::<(Position,)>(&mut components, &(), entity)
        .expect("retrieval of just `Position` should succeed");
    assert_eq!(refs_mut, Some((&mut position,)));
    assert_eq!(storage.entities(), [entity]);

    let error = storage
        .get_mut::<(Position, Mass, ())>(&mut components, &(), entity)
        .expect_err("retrieval of `Position`, `Mass` and `()` should fail");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<()>()).into(),
    );

    let refs_mut = storage
        .get_mut::<(Mass, Position)>(&mut components, &(), entity)
        .expect("retrieval of `Mass` and `Position` should succeed");
    assert_eq!(refs_mut, Some((&mut mass, &mut position)));
    assert_eq!(storage.entities(), [entity]);

    let slices = storage
        .components::<(Position,)>(&mut components, &())
        .expect("retrieval of slice of just `Position` should succeed");
    assert_eq!(slices, ([position].as_slice(),));

    let error = storage
        .components::<(Position, Mass, ())>(&mut components, &())
        .expect_err("retrieval of slice of `(Position, Mass, ())` should fail");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<()>()).into(),
    );

    let slices = storage
        .components::<(Mass, Position)>(&mut components, &())
        .expect("retrieval of slice of `(Mass, Position)` should succeed");
    assert_eq!(slices, ([mass].as_slice(), [position].as_slice()));

    let error = storage
        .remove::<(Position,)>(&mut components, &(), entity)
        .expect_err("removal of just `Position` should fail");
    assert_eq!(error, TooFewComponentsError::new().into());

    let error = storage
        .remove::<(Position, Mass, ())>(&mut components, &(), entity)
        .expect_err("removal of `Position`, `Mass` and `()` should fail");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<()>()).into(),
    );

    let value = storage
        .remove::<(Mass, Position)>(&mut components, &(), entity)
        .expect("removal of `Mass` and `Position` should succeed");
    assert_eq!(value, Some((mass, position)));
    assert_eq!(storage.entities(), []);
}
