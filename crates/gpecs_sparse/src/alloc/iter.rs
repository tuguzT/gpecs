use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr,
};

use crate::{
    item::DenseItem,
    soa::{
        traits::{
            AllocSoa, MutPtrs, Ptrs, RawSoa, SliceMutPtrs, SlicePtrs, Slices, SlicesMut, Soa,
            SoaCloneToUninit, SoaOwned, SoaRead, SoaReadOwned,
        },
        vec,
    },
};

pub struct IntoKeys<K, V>
where
    V: RawSoa + ?Sized,
{
    inner: core_alloc::vec::IntoIter<K>,
    context: V::Context,
}

impl<K, V> IntoKeys<K, V>
where
    K: Clone,
    V: AllocSoa + ?Sized,
{
    #[inline]
    #[expect(clippy::unnecessary_to_owned, reason = "false positive")]
    pub(super) fn new(vec: vec::SoaVec<DenseItem<K, V>>) -> Self {
        let (keys, _) = vec.as_slice_ptrs().into_parts();
        let keys = unsafe { keys.as_ref_unchecked() };

        let inner = keys.to_vec().into_iter();
        let context = vec.into_context().into_inner();

        Self { inner, context }
    }
}

impl<K, V> IntoKeys<K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &V::Context {
        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub fn as_ptr(&self) -> *const K {
        let (_, ptr) = self.as_ptr_with_context();
        ptr
    }

    #[inline]
    pub fn as_ptr_with_context(&self) -> (&V::Context, *const K) {
        let Self { context, inner } = self;
        let ptr = inner.as_slice().as_ptr();
        (context, ptr)
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut K {
        let (_, ptr) = self.as_mut_ptr_with_context();
        ptr
    }

    #[inline]
    pub fn as_mut_ptr_with_context(&mut self) -> (&V::Context, *mut K) {
        let Self { context, inner } = self;
        let ptr = inner.as_mut_slice().as_mut_ptr();
        (context, ptr)
    }

    #[inline]
    pub fn as_slice_ptr(&self) -> *const [K] {
        let (_, slice) = self.as_slice_ptr_with_context();
        slice
    }

    #[inline]
    pub fn as_slice_ptr_with_context(&self) -> (&V::Context, *const [K]) {
        let Self { context, inner } = self;
        let slice = ptr::from_ref(inner.as_slice());
        (context, slice)
    }

    #[inline]
    pub fn as_mut_slice_ptr(&mut self) -> *mut [K] {
        let (_, slice) = self.as_mut_slice_ptr_with_context();
        slice
    }

    #[inline]
    pub fn as_mut_slice_ptr_with_context(&mut self) -> (&V::Context, *mut [K]) {
        let Self { context, inner } = self;
        let slice = ptr::from_mut(inner.as_mut_slice());
        (context, slice)
    }

    #[inline]
    pub fn as_slice(&self) -> &[K] {
        let (_, slice) = self.as_slice_with_context();
        slice
    }

    #[inline]
    pub fn as_slice_with_context(&self) -> (&V::Context, &[K]) {
        let Self { context, inner } = self;
        let slice = inner.as_slice();
        (context, slice)
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [K] {
        let (_, slice) = self.as_mut_slice_with_context();
        slice
    }

    #[inline]
    pub fn as_mut_slice_with_context(&mut self) -> (&V::Context, &mut [K]) {
        let Self { context, inner } = self;
        let mut_slice = inner.as_mut_slice();
        (context, mut_slice)
    }
}

impl<K, V> Debug for IntoKeys<K, V>
where
    K: Debug,
    V: RawSoa + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = &self.as_slice();
        f.debug_tuple("IntoKeys").field(keys).finish()
    }
}

impl<K, V> Default for IntoKeys<K, V>
where
    V: RawSoa + ?Sized,
    V::Context: Default,
{
    #[inline]
    fn default() -> Self {
        let context = V::Context::default();
        let inner = core_alloc::vec::IntoIter::default();
        Self { inner, context }
    }
}

impl<K, V> Clone for IntoKeys<K, V>
where
    K: Clone,
    V: RawSoa + ?Sized,
    V::Context: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { context, inner } = self;

        let context = context.clone();
        let inner = inner.clone();
        Self { inner, context }
    }
}

impl<K, V> AsRef<[K]> for IntoKeys<K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &[K] {
        self.as_slice()
    }
}

impl<K, V> AsMut<[K]> for IntoKeys<K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [K] {
        self.as_mut_slice()
    }
}

impl<K, V> Iterator for IntoKeys<K, V>
where
    V: RawSoa + ?Sized,
{
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner, .. } = self;
        inner.count()
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner, .. } = self;
        inner.fold(init, f)
    }
}

impl<K, V> DoubleEndedIterator for IntoKeys<K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        inner.next_back()
    }
}

impl<K, V> ExactSizeIterator for IntoKeys<K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        Self::len(self)
    }
}

impl<K, V> FusedIterator for IntoKeys<K, V> where V: RawSoa + ?Sized {}

#[repr(transparent)]
pub struct IntoValues<K, V, R>
where
    V: AllocSoa + ?Sized,
    R: ?Sized,
{
    inner: vec::IntoIter<DenseItem<K, V>, DenseItem<K, R>>,
}

impl<K, V, R> IntoValues<K, V, R>
where
    V: AllocSoa + ?Sized,
    R: ?Sized,
{
    #[inline]
    pub(super) fn new(inner: vec::IntoIter<DenseItem<K, V>, DenseItem<K, R>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &V::Context {
        let Self { inner } = self;
        inner.context()
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'_, V> {
        let (_, ptrs) = self.as_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&V::Context, Ptrs<'_, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_ptrs_with_context();
        let (_, ptrs) = ptrs.into_parts();
        (context, ptrs)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'_, V> {
        let (_, ptrs) = self.as_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&V::Context, MutPtrs<'_, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_mut_ptrs_with_context();
        let (_, ptrs) = ptrs.into_parts();
        (context, ptrs)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'_, V> {
        let (_, values) = self.as_slice_ptrs_with_context();
        values
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&V::Context, SlicePtrs<'_, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_slice_ptrs_with_context();
        let (_, values) = ptrs.into_parts();
        (context, values)
    }

    #[inline]
    pub fn as_mut_slice_ptrs(&mut self) -> SliceMutPtrs<'_, V> {
        let (_, values) = self.as_mut_slice_ptrs_with_context();
        values
    }

    #[inline]
    pub fn as_mut_slice_ptrs_with_context(&mut self) -> (&V::Context, SliceMutPtrs<'_, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_mut_slice_ptrs_with_context();
        let (_, values) = ptrs.into_parts();
        (context, values)
    }
}

impl<'a, K, V, R> IntoValues<K, V, R>
where
    V: AllocSoa + Soa<'a> + ?Sized,
    R: ?Sized,
{
    #[inline]
    pub fn as_slices(&'a self) -> Slices<'a, 'a, V> {
        let (_, values) = self.as_slices_with_context();
        values
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a V::Context, Slices<'a, 'a, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_slices_with_context();
        let (_, values) = slices.into_parts();
        (context, values)
    }

    #[inline]
    pub fn as_mut_slices(&'a mut self) -> SlicesMut<'a, 'a, V> {
        let (_, values) = self.as_mut_slices_with_context();
        values
    }

    #[inline]
    pub fn as_mut_slices_with_context(&'a mut self) -> (&'a V::Context, SlicesMut<'a, 'a, V>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_mut_slices_with_context();
        let (_, values) = slices.into_parts();
        (context, values)
    }
}

impl<K, V, R> Debug for IntoValues<K, V, R>
where
    V: SoaOwned + AllocSoa + ?Sized,
    R: ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slices();
        f.debug_tuple("IntoValues").field(values).finish()
    }
}

impl<K, V, R> Default for IntoValues<K, V, R>
where
    V: AllocSoa + ?Sized,
    V::Context: Default,
    R: ?Sized,
{
    #[inline]
    fn default() -> Self {
        let inner = vec::IntoIter::default();
        Self::new(inner)
    }
}

impl<K, V, R> Clone for IntoValues<K, V, R>
where
    K: Clone,
    V: AllocSoa + SoaCloneToUninit + ?Sized,
    V::Context: Clone,
    R: ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;
        let inner = inner.clone();
        Self { inner }
    }
}

impl<K, V, R, U> AsRef<[U]> for IntoValues<K, V, R>
where
    V: SoaOwned + AllocSoa + ?Sized,
    R: ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Into<&'a [U]>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices().into()
    }
}

impl<K, V, R, U> AsMut<[U]> for IntoValues<K, V, R>
where
    V: SoaOwned + AllocSoa + ?Sized,
    R: ?Sized,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Into<&'a mut [U]>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [U] {
        self.as_mut_slices().into()
    }
}

impl<K, V, R> Iterator for IntoValues<K, V, R>
where
    V: AllocSoa + SoaReadOwned<R> + ?Sized,
{
    type Item = R;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|DenseItem { value, .. }| value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self { inner } = self;
        inner.fold(init, |acc, DenseItem { value, .. }| f(acc, value))
    }
}

impl<K, V, R> DoubleEndedIterator for IntoValues<K, V, R>
where
    V: AllocSoa + SoaReadOwned<R> + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|DenseItem { value, .. }| value)
    }
}

impl<K, V, R> ExactSizeIterator for IntoValues<K, V, R>
where
    V: AllocSoa + SoaReadOwned<R> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        Self::len(self)
    }
}

impl<K, V, R> FusedIterator for IntoValues<K, V, R> where V: AllocSoa + SoaReadOwned<R> + ?Sized {}

#[repr(transparent)]
pub struct IntoIter<K, V, R>
where
    V: AllocSoa + ?Sized,
    R: ?Sized,
{
    inner: vec::IntoIter<DenseItem<K, V>, DenseItem<K, R>>,
}

impl<K, V, R> IntoIter<K, V, R>
where
    V: AllocSoa + ?Sized,
    R: ?Sized,
{
    #[inline]
    pub(super) fn new(inner: vec::IntoIter<DenseItem<K, V>, DenseItem<K, R>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &V::Context {
        let Self { inner } = self;
        inner.context()
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const K, Ptrs<'_, V>) {
        let (_, key, value) = self.as_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&V::Context, *const K, Ptrs<'_, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*mut K, MutPtrs<'_, V>) {
        let (_, key, value) = self.as_mut_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&V::Context, *mut K, MutPtrs<'_, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_mut_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (*const [K], SlicePtrs<'_, V>) {
        let (_, keys, values) = self.as_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&V::Context, *const [K], SlicePtrs<'_, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_slice_ptrs_with_context();
        let (keys, values) = ptrs.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn as_mut_slice_ptrs(&mut self) -> (*mut [K], SliceMutPtrs<'_, V>) {
        let (_, keys, values) = self.as_mut_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_mut_slice_ptrs_with_context(
        &mut self,
    ) -> (&V::Context, *mut [K], SliceMutPtrs<'_, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_mut_slice_ptrs_with_context();
        let (keys, values) = ptrs.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn as_key_slice(&self) -> &[K] {
        let (_, keys) = self.as_key_slice_with_context();
        keys
    }

    #[inline]
    pub fn as_key_slice_with_context(&self) -> (&V::Context, &[K]) {
        let (context, keys, _) = self.as_slice_ptrs_with_context();
        let keys = unsafe { keys.as_ref_unchecked() };
        (context, keys)
    }

    #[inline]
    pub fn as_mut_key_slice(&mut self) -> &mut [K] {
        let (_, keys) = self.as_mut_key_slice_with_context();
        keys
    }

    #[inline]
    pub fn as_mut_key_slice_with_context(&mut self) -> (&V::Context, &mut [K]) {
        let (context, keys, _) = self.as_mut_slice_ptrs_with_context();
        let keys = unsafe { keys.as_mut_unchecked() };
        (context, keys)
    }
}

impl<'a, K, V, R> IntoIter<K, V, R>
where
    V: AllocSoa + Soa<'a> + ?Sized,
    R: ?Sized,
{
    #[inline]
    pub fn as_value_slices(&'a self) -> Slices<'a, 'a, V> {
        let (_, values) = self.as_value_slices_with_context();
        values
    }

    #[inline]
    pub fn as_value_slices_with_context(&'a self) -> (&'a V::Context, Slices<'a, 'a, V>) {
        let (context, _, values) = self.as_slices_with_context();
        (context, values)
    }

    #[inline]
    pub fn as_mut_value_slices(&'a mut self) -> SlicesMut<'a, 'a, V> {
        let (_, values) = self.as_mut_value_slices_with_context();
        values
    }

    #[inline]
    pub fn as_mut_value_slices_with_context(
        &'a mut self,
    ) -> (&'a V::Context, SlicesMut<'a, 'a, V>) {
        let (context, _, values) = self.as_mut_slices_with_context();
        (context, values)
    }

    #[inline]
    pub fn as_slices(&'a self) -> (&'a [K], Slices<'a, 'a, V>) {
        let (_, keys, values) = self.as_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a V::Context, &'a [K], Slices<'a, 'a, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_slices_with_context();
        let (keys, values) = ptrs.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn as_mut_slices(&'a mut self) -> (&'a mut [K], SlicesMut<'a, 'a, V>) {
        let (_, keys, values) = self.as_mut_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_mut_slices_with_context(
        &'a mut self,
    ) -> (&'a V::Context, &'a mut [K], SlicesMut<'a, 'a, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_mut_slices_with_context();
        let (keys, values) = ptrs.into_parts();
        (context, keys, values)
    }
}

impl<K, V, R> Debug for IntoIter<K, V, R>
where
    K: Debug,
    V: SoaOwned + AllocSoa + ?Sized,
    R: ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = &self.as_slices();
        f.debug_struct("IntoIter")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V, R> Default for IntoIter<K, V, R>
where
    V: AllocSoa + ?Sized,
    V::Context: Default,
    R: ?Sized,
{
    #[inline]
    fn default() -> Self {
        let inner = vec::IntoIter::default();
        Self::new(inner)
    }
}

impl<K, V, R> Clone for IntoIter<K, V, R>
where
    K: Clone,
    V: AllocSoa + SoaCloneToUninit + ?Sized,
    V::Context: Clone,
    R: ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;
        let inner = inner.clone();
        Self { inner }
    }
}

impl<K, V, R, U> AsRef<[U]> for IntoIter<K, V, R>
where
    V: SoaOwned + AllocSoa + ?Sized,
    R: ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Into<&'a [U]>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_value_slices().into()
    }
}

impl<K, V, R, U> AsMut<[U]> for IntoIter<K, V, R>
where
    V: SoaOwned + AllocSoa + ?Sized,
    R: ?Sized,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Into<&'a mut [U]>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [U] {
        self.as_mut_value_slices().into()
    }
}

impl<K, V, R> Iterator for IntoIter<K, V, R>
where
    V: AllocSoa + SoaReadOwned<R> + ?Sized,
{
    type Item = (K, R);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(Into::into)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.count()
    }
}

impl<K, V, R> DoubleEndedIterator for IntoIter<K, V, R>
where
    V: AllocSoa + SoaReadOwned<R> + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(Into::into)
    }
}

impl<K, V, R> ExactSizeIterator for IntoIter<K, V, R>
where
    V: AllocSoa + SoaReadOwned<R> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        Self::len(self)
    }
}

impl<K, V, R> FusedIterator for IntoIter<K, V, R> where V: AllocSoa + SoaReadOwned<R> + ?Sized {}

#[repr(transparent)]
pub struct Drain<'a, K, V, R>
where
    V: AllocSoa + ?Sized,
    V::Context: 'a,
    R: ?Sized,
{
    inner: vec::Drain<'a, DenseItem<K, V>, DenseItem<K, R>>,
}

impl<'a, K, V, R> Drain<'a, K, V, R>
where
    V: AllocSoa + ?Sized,
    R: ?Sized,
{
    #[inline]
    pub(super) fn new(inner: vec::Drain<'a, DenseItem<K, V>, DenseItem<K, R>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &V::Context {
        let Self { inner } = self;
        inner.context()
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const K, Ptrs<'_, V>) {
        let (_, key, value) = self.as_ptrs_with_context();
        (key, value)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&V::Context, *const K, Ptrs<'_, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_ptrs_with_context();
        let (key, value) = ptrs.into_parts();
        (context, key, value)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (*const [K], SlicePtrs<'_, V>) {
        let (_, keys, values) = self.as_slice_ptrs_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&V::Context, *const [K], SlicePtrs<'_, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_slice_ptrs_with_context();
        let (keys, values) = ptrs.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn as_key_slice(&self) -> &[K] {
        let (_, keys) = self.as_key_slice_with_context();
        keys
    }

    #[inline]
    pub fn as_key_slice_with_context(&self) -> (&V::Context, &[K]) {
        let (context, keys, _) = self.as_slice_ptrs_with_context();
        let keys = unsafe { keys.as_ref_unchecked() };
        (context, keys)
    }
}

impl<'a, K, V, R> Drain<'_, K, V, R>
where
    V: AllocSoa + Soa<'a> + ?Sized,
    R: ?Sized,
{
    #[inline]
    pub fn as_value_slices(&'a self) -> Slices<'a, 'a, V> {
        let (_, values) = self.as_value_slices_with_context();
        values
    }

    #[inline]
    pub fn as_value_slices_with_context(&'a self) -> (&'a V::Context, Slices<'a, 'a, V>) {
        let (context, _, values) = self.as_slices_with_context();
        (context, values)
    }

    #[inline]
    pub fn as_slices(&'a self) -> (&'a [K], Slices<'a, 'a, V>) {
        let (_, keys, values) = self.as_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a V::Context, &'a [K], Slices<'a, 'a, V>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_slices_with_context();
        let (keys, values) = ptrs.into_parts();
        (context, keys, values)
    }
}

impl<K, V, R> Debug for Drain<'_, K, V, R>
where
    K: Debug,
    V: SoaOwned + AllocSoa + ?Sized,
    R: ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = &self.as_slices();
        f.debug_struct("Drain")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V, R, U> AsRef<[U]> for Drain<'_, K, V, R>
where
    V: SoaOwned + AllocSoa + ?Sized,
    R: ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Into<&'a [U]>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_value_slices().into()
    }
}

impl<'a, K, V, R> Iterator for Drain<'a, K, V, R>
where
    V: AllocSoa + SoaRead<'a, R> + ?Sized,
{
    type Item = (K, R);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(Into::into)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<'a, K, V, R> DoubleEndedIterator for Drain<'a, K, V, R>
where
    V: AllocSoa + SoaRead<'a, R> + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(Into::into)
    }
}

impl<'a, K, V, R> ExactSizeIterator for Drain<'a, K, V, R>
where
    V: AllocSoa + SoaRead<'a, R> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<'a, K, V, R> FusedIterator for Drain<'a, K, V, R> where V: AllocSoa + SoaRead<'a, R> + ?Sized {}
