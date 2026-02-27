use core::{
    alloc::{Layout, LayoutError},
    borrow::{Borrow, BorrowMut},
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    slice,
};

use crate::{
    field::{FieldDescriptor, FieldDescriptors},
    traits::{
        AllocSoaContext, AllocSoaTrusted, CloneToUninitSoaContext, RawSoa, RawSoaContext,
        ReadSoaContext, Refs, RefsMut, SoaAsMutRefs, SoaAsRefs, SoaContext, WriteSoaContext,
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

pub struct IdentityContext<T>(PhantomData<fn() -> T>);

impl<T> IdentityContext<T> {
    #[inline]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Debug for IdentityContext<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IdentityContext").finish_non_exhaustive()
    }
}

impl<T> Default for IdentityContext<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for IdentityContext<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for IdentityContext<T> {}

impl<T> PartialEq for IdentityContext<T> {
    fn eq(&self, other: &Self) -> bool {
        let Self(this) = self;
        let Self(other) = other;
        this == other
    }
}

impl<T> Eq for IdentityContext<T> {}

impl<T> PartialOrd for IdentityContext<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for IdentityContext<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self(this) = self;
        let Self(other) = other;
        this.cmp(other)
    }
}

impl<T> Hash for IdentityContext<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self(this) = self;
        this.hash(state);
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

impl<'a, T> FieldDescriptors<'a> for Identity<T>
where
    T: FieldDescriptors<'a> + ?Sized,
{
    type Output = T::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.as_inner().field_descriptors()
    }
}

unsafe impl<T> RawSoaContext for IdentityContext<T> {
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
    type Context = IdentityContext<T>;
    type Fields = Identity<T>;
}

unsafe impl<T> CloneToUninitSoaContext for IdentityContext<T>
where
    T: Clone,
{
    #[inline]
    unsafe fn clone_to_uninit(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>) {
        let src = unsafe { &*src }.clone();
        unsafe { ptr::write(dst, src) }
    }
}

unsafe impl<T> ReadSoaContext<Identity<T>> for IdentityContext<T> {
    #[inline]
    unsafe fn read(&self, src: Self::Ptrs<'_>) -> Identity<T> {
        unsafe { ptr::read(src) }
    }
}

unsafe impl<T> WriteSoaContext<Identity<T>> for IdentityContext<T> {
    #[inline]
    unsafe fn write(&self, dst: Self::MutPtrs<'_>, value: Identity<T>) {
        unsafe { ptr::write(dst, value) }
    }
}

impl<'a, T> FieldDescriptors<'a> for IdentityContext<T> {
    type Output = [FieldDescriptor; 1];

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        [FieldDescriptor::of::<T>()]
    }
}

unsafe impl<T> AllocSoaContext for IdentityContext<T> {
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

unsafe impl<'data, T> SoaContext<'data> for IdentityContext<T>
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
        unsafe { &*ptrs }
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
        unsafe { &mut *ptrs }
    }

    #[inline]
    fn mut_refs_as_mut_ptrs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::MutPtrs<'a> {
        ptr::from_mut(refs)
    }

    #[inline]
    fn mut_refs_as_refs<'a>(&'a self, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        &*refs
    }

    type Slices<'a> = &'data [Identity<T>];

    #[inline]
    fn upcast_slices<'short, 'long: 'short>(from: Self::Slices<'long>) -> Self::Slices<'short> {
        from
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'a>(&'a self, slices: Self::SlicePtrs<'a>) -> Self::Slices<'a> {
        unsafe { slice::from_raw_parts(slices.cast(), slices.len()) }
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
        unsafe { slice::from_raw_parts_mut(slices.cast(), slices.len()) }
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
        &*slices
    }
}

impl<'a, T> SoaAsRefs<'a> for Identity<T>
where
    T: 'a,
{
    #[inline]
    fn as_refs(&'a self, _context: &'a Self::Context) -> Refs<'a, 'a, Self> {
        self
    }
}

impl<'a, T> SoaAsMutRefs<'a> for Identity<T>
where
    T: 'a,
{
    #[inline]
    fn as_mut_refs(&'a mut self, _context: &'a Self::Context) -> RefsMut<'a, 'a, Self> {
        self
    }
}
