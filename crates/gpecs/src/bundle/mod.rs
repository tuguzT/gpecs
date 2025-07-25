use gpecs_soa_erased::field::ErasedFieldMutPtr;

use crate::{
    component::registry::{ComponentId, ComponentRegistry},
    soa::traits::Soa,
};

mod impls;

/// Non-empty collection of [components](crate::component::Component).
#[allow(unsafe_code)]
pub unsafe trait Bundle: Soa + 'static {
    /// Static [`Context`](Soa::Context) instance of this bundle.
    ///
    /// This ensures that components of this bundle are known at compile time.
    const CONTEXT: &'static Self::Context;

    /// Order of component identifiers should be the same as
    /// the order of corresponding [descriptors](Soa::FieldDescriptors).
    type MaybeComponentIds: IntoIterator<Item = Option<ComponentId>>;

    fn get_components(components: &ComponentRegistry) -> Self::MaybeComponentIds;

    /// Order of component identifiers should be the same as
    /// the order of corresponding [descriptors](Soa::FieldDescriptors).
    type ComponentIds: IntoIterator<Item = ComponentId>;

    fn register_components(components: &mut ComponentRegistry) -> Self::ComponentIds;

    unsafe fn ptrs_from_iter<I>(components: &ComponentRegistry, iter: I) -> Self::MutPtrs<'static>
    where
        I: IntoIterator<Item = (ComponentId, ErasedFieldMutPtr)>;
}
