use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ops::Range,
};

use crate::component::Component;

use super::{
    ComponentId, ComponentTypeIdMap, ErasedDropComponentDescriptor,
    traits::{ComponentIdFrom, ComponentIdFromOrInsertWith, FromComponentType},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentInfo<Meta = ErasedDropComponentDescriptor> {
    component_id: ComponentId,
    meta: Meta,
}

impl<Meta> ComponentInfo<Meta> {
    #[inline]
    pub fn component_id(&self) -> ComponentId {
        let Self { component_id, .. } = *self;
        component_id
    }

    #[inline]
    pub fn as_meta(&self) -> &Meta {
        let Self { meta, .. } = self;
        meta
    }
}

#[derive(Debug, Clone)]
pub struct ComponentRegistry<Meta = ErasedDropComponentDescriptor, Mapping = ComponentTypeIdMap>
where
    Mapping: ?Sized,
{
    components: Vec<ComponentInfo<Meta>>,
    mapping: Mapping,
}

impl<Meta, Mapping> ComponentRegistry<Meta, Mapping> {
    #[inline]
    pub unsafe fn with_mapping(mapping: Mapping) -> Self {
        Self {
            components: Vec::new(),
            mapping,
        }
    }
}

impl<Meta, Mapping> ComponentRegistry<Meta, Mapping>
where
    Mapping: Default,
{
    #[inline]
    pub fn new() -> Self {
        let mapping = Mapping::default();
        unsafe { Self::with_mapping(mapping) }
    }
}

impl<Meta, Mapping> ComponentRegistry<Meta, Mapping>
where
    Mapping: ?Sized,
{
    #[inline]
    pub fn register_component_with(&mut self, meta: Meta) -> ComponentId {
        let Self { components, .. } = self;
        Self::register_inner(components, meta)
    }

    #[inline]
    fn register_inner(components: &mut Vec<ComponentInfo<Meta>>, meta: Meta) -> ComponentId {
        let index = components.len();
        let component_id = component_id_from_usize(index);

        let info = ComponentInfo { component_id, meta };
        components.push(info);

        component_id
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { components, .. } = self;
        components.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { components, .. } = self;
        components.is_empty()
    }

    #[inline]
    pub fn get_component_info(&self, id: ComponentId) -> Option<&ComponentInfo<Meta>> {
        let Self { components, .. } = self;

        let index = component_id_into_usize(id);
        components.get(index)
    }

    #[inline]
    pub fn component_ids(&self) -> ComponentIds {
        let len = self.len();
        let len = component_id_from_usize(len).into_u32();
        ComponentIds { inner: 0..len }
    }
}

impl<Meta, Mapping> ComponentRegistry<Meta, Mapping>
where
    Mapping: ComponentIdFrom + ?Sized,
{
    #[inline]
    pub fn component_id_from(&self, key: Mapping::Key) -> Option<ComponentId> {
        let Self { mapping, .. } = self;
        mapping.component_id_from(key)
    }

    #[inline]
    pub fn component_id<T>(&self) -> Option<ComponentId>
    where
        T: Component,
        Mapping::Key: FromComponentType,
    {
        let key = FromComponentType::from_component::<T>();
        self.component_id_from(key)
    }
}

impl<Meta, Mapping> ComponentRegistry<Meta, Mapping>
where
    Meta: FromComponentType,
    Mapping: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
{
    #[inline]
    pub fn register_component<T>(&mut self) -> ComponentId
    where
        T: Component,
    {
        let Self {
            components,
            mapping,
        } = self;

        let key = FromComponentType::from_component::<T>();
        let new_meta = || {
            let meta = Meta::from_component::<T>();
            Self::register_inner(components, meta)
        };
        mapping.component_id_from_or_insert_with(key, new_meta)
    }
}

impl<Meta, Mapping> Default for ComponentRegistry<Meta, Mapping>
where
    Mapping: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ComponentIds {
    inner: Range<u32>,
}

impl ComponentIds {
    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { inner } = self;
        inner.is_empty()
    }
}

impl Debug for ComponentIds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;

        let Range { start, end } = *inner;
        let ids = component_id_trusted(start)..component_id_trusted(end);
        f.debug_struct("ComponentIds").field("ids", &ids).finish()
    }
}

impl Iterator for ComponentIds {
    type Item = ComponentId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(component_id_trusted)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(component_id_trusted)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(component_id_trusted)
    }

    #[inline]
    fn min(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.min().map(component_id_trusted)
    }

    #[inline]
    fn max(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.max().map(component_id_trusted)
    }

    #[inline]
    fn is_sorted(self) -> bool {
        let Self { inner } = self;
        inner.is_sorted()
    }
}

impl DoubleEndedIterator for ComponentIds {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(component_id_trusted)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(component_id_trusted)
    }
}

impl ExactSizeIterator for ComponentIds {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for ComponentIds {}

#[inline]
fn component_id_from_usize(index: usize) -> ComponentId {
    let id = index.try_into().expect("`ComponentId` overflow");
    component_id_trusted(id)
}

#[inline]
fn component_id_into_usize(id: ComponentId) -> usize {
    let id = id.into_u32();
    id.try_into().expect("`ComponentId` overflow")
}

#[inline]
fn component_id_trusted(id: u32) -> ComponentId {
    unsafe { ComponentId::from_u32(id) }
}
