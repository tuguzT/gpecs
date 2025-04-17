use crate::{
    component::registry::ComponentRegistry,
    executor::gpu::component::{
        registry::{GpuComponentId, GpuComponentRegistry},
        GpuComponent,
    },
    soa::identity::Identity,
};

use super::GpuBundle;

impl<T> GpuBundle for Identity<T>
where
    T: GpuComponent,
{
    type MaybeGpuComponentIds = [Option<GpuComponentId>; 1];

    #[inline]
    fn get_gpu_components(
        components: &ComponentRegistry,
        gpu_components: &GpuComponentRegistry,
    ) -> Self::MaybeGpuComponentIds {
        let component_id = components
            .component_id::<T>()
            .map(|id| gpu_components.map_component_id(id))
            .flatten();
        [component_id]
    }

    type GpuComponentIds = [GpuComponentId; 1];

    #[inline]
    fn register_gpu_components(
        components: &mut ComponentRegistry,
        gpu_components: &mut GpuComponentRegistry,
    ) -> Self::GpuComponentIds {
        let component_id = gpu_components.register_component::<T>(components);
        [component_id]
    }
}
