#![expect(clippy::module_inception)]

use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::Index,
    ptr,
};

use gpecs_ptr::slice::{CoreSliceItemPtrs, SliceItemPtrs};

use crate::{
    algo::{
        check_parts, sparse_contains_key, sparse_get, sparse_get_epoch, sparse_get_with_key,
        sparse_index,
    },
    error::FromPartsError,
    item::{
        DefaultSparseItem, KeyValuePair, KeyValuePtrs, KeyValueSlicePtrs, KeyValueSlices,
        SparseItem,
    },
    iter::{Iter, Keys, RawIter, RawKeys, RawValues, Values},
    key::Key,
    soa::{
        slice::SoaSlices,
        traits::{Ptrs, RawSoa, Refs, SlicePtrs, Slices, Soa, SoaContext, SoaOwned},
    },
    view::EpochSparseViewPtr,
};

pub type SparseView<'ctx, 'a, T, S = DefaultSparseItem<usize>, P = CoreSliceItemPtrs<usize>> =
    EpochSparseView<'ctx, 'a, usize, T, S, P>;

pub struct EpochSparseView<'ctx, 'a, K, V, S = DefaultSparseItem<K>, P = CoreSliceItemPtrs<K>>
where
    K: Key + 'a,
    V: RawSoa<Context: 'ctx> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    dense: SoaSlices<'ctx, 'a, KeyValuePair<K, V, P>>,
    sparse: &'a [S],
}

impl<'ctx, 'a, K, V, S, P> EpochSparseView<'ctx, 'a, K, V, S, P>
where
    K: Key,
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    pub fn new(
        dense: SoaSlices<'ctx, 'a, KeyValuePair<K, V, P>>,
        sparse: &'a [S],
    ) -> Result<Self, FromPartsError<K>> {
        check_parts(dense.slices(), sparse)?;

        let me = unsafe { Self::from_parts(dense, sparse) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(
        dense: SoaSlices<'ctx, 'a, KeyValuePair<K, V, P>>,
        sparse: &'a [S],
    ) -> Self {
        Self { dense, sparse }
    }

    #[inline]
    pub fn into_parts(self) -> (SoaSlices<'ctx, 'a, KeyValuePair<K, V, P>>, &'a [S]) {
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
    pub fn as_view_ptr(&self) -> EpochSparseViewPtr<'_, K, V, S, P> {
        let Self { dense, sparse } = self;

        let dense = dense.slice_ptrs();
        let sparse = ptr::from_ref(*sparse);
        unsafe { EpochSparseViewPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn into_view_ptr(self) -> EpochSparseViewPtr<'ctx, K, V, S, P> {
        let Self { dense, sparse } = self;

        let dense = dense.into_slice_ptrs();
        let sparse = ptr::from_ref(sparse);
        unsafe { EpochSparseViewPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn as_view(&self) -> EpochSparseView<'_, '_, K, V, S, P> {
        unsafe { self.as_view_ptr().as_ref_unchecked() }
    }

    #[inline]
    pub fn as_ptrs(&self) -> (KeyValuePtrs<'_, K, V, P::Const>, *const S) {
        let (_, dense, sparse) = self.as_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_ptrs_with_context(
        &self,
    ) -> (&V::Context, KeyValuePtrs<'_, K, V, P::Const>, *const S) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.as_ptrs_with_context();
        let sparse = sparse.as_ptr();
        (context, dense, sparse)
    }

    #[inline]
    pub fn as_key_ptr(&self) -> P::Const {
        let (_, key) = self.as_key_ptr_with_context();
        key
    }

    #[inline]
    pub fn as_key_ptr_with_context(&self) -> (&V::Context, P::Const) {
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
    pub fn as_dense_ptrs(&self) -> KeyValuePtrs<'_, K, V, P::Const> {
        let (_, dense) = self.as_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_dense_ptrs_with_context(&self) -> (&V::Context, KeyValuePtrs<'_, K, V, P::Const>) {
        let (context, dense, _) = self.as_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn as_sparse_ptr(&self) -> *const S {
        let (_, sparse) = self.as_sparse_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn as_sparse_ptr_with_context(&self) -> (&V::Context, *const S) {
        let (context, _, sparse) = self.as_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn into_ptrs(self) -> (KeyValuePtrs<'ctx, K, V, P::Const>, *const S) {
        let (_, dense, sparse) = self.into_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_ptrs_with_context(
        self,
    ) -> (
        &'ctx V::Context,
        KeyValuePtrs<'ctx, K, V, P::Const>,
        *const S,
    ) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_ptrs_with_context();
        let sparse = sparse.as_ptr();
        (context, dense, sparse)
    }

    #[inline]
    pub fn into_key_ptr(self) -> P::Const {
        let (_, key) = self.into_key_ptr_with_context();
        key
    }

    #[inline]
    pub fn into_key_ptr_with_context(self) -> (&'ctx V::Context, P::Const) {
        let (context, dense) = self.into_dense_ptrs_with_context();
        let (key, _) = dense.into_parts();
        (context, key)
    }

    #[inline]
    pub fn into_value_ptrs(self) -> Ptrs<'ctx, V> {
        let (_, value) = self.into_value_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_value_ptrs_with_context(self) -> (&'ctx V::Context, Ptrs<'ctx, V>) {
        let (context, dense) = self.into_dense_ptrs_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn into_dense_ptrs(self) -> KeyValuePtrs<'ctx, K, V, P::Const> {
        let (_, dense) = self.into_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, KeyValuePtrs<'ctx, K, V, P::Const>) {
        let (context, dense, _) = self.into_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_sparse_ptr(self) -> *const S {
        let (_, sparse) = self.into_sparse_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_ptr_with_context(self) -> (&'ctx V::Context, *const S) {
        let (context, _, sparse) = self.into_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (KeyValueSlicePtrs<'_, K, V, P::Const>, *const [S]) {
        let (_, dense, sparse) = self.as_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn as_slice_ptrs_with_context(
        &self,
    ) -> (
        &V::Context,
        KeyValueSlicePtrs<'_, K, V, P::Const>,
        *const [S],
    ) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.as_slice_ptrs_with_context();
        let sparse = ptr::from_ref(*sparse);
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
    pub fn as_dense_slice_ptrs(&self) -> KeyValueSlicePtrs<'_, K, V, P::Const> {
        let (_, dense) = self.as_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_dense_slice_ptrs_with_context(
        &self,
    ) -> (&V::Context, KeyValueSlicePtrs<'_, K, V, P::Const>) {
        let (context, dense, _) = self.as_slice_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn as_sparse_slice_ptr(&self) -> *const [S] {
        let (_, sparse) = self.as_sparse_slice_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn as_sparse_slice_ptr_with_context(&self) -> (&V::Context, *const [S]) {
        let (context, _, sparse) = self.as_slice_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> (KeyValueSlicePtrs<'ctx, K, V, P::Const>, *const [S]) {
        let (_, dense, sparse) = self.into_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn into_slice_ptrs_with_context(
        self,
    ) -> (
        &'ctx V::Context,
        KeyValueSlicePtrs<'ctx, K, V, P::Const>,
        *const [S],
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
    pub fn into_key_slice_ptr_with_context(self) -> (&'ctx V::Context, *const [K]) {
        let (context, dense) = self.into_dense_slice_ptrs_with_context();
        let (key, _) = dense.into_parts();
        (context, key)
    }

    #[inline]
    pub fn into_value_slice_ptrs(self) -> SlicePtrs<'ctx, V> {
        let (_, value) = self.into_value_slice_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_value_slice_ptrs_with_context(self) -> (&'ctx V::Context, SlicePtrs<'ctx, V>) {
        let (context, dense) = self.into_dense_slice_ptrs_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn into_dense_slice_ptrs(self) -> KeyValueSlicePtrs<'ctx, K, V, P::Const> {
        let (_, dense) = self.into_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_slice_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, KeyValueSlicePtrs<'ctx, K, V, P::Const>) {
        let (context, dense, _) = self.into_slice_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_sparse_slice_ptr(self) -> *const [S] {
        let (_, sparse) = self.into_sparse_slice_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_slice_ptr_with_context(self) -> (&'ctx V::Context, *const [S]) {
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
        let keys = unsafe { keys.as_ref_unchecked() };
        (context, keys)
    }

    #[inline]
    pub fn as_sparse_slice(&self) -> &[S] {
        let (_, sparse) = self.as_sparse_slice_with_context();
        sparse
    }

    #[inline]
    pub fn as_sparse_slice_with_context(&self) -> (&V::Context, &[S]) {
        let (context, sparse) = self.as_sparse_slice_ptr_with_context();
        let sparse = unsafe { sparse.as_ref_unchecked() };
        (context, sparse)
    }

    #[inline]
    pub fn into_key_slice(self) -> &'a [K] {
        let (_, keys) = self.into_key_slice_with_context();
        keys
    }

    #[inline]
    pub fn into_key_slice_with_context(self) -> (&'ctx V::Context, &'a [K]) {
        let (context, keys) = self.into_key_slice_ptr_with_context();
        let keys = unsafe { keys.as_ref_unchecked() };
        (context, keys)
    }

    #[inline]
    pub fn into_sparse_slice(self) -> &'a [S] {
        let (_, sparse) = self.into_sparse_slice_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_slice_with_context(self) -> (&'ctx V::Context, &'a [S]) {
        let (context, sparse) = self.into_sparse_slice_ptr_with_context();
        let sparse = unsafe { sparse.as_ref_unchecked() };
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
    pub unsafe fn into_get_unchecked(self, key: K) -> Ptrs<'ctx, V> {
        let (_, ptrs) = unsafe { self.into_get_unchecked_with_context(key) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_with_context(
        self,
        key: K,
    ) -> (&'ctx V::Context, Ptrs<'ctx, V>) {
        let view_ptr = self.into_view_ptr();
        unsafe { view_ptr.into_get_unchecked_with_context(key) }
    }

    #[inline]
    pub unsafe fn get_with_key_unchecked(
        &self,
        sparse_index: K::SparseIndex,
    ) -> (P::Const, Ptrs<'_, V>) {
        let (_, key, value) = unsafe { self.get_with_key_unchecked_with_context(sparse_index) };
        (key, value)
    }

    #[inline]
    pub unsafe fn get_with_key_unchecked_with_context(
        &self,
        sparse_index: K::SparseIndex,
    ) -> (&V::Context, P::Const, Ptrs<'_, V>) {
        let view_ptr = self.as_view_ptr();
        unsafe { view_ptr.into_get_with_key_unchecked_with_context(sparse_index) }
    }

    #[inline]
    pub unsafe fn into_get_with_key_unchecked(
        self,
        sparse_index: K::SparseIndex,
    ) -> (P::Const, Ptrs<'ctx, V>) {
        let (_, key, value) =
            unsafe { self.into_get_with_key_unchecked_with_context(sparse_index) };
        (key, value)
    }

    #[inline]
    pub unsafe fn into_get_with_key_unchecked_with_context(
        self,
        sparse_index: K::SparseIndex,
    ) -> (&'ctx V::Context, P::Const, Ptrs<'ctx, V>) {
        let view_ptr = self.into_view_ptr();
        unsafe { view_ptr.into_get_with_key_unchecked_with_context(sparse_index) }
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: K::SparseIndex) -> Option<K::Epoch> {
        let dense_keys = self.as_key_slice();
        let sparse = self.as_sparse_slice();
        sparse_get_epoch(dense_keys, sparse, sparse_index)
    }

    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let dense_keys = self.as_key_slice();
        let sparse = self.as_sparse_slice();
        sparse_contains_key(dense_keys, sparse, key)
    }

    #[inline]
    pub fn raw_keys(&self) -> RawKeys<'_, K, V> {
        let (_, iter) = self.raw_keys_with_context();
        iter
    }

    #[inline]
    pub fn raw_keys_with_context(&self) -> (&V::Context, RawKeys<'_, K, V>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.as_slice_ptrs_with_context();
        let (keys, _) = slices.into();
        let iter = RawKeys::new(context.as_inner(), keys);
        (context, iter)
    }

    #[inline]
    pub fn into_raw_keys(self) -> RawKeys<'ctx, K, V> {
        let (_, iter) = self.into_raw_keys_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_keys_with_context(self) -> (&'ctx V::Context, RawKeys<'ctx, K, V>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.into_slice_ptrs_with_context();
        let (keys, _) = slices.into();
        let iter = RawKeys::new(context.as_inner(), keys);
        (context, iter)
    }

    #[inline]
    pub fn raw_values(&self) -> RawValues<'_, K, V, P> {
        let (_, iter) = self.raw_values_with_context();
        iter
    }

    #[inline]
    pub fn raw_values_with_context(&self) -> (&V::Context, RawValues<'_, K, V, P>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.raw_iter_with_context();
        let iter = RawValues::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_raw_values(self) -> RawValues<'ctx, K, V, P> {
        let (_, iter) = self.into_raw_values_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_values_with_context(self) -> (&'ctx V::Context, RawValues<'ctx, K, V, P>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_with_context();
        let iter = RawValues::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn raw_iter(&self) -> RawIter<'_, K, V, P> {
        let (_, iter) = self.raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn raw_iter_with_context(&self) -> (&V::Context, RawIter<'_, K, V, P>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.raw_iter_with_context();
        let iter = RawIter::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_raw_iter(self) -> RawIter<'ctx, K, V, P> {
        let (_, iter) = self.into_raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_iter_with_context(self) -> (&'ctx V::Context, RawIter<'ctx, K, V, P>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_with_context();
        let iter = RawIter::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_, '_, K, V> {
        let (_, iter) = self.keys_with_context();
        iter
    }

    #[inline]
    pub fn keys_with_context(&self) -> (&V::Context, Keys<'_, '_, K, V>) {
        let (context, iter) = self.raw_keys_with_context();
        let iter = unsafe { iter.as_ref_unchecked() };
        (context, iter)
    }

    #[inline]
    pub fn into_keys(self) -> Keys<'ctx, 'a, K, V> {
        let (_, iter) = self.into_keys_with_context();
        iter
    }

    #[inline]
    pub fn into_keys_with_context(self) -> (&'ctx V::Context, Keys<'ctx, 'a, K, V>) {
        let (context, iter) = self.into_raw_keys_with_context();
        let iter = unsafe { iter.as_ref_unchecked() };
        (context, iter)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_keys(&self) -> crate::iter::ParKeys<'_, '_, K, V> {
        let (_, keys) = self.par_keys_with_context();
        keys
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_keys_with_context(&self) -> (&V::Context, crate::iter::ParKeys<'_, '_, K, V>) {
        let (context, keys) = self.as_key_slice_with_context();
        let keys = crate::iter::ParKeys::new(context, keys);
        (context, keys)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn into_par_keys(self) -> crate::iter::ParKeys<'ctx, 'a, K, V> {
        let (_, keys) = self.into_par_keys_with_context();
        keys
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn into_par_keys_with_context(
        self,
    ) -> (&'ctx V::Context, crate::iter::ParKeys<'ctx, 'a, K, V>) {
        let (context, keys) = self.into_key_slice_with_context();
        let keys = crate::iter::ParKeys::new(context, keys);
        (context, keys)
    }
}

impl<'ctx, 'a, K, V, S, P> EpochSparseView<'ctx, 'a, K, V, S, P>
where
    K: Key,
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    pub fn into_slices(self) -> (KeyValueSlices<'ctx, 'a, K, V, P::Const>, &'a [S]) {
        let (_, dense, sparse) = self.into_slices_with_context();
        (dense, sparse)
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn into_slices_with_context(
        self,
    ) -> (
        &'ctx V::Context,
        KeyValueSlices<'ctx, 'a, K, V, P::Const>,
        &'a [S],
    ) {
        let (context, dense, sparse) = self.into_slice_ptrs_with_context();
        let dense = unsafe { dense.as_ref_unchecked(context) };
        let sparse = unsafe { sparse.as_ref_unchecked() };
        (context, dense, sparse)
    }

    #[inline]
    pub fn into_value_slices(self) -> Slices<'ctx, 'a, V> {
        let (_, value) = self.into_value_slices_with_context();
        value
    }

    #[inline]
    pub fn into_value_slices_with_context(self) -> (&'ctx V::Context, Slices<'ctx, 'a, V>) {
        let (context, dense) = self.into_dense_slices_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn into_dense_slices(self) -> KeyValueSlices<'ctx, 'a, K, V, P::Const> {
        let (_, dense) = self.into_dense_slices_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_slices_with_context(
        self,
    ) -> (&'ctx V::Context, KeyValueSlices<'ctx, 'a, K, V, P::Const>) {
        let (context, dense, _) = self.into_slices_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_get(self, key: K) -> Option<Refs<'ctx, 'a, V>> {
        let (_, refs) = self.into_get_with_context(key);
        refs
    }

    #[inline]
    pub fn into_get_with_context(self, key: K) -> (&'ctx V::Context, Option<Refs<'ctx, 'a, V>>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let refs = sparse_get(dense.map(From::from), sparse, key);
        (context, refs)
    }

    #[inline]
    #[track_caller]
    pub fn into_index(self, key: K) -> Refs<'ctx, 'a, V>
    where
        K: Debug,
    {
        let (_, refs) = self.into_index_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_with_context(self, key: K) -> (&'ctx V::Context, Refs<'ctx, 'a, V>)
    where
        K: Debug,
    {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let refs = sparse_index(dense.map(From::from), sparse, key);
        (context, refs)
    }

    #[inline]
    pub fn into_get_with_key(self, sparse_index: K::SparseIndex) -> Option<(K, Refs<'ctx, 'a, V>)> {
        let (_, pair) = self.into_get_with_key_with_context(sparse_index);
        pair
    }

    #[inline]
    pub fn into_get_with_key_with_context(
        self,
        sparse_index: K::SparseIndex,
    ) -> (&'ctx V::Context, Option<(K, Refs<'ctx, 'a, V>)>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let pair = sparse_get_with_key(dense.map(From::from), sparse, sparse_index);
        (context, pair)
    }

    #[inline]
    pub fn into_values(self) -> Values<'ctx, 'a, K, V, P> {
        let (_, iter) = self.into_values_with_context();
        iter
    }

    #[inline]
    pub fn into_values_with_context(self) -> (&'ctx V::Context, Values<'ctx, 'a, K, V, P>) {
        let (context, iter) = self.into_raw_values_with_context();
        let iter = unsafe { iter.as_ref_unchecked() };
        (context, iter)
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'ctx V::Context, Iter<'ctx, 'a, K, V, P>) {
        let (context, iter) = self.into_raw_iter_with_context();
        let iter = unsafe { iter.as_ref_unchecked() };
        (context, iter)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn into_par_iter(self) -> crate::iter::ParIter<'ctx, 'a, K, V, P> {
        let (_, iter) = self.into_par_iter_with_context();
        iter
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn into_par_iter_with_context(
        self,
    ) -> (&'ctx V::Context, crate::iter::ParIter<'ctx, 'a, K, V, P>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_par_iter_with_context();
        let iter = crate::iter::ParIter::new(inner);
        (context, iter)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn into_par_values(self) -> crate::iter::ParValues<'ctx, 'a, K, V, P> {
        let (_, iter) = self.into_par_values_with_context();
        iter
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn into_par_values_with_context(
        self,
    ) -> (&'ctx V::Context, crate::iter::ParValues<'ctx, 'a, K, V, P>) {
        let (context, inner) = self.into_par_iter_with_context();
        let values = crate::iter::ParValues::new(inner);
        (context, values)
    }
}

impl<'a, K, V, S, P> EpochSparseView<'_, '_, K, V, S, P>
where
    K: Key,
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    pub fn as_slices(&'a self) -> (KeyValueSlices<'a, 'a, K, V, P::Const>, &'a [S]) {
        let (_, dense, sparse) = self.as_slices_with_context();
        (dense, sparse)
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn as_slices_with_context(
        &'a self,
    ) -> (
        &'a V::Context,
        KeyValueSlices<'a, 'a, K, V, P::Const>,
        &'a [S],
    ) {
        let (context, dense, sparse) = self.as_slice_ptrs_with_context();
        let dense = unsafe { dense.as_ref_unchecked(context) };
        let sparse = unsafe { sparse.as_ref_unchecked() };
        (context, dense, sparse)
    }

    #[inline]
    pub fn as_value_slices(&'a self) -> Slices<'a, 'a, V> {
        let (_, value) = self.as_value_slices_with_context();
        value
    }

    #[inline]
    pub fn as_value_slices_with_context(&'a self) -> (&'a V::Context, Slices<'a, 'a, V>) {
        let (context, dense) = self.as_dense_slices_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn as_dense_slices(&'a self) -> KeyValueSlices<'a, 'a, K, V, P::Const> {
        let (_, dense) = self.as_dense_slices_with_context();
        dense
    }

    #[inline]
    pub fn as_dense_slices_with_context(
        &'a self,
    ) -> (&'a V::Context, KeyValueSlices<'a, 'a, K, V, P::Const>) {
        let (context, dense, _) = self.as_slices_with_context();
        (context, dense)
    }

    #[inline]
    pub fn get(&'a self, key: K) -> Option<Refs<'a, 'a, V>> {
        let (_, refs) = self.get_with_context(key);
        refs
    }

    #[inline]
    pub fn get_with_context(&'a self, key: K) -> (&'a V::Context, Option<Refs<'a, 'a, V>>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.iter_with_context();
        let refs = sparse_get(dense.map(From::from), sparse, key);
        (context, refs)
    }

    #[inline]
    #[track_caller]
    pub fn index(&'a self, key: K) -> Refs<'a, 'a, V>
    where
        K: Debug,
    {
        let (_, refs) = self.index_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_with_context(&'a self, key: K) -> (&'a V::Context, Refs<'a, 'a, V>)
    where
        K: Debug,
    {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.iter_with_context();
        let refs = sparse_index(dense.map(From::from), sparse, key);
        (context, refs)
    }

    #[inline]
    pub fn get_with_key(&'a self, sparse_index: K::SparseIndex) -> Option<(K, Refs<'a, 'a, V>)> {
        let (_, pair) = self.get_with_key_with_context(sparse_index);
        pair
    }

    #[inline]
    pub fn get_with_key_with_context(
        &'a self,
        sparse_index: K::SparseIndex,
    ) -> (&'a V::Context, Option<(K, Refs<'a, 'a, V>)>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.iter_with_context();
        let pair = sparse_get_with_key(dense.map(From::from), sparse, sparse_index);
        (context, pair)
    }

    #[inline]
    pub fn values(&'a self) -> Values<'a, 'a, K, V, P> {
        let (_, iter) = self.values_with_context();
        iter
    }

    #[inline]
    pub fn values_with_context(&'a self) -> (&'a V::Context, Values<'a, 'a, K, V, P>) {
        let (context, iter) = self.raw_values_with_context();
        let iter = unsafe { iter.as_ref_unchecked() };
        (context, iter)
    }

    #[inline]
    pub fn iter(&'a self) -> Iter<'a, 'a, K, V, P> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&'a self) -> (&'a V::Context, Iter<'a, 'a, K, V, P>) {
        let (context, iter) = self.raw_iter_with_context();
        let iter = unsafe { iter.as_ref_unchecked() };
        (context, iter)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter(&'a self) -> crate::iter::ParIter<'a, 'a, K, V, P> {
        let (_, iter) = self.par_iter_with_context();
        iter
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter_with_context(
        &'a self,
    ) -> (&'a V::Context, crate::iter::ParIter<'a, 'a, K, V, P>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.slices_with_context();
        let iter = crate::iter::ParIter::new(slices.into_par_iter());
        (context, iter)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_values(&'a self) -> crate::iter::ParValues<'a, 'a, K, V, P> {
        let (_, iter) = self.par_values_with_context();
        iter
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_values_with_context(
        &'a self,
    ) -> (&'a V::Context, crate::iter::ParValues<'a, 'a, K, V, P>) {
        let (context, inner) = self.par_iter_with_context();
        let values = crate::iter::ParValues::new(inner);
        (context, values)
    }
}

impl<'ctx, 'a, K, V, S, P> Debug for EpochSparseView<'ctx, 'a, K, V, S, P>
where
    K: Key,
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Debug,
    SoaSlices<'ctx, 'a, KeyValuePair<K, V, P>>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { dense, sparse } = self;

        f.debug_struct("EpochSparseView")
            .field("dense", dense)
            .field("sparse", sparse)
            .finish()
    }
}

impl<'ctx, K, V, S, P> From<&'ctx V::Context> for EpochSparseView<'ctx, '_, K, V, S, P>
where
    K: Key,
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn from(context: &'ctx V::Context) -> Self {
        let view_ptr = EpochSparseViewPtr::from(context);
        unsafe { view_ptr.as_ref_unchecked() }
    }
}

impl<'ctx, K, V, S, P> Default for EpochSparseView<'ctx, '_, K, V, S, P>
where
    K: Key,
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    &'ctx V::Context: Default,
{
    #[inline]
    fn default() -> Self {
        let context: &V::Context = Default::default();
        Self::from(context)
    }
}

impl<K, V, S, P> Clone for EpochSparseView<'_, '_, K, V, S, P>
where
    K: Key,
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { dense, sparse } = self;

        let dense = dense.clone();
        Self { dense, sparse }
    }
}

impl<'ctx, 'a, K, V, S, P> Copy for EpochSparseView<'ctx, 'a, K, V, S, P>
where
    K: Key,
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    SoaSlices<'ctx, 'a, KeyValuePair<K, V, P>>: Copy,
{
}

impl<'ctx, 'a, K, V, S, P> PartialEq for EpochSparseView<'ctx, 'a, K, V, S, P>
where
    K: Key,
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + PartialEq,
    SoaSlices<'ctx, 'a, KeyValuePair<K, V, P>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse) == other
    }
}

impl<'ctx, 'a, K, V, S, P> Eq for EpochSparseView<'ctx, 'a, K, V, S, P>
where
    K: Key,
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Eq,
    SoaSlices<'ctx, 'a, KeyValuePair<K, V, P>>: Eq,
{
}

impl<'ctx, 'a, K, V, S, P> PartialOrd for EpochSparseView<'ctx, 'a, K, V, S, P>
where
    K: Key,
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + PartialOrd,
    SoaSlices<'ctx, 'a, KeyValuePair<K, V, P>>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse).partial_cmp(&other)
    }
}

impl<'ctx, 'a, K, V, S, P> Ord for EpochSparseView<'ctx, 'a, K, V, S, P>
where
    K: Key,
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Ord,
    SoaSlices<'ctx, 'a, KeyValuePair<K, V, P>>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse).cmp(&other)
    }
}

impl<'ctx, 'a, K, V, S, P> Hash for EpochSparseView<'ctx, 'a, K, V, S, P>
where
    K: Key,
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Hash,
    SoaSlices<'ctx, 'a, KeyValuePair<K, V, P>>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { dense, sparse } = self;
        (dense, sparse).hash(state);
    }
}

impl<T, K, V, S, P> Index<K> for EpochSparseView<'_, '_, K, V, S, P>
where
    K: Key + Debug,
    V: SoaOwned + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx, 'a> V::Context: SoaContext<'a, V, Refs<'ctx> = &'a T>,
{
    type Output = T;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        Self::index(self, key)
    }
}

impl<T, K, V, S, P> AsRef<[T]> for EpochSparseView<'_, '_, K, V, S, P>
where
    K: Key,
    V: SoaOwned + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Into<&'a [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_value_slices().into()
    }
}

impl<K, V, S, P> AsRef<Self> for EpochSparseView<'_, '_, K, V, S, P>
where
    K: Key,
    V: RawSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<'a, K, V, S, P> IntoIterator for &'a EpochSparseView<'_, '_, K, V, S, P>
where
    K: Key,
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (&'a K, Refs<'a, 'a, V>);
    type IntoIter = Iter<'a, 'a, K, V, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'ctx, 'a, K, V, S, P> IntoIterator for EpochSparseView<'ctx, 'a, K, V, S, P>
where
    K: Key,
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (&'a K, Refs<'ctx, 'a, V>);
    type IntoIter = Iter<'ctx, 'a, K, V, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}

#[cfg(feature = "rayon")]
impl<'a, K, V, S, P> rayon::iter::IntoParallelIterator for &'a EpochSparseView<'_, '_, K, V, S, P>
where
    K: Key + Sync,
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Sync,
    V::Fields: Sync,
    Refs<'a, 'a, V>: Send,
{
    type Item = (&'a K, Refs<'a, 'a, V>);
    type Iter = crate::iter::ParIter<'a, 'a, K, V, P>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'ctx, 'a, K, V, S, P> rayon::iter::IntoParallelIterator
    for EpochSparseView<'ctx, 'a, K, V, S, P>
where
    K: Key + Sync,
    V: Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Sync,
    V::Fields: Sync,
    Refs<'ctx, 'a, V>: Send,
{
    type Item = (&'a K, Refs<'ctx, 'a, V>);
    type Iter = crate::iter::ParIter<'ctx, 'a, K, V, P>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.into_par_iter()
    }
}
