use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    algo::sparse_get_unchecked,
    item::{
        DefaultSparseItem, KeyValueMutPtrs, KeyValueMutSlicePtrs, KeyValuePair, KeyValuePtrs,
        KeyValueSlicePtrs, SparseItem,
    },
    iter::{RawIter, RawIterMut, RawKeys, RawValues, RawValuesMut},
    key::Key,
    soa::{
        identity::Identity,
        slice::SoaSliceMutPtrs,
        traits::{MutPtrs, Ptrs, RawSoa, SliceMutPtrs, SlicePtrs},
    },
    view::{EpochSparseView, EpochSparseViewMut, EpochSparseViewPtr},
};

pub type SparseViewMutPtr<'ctx, T, S = DefaultSparseItem<usize>> =
    EpochSparseViewMutPtr<'ctx, usize, T, S>;

pub struct EpochSparseViewMutPtr<'ctx, K, V, S = DefaultSparseItem<K>>
where
    K: Key,
    V: RawSoa<Context: 'ctx> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    dense: SoaSliceMutPtrs<'ctx, KeyValuePair<K, V>>,
    sparse: *mut [S],
}

impl<'ctx, K, V, S> EpochSparseViewMutPtr<'ctx, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    pub unsafe fn from_parts(
        dense: SoaSliceMutPtrs<'ctx, KeyValuePair<K, V>>,
        sparse: *mut [S],
    ) -> Self {
        Self { dense, sparse }
    }

    #[inline]
    pub fn into_parts(self) -> (SoaSliceMutPtrs<'ctx, KeyValuePair<K, V>>, *mut [S]) {
        let Self { dense, sparse } = self;
        (dense, sparse)
    }

    #[inline]
    pub fn cast_const(self) -> EpochSparseViewPtr<'ctx, K, V, S> {
        let Self { dense, sparse } = self;

        let dense = dense.cast_const();
        let sparse = sparse.cast_const();
        unsafe { EpochSparseViewPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> EpochSparseView<'ctx, 'a, K, V, S> {
        unsafe { self.cast_const().as_ref_unchecked() }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked<'a>(self) -> EpochSparseViewMut<'ctx, 'a, K, V, S> {
        let Self { dense, sparse } = self;

        let dense = unsafe { dense.as_mut_unchecked() };
        let sparse = unsafe { sparse.as_mut_unchecked() };
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
    pub fn as_ptrs(&self) -> (KeyValuePtrs<'_, K, V>, *const S) {
        let (_, dense, sparse) = self.as_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&V::Context, KeyValuePtrs<'_, K, V>, *const S) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.as_ptrs_with_context();
        let sparse = sparse.cast();
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

        let (context, dense) = dense.into_ptrs_with_context();
        let sparse = sparse.cast();
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
        let sparse = sparse.cast_const();
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

        let (context, dense) = dense.into_slice_ptrs_with_context();
        let sparse = sparse.cast_const();
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
        let (context, dense) = dense.into_mut_slice_ptrs_with_context();
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
    pub unsafe fn get_unchecked(&self, key: K) -> Ptrs<'_, V> {
        let (_, ptrs) = unsafe { self.get_unchecked_with_context(key) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_with_context(&self, key: K) -> (&V::Context, Ptrs<'_, V>) {
        let Self { ref dense, sparse } = *self;

        let (context, dense) = dense.iter_with_context();
        let dense = dense.map(From::from);
        let sparse_index = key.sparse_index();
        let (_, value) = unsafe { sparse_get_unchecked::<K, _, _>(dense, sparse, sparse_index) };
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
        let sparse_index = key.sparse_index();
        let (_, value) = unsafe { sparse_get_unchecked::<K, _, _>(dense, sparse, sparse_index) };
        (context, value)
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
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let dense = dense.cast_const().map(From::from);
        let sparse_index = key.sparse_index();
        let (_, value) = unsafe { sparse_get_unchecked::<K, _, _>(dense, sparse, sparse_index) };
        (context, value)
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
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let dense = dense.map(From::from);
        let sparse_index = key.sparse_index();
        let (_, value) = unsafe { sparse_get_unchecked::<K, _, _>(dense, sparse, sparse_index) };
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
        let (key, value) = unsafe { sparse_get_unchecked::<K, _, _>(dense, sparse, sparse_index) };
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
        let (key, value) = unsafe { sparse_get_unchecked::<K, _, _>(dense, sparse, sparse_index) };
        (context, key, value)
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
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let dense = dense.cast_const().map(From::from);
        let (key, value) = unsafe { sparse_get_unchecked::<K, _, _>(dense, sparse, sparse_index) };
        (context, key, value)
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
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let dense = dense.map(From::from);
        let (key, value) = unsafe { sparse_get_unchecked::<K, _, _>(dense, sparse, sparse_index) };
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

        let (context, slices) = dense.as_slice_ptrs_with_context();
        let (keys, _) = slices.into();
        let iter = RawKeys::new(context.as_inner(), keys);
        (context, iter)
    }

    #[inline]
    pub fn into_keys(self) -> RawKeys<'ctx, K, V> {
        let (_, iter) = self.into_keys_with_context();
        iter
    }

    #[inline]
    pub fn into_keys_with_context(self) -> (&'ctx V::Context, RawKeys<'ctx, K, V>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.into_slice_ptrs_with_context();
        let (keys, _) = slices.into();
        let iter = RawKeys::new(context.as_inner(), keys);
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
    pub fn into_values(self) -> RawValues<'ctx, K, V> {
        let (_, iter) = self.into_values_with_context();
        iter
    }

    #[inline]
    pub fn into_values_with_context(self) -> (&'ctx V::Context, RawValues<'ctx, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_iter_with_context();
        let iter = RawValues::from_inner(inner.cast_const());
        (context, iter)
    }

    #[inline]
    pub fn into_values_mut(self) -> RawValuesMut<'ctx, K, V> {
        let (_, iter) = self.into_values_mut_with_context();
        iter
    }

    #[inline]
    pub fn into_values_mut_with_context(self) -> (&'ctx V::Context, RawValuesMut<'ctx, K, V>) {
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
    pub fn into_iter_with_context(self) -> (&'ctx V::Context, RawIterMut<'ctx, K, V>) {
        let Self { dense, .. } = self;

        let (context, iter) = dense.into_iter_with_context();
        let iter = RawIterMut::from_inner(iter);
        (context, iter)
    }
}

impl<'ctx, K, V, S> From<&'ctx V::Context> for EpochSparseViewMutPtr<'ctx, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn from(context: &'ctx V::Context) -> Self {
        let context = Identity::from_inner_ref(context);
        let dense = SoaSliceMutPtrs::from(context);
        let sparse = ptr::from_mut(Default::default());
        unsafe { Self::from_parts(dense, sparse) }
    }
}

impl<'ctx, K, V, S> Default for EpochSparseViewMutPtr<'ctx, K, V, S>
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

impl<K, V, S> Debug for EpochSparseViewMutPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx> SlicePtrs<'ctx, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { dense, sparse } = self;

        f.debug_struct("EpochSparseViewMutPtr")
            .field("dense", dense)
            .field("sparse", sparse)
            .finish()
    }
}

impl<K, V, S> Clone for EpochSparseViewMutPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { ref dense, sparse } = *self;

        let dense = dense.clone();
        Self { dense, sparse }
    }
}

impl<K, V, S> Copy for EpochSparseViewMutPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx> MutPtrs<'ctx, V>: Copy,
{
}

impl<K, V, S> PartialEq for EpochSparseViewMutPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: PartialEq,
    for<'ctx> MutPtrs<'ctx, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse) == other
    }
}

impl<K, V, S> Eq for EpochSparseViewMutPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Eq,
    for<'ctx> MutPtrs<'ctx, V>: Eq,
{
}

impl<K, V, S> PartialOrd for EpochSparseViewMutPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: PartialOrd,
    for<'ctx> MutPtrs<'ctx, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse).partial_cmp(&other)
    }
}

impl<K, V, S> Ord for EpochSparseViewMutPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Ord,
    for<'ctx> MutPtrs<'ctx, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse).cmp(&other)
    }
}

impl<K, V, S> Hash for EpochSparseViewMutPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Hash,
    for<'ctx> MutPtrs<'ctx, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { dense, sparse } = self;
        (dense, sparse).hash(state);
    }
}

impl<'a, K, V, S> IntoIterator for &'a EpochSparseViewMutPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (*const K, Ptrs<'a, V>);
    type IntoIter = RawIter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut EpochSparseViewMutPtr<'_, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (*mut K, MutPtrs<'a, V>);
    type IntoIter = RawIterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'ctx, K, V, S> IntoIterator for EpochSparseViewMutPtr<'ctx, K, V, S>
where
    K: Key,
    V: RawSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (*mut K, MutPtrs<'ctx, V>);
    type IntoIter = RawIterMut<'ctx, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}
