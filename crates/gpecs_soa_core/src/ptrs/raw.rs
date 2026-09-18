use core::mem::ManuallyDrop;

use crate::{
    ptrs::SlicePtrsIndex,
    refs, slices,
    traits::{
        CloneToUninitSoaContext, MutPtrs, NonNullPtrs, Ptrs, RawSoa, RawSoaContext, ReadSoaContext,
        Refs, RefsMut, SliceMutPtrs, SlicePtrs, Slices, SlicesMut, Soa, SoaCloneToUninit, SoaRead,
        SoaWrite, WriteSoaContext,
    },
};

#[inline]
pub fn upcast<'short, 'long: 'short, T>(from: Ptrs<'long, T>) -> Ptrs<'short, T>
where
    T: RawSoa + ?Sized,
{
    T::Context::ptrs_upcast(from)
}

#[inline]
pub fn dangling<T>(context: &T::Context) -> Ptrs<'_, T>
where
    T: RawSoa + ?Sized,
{
    context.ptrs_dangling()
}

#[inline]
pub unsafe fn add<'a, T>(context: &'a T::Context, ptrs: Ptrs<'a, T>, count: usize) -> Ptrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    unsafe { context.ptrs_add(ptrs, count) }
}

#[inline]
pub unsafe fn offset_from<T>(context: &T::Context, ptrs: Ptrs<'_, T>, origin: Ptrs<'_, T>) -> isize
where
    T: RawSoa + ?Sized,
{
    unsafe { context.ptrs_offset_from(ptrs, origin) }
}

#[inline]
pub fn upcast_mut<'short, 'long: 'short, T>(from: MutPtrs<'long, T>) -> MutPtrs<'short, T>
where
    T: RawSoa + ?Sized,
{
    T::Context::mut_ptrs_upcast(from)
}

#[inline]
pub fn dangling_mut<T>(context: &T::Context) -> MutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
{
    context.mut_ptrs_dangling()
}

#[inline]
pub unsafe fn add_mut<'a, T>(
    context: &'a T::Context,
    ptrs: MutPtrs<'a, T>,
    count: usize,
) -> MutPtrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    unsafe { context.mut_ptrs_add(ptrs, count) }
}

#[inline]
pub unsafe fn offset_from_mut<T>(
    context: &T::Context,
    ptrs: MutPtrs<'_, T>,
    origin: Ptrs<'_, T>,
) -> isize
where
    T: RawSoa + ?Sized,
{
    unsafe { context.mut_ptrs_offset_from(ptrs, origin) }
}

#[inline]
pub fn cast_const<'a, T>(context: &'a T::Context, ptrs: MutPtrs<'a, T>) -> Ptrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    context.ptrs_cast_const(ptrs)
}

#[inline]
pub fn cast_mut<'a, T>(context: &'a T::Context, ptrs: Ptrs<'a, T>) -> MutPtrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    context.ptrs_cast_mut(ptrs)
}

#[inline]
pub unsafe fn swap_nonoverlapping<T>(
    context: &T::Context,
    x: MutPtrs<'_, T>,
    y: MutPtrs<'_, T>,
    count: usize,
) where
    T: RawSoa + ?Sized,
{
    unsafe { context.ptrs_swap_nonoverlapping(x, y, count) }
}

#[inline]
pub unsafe fn copy_nonoverlapping<T>(
    context: &T::Context,
    src: Ptrs<'_, T>,
    dst: MutPtrs<'_, T>,
    count: usize,
) where
    T: RawSoa + ?Sized,
{
    unsafe { context.ptrs_copy_nonoverlapping(src, dst, count) }
}

#[inline]
pub unsafe fn drop_in_place<T>(context: &T::Context, to_drop: MutPtrs<'_, T>)
where
    T: RawSoa + ?Sized,
{
    unsafe { context.ptrs_drop_in_place(to_drop) }
}

#[inline]
pub fn upcast_nonnull<'short, 'long: 'short, T>(
    from: NonNullPtrs<'long, T>,
) -> NonNullPtrs<'short, T>
where
    T: RawSoa + ?Sized,
{
    T::Context::nonnull_ptrs_upcast(from)
}

#[inline]
pub fn dangling_nonnull<T>(context: &T::Context) -> NonNullPtrs<'_, T>
where
    T: RawSoa + ?Sized,
{
    context.nonnull_ptrs_dangling()
}

#[inline]
pub unsafe fn nonnull_from_mut<'a, T>(
    context: &'a T::Context,
    ptrs: MutPtrs<'a, T>,
) -> NonNullPtrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    unsafe { context.nonnull_ptrs_from_mut_ptrs(ptrs) }
}

#[inline]
pub fn nonnull_as_ptrs<'a, T>(context: &'a T::Context, ptrs: NonNullPtrs<'a, T>) -> Ptrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    context.nonnull_ptrs_as_ptrs(ptrs)
}

#[inline]
pub fn nonnull_as_mut_ptrs<'a, T>(
    context: &'a T::Context,
    ptrs: NonNullPtrs<'a, T>,
) -> MutPtrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    context.nonnull_ptrs_as_mut_ptrs(ptrs)
}

#[inline]
pub fn upcast_slices<'short, 'long: 'short, T>(from: SlicePtrs<'long, T>) -> SlicePtrs<'short, T>
where
    T: RawSoa + ?Sized,
{
    T::Context::slice_ptrs_upcast(from)
}

#[inline]
pub fn slices_from_raw_parts<'a, T>(
    context: &'a T::Context,
    data: Ptrs<'a, T>,
    len: usize,
) -> SlicePtrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    context.slice_ptrs_from_raw_parts(data, len)
}

#[inline]
pub fn slices_len<T>(context: &T::Context, slices: &SlicePtrs<'_, T>) -> usize
where
    T: RawSoa + ?Sized,
{
    context.slice_ptrs_len(slices)
}

#[inline]
pub fn slices_as_ptrs<'a, T>(context: &'a T::Context, slices: SlicePtrs<'a, T>) -> Ptrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    context.slice_ptrs_as_ptrs(slices)
}

#[inline]
pub fn upcast_slices_mut<'short, 'long: 'short, T>(
    from: SliceMutPtrs<'long, T>,
) -> SliceMutPtrs<'short, T>
where
    T: RawSoa + ?Sized,
{
    T::Context::mut_slice_ptrs_upcast(from)
}

#[inline]
pub fn slices_from_raw_parts_mut<'a, T>(
    context: &'a T::Context,
    data: MutPtrs<'a, T>,
    len: usize,
) -> SliceMutPtrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    context.mut_slice_ptrs_from_raw_parts(data, len)
}

#[inline]
pub fn slices_len_mut<T>(context: &T::Context, slices: &SliceMutPtrs<'_, T>) -> usize
where
    T: RawSoa + ?Sized,
{
    context.mut_slice_ptrs_len(slices)
}

#[inline]
pub fn mut_slices_as_ptrs<'a, T>(
    context: &'a T::Context,
    slices: SliceMutPtrs<'a, T>,
) -> Ptrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    context.mut_slice_ptrs_as_ptrs(slices)
}

#[inline]
pub fn mut_slices_as_mut_ptrs<'a, T>(
    context: &'a T::Context,
    slices: SliceMutPtrs<'a, T>,
) -> MutPtrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    context.mut_slice_ptrs_as_mut_ptrs(slices)
}

#[inline]
pub fn slices_cast_const<'a, T>(
    context: &'a T::Context,
    slices: SliceMutPtrs<'a, T>,
) -> SlicePtrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    context.slice_ptrs_cast_const(slices)
}

#[inline]
pub fn slices_cast_mut<'a, T>(
    context: &'a T::Context,
    slices: SlicePtrs<'a, T>,
) -> SliceMutPtrs<'a, T>
where
    T: RawSoa + ?Sized,
{
    context.slice_ptrs_cast_mut(slices)
}

#[inline]
pub unsafe fn slices_drop_in_place<T>(context: &T::Context, slices_to_drop: SliceMutPtrs<'_, T>)
where
    T: RawSoa + ?Sized,
{
    unsafe { context.slices_drop_in_place(slices_to_drop) }
}

#[inline]
pub unsafe fn as_refs_unchecked<'a, 'data, T>(
    context: &'a T::Context,
    ptrs: Ptrs<'a, T>,
) -> Refs<'a, 'data, T>
where
    T: Soa<'data> + ?Sized,
{
    unsafe { refs::from_ptrs::<T>(context, ptrs) }
}

#[inline]
pub unsafe fn as_mut_refs_unchecked<'a, 'data, T>(
    context: &'a T::Context,
    ptrs: MutPtrs<'a, T>,
) -> RefsMut<'a, 'data, T>
where
    T: Soa<'data> + ?Sized,
{
    unsafe { refs::from_mut_ptrs::<T>(context, ptrs) }
}

#[inline]
pub unsafe fn as_slices_unchecked<'a, 'data, T>(
    context: &'a T::Context,
    slices: SlicePtrs<'a, T>,
) -> Slices<'a, 'data, T>
where
    T: Soa<'data> + ?Sized,
{
    unsafe { slices::from_slice_ptrs::<T>(context, slices) }
}

#[inline]
pub unsafe fn as_mut_slices_unchecked<'a, 'data, T>(
    context: &'a T::Context,
    slices: SliceMutPtrs<'a, T>,
) -> SlicesMut<'a, 'data, T>
where
    T: Soa<'data> + ?Sized,
{
    unsafe { slices::from_mut_slice_ptrs::<T>(context, slices) }
}

#[inline]
pub unsafe fn clone_to_uninit<T>(context: &T::Context, src: Ptrs<'_, T>, dst: MutPtrs<'_, T>)
where
    T: SoaCloneToUninit + ?Sized,
{
    unsafe { context.ptrs_clone_to_uninit(src, dst) }
}

#[inline]
pub unsafe fn read<'a, T, R>(context: &'a T::Context, src: Ptrs<'a, T>) -> R
where
    T: SoaRead<'a, R> + ?Sized,
{
    unsafe { context.ptrs_read(src) }
}

#[inline]
pub unsafe fn write<T, W>(context: &T::Context, dst: MutPtrs<'_, T>, value: W)
where
    T: SoaWrite<W> + ?Sized,
{
    unsafe { context.ptrs_write(dst, value) }
}

/// Version of [`core::ptr::replace()`] but for [SoA](RawSoa) types.
pub unsafe fn replace<'a, T, R, W>(context: &'a T::Context, dst: MutPtrs<'a, T>, src: W) -> R
where
    T: SoaRead<'a, R> + SoaWrite<W> + ?Sized,
{
    let result = unsafe {
        let src = context.ptrs_cast_const(dst.clone());
        context.ptrs_read(src)
    };
    let slot = ManuallyDrop::new(result);

    unsafe { context.ptrs_write(dst, src) }
    ManuallyDrop::into_inner(slot)
}

#[inline]
pub unsafe fn get_unchecked<'a, T, I>(
    context: &'a T::Context,
    slices: SlicePtrs<'a, T>,
    index: I,
) -> I::Ptrs<'a>
where
    T: RawSoa + ?Sized,
    I: SlicePtrsIndex<T>,
{
    unsafe { index.get_unchecked(context, slices) }
}

#[inline]
pub fn get<'a, T, I>(
    context: &'a T::Context,
    slices: SlicePtrs<'a, T>,
    index: I,
) -> Option<I::Ptrs<'a>>
where
    T: RawSoa + ?Sized,
    I: SlicePtrsIndex<T>,
{
    index.get_ptrs(context, slices)
}

#[inline]
pub fn index<'a, T, I>(context: &'a T::Context, slices: SlicePtrs<'a, T>, index: I) -> I::Ptrs<'a>
where
    T: RawSoa + ?Sized,
    I: SlicePtrsIndex<T>,
{
    index.index_ptrs(context, slices)
}

#[inline]
pub unsafe fn get_unchecked_mut<'a, T, I>(
    context: &'a T::Context,
    slices: SliceMutPtrs<'a, T>,
    index: I,
) -> I::MutPtrs<'a>
where
    T: RawSoa + ?Sized,
    I: SlicePtrsIndex<T>,
{
    unsafe { index.get_unchecked_mut(context, slices) }
}

#[inline]
pub fn get_mut<'a, T, I>(
    context: &'a T::Context,
    slices: SliceMutPtrs<'a, T>,
    index: I,
) -> Option<I::MutPtrs<'a>>
where
    T: RawSoa + ?Sized,
    I: SlicePtrsIndex<T>,
{
    index.get_mut_ptrs(context, slices)
}

#[inline]
pub fn index_mut<'a, T, I>(
    context: &'a T::Context,
    slices: SliceMutPtrs<'a, T>,
    index: I,
) -> I::MutPtrs<'a>
where
    T: RawSoa + ?Sized,
    I: SlicePtrsIndex<T>,
{
    index.index_mut_ptrs(context, slices)
}
