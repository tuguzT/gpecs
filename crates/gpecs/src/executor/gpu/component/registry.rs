use std::{
    alloc::Layout,
    any::{self, TypeId},
    borrow::Cow,
    fmt::{self, Debug},
    iter::FusedIterator,
    slice,
};

use gpecs_sparse::set::EpochSparseSet;

pub use gpecs_component::registry::GpuComponentId;

use crate::{
    component::registry::ComponentId,
    context::{ComponentDescriptor, Components},
};

use super::GpuComponent;

#[derive(Debug, Clone)]
pub struct GpuComponentDescriptor {
    name: Cow<'static, str>,
    type_id: Option<TypeId>,
    layout: Layout,
}

impl GpuComponentDescriptor {
    #[inline]
    pub fn new<N, I>(name: N, type_id: I, layout: Layout) -> Self
    where
        N: Into<Cow<'static, str>>,
        I: Into<Option<TypeId>>,
    {
        Self {
            name: name.into(),
            type_id: type_id.into(),
            layout,
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
            layout: Layout::new::<T>(),
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
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }
}

impl From<GpuComponentDescriptor> for ComponentDescriptor {
    fn from(value: GpuComponentDescriptor) -> Self {
        let GpuComponentDescriptor {
            name,
            type_id,
            layout,
        } = value;
        Self::new(name, type_id, layout, None)
    }
}

#[derive(Debug, Default)]
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
    pub fn register_component<T>(&mut self, components: &mut Components) -> GpuComponentId
    where
        T: GpuComponent,
    {
        let id = components.register_component::<T>();
        let id = gpu_component_id_trusted(id);

        let Self { components } = self;
        components.insert(id.into_u32(), ());

        id
    }

    #[inline]
    pub fn register_component_with(
        &mut self,
        components: &mut Components,
        descriptor: GpuComponentDescriptor,
    ) -> GpuComponentId {
        let id = components.register_component_with(descriptor.into());
        let id = gpu_component_id_trusted(id);

        let Self { components } = self;
        components.insert(id.into_u32(), ());

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
        components.contains_key(id.into_u32())
    }

    #[inline]
    pub fn map_component_id(&self, id: ComponentId) -> Option<GpuComponentId> {
        self.contains(id).then_some(gpu_component_id_trusted(id))
    }

    #[inline]
    pub fn component_ids(&self) -> GpuComponentIds<'_> {
        let Self { components } = self;

        let inner = components.as_key_slice().iter();
        GpuComponentIds { inner }
    }
}

#[derive(Clone)]
pub struct GpuComponentIds<'a> {
    inner: slice::Iter<'a, u32>,
}

impl Debug for GpuComponentIds<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl Iterator for GpuComponentIds<'_> {
    type Item = GpuComponentId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().copied().map(gpu_component_id_u32_trusted)
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
        inner.last().copied().map(gpu_component_id_u32_trusted)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).copied().map(gpu_component_id_u32_trusted)
    }

    #[inline]
    fn for_each<F>(self, f: F)
    where
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.copied().map(gpu_component_id_u32_trusted).for_each(f);
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner
            .copied()
            .map(gpu_component_id_u32_trusted)
            .fold(init, f)
    }

    #[inline]
    fn all<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.copied().map(gpu_component_id_u32_trusted).all(f)
    }

    #[inline]
    fn any<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.copied().map(gpu_component_id_u32_trusted).any(f)
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .copied()
            .map(gpu_component_id_u32_trusted)
            .find(predicate)
    }

    #[inline]
    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { inner } = self;
        inner.copied().map(gpu_component_id_u32_trusted).find_map(f)
    }

    #[inline]
    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .copied()
            .map(gpu_component_id_u32_trusted)
            .position(predicate)
    }

    #[inline]
    fn rposition<P>(&mut self, predicate: P) -> Option<usize>
    where
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .copied()
            .map(gpu_component_id_u32_trusted)
            .rposition(predicate)
    }
}

impl DoubleEndedIterator for GpuComponentIds<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().copied().map(gpu_component_id_u32_trusted)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).copied().map(gpu_component_id_u32_trusted)
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

#[inline]
fn gpu_component_id_trusted(id: ComponentId) -> GpuComponentId {
    unsafe { GpuComponentId::from_id(id) }
}

#[inline]
fn gpu_component_id_u32_trusted(id: u32) -> GpuComponentId {
    unsafe { GpuComponentId::from_u32(id) }
}
