use gpecs_component::{
    erased::{ErasedComponentMutPtr, ErasedComponentMutSlicePtr, WithErasedDrop},
    registry::{ComponentId, traits::WithComponentId},
};
use gpecs_soa_erased::{
    BufferOffsetsFrom, BufferOffsetsFromSelf,
    ptr::slice::MutSliceItemPtr,
    soa::{field::FieldLayouts, identity::Identity, layout::WithLayout},
};

use crate::erased::{ErasedArchetypeView, Iter};

pub trait ErasedArchetypeMeta:
    WithLayout
    + for<'a> BufferOffsetsFromSelf<
        BufferOffsets: BufferOffsetsFrom<&'a Self> + BufferOffsetsFrom<(ComponentId, &'a Self)>,
    > + 'static
{
}

impl<T> ErasedArchetypeMeta for T where
    T: WithLayout
        + for<'a> BufferOffsetsFromSelf<
            BufferOffsets: BufferOffsetsFrom<&'a Self> + BufferOffsetsFrom<(ComponentId, &'a Self)>,
        > + ?Sized
        + 'static
{
}

pub trait ErasedArchetypeKind:
    for<'a> FieldLayouts<
        'a,
        Output = ErasedArchetypeView<'a, Self::Meta>,
        OutputIter: ErasedArchetypeIterator,
        OutputItem: BufferOffsetsFromSelf,
    >
{
    type Meta: ErasedArchetypeMeta;
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
    Meta: ErasedArchetypeMeta,
{
    type Meta = Meta;
}

pub unsafe trait ErasedArchetypeIterator:
    Iterator<Item: WithLayout> + for<'a> FieldLayouts<'a, OutputItem: WithComponentId>
{
}

unsafe impl<Meta> ErasedArchetypeIterator for Iter<'_, Meta> where Meta: WithLayout + 'static {}

pub trait IntoErasedArchetypeIterator: IntoIterator<IntoIter: ErasedArchetypeIterator> {}

impl<T> IntoErasedArchetypeIterator for T where
    T: IntoIterator<IntoIter: ErasedArchetypeIterator> + ?Sized
{
}

pub unsafe trait ErasedBundleDrop<Meta> {
    #[inline]
    unsafe fn drop_in_place_with<P>(to_drop: ErasedComponentMutPtr<P>, meta: &Meta)
    where
        P: MutSliceItemPtr,
    {
        let _ = (to_drop, meta);
    }

    #[inline]
    unsafe fn drop_in_place_slice_with<P>(to_drop: ErasedComponentMutSlicePtr<P>, meta: &Meta)
    where
        P: MutSliceItemPtr,
    {
        let _ = (to_drop, meta);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MustNotDrop;

unsafe impl<Meta> ErasedBundleDrop<Meta> for MustNotDrop {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MustDrop;

unsafe impl<Meta> ErasedBundleDrop<Meta> for MustDrop
where
    Meta: WithErasedDrop,
{
    #[inline]
    unsafe fn drop_in_place_with<P>(to_drop: ErasedComponentMutPtr<P>, meta: &Meta)
    where
        P: MutSliceItemPtr,
    {
        let Some(erased_drop) = meta.erased_drop() else {
            return;
        };
        unsafe { erased_drop.drop_in_place(to_drop) }
    }

    #[inline]
    unsafe fn drop_in_place_slice_with<P>(to_drop: ErasedComponentMutSlicePtr<P>, meta: &Meta)
    where
        P: MutSliceItemPtr,
    {
        let Some(erased_drop) = meta.erased_drop() else {
            return;
        };
        unsafe { erased_drop.drop_in_place_slice(to_drop) }
    }
}
