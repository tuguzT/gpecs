use gpecs::{
    archetype::registry::ArchetypeRegistry,
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

impl Component for Position {}
impl Component for Mass {}

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
        .register_archetype::<(Position,)>((), &mut components)
        .expect("archetype of single component should be registered successfully");
    assert_eq!(archetypes.len(), 1);
    assert_eq!(id.index(), 0);
    assert_eq!(
        archetypes
            .archetype_id::<(Position,)>(&(), &mut components)
            .unwrap(),
        Some(id)
    );

    let same_id = archetypes
        .register_archetype::<(Position,)>((), &mut components)
        .expect("archetype of single component should be registered successfully");
    assert_eq!(same_id, id);

    let id = archetypes
        .register_archetype::<(Position, Mass)>((), &mut components)
        .expect("archetype of two components should be registered successfully");
    assert_eq!(archetypes.len(), 2);
    assert_eq!(id.index(), 1);
    assert_eq!(
        archetypes
            .archetype_id::<(Position, Mass)>(&(), &mut components)
            .unwrap(),
        Some(id)
    );
}
