use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr, slice,
};

use gpecs_sparse::set::EpochSparseSet;
use wgpu::Device;

use crate::{
    archetype::{
        error::DuplicateComponentError,
        registry::{ArchetypeId, ArchetypeRegistry},
    },
    component::registry::{ComponentId, ComponentRegistry},
    executor::gpu::{
        bundle::GpuBundle,
        component::registry::{GpuComponentId, GpuComponentRegistry},
    },
    soa::identity::Identity,
};

use super::storage::GpuArchetypeStorage;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct GpuArchetypeId(ArchetypeId);

impl GpuArchetypeId {
    #[inline]
    pub const fn into_id(self) -> ArchetypeId {
        let Self(id) = self;
        id
    }

    #[inline]
    pub const fn into_u32(self) -> u32 {
        let Self(id) = self;
        id.into_u32()
    }

    #[inline]
    #[allow(unsafe_code)]
    pub const unsafe fn from_id(id: ArchetypeId) -> Self {
        Self(id)
    }

    #[inline]
    #[allow(unsafe_code)]
    pub const unsafe fn from_u32(id: u32) -> Self {
        let id = unsafe { ArchetypeId::from_u32(id) };
        Self(id)
    }
}

impl From<GpuArchetypeId> for u32 {
    #[inline]
    fn from(value: GpuArchetypeId) -> Self {
        value.into_u32()
    }
}

impl From<GpuArchetypeId> for ArchetypeId {
    #[inline]
    fn from(value: GpuArchetypeId) -> Self {
        let GpuArchetypeId(id) = value;
        id
    }
}

impl Debug for GpuArchetypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = &self.into_u32();
        f.debug_tuple("GpuArchetypeId").field(value).finish()
    }
}

#[derive(Debug)]
pub struct GpuArchetypeInfo {
    id: GpuArchetypeId,
    storage: GpuArchetypeStorage,
}

impl GpuArchetypeInfo {
    #[inline]
    pub fn id(&self) -> GpuArchetypeId {
        let Self { id, .. } = *self;
        id
    }

    #[inline]
    pub fn storage(&self) -> &GpuArchetypeStorage {
        let Self { storage, .. } = self;
        storage
    }

    #[inline]
    pub fn storage_mut(&mut self) -> &mut GpuArchetypeStorage {
        let Self { storage, .. } = self;
        storage
    }
}

type GpuArchetypes = EpochSparseSet<u32, Identity<GpuArchetypeInfo>>;

#[derive(Debug, Default)]
pub struct GpuArchetypeRegistry {
    gpu_archetypes: GpuArchetypes,
}

impl GpuArchetypeRegistry {
    #[inline]
    pub fn new() -> Self {
        Self {
            gpu_archetypes: GpuArchetypes::new(),
        }
    }

    #[inline]
    pub fn register_archetype<B>(
        &mut self,
        components: &mut ComponentRegistry,
        archetypes: &mut ArchetypeRegistry,
        gpu_components: &mut GpuComponentRegistry,
        gpu_device: &Device,
    ) -> Result<GpuArchetypeId, DuplicateComponentError>
    where
        B: GpuBundle,
    {
        let _components = B::register_gpu_components(components, gpu_components);
        let archetype_id = archetypes.register_archetype::<B>(components)?;

        let Self { gpu_archetypes, .. } = self;
        let archetype_id = Self::register(
            components,
            archetypes,
            gpu_archetypes,
            gpu_device,
            archetype_id,
        );
        Ok(archetype_id)
    }

    #[inline]
    pub fn register_archetype_from<I>(
        &mut self,
        components: &ComponentRegistry,
        archetypes: &mut ArchetypeRegistry,
        gpu_device: &Device,
        component_ids: I,
    ) -> Result<GpuArchetypeId, DuplicateComponentError>
    where
        I: IntoIterator<Item = GpuComponentId>,
    {
        let component_ids = component_ids.into_iter().map(ComponentId::from);
        let archetype_id = archetypes.register_archetype_from(components, component_ids)?;

        let Self { gpu_archetypes, .. } = self;
        let archetype_id = Self::register(
            components,
            archetypes,
            gpu_archetypes,
            gpu_device,
            archetype_id,
        );
        Ok(archetype_id)
    }

    #[inline]
    fn register(
        components: &ComponentRegistry,
        archetypes: &ArchetypeRegistry,
        gpu_archetypes: &mut GpuArchetypes,
        gpu_device: &Device,
        archetype_id: ArchetypeId,
    ) -> GpuArchetypeId {
        let gpu_archetype_id = GpuArchetypeId(archetype_id);

        archetypes
            .archetypes_before_inclusive(archetype_id)
            .for_each(|info| {
                let archetype_id = info.id();
                gpu_archetypes
                    .entry(archetype_id.into_u32())
                    .or_insert_with(|| {
                        let id = GpuArchetypeId(archetype_id);
                        let storage = GpuArchetypeStorage::new(components, gpu_device, info);
                        GpuArchetypeInfo { id, storage }.into()
                    });
            });

        gpu_archetype_id
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { gpu_archetypes } = self;
        gpu_archetypes.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { gpu_archetypes } = self;
        gpu_archetypes.is_empty()
    }

    #[inline]
    pub fn get_archetype_info(&self, archetype_id: GpuArchetypeId) -> Option<&GpuArchetypeInfo> {
        let Self { gpu_archetypes } = self;
        gpu_archetypes
            .get(archetype_id.into_u32())
            .map(Identity::as_inner)
    }

    #[inline]
    pub fn contains(&self, id: ArchetypeId) -> bool {
        let Self { gpu_archetypes } = self;
        gpu_archetypes.contains_key(id.into_u32())
    }

    #[inline]
    pub fn map_archetype_id(&self, id: ArchetypeId) -> Option<GpuArchetypeId> {
        self.contains(id).then_some(GpuArchetypeId(id))
    }

    #[inline]
    pub fn archetype_ids(&self) -> GpuArchetypeIds<'_> {
        let Self { gpu_archetypes } = self;

        // SAFETY: `GpuArchetypeId` is a #[repr(transparent)] struct around `ArchetypeId`,
        // which is #[repr(transparent)] around `u32`.
        #[allow(unsafe_code)]
        let archetype_ids = unsafe {
            let slice = gpu_archetypes.as_keys_slice();
            &*(ptr::from_ref(slice) as *const [GpuArchetypeId])
        };
        let inner = archetype_ids.iter();
        GpuArchetypeIds { inner }
    }
}

#[derive(Clone)]
pub struct GpuArchetypeIds<'a> {
    inner: slice::Iter<'a, GpuArchetypeId>,
}

impl Debug for GpuArchetypeIds<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;

        let ids = inner.as_slice();
        f.debug_struct("GpuArchetypeIds")
            .field("ids", &ids)
            .finish()
    }
}

impl Iterator for GpuArchetypeIds<'_> {
    type Item = GpuArchetypeId;

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

impl DoubleEndedIterator for GpuArchetypeIds<'_> {
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

impl ExactSizeIterator for GpuArchetypeIds<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for GpuArchetypeIds<'_> {}
