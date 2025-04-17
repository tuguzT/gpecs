use std::{
    any::{self, TypeId},
    borrow::Cow,
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr,
};

use gpecs_sparse::set::EpochSparseSet;

use crate::{
    component::registry::{ComponentDescriptor, ComponentId, ComponentRegistry},
    soa::traits::FieldDescriptor,
};

use super::GpuComponent;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct GpuComponentId(ComponentId);

impl GpuComponentId {
    #[inline]
    pub fn index(&self) -> usize {
        let Self(id) = *self;
        id.index()
    }

    #[inline]
    pub const fn into_inner(self) -> u32 {
        let Self(id) = self;
        id.into_inner()
    }
}

impl From<GpuComponentId> for ComponentId {
    #[inline]
    fn from(value: GpuComponentId) -> Self {
        let GpuComponentId(id) = value;
        id
    }
}

#[derive(Debug, Clone)]
pub struct GpuComponentDescriptor {
    name: Cow<'static, str>,
    type_id: Option<TypeId>,
    desc: FieldDescriptor,
}

impl GpuComponentDescriptor {
    #[inline]
    pub fn new<N, I>(name: N, type_id: I, desc: FieldDescriptor) -> Self
    where
        N: Into<Cow<'static, str>>,
        I: Into<Option<TypeId>>,
    {
        Self {
            name: name.into(),
            type_id: type_id.into(),
            desc,
        }
    }

    #[inline]
    pub fn of<T>() -> Self
    where
        T: GpuComponent,
    {
        Self {
            name: any::type_name::<T>().into(),
            type_id: Some(TypeId::of::<T>()),
            desc: FieldDescriptor::of::<T>(),
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
}

impl From<GpuComponentDescriptor> for ComponentDescriptor {
    fn from(value: GpuComponentDescriptor) -> Self {
        let GpuComponentDescriptor {
            name,
            type_id,
            desc,
        } = value;
        ComponentDescriptor::new(name, type_id, desc, None)
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpuComponentRegistry {
    components: EpochSparseSet<u32, ()>,
}

impl GpuComponentRegistry {
    #[inline]
    pub fn new() -> Self {
        Self {
            components: EpochSparseSet::new(),
        }
    }

    #[inline]
    pub fn register_component<T>(&mut self, components: &mut ComponentRegistry) -> GpuComponentId
    where
        T: GpuComponent,
    {
        let id = components.register_component::<T>();
        let id = GpuComponentId(id);

        let Self { components } = self;
        components.insert(id.into_inner(), ());

        id
    }

    #[inline]
    pub fn register_component_with(
        &mut self,
        components: &mut ComponentRegistry,
        descriptor: GpuComponentDescriptor,
    ) -> GpuComponentId {
        let id = components.register_component_with(descriptor.into());
        let id = GpuComponentId(id);

        let Self { components } = self;
        components.insert(id.into_inner(), ());

        id
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { components } = self;
        components.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { components } = self;
        components.is_empty()
    }

    #[inline]
    pub fn contains(&self, id: ComponentId) -> bool {
        let Self { components } = self;
        components.contains_key(id.into_inner())
    }

    #[inline]
    pub fn map_component_id(&self, id: ComponentId) -> Option<GpuComponentId> {
        self.contains(id).then_some(GpuComponentId(id))
    }

    #[inline]
    pub fn component_ids(&self) -> GpuComponentIds {
        let Self { components } = self;

        // SAFETY: `GpuComponentId` is a #[repr(transparent)] struct around `ComponentId`,
        // which is #[repr(transparent)] around `u32`.
        #[allow(unsafe_code)]
        let component_ids = unsafe {
            let slice = components.as_keys_slice();
            &*(ptr::from_ref(slice) as *const [GpuComponentId])
        };
        let inner = component_ids.iter();
        GpuComponentIds { inner }
    }
}

#[derive(Clone)]
pub struct GpuComponentIds<'a> {
    inner: std::slice::Iter<'a, GpuComponentId>,
}

impl Debug for GpuComponentIds<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;

        let component_ids = inner.as_slice();
        f.debug_struct("GpuComponentIds")
            .field("component_ids", &component_ids)
            .finish()
    }
}

impl Iterator for GpuComponentIds<'_> {
    type Item = GpuComponentId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().copied()
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
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().copied()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).copied()
    }

    #[inline]
    fn for_each<F>(self, f: F)
    where
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.copied().for_each(f)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner.copied().fold(init, f)
    }

    #[inline]
    fn all<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.copied().all(f)
    }

    #[inline]
    fn any<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.copied().any(f)
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.copied().find(predicate)
    }

    #[inline]
    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { inner } = self;
        inner.copied().find_map(f)
    }

    #[inline]
    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.copied().position(predicate)
    }

    #[inline]
    fn rposition<P>(&mut self, predicate: P) -> Option<usize>
    where
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.copied().rposition(predicate)
    }
}

impl DoubleEndedIterator for GpuComponentIds<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().copied()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).copied()
    }
}

impl ExactSizeIterator for GpuComponentIds<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for GpuComponentIds<'_> {}
