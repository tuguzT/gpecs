use core::{
    alloc::{Layout, LayoutError},
    array::IntoIter,
    borrow::{Borrow, BorrowMut},
    cmp,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

use crate::{
    field::FieldLayouts,
    traits::{
        AllocSoaContext, AllocSoaTrusted, CloneToUninitSoaContext, RawSoa, RawSoaContext,
        ReadSoaContext, SoaContext, WriteSoaContext,
    },
};

#[derive(Debug, Default, Clone, Copy, Eq, Ord, Hash)]
#[repr(transparent)]
pub struct Identity<T>(pub T)
where
    T: ?Sized;

impl<T> Identity<T>
where
    T: ?Sized,
{
    #[inline]
    pub const fn from_inner_ref(inner: &T) -> &Self {
        // SAFETY: Self is `#[repr(transparent)]` over `T`.
        unsafe { (ptr::from_ref(inner) as *const Self).as_ref_unchecked() }
    }

    #[inline]
    pub const fn from_inner_mut(inner: &mut T) -> &mut Self {
        // SAFETY: Self is `#[repr(transparent)]` over `T`.
        unsafe { (ptr::from_mut(inner) as *mut Self).as_mut_unchecked() }
    }

    #[inline]
    pub const fn as_inner(&self) -> &T {
        let Self(inner) = self;
        inner
    }

    #[inline]
    pub const fn as_inner_mut(&mut self) -> &mut T {
        let Self(inner) = self;
        inner
    }
}

impl<T> Identity<T> {
    #[inline]
    pub const fn from_inner(inner: T) -> Self {
        Self(inner)
    }

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
        unsafe { inner.as_ref_unchecked() }
    }

    #[inline]
    fn as_inner_mut(&mut self) -> &mut [T] {
        let inner = ptr::from_mut(self).as_inner_mut_ptr();
        unsafe { inner.as_mut_unchecked() }
    }
}

pub trait AsIdentitySlice<T>: private::Sealed {
    fn as_identity_slice(&self) -> &[Identity<T>];
    fn as_identity_slice_mut(&mut self) -> &mut [Identity<T>];
}

impl<T> AsIdentitySlice<T> for [T] {
    #[inline]
    fn as_identity_slice(&self) -> &[Identity<T>] {
        let inner = ptr::from_ref(self) as *const [_];
        unsafe { inner.as_ref_unchecked() }
    }

    #[inline]
    fn as_identity_slice_mut(&mut self) -> &mut [Identity<T>] {
        let inner = ptr::from_mut(self) as *mut [_];
        unsafe { inner.as_mut_unchecked() }
    }
}

mod private {
    use super::Identity;

    pub trait Sealed {}

    impl<T> Sealed for *const Identity<T> where T: ?Sized {}

    impl<T> Sealed for *mut Identity<T> where T: ?Sized {}

    impl<T> Sealed for *const [Identity<T>] {}

    impl<T> Sealed for *mut [Identity<T>] {}

    impl<T> Sealed for [T] {}
}

impl<T> From<T> for Identity<T> {
    #[inline]
    fn from(inner: T) -> Self {
        Self::from_inner(inner)
    }
}

impl<T, U> PartialEq<Identity<U>> for Identity<T>
where
    T: PartialEq<U> + ?Sized,
    U: ?Sized,
{
    #[inline]
    fn eq(&self, other: &Identity<U>) -> bool {
        self.as_inner() == other.as_inner()
    }
}

impl<T, U> PartialOrd<Identity<U>> for Identity<T>
where
    T: PartialOrd<U> + ?Sized,
    U: ?Sized,
{
    #[inline]
    fn partial_cmp(&self, other: &Identity<U>) -> Option<cmp::Ordering> {
        let this = self.as_inner();
        let other = other.as_inner();
        this.partial_cmp(other)
    }
}

impl<T> Deref for Identity<T>
where
    T: ?Sized,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.as_inner()
    }
}

impl<T> DerefMut for Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.as_inner_mut()
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

impl<T, A> FromIterator<A> for Identity<T>
where
    T: FromIterator<A>,
{
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        Identity(T::from_iter(iter))
    }
}

impl<T> IntoIterator for Identity<T>
where
    T: IntoIterator,
{
    type Item = T::Item;
    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

impl<T, A> Extend<A> for Identity<T>
where
    T: Extend<A>,
{
    fn extend<I: IntoIterator<Item = A>>(&mut self, iter: I) {
        self.as_inner_mut().extend(iter);
    }
}

impl<'a, T, U> FieldLayouts<'a, Identity<U>> for Identity<T>
where
    T: FieldLayouts<'a, U> + ?Sized,
    U: ?Sized,
{
    type Output = T::Output;
    type OutputIter = T::OutputIter;
    type OutputItem = T::OutputItem;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        self.as_inner().field_layouts()
    }
}

unsafe impl<T> RawSoaContext<Identity<T>> for () {
    type Ptrs<'a> = *const Identity<T>;

    #[inline]
    fn upcast_ptrs<'short, 'long: 'short>(from: Self::Ptrs<'long>) -> Self::Ptrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling(&self) -> Self::Ptrs<'_> {
        ptr::dangling()
    }

    #[inline]
    unsafe fn ptrs_add<'a>(&'a self, ptrs: Self::Ptrs<'a>, offset: usize) -> Self::Ptrs<'a> {
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from(&self, ptrs: Self::Ptrs<'_>, origin: Self::Ptrs<'_>) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    type MutPtrs<'a> = *mut Identity<T>;

    #[inline]
    fn upcast_mut_ptrs<'short, 'long: 'short>(from: Self::MutPtrs<'long>) -> Self::MutPtrs<'short> {
        from
    }

    #[inline]
    fn ptrs_dangling_mut(&self) -> Self::MutPtrs<'_> {
        ptr::dangling_mut()
    }

    #[inline]
    unsafe fn ptrs_add_mut<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        offset: usize,
    ) -> Self::MutPtrs<'a> {
        unsafe { ptrs.add(offset) }
    }

    #[inline]
    unsafe fn ptrs_offset_from_mut(
        &self,
        ptrs: Self::MutPtrs<'_>,
        origin: Self::Ptrs<'_>,
    ) -> isize {
        unsafe { ptrs.offset_from(origin) }
    }

    #[inline]
    fn ptrs_cast_const<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::Ptrs<'a> {
        ptrs.cast_const()
    }

    #[inline]
    fn ptrs_cast_mut<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::MutPtrs<'a> {
        ptrs.cast_mut()
    }

    #[inline]
    unsafe fn ptrs_swap(&self, a: Self::MutPtrs<'_>, b: Self::MutPtrs<'_>) {
        unsafe { ptr::swap(a, b) }
    }

    #[inline]
    unsafe fn ptrs_copy_forward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_backward(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_nonoverlapping(
        &self,
        src: Self::Ptrs<'_>,
        dst: Self::MutPtrs<'_>,
        len: usize,
    ) {
        unsafe { ptr::copy_nonoverlapping(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_drop_in_place(&self, ptrs: Self::MutPtrs<'_>) {
        unsafe { ptr::drop_in_place(ptrs) }
    }

    type NonNullPtrs<'a> = NonNull<Identity<T>>;

    #[inline]
    fn upcast_nonnull_ptrs<'short, 'long: 'short>(
        from: Self::NonNullPtrs<'long>,
    ) -> Self::NonNullPtrs<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_nonnull<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::NonNullPtrs<'a> {
        unsafe { NonNull::new_unchecked(ptrs) }
    }

    #[inline]
    fn nonnull_to_ptrs<'a>(&'a self, ptrs: Self::NonNullPtrs<'a>) -> Self::MutPtrs<'a> {
        ptrs.as_ptr()
    }

    type SlicePtrs<'a> = *const [Identity<T>];

    #[inline]
    fn upcast_slice_ptrs<'short, 'long: 'short>(
        from: Self::SlicePtrs<'long>,
    ) -> Self::SlicePtrs<'short> {
        from
    }

    #[inline]
    fn slice_ptrs_from_raw_parts<'a>(
        &'a self,
        ptrs: Self::Ptrs<'a>,
        len: usize,
    ) -> Self::SlicePtrs<'a> {
        ptr::slice_from_raw_parts(ptrs, len)
    }

    #[inline]
    fn slice_ptrs_len(&self, slices: &Self::SlicePtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Ptrs<'a> {
        slices.cast()
    }

    type SliceMutPtrs<'a> = *mut [Identity<T>];

    #[inline]
    fn upcast_mut_slice_ptrs<'short, 'long: 'short>(
        from: Self::SliceMutPtrs<'long>,
    ) -> Self::SliceMutPtrs<'short> {
        from
    }

    #[inline]
    fn mut_slice_ptrs_from_raw_parts<'a>(
        &'a self,
        ptrs: Self::MutPtrs<'a>,
        len: usize,
    ) -> Self::SliceMutPtrs<'a> {
        ptr::slice_from_raw_parts_mut(ptrs, len)
    }

    #[inline]
    fn mut_slice_ptrs_len(&self, slices: &Self::SliceMutPtrs<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn mut_slice_ptrs_as_ptrs<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::MutPtrs<'a> {
        slices.cast()
    }

    #[inline]
    fn slice_ptrs_cast_const<'a>(&'a self, slices: Self::SliceMutPtrs<'a>) -> Self::SlicePtrs<'a> {
        slices.cast_const()
    }

    #[inline]
    fn slice_ptrs_cast_mut<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::SliceMutPtrs<'a> {
        slices.cast_mut()
    }

    #[inline]
    unsafe fn slices_drop_in_place(&self, slices: Self::SliceMutPtrs<'_>) {
        unsafe { ptr::drop_in_place(slices) }
    }
}

unsafe impl<T> RawSoa for Identity<T> {
    type Context = ();
    type Fields = Identity<T>;
}

unsafe impl<T> CloneToUninitSoaContext<Identity<T>> for ()
where
    T: Clone,
{
    #[inline]
    unsafe fn clone_to_uninit(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>) {
        let src = unsafe { src.as_ref_unchecked() }.clone();
        unsafe { ptr::write(dst, src) }
    }
}

unsafe impl<'a, T> ReadSoaContext<'a, Identity<T>, Identity<T>> for () {
    #[inline]
    unsafe fn read(&'a self, src: Self::Ptrs<'a>) -> Identity<T> {
        unsafe { ptr::read(src) }
    }
}

unsafe impl<T> WriteSoaContext<Identity<T>, Identity<T>> for () {
    #[inline]
    unsafe fn write(&self, dst: Self::MutPtrs<'_>, value: Identity<T>) {
        unsafe { ptr::write(dst, value) }
    }
}

impl<'a, T> FieldLayouts<'a, Identity<T>> for () {
    type Output = [Layout; 1];
    type OutputIter = IntoIter<Layout, 1>;
    type OutputItem = Layout;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        [Layout::new::<T>()]
    }
}

unsafe impl<T> AllocSoaContext<Identity<T>> for () {
    #[inline]
    fn buffer_layout(&self, capacity: usize) -> Result<Layout, LayoutError> {
        Layout::array::<T>(capacity)
    }

    #[inline]
    fn capacity_from(&self, buffer_layout: Layout) -> usize {
        buffer_layout
            .size()
            .checked_div(size_of::<T>())
            .unwrap_or(usize::MAX)
    }

    #[inline]
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, _capacity: usize) -> Self::Ptrs<'_> {
        buffer.cast()
    }

    #[inline]
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, _capacity: usize) -> Self::MutPtrs<'_> {
        buffer.cast()
    }
}

unsafe impl<T> AllocSoaTrusted for Identity<T> {}

unsafe impl<'data, T> SoaContext<'data, Identity<T>> for ()
where
    T: 'data,
{
    type Refs<'a> = &'data Identity<T>;

    #[inline]
    fn upcast_refs<'short, 'long: 'short>(from: Self::Refs<'long>) -> Self::Refs<'short> {
        from
    }

    #[inline]
    unsafe fn ptrs_to_refs<'a>(&'a self, ptrs: Self::Ptrs<'a>) -> Self::Refs<'a> {
        unsafe { ptrs.as_ref_unchecked() }
    }

    #[inline]
    fn refs_as_ptrs<'a>(&'a self, refs: Self::Refs<'a>) -> Self::Ptrs<'a> {
        ptr::from_ref(refs)
    }

    type RefsMut<'a> = &'data mut Identity<T>;

    #[inline]
    fn upcast_mut_refs<'short, 'long: 'short>(from: Self::RefsMut<'long>) -> Self::RefsMut<'short> {
        from
    }

    #[inline]
    unsafe fn mut_ptrs_to_mut_refs<'a>(&'a self, ptrs: Self::MutPtrs<'a>) -> Self::RefsMut<'a> {
        unsafe { ptrs.as_mut_unchecked() }
    }

    #[inline]
    fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a> {
        ptr::from_mut(refs)
    }

    #[inline]
    fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        refs
    }

    type Slices<'a> = &'data [Identity<T>];

    #[inline]
    fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        from
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Slices<'a> {
        unsafe { slices.as_ref_unchecked() }
    }

    #[inline]
    fn slices_as_slice_ptrs<'a>(&'a self, slices: Self::Slices<'a>) -> Self::SlicePtrs<'a> {
        ptr::from_ref(slices)
    }

    #[inline]
    fn slices_len(&self, slices: &Self::Slices<'_>) -> usize {
        slices.len()
    }

    type SlicesMut<'a> = &'data mut [Identity<T>];

    #[inline]
    fn upcast_mut_slices<'short, 'long: 'short>(
        from: Self::SlicesMut<'long>,
    ) -> Self::SlicesMut<'short> {
        from
    }

    #[inline]
    unsafe fn mut_slice_ptrs_to_mut_slices<'a>(
        &'a self,
        slices: Self::SliceMutPtrs<'a>,
    ) -> Self::SlicesMut<'a> {
        unsafe { slices.as_mut_unchecked() }
    }

    #[inline]
    fn mut_slices_as_mut_slice_ptrs<'a>(
        &'a self,
        slices: Self::SlicesMut<'a>,
    ) -> Self::SliceMutPtrs<'a> {
        ptr::from_mut(slices)
    }

    #[inline]
    fn mut_slices_len(&self, slices: &Self::SlicesMut<'_>) -> usize {
        slices.len()
    }

    #[inline]
    fn mut_slices_as_slices<'a>(&'a self, slices: Self::SlicesMut<'a>) -> Self::Slices<'a> {
        slices
    }
}
