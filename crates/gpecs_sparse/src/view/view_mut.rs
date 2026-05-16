use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem::swap,
    ops::{Index, IndexMut},
    ptr,
};

use crate::{
    algo::{
        check_parts, sparse_contains_key, sparse_get, sparse_get_epoch, sparse_get_with_key,
        sparse_index, sparse_item_by_key, sparse_item_mut_by_index, sparse_item_mut_by_key,
    },
    assert::{
        assert_compatible_key, assert_equal_epoch, assert_equal_key, assert_equal_sparse_index,
        unwrap_dense, unwrap_dense_from_sparse_index, unwrap_dense_index, unwrap_dense_pair,
        unwrap_into_index, unwrap_into_usize, unwrap_sparse_item, unwrap_sparse_pair,
    },
    error::FromPartsError,
    item::{
        DefaultSparseItem, KeyValueMutPtrs, KeyValueMutSlicePtrs, KeyValueMutSlices, KeyValuePair,
        KeyValuePtrs, KeyValueSlicePtrs, KeyValueSlices, SparseItem,
    },
    iter::{
        Iter, IterMut, Keys, RawIter, RawIterMut, RawKeys, RawValues, RawValuesMut, Values,
        ValuesMut,
    },
    key::{Epoch, Key},
    soa::{
        slice::{Iter as SoaIter, SoaSliceMutPtrs, SoaSlices, SoaSlicesMut},
        traits::{
            MutPtrs, Ptrs, RawSoa, RawSoaContext, Refs, RefsMut, SliceMutPtrs, SlicePtrs, Slices,
            SlicesMut, Soa, SoaContext, SoaOwned,
        },
    },
    view::{EpochSparseView, EpochSparseViewMutPtr, EpochSparseViewPtr},
};

pub type SparseViewMut<'ctx, 'a, T, S = DefaultSparseItem<usize>> =
    EpochSparseViewMut<'ctx, 'a, usize, T, S>;

pub struct EpochSparseViewMut<'ctx, 'a, K, V, S = DefaultSparseItem<K>>
where
    K: Key + 'a,
    V: RawSoa<Context: 'ctx> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    dense: SoaSlicesMut<'ctx, 'a, KeyValuePair<K, V>>,
    sparse: &'a mut [S],
}

impl<'ctx, 'a, K, V, S> EpochSparseViewMut<'ctx, 'a, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    pub fn new(
        dense: SoaSlicesMut<'ctx, 'a, KeyValuePair<K, V>>,
        sparse: &'a mut [S],
    ) -> Result<Self, FromPartsError<K>> {
        check_parts(dense.slices(), sparse)?;

        let me = unsafe { Self::from_parts(dense, sparse) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts(
        dense: SoaSlicesMut<'ctx, 'a, KeyValuePair<K, V>>,
        sparse: &'a mut [S],
    ) -> Self {
        Self { dense, sparse }
    }

    #[inline]
    pub fn into_parts(self) -> (SoaSlicesMut<'ctx, 'a, KeyValuePair<K, V>>, &'a mut [S]) {
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
    pub fn as_view_ptr(&self) -> EpochSparseViewPtr<'_, K, V, S> {
        let Self { dense, sparse } = self;

        let dense = dense.slice_ptrs();
        let sparse = ptr::from_ref(*sparse);
        unsafe { EpochSparseViewPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn into_view_ptr(self) -> EpochSparseViewPtr<'ctx, K, V, S> {
        let Self { dense, sparse } = self;

        let dense = dense.into_slice_ptrs();
        let sparse = ptr::from_ref(sparse);
        unsafe { EpochSparseViewPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn as_mut_view_ptr(&mut self) -> EpochSparseViewMutPtr<'_, K, V, S> {
        let Self { dense, sparse } = self;

        let dense = dense.mut_slice_ptrs();
        let sparse = ptr::from_mut(*sparse);
        unsafe { EpochSparseViewMutPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn into_mut_view_ptr(self) -> EpochSparseViewMutPtr<'ctx, K, V, S> {
        let Self { dense, sparse } = self;

        let dense = dense.into_mut_slice_ptrs();
        let sparse = ptr::from_mut(sparse);
        unsafe { EpochSparseViewMutPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn as_view(&self) -> EpochSparseView<'_, '_, K, V, S> {
        unsafe { self.as_view_ptr().as_ref_unchecked() }
    }

    #[inline]
    pub fn as_mut_view(&mut self) -> EpochSparseViewMut<'_, '_, K, V, S> {
        unsafe { self.as_mut_view_ptr().as_mut_unchecked() }
    }

    #[inline]
    pub fn as_ptrs(&self) -> (KeyValuePtrs<'_, K, V>, *const S) {
        let (_, dense, sparse) = self.as_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&V::Context, KeyValuePtrs<'_, K, V>, *const S) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.as_ptrs_with_context();
        let sparse = sparse.as_ptr();
        (context, dense, sparse)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (KeyValueMutPtrs<'_, K, V>, *mut S) {
        let (_, dense, sparse) = self.as_mut_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&V::Context, KeyValueMutPtrs<'_, K, V>, *mut S) {
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
    pub fn as_mut_key_ptr(&mut self) -> *mut K {
        let (_, key) = self.as_mut_key_ptr_with_context();
        key
    }

    #[inline]
    pub fn as_mut_key_ptr_with_context(&mut self) -> (&V::Context, *mut K) {
        let (context, dense) = self.as_mut_dense_ptrs_with_context();
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
    pub fn as_mut_value_ptrs(&mut self) -> MutPtrs<'_, V> {
        let (_, value) = self.as_mut_value_ptrs_with_context();
        value
    }

    #[inline]
    pub fn as_mut_value_ptrs_with_context(&mut self) -> (&V::Context, MutPtrs<'_, V>) {
        let (context, dense) = self.as_mut_dense_ptrs_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn as_dense_ptrs(&self) -> KeyValuePtrs<'_, K, V> {
        let (_, dense) = self.as_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_dense_ptrs_with_context(&self) -> (&V::Context, KeyValuePtrs<'_, K, V>) {
        let (context, dense, _) = self.as_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn as_mut_dense_ptrs(&mut self) -> KeyValueMutPtrs<'_, K, V> {
        let (_, dense) = self.as_mut_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_mut_dense_ptrs_with_context(&mut self) -> (&V::Context, KeyValueMutPtrs<'_, K, V>) {
        let (context, dense, _) = self.as_mut_ptrs_with_context();
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
    pub fn as_mut_sparse_ptr(&mut self) -> *mut S {
        let (_, sparse) = self.as_mut_sparse_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn as_mut_sparse_ptr_with_context(&mut self) -> (&V::Context, *mut S) {
        let (context, _, sparse) = self.as_mut_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn into_ptrs(self) -> (KeyValuePtrs<'ctx, K, V>, *const S) {
        let (_, dense, sparse) = self.into_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'ctx V::Context, KeyValuePtrs<'ctx, K, V>, *const S) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_slice_ptrs().into_ptrs_with_context();
        let sparse = sparse.as_ptr();
        (context, dense, sparse)
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> (KeyValueMutPtrs<'ctx, K, V>, *mut S) {
        let (_, dense, sparse) = self.into_mut_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_mut_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, KeyValueMutPtrs<'ctx, K, V>, *mut S) {
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
    pub fn into_mut_key_ptr(self) -> *mut K {
        let (_, key) = self.into_mut_key_ptr_with_context();
        key
    }

    #[inline]
    pub fn into_mut_key_ptr_with_context(self) -> (&'ctx V::Context, *mut K) {
        let (context, dense) = self.into_mut_dense_ptrs_with_context();
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
    pub fn into_mut_value_ptrs(self) -> MutPtrs<'ctx, V> {
        let (_, value) = self.into_mut_value_ptrs_with_context();
        value
    }

    #[inline]
    pub fn into_mut_value_ptrs_with_context(self) -> (&'ctx V::Context, MutPtrs<'ctx, V>) {
        let (context, dense) = self.into_mut_dense_ptrs_with_context();
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn into_dense_ptrs(self) -> KeyValuePtrs<'ctx, K, V> {
        let (_, dense) = self.into_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_ptrs_with_context(self) -> (&'ctx V::Context, KeyValuePtrs<'ctx, K, V>) {
        let (context, dense, _) = self.into_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_mut_dense_ptrs(self) -> KeyValueMutPtrs<'ctx, K, V> {
        let (_, dense) = self.into_mut_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_mut_dense_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, KeyValueMutPtrs<'ctx, K, V>) {
        let (context, dense, _) = self.into_mut_ptrs_with_context();
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
    pub fn into_mut_sparse_ptr(self) -> *mut S {
        let (_, sparse) = self.into_mut_sparse_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_mut_sparse_ptr_with_context(self) -> (&'ctx V::Context, *mut S) {
        let (context, _, sparse) = self.into_mut_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> (KeyValueSlicePtrs<'_, K, V>, *const [S]) {
        let (_, dense, sparse) = self.as_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(
        &self,
    ) -> (&V::Context, KeyValueSlicePtrs<'_, K, V>, *const [S]) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.as_slice_ptrs_with_context();
        let sparse = ptr::from_ref(*sparse);
        (context, dense, sparse)
    }

    #[inline]
    pub fn as_mut_slice_ptrs(&mut self) -> (KeyValueMutSlicePtrs<'_, K, V>, *mut [S]) {
        let (_, dense, sparse) = self.as_mut_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_mut_slice_ptrs_with_context(
        &mut self,
    ) -> (&V::Context, KeyValueMutSlicePtrs<'_, K, V>, *mut [S]) {
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
    pub fn as_dense_slice_ptrs(&self) -> KeyValueSlicePtrs<'_, K, V> {
        let (_, dense) = self.as_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_dense_slice_ptrs_with_context(&self) -> (&V::Context, KeyValueSlicePtrs<'_, K, V>) {
        let (context, dense, _) = self.as_slice_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn as_mut_dense_slice_ptrs(&mut self) -> KeyValueMutSlicePtrs<'_, K, V> {
        let (_, dense) = self.as_mut_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_mut_dense_slice_ptrs_with_context(
        &mut self,
    ) -> (&V::Context, KeyValueMutSlicePtrs<'_, K, V>) {
        let (context, dense, _) = self.as_mut_slice_ptrs_with_context();
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
    pub fn as_mut_sparse_slice_ptr(&mut self) -> *mut [S] {
        let (_, sparse) = self.as_mut_sparse_slice_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn as_mut_sparse_slice_ptr_with_context(&mut self) -> (&V::Context, *mut [S]) {
        let (context, _, sparse) = self.as_mut_slice_ptrs_with_context();
        (context, sparse)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> (KeyValueSlicePtrs<'ctx, K, V>, *const [S]) {
        let (_, dense, sparse) = self.into_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, KeyValueSlicePtrs<'ctx, K, V>, *const [S]) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_slice_ptrs().into_slice_ptrs_with_context();
        let sparse = ptr::from_ref(sparse);
        (context, dense, sparse)
    }

    #[inline]
    pub fn into_mut_slice_ptrs(self) -> (KeyValueMutSlicePtrs<'ctx, K, V>, *mut [S]) {
        let (_, dense, sparse) = self.into_mut_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_mut_slice_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, KeyValueMutSlicePtrs<'ctx, K, V>, *mut [S]) {
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
    pub fn into_dense_slice_ptrs(self) -> KeyValueSlicePtrs<'ctx, K, V> {
        let (_, dense) = self.into_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_slice_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, KeyValueSlicePtrs<'ctx, K, V>) {
        let (context, dense, _) = self.into_slice_ptrs_with_context();
        (context, dense)
    }

    #[inline]
    pub fn into_mut_dense_slice_ptrs(self) -> KeyValueMutSlicePtrs<'ctx, K, V> {
        let (_, dense) = self.into_mut_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_mut_dense_slice_ptrs_with_context(
        self,
    ) -> (&'ctx V::Context, KeyValueMutSlicePtrs<'ctx, K, V>) {
        let (context, dense, _) = self.into_mut_slice_ptrs_with_context();
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
    pub fn into_mut_sparse_slice_ptr(self) -> *mut [S] {
        let (_, sparse) = self.into_mut_sparse_slice_ptr_with_context();
        sparse
    }

    #[inline]
    pub fn into_mut_sparse_slice_ptr_with_context(self) -> (&'ctx V::Context, *mut [S]) {
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
        let keys = unsafe { keys.as_ref_unchecked() };
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
        let keys = unsafe { keys.as_mut_unchecked() };
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
    pub unsafe fn as_mut_sparse_slice(&mut self) -> &mut [S] {
        let (_, sparse) = unsafe { self.as_mut_sparse_slice_with_context() };
        sparse
    }

    #[inline]
    pub unsafe fn as_mut_sparse_slice_with_context(&mut self) -> (&V::Context, &mut [S]) {
        let (context, sparse) = self.as_mut_sparse_slice_ptr_with_context();
        let sparse = unsafe { sparse.as_mut_unchecked() };
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
    pub unsafe fn into_mut_key_slice(self) -> &'a mut [K] {
        let (_, keys) = unsafe { self.into_mut_key_slice_with_context() };
        keys
    }

    #[inline]
    pub unsafe fn into_mut_key_slice_with_context(self) -> (&'ctx V::Context, &'a mut [K]) {
        let (context, keys) = self.into_mut_key_slice_ptr_with_context();
        let keys = unsafe { keys.as_mut_unchecked() };
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
    pub unsafe fn into_mut_sparse_slice(self) -> &'a mut [S] {
        let (_, sparse) = unsafe { self.into_mut_sparse_slice_with_context() };
        sparse
    }

    #[inline]
    pub unsafe fn into_mut_sparse_slice_with_context(self) -> (&'ctx V::Context, &'a mut [S]) {
        let (context, sparse) = self.into_mut_sparse_slice_ptr_with_context();
        let sparse = unsafe { sparse.as_mut_unchecked() };
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
        let view_ptr = self.as_mut_view_ptr();
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
        let view_ptr = self.into_mut_view_ptr();
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
        let view_ptr = self.as_mut_view_ptr();
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
        let view_ptr = self.into_mut_view_ptr();
        unsafe { view_ptr.into_get_mut_with_key_unchecked_with_context(sparse_index) }
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

    pub fn invalidate_epoch(&mut self, key: K) -> Option<K> {
        let Self { dense, sparse } = self;

        let sparse_item = sparse_item_mut_by_key(sparse, key)?;
        let dense_index = sparse_item.dense_index()?;
        let dense_index_usize = unwrap_into_usize(dense_index);

        let (keys, _) = dense.as_mut_slice_ptrs().into_parts();
        let keys = unsafe { keys.as_mut_unchecked() };

        let dense_key = unwrap_dense(keys, dense_index_usize);
        assert_equal_key(key, *dense_key);

        let epoch = sparse_item.epoch().next();
        *sparse_item = S::occupied(epoch, dense_index);
        *dense_key = K::new(key.sparse_index(), epoch);

        Some(*dense_key)
    }

    pub unsafe fn replace_epoch(
        &mut self,
        sparse_index: K::SparseIndex,
        epoch: K::Epoch,
    ) -> Option<K> {
        let Self { dense, sparse } = self;

        let sparse_item = sparse_item_mut_by_index::<K, _>(sparse, sparse_index)?;
        let dense_index = sparse_item.dense_index()?;
        let dense_index_usize = unwrap_into_usize(dense_index);

        let (keys, _) = dense.as_mut_slice_ptrs().into_parts();
        let keys = unsafe { keys.as_mut_unchecked() };

        let dense_key = unwrap_dense(keys, dense_index_usize);
        assert_equal_sparse_index(sparse_index, dense_key.sparse_index());

        *sparse_item = S::occupied(epoch, dense_index);
        *dense_key = K::new(sparse_index, epoch);

        Some(*dense_key)
    }

    pub fn replace_key(&mut self, key: K) -> Option<K> {
        let Self { dense, sparse } = self;

        let dense_index = sparse_item_by_key(sparse, key)
            .copied()
            .and_then(S::dense_index)?;
        let dense_index = unwrap_into_usize(dense_index);

        let (keys, _) = dense.as_mut_slice_ptrs().into_parts();
        let keys = unsafe { keys.as_mut_unchecked() };

        let dense_key = unwrap_dense(keys, dense_index);
        assert_compatible_key(key, *dense_key);

        *dense_key = key;
        Some(*dense_key)
    }

    #[inline]
    pub fn swap(&mut self, first_key: K, second_key: K) {
        let Self { dense, sparse } = self;

        let (context, slices) = dense.as_mut_slice_ptrs_with_context();
        let (_, values) = slices.into_parts();
        let dense = SoaSliceMutPtrs::<V>::new(context, values);

        let first_index = unwrap_into_usize(first_key.sparse_index());
        let second_index = unwrap_into_usize(second_key.sparse_index());
        if first_index == second_index {
            return;
        }

        let first_index = {
            let first_item = unwrap_sparse_item(sparse, first_index);
            assert_equal_epoch(first_item.epoch(), first_key.epoch());
            let first_index = unwrap_dense_index(first_item);
            unwrap_into_usize(first_index)
        };
        let second_index = {
            let second_item = unwrap_sparse_item(sparse, second_index);
            assert_equal_epoch(second_item.epoch(), second_key.epoch());
            let second_index = unwrap_dense_index(second_item);
            unwrap_into_usize(second_index)
        };

        let (first_value, second_value) = unwrap_dense_pair(dense, first_index, second_index);
        unsafe { context.as_inner().ptrs_swap(first_value, second_value) }
    }

    #[inline]
    pub fn swap_keys(&mut self, first_key: K, second_key: K) {
        let Self { dense, sparse } = self;
        let (keys, _) = dense.as_mut_slice_ptrs().into_parts();
        let keys = unsafe { keys.as_mut_unchecked() };

        let first_index = unwrap_into_usize(first_key.sparse_index());
        let second_index = unwrap_into_usize(second_key.sparse_index());
        if first_index == second_index {
            return;
        }

        let (first_item, second_item) =
            unwrap_sparse_pair(sparse.iter_mut(), first_index, second_index);

        let first_index = {
            assert_equal_epoch(first_item.epoch(), first_key.epoch());
            let first_index = unwrap_dense_index(first_item);
            unwrap_into_usize(first_index)
        };
        let second_index = {
            assert_equal_epoch(second_item.epoch(), second_key.epoch());
            let second_index = unwrap_dense_index(second_item);
            unwrap_into_usize(second_index)
        };

        let (first_key, second_key) = unwrap_dense_pair(keys, first_index, second_index);
        swap(first_item, second_item);
        swap(first_key, second_key);
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

        let (context, slices) = dense.into_slice_ptrs().into_slice_ptrs_with_context();
        let (keys, _) = slices.into();
        let iter = RawKeys::new(context.as_inner(), keys);
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

impl<'ctx, 'a, K, V, S> EpochSparseViewMut<'ctx, 'a, K, V, S>
where
    K: Key,
    V: Soa<'a> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    pub fn into_slices(self) -> (KeyValueSlices<'ctx, 'a, K, V>, &'a [S]) {
        let (_, dense, sparse) = self.into_slices_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_slices_with_context(
        self,
    ) -> (&'ctx V::Context, KeyValueSlices<'ctx, 'a, K, V>, &'a [S]) {
        let (context, dense, sparse) = self.into_slice_ptrs_with_context();
        let dense = unsafe { dense.as_ref_unchecked(context) };
        let sparse = unsafe { sparse.as_ref_unchecked() };
        (context, dense, sparse)
    }

    #[inline]
    pub unsafe fn into_mut_slices(self) -> (KeyValueMutSlices<'ctx, 'a, K, V>, &'a mut [S]) {
        let (_, dense, sparse) = unsafe { self.into_mut_slices_with_context() };
        (dense, sparse)
    }

    #[inline]
    pub unsafe fn into_mut_slices_with_context(
        self,
    ) -> (
        &'ctx V::Context,
        KeyValueMutSlices<'ctx, 'a, K, V>,
        &'a mut [S],
    ) {
        let (context, dense, sparse) = self.into_mut_slice_ptrs_with_context();
        let dense = unsafe { dense.as_mut_unchecked(context) };
        let sparse = unsafe { sparse.as_mut_unchecked() };
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
    pub fn into_mut_value_slices(self) -> SlicesMut<'ctx, 'a, V> {
        let (_, value) = self.into_mut_value_slices_with_context();
        value
    }

    #[inline]
    pub fn into_mut_value_slices_with_context(self) -> (&'ctx V::Context, SlicesMut<'ctx, 'a, V>) {
        let (context, dense) = unsafe { self.into_mut_dense_slices_with_context() };
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn into_dense_slices(self) -> KeyValueSlices<'ctx, 'a, K, V> {
        let (_, dense) = self.into_dense_slices_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_slices_with_context(
        self,
    ) -> (&'ctx V::Context, KeyValueSlices<'ctx, 'a, K, V>) {
        let (context, dense, _) = self.into_slices_with_context();
        (context, dense)
    }

    #[inline]
    pub unsafe fn into_mut_dense_slices(self) -> KeyValueMutSlices<'ctx, 'a, K, V> {
        let (_, dense) = unsafe { self.into_mut_dense_slices_with_context() };
        dense
    }

    #[inline]
    pub unsafe fn into_mut_dense_slices_with_context(
        self,
    ) -> (&'ctx V::Context, KeyValueMutSlices<'ctx, 'a, K, V>) {
        let (context, dense, _) = unsafe { self.into_mut_slices_with_context() };
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

        let (context, dense) = SoaSlices::from(dense).into_iter_with_context();
        let refs = sparse_get(dense.map(From::from), sparse, key);
        (context, refs)
    }

    #[inline]
    pub fn into_get_mut(self, key: K) -> Option<RefsMut<'ctx, 'a, V>> {
        let (_, refs) = self.into_get_mut_with_context(key);
        refs
    }

    #[inline]
    pub fn into_get_mut_with_context(
        self,
        key: K,
    ) -> (&'ctx V::Context, Option<RefsMut<'ctx, 'a, V>>) {
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

        let (context, dense) = SoaSlices::from(dense).into_iter_with_context();
        let refs = sparse_index(dense.map(From::from), sparse, key);
        (context, refs)
    }

    #[inline]
    pub fn into_index_mut(self, key: K) -> RefsMut<'ctx, 'a, V>
    where
        K: Debug,
    {
        let (_, refs) = self.into_index_mut_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_mut_with_context(self, key: K) -> (&'ctx V::Context, RefsMut<'ctx, 'a, V>)
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
        let pair = sparse_get_with_key(dense.map(From::from), sparse, sparse_index)
            .map(|(key, value)| (key, context.as_inner().mut_refs_as_refs(value)));
        (context, pair)
    }

    #[inline]
    pub fn into_get_mut_with_key(
        self,
        sparse_index: K::SparseIndex,
    ) -> Option<(K, RefsMut<'ctx, 'a, V>)> {
        let (_, pair) = self.into_get_mut_with_key_with_context(sparse_index);
        pair
    }

    #[inline]
    pub fn into_get_mut_with_key_with_context(
        self,
        sparse_index: K::SparseIndex,
    ) -> (&'ctx V::Context, Option<(K, RefsMut<'ctx, 'a, V>)>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let pair = sparse_get_with_key(dense.map(From::from), sparse, sparse_index);
        (context, pair)
    }

    #[inline]
    pub fn into_values(self) -> Values<'ctx, 'a, K, V> {
        let (_, iter) = self.into_values_with_context();
        iter
    }

    #[inline]
    pub fn into_values_with_context(self) -> (&'ctx V::Context, Values<'ctx, 'a, K, V>) {
        let (context, iter) = self.into_raw_values_with_context();
        let iter = unsafe { iter.as_ref_unchecked() };
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
        let iter = unsafe { iter.as_mut_unchecked() };
        (context, iter)
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'ctx V::Context, IterMut<'ctx, 'a, K, V>) {
        let (context, iter) = self.into_raw_iter_mut_with_context();
        let iter = unsafe { iter.as_mut_unchecked() };
        (context, iter)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn into_par_iter(self) -> crate::iter::ParIterMut<'ctx, 'a, K, V> {
        let (_, iter) = self.into_par_iter_with_context();
        iter
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn into_par_iter_with_context(
        self,
    ) -> (&'ctx V::Context, crate::iter::ParIterMut<'ctx, 'a, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_par_iter_with_context();
        let iter = crate::iter::ParIterMut::new(inner);
        (context, iter)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn into_par_values_mut(self) -> crate::iter::ParValuesMut<'ctx, 'a, K, V> {
        let (_, iter) = self.into_par_values_mut_with_context();
        iter
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn into_par_values_mut_with_context(
        self,
    ) -> (&'ctx V::Context, crate::iter::ParValuesMut<'ctx, 'a, K, V>) {
        let (context, inner) = self.into_par_iter_with_context();
        let values = crate::iter::ParValuesMut::new(inner);
        (context, values)
    }
}

impl<'a, K, V, S> EpochSparseViewMut<'_, '_, K, V, S>
where
    K: Key,
    V: Soa<'a> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    pub fn as_slices(&'a self) -> (KeyValueSlices<'a, 'a, K, V>, &'a [S]) {
        let (_, dense, sparse) = self.as_slices_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_slices_with_context(
        &'a self,
    ) -> (&'a V::Context, KeyValueSlices<'a, 'a, K, V>, &'a [S]) {
        let (context, dense, sparse) = self.as_slice_ptrs_with_context();
        let dense = unsafe { dense.as_ref_unchecked(context) };
        let sparse = unsafe { sparse.as_ref_unchecked() };
        (context, dense, sparse)
    }

    #[inline]
    pub unsafe fn as_mut_slices(&'a mut self) -> (KeyValueMutSlices<'a, 'a, K, V>, &'a mut [S]) {
        let (_, dense, sparse) = unsafe { self.as_mut_slices_with_context() };
        (dense, sparse)
    }

    #[inline]
    pub unsafe fn as_mut_slices_with_context(
        &'a mut self,
    ) -> (&'a V::Context, KeyValueMutSlices<'a, 'a, K, V>, &'a mut [S]) {
        let (context, dense, sparse) = self.as_mut_slice_ptrs_with_context();
        let dense = unsafe { dense.as_mut_unchecked(context) };
        let sparse = unsafe { sparse.as_mut_unchecked() };
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
    pub fn as_mut_value_slices(&'a mut self) -> SlicesMut<'a, 'a, V> {
        let (_, value) = self.as_mut_value_slices_with_context();
        value
    }

    #[inline]
    pub fn as_mut_value_slices_with_context(
        &'a mut self,
    ) -> (&'a V::Context, SlicesMut<'a, 'a, V>) {
        let (context, dense) = unsafe { self.as_mut_dense_slices_with_context() };
        let (_, value) = dense.into_parts();
        (context, value)
    }

    #[inline]
    pub fn as_dense_slices(&'a self) -> KeyValueSlices<'a, 'a, K, V> {
        let (_, dense) = self.as_dense_slices_with_context();
        dense
    }

    #[inline]
    pub fn as_dense_slices_with_context(
        &'a self,
    ) -> (&'a V::Context, KeyValueSlices<'a, 'a, K, V>) {
        let (context, dense, _) = self.as_slices_with_context();
        (context, dense)
    }

    #[inline]
    pub unsafe fn as_mut_dense_slices(&'a mut self) -> KeyValueMutSlices<'a, 'a, K, V> {
        let (_, dense) = unsafe { self.as_mut_dense_slices_with_context() };
        dense
    }

    #[inline]
    pub unsafe fn as_mut_dense_slices_with_context(
        &'a mut self,
    ) -> (&'a V::Context, KeyValueMutSlices<'a, 'a, K, V>) {
        let (context, dense, _) = unsafe { self.as_mut_slices_with_context() };
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
    pub fn get_mut(&'a mut self, key: K) -> Option<RefsMut<'a, 'a, V>> {
        let (_, refs) = self.get_mut_with_context(key);
        refs
    }

    #[inline]
    pub fn get_mut_with_context(
        &'a mut self,
        key: K,
    ) -> (&'a V::Context, Option<RefsMut<'a, 'a, V>>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.iter_mut_with_context();
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
    #[track_caller]
    pub fn index_mut(&'a mut self, key: K) -> RefsMut<'a, 'a, V>
    where
        K: Debug,
    {
        let (_, refs) = self.index_mut_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_mut_with_context(&'a mut self, key: K) -> (&'a V::Context, RefsMut<'a, 'a, V>)
    where
        K: Debug,
    {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.iter_mut_with_context();
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
    pub fn get_mut_with_key(
        &'a mut self,
        sparse_index: K::SparseIndex,
    ) -> Option<(K, RefsMut<'a, 'a, V>)> {
        let (_, pair) = self.get_mut_with_key_with_context(sparse_index);
        pair
    }

    #[inline]
    pub fn get_mut_with_key_with_context(
        &'a mut self,
        sparse_index: K::SparseIndex,
    ) -> (&'a V::Context, Option<(K, RefsMut<'a, 'a, V>)>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.iter_mut_with_context();
        let pair = sparse_get_with_key(dense.map(From::from), sparse, sparse_index);
        (context, pair)
    }

    #[inline]
    pub fn values(&'a self) -> Values<'a, 'a, K, V> {
        let (_, iter) = self.values_with_context();
        iter
    }

    #[inline]
    pub fn values_with_context(&'a self) -> (&'a V::Context, Values<'a, 'a, K, V>) {
        let (context, iter) = self.raw_values_with_context();
        let iter = unsafe { iter.as_ref_unchecked() };
        (context, iter)
    }

    #[inline]
    pub fn values_mut(&'a mut self) -> ValuesMut<'a, 'a, K, V> {
        let (_, iter) = self.values_mut_with_context();
        iter
    }

    #[inline]
    pub fn values_mut_with_context(&'a mut self) -> (&'a V::Context, ValuesMut<'a, 'a, K, V>) {
        let (context, iter) = self.raw_values_mut_with_context();
        let iter = unsafe { iter.as_mut_unchecked() };
        (context, iter)
    }

    #[inline]
    pub fn iter(&'a self) -> Iter<'a, 'a, K, V> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&'a self) -> (&'a V::Context, Iter<'a, 'a, K, V>) {
        let (context, iter) = self.raw_iter_with_context();
        let iter = unsafe { iter.as_ref_unchecked() };
        (context, iter)
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> IterMut<'a, 'a, K, V> {
        let (_, iter) = self.iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn iter_mut_with_context(&'a mut self) -> (&'a V::Context, IterMut<'a, 'a, K, V>) {
        let (context, iter) = self.raw_iter_mut_with_context();
        let iter = unsafe { iter.as_mut_unchecked() };
        (context, iter)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter(&'a self) -> crate::iter::ParIter<'a, 'a, K, V> {
        let (_, iter) = self.par_iter_with_context();
        iter
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter_with_context(&'a self) -> (&'a V::Context, crate::iter::ParIter<'a, 'a, K, V>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.slices_with_context();
        let iter = crate::iter::ParIter::new(slices.into_par_iter());
        (context, iter)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter_mut(&'a mut self) -> crate::iter::ParIterMut<'a, 'a, K, V> {
        let (_, iter) = self.par_iter_mut_with_context();
        iter
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter_mut_with_context(
        &'a mut self,
    ) -> (&'a V::Context, crate::iter::ParIterMut<'a, 'a, K, V>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.mut_slices_with_context();
        let iter = crate::iter::ParIterMut::new(slices.into_par_iter());
        (context, iter)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_values(&'a self) -> crate::iter::ParValues<'a, 'a, K, V> {
        let (_, iter) = self.par_values_with_context();
        iter
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_values_with_context(
        &'a self,
    ) -> (&'a V::Context, crate::iter::ParValues<'a, 'a, K, V>) {
        let (context, inner) = self.par_iter_with_context();
        let values = crate::iter::ParValues::new(inner);
        (context, values)
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_values_mut(&'a mut self) -> crate::iter::ParValuesMut<'a, 'a, K, V> {
        let (_, iter) = self.par_values_mut_with_context();
        iter
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_values_mut_with_context(
        &'a mut self,
    ) -> (&'a V::Context, crate::iter::ParValuesMut<'a, 'a, K, V>) {
        let (context, inner) = self.par_iter_mut_with_context();
        let values = crate::iter::ParValuesMut::new(inner);
        (context, values)
    }
}

impl<K, V, S> EpochSparseViewMut<'_, '_, K, V, S>
where
    K: Key,
    V: SoaOwned + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    pub fn sort_unstable(&mut self)
    where
        for<'ctx, 'a> Refs<'ctx, 'a, V>: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by_key(|&key| {
                let sparse_index = key.sparse_index();
                let dense = values.clone();
                unwrap_dense_from_sparse_index::<K, _>(sparse_index, dense, sparse)
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
        for<'a> F: FnMut((K, Refs<'_, 'a, V>), (K, Refs<'_, 'a, V>)) -> cmp::Ordering,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by(|&lhs_key, &rhs_key| {
                let dense = values.clone();
                let lhs_index = lhs_key.sparse_index();
                let lhs_value = unwrap_dense_from_sparse_index::<K, _>(lhs_index, dense, sparse);
                let lhs = (lhs_key, lhs_value);

                let dense = values.clone();
                let rhs_index = rhs_key.sparse_index();
                let rhs_value = unwrap_dense_from_sparse_index::<K, _>(rhs_index, dense, sparse);
                let rhs = (rhs_key, rhs_value);

                f(lhs, rhs)
            });
        });
    }

    #[inline]
    pub fn sort_unstable_by_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, Refs<'_, '_, V>)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by_key(|&key| {
                let sparse_index = key.sparse_index();
                let dense = values.clone();
                let value = unwrap_dense_from_sparse_index::<K, _>(sparse_index, dense, sparse);
                f((key, value))
            });
        });
    }

    // Implementation was borrowed from the links below:
    // https://skypjack.github.io/2019-09-25-ecs-baf-part-5/#:~:text=Mixing%20in%2Dplace%20sorting%20and%20permutations
    // https://github.com/skypjack/entt/blob/8b0ef2b94234def2053c9a8a2591f4a5e87cf0ea/src/entt/entity/sparse_set.hpp#L964
    pub(crate) fn sort_impl<SortKeys>(&mut self, sort_keys: SortKeys)
    where
        SortKeys: FnOnce(&mut [K], SoaIter<V>, &[S]),
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
                let dense_index = unwrap_dense_index(sparse_item);
                unwrap_into_usize(dense_index)
            };

            while curr != next {
                let (curr_item, next_item) = {
                    let first_index = unwrap_dense(keys, curr).sparse_index();
                    let first_index = unwrap_into_usize(first_index);
                    let second_index = unwrap_dense(keys, next).sparse_index();
                    let second_index = unwrap_into_usize(second_index);
                    unwrap_sparse_pair(sparse.iter_mut(), first_index, second_index)
                };
                let curr_dense_index = unwrap_dense_index(curr_item);
                let next_dense_index = unwrap_dense_index(next_item);
                values.swap(
                    unwrap_into_usize(curr_dense_index),
                    unwrap_into_usize(next_dense_index),
                );

                *curr_item = S::occupied(curr_item.epoch(), unwrap_into_index(curr));
                curr = next;
                next = unwrap_into_usize(next_dense_index);
            }
        }
    }
}

impl<'ctx, 'a, K, V, S> Debug for EpochSparseViewMut<'ctx, 'a, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Debug,
    SoaSlicesMut<'ctx, 'a, KeyValuePair<K, V>>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { dense, sparse } = self;

        f.debug_struct("EpochSparseViewMut")
            .field("dense", dense)
            .field("sparse", sparse)
            .finish()
    }
}

impl<'ctx, K, V, S> From<&'ctx V::Context> for EpochSparseViewMut<'ctx, '_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn from(context: &'ctx V::Context) -> Self {
        let view_mut_ptr = EpochSparseViewMutPtr::from(context);
        unsafe { view_mut_ptr.as_mut_unchecked() }
    }
}

impl<'ctx, K, V, S> Default for EpochSparseViewMut<'ctx, '_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    &'ctx V::Context: Default,
{
    #[inline]
    fn default() -> Self {
        let context: &V::Context = Default::default();
        Self::from(context)
    }
}

impl<'ctx, 'a, K, V, S> PartialEq for EpochSparseViewMut<'ctx, 'a, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + PartialEq,
    SoaSlicesMut<'ctx, 'a, KeyValuePair<K, V>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse) == other
    }
}

impl<'ctx, 'a, K, V, S> Eq for EpochSparseViewMut<'ctx, 'a, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Eq,
    SoaSlicesMut<'ctx, 'a, KeyValuePair<K, V>>: Eq,
{
}

impl<'ctx, 'a, K, V, S> PartialOrd for EpochSparseViewMut<'ctx, 'a, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + PartialOrd,
    SoaSlicesMut<'ctx, 'a, KeyValuePair<K, V>>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse).partial_cmp(&other)
    }
}

impl<'ctx, 'a, K, V, S> Ord for EpochSparseViewMut<'ctx, 'a, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Ord,
    SoaSlicesMut<'ctx, 'a, KeyValuePair<K, V>>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse).cmp(&other)
    }
}

impl<'ctx, 'a, K, V, S> Hash for EpochSparseViewMut<'ctx, 'a, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Hash,
    SoaSlicesMut<'ctx, 'a, KeyValuePair<K, V>>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { dense, sparse } = self;
        (dense, sparse).hash(state);
    }
}

impl<T, K, V, S> Index<K> for EpochSparseViewMut<'_, '_, K, V, S>
where
    K: Key + Debug,
    V: SoaOwned + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx, 'a> V::Context: SoaContext<'a, V, Refs<'ctx> = &'a T>,
{
    type Output = T;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        Self::index(self, key)
    }
}

impl<T, K, V, S> IndexMut<K> for EpochSparseViewMut<'_, '_, K, V, S>
where
    K: Key + Debug,
    V: SoaOwned + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx, 'a> V::Context: SoaContext<'a, V, Refs<'ctx> = &'a T, RefsMut<'ctx> = &'a mut T>,
{
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        Self::index_mut(self, key)
    }
}

impl<T, K, V, S> AsRef<[T]> for EpochSparseViewMut<'_, '_, K, V, S>
where
    K: Key,
    V: SoaOwned + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Into<&'a [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_value_slices().into()
    }
}

impl<T, K, V, S> AsMut<[T]> for EpochSparseViewMut<'_, '_, K, V, S>
where
    K: Key,
    V: SoaOwned + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Into<&'a mut [T]>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_value_slices().into()
    }
}

impl<K, V, S> AsRef<Self> for EpochSparseViewMut<'_, '_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<K, V, S> AsMut<Self> for EpochSparseViewMut<'_, '_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'a, K, V, S> IntoIterator for &'a EpochSparseViewMut<'_, '_, K, V, S>
where
    K: Key,
    V: Soa<'a> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (&'a K, Refs<'a, 'a, V>);
    type IntoIter = Iter<'a, 'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut EpochSparseViewMut<'_, '_, K, V, S>
where
    K: Key,
    V: Soa<'a> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (&'a K, RefsMut<'a, 'a, V>);
    type IntoIter = IterMut<'a, 'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'ctx, 'a, K, V, S> IntoIterator for EpochSparseViewMut<'ctx, 'a, K, V, S>
where
    K: Key,
    V: Soa<'a> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (&'a K, RefsMut<'ctx, 'a, V>);
    type IntoIter = IterMut<'ctx, 'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}

#[cfg(feature = "rayon")]
impl<'a, K, V, S> rayon::iter::IntoParallelIterator for &'a EpochSparseViewMut<'_, '_, K, V, S>
where
    K: Key + Sync,
    V: Soa<'a> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Sync,
    V::Fields: Sync,
    Refs<'a, 'a, V>: Send,
{
    type Item = (&'a K, Refs<'a, 'a, V>);
    type Iter = crate::iter::ParIter<'a, 'a, K, V>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a, K, V, S> rayon::iter::IntoParallelIterator for &'a mut EpochSparseViewMut<'_, '_, K, V, S>
where
    K: Key + Send + Sync,
    V: Soa<'a> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Sync,
    V::Fields: Send,
    RefsMut<'a, 'a, V>: Send,
{
    type Item = (&'a K, RefsMut<'a, 'a, V>);
    type Iter = crate::iter::ParIterMut<'a, 'a, K, V>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.par_iter_mut()
    }
}

#[cfg(feature = "rayon")]
impl<'ctx, 'a, K, V, S> rayon::iter::IntoParallelIterator for EpochSparseViewMut<'ctx, 'a, K, V, S>
where
    K: Key + Send + Sync,
    V: Soa<'a> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Sync,
    V::Fields: Send,
    RefsMut<'ctx, 'a, V>: Send,
{
    type Item = (&'a K, RefsMut<'ctx, 'a, V>);
    type Iter = crate::iter::ParIterMut<'ctx, 'a, K, V>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.into_par_iter()
    }
}

impl<'ctx, 'a, K, V, S> From<EpochSparseViewMut<'ctx, 'a, K, V, S>>
    for EpochSparseView<'ctx, 'a, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn from(value: EpochSparseViewMut<'ctx, 'a, K, V, S>) -> Self {
        let (dense, sparse) = value.into_parts();
        let dense = dense.into();
        unsafe { Self::from_parts(dense, sparse) }
    }
}
