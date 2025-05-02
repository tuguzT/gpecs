use std::{
    any::{self, TypeId},
    borrow::Cow,
    collections::HashMap,
    fmt::{self, Debug},
    iter::FusedIterator,
    mem,
    ops::Range,
    ptr,
};

pub use gpecs_types::component::ComponentId;

use crate::soa::traits::FieldDescriptor;

use super::Component;

pub type DropFn = unsafe fn(to_drop: *mut u8);

#[derive(Debug, Clone)]
pub struct ComponentDescriptor {
    name: Cow<'static, str>,
    type_id: Option<TypeId>,
    desc: FieldDescriptor,
    drop_fn: Option<DropFn>,
}

impl ComponentDescriptor {
    #[inline]
    pub fn new<N, I>(name: N, type_id: I, desc: FieldDescriptor, drop_fn: Option<DropFn>) -> Self
    where
        N: Into<Cow<'static, str>>,
        I: Into<Option<TypeId>>,
    {
        Self {
            name: name.into(),
            type_id: type_id.into(),
            desc,
            drop_fn,
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn of<T>() -> Self
    where
        T: Component,
    {
        let to_drop: DropFn = |to_drop| {
            let to_drop = to_drop.cast();
            unsafe { ptr::drop_in_place::<T>(to_drop) };
        };

        Self {
            name: any::type_name::<T>().into(),
            type_id: Some(TypeId::of::<T>()),
            desc: FieldDescriptor::of::<T>(),
            drop_fn: mem::needs_drop::<T>().then(|| to_drop),
        }
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
    pub fn drop_fn(&self) -> Option<DropFn> {
        let Self { drop_fn, .. } = *self;
        drop_fn
    }
}

#[derive(Debug, Clone)]
pub struct ComponentInfo {
    id: ComponentId,
    descriptor: ComponentDescriptor,
}

impl ComponentInfo {
    #[inline]
    pub fn id(&self) -> ComponentId {
        let Self { id, .. } = *self;
        id
    }

    #[inline]
    pub fn type_id(&self) -> Option<TypeId> {
        let Self { descriptor, .. } = self;
        descriptor.type_id()
    }

    #[inline]
    pub fn name(&self) -> &str {
        let Self { descriptor, .. } = self;
        descriptor.name()
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { descriptor, .. } = self;
        descriptor.descriptor()
    }

    #[inline]
    pub fn drop_fn(&self) -> Option<DropFn> {
        let Self { descriptor, .. } = self;
        descriptor.drop_fn()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ComponentRegistry {
    components: Vec<ComponentInfo>,
    type_ids: HashMap<TypeId, ComponentId>,
}

impl ComponentRegistry {
    #[inline]
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            type_ids: HashMap::new(),
        }
    }

    #[inline]
    pub fn register_component<T>(&mut self) -> ComponentId
    where
        T: Component,
    {
        let Self {
            components,
            type_ids,
        } = self;

        let type_id = TypeId::of::<T>();
        type_ids
            .entry(type_id)
            .or_insert_with(|| {
                let descriptor = ComponentDescriptor::of::<T>();
                Self::register_inner(components, descriptor)
            })
            .clone()
    }

    #[inline]
    pub fn register_component_with(&mut self, descriptor: ComponentDescriptor) -> ComponentId {
        let Self { components, .. } = self;
        Self::register_inner(components, descriptor)
    }

    #[inline]
    fn register_inner(
        components: &mut Vec<ComponentInfo>,
        descriptor: ComponentDescriptor,
    ) -> ComponentId {
        let index = components.len();
        let id = component_id_from_usize(index);

        let info = ComponentInfo { id, descriptor };
        components.push(info);

        id
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
    pub fn get_component_info(&self, id: ComponentId) -> Option<&ComponentInfo> {
        let Self { components, .. } = self;

        let index = component_id_into_usize(id);
        components.get(index)
    }

    #[inline]
    pub fn component_id_from(&self, type_id: TypeId) -> Option<ComponentId> {
        let Self { type_ids, .. } = self;
        type_ids.get(&type_id).cloned()
    }

    #[inline]
    pub fn component_id<T>(&self) -> Option<ComponentId>
    where
        T: Component,
    {
        let type_id = TypeId::of::<T>();
        self.component_id_from(type_id)
    }

    #[inline]
    pub fn component_ids(&self) -> ComponentIds {
        let len = self.len();
        let len = component_id_from_usize(len).into_u32();
        ComponentIds { inner: 0..len }
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
#[allow(unsafe_code)]
fn component_id_trusted(id: u32) -> ComponentId {
    unsafe { ComponentId::from_u32(id) }
}
