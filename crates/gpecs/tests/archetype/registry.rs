use gpecs::{
    archetype::registry::ArchetypeRegistry,
    bundle::Bundle,
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
        .register_archetype_with_components(&components, component_ids)
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
        .register_archetype_with_components(&components, component_ids)
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

    let position = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let mass = Mass { value: 42 };
    let tag = Tag;

    let storage = archetypes
        .get_info_mut(archetype_subset)
        .expect("archetype should exist")
        .storage_mut();
    assert!(!storage.contains(entity));

    let prev_value = storage
        .insert::<(Position, Mass)>(&mut components, &(), entity, (position, mass))
        .expect("bundle should be compatible");
    assert_eq!(prev_value, None);
    assert!(storage.contains(entity));

    archetypes.insert_component::<Tag>(&mut components, entity, tag);

    let storage = archetypes
        .get_info_mut(archetype_subset)
        .expect("archetype should exist")
        .storage_mut();
    assert!(!storage.contains(entity));

    let storage = archetypes
        .get_info_mut(archetype)
        .expect("archetype should exist")
        .storage_mut();
    assert!(storage.contains(entity));

    archetypes.remove_component::<Tag>(&mut components, entity);

    let storage = archetypes
        .get_info_mut(archetype_subset)
        .expect("archetype should exist")
        .storage_mut();
    assert!(storage.contains(entity));

    let storage = archetypes
        .get_info_mut(archetype)
        .expect("archetype should exist")
        .storage_mut();
    assert!(!storage.contains(entity));
}

#[test]
fn exchange_components_empty_registry() {
    let mut entities = EntityRegistry::new();
    let mut components = ComponentRegistry::new();
    let mut archetypes = ArchetypeRegistry::new();

    let entity = entities.spawn(Default::default(), ());
    archetypes.insert_component::<Tag>(&mut components, entity, Tag);

    let component_ids = <(Tag,)>::register_components(&(), &mut components)
        .expect("archetype of only `Tag` should contain unique component ids");
    let archetype = archetypes
        .register_archetype_with_components(&components, component_ids)
        .expect("archetype of only `Tag` should contain unique component ids");
    let storage = archetypes
        .get_info_mut(archetype)
        .expect("archetype should exist")
        .storage_mut();
    assert!(storage.contains(entity));

    archetypes.remove_component::<Tag>(&mut components, entity);

    let storage = archetypes
        .get_info_mut(archetype)
        .expect("archetype should exist")
        .storage_mut();
    assert!(!storage.contains(entity));
}
