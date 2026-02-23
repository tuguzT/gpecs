use crate::{
    component::{
        erased::{ErasedComponentMutPtr, ErasedComponentPtr, error::DowncastErrorKind},
        registry::{ComponentId, ComponentRegistry},
    },
    soa::traits::{AllocSoa, MutPtrs, Ptrs, SoaOwned, SoaRead, SoaWrite},
};

/// Non-empty collection of [components](crate::component::Component).
///
/// # Safety
///
/// Order of component identifiers defined by
/// [`GetComponents`](Bundle::GetComponents) or [`RegisterComponents`](Bundle::RegisterComponents) assotiated types
/// should be the same as the order of corresponding [descriptors](crate::soa::field::FieldDescriptors::Output).
pub unsafe trait Bundle: SoaOwned + AllocSoa + SoaRead + SoaWrite + 'static {
    /// Static [SoA context](crate::soa::traits::SoaContext) instance of this bundle.
    ///
    /// This ensures that components of this bundle are known at compile time.
    const CONTEXT: &'static Self::Context;

    /// Non-empty collection of all already registered components of this bundle.
    ///
    /// If some component was not registered yet,
    /// [`None`] should be returned by its iterator.
    type GetComponents: IntoIterator<Item = Option<ComponentId>>;

    /// Retrieves identifiers of all already registered components of this bundle.
    fn get_components(components: &ComponentRegistry) -> Self::GetComponents;

    /// Non-empty collection of all components of this bundle.
    ///
    /// If some component was not registered yet,
    /// it should be registered by this method and its identifier should be returned by its iterator.
    type RegisterComponents: IntoIterator<Item = ComponentId>;

    /// Registers all components of this bundle inside of provided registry
    /// and returns their identifiers.
    fn register_components(components: &mut ComponentRegistry) -> Self::RegisterComponents;

    /// Attempts to downcast input collection of erased component pointers
    /// into the collection of pointers to components of this bundle.
    ///
    /// Note that the order of input pointers **may not** match
    /// with the order of components in this bundle.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - some of the components of this bundle were not registered,
    /// - some of the input pointers cannot be converted to the component of this bundle.
    fn ptrs_from_erased<I>(
        components: &ComponentRegistry,
        iter: I,
    ) -> Result<Ptrs<'static, Self>, DowncastErrorKind>
    where
        I: IntoIterator<Item = ErasedComponentPtr>;

    /// Attempts to downcast input collection of erased mutable component pointers
    /// into the collection of mutable pointers to components of this bundle.
    ///
    /// Note that the order of input pointers **may not** match
    /// with the order of components in this bundle.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - some of the components of this bundle were not registered,
    /// - some of the input pointers cannot be converted to the component of this bundle.
    fn mut_ptrs_from_erased<I>(
        components: &ComponentRegistry,
        iter: I,
    ) -> Result<MutPtrs<'static, Self>, DowncastErrorKind>
    where
        I: IntoIterator<Item = ErasedComponentMutPtr>;
}
