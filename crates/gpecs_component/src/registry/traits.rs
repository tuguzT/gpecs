use core::{
    alloc::Layout,
    any::{self, TypeId},
};

use crate::{Component, registry::ComponentId};

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

pub unsafe trait ComponentIdFrom {
    type Key;

    fn component_id_from(&self, key: Self::Key) -> Option<ComponentId>;
}

unsafe impl<T> ComponentIdFrom for &T
where
    T: ComponentIdFrom + ?Sized,
{
    type Key = T::Key;

    #[inline]
    fn component_id_from(&self, key: Self::Key) -> Option<ComponentId> {
        (**self).component_id_from(key)
    }
}

unsafe impl<T> ComponentIdFrom for &mut T
where
    T: ComponentIdFrom + ?Sized,
{
    type Key = T::Key;

    #[inline]
    fn component_id_from(&self, key: Self::Key) -> Option<ComponentId> {
        (**self).component_id_from(key)
    }
}

pub unsafe trait ComponentIdFromOrInsertWith: ComponentIdFrom {
    fn component_id_from_or_insert_with<F>(&mut self, key: Self::Key, f: F) -> ComponentId
    where
        F: FnOnce() -> ComponentId;
}

unsafe impl<T> ComponentIdFromOrInsertWith for &mut T
where
    T: ComponentIdFromOrInsertWith + ?Sized,
{
    #[inline]
    fn component_id_from_or_insert_with<F>(&mut self, key: Self::Key, f: F) -> ComponentId
    where
        F: FnOnce() -> ComponentId,
    {
        (**self).component_id_from_or_insert_with(key, f)
    }
}
