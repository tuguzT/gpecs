use std::{
    any::{self, TypeId},
    borrow::Cow,
    collections::HashMap,
};

use crate::{
    component::{
        Component,
        erased::{ErasedDrop, WithErasedDrop},
        registry::{
            self, ComponentId, ComponentRegistry,
            traits::{ComponentIdFrom, ComponentIdFromOrInsertWith, FromComponentType},
        },
    },
    hash::BuildHasher,
    soa::field::FieldDescriptor,
};

pub type Components = ComponentRegistry<ErasedDropComponentDescriptor, ComponentTypeIdMap>;
pub type ComponentInfo<'a> = registry::ComponentInfo<&'a ErasedDropComponentDescriptor>;

#[derive(Debug, Clone)]
pub struct ErasedDropComponentDescriptor {
    name: Cow<'static, str>,
    type_id: Option<TypeId>,
    desc: FieldDescriptor,
    erased_drop: Option<ErasedDrop>,
}

impl ErasedDropComponentDescriptor {
    #[inline]
    pub fn new<N>(
        name: N,
        type_id: Option<TypeId>,
        desc: FieldDescriptor,
        erased_drop: Option<ErasedDrop>,
    ) -> Self
    where
        N: Into<Cow<'static, str>>,
    {
        Self {
            name: name.into(),
            type_id,
            desc,
            erased_drop,
        }
    }

    #[inline]
    pub fn of<T>() -> Self
    where
        T: Component,
    {
        let name = any::type_name::<T>();
        let type_id = Some(TypeId::of::<T>());
        let desc = FieldDescriptor::of::<T>();
        let erased_drop = ErasedDrop::of::<T>();
        Self::new(name, type_id, desc, erased_drop)
    }

    #[inline]
    pub fn type_id(&self) -> Option<TypeId> {
        let Self { type_id, .. } = *self;
        type_id
    }

    #[inline]
    pub fn name(&self) -> &str {
        let Self { name, .. } = self;
        name.as_ref()
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { desc, .. } = *self;
        desc
    }

    #[inline]
    pub fn erased_drop(&self) -> Option<ErasedDrop> {
        let Self { erased_drop, .. } = *self;
        erased_drop
    }
}

impl AsRef<str> for ErasedDropComponentDescriptor {
    #[inline]
    fn as_ref(&self) -> &str {
        self.name()
    }
}

impl AsRef<FieldDescriptor> for ErasedDropComponentDescriptor {
    #[inline]
    fn as_ref(&self) -> &FieldDescriptor {
        let Self { desc, .. } = self;
        desc
    }
}

impl FromComponentType for ErasedDropComponentDescriptor {
    #[inline]
    fn from_component<T: Component>() -> Self {
        Self::of::<T>()
    }
}

impl WithErasedDrop for ErasedDropComponentDescriptor {
    #[inline]
    fn erased_drop(&self) -> Option<ErasedDrop> {
        Self::erased_drop(self)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ComponentTypeIdMap {
    map: HashMap<TypeId, ComponentId, BuildHasher>,
}

unsafe impl ComponentIdFrom for ComponentTypeIdMap {
    type Key = TypeId;

    #[inline]
    fn component_id_from(&self, key: Self::Key) -> Option<ComponentId> {
        let Self { map, .. } = self;
        map.get(&key).copied()
    }
}

unsafe impl ComponentIdFromOrInsertWith for ComponentTypeIdMap {
    #[inline]
    fn component_id_from_or_insert_with<F>(&mut self, key: Self::Key, f: F) -> ComponentId
    where
        F: FnOnce() -> ComponentId,
    {
        let Self { map } = self;
        *map.entry(key).or_insert_with(f)
    }
}
