use crate::{
    bundle::Bundle,
    context::Components,
    executor::gpu::component::registry::{GpuComponentId, GpuComponentRegistry},
};

mod impls;

/// Non-empty collection of [GPU components](crate::executor::gpu::component::GpuComponent).
///
/// # Safety
///
/// Order of component identifiers defined by [`GetGpuComponents`](GpuBundle::GetGpuComponents) and [`RegisterGpuComponents`](GpuBundle::RegisterGpuComponents)
/// should be the same as the order of corresponding [descriptors](crate::soa::field::FieldDescriptors::Output).
pub unsafe trait GpuBundle: Bundle + Copy + Send + Sync {
    /// Non-empty collection of all already registered GPU components of this bundle.
    ///
    /// If some component was not registered yet,
    /// [`None`] should be returned by its iterator.
    type GetGpuComponents: IntoIterator<Item = Option<GpuComponentId>>;

    /// Retrieves identifiers of all already registered GPU components of this bundle.
    fn get_gpu_components(
        components: &Components,
        gpu_components: &GpuComponentRegistry,
    ) -> Self::GetGpuComponents;

    /// Non-empty collection of all GPU components of this bundle.
    ///
    /// If some component was not registered yet,
    /// it should be registered by this method and its identifier should be returned by its iterator.
    type RegisterGpuComponents: IntoIterator<Item = GpuComponentId>;

    /// Registers all GPU components of this bundle inside of provided registry
    /// and returns their identifiers.
    fn register_gpu_components(
        components: &mut Components,
        gpu_components: &mut GpuComponentRegistry,
    ) -> Self::RegisterGpuComponents;
}
