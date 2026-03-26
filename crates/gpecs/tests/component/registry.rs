use std::{
    alloc::Layout,
    any::{TypeId, type_name},
};

use gpecs::{
    component::registry::{ComponentRegistry, ErasedDropComponentDescriptor},
    soa::field::FieldDescriptor,
};

use crate::common::{Mass, Position};

#[test]
fn new() {
    let components: ComponentRegistry = ComponentRegistry::new();
    assert_eq!(components.len(), 0);
    assert!(components.component_ids().is_empty());
}

#[test]
fn register_type() {
    let mut components: ComponentRegistry = ComponentRegistry::new();
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
    assert_eq!(info.as_meta().type_id(), Some(TypeId::of::<Position>()));
    assert_eq!(info.as_meta().name(), type_name::<Position>());
    assert_eq!(
        info.as_meta().descriptor().layout(),
        Layout::new::<Position>(),
    );

    let id = components.register_component::<Mass>();
    assert_eq!(components.len(), 2);
    assert_eq!(id.into_u32(), 1);
    assert_eq!(components.component_id::<Mass>(), Some(id));
    assert!(components.component_ids().any(|item| item == id));

    let info = components
        .get_component_info(id)
        .expect("info of just registered component should present");
    assert_eq!(info.component_id(), id);
    assert_eq!(info.as_meta().type_id(), Some(TypeId::of::<Mass>()));
    assert_eq!(info.as_meta().name(), type_name::<Mass>());
    assert_eq!(info.as_meta().descriptor().layout(), Layout::new::<Mass>());
}

#[test]
fn register_with_descriptor() {
    let mut components: ComponentRegistry = ComponentRegistry::new();
    components.register_component::<Position>();
    assert_eq!(components.len(), 1);

    let field_desc = FieldDescriptor::of::<f32>();
    let descriptor = ErasedDropComponentDescriptor::new("Mass", None, field_desc, None);
    let id = components.register_component_with(descriptor);
    assert_eq!(components.len(), 2);
    assert_eq!(id.into_u32(), 1);
    assert!(components.component_ids().any(|item| item == id));

    let info = components
        .get_component_info(id)
        .expect("info of just registered component should present");
    assert_eq!(info.component_id(), id);
    assert_eq!(info.as_meta().type_id(), None);
    assert_eq!(info.as_meta().name(), "Mass");
    assert_eq!(info.as_meta().descriptor().layout(), Layout::new::<f32>());
    assert!(info.as_meta().erased_drop().is_none());
}
