use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr, slice,
};

use crate::{
    item::DenseItem,
    soa::{
        traits::{MutPtrs, Ptrs, RawSoa, SliceMutPtrs, SlicePtrs, Soa, SoaCloneToUninit, SoaRead},
        vec,
    },
};

pub struct IntoKeys<K, V>
where
    V: RawSoa + ?Sized,
{
    context: V::Context,
    inner: core_alloc::vec::IntoIter<K>,
}

impl<K, V> IntoKeys<K, V>
where
    K: Clone,
    V: RawSoa + ?Sized,
{
    #[inline]
    #[expect(clippy::unnecessary_to_owned, reason = "false positive")]
    pub(super) fn new(vec: vec::SoaVec<DenseItem<K, V>>) -> Self {
        let (keys, _) = vec.as_slice_ptrs().into_parts();
        let keys = unsafe { slice::from_raw_parts(keys.cast(), keys.len()) };

        let inner = keys.to_vec().into_iter();
        let context = vec.into_context().into_inner();

        Self { context, inner }
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
        Self { context, inner }
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
        Self { context, inner }
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
pub struct IntoValues<K, V>
where
    V: RawSoa + ?Sized,
{
    inner: vec::IntoIter<DenseItem<K, V>>,
}

impl<K, V> IntoValues<K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(super) fn new(inner: vec::IntoIter<DenseItem<K, V>>) -> Self {
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

impl<'a, K, V> IntoValues<K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn as_slices(&'a self) -> V::Slices<'a> {
        let (_, values) = self.as_slices_with_context();
        values
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a V::Context, V::Slices<'a>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_slices_with_context();
        let (_, values) = slices.into_parts();
        (context, values)
    }

    #[inline]
    pub fn as_mut_slices(&'a mut self) -> V::SlicesMut<'a> {
        let (_, values) = self.as_mut_slices_with_context();
        values
    }

    #[inline]
    pub fn as_mut_slices_with_context(&'a mut self) -> (&'a V::Context, V::SlicesMut<'a>) {
        let Self { inner } = self;

        let (context, slices) = inner.as_mut_slices_with_context();
        let (_, values) = slices.into_parts();
        (context, values)
    }
}

impl<K, V> Debug for IntoValues<K, V>
where
    V: ?Sized,
    for<'ctx, 'a> V: Soa<'a, Slices<'ctx>: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.as_slices();
        f.debug_tuple("IntoValues").field(values).finish()
    }
}

impl<K, V> Default for IntoValues<K, V>
where
    V: RawSoa + ?Sized,
    V::Context: Default,
{
    #[inline]
    fn default() -> Self {
        let inner = vec::IntoIter::default();
        Self::new(inner)
    }
}

impl<K, V> Clone for IntoValues<K, V>
where
    K: Clone,
    V: SoaCloneToUninit + ?Sized,
    V::Context: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;
        let inner = inner.clone();
        Self { inner }
    }
}

impl<T, K, V> AsRef<[T]> for IntoValues<K, V>
where
    V: ?Sized,
    for<'ctx, 'a> V: Soa<'a, Slices<'ctx>: Into<&'a [T]>>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slices().into()
    }
}

impl<T, K, V> AsMut<[T]> for IntoValues<K, V>
where
    V: ?Sized,
    for<'ctx, 'a> V: Soa<'a, SlicesMut<'ctx>: Into<&'a mut [T]>>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slices().into()
    }
}

impl<K, V> Iterator for IntoValues<K, V>
where
    V: SoaRead,
{
    type Item = V;

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

impl<K, V> DoubleEndedIterator for IntoValues<K, V>
where
    V: SoaRead,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(|DenseItem { value, .. }| value)
    }
}

impl<K, V> ExactSizeIterator for IntoValues<K, V>
where
    V: SoaRead,
{
    #[inline]
    fn len(&self) -> usize {
        Self::len(self)
    }
}

impl<K, V> FusedIterator for IntoValues<K, V> where V: SoaRead {}

#[repr(transparent)]
pub struct IntoIter<K, V>
where
    V: RawSoa + ?Sized,
{
    inner: vec::IntoIter<DenseItem<K, V>>,
}

impl<K, V> IntoIter<K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(super) fn new(inner: vec::IntoIter<DenseItem<K, V>>) -> Self {
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
        let keys = unsafe { slice::from_raw_parts(keys.cast(), keys.len()) };
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
        let keys = unsafe { slice::from_raw_parts_mut(keys.cast(), keys.len()) };
        (context, keys)
    }
}

impl<'a, K, V> IntoIter<K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn as_value_slices(&'a self) -> V::Slices<'a> {
        let (_, values) = self.as_value_slices_with_context();
        values
    }

    #[inline]
    pub fn as_value_slices_with_context(&'a self) -> (&'a V::Context, V::Slices<'a>) {
        let (context, _, values) = self.as_slices_with_context();
        (context, values)
    }

    #[inline]
    pub fn as_mut_value_slices(&'a mut self) -> V::SlicesMut<'a> {
        let (_, values) = self.as_mut_value_slices_with_context();
        values
    }

    #[inline]
    pub fn as_mut_value_slices_with_context(&'a mut self) -> (&'a V::Context, V::SlicesMut<'a>) {
        let (context, _, values) = self.as_mut_slices_with_context();
        (context, values)
    }

    #[inline]
    pub fn as_slices(&'a self) -> (&'a [K], V::Slices<'a>) {
        let (_, keys, values) = self.as_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a V::Context, &'a [K], V::Slices<'a>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_slices_with_context();
        let (keys, values) = ptrs.into_parts();
        (context, keys, values)
    }

    #[inline]
    pub fn as_mut_slices(&'a mut self) -> (&'a mut [K], V::SlicesMut<'a>) {
        let (_, keys, values) = self.as_mut_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_mut_slices_with_context(
        &'a mut self,
    ) -> (&'a V::Context, &'a mut [K], V::SlicesMut<'a>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_mut_slices_with_context();
        let (keys, values) = ptrs.into_parts();
        (context, keys, values)
    }
}

impl<K, V> Debug for IntoIter<K, V>
where
    K: Debug,
    V: ?Sized,
    for<'ctx, 'a> V: Soa<'a, Slices<'ctx>: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = &self.as_slices();
        f.debug_struct("IntoIter")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> Default for IntoIter<K, V>
where
    V: RawSoa + ?Sized,
    V::Context: Default,
{
    #[inline]
    fn default() -> Self {
        let inner = vec::IntoIter::default();
        Self::new(inner)
    }
}

impl<K, V> Clone for IntoIter<K, V>
where
    K: Clone,
    V: SoaCloneToUninit + ?Sized,
    V::Context: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;
        let inner = inner.clone();
        Self { inner }
    }
}

impl<T, K, V> AsRef<[T]> for IntoIter<K, V>
where
    V: ?Sized,
    for<'ctx, 'a> V: Soa<'a, Slices<'ctx>: Into<&'a [T]>>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_value_slices().into()
    }
}

impl<T, K, V> AsMut<[T]> for IntoIter<K, V>
where
    V: ?Sized,
    for<'ctx, 'a> V: Soa<'a, SlicesMut<'ctx>: Into<&'a mut [T]>>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_value_slices().into()
    }
}

impl<K, V> Iterator for IntoIter<K, V>
where
    V: SoaRead,
{
    type Item = (K, V);

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

impl<K, V> DoubleEndedIterator for IntoIter<K, V>
where
    V: SoaRead,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(Into::into)
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V>
where
    V: SoaRead,
{
    #[inline]
    fn len(&self) -> usize {
        Self::len(self)
    }
}

impl<K, V> FusedIterator for IntoIter<K, V> where V: SoaRead {}

#[repr(transparent)]
pub struct Drain<'a, K, V>
where
    V: RawSoa + ?Sized,
{
    inner: vec::Drain<'a, DenseItem<K, V>>,
}

impl<'a, K, V> Drain<'a, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub(super) fn new(inner: vec::Drain<'a, DenseItem<K, V>>) -> Self {
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
        let keys = unsafe { slice::from_raw_parts(keys.cast(), keys.len()) };
        (context, keys)
    }
}

impl<'a, K, V> Drain<'_, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn as_value_slices(&'a self) -> V::Slices<'a> {
        let (_, values) = self.as_value_slices_with_context();
        values
    }

    #[inline]
    pub fn as_value_slices_with_context(&'a self) -> (&'a V::Context, V::Slices<'a>) {
        let (context, _, values) = self.as_slices_with_context();
        (context, values)
    }

    #[inline]
    pub fn as_slices(&'a self) -> (&'a [K], V::Slices<'a>) {
        let (_, keys, values) = self.as_slices_with_context();
        (keys, values)
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a V::Context, &'a [K], V::Slices<'a>) {
        let Self { inner } = self;

        let (context, ptrs) = inner.as_slices_with_context();
        let (keys, values) = ptrs.into_parts();
        (context, keys, values)
    }
}

impl<K, V> Debug for Drain<'_, K, V>
where
    K: Debug,
    V: ?Sized,
    for<'ctx, 'a> V: Soa<'a, Slices<'ctx>: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keys, values) = &self.as_slices();
        f.debug_struct("Drain")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<T, K, V> AsRef<[T]> for Drain<'_, K, V>
where
    V: ?Sized,
    for<'ctx, 'a> V: Soa<'a, Slices<'ctx>: Into<&'a [T]>>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_value_slices().into()
    }
}

impl<K, V> Iterator for Drain<'_, K, V>
where
    V: SoaRead,
{
    type Item = (K, V);

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

impl<K, V> DoubleEndedIterator for Drain<'_, K, V>
where
    V: SoaRead,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(Into::into)
    }
}

impl<K, V> ExactSizeIterator for Drain<'_, K, V>
where
    V: SoaRead,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<K, V> FusedIterator for Drain<'_, K, V> where V: SoaRead {}
