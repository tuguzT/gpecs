#![expect(clippy::module_inception)]

use gpecs_sparse::{
    item::{DefaultSparseItem, SparseItem},
    set::EpochSparseSet,
};
use wgpu::Device;

use crate::{
    archetype::{
        erased::error::{ArchetypeError, DuplicateComponentError},
        registry::{ArchetypeId, ArchetypeRegistry},
    },
    component::registry::ComponentId,
    context::Components,
    executor::gpu::{
        archetype::{
            registry::{GpuArchetypeId, GpuArchetypeIds, id::gpu_archetype_id_trusted},
            storage::GpuArchetypeStorage,
        },
        bundle::GpuBundle,
        component::registry::{GpuComponentId, GpuComponentRegistry},
    },
    soa::identity::Identity,
};

type Inner<S> = EpochSparseSet<u32, Identity<GpuArchetypeStorage>, S>;

#[derive(Debug, Default)]
pub struct GpuArchetypeRegistry<S = DefaultSparseItem<u32>>
where
    S: SparseItem<Index = u32, Epoch = ()>,
{
    gpu_archetypes: Inner<S>,
}

impl<S> GpuArchetypeRegistry<S>
where
    S: SparseItem<Index = u32, Epoch = ()>,
{
    #[inline]
    pub fn new() -> Self {
        let gpu_archetypes = Inner::new();
        Self { gpu_archetypes }
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
        let _components = B::register_gpu_components(components, gpu_components)?;
        let archetype_id = archetypes.register_archetype_of::<B>(components)?;

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
        gpu_archetypes: &mut Inner<S>,
        gpu_device: &Device,
        archetype_id: ArchetypeId,
    ) -> GpuArchetypeId {
        let gpu_archetype_id = gpu_archetype_id_trusted(archetype_id);

        let Some(archetypes_before) = archetypes.archetypes_before_inclusive(archetype_id) else {
            unreachable!("{archetype_id} should be registered prior to this call")
        };
        archetypes_before.for_each(|(id, storage)| {
            let id = gpu_archetype_id_trusted(id);
            gpu_archetypes
                .entry(id.into_u32())
                .or_insert_with(|| GpuArchetypeStorage::new(gpu_device, id, storage).into());
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
    pub fn get_archetype_storage(
        &self,
        archetype_id: GpuArchetypeId,
    ) -> Option<&GpuArchetypeStorage> {
        let Self { gpu_archetypes } = self;
        gpu_archetypes
            .get(archetype_id.into_u32())
            .map(Identity::as_inner)
    }

    #[inline]
    pub unsafe fn get_archetype_storage_mut(
        &mut self,
        archetype_id: GpuArchetypeId,
    ) -> Option<&mut GpuArchetypeStorage> {
        let Self { gpu_archetypes } = self;
        gpu_archetypes
            .get_mut(archetype_id.into_u32())
            .map(Identity::as_inner_mut)
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
        GpuArchetypeIds::new(inner)
    }
}
