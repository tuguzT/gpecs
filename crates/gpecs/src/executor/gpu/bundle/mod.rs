use crate::{
    archetype::erased::error::{ArchetypeError, DuplicateComponentError},
    bundle::Bundle,
    context::Components,
    executor::gpu::component::registry::{GpuComponentId, GpuComponentRegistry},
};

mod impls;

/// Non-empty collection of [GPU components](crate::executor::gpu::component::GpuComponent).
pub unsafe trait GpuBundle: Bundle + Copy + Send + Sync {
    /// Non-empty collection of identifiers for all GPU components of this bundle.
    ///
    /// Order of these identifiers should be the same
    /// as the order of corresponding [layouts](gpecs_soa_erased::soa::field::FieldLayouts::Output).
    type GpuComponents: IntoIterator<Item = GpuComponentId>;

    /// Retrieves identifiers of all already registered GPU components of this bundle.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - some of the components of this bundle were not registered,
    /// - some of the components are occuring more than once in the type itself
    ///   (in other words, there are duplicated components).
    fn get_gpu_components(
        components: &Components,
        gpu_components: &GpuComponentRegistry,
    ) -> Result<Self::GpuComponents, ArchetypeError>;

    /// Registers all GPU components of this bundle inside of provided registry
    /// and returns their identifiers.
    ///
    /// # Errors
    ///
    /// This function returns an error if some of the components
    /// are occuring more than once in the type itself (in other words, there are duplicated components).
    fn register_gpu_components(
        components: &mut Components,
        gpu_components: &mut GpuComponentRegistry,
    ) -> Result<Self::GpuComponents, DuplicateComponentError>;
}
