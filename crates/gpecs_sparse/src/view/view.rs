#![expect(clippy::module_inception)]

use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::Index,
    ptr, slice,
};

use crate::{
    algo::{sparse_contains_key, sparse_get, sparse_get_epoch, sparse_get_with_key, sparse_index},
    error::FromPartsError,
    item::{DenseItem, DensePtrs, DenseSlicePtrs, SparseItem},
    iter::{Iter, Keys, RawIter, RawKeys, RawValues, Values},
    key::Key,
    soa::{
        slice::SoaSlices,
        traits::{Ptrs, RawSoa, SlicePtrs, Soa},
    },
    view::{EpochSparseViewPtr, assert::check_parts},
};

// TODO: add support for raw SoA types

pub struct EpochSparseView<'c, 'a, K, V>
where
    K: Key + 'c + 'a,
    V: RawSoa + ?Sized + 'c + 'a,
{
    dense: SoaSlices<'c, 'a, DenseItem<K, V>>,
    sparse: &'a [SparseItem<K>],
}

impl<'c, 'a, K, V> EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    pub unsafe fn from_parts(
        dense: SoaSlices<'c, 'a, DenseItem<K, V>>,
        sparse: &'a [SparseItem<K>],
    ) -> Self {
        Self { dense, sparse }
    }

    #[inline]
    pub fn into_parts(self) -> (SoaSlices<'c, 'a, DenseItem<K, V>>, &'a [SparseItem<K>]) {
        let Self { dense, sparse } = self;
        (dense, sparse)
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { dense, .. } = self;
        dense.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn sparse_len(&self) -> usize {
        let Self { sparse, .. } = self;
        sparse.len()
    }

    #[inline]
    pub fn sparse_is_empty(&self) -> bool {
        self.sparse_len() == 0
    }

    #[inline]
    pub fn as_view_ptr(&self) -> EpochSparseViewPtr<'_, K, V> {
        let Self { ref dense, sparse } = *self;

        let dense = dense.slice_ptrs();
        let sparse = ptr::from_ref(sparse);
        unsafe { EpochSparseViewPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn into_view_ptr(self) -> EpochSparseViewPtr<'c, K, V> {
        let Self { dense, sparse } = self;

        let dense = dense.into_slice_ptrs();
        let sparse = ptr::from_ref(sparse);
        unsafe { EpochSparseViewPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn view(&self) -> EpochSparseView<'_, '_, K, V> {
        unsafe { self.as_view_ptr().deref() }
    }

    #[inline]
    pub fn as_ptrs(&self) -> (DensePtrs<'_, K, V>, *const SparseItem<K>) {
        let (_, dense, sparse) = self.as_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&V::Context, DensePtrs<'_, K, V>, *const SparseItem<K>) {
        let Self { ref dense, sparse } = *self;

        let (context, dense) = dense.as_ptrs_with_context();
        let sparse = sparse.as_ptr();
        (context, dense, sparse)
    }

    #[inline]
    pub fn as_key_ptr(&self) -> *const K {
        let (_, key) = self.as_key_ptr_with_context();
        key
    }

    #[inline]
    pub fn as_key_ptr_with_context(&self) -> (&V::Context, *const K) {
        let (context, dense) = self.as_dense_ptrs_with_context();
        let (key, _) = dense.into_parts();
        (context, key)
    }

    #[inline]
    pub fn as_value_ptrs(&self) -> Ptrs<'_, V> {
        let (_, value) = self.as_value_ptrs_with_context();
        value
    }

    #[inline]
    pub fn as_value_ptrs_with_context(&self) -> (&V::Context, Ptrs<'_, V>) {
        let (context, dense) = self.as_dense_ptrs_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn as_dense_ptrs(&self) -> DensePtrs<'_, K, V> {
        let (_, dense) = self.as_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_dense_ptrs_with_context(&self) -> (&V::Context, DensePtrs<'_, K, V>) {
        let (context, dense, _) = self.as_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn as_sparse_ptr(&self) -> *const SparseItem<K> {
        let (_, sparse) = self.as_sparse_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn as_sparse_ptr_with_context(&self) -> (&V::Context, *const SparseItem<K>) {
        let (context, _, sparse) = self.as_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn into_ptrs(self) -> (DensePtrs<'c, K, V>, *const SparseItem<K>) {
        let (_, dense, sparse) = self.into_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_ptrs_with_context(
        self,
    ) -> (&'c V::Context, DensePtrs<'c, K, V>, *const SparseItem<K>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_ptrs_with_context();
        let sparse = sparse.as_ptr();
        (context, dense, sparse)
    }

    #[inline]
    pub fn into_key_ptr(self) -> *const K {
        let (_, key) = self.into_key_ptr_with_context();
        key
    }

    #[inline]
    pub fn into_key_ptr_with_context(self) -> (&'c V::Context, *const K) {
        let (context, dense) = self.into_dense_ptrs_with_context();
        let (key, _) = dense.into_parts();
        (context, key)
    }

    #[inline]
    pub fn into_value_ptrs(self) -> Ptrs<'c, V> {
        let (_, value) = self.into_value_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_value_ptrs_with_context(self) -> (&'c V::Context, Ptrs<'c, V>) {
        let (context, dense) = self.into_dense_ptrs_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn into_dense_ptrs(self) -> DensePtrs<'c, K, V> {
        let (_, dense) = self.into_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_ptrs_with_context(self) -> (&'c V::Context, DensePtrs<'c, K, V>) {
        let (context, dense, _) = self.into_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_sparse_ptr(self) -> *const SparseItem<K> {
        let (_, sparse) = self.into_sparse_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_ptr_with_context(self) -> (&'c V::Context, *const SparseItem<K>) {
        let (context, _, sparse) = self.into_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (DenseSlicePtrs<'_, K, V>, *const [SparseItem<K>]) {
        let (_, dense, sparse) = self.as_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(
        &self,
    ) -> (
        &V::Context,
        DenseSlicePtrs<'_, K, V>,
        *const [SparseItem<K>],
    ) {
        let Self { ref dense, sparse } = *self;
        let (context, dense) = dense.as_slice_ptrs_with_context();
        (context, dense, sparse)
    }

    #[inline]
    pub fn as_key_slice_ptr(&self) -> *const [K] {
        let (_, key) = self.as_key_slice_ptr_with_context();
        key
    }

    #[inline]
    pub fn as_key_slice_ptr_with_context(&self) -> (&V::Context, *const [K]) {
        let (context, dense) = self.as_dense_slice_ptrs_with_context();
        let (key, _) = dense.into_parts();
        (context, key)
    }

    #[inline]
    pub fn as_value_slice_ptrs(&self) -> SlicePtrs<'_, V> {
        let (_, value) = self.as_value_slice_ptrs_with_context();
        value
    }

    #[inline]
    pub fn as_value_slice_ptrs_with_context(&self) -> (&V::Context, SlicePtrs<'_, V>) {
        let (context, dense) = self.as_dense_slice_ptrs_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn as_dense_slice_ptrs(&self) -> DenseSlicePtrs<'_, K, V> {
        let (_, dense) = self.as_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_dense_slice_ptrs_with_context(&self) -> (&V::Context, DenseSlicePtrs<'_, K, V>) {
        let (context, dense, _) = self.as_slice_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn as_sparse_slice_ptr(&self) -> *const [SparseItem<K>] {
        let (_, sparse) = self.as_sparse_slice_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn as_sparse_slice_ptr_with_context(&self) -> (&V::Context, *const [SparseItem<K>]) {
        let (context, _, sparse) = self.as_slice_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> (DenseSlicePtrs<'c, K, V>, *const [SparseItem<K>]) {
        let (_, dense, sparse) = self.into_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(
        self,
    ) -> (
        &'c V::Context,
        DenseSlicePtrs<'c, K, V>,
        *const [SparseItem<K>],
    ) {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.into_slice_ptrs_with_context();
        (context, dense, sparse)
    }

    #[inline]
    pub fn into_key_slice_ptr(self) -> *const [K] {
        let (_, key) = self.into_key_slice_ptr_with_context();
        key
    }

    #[inline]
    pub fn into_key_slice_ptr_with_context(self) -> (&'c V::Context, *const [K]) {
        let (context, dense) = self.into_dense_slice_ptrs_with_context();
        let (key, _) = dense.into_parts();
        (context, key)
    }

    #[inline]
    pub fn into_value_slice_ptrs(self) -> SlicePtrs<'c, V> {
        let (_, value) = self.into_value_slice_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_value_slice_ptrs_with_context(self) -> (&'c V::Context, SlicePtrs<'c, V>) {
        let (context, dense) = self.into_dense_slice_ptrs_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn into_dense_slice_ptrs(self) -> DenseSlicePtrs<'c, K, V> {
        let (_, dense) = self.into_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_slice_ptrs_with_context(self) -> (&'c V::Context, DenseSlicePtrs<'c, K, V>) {
        let (context, dense, _) = self.into_slice_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_sparse_slice_ptr(self) -> *const [SparseItem<K>] {
        let (_, sparse) = self.into_sparse_slice_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_slice_ptr_with_context(self) -> (&'c V::Context, *const [SparseItem<K>]) {
        let (context, _, sparse) = self.into_slice_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn as_key_slice(&self) -> &[K] {
        let (_, keys) = self.as_key_slice_with_context();
        keys
    }

    #[inline]
    pub fn as_key_slice_with_context(&self) -> (&V::Context, &[K]) {
        let (context, keys) = self.as_key_slice_ptr_with_context();
        let keys = unsafe { slice::from_raw_parts(keys.cast(), keys.len()) };
        (context, keys)
    }

    #[inline]
    pub fn as_sparse_slice(&self) -> &[SparseItem<K>] {
        let (_, sparse) = self.as_sparse_slice_with_context();
        sparse
    }

    #[inline]
    pub fn as_sparse_slice_with_context(&self) -> (&V::Context, &[SparseItem<K>]) {
        let (context, sparse) = self.as_sparse_slice_ptr_with_context();
        let sparse = unsafe { slice::from_raw_parts(sparse.cast(), sparse.len()) };
        (context, sparse)
    }

    #[inline]
    pub fn into_key_slice(self) -> &'a [K] {
        let (_, keys) = self.into_key_slice_with_context();
        keys
    }

    #[inline]
    pub fn into_key_slice_with_context(self) -> (&'c V::Context, &'a [K]) {
        let (context, keys) = self.into_key_slice_ptr_with_context();
        let keys = unsafe { slice::from_raw_parts(keys.cast(), keys.len()) };
        (context, keys)
    }

    #[inline]
    pub fn into_sparse_slice(self) -> &'a [SparseItem<K>] {
        let (_, sparse) = self.into_sparse_slice_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_slice_with_context(self) -> (&'c V::Context, &'a [SparseItem<K>]) {
        let (context, sparse) = self.into_sparse_slice_ptr_with_context();
        let sparse = unsafe { slice::from_raw_parts(sparse.cast(), sparse.len()) };
        (context, sparse)
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, key: K) -> Ptrs<'_, V> {
        let (_, ptrs) = unsafe { self.get_unchecked_with_context(key) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_with_context(&self, key: K) -> (&V::Context, Ptrs<'_, V>) {
        let view_ptr = self.as_view_ptr();
        unsafe { view_ptr.into_get_unchecked_with_context(key) }
    }

    #[inline]
    pub unsafe fn into_get_unchecked(self, key: K) -> Ptrs<'c, V> {
        let (_, ptrs) = unsafe { self.into_get_unchecked_with_context(key) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_with_context(self, key: K) -> (&'c V::Context, Ptrs<'c, V>) {
        let view_ptr = self.into_view_ptr();
        unsafe { view_ptr.into_get_unchecked_with_context(key) }
    }

    #[inline]
    pub unsafe fn get_with_key_unchecked(
        &self,
        sparse_index: K::SparseIndex,
    ) -> (*const K, Ptrs<'_, V>) {
        let (_, key, value) = unsafe { self.get_with_key_unchecked_with_context(sparse_index) };
        (key, value)
    }

    #[inline]
    pub unsafe fn get_with_key_unchecked_with_context(
        &self,
        sparse_index: K::SparseIndex,
    ) -> (&V::Context, *const K, Ptrs<'_, V>) {
        let view_ptr = self.as_view_ptr();
        unsafe { view_ptr.into_get_with_key_unchecked_with_context(sparse_index) }
    }

    #[inline]
    pub unsafe fn into_get_with_key_unchecked(
        self,
        sparse_index: K::SparseIndex,
    ) -> (*const K, Ptrs<'c, V>) {
        let (_, key, value) =
            unsafe { self.into_get_with_key_unchecked_with_context(sparse_index) };
        (key, value)
    }

    #[inline]
    pub unsafe fn into_get_with_key_unchecked_with_context(
        self,
        sparse_index: K::SparseIndex,
    ) -> (&'c V::Context, *const K, Ptrs<'c, V>) {
        let view_ptr = self.into_view_ptr();
        unsafe { view_ptr.into_get_with_key_unchecked_with_context(sparse_index) }
    }

    #[inline]
    pub fn raw_keys(&self) -> RawKeys<'_, K, V> {
        let (_, iter) = self.raw_keys_with_context();
        iter
    }

    #[inline]
    pub fn raw_keys_with_context(&self) -> (&V::Context, RawKeys<'_, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.raw_iter_with_context();
        let iter = RawKeys::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_raw_keys(self) -> RawKeys<'c, K, V> {
        let (_, iter) = self.into_raw_keys_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_keys_with_context(self) -> (&'c V::Context, RawKeys<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_with_context();
        let iter = RawKeys::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn raw_values(&self) -> RawValues<'_, K, V> {
        let (_, iter) = self.raw_values_with_context();
        iter
    }

    #[inline]
    pub fn raw_values_with_context(&self) -> (&V::Context, RawValues<'_, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.raw_iter_with_context();
        let iter = RawValues::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_raw_values(self) -> RawValues<'c, K, V> {
        let (_, iter) = self.into_raw_values_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_values_with_context(self) -> (&'c V::Context, RawValues<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_with_context();
        let iter = RawValues::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn raw_iter(&self) -> RawIter<'_, K, V> {
        let (_, iter) = self.raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn raw_iter_with_context(&self) -> (&V::Context, RawIter<'_, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.raw_iter_with_context();
        let iter = RawIter::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_raw_iter(self) -> RawIter<'c, K, V> {
        let (_, iter) = self.into_raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_iter_with_context(self) -> (&'c V::Context, RawIter<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_with_context();
        let iter = RawIter::from_inner(inner);
        (context, iter)
    }
}

impl<'c, 'a, K, V> EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized, // TODO: replace with RawSoa to see which methods to move
{
    #[inline]
    pub fn new(
        dense: SoaSlices<'c, 'a, DenseItem<K, V>>,
        sparse: &'a [SparseItem<K>],
    ) -> Result<Self, FromPartsError<K>> {
        check_parts(&dense, sparse)?;

        let me = unsafe { Self::from_parts(dense, sparse) };
        Ok(me)
    }

    // TODO: rename slice methods according to pointer ones

    #[inline]
    pub fn as_slices(&self) -> V::Slices<'_, '_> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&V::Context, V::Slices<'_, '_>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.as_slices_with_context();
        let (_, values) = slices.into_parts();
        (context, values)
    }

    #[inline]
    pub fn into_slices(self) -> V::Slices<'c, 'a> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'c V::Context, V::Slices<'c, 'a>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.into_slices_with_context();
        let (_, values) = slices.into_parts();
        (context, values)
    }

    #[inline]
    pub fn get(&self, key: K) -> Option<V::Refs<'_, '_>> {
        let (_, refs) = self.get_with_context(key);
        refs
    }

    #[inline]
    pub fn get_with_context(&self, key: K) -> (&V::Context, Option<V::Refs<'_, '_>>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.iter_with_context();
        let refs = sparse_get(dense.map(From::from), sparse, key);
        (context, refs)
    }

    #[inline]
    pub fn into_get(self, key: K) -> Option<V::Refs<'c, 'a>> {
        let (_, refs) = self.into_get_with_context(key);
        refs
    }

    #[inline]
    pub fn into_get_with_context(self, key: K) -> (&'c V::Context, Option<V::Refs<'c, 'a>>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let refs = sparse_get(dense.map(From::from), sparse, key);
        (context, refs)
    }

    #[inline]
    #[track_caller]
    pub fn index(&self, key: K) -> V::Refs<'_, '_>
    where
        K: Debug,
    {
        let (_, refs) = self.index_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_with_context(&self, key: K) -> (&V::Context, V::Refs<'_, '_>)
    where
        K: Debug,
    {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.iter_with_context();
        let refs = sparse_index(dense.map(From::from), sparse, key);
        (context, refs)
    }

    #[inline]
    #[track_caller]
    pub fn into_index(self, key: K) -> V::Refs<'c, 'a>
    where
        K: Debug,
    {
        let (_, refs) = self.into_index_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_with_context(self, key: K) -> (&'c V::Context, V::Refs<'c, 'a>)
    where
        K: Debug,
    {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let refs = sparse_index(dense.map(From::from), sparse, key);
        (context, refs)
    }

    #[inline]
    pub fn get_with_key(&self, sparse_index: K::SparseIndex) -> Option<(K, V::Refs<'_, '_>)> {
        let (_, pair) = self.get_with_key_with_context(sparse_index);
        pair
    }

    #[inline]
    pub fn get_with_key_with_context(
        &self,
        sparse_index: K::SparseIndex,
    ) -> (&V::Context, Option<(K, V::Refs<'_, '_>)>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.iter_with_context();
        let pair = sparse_get_with_key(dense.map(From::from), sparse, sparse_index);
        (context, pair)
    }

    #[inline]
    pub fn into_get_with_key(self, sparse_index: K::SparseIndex) -> Option<(K, V::Refs<'c, 'a>)> {
        let (_, pair) = self.into_get_with_key_with_context(sparse_index);
        pair
    }

    #[inline]
    pub fn into_get_with_key_with_context(
        self,
        sparse_index: K::SparseIndex,
    ) -> (&'c V::Context, Option<(K, V::Refs<'c, 'a>)>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let pair = sparse_get_with_key(dense.map(From::from), sparse, sparse_index);
        (context, pair)
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: K::SparseIndex) -> Option<K::Epoch> {
        let Self { dense, sparse } = self;

        let (keys, _) = dense.as_slices().into_parts();
        sparse_get_epoch(keys, sparse, sparse_index)
    }

    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let Self { dense, sparse } = self;

        let (keys, _) = dense.as_slices().into_parts();
        sparse_contains_key(keys, sparse, key)
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_, '_, K, V> {
        let (_, iter) = self.keys_with_context();
        iter
    }

    #[inline]
    pub fn keys_with_context(&self) -> (&V::Context, Keys<'_, '_, K, V>) {
        let (context, iter) = self.raw_keys_with_context();
        let iter = unsafe { iter.deref() };
        (context, iter)
    }

    #[inline]
    pub fn into_keys(self) -> Keys<'c, 'a, K, V> {
        let (_, iter) = self.into_keys_with_context();
        iter
    }

    #[inline]
    pub fn into_keys_with_context(self) -> (&'c V::Context, Keys<'c, 'a, K, V>) {
        let (context, iter) = self.into_raw_keys_with_context();
        let iter = unsafe { iter.deref() };
        (context, iter)
    }

    #[inline]
    pub fn values(&self) -> Values<'_, '_, K, V> {
        let (_, iter) = self.values_with_context();
        iter
    }

    #[inline]
    pub fn values_with_context(&self) -> (&V::Context, Values<'_, '_, K, V>) {
        let (context, iter) = self.raw_values_with_context();
        let iter = unsafe { iter.deref() };
        (context, iter)
    }

    #[inline]
    pub fn into_values(self) -> Values<'c, 'a, K, V> {
        let (_, iter) = self.into_values_with_context();
        iter
    }

    #[inline]
    pub fn into_values_with_context(self) -> (&'c V::Context, Values<'c, 'a, K, V>) {
        let (context, iter) = self.into_raw_values_with_context();
        let iter = unsafe { iter.deref() };
        (context, iter)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, '_, K, V> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&self) -> (&V::Context, Iter<'_, '_, K, V>) {
        let (context, iter) = self.raw_iter_with_context();
        let iter = unsafe { iter.deref() };
        (context, iter)
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'c V::Context, Iter<'c, 'a, K, V>) {
        let (context, iter) = self.into_raw_iter_with_context();
        let iter = unsafe { iter.deref() };
        (context, iter)
    }
}

impl<'c, 'a, K, V> Debug for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SparseItem<K>: Debug,
    SoaSlices<'c, 'a, DenseItem<K, V>>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { dense, sparse } = self;
        f.debug_struct("EpochSparseView")
            .field("dense", dense)
            .field("sparse", sparse)
            .finish()
    }
}

impl<'c, K, V> From<&'c V::Context> for EpochSparseView<'c, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn from(context: &'c V::Context) -> Self {
        let view_ptr = EpochSparseViewPtr::from(context);
        unsafe { view_ptr.deref() }
    }
}

impl<K, V> Clone for EpochSparseView<'_, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { dense, sparse } = self;
        let dense = dense.clone();
        Self { dense, sparse }
    }
}

impl<'c, 'a, K, V> Copy for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlices<'c, 'a, DenseItem<K, V>>: Copy,
{
}

impl<'c, 'a, K, V> PartialEq for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlices<'c, 'a, DenseItem<K, V>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { dense, sparse } = self;
        *dense == other.dense && *sparse == other.sparse
    }
}

impl<'c, 'a, K, V> Eq for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlices<'c, 'a, DenseItem<K, V>>: Eq,
{
}

impl<'c, 'a, K, V> PartialOrd for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlices<'c, 'a, DenseItem<K, V>>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { dense, sparse } = self;
        match dense.partial_cmp(&other.dense) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        sparse.partial_cmp(&other.sparse)
    }
}

impl<'c, 'a, K, V> Ord for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlices<'c, 'a, DenseItem<K, V>>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { dense, sparse } = self;
        match dense.cmp(&other.dense) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        sparse.cmp(&other.sparse)
    }
}

impl<'c, 'a, K, V> Hash for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SparseItem<K>: Hash,
    SoaSlices<'c, 'a, DenseItem<K, V>>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { dense, sparse } = self;
        dense.hash(state);
        sparse.hash(state);
    }
}

impl<T, K, V> Index<K> for EpochSparseView<'_, '_, K, V>
where
    K: Key + Debug,
    V: Soa + ?Sized,
    for<'c, 'any> V: Soa<Refs<'c, 'any> = &'any T> + 'any,
{
    type Output = T;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        Self::index(self, key)
    }
}

impl<T, K, V> AsRef<[T]> for EpochSparseView<'_, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    for<'c, 'any> V::Slices<'c, 'any>: Into<&'any [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slices().into()
    }
}

impl<K, V> AsRef<Self> for EpochSparseView<'_, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<'r, K, V> IntoIterator for &'r EpochSparseView<'_, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    type Item = (&'r K, V::Refs<'r, 'r>);
    type IntoIter = Iter<'r, 'r, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'c, 'a, K, V> IntoIterator for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    type Item = (&'a K, V::Refs<'c, 'a>);
    type IntoIter = Iter<'c, 'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}
