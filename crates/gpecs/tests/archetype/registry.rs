use gpecs::{
    archetype::{
        error::{ExclusiveComponentError, IncompatibleBundleError, InsertBundleError},
        registry::ArchetypeRegistry,
    },
    bundle::{
        error::{ComponentNotRegisteredError, DuplicateComponentError},
        Bundle,
    },
    component::{registry::ComponentRegistry, Component},
    entity::registry::EntityRegistry,
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
        .register_archetype::<(Position, Mass, Tag)>(&mut components, &())
        .expect("archetype of `Position`, `Mass` and `Tag` should contain unique component ids");
    assert!(archetypes.len() > 1);
    assert_eq!(
        archetypes
            .archetype_id::<(Position, Mass, Tag)>(&mut components, &())
            .expect("archetype of `Position`, `Mass` and `Tag` should contain unique component ids")
            .expect("archetype of `Position`, `Mass` and `Tag` should be already registered"),
        id,
    );
    assert!(archetypes.archetype_ids().any(|item| item == id));

    let same_id = archetypes
        .register_archetype::<(Mass, Tag, Position)>(&mut components, &())
        .expect("archetype of `Position`, `Mass` and `Tag` should contain unique component ids");
    assert_eq!(same_id, id);

    let component_ids = <(Mass, Tag, Position)>::register_components(&(), &mut components)
        .expect("archetype of `Position`, `Mass` and `Tag` should contain unique component ids");
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

    let component_ids = <(Position,)>::register_components(&(), &mut components)
        .expect("archetype of only `Position` should contain unique component ids");
    assert_ne!(
        archetypes
            .archetype_id_from(component_ids)
            .expect("archetype of only `Position` should contain unique component ids")
            .expect("archetype of only `Position` should be already registered"),
        id,
    );

    let component_ids = <(Mass,)>::register_components(&(), &mut components)
        .expect("archetype of only `Mass` should contain unique component ids");
    assert_ne!(
        archetypes
            .archetype_id_from(component_ids)
            .expect("archetype of only `Mass` should contain unique component ids")
            .expect("archetype of only `Mass` should be already registered"),
        id,
    );

    let component_ids = <(Tag,)>::register_components(&(), &mut components)
        .expect("archetype of only `Tag` should contain unique component ids");
    assert_ne!(
        archetypes
            .archetype_id_from(component_ids)
            .expect("archetype of only `Tag` should contain unique component ids")
            .expect("archetype of only `Tag` should be already registered"),
        id,
    );

    let component_ids = <(Mass, Tag)>::register_components(&(), &mut components)
        .expect("archetype of `Tag` and `Mass` should contain unique component ids");
    assert_ne!(
        archetypes
            .archetype_id_from(component_ids)
            .expect("archetype of `Tag` and `Mass` should contain unique component ids")
            .expect("archetype of `Tag` and `Mass` should be already registered"),
        id,
    );

    let new_id = archetypes
        .register_archetype::<(Mass, Tag)>(&mut components, &())
        .expect("archetype of `Tag` and `Mass` should contain unique component ids");
    assert_ne!(new_id, id);
    assert_eq!(
        archetypes
            .archetype_id::<(Mass, Tag)>(&mut components, &())
            .expect("archetype of `Tag` and `Mass` should contain unique component ids")
            .expect("archetype of `Tag` and `Mass` should be already registered"),
        new_id,
    );
    assert!(archetypes.archetype_ids().any(|item| item == new_id));
    let id = new_id;

    let component_ids = <(Mass, Tag)>::register_components(&(), &mut components)
        .expect("archetype of `Tag` and `Mass` should contain unique component ids");
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
        .register_archetype::<(Position, Mass, Tag)>(&mut components, &())
        .expect("archetype of `Position`, `Mass` and `Tag` should contain unique component ids");
    let archetype_subset = archetypes
        .register_archetype::<(Position, Mass)>(&mut components, &())
        .expect("archetype of `Position` and `Mass` should contain unique component ids");

    let entity = entities.spawn(Default::default(), ());

    let mut position = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let mut mass = Mass { value: 42 };
    let mut tag = Tag;

    let storage = archetypes
        .get_archetype_info(archetype_subset)
        .expect("archetype should exist")
        .storage();
    assert!(!storage.contains(entity));

    let error = archetypes
        .get_bundle::<(Mass, Position)>(&mut components, &(), entity)
        .expect_err("entity should not have `Mass` and `Position` components yet");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Mass>()).into(),
    );

    archetypes
        .insert_bundle_exact::<(Position, Mass)>(&mut components, &(), entity, (position, mass))
        .expect("entity should not have `Position` and `Mass` components yet");

    let InsertBundleError { value, reason, .. } = archetypes
        .insert_bundle_exact::<(Mass, Position)>(&mut components, &(), entity, (mass, position))
        .expect_err("entity should already have `Position` and `Mass` components");
    assert_eq!(
        reason,
        DuplicateComponentError::new(components.register_component::<Mass>()),
    );
    assert_eq!(value, (mass, position));

    let storage = archetypes
        .get_archetype_info(archetype_subset)
        .expect("archetype should exist")
        .storage();
    assert!(storage.contains(entity));

    let refs = archetypes
        .get_bundle_mut::<(Mass, Position)>(&mut components, &(), entity)
        .expect("entity should have `Mass` and `Position` components");
    assert_eq!(refs, (&mut mass, &mut position));

    let error = archetypes
        .get_bundle::<(Mass, Tag, Position)>(&mut components, &(), entity)
        .expect_err("entity should not have `Tag` component yet");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Tag>()).into(),
    );

    archetypes
        .insert_bundle_exact::<(Tag,)>(&mut components, &(), entity, (tag,))
        .expect("entity should not have `Tag` component yet");

    let InsertBundleError { value, reason, .. } = archetypes
        .insert_bundle_exact::<(Tag,)>(&mut components, &(), entity, (tag,))
        .expect_err("entity already has `Tag` component");
    assert_eq!(
        reason,
        DuplicateComponentError::new(components.register_component::<Tag>()),
    );
    assert_eq!(value, (tag,));

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
        .get_bundle_mut::<(Mass, Tag, Position)>(&mut components, &(), entity)
        .expect("entity should have `Mass`, `Tag` and `Position` components");
    assert_eq!(refs, (&mut mass, &mut tag, &mut position));

    let (tag,) = archetypes
        .remove_bundle_exact::<(Tag,)>(&mut components, &(), entity)
        .expect("entity should have `Tag` component");
    assert_eq!(tag, Tag);

    let error = archetypes
        .remove_bundle_exact::<(Tag,)>(&mut components, &(), entity)
        .expect_err("entity should not have `Tag` component");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Tag>()).into(),
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
        .get_bundle_mut::<(Mass, Tag, Position)>(&mut components, &(), entity)
        .expect_err("entity should not have `Tag` component");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Tag>()).into(),
    );

    let refs = archetypes
        .get_bundle::<(Mass, Position)>(&mut components, &(), entity)
        .expect("entity should have `Mass` and `Position` components");
    assert_eq!(refs, (&mass, &position));

    let error = archetypes
        .remove_bundle_exact::<(Mass, Tag, Position)>(&mut components, &(), entity)
        .expect_err("entity should not have `Tag` component");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Tag>()).into(),
    );

    let value = archetypes
        .remove_bundle_exact::<(Mass, Position)>(&mut components, &(), entity)
        .expect("entity should have `Mass` and `Position` components");
    assert_eq!(value, (mass, position));

    let error = archetypes
        .remove_bundle_exact::<(Mass, Position)>(&mut components, &(), entity)
        .expect_err("entity should not have `Mass` and `Position` components");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Mass>()).into(),
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
        .remove_bundle_exact::<(Position, Mass)>(&mut components, &(), entity)
        .expect_err("entity should not have `Position` and `Mass` component");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Mass>()).into(),
    );
}

#[test]
fn exchange_components_empty_registry() {
    let mut entities = EntityRegistry::new();
    let mut components = ComponentRegistry::new();
    let mut archetypes = ArchetypeRegistry::new();

    let entity = entities.spawn(Default::default(), ());

    let error = archetypes
        .get_bundle::<(Mass, Tag)>(&components, &(), entity)
        .expect_err("entity should not have `Mass` and `Tag` components yet");
    assert_eq!(
        std::mem::discriminant(&error),
        std::mem::discriminant(&IncompatibleBundleError::from(
            ComponentNotRegisteredError::new(),
        )),
    );

    let mass = Mass { value: 42 };
    let tag = Tag;
    archetypes
        .insert_bundle_exact::<(Tag,)>(&mut components, &(), entity, (tag,))
        .expect("entity should not have `Tag` component yet");

    let (&tag,) = archetypes
        .get_bundle::<(Tag,)>(&components, &(), entity)
        .expect("entity should have `Tag` component");
    assert_eq!(tag, Tag);

    let error = archetypes
        .get_bundle::<(Mass, Tag)>(&components, &(), entity)
        .expect_err("entity should not have `Mass` and `Tag` components yet");
    assert_eq!(
        std::mem::discriminant(&error),
        std::mem::discriminant(&IncompatibleBundleError::from(
            ComponentNotRegisteredError::new(),
        )),
    );

    let InsertBundleError { value, reason, .. } = archetypes
        .insert_bundle_exact::<(Mass, Tag)>(&mut components, &(), entity, (mass, tag))
        .expect_err("entity already has `Tag` component");
    assert_eq!(
        reason,
        DuplicateComponentError::new(components.register_component::<Tag>()),
    );
    assert_eq!(value, (mass, tag));

    let component_ids = <(Tag,)>::register_components(&(), &mut components)
        .expect("archetype of only `Tag` should contain unique component ids");
    let archetype = archetypes
        .register_archetype_from(&components, component_ids)
        .expect("archetype of only `Tag` should contain unique component ids");
    let storage = archetypes
        .get_archetype_info(archetype)
        .expect("archetype should exist")
        .storage();
    assert!(storage.contains(entity));

    let (tag,) = archetypes
        .remove_bundle_exact::<(Tag,)>(&mut components, &(), entity)
        .expect("entity should have `Tag` component");
    assert_eq!(tag, Tag);

    let error = archetypes
        .remove_bundle_exact::<(Tag,)>(&mut components, &(), entity)
        .expect_err("entity should not have `Tag` component");
    assert_eq!(
        error,
        ExclusiveComponentError::new(components.register_component::<Tag>()).into(),
    );

    let storage = archetypes
        .get_archetype_info(archetype)
        .expect("archetype should exist")
        .storage();
    assert!(!storage.contains(entity));
}
