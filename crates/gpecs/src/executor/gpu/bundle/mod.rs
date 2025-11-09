use crate::{
    bundle::Bundle,
    component::registry::ComponentRegistry,
    executor::gpu::component::registry::{GpuComponentId, GpuComponentRegistry},
};

mod impls;

pub unsafe trait GpuBundle: Bundle + Copy + Send + Sync {
    /// Order of component identifiers should be the same as
    /// the order of corresponding [descriptors](crate::soa::traits::Soa::FieldDescriptors).
    type MaybeGpuComponentIds: IntoIterator<Item = Option<GpuComponentId>>;

    fn get_gpu_components(
        components: &ComponentRegistry,
        gpu_components: &GpuComponentRegistry,
    ) -> Self::MaybeGpuComponentIds;

    /// Order of component identifiers should be the same as
    /// the order of corresponding [descriptors](crate::soa::traits::Soa::FieldDescriptors).
    type GpuComponentIds: IntoIterator<Item = GpuComponentId>;

    fn register_gpu_components(
        components: &mut ComponentRegistry,
        gpu_components: &mut GpuComponentRegistry,
    ) -> Self::GpuComponentIds;
}
