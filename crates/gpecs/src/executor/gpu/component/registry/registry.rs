#![expect(clippy::module_inception)]

use gpecs_sparse::{
    item::{DefaultSparseItem, SparseItem},
    set::EpochSparseSet,
};

use crate::{
    component::registry::ComponentId,
    context::Components,
    executor::gpu::component::{
        GpuComponent,
        registry::{
            GpuComponentId, GpuComponentIds, descriptor::GpuComponentDescriptor,
            id::gpu_component_id_trusted,
        },
    },
};

#[derive(Debug, Default)]
pub struct GpuComponentRegistry<S = DefaultSparseItem<u32>>
where
    S: SparseItem<Index = u32, Epoch = ()>,
{
    components: EpochSparseSet<u32, (), S>,
}

impl<S> GpuComponentRegistry<S>
where
    S: SparseItem<Index = u32, Epoch = ()>,
{
    #[inline]
    pub fn new() -> Self {
        let components = EpochSparseSet::new();
        Self { components }
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
        GpuComponentIds::new(inner)
    }
}
