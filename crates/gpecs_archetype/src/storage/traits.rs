use core::ops::Deref;

use gpecs_soa_erased::{
    ptr::slice::{ConstPtr, MutPtr, NonNullPtr, SliceItemPtrs},
    soa::{
        layout::WithLayout,
        traits::{AllocSoa, SoaContext, SoaOwned},
    },
    storage::AlignedStorage,
};

use crate::{
    bundle::erased::{
        ErasedBorrowedViewBundle, ErasedBundleMutPtrs, ErasedBundleMutRefs,
        ErasedBundleMutSlicePtrs, ErasedBundleMutSlices, ErasedBundleNonNullPtrs, ErasedBundlePtrs,
        ErasedBundleRefs, ErasedBundleSlicePtrs, ErasedBundleSlices, traits::ErasedArchetypeKind,
        traits::ErasedBundleDrop,
    },
    erased::ErasedArchetypeView,
};

pub trait ErasedArchetypeSoa:
    SoaOwned<
        Context: for<'data, 'a> SoaContext<
            'data,
            Self,
            Ptrs<'a> = ErasedBundlePtrs<Self::Archetype<'a>, ConstPtr<Self::Ptrs>>,
            MutPtrs<'a> = ErasedBundleMutPtrs<Self::Archetype<'a>, MutPtr<Self::Ptrs>>,
            NonNullPtrs<'a> = ErasedBundleNonNullPtrs<Self::Archetype<'a>, NonNullPtr<Self::Ptrs>>,
            SlicePtrs<'a> = ErasedBundleSlicePtrs<Self::Archetype<'a>, ConstPtr<Self::Ptrs>>,
            SliceMutPtrs<'a> = ErasedBundleMutSlicePtrs<Self::Archetype<'a>, MutPtr<Self::Ptrs>>,
            Refs<'a> = ErasedBundleRefs<'data, Self::Archetype<'a>, ConstPtr<Self::Ptrs>>,
            RefsMut<'a> = ErasedBundleMutRefs<'data, Self::Archetype<'a>, MutPtr<Self::Ptrs>>,
            Slices<'a> = ErasedBundleSlices<'data, Self::Archetype<'a>, ConstPtr<Self::Ptrs>>,
            SlicesMut<'a> = ErasedBundleMutSlices<'data, Self::Archetype<'a>, MutPtr<Self::Ptrs>>,
        > + Deref<Target: ErasedArchetypeKind<Meta = Self::Meta>>,
    > + AllocSoa
{
    type Meta: WithLayout + 'static;
    type Archetype<'a>: ErasedArchetypeKind<Meta = Self::Meta>;
    type DropKind: ErasedBundleDrop<Self::Meta>;
    type Ptrs: SliceItemPtrs;
}

impl<'view, Meta, D, S, P> ErasedArchetypeSoa for ErasedBorrowedViewBundle<'view, Meta, D, S, P>
where
    Meta: WithLayout + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage<Item: 'static>,
    P: SliceItemPtrs<Item = S::Item>,
{
    type Meta = Meta;
    type Archetype<'a> = ErasedArchetypeView<'view, Meta>;
    type DropKind = D;
    type Ptrs = P;
}
