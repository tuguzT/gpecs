use gpecs_soa_erased::field::ErasedFieldMutPtr;

use crate::{
    component::registry::{ComponentId, ComponentRegistry},
    soa::traits::{MutPtrs, Soa, SoaRead, SoaWrite},
};

mod impls;

/// Non-empty collection of [components](crate::component::Component).
pub unsafe trait Bundle: Soa + SoaRead + SoaWrite + 'static {
    /// Static [`Context`](Soa::Context) instance of this bundle.
    ///
    /// This ensures that components of this bundle are known at compile time.
    const CONTEXT: &'static Self::Context;

    /// Order of component identifiers should be the same as
    /// the order of corresponding [descriptors](crate::soa::traits::RawSoaContext::FieldDescriptors).
    type MaybeComponentIds: IntoIterator<Item = Option<ComponentId>>;

    fn get_components(components: &ComponentRegistry) -> Self::MaybeComponentIds;

    /// Order of component identifiers should be the same as
    /// the order of corresponding [descriptors](crate::soa::traits::RawSoaContext::FieldDescriptors).
    type ComponentIds: IntoIterator<Item = ComponentId>;

    fn register_components(components: &mut ComponentRegistry) -> Self::ComponentIds;

    unsafe fn ptrs_from_iter<I>(components: &ComponentRegistry, iter: I) -> MutPtrs<'static, Self>
    where
        I: IntoIterator<Item = (ComponentId, ErasedFieldMutPtr)>;
}
