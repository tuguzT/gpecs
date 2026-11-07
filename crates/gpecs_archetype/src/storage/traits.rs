use core::ops::Deref;

use gpecs_soa_erased::{
    ptr::slice::{ConstPtr, MutPtr, NonNullPtr, SliceItemPtrs},
    soa::traits::{SoaAlloc, SoaContext, SoaOwned},
    storage::AlignedStorage,
};

use crate::{
    bundle::erased::{
        ErasedBorrowedViewBundle, ErasedBundleMutPtrs, ErasedBundleMutRefs,
        ErasedBundleMutSlicePtrs, ErasedBundleMutSlices, ErasedBundleNonNullPtrs,
        ErasedBundleNonNullSlicePtrs, ErasedBundlePtrs, ErasedBundleRefs, ErasedBundleSlicePtrs,
        ErasedBundleSlices,
        traits::{ErasedArchetypeKind, ErasedArchetypeMeta, ErasedBundleDrop},
    },
    erased::ErasedArchetypeView,
};

pub trait ErasedArchetypeSoa:
    SoaOwned<
        Context: for<'a, 'data> SoaContext<
            'data,
            Self,
            Ptrs<'a> = ErasedArchetypeSoaPtrs<'a, Self>,
            MutPtrs<'a> = ErasedArchetypeSoaMutPtrs<'a, Self>,
            NonNullPtrs<'a> = ErasedArchetypeSoaNonNullPtrs<'a, Self>,
            SlicePtrs<'a> = ErasedArchetypeSoaSlicePtrs<'a, Self>,
            SliceMutPtrs<'a> = ErasedArchetypeSoaMutSlicePtrs<'a, Self>,
            SliceNonNullPtrs<'a> = ErasedArchetypeSoaNonNullSlicePtrs<'a, Self>,
            Refs<'a> = ErasedArchetypeSoaRefs<'a, 'data, Self>,
            RefsMut<'a> = ErasedArchetypeSoaMutRefs<'a, 'data, Self>,
            Slices<'a> = ErasedArchetypeSoaSlices<'a, 'data, Self>,
            SlicesMut<'a> = ErasedArchetypeSoaSlicesMut<'a, 'data, Self>,
        > + Deref<Target: ErasedArchetypeKind<Meta = Self::Meta>>,
    > + SoaAlloc
{
    type Meta: ErasedArchetypeMeta;
    type Archetype<'a>: ErasedArchetypeKind<Meta = Self::Meta>;
    type DropKind: ErasedBundleDrop<Self::Meta>;
    type Ptrs: SliceItemPtrs;
}

pub type ErasedArchetypeSoaPtrs<'a, T> = ErasedBundlePtrs<
    <T as ErasedArchetypeSoa>::Archetype<'a>,
    ConstPtr<<T as ErasedArchetypeSoa>::Ptrs>,
>;
pub type ErasedArchetypeSoaMutPtrs<'a, T> = ErasedBundleMutPtrs<
    <T as ErasedArchetypeSoa>::Archetype<'a>,
    MutPtr<<T as ErasedArchetypeSoa>::Ptrs>,
>;
pub type ErasedArchetypeSoaNonNullPtrs<'a, T> = ErasedBundleNonNullPtrs<
    <T as ErasedArchetypeSoa>::Archetype<'a>,
    NonNullPtr<<T as ErasedArchetypeSoa>::Ptrs>,
>;
pub type ErasedArchetypeSoaSlicePtrs<'a, T> = ErasedBundleSlicePtrs<
    <T as ErasedArchetypeSoa>::Archetype<'a>,
    ConstPtr<<T as ErasedArchetypeSoa>::Ptrs>,
>;
pub type ErasedArchetypeSoaMutSlicePtrs<'a, T> = ErasedBundleMutSlicePtrs<
    <T as ErasedArchetypeSoa>::Archetype<'a>,
    MutPtr<<T as ErasedArchetypeSoa>::Ptrs>,
>;
pub type ErasedArchetypeSoaNonNullSlicePtrs<'a, T> = ErasedBundleNonNullSlicePtrs<
    <T as ErasedArchetypeSoa>::Archetype<'a>,
    NonNullPtr<<T as ErasedArchetypeSoa>::Ptrs>,
>;
pub type ErasedArchetypeSoaRefs<'a, 'data, T> = ErasedBundleRefs<
    'data,
    <T as ErasedArchetypeSoa>::Archetype<'a>,
    NonNullPtr<<T as ErasedArchetypeSoa>::Ptrs>,
>;
pub type ErasedArchetypeSoaMutRefs<'a, 'data, T> = ErasedBundleMutRefs<
    'data,
    <T as ErasedArchetypeSoa>::Archetype<'a>,
    NonNullPtr<<T as ErasedArchetypeSoa>::Ptrs>,
>;
pub type ErasedArchetypeSoaSlices<'a, 'data, T> = ErasedBundleSlices<
    'data,
    <T as ErasedArchetypeSoa>::Archetype<'a>,
    NonNullPtr<<T as ErasedArchetypeSoa>::Ptrs>,
>;
pub type ErasedArchetypeSoaSlicesMut<'a, 'data, T> = ErasedBundleMutSlices<
    'data,
    <T as ErasedArchetypeSoa>::Archetype<'a>,
    NonNullPtr<<T as ErasedArchetypeSoa>::Ptrs>,
>;

impl<'view, Meta, D, S, P> ErasedArchetypeSoa for ErasedBorrowedViewBundle<'view, Meta, D, S, P>
where
    Meta: ErasedArchetypeMeta,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorage<Item: 'static>,
    P: SliceItemPtrs<Item = S::Item>,
{
    type Meta = Meta;
    type Archetype<'a> = ErasedArchetypeView<'view, Meta>;
    type DropKind = D;
    type Ptrs = P;
}
