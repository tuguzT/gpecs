use alloc::vec::Vec;
use core::{
    alloc::{Layout, LayoutError},
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    slice,
};

use crate::traits::{debug_assert_ptr_is_aligned, impls::collect_array, FieldDescriptor, Soa};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct Identity<T>(pub T)
where
    T: ?Sized;

impl<T> Identity<T> {
    #[inline]
    pub fn into_inner(self) -> T {
        let Self(inner) = self;
        inner
    }
}

impl<T> From<T> for Identity<T> {
    #[inline]
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> Deref for Identity<T>
where
    T: ?Sized,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        let Self(inner) = self;
        inner
    }
}

impl<T> DerefMut for Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        let Self(inner) = self;
        inner
    }
}

impl<T> AsRef<T> for Identity<T>
where
    T: ?Sized,
    <Self as Deref>::Target: AsRef<T>,
{
    #[inline]
    fn as_ref(&self) -> &T {
        (**self).as_ref()
    }
}

impl<T> AsRef<Self> for Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T> AsMut<T> for Identity<T>
where
    T: ?Sized,
    <Self as Deref>::Target: AsMut<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        (**self).as_mut()
    }
}

impl<T> AsMut<Self> for Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T> Borrow<T> for Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn borrow(&self) -> &T {
        self
    }
}

impl<T> BorrowMut<T> for Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

unsafe impl<T> Soa for Identity<T> {
    type Context = ();
    type Fields = T;

    type FieldDescriptors<'a> = [FieldDescriptor; 1];

    #[inline]
    fn field_descriptors(_: &Self::Context) -> Self::FieldDescriptors<'_> {
        [FieldDescriptor::of::<T>()]
    }

    type FieldOffsets<'a> = [usize; 1];

    #[inline]
    fn buffer_layout(
        _: &Self::Context,
        capacity: usize,
    ) -> Result<(Layout, Self::FieldOffsets<'_>), LayoutError> {
        let offsets = [0];
        let layout = Layout::array::<T>(capacity)?;
        Ok((layout, offsets))
    }

    #[inline]
    fn capacity_from(_: &Self::Context, buffer_layout: Layout) -> usize {
        buffer_layout
            .size()
            .checked_div(size_of::<T>())
            .unwrap_or(usize::MAX)
    }

    type Ptrs = *const T;
    type MutPtrs = *mut T;

    type ErasedPtrs = [*const u8; 1];
    type ErasedMutPtrs = [*mut u8; 1];

    #[inline]
    fn ptrs_erase(_: &Self::Context, ptrs: Self::Ptrs) -> Self::ErasedPtrs {
        [ptrs.cast()]
    }

    #[inline]
    fn ptrs_erase_mut(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::ErasedMutPtrs {
        [ptrs.cast()]
    }

    #[inline]
    #[track_caller]
    fn ptrs_restore(_: &Self::Context, ptrs: impl IntoIterator<Item = *const u8>) -> Self::Ptrs {
        let ptrs: [_; 1] = collect_array(ptrs);
        let ptr = ptrs[0].cast();
        debug_assert_ptr_is_aligned(ptr);
        ptr
    }

    #[inline]
    #[track_caller]
    fn ptrs_restore_mut(
        _: &Self::Context,
        ptrs: impl IntoIterator<Item = *mut u8>,
    ) -> Self::MutPtrs {
        let ptrs: [_; 1] = collect_array(ptrs);
        let ptr = ptrs[0].cast();
        debug_assert_ptr_is_aligned(ptr);
        ptr
    }

    #[inline]
    fn ptrs_dangling(_: &Self::Context) -> Self::MutPtrs {
        ptr::dangling_mut()
    }

    #[inline]
    fn ptrs_cast_const(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs {
        ptrs.cast_const()
    }

    #[inline]
    fn ptrs_cast_mut(_: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs {
        ptrs.cast_mut()
    }

    #[inline]
    unsafe fn ptrs_add(_: &Self::Context, ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_add_mut(_: &Self::Context, ptrs: Self::MutPtrs, offset: usize) -> Self::MutPtrs {
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(_: &Self::Context, ptrs: Self::Ptrs, origin: Self::Ptrs) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
        _: &Self::Context,
        ptrs: Self::MutPtrs,
        origin: Self::Ptrs,
    ) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline]
    unsafe fn ptrs_swap(_: &Self::Context, a: Self::MutPtrs, b: Self::MutPtrs) {
        unsafe { ptr::swap(a, b) }
    }

    #[inline]
    unsafe fn ptrs_copy(_: &Self::Context, src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_rev(_: &Self::Context, src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        _: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    ) {
        unsafe { ptr::copy_nonoverlapping(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_read(_: &Self::Context, src: Self::Ptrs) -> Self {
        let value = unsafe { ptr::read(src) };
        value.into()
    }

    #[inline]
    unsafe fn ptrs_write(_: &Self::Context, dst: Self::MutPtrs, value: Self) {
        let src = value.into_inner();
        unsafe { ptr::write(dst, src) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(_: &Self::Context, ptrs: Self::MutPtrs) {
        unsafe { ptr::drop_in_place(ptrs) }
    }

    type NonNullPtrs = NonNull<T>;

    #[inline]
    unsafe fn ptrs_to_nonnull(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
        unsafe { NonNull::new_unchecked(ptrs) }
    }

    #[inline]
    fn nonnull_to_ptrs(_: &Self::Context, ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
        ptrs.as_ptr()
    }

    type Vecs = Vec<T>;

    #[inline]
    fn vecs_with_capacity(_: &Self::Context, capacity: usize) -> Self::Vecs {
        Vec::with_capacity(capacity)
    }

    #[inline]
    fn vecs_as_ptrs(_: &Self::Context, vecs: &Self::Vecs) -> Self::Ptrs {
        vecs.as_ptr()
    }

    #[inline]
    fn mut_vecs_as_ptrs(_: &Self::Context, vecs: &mut Self::Vecs) -> Self::MutPtrs {
        vecs.as_mut_ptr()
    }

    #[inline]
    fn vecs_len(_: &Self::Context, vecs: &Self::Vecs) -> usize {
        vecs.len()
    }

    #[inline]
    unsafe fn vecs_set_len(_: &Self::Context, vecs: &mut Self::Vecs, len: usize) {
        unsafe { vecs.set_len(len) }
    }

    type Refs<'a>
        = &'a T
    where
        Self: 'a;

    type RefsMut<'a>
        = &'a mut T
    where
        Self: 'a;

    #[inline]
    unsafe fn ptrs_to_refs<'a>(_: &Self::Context, ptrs: Self::Ptrs) -> Self::Refs<'a> {
        unsafe { &*ptrs }
    }

    #[inline]
    unsafe fn ptrs_to_refs_mut<'a>(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::RefsMut<'a> {
        unsafe { &mut *ptrs }
    }

    #[inline]
    fn refs_as_ptrs(_: &Self::Context, refs: Self::Refs<'_>) -> Self::Ptrs {
        ptr::from_ref(refs)
    }

    #[inline]
    fn mut_refs_as_ptrs(_: &Self::Context, refs: Self::RefsMut<'_>) -> Self::MutPtrs {
        ptr::from_mut(refs)
    }

    #[inline]
    fn mut_refs_as_refs<'a>(_: &Self::Context, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        &*refs
    }

    type SlicePtrs = *const [T];
    type SliceMutPtrs = *mut [T];

    #[inline]
    fn slices_from_raw_parts(_: &Self::Context, ptrs: Self::Ptrs, len: usize) -> Self::SlicePtrs {
        ptr::slice_from_raw_parts(ptrs, len)
    }

    #[inline]
    fn slices_from_raw_parts_mut(
        _: &Self::Context,
        ptrs: Self::MutPtrs,
        len: usize,
    ) -> Self::SliceMutPtrs {
        ptr::slice_from_raw_parts_mut(ptrs, len)
    }

    #[inline]
    fn slice_ptrs_cast_const(_: &Self::Context, slices: Self::SliceMutPtrs) -> Self::SlicePtrs {
        slices.cast_const()
    }

    #[inline]
    fn slice_ptrs_cast_mut(_: &Self::Context, slices: Self::SlicePtrs) -> Self::SliceMutPtrs {
        slices.cast_mut()
    }

    #[inline]
    fn slice_ptrs_len(_: &Self::Context, slices: Self::SlicePtrs) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_ptrs_len_mut(_: &Self::Context, slices: Self::SliceMutPtrs) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_ptrs_as_ptrs(_: &Self::Context, slices: Self::SlicePtrs) -> Self::Ptrs {
        slices.cast()
    }

    #[inline]
    fn mut_slice_ptrs_as_ptrs(_: &Self::Context, slices: Self::SliceMutPtrs) -> Self::MutPtrs {
        slices.cast()
    }

    type Slices<'a>
        = &'a [T]
    where
        Self: 'a;

    type SlicesMut<'a>
        = &'a mut [T]
    where
        Self: 'a;

    #[inline]
    unsafe fn slice_ptrs_to_slices<'a>(
        _: &Self::Context,
        slices: Self::SlicePtrs,
    ) -> Self::Slices<'a> {
        let data = slices.cast();
        let len = slices.len();
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices_mut<'a>(
        _: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicesMut<'a> {
        let data = slices.cast();
        let len = slices.len();
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline]
    fn slices_len(_: &Self::Context, slices: &Self::Slices<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slices_len_mut(_: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_refs_as_slice_ptrs(_: &Self::Context, slices: Self::Slices<'_>) -> Self::SlicePtrs {
        ptr::from_ref(slices)
    }

    #[inline]
    fn mut_slice_refs_as_slice_ptrs(
        _: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::SliceMutPtrs {
        ptr::from_mut(slices)
    }

    #[inline]
    fn mut_slices_as_slices<'a>(
        _: &Self::Context,
        slices: Self::SlicesMut<'a>,
    ) -> Self::Slices<'a> {
        &*slices
    }

    #[inline]
    fn slice_refs_as_ptrs(_: &Self::Context, slices: Self::Slices<'_>) -> Self::Ptrs {
        slices.as_ptr()
    }

    #[inline]
    fn mut_slice_refs_as_ptrs(_: &Self::Context, slices: Self::SlicesMut<'_>) -> Self::MutPtrs {
        slices.as_mut_ptr()
    }

    #[inline]
    unsafe fn slices_drop_in_place(_: &Self::Context, slices: Self::SliceMutPtrs) {
        unsafe { ptr::drop_in_place(slices) }
    }
}
