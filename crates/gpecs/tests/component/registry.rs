use std::{
    alloc::Layout,
    any::{type_name, TypeId},
};

use gpecs::component::{
    registry::{ComponentDescriptor, ComponentRegistry},
    Component,
};

struct Position {
    _x: f32,
    _y: f32,
    _z: f32,
}

impl Component for Position {}

#[test]
fn new() {
    let components = ComponentRegistry::new();
    assert_eq!(components.len(), 0);
}

#[test]
fn register_type() {
    let mut components = ComponentRegistry::new();
    assert_eq!(components.len(), 0);
    assert_eq!(components.component_id::<Position>(), None);

    let id = components.register_component::<Position>();
    assert_eq!(components.len(), 1);
    assert_eq!(id.index(), 0);
    assert_eq!(components.component_id::<Position>(), Some(id));

    assert_eq!(components.register_component::<Position>(), id);

    let info = components
        .get_info(id)
        .expect("info of just registered component should present");
    assert_eq!(info.id(), id);
    assert_eq!(info.type_id(), Some(TypeId::of::<Position>()));
    assert_eq!(info.name(), type_name::<Position>());
    assert_eq!(info.layout(), Layout::new::<Position>());
}

#[test]
fn register_with_descriptor() {
    let mut components = ComponentRegistry::new();
    components.register_component::<Position>();
    assert_eq!(components.len(), 1);

    let descriptor = ComponentDescriptor::new("Mass", Layout::new::<f32>());
    let id = components.register_component_with_descriptor(descriptor);
    assert_eq!(components.len(), 2);
    assert_eq!(id.index(), 1);

    let info = components
        .get_info(id)
        .expect("info of just registered component should present");
    assert_eq!(info.id(), id);
    assert_eq!(info.type_id(), None);
    assert_eq!(info.name(), "Mass");
    assert_eq!(info.layout(), Layout::new::<f32>());
}
