use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr, slice,
};

use crate::{
    algo::sparse_get_unchecked,
    item::{
        DenseContext, DenseItem, DenseMutPtrs, DensePtrs, DenseSliceMutPtrs, DenseSlicePtrs,
        SparseItem,
    },
    iter::{RawIter, RawIterMut, RawKeys, RawValues, RawValuesMut},
    key::Key,
    soa::{
        slice::SoaSliceMutPtrs,
        traits::{MutPtrs, Ptrs, RawSoa, SliceMutPtrs, SlicePtrs},
    },
    view::{EpochSparseView, EpochSparseViewMut, EpochSparseViewPtr},
};

pub struct EpochSparseViewMutPtr<'c, K, V>
where
    K: Key + 'c,
    V: RawSoa + ?Sized + 'c,
{
    dense: SoaSliceMutPtrs<'c, DenseItem<K, V>>,
    sparse: *mut [SparseItem<K>],
}

impl<'c, K, V> EpochSparseViewMutPtr<'c, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    pub unsafe fn from_parts(
        dense: SoaSliceMutPtrs<'c, DenseItem<K, V>>,
        sparse: *mut [SparseItem<K>],
    ) -> Self {
        Self { dense, sparse }
    }

    #[inline]
    pub fn into_parts(self) -> (SoaSliceMutPtrs<'c, DenseItem<K, V>>, *mut [SparseItem<K>]) {
        let Self { dense, sparse } = self;
        (dense, sparse)
    }

    #[inline]
    pub fn cast_const(self) -> EpochSparseViewPtr<'c, K, V> {
        let Self { dense, sparse } = self;

        let dense = dense.cast_const();
        let sparse = sparse.cast_const();
        unsafe { EpochSparseViewPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> EpochSparseView<'c, 'a, K, V> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> EpochSparseViewMut<'c, 'a, K, V> {
        let Self { dense, sparse } = self;

        let dense = unsafe { dense.deref_mut() };
        let sparse = unsafe { slice::from_raw_parts_mut(sparse.cast(), sparse.len()) };
        unsafe { EpochSparseViewMut::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn context(&self) -> &V::Context {
        let Self { dense, .. } = self;
        dense.context()
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
    pub fn as_ptrs(&self) -> (DensePtrs<'_, K, V>, *const SparseItem<K>) {
        let (_, dense, sparse) = self.as_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&V::Context, DensePtrs<'_, K, V>, *const SparseItem<K>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.as_ptrs_with_context();
        let sparse = sparse.cast();
        (context, dense, sparse)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (DenseMutPtrs<'_, K, V>, *mut SparseItem<K>) {
        let (_, dense, sparse) = self.as_mut_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(
        &mut self,
    ) -> (&V::Context, DenseMutPtrs<'_, K, V>, *mut SparseItem<K>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.as_mut_ptrs_with_context();
        let sparse = sparse.cast();
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
    pub fn as_key_mut_ptr(&mut self) -> *mut K {
        let (_, key) = self.as_key_mut_ptr_with_context();
        key
    }

    #[inline]
    pub fn as_key_mut_ptr_with_context(&mut self) -> (&V::Context, *mut K) {
        let (context, dense) = self.as_dense_mut_ptrs_with_context();
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
    pub fn as_value_mut_ptrs(&mut self) -> MutPtrs<'_, V> {
        let (_, value) = self.as_value_mut_ptrs_with_context();
        value
    }

    #[inline]
    pub fn as_value_mut_ptrs_with_context(&mut self) -> (&V::Context, MutPtrs<'_, V>) {
        let (context, dense) = self.as_dense_mut_ptrs_with_context();
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
    pub fn as_dense_mut_ptrs(&mut self) -> DenseMutPtrs<'_, K, V> {
        let (_, dense) = self.as_dense_mut_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_dense_mut_ptrs_with_context(&mut self) -> (&V::Context, DenseMutPtrs<'_, K, V>) {
        let (context, dense, _) = self.as_mut_ptrs_with_context();
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
    pub fn as_sparse_mut_ptr(&mut self) -> *mut SparseItem<K> {
        let (_, sparse) = self.as_sparse_mut_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn as_sparse_mut_ptr_with_context(&mut self) -> (&V::Context, *mut SparseItem<K>) {
        let (context, _, sparse) = self.as_mut_ptrs_with_context();
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
        let sparse = sparse.cast();
        (context, dense, sparse)
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> (DenseMutPtrs<'c, K, V>, *mut SparseItem<K>) {
        let (_, dense, sparse) = self.into_mut_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_mut_ptrs_with_context(
        self,
    ) -> (&'c V::Context, DenseMutPtrs<'c, K, V>, *mut SparseItem<K>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_mut_ptrs_with_context();
        let sparse = sparse.cast();
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
    pub fn into_key_mut_ptr(self) -> *mut K {
        let (_, key) = self.into_key_mut_ptr_with_context();
        key
    }

    #[inline]
    pub fn into_key_mut_ptr_with_context(self) -> (&'c V::Context, *mut K) {
        let (context, dense) = self.into_dense_mut_ptrs_with_context();
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
    pub fn into_value_mut_ptrs(self) -> MutPtrs<'c, V> {
        let (_, value) = self.into_value_mut_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_value_mut_ptrs_with_context(self) -> (&'c V::Context, MutPtrs<'c, V>) {
        let (context, dense) = self.into_dense_mut_ptrs_with_context();
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
    pub fn into_dense_mut_ptrs(self) -> DenseMutPtrs<'c, K, V> {
        let (_, dense) = self.into_dense_mut_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_mut_ptrs_with_context(self) -> (&'c V::Context, DenseMutPtrs<'c, K, V>) {
        let (context, dense, _) = self.into_mut_ptrs_with_context();
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
    pub fn into_sparse_mut_ptr(self) -> *mut SparseItem<K> {
        let (_, sparse) = self.into_sparse_mut_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_mut_ptr_with_context(self) -> (&'c V::Context, *mut SparseItem<K>) {
        let (context, _, sparse) = self.into_mut_ptrs_with_context();
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
        let Self { dense, sparse } = self;

        let (context, dense) = dense.as_slice_ptrs_with_context();
        let sparse = sparse.cast_const();
        (context, dense, sparse)
    }

    #[inline]
    pub fn as_mut_slice_ptrs(&mut self) -> (DenseSliceMutPtrs<'_, K, V>, *mut [SparseItem<K>]) {
        let (_, dense, sparse) = self.as_mut_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_mut_slice_ptrs_with_context(
        &mut self,
    ) -> (
        &V::Context,
        DenseSliceMutPtrs<'_, K, V>,
        *mut [SparseItem<K>],
    ) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.as_mut_slice_ptrs_with_context();
        let sparse = *sparse;
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
    pub fn as_mut_key_slice_ptr(&mut self) -> *mut [K] {
        let (_, key) = self.as_mut_key_slice_ptr_with_context();
        key
    }

    #[inline]
    pub fn as_mut_key_slice_ptr_with_context(&mut self) -> (&V::Context, *mut [K]) {
        let (context, dense) = self.as_mut_dense_slice_ptrs_with_context();
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
    pub fn as_mut_value_slice_ptrs(&mut self) -> SliceMutPtrs<'_, V> {
        let (_, value) = self.as_mut_value_slice_ptrs_with_context();
        value
    }

    #[inline]
    pub fn as_mut_value_slice_ptrs_with_context(&mut self) -> (&V::Context, SliceMutPtrs<'_, V>) {
        let (context, dense) = self.as_mut_dense_slice_ptrs_with_context();
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
    pub fn as_mut_dense_slice_ptrs(&mut self) -> DenseSliceMutPtrs<'_, K, V> {
        let (_, dense) = self.as_mut_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_mut_dense_slice_ptrs_with_context(
        &mut self,
    ) -> (&V::Context, DenseSliceMutPtrs<'_, K, V>) {
        let (context, dense, _) = self.as_mut_slice_ptrs_with_context();
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
    pub fn as_mut_sparse_slice_ptr(&mut self) -> *mut [SparseItem<K>] {
        let (_, sparse) = self.as_mut_sparse_slice_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn as_mut_sparse_slice_ptr_with_context(&mut self) -> (&V::Context, *mut [SparseItem<K>]) {
        let (context, _, sparse) = self.as_mut_slice_ptrs_with_context();
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
        let sparse = sparse.cast_const();
        (context, dense, sparse)
    }

    #[inline]
    pub fn into_mut_slice_ptrs(self) -> (DenseSliceMutPtrs<'c, K, V>, *mut [SparseItem<K>]) {
        let (_, dense, sparse) = self.into_mut_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_mut_slice_ptrs_with_context(
        self,
    ) -> (
        &'c V::Context,
        DenseSliceMutPtrs<'c, K, V>,
        *mut [SparseItem<K>],
    ) {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.into_mut_slice_ptrs_with_context();
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
    pub fn into_mut_key_slice_ptr(self) -> *mut [K] {
        let (_, key) = self.into_mut_key_slice_ptr_with_context();
        key
    }

    #[inline]
    pub fn into_mut_key_slice_ptr_with_context(self) -> (&'c V::Context, *mut [K]) {
        let (context, dense) = self.into_mut_dense_slice_ptrs_with_context();
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
    pub fn into_mut_value_slice_ptrs(self) -> SliceMutPtrs<'c, V> {
        let (_, value) = self.into_mut_value_slice_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_mut_value_slice_ptrs_with_context(self) -> (&'c V::Context, SliceMutPtrs<'c, V>) {
        let (context, dense) = self.into_mut_dense_slice_ptrs_with_context();
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
    pub fn into_mut_dense_slice_ptrs(self) -> DenseSliceMutPtrs<'c, K, V> {
        let (_, dense) = self.into_mut_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_mut_dense_slice_ptrs_with_context(
        self,
    ) -> (&'c V::Context, DenseSliceMutPtrs<'c, K, V>) {
        let (context, dense, _) = self.into_mut_slice_ptrs_with_context();
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
    pub fn into_mut_sparse_slice_ptr(self) -> *mut [SparseItem<K>] {
        let (_, sparse) = self.into_mut_sparse_slice_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_mut_sparse_slice_ptr_with_context(self) -> (&'c V::Context, *mut [SparseItem<K>]) {
        let (context, _, sparse) = self.into_mut_slice_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, key: K) -> Ptrs<'_, V> {
        let (_, ptrs) = unsafe { self.get_unchecked_with_context(key) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_with_context(&self, key: K) -> (&V::Context, Ptrs<'_, V>) {
        let Self { ref dense, sparse } = *self;

        let (context, dense) = dense.iter_with_context();
        let dense = dense.map(From::from);
        let (_, value) = unsafe { sparse_get_unchecked(dense, sparse, key.sparse_index()) };
        (context, value)
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, key: K) -> MutPtrs<'_, V> {
        let (_, ptrs) = unsafe { self.get_unchecked_mut_with_context(key) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_mut_with_context(
        &mut self,
        key: K,
    ) -> (&V::Context, MutPtrs<'_, V>) {
        let Self {
            ref mut dense,
            sparse,
        } = *self;

        let (context, dense) = dense.iter_mut_with_context();
        let dense = dense.map(From::from);
        let (_, value) = unsafe { sparse_get_unchecked(dense, sparse, key.sparse_index()) };
        (context, value)
    }

    #[inline]
    pub unsafe fn into_get_unchecked(self, key: K) -> Ptrs<'c, V> {
        let (_, ptrs) = unsafe { self.into_get_unchecked_with_context(key) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_with_context(self, key: K) -> (&'c V::Context, Ptrs<'c, V>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let dense = dense.cast_const().map(From::from);
        let (_, value) = unsafe { sparse_get_unchecked(dense, sparse, key.sparse_index()) };
        (context, value)
    }

    #[inline]
    pub unsafe fn into_get_unchecked_mut(self, key: K) -> MutPtrs<'c, V> {
        let (_, ptrs) = unsafe { self.into_get_unchecked_mut_with_context(key) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_mut_with_context(
        self,
        key: K,
    ) -> (&'c V::Context, MutPtrs<'c, V>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let dense = dense.map(From::from);
        let (_, value) = unsafe { sparse_get_unchecked(dense, sparse, key.sparse_index()) };
        (context, value)
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
        let Self { ref dense, sparse } = *self;

        let (context, dense) = dense.iter_with_context();
        let dense = dense.map(From::from);
        let (key, value) = unsafe { sparse_get_unchecked(dense, sparse, sparse_index) };
        (context, key, value)
    }

    #[inline]
    pub unsafe fn get_mut_with_key_unchecked(
        &mut self,
        sparse_index: K::SparseIndex,
    ) -> (*mut K, MutPtrs<'_, V>) {
        let (_, key, value) = unsafe { self.get_mut_with_key_unchecked_with_context(sparse_index) };
        (key, value)
    }

    #[inline]
    pub unsafe fn get_mut_with_key_unchecked_with_context(
        &mut self,
        sparse_index: K::SparseIndex,
    ) -> (&V::Context, *mut K, MutPtrs<'_, V>) {
        let Self {
            ref mut dense,
            sparse,
        } = *self;

        let (context, dense) = dense.iter_mut_with_context();
        let dense = dense.map(From::from);
        let (key, value) = unsafe { sparse_get_unchecked(dense, sparse, sparse_index) };
        (context, key, value)
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
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let dense = dense.cast_const().map(From::from);
        let (key, value) = unsafe { sparse_get_unchecked(dense, sparse, sparse_index) };
        (context, key, value)
    }

    #[inline]
    pub unsafe fn into_get_mut_with_key_unchecked(
        self,
        sparse_index: K::SparseIndex,
    ) -> (*mut K, MutPtrs<'c, V>) {
        let (_, key, value) =
            unsafe { self.into_get_mut_with_key_unchecked_with_context(sparse_index) };
        (key, value)
    }

    #[inline]
    pub unsafe fn into_get_mut_with_key_unchecked_with_context(
        self,
        sparse_index: K::SparseIndex,
    ) -> (&'c V::Context, *mut K, MutPtrs<'c, V>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let dense = dense.map(From::from);
        let (key, value) = unsafe { sparse_get_unchecked(dense, sparse, sparse_index) };
        (context, key, value)
    }

    #[inline]
    pub fn keys(&self) -> RawKeys<'_, K, V> {
        let (_, iter) = self.keys_with_context();
        iter
    }

    #[inline]
    pub fn keys_with_context(&self) -> (&V::Context, RawKeys<'_, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.iter_with_context();
        let iter = RawKeys::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_keys(self) -> RawKeys<'c, K, V> {
        let (_, iter) = self.into_keys_with_context();
        iter
    }

    #[inline]
    pub fn into_keys_with_context(self) -> (&'c V::Context, RawKeys<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_iter_with_context();
        let iter = RawKeys::from_inner(inner.cast_const());
        (context, iter)
    }

    #[inline]
    pub fn values(&self) -> RawValues<'_, K, V> {
        let (_, iter) = self.values_with_context();
        iter
    }

    #[inline]
    pub fn values_with_context(&self) -> (&V::Context, RawValues<'_, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.iter_with_context();
        let iter = RawValues::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn values_mut(&mut self) -> RawValuesMut<'_, K, V> {
        let (_, iter) = self.values_mut_with_context();
        iter
    }

    #[inline]
    pub fn values_mut_with_context(&mut self) -> (&V::Context, RawValuesMut<'_, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.iter_mut_with_context();
        let iter = RawValuesMut::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_values(self) -> RawValues<'c, K, V> {
        let (_, iter) = self.into_values_with_context();
        iter
    }

    #[inline]
    pub fn into_values_with_context(self) -> (&'c V::Context, RawValues<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_iter_with_context();
        let iter = RawValues::from_inner(inner.cast_const());
        (context, iter)
    }

    #[inline]
    pub fn into_values_mut(self) -> RawValuesMut<'c, K, V> {
        let (_, iter) = self.into_values_mut_with_context();
        iter
    }

    #[inline]
    pub fn into_values_mut_with_context(self) -> (&'c V::Context, RawValuesMut<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_iter_with_context();
        let iter = RawValuesMut::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn iter(&self) -> RawIter<'_, K, V> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&self) -> (&V::Context, RawIter<'_, K, V>) {
        let Self { dense, .. } = self;

        let (context, iter) = dense.iter_with_context();
        let iter = RawIter::from_inner(iter);
        (context, iter)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> RawIterMut<'_, K, V> {
        let (_, iter) = self.iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn iter_mut_with_context(&mut self) -> (&V::Context, RawIterMut<'_, K, V>) {
        let Self { dense, .. } = self;

        let (context, iter) = dense.iter_mut_with_context();
        let iter = RawIterMut::from_inner(iter);
        (context, iter)
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'c V::Context, RawIterMut<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, iter) = dense.into_iter_with_context();
        let iter = RawIterMut::from_inner(iter);
        (context, iter)
    }
}

impl<'c, K, V> From<&'c V::Context> for EpochSparseViewMutPtr<'c, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(context: &'c V::Context) -> Self {
        let context = DenseContext::from_inner_ref(context);
        let dense = SoaSliceMutPtrs::from(context);
        let sparse = ptr::from_mut(Default::default());
        unsafe { Self::from_parts(dense, sparse) }
    }
}

impl<K, V> Debug for EpochSparseViewMutPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    for<'c> SlicePtrs<'c, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { dense, sparse } = self;
        f.debug_struct("EpochSparseViewMutPtr")
            .field("dense", dense)
            .field("sparse", sparse)
            .finish()
    }
}

impl<K, V> Clone for EpochSparseViewMutPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { ref dense, sparse } = *self;
        let dense = dense.clone();
        Self { dense, sparse }
    }
}

impl<K, V> Copy for EpochSparseViewMutPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    for<'c> MutPtrs<'c, V>: Copy,
{
}

impl<K, V> PartialEq for EpochSparseViewMutPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: PartialEq,
    for<'c> MutPtrs<'c, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { dense, sparse } = self;
        *dense == other.dense && ptr::eq(*sparse, other.sparse)
    }
}

impl<K, V> Eq for EpochSparseViewMutPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: Eq,
    for<'c> MutPtrs<'c, V>: Eq,
{
}

impl<K, V> PartialOrd for EpochSparseViewMutPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: PartialOrd,
    for<'c> MutPtrs<'c, V>: PartialOrd,
{
    #[expect(ambiguous_wide_pointer_comparisons)]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { dense, sparse } = self;

        match dense.partial_cmp(&other.dense) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        sparse.partial_cmp(&other.sparse)
    }
}

impl<K, V> Ord for EpochSparseViewMutPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: Ord,
    for<'c> MutPtrs<'c, V>: Ord,
{
    #[expect(ambiguous_wide_pointer_comparisons)]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { dense, sparse } = self;

        match dense.cmp(&other.dense) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        sparse.cmp(&other.sparse)
    }
}

impl<K, V> Hash for EpochSparseViewMutPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: Hash,
    for<'c> MutPtrs<'c, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { dense, sparse } = self;
        dense.hash(state);
        sparse.hash(state);
    }
}

impl<'r, K, V> IntoIterator for &'r EpochSparseViewMutPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    type Item = (*const K, Ptrs<'r, V>);
    type IntoIter = RawIter<'r, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'r, K, V> IntoIterator for &'r mut EpochSparseViewMutPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    type Item = (*mut K, MutPtrs<'r, V>);
    type IntoIter = RawIterMut<'r, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'c, K, V> IntoIterator for EpochSparseViewMutPtr<'c, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    type Item = (*mut K, MutPtrs<'c, V>);
    type IntoIter = RawIterMut<'c, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}
