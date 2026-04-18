use gpecs_soa_erased::{
    ErasedSoa, ptr::slice::SliceItemPtrs, soa::field::FieldDescriptor, storage::AlignedStorage,
};

use crate::{
    bundle::erased::{
        ErasedBorrowedViewBundle, ErasedBundleIntoIterKind, ErasedBundleKind,
        traits::ErasedBundleDrop,
    },
    erased::ErasedArchetype,
};

pub type ErasedBundle<Meta, D, S, P> = ErasedBundleKind<ErasedArchetype<Meta>, D, S, P>;
pub type ErasedBorrowedBundle<'a, Meta, D, S, P> =
    ErasedBundleKind<&'a ErasedArchetype<Meta>, D, S, P>;

pub type ErasedBundleIntoIter<S, Meta, F, P> =
    ErasedBundleIntoIterKind<S, ErasedArchetype<Meta>, F, P>;
pub type ErasedBorrowedBundleIntoIter<'a, S, Meta, F, P> =
    ErasedBundleIntoIterKind<S, &'a ErasedArchetype<Meta>, F, P>;

impl<'a, Meta, D, S, P> From<ErasedBorrowedBundle<'a, Meta, D, S, P>>
    for ErasedBorrowedViewBundle<'a, Meta, D, S, P>
where
    Meta: AsRef<FieldDescriptor> + Clone + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    fn from(bundle: ErasedBorrowedBundle<'a, Meta, D, S, P>) -> Self {
        let (storage, archetype) = bundle.into_inner().into_parts();
        let archetype = archetype.as_view();

        let inner = unsafe { ErasedSoa::from_parts(storage, archetype) };
        unsafe { Self::from_inner(inner) }
    }
}

impl<'a, Meta, D, S, P> From<ErasedBorrowedBundle<'a, Meta, D, S, P>>
    for ErasedBundle<Meta, D, S, P>
where
    Meta: AsRef<FieldDescriptor> + Clone + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    fn from(bundle: ErasedBorrowedBundle<'a, Meta, D, S, P>) -> Self {
        let (storage, archetype) = bundle.into_inner().into_parts();
        let archetype = archetype.clone();

        let inner = unsafe { ErasedSoa::from_parts(storage, archetype) };
        unsafe { Self::from_inner(inner) }
    }
}

impl<'a, Meta, D, S, P> From<ErasedBorrowedViewBundle<'a, Meta, D, S, P>>
    for ErasedBundle<Meta, D, S, P>
where
    Meta: AsRef<FieldDescriptor> + Clone + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    fn from(bundle: ErasedBorrowedViewBundle<'a, Meta, D, S, P>) -> Self {
        let (storage, archetype) = bundle.into_inner().into_parts();
        let archetype = archetype.into();

        let inner = unsafe { ErasedSoa::from_parts(storage, archetype) };
        unsafe { Self::from_inner(inner) }
    }
}
