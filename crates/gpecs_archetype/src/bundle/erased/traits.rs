use gpecs_component::{erased::WithErasedDrop, registry::traits::WithComponentId};
use gpecs_soa_erased::{
    ptr::slice::MutSliceItemPtr,
    soa::{
        field::{FieldDescriptor, FieldDescriptors},
        identity::Identity,
    },
};
use itertools::zip_eq;

use crate::{
    bundle::erased::{ErasedBundleMutPtrs, ErasedBundleMutSlicePtrs},
    erased::{ErasedArchetypeView, Iter},
};

pub trait ErasedArchetypeKind:
    for<'a> FieldDescriptors<'a, Output = ErasedArchetypeView<'a, Self::Meta>>
{
    type Meta: AsRef<FieldDescriptor> + 'static;
}

impl<T> ErasedArchetypeKind for &T
where
    T: ErasedArchetypeKind + ?Sized,
{
    type Meta = T::Meta;
}

impl<T> ErasedArchetypeKind for Identity<T>
where
    T: ErasedArchetypeKind + ?Sized,
{
    type Meta = T::Meta;
}

impl<Meta> ErasedArchetypeKind for ErasedArchetypeView<'_, Meta>
where
    Meta: AsRef<FieldDescriptor> + 'static,
{
    type Meta = Meta;
}

pub unsafe trait ErasedArchetypeIterator:
    Iterator<Item: AsRef<FieldDescriptor>>
    + for<'a> FieldDescriptors<'a, Output: IntoIterator<Item: WithComponentId>>
{
}

unsafe impl<Meta> ErasedArchetypeIterator for Iter<'_, Meta> where
    Meta: AsRef<FieldDescriptor> + 'static
{
}

pub trait IntoErasedArchetypeIterator: IntoIterator<IntoIter: ErasedArchetypeIterator> {}

impl<T> IntoErasedArchetypeIterator for T where
    T: IntoIterator<IntoIter: ErasedArchetypeIterator> + ?Sized
{
}

pub unsafe trait ErasedBundleDrop<Meta> {
    unsafe fn ptrs_drop_in_place<T, U, P>(archetype: &T, ptrs: &mut ErasedBundleMutPtrs<U, P>)
    where
        T: ErasedArchetypeKind<Meta = Meta> + ?Sized,
        U: ErasedArchetypeKind + ?Sized,
        P: MutSliceItemPtr;

    unsafe fn slices_drop_in_place<T, U, P>(
        archetype: &T,
        slices: &mut ErasedBundleMutSlicePtrs<U, P>,
    ) where
        T: ErasedArchetypeKind<Meta = Meta> + ?Sized,
        U: ErasedArchetypeKind + ?Sized,
        P: MutSliceItemPtr;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MustNotDrop;

unsafe impl<Meta> ErasedBundleDrop<Meta> for MustNotDrop {
    #[inline]
    unsafe fn ptrs_drop_in_place<T, U, P>(_: &T, _: &mut ErasedBundleMutPtrs<U, P>)
    where
        T: ErasedArchetypeKind<Meta = Meta> + ?Sized,
        U: ErasedArchetypeKind + ?Sized,
        P: MutSliceItemPtr,
    {
    }

    #[inline]
    unsafe fn slices_drop_in_place<T, U, P>(_: &T, _: &mut ErasedBundleMutSlicePtrs<U, P>)
    where
        T: ErasedArchetypeKind<Meta = Meta> + ?Sized,
        U: ErasedArchetypeKind + ?Sized,
        P: MutSliceItemPtr,
    {
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MustDrop;

unsafe impl<Meta> ErasedBundleDrop<Meta> for MustDrop
where
    Meta: WithErasedDrop,
{
    #[inline]
    unsafe fn ptrs_drop_in_place<T, U, P>(archetype: &T, ptrs: &mut ErasedBundleMutPtrs<U, P>)
    where
        T: ErasedArchetypeKind<Meta = Meta> + ?Sized,
        U: ErasedArchetypeKind + ?Sized,
        P: MutSliceItemPtr,
    {
        let archetype = archetype.field_descriptors();
        for (component_info, to_drop) in zip_eq(archetype, ptrs) {
            let Some(erased_drop) = component_info.erased_drop() else {
                continue;
            };
            unsafe { erased_drop.drop_in_place(to_drop) }
        }
    }

    #[inline]
    unsafe fn slices_drop_in_place<T, U, P>(
        archetype: &T,
        slices: &mut ErasedBundleMutSlicePtrs<U, P>,
    ) where
        T: ErasedArchetypeKind<Meta = Meta> + ?Sized,
        U: ErasedArchetypeKind + ?Sized,
        P: MutSliceItemPtr,
    {
        let archetype = archetype.field_descriptors();
        for (component_info, to_drop) in zip_eq(archetype, slices) {
            let Some(erased_drop) = component_info.erased_drop() else {
                continue;
            };
            unsafe { erased_drop.drop_in_place_slice(to_drop) }
        }
    }
}
