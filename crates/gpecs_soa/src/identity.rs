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
    field::FieldDescriptor,
    ptr::assert_ptr_is_aligned,
    traits::{
        MutPtrs, Ptrs, RawSoa, RawSoaContext, SliceMutPtrs, SlicePtrs, Soa, SoaCloneToUninit,
        SoaRead, SoaTrustedFields, SoaWrite,
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

unsafe impl<T> RawSoaContext for IdentityContext<T> {
    type FieldDescriptors<'a> = [FieldDescriptor; 1];

    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: Self::FieldDescriptors<'long>,
    ) -> Self::FieldDescriptors<'short> {
        from
    }

    #[inline]
    fn field_descriptors(&self) -> Self::FieldDescriptors<'_> {
        [FieldDescriptor::of::<T>()]
    }

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
    unsafe fn ptrs_from_buffer(&self, buffer: *const u8, _capacity: usize) -> Self::Ptrs<'_> {
        let ptrs = buffer.cast();
        assert_ptr_is_aligned(ptrs);
        ptrs
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
    unsafe fn ptrs_from_buffer_mut(&self, buffer: *mut u8, _capacity: usize) -> Self::MutPtrs<'_> {
        let ptrs = buffer.cast();
        assert_ptr_is_aligned(ptrs);
        ptrs
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
    unsafe fn ptrs_copy(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize) {
        unsafe { ptr::copy(src, dst, len) }
    }

    #[inline]
    unsafe fn ptrs_copy_rev(&self, src: Self::Ptrs<'_>, dst: Self::MutPtrs<'_>, len: usize) {
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

unsafe impl<T> Soa for Identity<T> {
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
        ptrs: Ptrs<'context, Self>,
    ) -> Self::Refs<'context, 'a> {
        unsafe { &*ptrs }
    }

    #[inline]
    unsafe fn ptrs_to_refs_mut<'context, 'a>(
        _context: &'context Self::Context,
        ptrs: MutPtrs<'context, Self>,
    ) -> Self::RefsMut<'context, 'a> {
        unsafe { &mut *ptrs }
    }

    #[inline]
    fn refs_as_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        refs: Self::Refs<'context, 'a>,
    ) -> Ptrs<'context, Self>
    where
        Self: 'a,
    {
        ptr::from_ref(refs)
    }

    #[inline]
    fn refs_mut_as_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        refs: Self::RefsMut<'context, 'a>,
    ) -> MutPtrs<'context, Self>
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
    fn value_as_refs<'a>(_context: &'a Self::Context, value: &'a Self) -> Self::Refs<'a, 'a>
    where
        Self: 'a,
    {
        value
    }

    #[inline]
    fn mut_value_as_refs<'a>(
        _context: &'a Self::Context,
        value: &'a mut Self,
    ) -> Self::RefsMut<'a, 'a>
    where
        Self: 'a,
    {
        value
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
    fn upcast_mut_slices<'short, 'long: 'short, 'a_short, 'a_long: 'a_short>(
        from: Self::SlicesMut<'long, 'a_long>,
    ) -> Self::SlicesMut<'short, 'a_short> {
        from
    }

    #[inline]
    unsafe fn slice_ptrs_to_slices<'context, 'a>(
        _context: &'context Self::Context,
        slices: SlicePtrs<'context, Self>,
    ) -> Self::Slices<'context, 'a> {
        let data = slices.cast();
        let len = slices.len();
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    unsafe fn mut_slice_ptrs_to_mut_slices<'context, 'a>(
        _context: &'context Self::Context,
        slices: SliceMutPtrs<'context, Self>,
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
    fn mut_slices_len<'a>(_context: &Self::Context, slices: &Self::SlicesMut<'_, 'a>) -> usize
    where
        Self: 'a,
    {
        slices.len()
    }

    #[inline]
    fn slices_as_slice_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::Slices<'context, 'a>,
    ) -> SlicePtrs<'context, Self>
    where
        Self: 'a,
    {
        ptr::from_ref(slices)
    }

    #[inline]
    fn mut_slices_as_slice_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> SliceMutPtrs<'context, Self>
    where
        Self: 'a,
    {
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
    fn slices_as_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::Slices<'context, 'a>,
    ) -> Ptrs<'context, Self>
    where
        Self: 'a,
    {
        slices.as_ptr()
    }

    #[inline]
    fn mut_slices_as_ptrs<'context, 'a>(
        _context: &'context Self::Context,
        slices: Self::SlicesMut<'context, 'a>,
    ) -> MutPtrs<'context, Self>
    where
        Self: 'a,
    {
        slices.as_mut_ptr()
    }
}

unsafe impl<T> SoaRead for Identity<T> {
    #[inline]
    unsafe fn read(_context: &Self::Context, src: Ptrs<'_, Self>) -> Self {
        unsafe { ptr::read(src) }
    }
}

unsafe impl<T> SoaWrite for Identity<T> {
    #[inline]
    unsafe fn write(_context: &Self::Context, dst: MutPtrs<'_, Self>, value: Self) {
        unsafe { ptr::write(dst, value) }
    }
}

unsafe impl<T> SoaCloneToUninit for Identity<T>
where
    T: Clone,
{
    #[inline]
    unsafe fn clone_to_uninit(
        _context: &Self::Context,
        src: Ptrs<'_, Self>,
        dst: MutPtrs<'_, Self>,
    ) {
        let src = unsafe { &*src }.clone();
        unsafe { ptr::write(dst, src) }
    }
}

unsafe impl<T> SoaTrustedFields for Identity<T> {}
