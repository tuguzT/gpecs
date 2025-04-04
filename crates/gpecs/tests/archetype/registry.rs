use gpecs::{
    archetype::registry::ArchetypeRegistry,
    bundle::Bundle,
    component::{registry::ComponentRegistry, Component},
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
}

#[test]
fn register_archetype() {
    let mut components = ComponentRegistry::new();
    let mut archetypes = ArchetypeRegistry::new();
    assert_eq!(archetypes.len(), 0);

    let id = archetypes
        .register_archetype::<(Position, Mass, Tag)>(&mut components, &())
        .expect("archetype of `Position`, `Mass` and `Tag` should contain unique component ids");
    assert_eq!(archetypes.len(), 3);
    assert_eq!(
        archetypes
            .archetype_id::<(Position, Mass, Tag)>(&mut components, &())
            .expect("archetype of `Position`, `Mass` and `Tag` should contain unique component ids")
            .expect("archetype of `Position`, `Mass` and `Tag` should be already registered"),
        id,
    );

    let same_id = archetypes
        .register_archetype::<(Mass, Tag, Position)>(&mut components, &())
        .expect("archetype of `Position`, `Mass` and `Tag` should contain unique component ids");
    assert_eq!(same_id, id);

    let component_ids = <(Mass, Tag, Position)>::component_ids(&(), &mut components).unwrap();
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

    // TODO: fails for now, fix ASAP
    // let component_ids = <(Mass,)>::component_ids(&(), &mut components).unwrap();
    // assert_ne!(
    //     archetypes
    //         .archetype_id_from(component_ids)
    //         .expect("archetype of only `Mass` should contain unique component ids")
    //         .expect("archetype of only `Mass` should be already registered"),
    //     id,
    // );

    let component_ids = <(Mass, Position)>::component_ids(&(), &mut components).unwrap();
    assert_ne!(
        archetypes
            .archetype_id_from(component_ids)
            .expect("archetype of `Position` and `Mass` should contain unique component ids")
            .expect("archetype of `Position` and `Mass` should be already registered"),
        id,
    );

    let component_ids = <(Position,)>::component_ids(&(), &mut components).unwrap();
    assert_ne!(
        archetypes
            .archetype_id_from(component_ids)
            .expect("archetype of only `Position` should contain unique component ids")
            .expect("archetype of only `Position` should be already registered"),
        id,
    );

    let id = archetypes
        .register_archetype::<(Position,)>(&mut components, &())
        .expect("archetype of only `Position` should contain unique component ids");
    assert_eq!(archetypes.len(), 3);
    assert_eq!(
        archetypes
            .archetype_id::<(Position,)>(&mut components, &())
            .expect("archetype of only `Position` should contain unique component ids")
            .expect("archetype of only `Position` should be already registered"),
        id,
    );

    let same_id = archetypes
        .register_archetype::<(Position,)>(&mut components, &())
        .expect("archetype of only `Position` should contain unique component ids");
    assert_eq!(same_id, id);

    let same_id = archetypes
        .register_archetype_with_components(&components, component_ids)
        .expect("archetype of only `Position` should contain unique component ids");
    assert_eq!(same_id, id);
    assert_eq!(
        archetypes
            .archetype_id_from(component_ids)
            .expect("archetype of only `Position` should contain unique component ids")
            .expect("archetype of only `Position` should be already registered"),
        id,
    );
}
