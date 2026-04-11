use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    slice,
};

use gpecs_sparse::set::EpochSparseSet;
use wgpu::Device;

pub use gpecs_archetype::registry::GpuArchetypeId;

use crate::{
    archetype::{
        erased::error::{ArchetypeError, DuplicateComponentError},
        registry::{ArchetypeId, ArchetypeRegistry},
    },
    component::registry::ComponentId,
    context::Components,
    executor::gpu::{
        bundle::GpuBundle,
        component::registry::{GpuComponentId, GpuComponentRegistry},
    },
    soa::identity::Identity,
};

use super::storage::GpuArchetypeStorage;

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
    pub fn register_archetype_of<B>(
        &mut self,
        components: &mut Components,
        archetypes: &mut ArchetypeRegistry,
        gpu_components: &mut GpuComponentRegistry,
        gpu_device: &Device,
    ) -> Result<GpuArchetypeId, DuplicateComponentError>
    where
        B: GpuBundle,
    {
        let _components = B::register_gpu_components(components, gpu_components);
        let archetype_id = archetypes.register_archetype_of::<B, _, _>(components)?;

        let Self { gpu_archetypes, .. } = self;
        let archetype_id = Self::register(archetypes, gpu_archetypes, gpu_device, archetype_id);
        Ok(archetype_id)
    }

    #[inline]
    pub fn register_archetype_from<I>(
        &mut self,
        components: &Components,
        archetypes: &mut ArchetypeRegistry,
        gpu_device: &Device,
        component_ids: I,
    ) -> Result<GpuArchetypeId, ArchetypeError>
    where
        I: IntoIterator<Item = GpuComponentId>,
    {
        let components = &components.as_view();
        let component_ids = component_ids.into_iter().map(ComponentId::from);
        let archetype_id = archetypes.register_archetype_from(components, component_ids)?;

        let Self { gpu_archetypes, .. } = self;
        let archetype_id = Self::register(archetypes, gpu_archetypes, gpu_device, archetype_id);
        Ok(archetype_id)
    }

    #[inline]
    fn register(
        archetypes: &ArchetypeRegistry,
        gpu_archetypes: &mut GpuArchetypes,
        gpu_device: &Device,
        archetype_id: ArchetypeId,
    ) -> GpuArchetypeId {
        let gpu_archetype_id = gpu_archetype_id_trusted(archetype_id);

        let Some(archetypes_before) = archetypes.archetypes_before_inclusive(archetype_id) else {
            unreachable!("{archetype_id} should be registered prior to this call")
        };
        archetypes_before.for_each(|info| {
            let id = info.id();
            let storage = info.storage();
            gpu_archetypes.entry(id.into_u32()).or_insert_with(|| {
                let id = gpu_archetype_id_trusted(id);
                let storage = GpuArchetypeStorage::new(gpu_device, id, storage);
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
        self.contains(id).then_some(gpu_archetype_id_trusted(id))
    }

    #[inline]
    pub fn archetype_ids(&self) -> GpuArchetypeIds<'_> {
        let Self { gpu_archetypes } = self;

        let inner = gpu_archetypes.as_key_slice().iter();
        GpuArchetypeIds { inner }
    }
}

#[derive(Clone)]
pub struct GpuArchetypeIds<'a> {
    inner: slice::Iter<'a, u32>,
}

impl Debug for GpuArchetypeIds<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl Iterator for GpuArchetypeIds<'_> {
    type Item = GpuArchetypeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().copied().map(gpu_archetype_id_u32_trusted)
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
        inner.last().copied().map(gpu_archetype_id_u32_trusted)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).copied().map(gpu_archetype_id_u32_trusted)
    }

    #[inline]
    fn for_each<F>(self, f: F)
    where
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.copied().map(gpu_archetype_id_u32_trusted).for_each(f);
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner
            .copied()
            .map(gpu_archetype_id_u32_trusted)
            .fold(init, f)
    }

    #[inline]
    fn all<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.copied().map(gpu_archetype_id_u32_trusted).all(f)
    }

    #[inline]
    fn any<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner.copied().map(gpu_archetype_id_u32_trusted).any(f)
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .copied()
            .map(gpu_archetype_id_u32_trusted)
            .find(predicate)
    }

    #[inline]
    fn find_map<B, F>(&mut self, f: F) -> Option<B>
    where
        F: FnMut(Self::Item) -> Option<B>,
    {
        let Self { inner } = self;
        inner.copied().map(gpu_archetype_id_u32_trusted).find_map(f)
    }

    #[inline]
    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        P: FnMut(Self::Item) -> bool,
    {
        let Self { inner } = self;
        inner
            .copied()
            .map(gpu_archetype_id_u32_trusted)
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
            .map(gpu_archetype_id_u32_trusted)
            .rposition(predicate)
    }
}

impl DoubleEndedIterator for GpuArchetypeIds<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().copied().map(gpu_archetype_id_u32_trusted)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).copied().map(gpu_archetype_id_u32_trusted)
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

#[inline]
fn gpu_archetype_id_trusted(id: ArchetypeId) -> GpuArchetypeId {
    unsafe { GpuArchetypeId::from_id(id) }
}

#[inline]
fn gpu_archetype_id_u32_trusted(id: u32) -> GpuArchetypeId {
    unsafe { GpuArchetypeId::from_u32(id) }
}
