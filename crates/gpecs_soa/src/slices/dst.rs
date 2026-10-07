use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::{Index, IndexMut},
    ptr::NonNull,
};

use crate::{
    buffer::dst::DstBuffer,
    ptrs::{IterMutPtrs, IterPtrs, SlicePtrsIndex, SoaViewMutPtrs, SoaViewPtrs},
    slices::{IndexHelper, IndexHelperMut, Iter, IterMut, SlicesIndex, SoaView, SoaViewMut},
    traits::{
        MutPtrs, Ptrs, Refs, RefsMut, SliceMutPtrs, SlicePtrs, Slices, SlicesMut, Soa,
        SoaAllocTrusted, SoaCloneToUninit, SoaContext, SoaOwned, SoaRawContext,
    },
};

#[repr(transparent)]
pub struct SoaSlice<T>
where
    T: SoaAllocTrusted + ?Sized,
{
    buffer: DstBuffer<T>,
}

impl<T> SoaSlice<T>
where
    T: SoaAllocTrusted + ?Sized,
{
    pub(crate) unsafe fn ptr_from_raw_parts(
        data: *const u8,
        len: usize,
        capacity: usize,
    ) -> *const Self {
        let buffer = unsafe { DstBuffer::ptr_from_raw_parts(data, len, capacity) };
        Self::ptr_from_inner(buffer)
    }

    pub(crate) unsafe fn ptr_from_raw_parts_mut(
        data: *mut u8,
        len: usize,
        capacity: usize,
    ) -> *mut Self {
        let buffer = unsafe { DstBuffer::ptr_from_raw_parts_mut(data, len, capacity) };
        Self::ptr_from_inner_mut(buffer)
    }

    pub(crate) unsafe fn ptr_from_raw_parts_nonnull(
        data: NonNull<u8>,
        len: usize,
        capacity: usize,
    ) -> NonNull<Self> {
        let buffer = unsafe { DstBuffer::ptr_from_raw_parts_nonnull(data, len, capacity) };
        Self::ptr_from_inner_nonnull(buffer)
    }

    fn ptr_from_inner(buffer: *const DstBuffer<T>) -> *const Self {
        // Self is transparent over `DstBuffer<T>`
        buffer as _
    }

    fn ptr_from_inner_mut(buffer: *mut DstBuffer<T>) -> *mut Self {
        // Self is transparent over `DstBuffer<T>`
        buffer as _
    }

    fn ptr_from_inner_nonnull(buffer: NonNull<DstBuffer<T>>) -> NonNull<Self> {
        // Self is transparent over `DstBuffer<T>`
        let ptr = Self::ptr_from_inner_mut(buffer.as_ptr());
        unsafe { NonNull::new_unchecked(ptr) }
    }

    pub(crate) fn ptr_as_ptr(this: *const Self) -> *const u8 {
        let this = Self::ptr_as_inner(this);
        DstBuffer::ptr_as_ptr(this)
    }

    pub(crate) fn ptr_as_mut_ptr(this: *mut Self) -> *mut u8 {
        let this = Self::ptr_as_inner_mut(this);
        DstBuffer::ptr_as_mut_ptr(this)
    }

    fn ptr_as_inner(this: *const Self) -> *const DstBuffer<T> {
        // Self is transparent over `DstBuffer<T>`
        this as _
    }

    fn ptr_as_inner_mut(this: *mut Self) -> *mut DstBuffer<T> {
        // Self is transparent over `DstBuffer<T>`
        this as _
    }

    pub(crate) unsafe fn ptr_len(this: *const Self) -> usize {
        let this = Self::ptr_as_inner(this);
        unsafe { DstBuffer::ptr_len(this) }
    }

    pub(crate) unsafe fn ptr_capacity(this: *const Self) -> usize {
        let this = Self::ptr_as_inner(this);
        unsafe { DstBuffer::ptr_capacity(this) }
    }

    #[inline]
    pub unsafe fn from_raw_parts<'a>(data: *const u8, len: usize, capacity: usize) -> &'a Self {
        let this = unsafe { Self::ptr_from_raw_parts(data, len, capacity) };
        unsafe { this.as_ref_unchecked() }
    }

    #[inline]
    pub unsafe fn from_raw_parts_mut<'a>(
        data: *mut u8,
        len: usize,
        capacity: usize,
    ) -> &'a mut Self {
        let this = unsafe { Self::ptr_from_raw_parts_mut(data, len, capacity) };
        unsafe { this.as_mut_unchecked() }
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let Self { buffer } = self;
        buffer.context()
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { buffer } = self;
        buffer.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { buffer } = self;
        buffer.is_empty()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { buffer } = self;
        buffer.capacity()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { buffer } = self;
        buffer.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
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
        let Self { buffer } = self;
        buffer.as_ptrs_with_context()
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'_, T> {
        let (_, ptrs) = self.as_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&T::Context, MutPtrs<'_, T>) {
        let Self { buffer } = self;
        buffer.as_mut_ptrs_with_context()
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
    pub fn as_view_ptrs(&self) -> SoaViewPtrs<'_, T> {
        let (_, view) = self.as_view_ptrs_with_context();
        view
    }

    #[inline]
    pub fn as_view_ptrs_with_context(&self) -> (&T::Context, SoaViewPtrs<'_, T>) {
        let (context, slices) = self.as_slice_ptrs_with_context();
        let view = SoaViewPtrs::new(context, slices);
        (context, view)
    }

    #[inline]
    pub fn as_mut_view_ptrs(&mut self) -> SoaViewMutPtrs<'_, T> {
        let (_, view) = self.as_mut_view_ptrs_with_context();
        view
    }

    #[inline]
    pub fn as_mut_view_ptrs_with_context(&mut self) -> (&T::Context, SoaViewMutPtrs<'_, T>) {
        let (context, slices) = self.as_mut_slice_ptrs_with_context();
        let view = SoaViewMutPtrs::new(context, slices);
        (context, view)
    }

    #[inline]
    pub fn as_view(&self) -> SoaView<'_, '_, T> {
        let (_, view) = self.as_view_with_context();
        view
    }

    #[inline]
    pub fn as_view_with_context(&self) -> (&T::Context, SoaView<'_, '_, T>) {
        let (context, view) = self.as_view_ptrs_with_context();
        let view = unsafe { view.as_ref_unchecked() };
        (context, view)
    }

    #[inline]
    pub fn as_mut_view(&mut self) -> SoaViewMut<'_, '_, T> {
        let (_, view) = self.as_mut_view_with_context();
        view
    }

    #[inline]
    pub fn as_mut_view_with_context(&mut self) -> (&T::Context, SoaViewMut<'_, '_, T>) {
        let (context, view) = self.as_mut_view_ptrs_with_context();
        let view = unsafe { view.as_mut_unchecked() };
        (context, view)
    }

    #[inline]
    #[track_caller]
    pub fn copy_from_slice(&mut self, src: &Self)
    where
        T::Fields: Copy,
    {
        let src = src.as_view();
        self.as_mut_view().copy_from_slices(&src);
    }

    #[inline]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::Ptrs<'_>
    where
        I: SlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.get_unchecked_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_with_context<I>(&self, index: I) -> (&T::Context, I::Ptrs<'_>)
    where
        I: SlicePtrsIndex<T>,
    {
        let view = self.as_view_ptrs();
        unsafe { view.into_get_unchecked_with_context(index) }
    }

    #[inline]
    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> I::MutPtrs<'_>
    where
        I: SlicePtrsIndex<T>,
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
        I: SlicePtrsIndex<T>,
    {
        let view = self.as_mut_view_ptrs();
        unsafe { view.into_get_unchecked_mut_with_context(index) }
    }

    #[inline]
    pub fn iter_ptrs(&self) -> IterPtrs<'_, T> {
        let (_, iter) = self.iter_ptrs_with_context();
        iter
    }

    #[inline]
    pub fn iter_ptrs_with_context(&self) -> (&T::Context, IterPtrs<'_, T>) {
        self.as_view().into_iter_ptrs_with_context()
    }

    #[inline]
    pub fn iter_mut_ptrs(&mut self) -> IterMutPtrs<'_, T> {
        let (_, iter) = self.iter_mut_ptrs_with_context();
        iter
    }

    #[inline]
    pub fn iter_mut_ptrs_with_context(&mut self) -> (&T::Context, IterMutPtrs<'_, T>) {
        self.as_mut_view().into_iter_mut_ptrs_with_context()
    }

    #[inline]
    #[track_caller]
    pub fn swap(&mut self, a: usize, b: usize) {
        self.as_mut_view().swap(a, b);
    }
}

impl<'a, T> SoaSlice<T>
where
    T: Soa<'a> + SoaAllocTrusted + ?Sized,
{
    #[inline]
    pub fn as_slices(&'a self) -> Slices<'a, 'a, T> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a T::Context, Slices<'a, 'a, T>) {
        let (context, slices) = self.as_slice_ptrs_with_context();
        let slices = unsafe { context.slices_from_slice_ptrs(slices) };
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
        let slices = unsafe { context.mut_slices_from_mut_slice_ptrs(slices) };
        (context, slices)
    }

    #[inline]
    pub fn get<I>(&'a self, index: I) -> Option<I::Refs<'a>>
    where
        I: SlicesIndex<'a, T>,
    {
        let (_, refs) = self.get_with_context(index);
        refs
    }

    #[inline]
    pub fn get_with_context<I>(&'a self, index: I) -> (&'a T::Context, Option<I::Refs<'a>>)
    where
        I: SlicesIndex<'a, T>,
    {
        self.as_view().into_get_with_context(index)
    }

    #[inline]
    pub fn get_mut<I>(&'a mut self, index: I) -> Option<I::RefsMut<'a>>
    where
        I: SlicesIndex<'a, T>,
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
        I: SlicesIndex<'a, T>,
    {
        self.as_mut_view().into_get_mut_with_context(index)
    }

    #[inline]
    #[track_caller]
    pub fn index<I>(&'a self, index: I) -> I::Refs<'a>
    where
        I: SlicesIndex<'a, T>,
    {
        let (_, refs) = self.index_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_with_context<I>(&'a self, index: I) -> (&'a T::Context, I::Refs<'a>)
    where
        I: SlicesIndex<'a, T>,
    {
        self.as_view().into_index_with_context(index)
    }

    #[inline]
    #[track_caller]
    pub fn index_mut<I>(&'a mut self, index: I) -> I::RefsMut<'a>
    where
        I: SlicesIndex<'a, T>,
    {
        let (_, refs) = self.index_mut_with_context(index);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_mut_with_context<I>(&'a mut self, index: I) -> (&'a T::Context, I::RefsMut<'a>)
    where
        I: SlicesIndex<'a, T>,
    {
        self.as_mut_view().into_index_mut_with_context(index)
    }

    #[inline]
    pub fn iter(&'a self) -> Iter<'a, 'a, T> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&'a self) -> (&'a T::Context, Iter<'a, 'a, T>) {
        self.as_view().into_iter_with_context()
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> IterMut<'a, 'a, T> {
        let (_, iter) = self.iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn iter_mut_with_context(&'a mut self) -> (&'a T::Context, IterMut<'a, 'a, T>) {
        self.as_mut_view().into_iter_with_context()
    }

    #[inline]
    pub fn contains<V>(&'a self, value: V) -> bool
    where
        Refs<'a, 'a, T>: PartialEq<V>,
    {
        self.iter().any(move |item| item.eq(&value))
    }
}

impl<T> SoaSlice<T>
where
    T: SoaOwned + SoaAllocTrusted + ?Sized,
{
    #[inline]
    pub fn sort_unstable_with_permutation<P>(&mut self, permutation: P)
    where
        P: AsMut<[usize]>,
        for<'ctx, 'a> Refs<'ctx, 'a, T>: Ord,
    {
        self.as_mut_view()
            .sort_unstable_with_permutation(permutation);
    }

    #[inline]
    pub fn sort_unstable_with_permutation_by<P, F>(&mut self, permutation: P, compare: F)
    where
        P: AsMut<[usize]>,
        for<'a> F: FnMut(Refs<'_, 'a, T>, Refs<'_, 'a, T>) -> cmp::Ordering,
    {
        self.as_mut_view()
            .sort_unstable_with_permutation_by(permutation, compare);
    }

    #[inline]
    pub fn sort_unstable_with_permutation_by_key<P, K, F>(&mut self, permutation: P, f: F)
    where
        P: AsMut<[usize]>,
        F: FnMut(Refs<'_, '_, T>) -> K,
        K: Ord,
    {
        self.as_mut_view()
            .sort_unstable_with_permutation_by_key(permutation, f);
    }
}

impl<T> SoaSlice<T>
where
    T: SoaAllocTrusted + SoaCloneToUninit + ?Sized,
{
    #[inline]
    #[track_caller]
    pub fn clone_from_slice(&mut self, src: &Self) {
        let src = src.as_view();
        self.as_mut_view().clone_from_slices(&src);
    }
}

impl<T> Debug for SoaSlice<T>
where
    T: SoaOwned + SoaAllocTrusted + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("SoaSlice").field(&slices).finish()
    }
}

impl<T> AsRef<Self> for SoaSlice<T>
where
    T: SoaAllocTrusted + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, U> AsRef<[U]> for SoaSlice<T>
where
    T: SoaAllocTrusted + ?Sized,
    for<'ctx, 'a> T: Soa<'a, Context: SoaContext<'a, T, Slices<'ctx> = &'a [U]>>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices()
    }
}

impl<T> AsMut<Self> for SoaSlice<T>
where
    T: SoaAllocTrusted + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, U> AsMut<[U]> for SoaSlice<T>
where
    T: SoaAllocTrusted + ?Sized,
    for<'ctx, 'a> T: Soa<'a, Context: SoaContext<'a, T, SlicesMut<'ctx> = &'a mut [U]>>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [U] {
        self.as_mut_slices()
    }
}

impl<T> Eq for SoaSlice<T>
where
    T: SoaOwned + SoaAllocTrusted + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Eq,
{
}

impl<T> Ord for SoaSlice<T>
where
    T: SoaOwned + SoaAllocTrusted + ?Sized,
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
    T: SoaOwned + SoaAllocTrusted + ?Sized,
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
    T: SoaAllocTrusted + ?Sized,
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
    T: SoaOwned + SoaAllocTrusted + ?Sized,
    U: ?Sized,
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
    T: SoaOwned + SoaAllocTrusted + ?Sized,
    U: ?Sized,
    for<'ctx, 'a> I: IndexHelperMut<'ctx, 'a, T, Output = U>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        Self::index_mut(self, index)
    }
}

impl<'a, T> IntoIterator for &'a SoaSlice<T>
where
    T: Soa<'a> + SoaAllocTrusted + ?Sized,
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
    T: Soa<'a> + SoaAllocTrusted + ?Sized,
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
    T: SoaAllocTrusted + ?Sized,
    T::Context: Send,
    T::Fields: Send,
{
}

unsafe impl<T> Sync for SoaSlice<T>
where
    T: SoaAllocTrusted + ?Sized,
    T::Context: Sync,
    T::Fields: Sync,
{
}
