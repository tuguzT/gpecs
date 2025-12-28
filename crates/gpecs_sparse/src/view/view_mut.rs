use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem::swap,
    ops::{Index, IndexMut},
    ptr, slice,
};

use crate::{
    algo::{sparse_contains_key, sparse_get, sparse_get_epoch, sparse_get_with_key, sparse_index},
    assert::{
        check_compatible_key, check_equal_epoch, check_equal_key, unwrap_dense,
        unwrap_dense_from_sparse_index, unwrap_dense_index, unwrap_dense_index_mut,
        unwrap_dense_pair, unwrap_into_index, unwrap_into_usize, unwrap_sparse_item,
        unwrap_sparse_items_pair_mut,
    },
    error::FromPartsError,
    item::{
        DenseItem, DenseMutPtrs, DensePtrs, DenseSliceMutPtrs, DenseSlicePtrs, DenseSlices,
        DenseSlicesMut, SparseItem,
    },
    iter::{
        Iter, IterMut, Keys, RawIter, RawIterMut, RawKeys, RawValues, RawValuesMut, Values,
        ValuesMut,
    },
    key::{Epoch, Key},
    soa::{
        self,
        slice::{Iter as SoaIter, SoaSlices, SoaSlicesMut},
        traits::{MutPtrs, Ptrs, RawSoa, SliceMutPtrs, SlicePtrs, Soa},
    },
    view::{EpochSparseView, EpochSparseViewMutPtr, EpochSparseViewPtr, assert::check_parts},
};

pub struct EpochSparseViewMut<'ctx, 'a, K, V>
where
    K: Key + 'ctx + 'a,
    V: RawSoa + ?Sized + 'ctx + 'a,
{
    dense: SoaSlicesMut<'ctx, 'a, DenseItem<K, V>>,
    sparse: &'a mut [SparseItem<K>],
}

impl<'ctx, 'a, K, V> EpochSparseViewMut<'ctx, 'a, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    pub unsafe fn from_parts(
        dense: SoaSlicesMut<'ctx, 'a, DenseItem<K, V>>,
        sparse: &'a mut [SparseItem<K>],
    ) -> Self {
        Self { dense, sparse }
    }

    #[inline]
    pub fn into_parts(
        self,
    ) -> (
        SoaSlicesMut<'ctx, 'a, DenseItem<K, V>>,
        &'a mut [SparseItem<K>],
    ) {
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
        let Self { dense, sparse } = self;

        let dense = dense.slice_ptrs();
        let sparse = ptr::from_ref(*sparse);
        unsafe { EpochSparseViewPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn into_view_ptr(self) -> EpochSparseViewPtr<'ctx, K, V> {
        let Self { dense, sparse } = self;

        let dense = dense.into_slice_ptrs();
        let sparse = ptr::from_ref(sparse);
        unsafe { EpochSparseViewPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn as_view_mut_ptr(&mut self) -> EpochSparseViewMutPtr<'_, K, V> {
        let Self { dense, sparse } = self;

        let dense = dense.mut_slice_ptrs();
        let sparse = ptr::from_mut(*sparse);
        unsafe { EpochSparseViewMutPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn into_view_mut_ptr(self) -> EpochSparseViewMutPtr<'ctx, K, V> {
        let Self { dense, sparse } = self;

        let dense = dense.into_mut_slice_ptrs();
        let sparse = ptr::from_mut(sparse);
        unsafe { EpochSparseViewMutPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn as_view(&self) -> EpochSparseView<'_, '_, K, V> {
        unsafe { self.as_view_ptr().deref() }
    }

    #[inline]
    pub fn as_view_mut(&mut self) -> EpochSparseViewMut<'_, '_, K, V> {
        unsafe { self.as_view_mut_ptr().deref_mut() }
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
        let sparse = sparse.as_ptr();
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
        let sparse = sparse.as_mut_ptr();
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
    pub fn into_ptrs(self) -> (DensePtrs<'ctx, K, V>, *const SparseItem<K>) {
        let (_, dense, sparse) = self.into_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_ptrs_with_context(
        self,
    ) -> (
        &'ctx V::Context,
        DensePtrs<'ctx, K, V>,
        *const SparseItem<K>,
    ) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_slice_ptrs().into_ptrs_with_context();
        let sparse = sparse.as_ptr();
        (context, dense, sparse)
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> (DenseMutPtrs<'ctx, K, V>, *mut SparseItem<K>) {
        let (_, dense, sparse) = self.into_mut_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_mut_ptrs_with_context(
        self,
    ) -> (
        &'ctx V::Context,
        DenseMutPtrs<'ctx, K, V>,
        *mut SparseItem<K>,
    ) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_mut_slice_ptrs().into_mut_ptrs_with_context();
        let sparse = sparse.as_mut_ptr();
        (context, dense, sparse)
    }

    #[inline]
    pub fn into_key_ptr(self) -> *const K {
        let (_, key) = self.into_key_ptr_with_context();
        key
    }

    #[inline]
    pub fn into_key_ptr_with_context(self) -> (&'ctx V::Context, *const K) {
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
    pub fn into_key_mut_ptr_with_context(self) -> (&'ctx V::Context, *mut K) {
        let (context, dense) = self.into_dense_mut_ptrs_with_context();
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
    pub fn into_value_mut_ptrs(self) -> MutPtrs<'ctx, V> {
        let (_, value) = self.into_value_mut_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_value_mut_ptrs_with_context(self) -> (&'ctx V::Context, MutPtrs<'ctx, V>) {
        let (context, dense) = self.into_dense_mut_ptrs_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn into_dense_ptrs(self) -> DensePtrs<'ctx, K, V> {
        let (_, dense) = self.into_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_ptrs_with_context(self) -> (&'ctx V::Context, DensePtrs<'ctx, K, V>) {
        let (context, dense, _) = self.into_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_dense_mut_ptrs(self) -> DenseMutPtrs<'ctx, K, V> {
        let (_, dense) = self.into_dense_mut_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_mut_ptrs_with_context(self) -> (&'ctx V::Context, DenseMutPtrs<'ctx, K, V>) {
        let (context, dense, _) = self.into_mut_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_sparse_ptr(self) -> *const SparseItem<K> {
        let (_, sparse) = self.into_sparse_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_ptr_with_context(self) -> (&'ctx V::Context, *const SparseItem<K>) {
        let (context, _, sparse) = self.into_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn into_sparse_mut_ptr(self) -> *mut SparseItem<K> {
        let (_, sparse) = self.into_sparse_mut_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_mut_ptr_with_context(self) -> (&'ctx V::Context, *mut SparseItem<K>) {
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
        let sparse = ptr::from_ref(*sparse);
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
        let sparse = ptr::from_mut(*sparse);
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
    pub fn into_slice_ptrs(self) -> (DenseSlicePtrs<'ctx, K, V>, *const [SparseItem<K>]) {
        let (_, dense, sparse) = self.into_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(
        self,
    ) -> (
        &'ctx V::Context,
        DenseSlicePtrs<'ctx, K, V>,
        *const [SparseItem<K>],
    ) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_slice_ptrs().into_slice_ptrs_with_context();
        let sparse = ptr::from_ref(sparse);
        (context, dense, sparse)
    }

    #[inline]
    pub fn into_mut_slice_ptrs(self) -> (DenseSliceMutPtrs<'ctx, K, V>, *mut [SparseItem<K>]) {
        let (_, dense, sparse) = self.into_mut_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_mut_slice_ptrs_with_context(
        self,
    ) -> (
        &'ctx V::Context,
        DenseSliceMutPtrs<'ctx, K, V>,
        *mut [SparseItem<K>],
    ) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense
            .into_mut_slice_ptrs()
            .into_mut_slice_ptrs_with_context();
        let sparse = ptr::from_mut(sparse);
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
    pub fn into_mut_key_slice_ptr(self) -> *mut [K] {
        let (_, key) = self.into_mut_key_slice_ptr_with_context();
        key
    }

    #[inline]
    pub fn into_mut_key_slice_ptr_with_context(self) -> (&'ctx V::Context, *mut [K]) {
        let (context, dense) = self.into_mut_dense_slice_ptrs_with_context();
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
    pub fn into_mut_value_slice_ptrs(self) -> SliceMutPtrs<'ctx, V> {
        let (_, value) = self.into_mut_value_slice_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_mut_value_slice_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, SliceMutPtrs<'ctx, V>) {
        let (context, dense) = self.into_mut_dense_slice_ptrs_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn into_dense_slice_ptrs(self) -> DenseSlicePtrs<'ctx, K, V> {
        let (_, dense) = self.into_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_slice_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, DenseSlicePtrs<'ctx, K, V>) {
        let (context, dense, _) = self.into_slice_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_mut_dense_slice_ptrs(self) -> DenseSliceMutPtrs<'ctx, K, V> {
        let (_, dense) = self.into_mut_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_mut_dense_slice_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, DenseSliceMutPtrs<'ctx, K, V>) {
        let (context, dense, _) = self.into_mut_slice_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_sparse_slice_ptr(self) -> *const [SparseItem<K>] {
        let (_, sparse) = self.into_sparse_slice_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_slice_ptr_with_context(self) -> (&'ctx V::Context, *const [SparseItem<K>]) {
        let (context, _, sparse) = self.into_slice_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn into_mut_sparse_slice_ptr(self) -> *mut [SparseItem<K>] {
        let (_, sparse) = self.into_mut_sparse_slice_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_mut_sparse_slice_ptr_with_context(
        self,
    ) -> (&'ctx V::Context, *mut [SparseItem<K>]) {
        let (context, _, sparse) = self.into_mut_slice_ptrs_with_context();
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
    pub unsafe fn as_mut_key_slice(&mut self) -> &mut [K] {
        let (_, keys) = unsafe { self.as_mut_key_slice_with_context() };
        keys
    }

    #[inline]
    pub unsafe fn as_mut_key_slice_with_context(&mut self) -> (&V::Context, &mut [K]) {
        let (context, keys) = self.as_mut_key_slice_ptr_with_context();
        let keys = unsafe { slice::from_raw_parts_mut(keys.cast(), keys.len()) };
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
    pub unsafe fn as_mut_sparse_slice(&mut self) -> &mut [SparseItem<K>] {
        let (_, sparse) = unsafe { self.as_mut_sparse_slice_with_context() };
        sparse
    }

    #[inline]
    pub unsafe fn as_mut_sparse_slice_with_context(
        &mut self,
    ) -> (&V::Context, &mut [SparseItem<K>]) {
        let (context, sparse) = self.as_mut_sparse_slice_ptr_with_context();
        let sparse = unsafe { slice::from_raw_parts_mut(sparse.cast(), sparse.len()) };
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
        let keys = unsafe { slice::from_raw_parts(keys.cast(), keys.len()) };
        (context, keys)
    }

    #[inline]
    pub unsafe fn into_mut_key_slice(self) -> &'a mut [K] {
        let (_, keys) = unsafe { self.into_mut_key_slice_with_context() };
        keys
    }

    #[inline]
    pub unsafe fn into_mut_key_slice_with_context(self) -> (&'ctx V::Context, &'a mut [K]) {
        let (context, keys) = self.into_mut_key_slice_ptr_with_context();
        let keys = unsafe { slice::from_raw_parts_mut(keys.cast(), keys.len()) };
        (context, keys)
    }

    #[inline]
    pub fn into_sparse_slice(self) -> &'a [SparseItem<K>] {
        let (_, sparse) = self.into_sparse_slice_with_context();
        sparse
    }

    #[inline]
    pub fn into_sparse_slice_with_context(self) -> (&'ctx V::Context, &'a [SparseItem<K>]) {
        let (context, sparse) = self.into_sparse_slice_ptr_with_context();
        let sparse = unsafe { slice::from_raw_parts(sparse.cast(), sparse.len()) };
        (context, sparse)
    }

    #[inline]
    pub unsafe fn into_mut_sparse_slice(self) -> &'a mut [SparseItem<K>] {
        let (_, sparse) = unsafe { self.into_mut_sparse_slice_with_context() };
        sparse
    }

    #[inline]
    pub unsafe fn into_mut_sparse_slice_with_context(
        self,
    ) -> (&'ctx V::Context, &'a mut [SparseItem<K>]) {
        let (context, sparse) = self.into_mut_sparse_slice_ptr_with_context();
        let sparse = unsafe { slice::from_raw_parts_mut(sparse.cast(), sparse.len()) };
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
    pub unsafe fn get_unchecked_mut(&mut self, key: K) -> MutPtrs<'_, V> {
        let (_, ptrs) = unsafe { self.get_unchecked_mut_with_context(key) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_mut_with_context(
        &mut self,
        key: K,
    ) -> (&V::Context, MutPtrs<'_, V>) {
        let view_ptr = self.as_view_mut_ptr();
        unsafe { view_ptr.into_get_unchecked_mut_with_context(key) }
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
    pub unsafe fn into_get_unchecked_mut(self, key: K) -> MutPtrs<'ctx, V> {
        let (_, ptrs) = unsafe { self.into_get_unchecked_mut_with_context(key) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_mut_with_context(
        self,
        key: K,
    ) -> (&'ctx V::Context, MutPtrs<'ctx, V>) {
        let view_ptr = self.into_view_mut_ptr();
        unsafe { view_ptr.into_get_unchecked_mut_with_context(key) }
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
        let view_ptr = self.as_view_mut_ptr();
        unsafe { view_ptr.into_get_mut_with_key_unchecked_with_context(sparse_index) }
    }

    #[inline]
    pub unsafe fn into_get_with_key_unchecked(
        self,
        sparse_index: K::SparseIndex,
    ) -> (*const K, Ptrs<'ctx, V>) {
        let (_, key, value) =
            unsafe { self.into_get_with_key_unchecked_with_context(sparse_index) };
        (key, value)
    }

    #[inline]
    pub unsafe fn into_get_with_key_unchecked_with_context(
        self,
        sparse_index: K::SparseIndex,
    ) -> (&'ctx V::Context, *const K, Ptrs<'ctx, V>) {
        let view_ptr = self.into_view_ptr();
        unsafe { view_ptr.into_get_with_key_unchecked_with_context(sparse_index) }
    }

    #[inline]
    pub unsafe fn into_get_mut_with_key_unchecked(
        self,
        sparse_index: K::SparseIndex,
    ) -> (*mut K, MutPtrs<'ctx, V>) {
        let (_, key, value) =
            unsafe { self.into_get_mut_with_key_unchecked_with_context(sparse_index) };
        (key, value)
    }

    #[inline]
    pub unsafe fn into_get_mut_with_key_unchecked_with_context(
        self,
        sparse_index: K::SparseIndex,
    ) -> (&'ctx V::Context, *mut K, MutPtrs<'ctx, V>) {
        let view_ptr = self.into_view_mut_ptr();
        unsafe { view_ptr.into_get_mut_with_key_unchecked_with_context(sparse_index) }
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
    pub fn into_raw_keys(self) -> RawKeys<'ctx, K, V> {
        let (_, iter) = self.into_raw_keys_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_keys_with_context(self) -> (&'ctx V::Context, RawKeys<'ctx, K, V>) {
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
    pub fn into_raw_values(self) -> RawValues<'ctx, K, V> {
        let (_, iter) = self.into_raw_values_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_values_with_context(self) -> (&'ctx V::Context, RawValues<'ctx, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_with_context();
        let iter = RawValues::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn raw_values_mut(&mut self) -> RawValuesMut<'_, K, V> {
        let (_, iter) = self.raw_values_mut_with_context();
        iter
    }

    #[inline]
    pub fn raw_values_mut_with_context(&mut self) -> (&V::Context, RawValuesMut<'_, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.raw_iter_mut_with_context();
        let iter = RawValuesMut::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_raw_values_mut(self) -> RawValuesMut<'ctx, K, V> {
        let (_, iter) = self.into_raw_values_mut_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_values_mut_with_context(self) -> (&'ctx V::Context, RawValuesMut<'ctx, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_mut_with_context();
        let iter = RawValuesMut::from_inner(inner);
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
    pub fn into_raw_iter(self) -> RawIter<'ctx, K, V> {
        let (_, iter) = self.into_raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_iter_with_context(self) -> (&'ctx V::Context, RawIter<'ctx, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_with_context();
        let iter = RawIter::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn raw_iter_mut(&mut self) -> RawIterMut<'_, K, V> {
        let (_, iter) = self.raw_iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn raw_iter_mut_with_context(&mut self) -> (&V::Context, RawIterMut<'_, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.raw_iter_mut_with_context();
        let iter = RawIterMut::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_raw_iter_mut(self) -> RawIterMut<'ctx, K, V> {
        let (_, iter) = self.into_raw_iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_iter_mut_with_context(self) -> (&'ctx V::Context, RawIterMut<'ctx, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_mut_with_context();
        let iter = RawIterMut::from_inner(inner);
        (context, iter)
    }
}

impl<'ctx, 'a, K, V> EpochSparseViewMut<'ctx, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    pub fn new(
        dense: SoaSlicesMut<'ctx, 'a, DenseItem<K, V>>,
        sparse: &'a mut [SparseItem<K>],
    ) -> Result<Self, FromPartsError<K>> {
        check_parts(&dense.slices(), sparse)?;

        let me = unsafe { Self::from_parts(dense, sparse) };
        Ok(me)
    }

    #[inline]
    pub fn as_slices(&self) -> (DenseSlices<'_, '_, K, V>, &[SparseItem<K>]) {
        let (_, dense, sparse) = self.as_slices_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_slices_with_context(
        &self,
    ) -> (&V::Context, DenseSlices<'_, '_, K, V>, &[SparseItem<K>]) {
        let (context, dense, sparse) = self.as_slice_ptrs_with_context();
        let dense = unsafe { dense.deref(context) };
        let sparse = unsafe { slice::from_raw_parts(sparse.cast(), sparse.len()) };
        (context, dense, sparse)
    }

    #[inline]
    pub unsafe fn as_mut_slices(&mut self) -> (DenseSlicesMut<'_, '_, K, V>, &mut [SparseItem<K>]) {
        let (_, dense, sparse) = unsafe { self.as_mut_slices_with_context() };
        (dense, sparse)
    }

    #[inline]
    pub unsafe fn as_mut_slices_with_context(
        &mut self,
    ) -> (
        &V::Context,
        DenseSlicesMut<'_, '_, K, V>,
        &mut [SparseItem<K>],
    ) {
        let (context, dense, sparse) = self.as_mut_slice_ptrs_with_context();
        let dense = unsafe { dense.deref_mut(context) };
        let sparse = unsafe { slice::from_raw_parts_mut(sparse.cast(), sparse.len()) };
        (context, dense, sparse)
    }

    #[inline]
    pub fn as_value_slices(&self) -> V::Slices<'_, '_> {
        let (_, value) = self.as_value_slices_with_context();
        value
    }

    #[inline]
    pub fn as_value_slices_with_context(&self) -> (&V::Context, V::Slices<'_, '_>) {
        let (context, dense) = self.as_dense_slices_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn as_mut_value_slices(&mut self) -> V::SlicesMut<'_, '_> {
        let (_, value) = self.as_mut_value_slices_with_context();
        value
    }

    #[inline]
    pub fn as_mut_value_slices_with_context(&mut self) -> (&V::Context, V::SlicesMut<'_, '_>) {
        let (context, dense) = unsafe { self.as_mut_dense_slices_with_context() };
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn as_dense_slices(&self) -> DenseSlices<'_, '_, K, V> {
        let (_, dense) = self.as_dense_slices_with_context();
        dense
    }

    #[inline]
    pub fn as_dense_slices_with_context(&self) -> (&V::Context, DenseSlices<'_, '_, K, V>) {
        let (context, dense, _) = self.as_slices_with_context();
        (context, dense)
    }

    #[inline]
    pub unsafe fn as_mut_dense_slices(&mut self) -> DenseSlicesMut<'_, '_, K, V> {
        let (_, dense) = unsafe { self.as_mut_dense_slices_with_context() };
        dense
    }

    #[inline]
    pub unsafe fn as_mut_dense_slices_with_context(
        &mut self,
    ) -> (&V::Context, DenseSlicesMut<'_, '_, K, V>) {
        let (context, dense, _) = unsafe { self.as_mut_slices_with_context() };
        (context, dense)
    }

    #[inline]
    pub fn into_slices(self) -> (DenseSlices<'ctx, 'a, K, V>, &'a [SparseItem<K>]) {
        let (_, dense, sparse) = self.into_slices_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_slices_with_context(
        self,
    ) -> (
        &'ctx V::Context,
        DenseSlices<'ctx, 'a, K, V>,
        &'a [SparseItem<K>],
    ) {
        let (context, dense, sparse) = self.into_slice_ptrs_with_context();
        let dense = unsafe { dense.deref(context) };
        let sparse = unsafe { slice::from_raw_parts(sparse.cast(), sparse.len()) };
        (context, dense, sparse)
    }

    #[inline]
    pub unsafe fn into_mut_slices(
        self,
    ) -> (DenseSlicesMut<'ctx, 'a, K, V>, &'a mut [SparseItem<K>]) {
        let (_, dense, sparse) = unsafe { self.into_mut_slices_with_context() };
        (dense, sparse)
    }

    #[inline]
    pub unsafe fn into_mut_slices_with_context(
        self,
    ) -> (
        &'ctx V::Context,
        DenseSlicesMut<'ctx, 'a, K, V>,
        &'a mut [SparseItem<K>],
    ) {
        let (context, dense, sparse) = self.into_mut_slice_ptrs_with_context();
        let dense = unsafe { dense.deref_mut(context) };
        let sparse = unsafe { slice::from_raw_parts_mut(sparse.cast(), sparse.len()) };
        (context, dense, sparse)
    }

    #[inline]
    pub fn into_value_slices(self) -> V::Slices<'ctx, 'a> {
        let (_, value) = self.into_value_slices_with_context();
        value
    }

    #[inline]
    pub fn into_value_slices_with_context(self) -> (&'ctx V::Context, V::Slices<'ctx, 'a>) {
        let (context, dense) = self.into_dense_slices_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn into_mut_value_slices(self) -> V::SlicesMut<'ctx, 'a> {
        let (_, value) = self.into_mut_value_slices_with_context();
        value
    }

    #[inline]
    pub fn into_mut_value_slices_with_context(self) -> (&'ctx V::Context, V::SlicesMut<'ctx, 'a>) {
        let (context, dense) = unsafe { self.into_mut_dense_slices_with_context() };
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn into_dense_slices(self) -> DenseSlices<'ctx, 'a, K, V> {
        let (_, dense) = self.into_dense_slices_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_slices_with_context(self) -> (&'ctx V::Context, DenseSlices<'ctx, 'a, K, V>) {
        let (context, dense, _) = self.into_slices_with_context();
        (context, dense)
    }

    #[inline]
    pub unsafe fn into_mut_dense_slices(self) -> DenseSlicesMut<'ctx, 'a, K, V> {
        let (_, dense) = unsafe { self.into_mut_dense_slices_with_context() };
        dense
    }

    #[inline]
    pub unsafe fn into_mut_dense_slices_with_context(
        self,
    ) -> (&'ctx V::Context, DenseSlicesMut<'ctx, 'a, K, V>) {
        let (context, dense, _) = unsafe { self.into_mut_slices_with_context() };
        (context, dense)
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
    pub fn into_get(self, key: K) -> Option<V::Refs<'ctx, 'a>> {
        let (_, refs) = self.into_get_with_context(key);
        refs
    }

    #[inline]
    pub fn into_get_with_context(self, key: K) -> (&'ctx V::Context, Option<V::Refs<'ctx, 'a>>) {
        let Self { dense, sparse } = self;

        let (context, dense) = SoaSlices::from(dense).into_iter_with_context();
        let refs = sparse_get(dense.map(From::from), sparse, key);
        (context, refs)
    }

    #[inline]
    pub fn get_mut(&mut self, key: K) -> Option<V::RefsMut<'_, '_>> {
        let (_, refs) = self.get_mut_with_context(key);
        refs
    }

    #[inline]
    pub fn get_mut_with_context(&mut self, key: K) -> (&V::Context, Option<V::RefsMut<'_, '_>>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.iter_mut_with_context();
        let refs = sparse_get(dense.map(From::from), sparse, key);
        (context, refs)
    }

    #[inline]
    pub fn into_get_mut(self, key: K) -> Option<V::RefsMut<'ctx, 'a>> {
        let (_, refs) = self.into_get_mut_with_context(key);
        refs
    }

    #[inline]
    pub fn into_get_mut_with_context(
        self,
        key: K,
    ) -> (&'ctx V::Context, Option<V::RefsMut<'ctx, 'a>>) {
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
    pub fn into_index(self, key: K) -> V::Refs<'ctx, 'a>
    where
        K: Debug,
    {
        let (_, refs) = self.into_index_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_with_context(self, key: K) -> (&'ctx V::Context, V::Refs<'ctx, 'a>)
    where
        K: Debug,
    {
        let Self { dense, sparse } = self;

        let (context, dense) = SoaSlices::from(dense).into_iter_with_context();
        let refs = sparse_index(dense.map(From::from), sparse, key);
        (context, refs)
    }

    #[inline]
    #[track_caller]
    pub fn index_mut(&mut self, key: K) -> V::RefsMut<'_, '_>
    where
        K: Debug,
    {
        let (_, refs) = self.index_mut_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_mut_with_context(&mut self, key: K) -> (&V::Context, V::RefsMut<'_, '_>)
    where
        K: Debug,
    {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.iter_mut_with_context();
        let refs = sparse_index(dense.map(From::from), sparse, key);
        (context, refs)
    }

    #[inline]
    pub fn into_index_mut(self, key: K) -> V::RefsMut<'ctx, 'a>
    where
        K: Debug,
    {
        let (_, refs) = self.into_index_mut_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_mut_with_context(self, key: K) -> (&'ctx V::Context, V::RefsMut<'ctx, 'a>)
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
    pub fn into_get_with_key(self, sparse_index: K::SparseIndex) -> Option<(K, V::Refs<'ctx, 'a>)> {
        let (_, pair) = self.into_get_with_key_with_context(sparse_index);
        pair
    }

    #[inline]
    pub fn into_get_with_key_with_context(
        self,
        sparse_index: K::SparseIndex,
    ) -> (&'ctx V::Context, Option<(K, V::Refs<'ctx, 'a>)>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let pair = sparse_get_with_key(dense.map(From::from), sparse, sparse_index)
            .map(|(key, value)| (key, V::refs_mut_as_refs(context, value)));
        (context, pair)
    }

    #[inline]
    pub fn get_mut_with_key(
        &mut self,
        sparse_index: K::SparseIndex,
    ) -> Option<(K, V::RefsMut<'_, '_>)> {
        let (_, pair) = self.get_mut_with_key_with_context(sparse_index);
        pair
    }

    #[inline]
    pub fn get_mut_with_key_with_context(
        &mut self,
        sparse_index: K::SparseIndex,
    ) -> (&V::Context, Option<(K, V::RefsMut<'_, '_>)>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.iter_mut_with_context();
        let pair = sparse_get_with_key(dense.map(From::from), sparse, sparse_index);
        (context, pair)
    }

    #[inline]
    pub fn into_get_mut_with_key(
        self,
        sparse_index: K::SparseIndex,
    ) -> Option<(K, V::RefsMut<'ctx, 'a>)> {
        let (_, pair) = self.into_get_mut_with_key_with_context(sparse_index);
        pair
    }

    #[inline]
    pub fn into_get_mut_with_key_with_context(
        self,
        sparse_index: K::SparseIndex,
    ) -> (&'ctx V::Context, Option<(K, V::RefsMut<'ctx, 'a>)>) {
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
    pub fn swap(&mut self, first_key: K, second_key: K) {
        let Self { dense, sparse } = self;

        let (context, slices) = dense.as_mut_slices_with_context();
        let (_, values) = slices.into_parts();
        let dense = SoaSlicesMut::<V>::new(context, values);

        let first_index = unwrap_into_usize(first_key.sparse_index());
        let second_index = unwrap_into_usize(second_key.sparse_index());
        if first_index == second_index {
            return;
        }

        let first_index = {
            let first_item = unwrap_sparse_item(sparse, first_index);
            check_equal_epoch(first_item.epoch, first_key.epoch());
            let first_index = unwrap_dense_index(first_item.kind());
            unwrap_into_usize(*first_index)
        };
        let second_index = {
            let second_item = unwrap_sparse_item(sparse, second_index);
            check_equal_epoch(second_item.epoch, second_key.epoch());
            let second_index = unwrap_dense_index(second_item.kind());
            unwrap_into_usize(*second_index)
        };

        let (first_value, second_value) = unwrap_dense_pair(dense, first_index, second_index);
        soa::mem::swap::<V>(context, first_value, second_value);
    }

    #[inline]
    pub fn swap_keys(&mut self, first_key: K, second_key: K) {
        let Self { dense, sparse } = self;
        let (keys, _) = dense.as_mut_slices().into_parts();

        let first_index = unwrap_into_usize(first_key.sparse_index());
        let second_index = unwrap_into_usize(second_key.sparse_index());
        if first_index == second_index {
            return;
        }

        let (first_item, second_item) =
            unwrap_sparse_items_pair_mut(sparse, first_index, second_index);

        let first_index = {
            check_equal_epoch(first_item.epoch, first_key.epoch());
            let first_index = unwrap_dense_index(first_item.kind());
            unwrap_into_usize(*first_index)
        };
        let second_index = {
            check_equal_epoch(second_item.epoch, second_key.epoch());
            let second_index = unwrap_dense_index(second_item.kind());
            unwrap_into_usize(*second_index)
        };

        let (first_key, second_key) = unwrap_dense_pair(keys, first_index, second_index);
        swap(first_item, second_item);
        swap(first_key, second_key);
    }

    pub fn invalidate_epoch(&mut self, key: K) -> Option<K> {
        let Self { dense, sparse } = self;

        let sparse_index = key.sparse_index();
        let sparse_item = sparse
            .get_mut(sparse_index.try_into().ok()?)
            .take_if(|item| item.epoch == key.epoch())?;
        let dense_index = unwrap_into_usize(*sparse_item.dense_index()?);

        let (keys, _) = dense.as_mut_slices().into_parts();
        let dense_key = unwrap_dense(keys, dense_index);
        check_equal_key(key, *dense_key);

        sparse_item.epoch = sparse_item.epoch.next();
        *dense_key = K::new(sparse_index, sparse_item.epoch);

        Some(*dense_key)
    }

    pub fn replace_key(&mut self, key: K) -> Option<K> {
        let Self { dense, sparse } = self;

        let sparse_index = key.sparse_index();
        let sparse_item = sparse
            .get_mut(sparse_index.try_into().ok()?)
            .take_if(|item| item.epoch == key.epoch())?;
        let dense_index = unwrap_into_usize(*sparse_item.dense_index()?);

        let (keys, _) = dense.as_mut_slices().into_parts();
        let dense_key = unwrap_dense(keys, dense_index);
        check_compatible_key(key, *dense_key);

        *dense_key = key;
        Some(*dense_key)
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        for<'ctx_any, 'any> V::Refs<'ctx_any, 'any>: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by_key(|&key| {
                let sparse_index = key.sparse_index();
                unwrap_dense_from_sparse_index(sparse_index, values.clone(), sparse)
            });
        });
    }

    #[inline]
    pub fn sort_keys_unstable(&mut self) {
        self.sort_impl(|keys, _, _| keys.sort_unstable());
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, mut f: F)
    where
        F: FnMut((K, V::Refs<'_, '_>), (K, V::Refs<'_, '_>)) -> cmp::Ordering,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by(|&lhs_key, &rhs_key| {
                let lhs_index = lhs_key.sparse_index();
                let lhs_value = unwrap_dense_from_sparse_index(lhs_index, values.clone(), sparse);
                let lhs = (lhs_key, lhs_value);

                let rhs_index = rhs_key.sparse_index();
                let rhs_value = unwrap_dense_from_sparse_index(rhs_index, values.clone(), sparse);
                let rhs = (rhs_key, rhs_value);

                f(lhs, rhs)
            });
        });
    }

    #[inline]
    pub fn sort_unstable_by_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, V::Refs<'_, '_>)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by_key(|&key| {
                let sparse_index = key.sparse_index();
                let value = unwrap_dense_from_sparse_index(sparse_index, values.clone(), sparse);
                f((key, value))
            });
        });
    }

    // Implementation was borrowed from the links below:
    // https://skypjack.github.io/2019-09-25-ecs-baf-part-5/#:~:text=Mixing%20in%2Dplace%20sorting%20and%20permutations
    // https://github.com/skypjack/entt/blob/8b0ef2b94234def2053c9a8a2591f4a5e87cf0ea/src/entt/entity/sparse_set.hpp#L964
    pub(crate) fn sort_impl<SortKeys>(&mut self, sort_keys: SortKeys)
    where
        SortKeys: FnOnce(&mut [K], SoaIter<V>, &[SparseItem<K>]),
    {
        let Self { dense, sparse } = self;

        let (context, slices) = dense.as_mut_slices_with_context();
        let (keys, values) = slices.into_parts();
        let mut values = SoaSlicesMut::<V>::new(context, values);

        sort_keys(keys, values.iter(), sparse);

        let keys = &keys[..];
        for pos in 0..keys.len() {
            let mut curr = pos;
            let mut next = {
                let sparse_index = unwrap_dense(keys, curr).sparse_index();
                let sparse_index = unwrap_into_usize(sparse_index);
                let sparse_item = unwrap_sparse_item(sparse, sparse_index);
                let dense_index = unwrap_dense_index(sparse_item.kind());
                unwrap_into_usize(*dense_index)
            };

            while curr != next {
                let (curr_item, next_item) = {
                    let first_index = unwrap_dense(keys, curr).sparse_index();
                    let first_index = unwrap_into_usize(first_index);
                    let second_index = unwrap_dense(keys, next).sparse_index();
                    let second_index = unwrap_into_usize(second_index);
                    unwrap_sparse_items_pair_mut(sparse, first_index, second_index)
                };
                let curr_dense_index = unwrap_dense_index_mut(curr_item.kind_mut());
                let next_dense_index = unwrap_dense_index_mut(next_item.kind_mut());
                values.swap(
                    unwrap_into_usize(*curr_dense_index),
                    unwrap_into_usize(*next_dense_index),
                );

                *curr_dense_index = unwrap_into_index(curr);
                curr = next;
                next = unwrap_into_usize(*next_dense_index);
            }
        }
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
    pub fn into_keys(self) -> Keys<'ctx, 'a, K, V> {
        let (_, iter) = self.into_keys_with_context();
        iter
    }

    #[inline]
    pub fn into_keys_with_context(self) -> (&'ctx V::Context, Keys<'ctx, 'a, K, V>) {
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
    pub fn into_values(self) -> Values<'ctx, 'a, K, V> {
        let (_, iter) = self.into_values_with_context();
        iter
    }

    #[inline]
    pub fn into_values_with_context(self) -> (&'ctx V::Context, Values<'ctx, 'a, K, V>) {
        let (context, iter) = self.into_raw_values_with_context();
        let iter = unsafe { iter.deref() };
        (context, iter)
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, '_, K, V> {
        let (_, iter) = self.values_mut_with_context();
        iter
    }

    #[inline]
    pub fn values_mut_with_context(&mut self) -> (&V::Context, ValuesMut<'_, '_, K, V>) {
        let (context, iter) = self.raw_values_mut_with_context();
        let iter = unsafe { iter.deref_mut() };
        (context, iter)
    }

    #[inline]
    pub fn into_values_mut(self) -> ValuesMut<'ctx, 'a, K, V> {
        let (_, iter) = self.into_values_mut_with_context();
        iter
    }

    #[inline]
    pub fn into_values_mut_with_context(self) -> (&'ctx V::Context, ValuesMut<'ctx, 'a, K, V>) {
        let (context, iter) = self.into_raw_values_mut_with_context();
        let iter = unsafe { iter.deref_mut() };
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
    pub fn iter_mut(&mut self) -> IterMut<'_, '_, K, V> {
        let (_, iter) = self.iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn iter_mut_with_context(&mut self) -> (&V::Context, IterMut<'_, '_, K, V>) {
        let (context, iter) = self.raw_iter_mut_with_context();
        let iter = unsafe { iter.deref_mut() };
        (context, iter)
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'ctx V::Context, IterMut<'ctx, 'a, K, V>) {
        let (context, iter) = self.into_raw_iter_mut_with_context();
        let iter = unsafe { iter.deref_mut() };
        (context, iter)
    }
}

impl<'ctx, 'a, K, V> Debug for EpochSparseViewMut<'ctx, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SparseItem<K>: Debug,
    SoaSlicesMut<'ctx, 'a, DenseItem<K, V>>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { dense, sparse } = self;
        f.debug_struct("EpochSparseViewMut")
            .field("dense", dense)
            .field("sparse", sparse)
            .finish()
    }
}

impl<'ctx, K, V> From<&'ctx V::Context> for EpochSparseViewMut<'ctx, '_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(context: &'ctx V::Context) -> Self {
        let view_mut_ptr = EpochSparseViewMutPtr::from(context);
        unsafe { view_mut_ptr.deref_mut() }
    }
}

impl<'ctx, 'a, K, V> PartialEq for EpochSparseViewMut<'ctx, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlicesMut<'ctx, 'a, DenseItem<K, V>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { dense, sparse } = self;
        *dense == other.dense && *sparse == other.sparse
    }
}

impl<'ctx, 'a, K, V> Eq for EpochSparseViewMut<'ctx, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlicesMut<'ctx, 'a, DenseItem<K, V>>: Eq,
{
}

impl<'ctx, 'a, K, V> PartialOrd for EpochSparseViewMut<'ctx, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlicesMut<'ctx, 'a, DenseItem<K, V>>: PartialOrd,
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

impl<'ctx, 'a, K, V> Ord for EpochSparseViewMut<'ctx, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlicesMut<'ctx, 'a, DenseItem<K, V>>: Ord,
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

impl<'ctx, 'a, K, V> Hash for EpochSparseViewMut<'ctx, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SparseItem<K>: Hash,
    SoaSlicesMut<'ctx, 'a, DenseItem<K, V>>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { dense, sparse } = self;
        dense.hash(state);
        sparse.hash(state);
    }
}

impl<T, K, V> Index<K> for EpochSparseViewMut<'_, '_, K, V>
where
    K: Key + Debug,
    V: Soa + ?Sized,
    for<'ctx, 'a> V: Soa<Refs<'ctx, 'a> = &'a T> + 'a,
{
    type Output = T;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        Self::index(self, key)
    }
}

impl<T, K, V> IndexMut<K> for EpochSparseViewMut<'_, '_, K, V>
where
    K: Key + Debug,
    V: Soa + ?Sized,
    for<'ctx, 'a> V: Soa<Refs<'ctx, 'a> = &'a T, RefsMut<'ctx, 'a> = &'a mut T> + 'a,
{
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        Self::index_mut(self, key)
    }
}

impl<T, K, V> AsRef<[T]> for EpochSparseViewMut<'_, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    for<'ctx, 'a> V::Slices<'ctx, 'a>: Into<&'a [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_value_slices().into()
    }
}

impl<T, K, V> AsMut<[T]> for EpochSparseViewMut<'_, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    for<'ctx, 'a> V::SlicesMut<'ctx, 'a>: Into<&'a mut [T]>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_value_slices().into()
    }
}

impl<K, V> AsRef<Self> for EpochSparseViewMut<'_, '_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<K, V> AsMut<Self> for EpochSparseViewMut<'_, '_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'a, K, V> IntoIterator for &'a EpochSparseViewMut<'_, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    type Item = (&'a K, V::Refs<'a, 'a>);
    type IntoIter = Iter<'a, 'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut EpochSparseViewMut<'_, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    type Item = (&'a K, V::RefsMut<'a, 'a>);
    type IntoIter = IterMut<'a, 'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'ctx, 'a, K, V> IntoIterator for EpochSparseViewMut<'ctx, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    type Item = (&'a K, V::RefsMut<'ctx, 'a>);
    type IntoIter = IterMut<'ctx, 'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}

impl<'ctx, 'a, K, V> From<EpochSparseViewMut<'ctx, 'a, K, V>> for EpochSparseView<'ctx, 'a, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: EpochSparseViewMut<'ctx, 'a, K, V>) -> Self {
        let (dense, sparse) = value.into_parts();
        let dense = dense.into();
        unsafe { Self::from_parts(dense, sparse) }
    }
}
