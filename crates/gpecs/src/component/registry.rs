use std::{
    any::{type_name, TypeId},
    borrow::Cow,
    collections::HashMap,
    fmt::{self, Debug},
    iter::FusedIterator,
    ops::Range,
};

use crate::soa::traits::FieldDescriptor;

use super::Component;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct ComponentId(usize);

impl ComponentId {
    #[inline]
    pub const fn index(&self) -> usize {
        let Self(id) = *self;
        id
    }
}

#[derive(Debug, Clone)]
pub struct ComponentDescriptor {
    name: Cow<'static, str>,
    type_id: Option<TypeId>,
    desc: FieldDescriptor,
}

impl ComponentDescriptor {
    #[inline]
    pub fn new<N>(name: N, desc: FieldDescriptor) -> Self
    where
        N: Into<Cow<'static, str>>,
    {
        Self {
            name: name.into(),
            type_id: None,
            desc,
        }
    }

    #[inline]
    pub fn of<T>() -> Self
    where
        T: Component,
    {
        Self {
            name: type_name::<T>().into(),
            type_id: Some(TypeId::of::<T>()),
            desc: FieldDescriptor::of::<T>(),
        }
    }

    #[inline]
    pub fn type_id(&self) -> Option<TypeId> {
        let Self { type_id, .. } = self;
        type_id.clone()
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
    pub fn register_component_with_descriptor(
        &mut self,
        descriptor: ComponentDescriptor,
    ) -> ComponentId {
        let Self { components, .. } = self;
        Self::register_inner(components, descriptor)
    }

    #[inline]
    fn register_inner(
        components: &mut Vec<ComponentInfo>,
        descriptor: ComponentDescriptor,
    ) -> ComponentId {
        let id = ComponentId(components.len());
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
    pub fn get_info(&self, id: ComponentId) -> Option<&ComponentInfo> {
        let Self { components, .. } = self;

        let index = id.index();
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
    pub fn components(&self) -> ComponentIds {
        let len = self.len();
        ComponentIds { inner: 0..len }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ComponentIds {
    inner: Range<usize>,
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
        let inner = ComponentId(start)..ComponentId(end);
        write!(f, "{inner:?}")
    }
}

impl Iterator for ComponentIds {
    type Item = ComponentId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(ComponentId)
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
        inner.nth(n).map(ComponentId)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(ComponentId)
    }

    #[inline]
    fn min(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.min().map(ComponentId)
    }

    #[inline]
    fn max(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.max().map(ComponentId)
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
        inner.next_back().map(ComponentId)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(ComponentId)
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
