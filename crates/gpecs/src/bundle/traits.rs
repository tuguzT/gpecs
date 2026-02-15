use std::mem::MaybeUninit;

use gpecs_soa_erased::{
    field::{ErasedFieldMutPtr, ErasedFieldPtr},
    ptr::slice::{ConstSliceItemPtr, MutSliceItemPtr},
};

use crate::{
    bundle::error::PtrsFromIterError,
    component::registry::{ComponentId, ComponentRegistry},
    soa::traits::{AllocSoa, MutPtrs, Ptrs, SoaOwned, SoaRead, SoaWrite},
};

/// Non-empty collection of [components](crate::component::Component).
pub unsafe trait Bundle: SoaOwned + AllocSoa + SoaRead + SoaWrite + 'static {
    /// Static [`Context`](crate::soa::traits::RawSoa::Context) instance of this bundle.
    ///
    /// This ensures that components of this bundle are known at compile time.
    const CONTEXT: &'static Self::Context;

    /// Order of component identifiers should be the same as
    /// the order of corresponding [descriptors](crate::soa::field::FieldDescriptors::Output).
    type MaybeComponentIds: IntoIterator<Item = Option<ComponentId>>;

    fn get_components(components: &ComponentRegistry) -> Self::MaybeComponentIds;

    /// Order of component identifiers should be the same as
    /// the order of corresponding [descriptors](crate::soa::field::FieldDescriptors::Output).
    type ComponentIds: IntoIterator<Item = ComponentId>;

    fn register_components(components: &mut ComponentRegistry) -> Self::ComponentIds;

    fn ptrs_from_erased<I, P>(
        components: &ComponentRegistry,
        iter: I,
    ) -> Result<Ptrs<'static, Self>, PtrsFromIterError<ErasedFieldPtr<P>>>
    where
        I: IntoIterator<Item = (ComponentId, ErasedFieldPtr<P>)>,
        P: ConstSliceItemPtr<Item = MaybeUninit<u8>>;

    fn mut_ptrs_from_erased<I, P>(
        components: &ComponentRegistry,
        iter: I,
    ) -> Result<MutPtrs<'static, Self>, PtrsFromIterError<ErasedFieldMutPtr<P>>>
    where
        I: IntoIterator<Item = (ComponentId, ErasedFieldMutPtr<P>)>,
        P: MutSliceItemPtr<Item = MaybeUninit<u8>>;
}
