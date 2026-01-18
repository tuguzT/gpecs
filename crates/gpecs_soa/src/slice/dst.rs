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
    traits::{
        AllocSoaTrusted, MutPtrs, Ptrs, RawSoaContext, Refs, RefsMut, SliceMutPtrs, SlicePtrs,
        Slices, SlicesMut, Soa, SoaCloneToUninit, SoaContext,
    },
};

use super::{
    IndexHelper, IndexHelperMut, Iter, IterMut, RawIter, RawIterMut, SoaSliceMutPtrs, SoaSlicePtrs,
    SoaSlicePtrsIndex, SoaSlices, SoaSlicesIndex, SoaSlicesMut,
};

#[repr(transparent)]
pub struct SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    buffer: [BufferData<T>],
}

impl<T> SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
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
        let (_, ptrs) = self.as_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&T::Context, Ptrs<'_, T>) {
        let ptr = self.as_ptr();
        let context = self.context();
        let capacity = self.capacity();

        let ptrs = unsafe { ptrs_from_buffer::<T>(context, ptr, capacity) };
        (context, ptrs)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'_, T> {
        let (_, ptrs) = self.as_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&T::Context, MutPtrs<'_, T>) {
        let ptr = self.as_mut_ptr();
        let context = self.context();
        let capacity = self.capacity();

        let ptrs = unsafe { ptrs_from_buffer_mut::<T>(context, ptr, capacity) };
        (context, ptrs)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'_, T> {
        let (_, slices) = self.as_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&T::Context, SlicePtrs<'_, T>) {
        let len = self.len();
        let (context, ptrs) = self.as_ptrs_with_context();

        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn as_mut_slice_ptrs(&mut self) -> SliceMutPtrs<'_, T> {
        let (_, slices) = self.as_mut_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_mut_slice_ptrs_with_context(&mut self) -> (&T::Context, SliceMutPtrs<'_, T>) {
        let len = self.len();
        let (context, ptrs) = self.as_mut_ptrs_with_context();

        let slices = context.mut_slice_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn slice_ptrs(&self) -> SoaSlicePtrs<'_, T> {
        let (context, slices) = self.as_slice_ptrs_with_context();
        SoaSlicePtrs::new(context, slices)
    }

    #[inline]
    pub fn mut_slice_ptrs(&mut self) -> SoaSliceMutPtrs<'_, T> {
        let (context, slices) = self.as_mut_slice_ptrs_with_context();
        SoaSliceMutPtrs::new(context, slices)
    }

    #[inline]
    pub fn slices(&self) -> SoaSlices<'_, '_, T> {
        unsafe { self.slice_ptrs().deref() }
    }

    #[inline]
    pub fn mut_slices(&mut self) -> SoaSlicesMut<'_, '_, T> {
        unsafe { self.mut_slice_ptrs().deref_mut() }
    }

    #[inline]
    #[track_caller]
    pub fn copy_from_slice(&mut self, src: &Self)
    where
        T::Fields: Copy,
    {
        let src = src.slices();
        self.mut_slices().copy_from_slices(&src);
    }

    #[inline]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::Ptrs<'_>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.get_unchecked_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_with_context<I>(&self, index: I) -> (&T::Context, I::Ptrs<'_>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        unsafe { self.slice_ptrs().into_get_unchecked_with_context(index) }
    }

    #[inline]
    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> I::MutPtrs<'_>
    where
        I: SoaSlicePtrsIndex<T>,
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
        I: SoaSlicePtrsIndex<T>,
    {
        let ptrs = self.mut_slice_ptrs();
        unsafe { ptrs.into_get_unchecked_mut_with_context(index) }
    }

    #[inline]
    pub fn raw_iter(&self) -> RawIter<'_, T> {
        let (_, iter) = self.raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn raw_iter_with_context(&self) -> (&T::Context, RawIter<'_, T>) {
        self.slices().into_raw_iter_with_context()
    }

    #[inline]
    pub fn raw_iter_mut(&mut self) -> RawIterMut<'_, T> {
        let (_, iter) = self.raw_iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn raw_iter_mut_with_context(&mut self) -> (&T::Context, RawIterMut<'_, T>) {
        self.mut_slices().into_raw_iter_mut_with_context()
    }

    #[inline]
    #[track_caller]
    pub fn swap(&mut self, a: usize, b: usize) {
        self.mut_slices().swap(a, b);
    }
}

impl<'a, T> SoaSlice<T>
where
    T: Soa<'a> + AllocSoaTrusted + ?Sized,
{
    #[inline]
    pub fn as_slices(&'a self) -> Slices<'a, 'a, T> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a T::Context, Slices<'a, 'a, T>) {
        let (context, slices) = self.as_slice_ptrs_with_context();
        let slices = unsafe { context.slice_ptrs_to_slices(slices) };
        (context, slices)
    }

    #[inline]
    pub fn as_mut_slices(&'a mut self) -> SlicesMut<'a, 'a, T> {
        let (_, slices) = self.as_mut_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_mut_slices_with_context(&'a mut self) -> (&'a T::Context, SlicesMut<'a, 'a, T>) {
        let (context, slices) = self.as_mut_slice_ptrs_with_context();
        let slices = unsafe { context.mut_slice_ptrs_to_mut_slices(slices) };
        (context, slices)
    }

    #[inline]
    pub fn get<I>(&'a self, index: I) -> Option<I::Refs<'a>>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.get_with_context(index);
        refs
    }

    #[inline]
    pub fn get_with_context<I>(&'a self, index: I) -> (&'a T::Context, Option<I::Refs<'a>>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        self.slices().into_get_with_context(index)
    }

    #[inline]
    pub fn get_mut<I>(&'a mut self, index: I) -> Option<I::RefsMut<'a>>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.get_mut_with_context(index);
        refs
    }

    #[inline]
    pub fn get_mut_with_context<I>(
        &'a mut self,
        index: I,
    ) -> (&'a T::Context, Option<I::RefsMut<'a>>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        self.mut_slices().into_get_mut_with_context(index)
    }

    #[inline]
    #[track_caller]
    pub fn index<I>(&'a self, index: I) -> I::Refs<'a>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_with_context<I>(&'a self, index: I) -> (&'a T::Context, I::Refs<'a>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        self.slices().into_index_with_context(index)
    }

    #[inline]
    #[track_caller]
    pub fn index_mut<I>(&'a mut self, index: I) -> I::RefsMut<'a>
    where
        I: SoaSlicesIndex<'a, T>,
    {
        let (_, refs) = self.index_mut_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_mut_with_context<I>(&'a mut self, index: I) -> (&'a T::Context, I::RefsMut<'a>)
    where
        I: SoaSlicesIndex<'a, T>,
    {
        self.mut_slices().into_index_mut_with_context(index)
    }

    #[inline]
    pub fn iter(&'a self) -> Iter<'a, 'a, T> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&'a self) -> (&'a T::Context, Iter<'a, 'a, T>) {
        self.slices().into_iter_with_context()
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> IterMut<'a, 'a, T> {
        let (_, iter) = self.iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn iter_mut_with_context(&'a mut self) -> (&'a T::Context, IterMut<'a, 'a, T>) {
        self.mut_slices().into_iter_with_context()
    }

    #[inline]
    pub fn contains<V>(&'a self, value: V) -> bool
    where
        Refs<'a, 'a, T>: PartialEq<V>,
    {
        let mut iter = self.into_iter();
        iter.any(move |item| item.eq(&value))
    }
}

impl<T> SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
    for<'a> T: Soa<'a>,
{
    #[inline]
    pub fn sort_unstable_with_permutation<P>(&mut self, permutation: P)
    where
        P: AsMut<[usize]>,
        for<'ctx, 'a> Refs<'ctx, 'a, T>: Ord,
    {
        self.mut_slices()
            .sort_unstable_with_permutation(permutation);
    }

    #[inline]
    pub fn sort_unstable_with_permutation_by<P, F>(&mut self, permutation: P, compare: F)
    where
        P: AsMut<[usize]>,
        for<'a> F: FnMut(Refs<'_, 'a, T>, Refs<'_, 'a, T>) -> cmp::Ordering,
    {
        self.mut_slices()
            .sort_unstable_with_permutation_by(permutation, compare);
    }

    #[inline]
    pub fn sort_unstable_with_permutation_by_key<P, K, F>(&mut self, permutation: P, f: F)
    where
        P: AsMut<[usize]>,
        F: FnMut(Refs<'_, '_, T>) -> K,
        K: Ord,
    {
        self.mut_slices()
            .sort_unstable_with_permutation_by_key(permutation, f);
    }
}

impl<T> SoaSlice<T>
where
    T: AllocSoaTrusted + SoaCloneToUninit + ?Sized,
{
    #[inline]
    #[track_caller]
    pub fn clone_from_slice(&mut self, src: &Self) {
        let src = src.slices();
        self.mut_slices().clone_from_slices(&src);
    }
}

impl<T> Debug for SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
    for<'a> T: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaSlice").field(&slices).finish()
    }
}

impl<T> AsRef<Self> for SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, U> AsRef<[U]> for SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
    for<'ctx, 'a> T: Soa<'a, Context: SoaContext<'a, Slices<'ctx> = &'a [U]>>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices()
    }
}

impl<T> AsMut<Self> for SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, U> AsMut<[U]> for SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
    for<'ctx, 'a> T: Soa<'a, Context: SoaContext<'a, SlicesMut<'ctx> = &'a mut [U]>>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [U] {
        self.as_mut_slices()
    }
}

impl<T> Eq for SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
    for<'a> T: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Eq,
{
}

impl<T> Ord for SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
    for<'a> T: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Ord,
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
    T: AllocSoaTrusted + ?Sized,
    for<'a> T: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let slices = self.as_slices();
        slices.hash(state);
    }
}

impl<T> Drop for SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    fn drop(&mut self) {
        if self.is_empty() {
            return;
        }

        let (context, slices) = self.as_mut_slice_ptrs_with_context();
        unsafe { context.slices_drop_in_place(slices) }
    }
}

impl<T, U, I> Index<I> for SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
    U: ?Sized,
    for<'a> T: Soa<'a>,
    for<'ctx, 'a> I: IndexHelper<'ctx, 'a, T, Output = U>,
{
    type Output = U;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Self::index(self, index)
    }
}

impl<T, U, I> IndexMut<I> for SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
    U: ?Sized,
    for<'a> T: Soa<'a>,
    for<'ctx, 'a> I: IndexHelperMut<'ctx, 'a, T, Output = U>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        Self::index_mut(self, index)
    }
}

impl<'a, T> IntoIterator for &'a SoaSlice<T>
where
    T: Soa<'a> + AllocSoaTrusted + ?Sized,
{
    type Item = Refs<'a, 'a, T>;
    type IntoIter = Iter<'a, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SoaSlice<T>
where
    T: Soa<'a> + AllocSoaTrusted + ?Sized,
{
    type Item = RefsMut<'a, 'a, T>;
    type IntoIter = IterMut<'a, 'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

unsafe impl<T> Send for SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
    T::Context: Send,
    T::Fields: Send,
{
}

unsafe impl<T> Sync for SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
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
    T: AllocSoaTrusted + ?Sized,
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
    T: AllocSoaTrusted + ?Sized,
{
    unsafe { &mut *slice_from_raw_parts_mut(data, len, capacity) }
}
