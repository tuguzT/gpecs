use core::{
    alloc::{Layout, LayoutError},
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    slice,
};

#[cfg(feature = "alloc")]
use core_alloc::vec::Vec;

#[cfg(feature = "alloc")]
use crate::traits::SoaVecs;
use crate::{
    desc::FieldDescriptor,
    traits::{
        impls::{collect_array, debug_assert_ptr_is_aligned},
        DefaultContext, Soa, SoaToOwned, SoaTrustedFields,
    },
};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
#[repr(transparent)]
pub struct Identity<T>(pub T)
where
    T: ?Sized;

impl<T> Identity<T>
where
    T: ?Sized,
{
    #[inline]
    pub fn as_inner(&self) -> &T {
        self
    }

    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut T {
        self
    }
}

impl<T> Identity<T> {
    #[inline]
    pub fn into_inner(self) -> T {
        let Self(inner) = self;
        inner
    }
}

pub trait IdentityPtr<T: ?Sized> {
    fn as_inner_ptr(self) -> *const T;
}

impl<T> IdentityPtr<T> for *const Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn as_inner_ptr(self) -> *const T {
        self as _
    }
}

pub trait IdentityMutPtr<T: ?Sized> {
    fn as_inner_mut_ptr(self) -> *mut T;
}

impl<T> IdentityMutPtr<T> for *mut Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn as_inner_mut_ptr(self) -> *mut T {
        self as _
    }
}

pub trait IdentitySlicePtr<T> {
    fn as_inner_ptr(self) -> *const [T];
}

impl<T> IdentitySlicePtr<T> for *const [Identity<T>] {
    #[inline]
    fn as_inner_ptr(self) -> *const [T] {
        self as _
    }
}

pub trait IdentitySliceMutPtr<T> {
    fn as_inner_mut_ptr(self) -> *mut [T];
}

impl<T> IdentitySliceMutPtr<T> for *mut [Identity<T>] {
    #[inline]
    fn as_inner_mut_ptr(self) -> *mut [T] {
        self as _
    }
}

pub trait IdentitySlice<T> {
    fn as_inner(&self) -> &[T];
    fn as_inner_mut(&mut self) -> &mut [T];
}

impl<T> IdentitySlice<T> for [Identity<T>] {
    #[inline]
    fn as_inner(&self) -> &[T] {
        let inner = ptr::from_ref(self).as_inner_ptr();
        unsafe { &*inner }
    }

    #[inline]
    fn as_inner_mut(&mut self) -> &mut [T] {
        let inner = ptr::from_mut(self).as_inner_mut_ptr();
        unsafe { &mut *inner }
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

impl<T> Clone for Identity<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        (**self).clone().into()
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        (**self).clone_from(source);
    }
}

unsafe impl<T> Soa for Identity<T> {
    type Context = DefaultContext;
    type Fields = Identity<T>;

    type FieldDescriptors<'context> = [FieldDescriptor; 1];

    #[inline]
    fn field_descriptors(_context: &Self::Context) -> Self::FieldDescriptors<'_> {
        [FieldDescriptor::of::<T>()]
    }

    type FieldOffsets<'context> = [usize; 1];

    #[inline]
    fn buffer_layout(
        _context: &Self::Context,
        capacity: usize,
    ) -> Result<(Layout, Self::FieldOffsets<'_>), LayoutError> {
        let offsets = [0];
        let layout = Layout::array::<T>(capacity)?;
        Ok((layout, offsets))
    }

    #[inline]
    fn capacity_from(_context: &Self::Context, buffer_layout: Layout) -> usize {
        buffer_layout
            .size()
            .checked_div(size_of::<T>())
            .unwrap_or(usize::MAX)
    }

    type Ptrs<'context> = *const Self;
    type MutPtrs<'context> = *mut Self;

    type ErasedPtrs<'context> = [*const u8; 1];
    type ErasedMutPtrs<'context> = [*mut u8; 1];

    #[inline]
    fn ptrs_erase<'context>(
        _context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::ErasedPtrs<'context> {
        [ptrs.cast()]
    }

    #[inline]
    fn ptrs_erase_mut<'context>(
        _context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::ErasedMutPtrs<'context> {
        [ptrs.cast()]
    }

    #[inline]
    #[track_caller]
    fn ptrs_restore(
        _context: &Self::Context,
        ptrs: impl IntoIterator<Item = *const u8>,
    ) -> Self::Ptrs<'_> {
        let ptrs: [_; 1] = collect_array(ptrs);
        let ptr = ptrs[0].cast();
        debug_assert_ptr_is_aligned(ptr);
        ptr
    }

    #[inline]
    #[track_caller]
    fn ptrs_restore_mut(
        _context: &Self::Context,
        ptrs: impl IntoIterator<Item = *mut u8>,
    ) -> Self::MutPtrs<'_> {
        let ptrs: [_; 1] = collect_array(ptrs);
        let ptr = ptrs[0].cast();
        debug_assert_ptr_is_aligned(ptr);
        ptr
    }

    #[inline]
    fn ptrs_dangling(_context: &Self::Context) -> Self::MutPtrs<'_> {
        ptr::dangling_mut()
    }

    #[inline]
    fn ptrs_cast_const<'context>(
        _context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::Ptrs<'context> {
        ptrs.cast_const()
    }

    #[inline]
    fn ptrs_cast_mut<'context>(
        _context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::MutPtrs<'context> {
        ptrs.cast_mut()
    }

    #[inline]
    unsafe fn ptrs_add<'context>(
        _context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
        offset: usize,
    ) -> Self::Ptrs<'context> {
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_add_mut<'context>(
        _context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
        offset: usize,
    ) -> Self::MutPtrs<'context> {
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(
        _context: &Self::Context,
        ptrs: Self::Ptrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
        _context: &Self::Context,
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline]
    unsafe fn ptrs_swap(_context: &Self::Context, a: Self::MutPtrs<'_>, b: Self::MutPtrs<'_>) {
        unsafe { ptr::swap(a, b) }
    }

    #[inline]
    unsafe fn ptrs_copy(
        _context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_rev(
        _context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        _context: &Self::Context,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { ptr::copy_nonoverlapping(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_read(_context: &Self::Context, src: Self::Ptrs<'_>) -> Self {
        unsafe { ptr::read(src) }
    }

    #[inline]
    unsafe fn ptrs_write(_context: &Self::Context, dst: Self::MutPtrs<'_>, value: Self) {
        unsafe { ptr::write(dst, value) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(_context: &Self::Context, ptrs: Self::MutPtrs<'_>) {
        unsafe { ptr::drop_in_place(ptrs) }
    }

    type NonNullPtrs<'context> = NonNull<Self>;

    #[inline]
    unsafe fn ptrs_to_nonnull<'context>(
        _context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::NonNullPtrs<'context> {
        unsafe { NonNull::new_unchecked(ptrs) }
    }

    #[inline]
    fn nonnull_to_ptrs<'context>(
        _context: &'context Self::Context,
        ptrs: Self::NonNullPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        ptrs.as_ptr()
    }

    type Refs<'context, 'a>
        = &'a Self
    where
        Self: 'a;

    type RefsMut<'context, 'a>
        = &'a mut Self
    where
        Self: 'a;

    #[inline]
    unsafe fn ptrs_to_refs<'context, 'a>(
        _context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
    ) -> Self::Refs<'context, 'a> {
        unsafe { &*ptrs }
    }

    #[inline]
    unsafe fn ptrs_to_refs_mut<'context, 'a>(
        _context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
    ) -> Self::RefsMut<'context, 'a> {
        unsafe { &mut *ptrs }
    }

    #[inline]
    fn refs_as_ptrs<'context>(
        _context: &'context Self::Context,
        refs: Self::Refs<'context, '_>,
    ) -> Self::Ptrs<'context> {
        ptr::from_ref(refs)
    }

    #[inline]
    fn mut_refs_as_ptrs<'context>(
        _context: &'context Self::Context,
        refs: Self::RefsMut<'context, '_>,
    ) -> Self::MutPtrs<'context> {
        ptr::from_mut(refs)
    }

    #[inline]
    fn mut_refs_as_refs<'context, 'a>(
        _context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        &*refs
    }

    type SlicePtrs<'context> = *const [Self];
    type SliceMutPtrs<'context> = *mut [Self];

    #[inline]
    fn slices_from_raw_parts<'context>(
        _context: &'context Self::Context,
        ptrs: Self::Ptrs<'context>,
        len: usize,
    ) -> Self::SlicePtrs<'context> {
        ptr::slice_from_raw_parts(ptrs, len)
    }

    #[inline]
    fn slices_from_raw_parts_mut<'context>(
        _context: &'context Self::Context,
        ptrs: Self::MutPtrs<'context>,
        len: usize,
    ) -> Self::SliceMutPtrs<'context> {
        ptr::slice_from_raw_parts_mut(ptrs, len)
    }

    #[inline]
    fn slice_ptrs_cast_const<'context>(
        _context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicePtrs<'context> {
        slices.cast_const()
    }

    #[inline]
    fn slice_ptrs_cast_mut<'context>(
        _context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::SliceMutPtrs<'context> {
        slices.cast_mut()
    }

    #[inline]
    fn slice_ptrs_len(_context: &Self::Context, slices: &Self::SlicePtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_ptrs_len_mut(_context: &Self::Context, slices: &Self::SliceMutPtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'context>(
        _context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::Ptrs<'context> {
        slices.cast()
    }

    #[inline]
    fn mut_slice_ptrs_as_ptrs<'context>(
        _context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        slices.cast()
    }

    type Slices<'context, 'a>
        = &'a [Self]
    where
        Self: 'a;

    type SlicesMut<'context, 'a>
        = &'a mut [Self]
    where
        Self: 'a;

    #[inline]
    unsafe fn slice_ptrs_to_slices<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::SlicePtrs<'context>,
    ) -> Self::Slices<'context, 'a> {
        let data = slices.cast();
        let len = slices.len();
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices_mut<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicesMut<'context, 'a> {
        let data = slices.cast();
        let len = slices.len();
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline]
    fn slices_len(_context: &Self::Context, slices: &Self::Slices<'_, '_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slices_len_mut(_context: &Self::Context, slices: &Self::SlicesMut<'_, '_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_refs_as_slice_ptrs<'context>(
        _context: &'context Self::Context,
        slices: Self::Slices<'context, '_>,
    ) -> Self::SlicePtrs<'context> {
        ptr::from_ref(slices)
    }

    #[inline]
    fn mut_slice_refs_as_slice_ptrs<'context>(
        _context: &'context Self::Context,
        slices: Self::SlicesMut<'context, '_>,
    ) -> Self::SliceMutPtrs<'context> {
        ptr::from_mut(slices)
    }

    #[inline]
    fn mut_slices_as_slices<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::Slices<'context, 'a> {
        &*slices
    }

    #[inline]
    fn slice_refs_as_ptrs<'context>(
        _context: &'context Self::Context,
        slices: Self::Slices<'context, '_>,
    ) -> Self::Ptrs<'context> {
        slices.as_ptr()
    }

    #[inline]
    fn mut_slice_refs_as_ptrs<'context>(
        _context: &'context Self::Context,
        slices: Self::SlicesMut<'context, '_>,
    ) -> Self::MutPtrs<'context> {
        slices.as_mut_ptr()
    }

    #[inline]
    unsafe fn slices_drop_in_place(_context: &Self::Context, slices: Self::SliceMutPtrs<'_>) {
        unsafe { ptr::drop_in_place(slices) }
    }
}

impl<'a, T> SoaToOwned<'_, 'a> for &'a Identity<T>
where
    T: Clone,
{
    type Owned = Identity<T>;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        (*self).clone()
    }

    #[inline]
    fn clone_into(&self, target: &mut Self::Owned) {
        target.clone_from(self);
    }

    #[inline]
    fn clone_into_refs<'context>(
        &self,
        _context: &'context <Self::Owned as Soa>::Context,
        target: <Self::Owned as Soa>::RefsMut<'context, '_>,
    ) {
        target.clone_from(self);
    }
}

#[cfg(feature = "alloc")]
unsafe impl<T> SoaVecs for Identity<T> {
    type Vecs = Vec<Self>;

    #[inline]
    fn vecs_with_capacity(_context: &Self::Context, capacity: usize) -> Self::Vecs {
        Vec::with_capacity(capacity)
    }

    #[inline]
    fn vecs_as_ptrs<'context>(
        _context: &'context Self::Context,
        vecs: &Self::Vecs,
    ) -> Self::Ptrs<'context> {
        vecs.as_ptr()
    }

    #[inline]
    fn mut_vecs_as_ptrs<'context>(
        _context: &'context Self::Context,
        vecs: &mut Self::Vecs,
    ) -> Self::MutPtrs<'context> {
        vecs.as_mut_ptr()
    }

    #[inline]
    fn vecs_len(_context: &Self::Context, vecs: &Self::Vecs) -> usize {
        vecs.len()
    }

    #[inline]
    unsafe fn vecs_set_len(_context: &Self::Context, vecs: &mut Self::Vecs, len: usize) {
        unsafe { vecs.set_len(len) }
    }
}

unsafe impl<T> SoaTrustedFields for Identity<T> {}
