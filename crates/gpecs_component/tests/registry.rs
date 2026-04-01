#![cfg(feature = "alloc")]

use std::{
    alloc::Layout,
    any::{TypeId, type_name},
};

use gpecs_component::{
    Component,
    registry::{ComponentIdMap, ComponentRegistry, traits::FromComponentType},
};

#[derive(Debug, PartialEq, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl Component for Position {}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Mass {
    value: u32,
}

impl Component for Mass {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ComponentDescriptor {
    name: &'static str,
    type_id: Option<TypeId>,
    layout: Layout,
}

unsafe impl FromComponentType for ComponentDescriptor {
    fn from_component<T: Component>() -> Self {
        Self {
            name: type_name::<T>(),
            type_id: Some(TypeId::of::<T>()),
            layout: Layout::new::<T>(),
        }
    }
}

type Components = ComponentRegistry<ComponentDescriptor, ComponentIdMap<TypeId>>;

#[test]
fn new() {
    let components = Components::new();
    assert_eq!(components.len(), 0);
    assert!(components.component_ids().is_empty());
}

#[test]
fn register_type() {
    let mut components = Components::new();
    assert_eq!(components.component_id::<Position>(), None);

    let id = components.register_component::<Position>();
    assert_eq!(components.len(), 1);
    assert_eq!(id.into_u32(), 0);
    assert_eq!(components.component_id::<Position>(), Some(id));
    assert!(components.component_ids().eq([id]));

    assert_eq!(components.register_component::<Position>(), id);

    let info = components
        .get_component_info(id)
        .expect("info of just registered component should present");
    assert_eq!(info.component_id(), id);
    assert_eq!(info.type_id, Some(TypeId::of::<Position>()));
    assert_eq!(info.name, type_name::<Position>());
    assert_eq!(info.layout, Layout::new::<Position>());

    let id = components.register_component::<Mass>();
    assert_eq!(components.len(), 2);
    assert_eq!(id.into_u32(), 1);
    assert_eq!(components.component_id::<Mass>(), Some(id));
    assert!(components.component_ids().any(|item| item == id));

    let info = components
        .get_component_info(id)
        .expect("info of just registered component should present");
    assert_eq!(info.component_id(), id);
    assert_eq!(info.type_id, Some(TypeId::of::<Mass>()));
    assert_eq!(info.name, type_name::<Mass>());
    assert_eq!(info.layout, Layout::new::<Mass>());
}

#[test]
fn register_with_descriptor() {
    let mut components = Components::new();
    components.register_component::<Position>();
    assert_eq!(components.len(), 1);

    let descriptor = ComponentDescriptor {
        name: "Sweden",
        type_id: None,
        layout: Layout::new::<f32>(),
    };
    let id = components.register_component_with(descriptor);
    assert_eq!(components.len(), 2);
    assert_eq!(id.into_u32(), 1);
    assert!(components.component_ids().any(|item| item == id));

    let info = components
        .get_component_info(id)
        .expect("info of just registered component should present");
    assert_eq!(info.component_id(), id);
    assert_eq!(info.type_id, None);
    assert_eq!(info.name, "Sweden");
    assert_eq!(info.layout, Layout::new::<f32>());
}
