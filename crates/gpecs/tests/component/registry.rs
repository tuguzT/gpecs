use std::{
    alloc::Layout,
    any::{type_name, TypeId},
};

use gpecs::{
    component::registry::{ComponentDescriptor, ComponentRegistry},
    soa::traits::FieldDescriptor,
};

use crate::common::{Mass, Position};

#[test]
fn new() {
    let components = ComponentRegistry::new();
    assert_eq!(components.len(), 0);
    assert!(components.component_ids().is_empty());
}

#[test]
fn register_type() {
    let mut components = ComponentRegistry::new();
    assert_eq!(components.component_id::<Position>(), None);

    let id = components.register_component::<Position>();
    assert_eq!(components.len(), 1);
    assert_eq!(id.index(), 0);
    assert_eq!(components.component_id::<Position>(), Some(id));
    assert!(components.component_ids().eq([id]));

    assert_eq!(components.register_component::<Position>(), id);

    let info = components
        .get_component_info(id)
        .expect("info of just registered component should present");
    assert_eq!(info.id(), id);
    assert_eq!(info.type_id(), Some(TypeId::of::<Position>()));
    assert_eq!(info.name(), type_name::<Position>());
    assert_eq!(info.descriptor().layout(), Layout::new::<Position>());

    let id = components.register_component::<Mass>();
    assert_eq!(components.len(), 2);
    assert_eq!(id.index(), 1);
    assert_eq!(components.component_id::<Mass>(), Some(id));
    assert!(components.component_ids().any(|item| item == id));

    let info = components
        .get_component_info(id)
        .expect("info of just registered component should present");
    assert_eq!(info.id(), id);
    assert_eq!(info.type_id(), Some(TypeId::of::<Mass>()));
    assert_eq!(info.name(), type_name::<Mass>());
    assert_eq!(info.descriptor().layout(), Layout::new::<Mass>());
}

#[test]
fn register_with_descriptor() {
    let mut components = ComponentRegistry::new();
    components.register_component::<Position>();
    assert_eq!(components.len(), 1);

    let descriptor = ComponentDescriptor::new("Mass", None, FieldDescriptor::of::<f32>(), None);
    let id = components.register_component_with(descriptor);
    assert_eq!(components.len(), 2);
    assert_eq!(id.index(), 1);
    assert!(components.component_ids().any(|item| item == id));

    let info = components
        .get_component_info(id)
        .expect("info of just registered component should present");
    assert_eq!(info.id(), id);
    assert_eq!(info.type_id(), None);
    assert_eq!(info.name(), "Mass");
    assert_eq!(info.descriptor().layout(), Layout::new::<f32>());
    assert_eq!(info.drop_fn(), None);
}
