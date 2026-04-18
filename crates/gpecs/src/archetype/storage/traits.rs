use std::ops::Deref;

use gpecs_archetype::bundle::erased::{
    ErasedBorrowedViewBundle, ErasedBundle, ErasedBundleMutRefs, ErasedBundleMutSlices,
    ErasedBundleRefs, ErasedBundleSlices, traits::ErasedBundleDrop,
};
use gpecs_soa_erased::{
    ptr::slice::{ConstPtr, MutPtr, NonNullPtr, SliceItemPtrs},
    storage::AlignedStorage,
};

use crate::{
    archetype::erased::{ErasedArchetype, ErasedArchetypeView},
    bundle::erased::{
        ErasedBundleMutPtrs, ErasedBundleMutSlicePtrs, ErasedBundleNonNullPtrs, ErasedBundlePtrs,
        ErasedBundleSlicePtrs, traits::ErasedArchetypeKind,
    },
    soa::{
        field::FieldDescriptor,
        traits::{AllocSoa, SoaContext, SoaOwned},
    },
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
    type Meta: AsRef<FieldDescriptor> + 'static;
    type Archetype<'a>: ErasedArchetypeKind<Meta = Self::Meta>;
    type Ptrs: SliceItemPtrs;
}

impl<'view, Meta, D, S, P> ErasedArchetypeSoa for ErasedBorrowedViewBundle<'view, Meta, D, S, P>
where
    Meta: AsRef<FieldDescriptor> + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage<Item: 'static>,
    P: SliceItemPtrs<Item = S::Item>,
{
    type Meta = Meta;
    type Archetype<'a> = ErasedArchetypeView<'view, Meta>;
    type Ptrs = P;
}

impl<Meta, D, S, P> ErasedArchetypeSoa for ErasedBundle<Meta, D, S, P>
where
    Meta: AsRef<FieldDescriptor> + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage<Item: 'static>,
    P: SliceItemPtrs<Item = S::Item>,
{
    type Meta = Meta;
    type Archetype<'a> = &'a ErasedArchetype<Meta>;
    type Ptrs = P;
}
