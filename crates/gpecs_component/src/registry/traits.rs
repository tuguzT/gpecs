use core::{
    alloc::Layout,
    any::{self, TypeId},
    borrow::Borrow,
};

use crate::{
    Component,
    registry::{ComponentId, ComponentInfo},
};

pub trait WithComponentId {
    fn component_id(&self) -> ComponentId;
}

impl<T> WithComponentId for T
where
    T: Borrow<ComponentId> + ?Sized,
{
    #[inline]
    fn component_id(&self) -> ComponentId {
        *self.borrow()
    }
}

impl<Meta> WithComponentId for ComponentInfo<Meta>
where
    Meta: ?Sized,
{
    #[inline]
    fn component_id(&self) -> ComponentId {
        ComponentInfo::component_id(self)
    }
}

pub unsafe trait FromComponentType: Sized {
    fn from_component<T: Component>() -> Self;
}

unsafe impl FromComponentType for TypeId {
    #[inline]
    fn from_component<T: Component>() -> Self {
        TypeId::of::<T>()
    }
}

unsafe impl FromComponentType for &str {
    #[inline]
    fn from_component<T: Component>() -> Self {
        any::type_name::<T>()
    }
}

unsafe impl FromComponentType for Layout {
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
