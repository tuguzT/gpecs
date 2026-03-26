use std::{
    alloc::Layout,
    any::{self, TypeId},
};

use crate::{
    component::{Component, erased::ErasedDrop, registry::ComponentId},
    soa::field::FieldDescriptor,
};

pub trait FromComponentType: Sized {
    fn from_component<T: Component>() -> Self;
}

impl FromComponentType for TypeId {
    #[inline]
    fn from_component<T: Component>() -> Self {
        TypeId::of::<T>()
    }
}

impl FromComponentType for &str {
    #[inline]
    fn from_component<T: Component>() -> Self {
        any::type_name::<T>()
    }
}

impl FromComponentType for Layout {
    #[inline]
    fn from_component<T: Component>() -> Self {
        Layout::new::<T>()
    }
}

impl FromComponentType for FieldDescriptor {
    #[inline]
    fn from_component<T: Component>() -> Self {
        FieldDescriptor::of::<T>()
    }
}

impl FromComponentType for Option<ErasedDrop> {
    #[inline]
    fn from_component<T: Component>() -> Self {
        ErasedDrop::of::<T>()
    }
}

pub trait ComponentIdFrom {
    type Key;

    fn component_id_from(&self, key: Self::Key) -> Option<ComponentId>;
}

impl<T> ComponentIdFrom for &T
where
    T: ComponentIdFrom,
{
    type Key = T::Key;

    #[inline]
    fn component_id_from(&self, key: Self::Key) -> Option<ComponentId> {
        (**self).component_id_from(key)
    }
}

impl<T> ComponentIdFrom for &mut T
where
    T: ComponentIdFrom,
{
    type Key = T::Key;

    #[inline]
    fn component_id_from(&self, key: Self::Key) -> Option<ComponentId> {
        (**self).component_id_from(key)
    }
}

pub trait ComponentIdFromOrInsertWith: ComponentIdFrom {
    fn component_id_from_or_insert_with<F>(&mut self, key: Self::Key, f: F) -> ComponentId
    where
        F: FnOnce() -> ComponentId;
}

impl<T> ComponentIdFromOrInsertWith for &mut T
where
    T: ComponentIdFromOrInsertWith,
{
    #[inline]
    fn component_id_from_or_insert_with<F>(&mut self, key: Self::Key, f: F) -> ComponentId
    where
        F: FnOnce() -> ComponentId,
    {
        (**self).component_id_from_or_insert_with(key, f)
    }
}
