use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::{Index, IndexMut},
    ptr,
};

use crate::{
    layout::BufferData,
    ptr::{
        SoaSlicePtr, ptrs_from_buffer, ptrs_from_buffer_mut, slice_from_raw_parts,
        slice_from_raw_parts_mut,
    },
    traits::{MutPtrs, Ptrs, SoaContext, SoaToOwned, SoaTrustedFields, SoaWrite},
};

use super::{IndexHelper, IndexHelperMut, Iter, IterMut, SoaSliceIndex, SoaSlices, SoaSlicesMut};

#[repr(transparent)]
pub struct SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
{
    buffer: [BufferData<T>],
}

impl<T> SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
{
    #[inline]
    pub fn context(&self) -> &T::Context {
        unsafe { ptr::from_ref(self).context() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { ptr::from_ref(self).len() }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        unsafe { ptr::from_ref(self).capacity() }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const BufferData<T> {
        let Self { buffer } = self;
        buffer.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut BufferData<T> {
        let Self { buffer } = self;
        buffer.as_mut_ptr()
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'_, T> {
        let ptr = self.as_ptr().cast_mut();
        let context = self.context();
        let capacity = self.capacity();
        unsafe { ptrs_from_buffer::<T>(context, ptr, capacity) }
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'_, T> {
        let ptr = self.as_mut_ptr();
        let context = self.context();
        let capacity = self.capacity();
        unsafe { ptrs_from_buffer_mut::<T>(context, ptr, capacity) }
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_, '_> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&T::Context, T::Slices<'_, '_>) {
        let len = self.len();
        let context = self.context();
        let ptrs = self.as_ptrs();

        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> T::SlicesMut<'_, '_> {
        let (_, slices) = self.as_mut_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_mut_slices_with_context(&mut self) -> (&T::Context, T::SlicesMut<'_, '_>) {
        let len = self.len();
        let context = unsafe { ptr::from_mut(self).context() };
        let ptrs = self.as_mut_ptrs();

        let slices = context.slice_mut_ptrs_from_raw_parts(ptrs, len);
        let slices = unsafe { T::slice_mut_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn slices(&self) -> SoaSlices<'_, '_, T> {
        let (context, slices) = self.as_slices_with_context();
        SoaSlices::new(context, slices)
    }

    #[inline]
    pub fn slices_mut(&mut self) -> SoaSlicesMut<'_, '_, T> {
        let (context, slices) = self.as_mut_slices_with_context();
        SoaSlicesMut::new(context, slices)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, '_, T> {
        self.slices().into_iter()
    }

    #[inline]
    pub fn iter_with_context(&self) -> (&T::Context, Iter<'_, '_, T>) {
        self.slices().into_iter_with_context()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, '_, T> {
        self.slices_mut().into_iter()
    }

    #[inline]
    pub fn iter_mut_with_context(&mut self) -> (&T::Context, IterMut<'_, '_, T>) {
        self.slices_mut().into_iter_with_context()
    }

    #[inline]
    pub fn contains<'me, V>(&'me self, value: V) -> bool
    where
        T::Refs<'me, 'me>: PartialEq<V>,
    {
        let mut iter = self.into_iter();
        iter.any(move |item| item.eq(&value))
    }

    #[inline]
    #[track_caller]
    pub fn copy_from_slice(&mut self, src: &Self)
    where
        T::Fields: Copy,
    {
        let src = src.slices();
        self.slices_mut().copy_from_slices(&src);
    }

    #[inline]
    pub fn get<I>(&self, index: I) -> Option<I::Refs<'_, '_>>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs) = self.get_with_context(index);
        refs
    }

    #[inline]
    pub fn get_with_context<I>(&self, index: I) -> (&T::Context, Option<I::Refs<'_, '_>>)
    where
        I: SoaSliceIndex<T>,
    {
        self.slices().into_get_with_context(index)
    }

    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<I::RefsMut<'_, '_>>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs) = self.get_mut_with_context(index);
        refs
    }

    #[inline]
    pub fn get_mut_with_context<I>(&mut self, index: I) -> (&T::Context, Option<I::RefsMut<'_, '_>>)
    where
        I: SoaSliceIndex<T>,
    {
        self.slices_mut().into_get_mut_with_context(index)
    }

    #[inline]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::Ptrs<'_>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, ptrs) = unsafe { self.get_unchecked_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_with_context<I>(&self, index: I) -> (&T::Context, I::Ptrs<'_>)
    where
        I: SoaSliceIndex<T>,
    {
        unsafe { self.slices().into_get_unchecked_with_context(index) }
    }

    #[inline]
    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> I::MutPtrs<'_>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, ptrs) = unsafe { self.get_unchecked_mut_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_mut_with_context<I>(
        &mut self,
        index: I,
    ) -> (&T::Context, I::MutPtrs<'_>)
    where
        I: SoaSliceIndex<T>,
    {
        unsafe { self.slices_mut().into_get_unchecked_mut_with_context(index) }
    }

    #[inline]
    #[track_caller]
    pub fn index<I>(&self, index: I) -> I::Refs<'_, '_>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs) = self.index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_with_context<I>(&self, index: I) -> (&T::Context, I::Refs<'_, '_>)
    where
        I: SoaSliceIndex<T>,
    {
        self.slices().into_index_with_context(index)
    }

    #[inline]
    #[track_caller]
    pub fn index_mut<I>(&mut self, index: I) -> I::RefsMut<'_, '_>
    where
        I: SoaSliceIndex<T>,
    {
        let (_, refs) = self.index_mut_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_mut_with_context<I>(&mut self, index: I) -> (&T::Context, I::RefsMut<'_, '_>)
    where
        I: SoaSliceIndex<T>,
    {
        self.slices_mut().into_index_mut_with_context(index)
    }

    #[inline]
    #[track_caller]
    pub fn swap(&mut self, a: usize, b: usize) {
        self.slices_mut().swap(a, b);
    }

    #[inline]
    pub fn sort_unstable_with_permutation<P>(&mut self, permutation: P)
    where
        P: AsMut<[usize]>,
        for<'c, 'any> T::Refs<'c, 'any>: Ord,
    {
        self.slices_mut()
            .sort_unstable_with_permutation(permutation);
    }

    #[inline]
    pub fn sort_unstable_with_permutation_by<P, F>(&mut self, permutation: P, compare: F)
    where
        P: AsMut<[usize]>,
        for<'c, 'any> F: FnMut(T::Refs<'c, 'any>, T::Refs<'c, 'any>) -> cmp::Ordering,
    {
        self.slices_mut()
            .sort_unstable_with_permutation_by(permutation, compare);
    }

    #[inline]
    pub fn sort_unstable_with_permutation_by_key<P, K, F>(&mut self, permutation: P, f: F)
    where
        P: AsMut<[usize]>,
        F: FnMut(T::Refs<'_, '_>) -> K,
        K: Ord,
    {
        self.slices_mut()
            .sort_unstable_with_permutation_by_key(permutation, f);
    }
}

impl<T> SoaSlice<T>
where
    T: SoaTrustedFields + SoaWrite,
{
    #[inline]
    #[track_caller]
    pub fn clone_from_slice(&mut self, src: &Self)
    where
        for<'c, 'any> T::Refs<'c, 'any>: SoaToOwned<'c, 'any, Owned = T>,
    {
        let src = src.slices();
        self.slices_mut().clone_from_slices(&src);
    }
}

impl<T> Debug for SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaSlice").field(&slices).finish()
    }
}

impl<T> AsRef<Self> for SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, U> AsRef<[U]> for SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
    for<'c, 'any> T: SoaTrustedFields<Slices<'c, 'any> = &'any [U]> + 'any,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices()
    }
}

impl<T> AsMut<Self> for SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, U> AsMut<[U]> for SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
    for<'c, 'any> T: SoaTrustedFields<SlicesMut<'c, 'any> = &'any mut [U]> + 'any,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [U] {
        self.as_mut_slices()
    }
}

impl<T> Eq for SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Eq,
{
}

impl<T> Ord for SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let this = self.as_slices();
        let other = other.as_slices();
        Ord::cmp(&this, &other)
    }
}

impl<T> Hash for SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let slices = self.as_slices();
        slices.hash(state);
    }
}

impl<T> Drop for SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
{
    #[inline]
    fn drop(&mut self) {
        if self.is_empty() {
            return;
        }

        let (context, slices) = self.slices_mut().into_slices_with_context();
        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        unsafe { context.slices_drop_in_place(slices) }
    }
}

impl<T, U, I> Index<I> for SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
    U: ?Sized,
    for<'c, 'any> I: IndexHelper<'c, 'any, T, Output = U>,
{
    type Output = U;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Self::index(self, index)
    }
}

impl<T, U, I> IndexMut<I> for SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
    U: ?Sized,
    for<'c, 'any> I: IndexHelperMut<'c, 'any, T, Output = U>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        Self::index_mut(self, index)
    }
}

impl<'r, T> IntoIterator for &'r SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
{
    type Item = T::Refs<'r, 'r>;
    type IntoIter = Iter<'r, 'r, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'r, T> IntoIterator for &'r mut SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
{
    type Item = T::RefsMut<'r, 'r>;
    type IntoIter = IterMut<'r, 'r, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

unsafe impl<T> Send for SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
    T::Context: Send,
    T::Fields: Send,
{
}

unsafe impl<T> Sync for SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
    T::Context: Sync,
    T::Fields: Sync,
{
}

#[inline]
pub unsafe fn from_raw_parts<'slice, T>(
    data: *const BufferData<T>,
    len: usize,
    capacity: usize,
) -> &'slice SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
{
    unsafe { &*slice_from_raw_parts(data, len, capacity) }
}

#[inline]
pub unsafe fn from_raw_parts_mut<'slice, T>(
    data: *mut BufferData<T>,
    len: usize,
    capacity: usize,
) -> &'slice mut SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
{
    unsafe { &mut *slice_from_raw_parts_mut(data, len, capacity) }
}
