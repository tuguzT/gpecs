use core::{
    alloc::{Layout, LayoutError},
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    slice,
};

use crate::{
    field::FieldDescriptor,
    traits::{
        Soa, SoaRead, SoaToOwned, SoaTrustedFields, SoaWrite, impls::debug_assert_ptr_is_aligned,
    },
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

pub trait IdentityPtr<T: ?Sized>: private::Sealed {
    #[expect(clippy::wrong_self_convention, reason = "method of pointer type")]
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

pub trait IdentityMutPtr<T: ?Sized>: private::Sealed {
    #[expect(clippy::wrong_self_convention, reason = "method of pointer type")]
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

pub trait IdentitySlicePtr<T>: private::Sealed {
    #[expect(clippy::wrong_self_convention, reason = "method of pointer type")]
    fn as_inner_ptr(self) -> *const [T];
}

impl<T> IdentitySlicePtr<T> for *const [Identity<T>] {
    #[inline]
    fn as_inner_ptr(self) -> *const [T] {
        self as _
    }
}

pub trait IdentitySliceMutPtr<T>: private::Sealed {
    #[expect(clippy::wrong_self_convention, reason = "method of pointer type")]
    fn as_inner_mut_ptr(self) -> *mut [T];
}

impl<T> IdentitySliceMutPtr<T> for *mut [Identity<T>] {
    #[inline]
    fn as_inner_mut_ptr(self) -> *mut [T] {
        self as _
    }
}

pub trait IdentitySlice<T>: private::Sealed {
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

mod private {
    use super::Identity;

    pub trait Sealed {}

    impl<T> Sealed for *const Identity<T> where T: ?Sized {}

    impl<T> Sealed for *mut Identity<T> where T: ?Sized {}

    impl<T> Sealed for *const [Identity<T>] {}

    impl<T> Sealed for *mut [Identity<T>] {}

    impl<T> Sealed for [Identity<T>] {}
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
    type Fields = Self;

    type FieldDescriptors<'context> = [FieldDescriptor; 1];

    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: Self::FieldDescriptors<'long>,
    ) -> Self::FieldDescriptors<'short> {
        from
    }

    #[inline]
    fn field_descriptors(_context: &Self::Context) -> Self::FieldDescriptors<'_> {
        [FieldDescriptor::of::<T>()]
    }

    #[inline]
    fn buffer_layout(_context: &Self::Context, capacity: usize) -> Result<Layout, LayoutError> {
        Layout::array::<T>(capacity)
    }

    #[inline]
    fn capacity_from(_context: &Self::Context, buffer_layout: Layout) -> usize {
        buffer_layout
            .size()
            .checked_div(size_of::<T>())
            .unwrap_or(usize::MAX)
    }

    type Ptrs<'context> = *const Self;

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        from
    }

    type MutPtrs<'context> = *mut Self;

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(_context: &Self::Context) -> Self::MutPtrs<'_> {
        ptr::dangling_mut()
    }

    #[inline]
    unsafe fn ptrs_from_buffer(
        _context: &Self::Context,
        buffer: *mut u8,
        _capacity: usize,
    ) -> Self::MutPtrs<'_> {
        let ptrs = buffer.cast();
        debug_assert_ptr_is_aligned(ptrs);
        ptrs
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
    unsafe fn ptrs_drop_in_place(_context: &Self::Context, ptrs: Self::MutPtrs<'_>) {
        unsafe { ptr::drop_in_place(ptrs) }
    }

    type NonNullPtrs<'context> = NonNull<Self>;

    #[inline]
    fn upcast_nonnull_ptrs<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        from
    }

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

    #[inline]
    fn upcast_refs<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::Refs<'long, 'a_long>,
    ) -> Self::Refs<'short, 'a_short> {
        from
    }

    type RefsMut<'context, 'a>
        = &'a mut Self
    where
        Self: 'a;

    #[inline]
    fn upcast_refs_mut<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::RefsMut<'long, 'a_long>,
    ) -> Self::RefsMut<'short, 'a_short> {
        from
    }

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
    fn refs_as_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        refs: Self::Refs<'context, 'a>,
    ) -> Self::Ptrs<'context>
    where
        Self: 'a,
    {
        ptr::from_ref(refs)
    }

    #[inline]
    fn refs_mut_as_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> Self::MutPtrs<'context>
    where
        Self: 'a,
    {
        ptr::from_mut(refs)
    }

    #[inline]
    fn refs_mut_as_refs<'context, 'a>(
        _context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> Self::Refs<'context, 'a> {
        &*refs
    }

    #[inline]
    fn value_as_refs<'context, 'a>(
        _context: &'context Self::Context,
        value: &'a Self,
    ) -> Self::Refs<'context, 'a>
    where
        Self: 'a,
        'a: 'context,
    {
        value
    }

    #[inline]
    fn mut_value_as_refs<'context, 'a>(
        _context: &'context Self::Context,
        value: &'a mut Self,
    ) -> Self::RefsMut<'context, 'a>
    where
        Self: 'a,
        'a: 'context,
    {
        value
    }

    type SlicePtrs<'context> = *const [Self];

    #[inline]
    fn upcast_slice_ptrs<'short, 'long: 'short>(
        from: Self::SlicePtrs<'long>,
    ) -> Self::SlicePtrs<'short> {
        from
    }

    type SliceMutPtrs<'context> = *mut [Self];

    #[inline]
    fn upcast_slice_mut_ptrs<'short, 'long: 'short>(
        from: Self::SliceMutPtrs<'long>,
    ) -> Self::SliceMutPtrs<'short> {
        from
    }

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
    fn slice_mut_ptrs_len(_context: &Self::Context, slices: &Self::SliceMutPtrs<'_>) -> usize {
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
    fn slice_mut_ptrs_as_ptrs<'context>(
        _context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::MutPtrs<'context> {
        slices.cast()
    }

    type Slices<'context, 'a>
        = &'a [Self]
    where
        Self: 'a;

    #[inline]
    fn upcast_slices<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::Slices<'long, 'a_long>,
    ) -> Self::Slices<'short, 'a_short> {
        from
    }

    type SlicesMut<'context, 'a>
        = &'a mut [Self]
    where
        Self: 'a;

    #[inline]
    fn upcast_slices_mut<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::SlicesMut<'long, 'a_long>,
    ) -> Self::SlicesMut<'short, 'a_short> {
        from
    }

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
    unsafe fn slice_mut_ptrs_to_slices<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::SliceMutPtrs<'context>,
    ) -> Self::SlicesMut<'context, 'a> {
        let data = slices.cast();
        let len = slices.len();
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    #[inline]
    fn slices_len<'a>(_context: &Self::Context, slices: &Self::Slices<'_, 'a>) -> usize
    where
        Self: 'a,
    {
        slices.len()
    }

    #[inline]
    fn slices_mut_len<'a>(_context: &Self::Context, slices: &Self::SlicesMut<'_, 'a>) -> usize
    where
        Self: 'a,
    {
        slices.len()
    }

    #[inline]
    fn slices_as_slice_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::Slices<'context, 'a>,
    ) -> Self::SlicePtrs<'context>
    where
        Self: 'a,
    {
        ptr::from_ref(slices)
    }

    #[inline]
    fn slices_mut_as_slice_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::SliceMutPtrs<'context>
    where
        Self: 'a,
    {
        ptr::from_mut(slices)
    }

    #[inline]
    fn slices_mut_as_slices<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::Slices<'context, 'a> {
        &*slices
    }

    #[inline]
    fn slices_as_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::Slices<'context, 'a>,
    ) -> Self::Ptrs<'context>
    where
        Self: 'a,
    {
        slices.as_ptr()
    }

    #[inline]
    fn slices_mut_as_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> Self::MutPtrs<'context>
    where
        Self: 'a,
    {
        slices.as_mut_ptr()
    }

    #[inline]
    unsafe fn slices_drop_in_place(_context: &Self::Context, slices: Self::SliceMutPtrs<'_>) {
        unsafe { ptr::drop_in_place(slices) }
    }
}

unsafe impl<T> SoaRead for Identity<T> {
    #[inline]
    unsafe fn read(_context: &Self::Context, src: Self::Ptrs<'_>) -> Self {
        unsafe { ptr::read(src) }
    }
}

unsafe impl<T> SoaWrite for Identity<T> {
    #[inline]
    unsafe fn write(_context: &Self::Context, dst: Self::MutPtrs<'_>, value: Self) {
        unsafe { ptr::write(dst, value) }
    }
}

impl<'a, T> SoaToOwned<'_, 'a> for &'a Identity<T>
where
    T: Clone,
{
    type Owned = Identity<T>;

    #[inline]
    fn to_owned(&self, _context: &<Self::Owned as Soa>::Context) -> Self::Owned {
        (*self).clone()
    }

    #[inline]
    fn clone_into(&self, _context: &<Self::Owned as Soa>::Context, target: &mut Self::Owned) {
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

unsafe impl<T> SoaTrustedFields for Identity<T> {}
