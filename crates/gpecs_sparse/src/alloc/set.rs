use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::{Index, IndexMut},
    ptr, slice,
};
use core_alloc::vec::Vec;

use crate::{
    algo::{check_parts, dense_keys, sparse_item_by_epoch},
    assert::{
        assert_dense_index_bounds, assert_equal_key, assert_key_bounds, unwrap_dense,
        unwrap_dense_index, unwrap_into_index, unwrap_into_usize, unwrap_sparse_item_mut,
    },
    error::{
        FromPartsError, TooLargeSparseIndexError, TooSmallSparseIndexError, TryModifyError,
        TryModifyErrorKind, TryReserveError,
    },
    item::{
        self, ArenaSparseItem, KeyValueMutPtrs, KeyValueMutSlicePtrs, KeyValueMutSlices,
        KeyValuePair, KeyValuePtrs, KeyValueSlicePtrs, KeyValueSlices, SparseItem,
    },
    iter::{
        Drain, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, RawIter, RawIterMut, RawKeys,
        RawValues, RawValuesMut, Values, ValuesMut,
    },
    key::{Epoch, Key},
    soa::{
        self,
        traits::{
            AllocSoa, MutPtrs, Ptrs, RawSoaContext, ReadSoaContext, Refs, RefsMut, SliceMutPtrs,
            SlicePtrs, Slices, SlicesMut, Soa, SoaContext, SoaOwned, SoaRead, SoaReadOwned,
            SoaWrite, WriteSoaContext,
        },
        vec::SoaVec,
    },
    view::{EpochSparseView, EpochSparseViewMut, EpochSparseViewMutPtr, EpochSparseViewPtr},
};

use super::{
    access::TryInsertAccess,
    arena,
    assert::{try_entry_failed, try_insert_failed, try_push_failed},
    entry::generate_entry_types,
};

pub type SparseSet<T, S = item::DefaultSparseItem<usize>> = EpochSparseSet<usize, T, S>;

pub struct EpochSparseSet<K, V, S = item::DefaultSparseItem<K>>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    dense: SoaVec<KeyValuePair<K, V>>,
    sparse: Vec<S>,
}

impl<K, V, S> EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    #[must_use]
    pub fn new() -> Self
    where
        V::Context: Default,
    {
        Self {
            dense: SoaVec::new(),
            sparse: Vec::new(),
        }
    }

    #[inline]
    #[must_use]
    pub fn with_context(context: V::Context) -> Self {
        Self {
            dense: SoaVec::with_context(context.into()),
            sparse: Vec::new(),
        }
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(dense: usize, sparse: usize) -> Self
    where
        V::Context: Default,
    {
        Self {
            dense: SoaVec::with_capacity(dense),
            sparse: Vec::with_capacity(sparse),
        }
    }

    #[inline]
    #[must_use]
    pub fn with_context_and_capacity(context: V::Context, dense: usize, sparse: usize) -> Self {
        Self {
            dense: SoaVec::with_context_and_capacity(context.into(), dense),
            sparse: Vec::with_capacity(sparse),
        }
    }

    #[inline]
    pub fn try_with_capacity(dense: usize, sparse: usize) -> Result<Self, TryReserveError>
    where
        V::Context: Default,
    {
        let mut me = Self::new();
        me.try_reserve(dense, sparse)?;
        Ok(me)
    }

    #[inline]
    pub fn try_with_context_and_capacity(
        context: V::Context,
        dense: usize,
        sparse: usize,
    ) -> Result<Self, TryReserveError> {
        let mut me = Self::with_context(context);
        me.try_reserve(dense, sparse)?;
        Ok(me)
    }

    #[inline]
    pub fn from_parts(
        dense: SoaVec<KeyValuePair<K, V>>,
        sparse: Vec<S>,
    ) -> Result<Self, FromPartsError<K>> {
        check_parts(dense.slices(), sparse.as_slice())?;

        let me = unsafe { Self::from_parts_unchecked(dense, sparse) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts_unchecked(dense: SoaVec<KeyValuePair<K, V>>, sparse: Vec<S>) -> Self {
        Self { dense, sparse }
    }

    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (SoaVec<KeyValuePair<K, V>>, Vec<S>) {
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
    pub fn context(&self) -> &V::Context {
        let Self { dense, .. } = self;
        dense.context()
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
    pub fn capacity(&self) -> usize {
        let Self { dense, .. } = self;
        dense.capacity()
    }

    #[inline]
    pub fn sparse_capacity(&self) -> usize {
        let Self { sparse, .. } = self;
        sparse.capacity()
    }

    #[inline]
    pub fn reserve(&mut self, additional_dense: usize, additional_sparse: usize) {
        let Self { dense, sparse } = self;

        dense.reserve(additional_dense);
        sparse.reserve(additional_sparse);
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional_dense: usize, additional_sparse: usize) {
        let Self { dense, sparse } = self;

        dense.reserve_exact(additional_dense);
        sparse.reserve_exact(additional_sparse);
    }

    #[inline]
    pub fn try_reserve(
        &mut self,
        additional_dense: usize,
        additional_sparse: usize,
    ) -> Result<(), TryReserveError> {
        let Self { dense, sparse } = self;

        dense.try_reserve(additional_dense)?;
        sparse.try_reserve(additional_sparse)?;
        Ok(())
    }

    #[inline]
    pub fn try_reserve_exact(
        &mut self,
        additional_dense: usize,
        additional_sparse: usize,
    ) -> Result<(), TryReserveError> {
        let Self { dense, sparse } = self;

        dense.try_reserve_exact(additional_dense)?;
        sparse.try_reserve_exact(additional_sparse)?;
        Ok(())
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        let Self { dense, sparse } = self;

        dense.shrink_to_fit();
        sparse.shrink_to_fit();
    }

    #[inline]
    pub fn dense_shrink_to_fit(&mut self) {
        let Self { dense, .. } = self;
        dense.shrink_to_fit();
    }

    #[inline]
    pub fn sparse_shrink_to_fit(&mut self) {
        let Self { sparse, .. } = self;
        sparse.shrink_to_fit();
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.truncate(min_capacity, min_capacity);

        let Self { dense, sparse } = self;
        dense.shrink_to(min_capacity);
        sparse.shrink_to(min_capacity);
    }

    #[inline]
    pub fn dense_shrink_to(&mut self, min_capacity: usize) {
        self.truncate(min_capacity, usize::MAX);

        let Self { dense, .. } = self;
        dense.shrink_to(min_capacity);
    }

    #[inline]
    pub fn sparse_shrink_to(&mut self, min_capacity: usize) {
        self.truncate(usize::MAX, min_capacity);

        let Self { sparse, .. } = self;
        sparse.shrink_to(min_capacity);
    }

    #[inline]
    pub fn as_view_ptr(&self) -> EpochSparseViewPtr<'_, K, V, S> {
        let Self { dense, sparse, .. } = self;
        unsafe { EpochSparseViewPtr::from_parts(dense.slice_ptrs(), sparse.as_slice()) }
    }

    #[inline]
    pub fn as_mut_view_ptr(&mut self) -> EpochSparseViewMutPtr<'_, K, V, S> {
        let Self { dense, sparse, .. } = self;
        unsafe { EpochSparseViewMutPtr::from_parts(dense.mut_slice_ptrs(), sparse.as_mut_slice()) }
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
        let sparse = ptr::from_ref(sparse.as_slice());
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
        let sparse = ptr::from_mut(sparse.as_mut_slice());
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
    pub fn get_epoch(&self, sparse_index: K::SparseIndex) -> Option<K::Epoch> {
        let view = self.as_view();
        view.get_epoch(sparse_index)
    }

    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let view = self.as_view();
        view.contains_key(key)
    }

    #[inline]
    pub fn invalidate_epoch(&mut self, key: K) -> Option<K> {
        let mut view_mut = self.as_mut_view();
        view_mut.invalidate_epoch(key)
    }

    #[inline]
    pub unsafe fn replace_epoch(
        &mut self,
        sparse_index: K::SparseIndex,
        epoch: K::Epoch,
    ) -> Option<K> {
        let mut view_mut = self.as_mut_view();
        unsafe { view_mut.replace_epoch(sparse_index, epoch) }
    }

    #[inline]
    pub fn replace_key(&mut self, key: K) -> Option<K> {
        let mut view_mut = self.as_mut_view();
        view_mut.replace_key(key)
    }

    #[inline]
    pub fn swap(&mut self, first_key: K, second_key: K) {
        let mut view_mut = self.as_mut_view();
        view_mut.swap(first_key, second_key);
    }

    #[inline]
    pub fn swap_keys(&mut self, first_key: K, second_key: K) {
        let mut view_mut = self.as_mut_view();
        view_mut.swap_keys(first_key, second_key);
    }

    #[inline]
    pub fn clear(&mut self) {
        let Self { dense, sparse } = self;

        for key in dense_keys(dense.slices()) {
            let sparse_index = unwrap_into_usize(key.sparse_index());
            sparse[sparse_index] = S::vacant(key.epoch().next());
        }
        dense.clear();
    }

    #[inline]
    pub fn clear_sparse(&mut self) {
        let Self { dense, sparse } = self;

        sparse.clear();
        dense.clear();
    }

    pub fn truncate(&mut self, dense_len: usize, sparse_len: usize) {
        let drop_in_place = |context: &V::Context, src: Option<MutPtrs<'_, V>>| {
            let Some(value) = src else { return };
            let value = V::Context::upcast_mut_ptrs(value);
            unsafe { context.ptrs_drop_in_place(value) }
        };

        for dense_index in (dense_len..self.len()).rev() {
            let key = self.as_key_slice()[dense_index];
            self.remove_into(key, drop_in_place);
        }
        self.dense.truncate(dense_len);

        for sparse_index in sparse_len..self.sparse_len() {
            let epoch = self.as_sparse_slice()[sparse_index].epoch();
            let key = K::new(unwrap_into_index(sparse_index), epoch.next());
            self.remove_into(key, drop_in_place);
        }
        self.sparse.truncate(sparse_len);
    }

    pub fn swap_remove_into<'a, F, R>(&'a mut self, key: K, f: F) -> R
    where
        F: FnOnce(&'a V::Context, Option<MutPtrs<'a, V>>) -> R,
    {
        let Self { dense, sparse } = self;

        let Some(sparse_index) = key.sparse_index().try_into().ok() else {
            return f(dense.context(), None);
        };

        let dense_index = sparse_item_by_epoch::<K, _>(sparse, sparse_index, key.epoch())
            .copied()
            .and_then(S::dense_index);
        let Some(dense_index) = dense_index else {
            return f(dense.context(), None);
        };
        let dense_index_usize = unwrap_into_usize(dense_index);
        assert_dense_index_bounds(dense_index_usize, dense.len());

        let (keys, _) = dense.slice_ptrs().into_slice_ptrs().into_parts();
        let result = dense.swap_remove_into(dense_index_usize, |context, src| {
            let &dense_key = unsafe { src.key.as_ref_unchecked() };
            assert_equal_key(key, dense_key);
            f(context, Some(src.value.into_inner()))
        });

        // SAFETY: `remove_into()` neither shrinks nor enlarges the dense buffer
        let keys: &[K] = unsafe { slice::from_raw_parts(keys.cast(), keys.len() - 1) };
        if let Some(key) = keys.get(dense_index_usize) {
            let sparse_index = unwrap_into_usize(key.sparse_index());
            let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);
            if sparse_item.is_occupied() {
                *sparse_item = S::occupied(sparse_item.epoch(), dense_index);
            }
        }
        sparse[sparse_index] = S::vacant(key.epoch().next());

        result
    }

    #[inline]
    pub fn swap_remove<'a, R>(&'a mut self, key: K) -> Option<R>
    where
        V: SoaRead<'a, R>,
    {
        self.swap_remove_into(key, |context, src| {
            let src = context.ptrs_cast_const(src?);
            let value = unsafe { context.read(src) };
            Some(value)
        })
    }

    pub fn remove_into<'a, F, R>(&'a mut self, key: K, f: F) -> R
    where
        F: FnOnce(&'a V::Context, Option<MutPtrs<'a, V>>) -> R,
    {
        let Self { dense, sparse } = self;

        let Some(sparse_index) = key.sparse_index().try_into().ok() else {
            return f(dense.context(), None);
        };

        let dense_index = sparse_item_by_epoch::<K, _>(sparse, sparse_index, key.epoch())
            .copied()
            .and_then(S::dense_index);
        let Some(dense_index) = dense_index else {
            return f(dense.context(), None);
        };
        let dense_index = unwrap_into_usize(dense_index);
        assert_dense_index_bounds(dense_index, dense.len());

        let (keys, _) = dense.slice_ptrs().into_slice_ptrs().into_parts();
        let result = dense.remove_into(dense_index, |context, src| {
            let dense_key = unsafe { ptr::read(src.key) };
            assert_equal_key(key, dense_key);
            f(context, Some(src.value.into_inner()))
        });

        // SAFETY: `remove_into()` neither shrinks nor enlarges the dense buffer
        let keys: &[K] = unsafe { slice::from_raw_parts(keys.cast(), keys.len() - 1) };
        for key in keys.iter().skip(dense_index) {
            let sparse_index = unwrap_into_usize(key.sparse_index());
            let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);
            let dense_index = unwrap_dense_index(sparse_item);

            let dense_index = unwrap_into_index(unwrap_into_usize(dense_index) - 1);
            *sparse_item = S::occupied(sparse_item.epoch(), dense_index);
        }
        sparse[sparse_index] = S::vacant(key.epoch().next());

        result
    }

    #[inline]
    pub fn remove<'a, R>(&'a mut self, key: K) -> Option<R>
    where
        V: SoaRead<'a, R>,
    {
        self.remove_into(key, |context, src| {
            let src = context.ptrs_cast_const(src?);
            let value = unsafe { context.read(src) };
            Some(value)
        })
    }

    pub fn pop_into<'a, F, R>(&'a mut self, f: F) -> R
    where
        F: FnOnce(&'a V::Context, Option<(K, MutPtrs<'a, V>)>) -> R,
    {
        let Self { dense, sparse } = self;

        dense.pop_into(|context, src| {
            let Some(KeyValueMutPtrs { key, value }) = src else {
                return f(context, None);
            };
            let key = unsafe { ptr::read(key) };

            let sparse_index = unwrap_into_usize(key.sparse_index());
            assert_key_bounds(sparse_index, sparse.len());

            let result = f(context, Some((key, value.into_inner())));
            sparse[sparse_index] = S::vacant(key.epoch().next());
            result
        })
    }

    #[inline]
    pub fn pop<'a, R>(&'a mut self) -> Option<(K, R)>
    where
        V: SoaRead<'a, R>,
    {
        self.pop_into(|context, src| {
            let (key, value) = src?;
            let src = context.ptrs_cast_const(value);
            let value = unsafe { context.read(src) };
            Some((key, value))
        })
    }

    #[inline]
    #[track_caller]
    pub fn insert_from<'a, F, R>(&'a mut self, key: K, f: F) -> R
    where
        F: FnOnce(&'a V::Context, Option<TryInsertAccess<'a, 'a, V>>) -> R,
    {
        self.try_insert_from(key, |context, dst| {
            let dst = dst.unwrap_or_else(|error| try_insert_failed(error));
            f(context, dst)
        })
    }

    #[inline]
    #[track_caller]
    pub fn insert<'a, R, W>(&'a mut self, key: K, value: W) -> Option<R>
    where
        V: SoaRead<'a, R> + SoaWrite<W>,
    {
        self.try_insert(key, value)
            .map_err(TryModifyError::into_source)
            .unwrap_or_else(|error| try_insert_failed(error))
    }

    pub fn try_insert_from<'a, F, R>(&'a mut self, key: K, f: F) -> R
    where
        F: FnOnce(
            &'a V::Context,
            Result<Option<TryInsertAccess<'a, 'a, V>>, TryModifyErrorKind<K>>,
        ) -> R,
    {
        let Self { dense, sparse } = self;

        let sparse_index = match key.sparse_index().try_into() {
            Ok(sparse_index) => sparse_index,
            Err(error) => {
                let context = dense.context();
                let error = TooLargeSparseIndexError::new(error).into();
                return f(context, Err(error));
            }
        };

        let new_sparse_len = sparse_index.saturating_add(1);
        if let Err(error) = sparse.try_reserve(new_sparse_len.saturating_sub(sparse.len())) {
            let context = dense.context();
            let error = TryReserveError::Sparse(error).into();
            return f(context, Err(error));
        }
        extend_sparse::<K, S>(sparse, new_sparse_len);

        let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);
        if key.epoch() < sparse_item.epoch() {
            return f(dense.context(), Ok(None));
        }

        if let Some(dense_index) = sparse_item.dense_index() {
            let (context, dense) = dense.mut_slice_ptrs().into_iter_with_context();

            let dense_index_usize = unwrap_into_usize(dense_index);
            let (dense_key, dense_value) = unwrap_dense(dense, dense_index_usize).into();

            let access = unsafe { TryInsertAccess::read_write_unchecked(dense_value) };
            let result = f(context, Ok(Some(access)));

            *sparse_item = S::occupied(key.epoch(), dense_index);
            unsafe { ptr::replace(dense_key, key) };

            return result;
        }

        let dense_index = match dense.len().try_into() {
            Ok(dense_index) => dense_index,
            Err(error) => {
                let context = dense.context();
                let error = TooSmallSparseIndexError::new(error).into();
                return f(context, Err(error));
            }
        };
        if let Err(error) = dense.try_reserve(1) {
            let context = dense.context();
            let error = TryReserveError::Dense(error).into();
            return f(context, Err(error));
        }

        dense.push_from(|context, ptrs| {
            let (key_ptr, value_ptrs) = ptrs.into();
            let result = f(context, Ok(Some(TryInsertAccess::write_only(value_ptrs))));

            *sparse_item = S::occupied(key.epoch(), dense_index);
            unsafe { ptr::write(key_ptr, key) }

            result
        })
    }

    #[inline]
    pub fn try_insert<'a, R, W>(
        &'a mut self,
        key: K,
        value: W,
    ) -> Result<Option<R>, TryModifyError<K, W>>
    where
        V: SoaRead<'a, R> + SoaWrite<W>,
    {
        self.try_insert_from(key, |context, dst| match dst {
            Ok(Some(TryInsertAccess::ReadWrite(dst))) => {
                let dst = dst.into_ptrs();
                let value = unsafe { soa::ptr::replace::<V, R, W>(context, dst, value) };
                Ok(Some(value))
            }
            Ok(Some(TryInsertAccess::WriteOnly(dst))) => {
                unsafe { context.write(dst.into_inner(), value) }
                Ok(None)
            }
            Ok(None) => Ok(None),
            Err(error) => Err(TryModifyError::new(error, value)),
        })
    }

    #[inline]
    #[track_caller]
    pub fn push_from<'a, F, R>(&'a mut self, f: F) -> R
    where
        F: FnOnce(&'a V::Context, K, MutPtrs<'a, V>) -> R,
    {
        self.try_push_from(|context, dst| {
            let (key, dst) = dst.unwrap_or_else(|error| try_insert_failed(error));
            f(context, key, dst)
        })
    }

    #[inline]
    #[track_caller]
    pub fn push<W>(&mut self, value: W) -> K
    where
        V: SoaWrite<W>,
    {
        self.try_push(value)
            .map_err(TryModifyError::into_source)
            .unwrap_or_else(|error| try_push_failed(error))
    }

    pub fn try_push_from<'a, F, R>(&'a mut self, f: F) -> R
    where
        F: FnOnce(&'a V::Context, Result<(K, MutPtrs<'a, V>), TryModifyErrorKind<K>>) -> R,
    {
        let Self { sparse, .. } = self;
        let (sparse_index, epoch) = sparse
            .iter()
            .enumerate()
            .find(|(_, sparse_item)| sparse_item.is_vacant())
            .map_or_else(
                || (sparse.len(), Default::default()),
                |(sparse_index, sparse_item)| (sparse_index, sparse_item.epoch()),
            );

        let sparse_index = match sparse_index.try_into() {
            Ok(sparse_index) => sparse_index,
            Err(error) => {
                let context = self.dense.context();
                let error = TooSmallSparseIndexError::new(error).into();
                return f(context, Err(error));
            }
        };
        let key = K::new(sparse_index, epoch);

        self.try_insert_from(key, |context, dst| match dst {
            Ok(Some(TryInsertAccess::WriteOnly(dst))) => f(context, Ok((key, dst.into_inner()))),
            Ok(Some(TryInsertAccess::ReadWrite(_))) => unreachable!("entry should be vacant"),
            Ok(None) => unreachable!("key epoch should be valid"),
            Err(error) => f(context, Err(error)),
        })
    }

    #[inline]
    pub fn try_push<W>(&mut self, value: W) -> Result<K, TryModifyError<K, W>>
    where
        V: SoaWrite<W>,
    {
        self.try_push_from(|context, dst| match dst {
            Ok((key, dst)) => {
                unsafe { context.write(dst, value) }
                Ok(key)
            }
            Err(error) => Err(TryModifyError::new(error, value)),
        })
    }
}

// TODO: generalize entries to work with any sparse item
impl<K, V> EpochSparseSet<K, V>
where
    K: Key,
    V: AllocSoa + ?Sized,
{
    #[inline]
    #[track_caller]
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        self.try_entry(key)
            .unwrap_or_else(|error| try_entry_failed(error))
    }

    #[inline]
    pub fn try_entry(&mut self, key: K) -> Result<Entry<'_, K, V>, TooLargeSparseIndexError<K>> {
        let Self { dense, sparse } = self;

        let sparse_index = key
            .sparse_index()
            .try_into()
            .map_err(TooLargeSparseIndexError::new)?;
        let Some(dense_index) = sparse_item_by_epoch::<K, _>(sparse, sparse_index, key.epoch())
            .copied()
            .and_then(item::DefaultSparseItem::into_dense_index)
        else {
            let entry = VacantEntry::new(key, self);
            return Ok(Entry::Vacant(entry));
        };

        let dense_index = unwrap_into_usize(dense_index);
        assert_dense_index_bounds(dense_index, dense.len());
        let entry = OccupiedEntry::new(key, dense_index, self);
        Ok(Entry::Occupied(entry))
    }
}

impl<K, V, S> EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    pub fn raw_keys(&self) -> RawKeys<'_, K, V> {
        let (_, iter) = self.raw_keys_with_context();
        iter
    }

    #[inline]
    pub fn raw_keys_with_context(&self) -> (&V::Context, RawKeys<'_, K, V>) {
        let view = self.as_view();
        view.into_raw_keys_with_context()
    }

    #[inline]
    pub fn raw_values(&self) -> RawValues<'_, K, V> {
        let (_, iter) = self.raw_values_with_context();
        iter
    }

    #[inline]
    pub fn raw_values_with_context(&self) -> (&V::Context, RawValues<'_, K, V>) {
        let view = self.as_view();
        view.into_raw_values_with_context()
    }

    #[inline]
    pub fn raw_values_mut(&mut self) -> RawValuesMut<'_, K, V> {
        let (_, iter) = self.raw_values_mut_with_context();
        iter
    }

    #[inline]
    pub fn raw_values_mut_with_context(&mut self) -> (&V::Context, RawValuesMut<'_, K, V>) {
        let view_mut = self.as_mut_view();
        view_mut.into_raw_values_mut_with_context()
    }

    #[inline]
    pub fn raw_iter(&self) -> RawIter<'_, K, V> {
        let (_, iter) = self.raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn raw_iter_with_context(&self) -> (&V::Context, RawIter<'_, K, V>) {
        let view = self.as_view();
        view.into_raw_iter_with_context()
    }

    #[inline]
    pub fn raw_iter_mut(&mut self) -> RawIterMut<'_, K, V> {
        let (_, iter) = self.raw_iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn raw_iter_mut_with_context(&mut self) -> (&V::Context, RawIterMut<'_, K, V>) {
        let view_mut = self.as_mut_view();
        view_mut.into_raw_iter_mut_with_context()
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_, '_, K, V> {
        let (_, iter) = self.keys_with_context();
        iter
    }

    #[inline]
    pub fn keys_with_context(&self) -> (&V::Context, Keys<'_, '_, K, V>) {
        let view = self.as_view();
        view.into_keys_with_context()
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
    #[must_use]
    pub fn into_keys(self) -> IntoKeys<K, V> {
        let Self { dense, .. } = self;
        IntoKeys::new(dense)
    }

    #[inline]
    pub fn drain<'a, R>(&'a mut self) -> Drain<'a, K, V, R>
    where
        V: SoaRead<'a, R>,
    {
        let Self { dense, sparse } = self;

        for KeyValuePtrs { key, .. } in dense.slice_ptrs() {
            let key = unsafe { key.as_ref_unchecked() };
            let sparse_index = unwrap_into_usize(key.sparse_index());
            sparse[sparse_index] = S::vacant(key.epoch().next());
        }

        Drain::new(dense.drain(..))
    }

    #[inline]
    #[must_use]
    pub fn into_values<R>(self) -> IntoValues<K, V, R>
    where
        V: SoaReadOwned<R>,
    {
        let Self { dense, .. } = self;
        let inner = dense.into_items();
        IntoValues::new(inner)
    }

    #[inline]
    pub fn into_items<R>(self) -> IntoIter<K, V, R>
    where
        V: SoaReadOwned<R>,
    {
        let Self { dense, .. } = self;
        let inner = dense.into_items();
        IntoIter::new(inner)
    }
}

impl<'a, K, V, S> EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + Soa<'a> + ?Sized,
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
        let view = self.as_view();
        view.into_get_with_context(key)
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
        let view_mut = self.as_mut_view();
        view_mut.into_get_mut_with_context(key)
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
        let view = self.as_view();
        view.into_index_with_context(key)
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
        let view_mut = self.as_mut_view();
        view_mut.into_index_mut_with_context(key)
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
        let view = self.as_view();
        view.into_get_with_key_with_context(sparse_index)
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
        let view_mut = self.as_mut_view();
        view_mut.into_get_mut_with_key_with_context(sparse_index)
    }

    #[inline]
    pub fn values(&'a self) -> Values<'a, 'a, K, V> {
        let (_, iter) = self.values_with_context();
        iter
    }

    #[inline]
    pub fn values_with_context(&'a self) -> (&'a V::Context, Values<'a, 'a, K, V>) {
        let view = self.as_view();
        view.into_values_with_context()
    }

    #[inline]
    pub fn values_mut(&'a mut self) -> ValuesMut<'a, 'a, K, V> {
        let (_, iter) = self.values_mut_with_context();
        iter
    }

    #[inline]
    pub fn values_mut_with_context(&'a mut self) -> (&'a V::Context, ValuesMut<'a, 'a, K, V>) {
        let view_mut = self.as_mut_view();
        view_mut.into_values_mut_with_context()
    }

    #[inline]
    pub fn iter(&'a self) -> Iter<'a, 'a, K, V> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&'a self) -> (&'a V::Context, Iter<'a, 'a, K, V>) {
        let view = self.as_view();
        view.into_iter_with_context()
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> IterMut<'a, 'a, K, V> {
        let (_, iter) = self.iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn iter_mut_with_context(&'a mut self) -> (&'a V::Context, IterMut<'a, 'a, K, V>) {
        let view_mut = self.as_mut_view();
        view_mut.into_iter_with_context()
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

impl<K, V, S> EpochSparseSet<K, V, S>
where
    K: Key,
    V: SoaOwned + AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(K, RefsMut<'_, '_, V>) -> bool,
    {
        let old_len = self.len();
        let Self { dense, sparse } = self;

        let mut last = 0;
        for curr in 0..old_len {
            let (&mut key, value) = dense.mut_slices().into_index_mut(curr).into();
            if !f(key, value) {
                let sparse_index = unwrap_into_usize(key.sparse_index());
                sparse[sparse_index] = S::vacant(key.epoch().next());
                continue;
            }

            dense.mut_slices().swap(curr, last);

            let sparse_index = unwrap_into_usize(key.sparse_index());
            let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);
            let dense_index = unwrap_dense_index(sparse_item);

            let dense_index = unwrap_into_index(unwrap_into_usize(dense_index) - (curr - last));
            *sparse_item = S::occupied(sparse_item.epoch(), dense_index);

            last += 1;
        }

        dense.truncate(last);
    }

    #[inline]
    pub fn sort(&mut self)
    where
        for<'ctx, 'a> Refs<'ctx, 'a, V>: Ord,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort();
    }

    #[inline]
    pub fn sort_keys(&mut self) {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_keys();
    }

    #[inline]
    pub fn sort_by<F>(&mut self, f: F)
    where
        for<'a> F: FnMut((K, Refs<'_, 'a, V>), (K, Refs<'_, 'a, V>)) -> cmp::Ordering,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_by(f);
    }

    #[inline]
    pub fn sort_by_key<T, F>(&mut self, f: F)
    where
        F: FnMut((K, Refs<'_, '_, V>)) -> T,
        T: Ord,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_by_key(f);
    }

    #[inline]
    pub fn sort_by_cached_key<T, F>(&mut self, f: F)
    where
        F: FnMut((K, Refs<'_, '_, V>)) -> T,
        T: Ord,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_by_cached_key(f);
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        for<'ctx, 'a> Refs<'ctx, 'a, V>: Ord,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_unstable();
    }

    #[inline]
    pub fn sort_keys_unstable(&mut self) {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_keys_unstable();
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, f: F)
    where
        for<'a> F: FnMut((K, Refs<'_, 'a, V>), (K, Refs<'_, 'a, V>)) -> cmp::Ordering,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_unstable_by(f);
    }

    #[inline]
    pub fn sort_unstable_by_key<T, F>(&mut self, f: F)
    where
        F: FnMut((K, Refs<'_, '_, V>)) -> T,
        T: Ord,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_unstable_by_key(f);
    }
}

impl<K, V, S> Debug for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Debug,
    SoaVec<KeyValuePair<K, V>>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { dense, sparse } = self;
        f.debug_struct("EpochSparseSet")
            .field("dense", dense)
            .field("sparse", sparse)
            .finish()
    }
}

impl<K, V, S> Default for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Default,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, S> PartialEq for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + PartialEq,
    SoaVec<KeyValuePair<K, V>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse) == other
    }
}

impl<K, V, S> Eq for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Eq,
    SoaVec<KeyValuePair<K, V>>: Eq,
{
}

impl<K, V, S> PartialOrd for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + PartialOrd,
    SoaVec<KeyValuePair<K, V>>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse).partial_cmp(&other)
    }
}

impl<K, V, S> Ord for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Ord,
    SoaVec<KeyValuePair<K, V>>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { dense, sparse } = self;

        let other = (&other.dense, &other.sparse);
        (dense, sparse).cmp(&other)
    }
}

impl<K, V, S> Hash for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Hash,
    SoaVec<KeyValuePair<K, V>>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { dense, sparse } = self;
        (dense, sparse).hash(state);
    }
}

impl<K, V, S> Clone for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    SoaVec<KeyValuePair<K, V>>: Clone,
{
    fn clone(&self) -> Self {
        let Self { dense, sparse } = self;
        Self {
            dense: dense.clone(),
            sparse: sparse.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        let Self { dense, sparse } = self;
        let Self {
            dense: source_dense,
            sparse: source_sparse,
        } = source;

        dense.clone_from(source_dense);
        sparse.clone_from(source_sparse);
    }
}

impl<T, K, V, S> Index<K> for EpochSparseSet<K, V, S>
where
    K: Key + Debug,
    V: SoaOwned + AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx, 'a> V::Context: SoaContext<'a, V, Refs<'ctx> = &'a T>,
{
    type Output = T;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        Self::index(self, key)
    }
}

impl<T, K, V, S> IndexMut<K> for EpochSparseSet<K, V, S>
where
    K: Key + Debug,
    V: AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx, 'a> V::Context: SoaContext<'a, V, Refs<'ctx> = &'a T, RefsMut<'ctx> = &'a mut T>,
{
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        Self::index_mut(self, key)
    }
}

impl<T, K, V, S> AsRef<[T]> for EpochSparseSet<K, V, S>
where
    K: Key,
    V: SoaOwned + AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Into<&'a [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_value_slices().into()
    }
}

impl<T, K, V, S> AsMut<[T]> for EpochSparseSet<K, V, S>
where
    K: Key,
    V: SoaOwned + AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Into<&'a mut [T]>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_value_slices().into()
    }
}

impl<K, V, S> AsRef<Self> for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<K, V, S> AsMut<Self> for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'a, K, V, S> IntoIterator for &'a EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + Soa<'a> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (&'a K, Refs<'a, 'a, V>);
    type IntoIter = Iter<'a, 'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + Soa<'a> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (&'a K, RefsMut<'a, 'a, V>);
    type IntoIter = IterMut<'a, 'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V, S> IntoIterator for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + SoaReadOwned<V>,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_items()
    }
}

#[cfg(feature = "rayon")]
impl<'a, K, V, S> rayon::iter::IntoParallelIterator for &'a EpochSparseSet<K, V, S>
where
    K: Key + Sync,
    V: AllocSoa + Soa<'a> + ?Sized,
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
impl<'a, K, V, S> rayon::iter::IntoParallelIterator for &'a mut EpochSparseSet<K, V, S>
where
    K: Key + Send + Sync,
    V: AllocSoa + Soa<'a> + ?Sized,
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

impl<K, V, S, W> FromIterator<KeyValuePair<K, W>> for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + SoaWrite<W> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Default,
{
    fn from_iter<I: IntoIterator<Item = KeyValuePair<K, W>>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let iter_len = {
            let (lower, upper) = iter.size_hint();
            upper.unwrap_or(lower)
        };

        let mut me = Self::with_capacity(iter_len, iter_len);
        for KeyValuePair { key, value } in iter {
            me.insert_from(key, |context, dst| {
                let Some(dst) = dst else { return };
                unsafe { dst.drop_in_place_then_write(context, value) }
            });
        }

        me
    }
}

impl<K, V, S, W> FromIterator<(K, W)> for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + SoaWrite<W> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Default,
{
    fn from_iter<T: IntoIterator<Item = (K, W)>>(iter: T) -> Self {
        iter.into_iter().map(KeyValuePair::from).collect()
    }
}

impl<K, V, S, W> Extend<KeyValuePair<K, W>> for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + SoaWrite<W> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    fn extend<I: IntoIterator<Item = KeyValuePair<K, W>>>(&mut self, iter: I) {
        let mut iter = iter.into_iter();
        while let Some(KeyValuePair { key, value }) = iter.next() {
            if self.len() == self.capacity() {
                let (lower, _) = iter.size_hint();
                self.reserve(lower.saturating_add(1), 0);
            }
            self.insert_from(key, |context, dst| {
                let Some(dst) = dst else { return };
                unsafe { dst.drop_in_place_then_write(context, value) }
            });
        }
    }
}

impl<K, V, S, W> Extend<(K, W)> for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + SoaWrite<W> + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    fn extend<I: IntoIterator<Item = (K, W)>>(&mut self, iter: I) {
        self.extend(iter.into_iter().map(KeyValuePair::from));
    }
}

impl<K, V, S> From<arena::EpochSparseArena<K, V, S>> for EpochSparseSet<K, V, S>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn from(value: arena::EpochSparseArena<K, V, S>) -> Self {
        let (dense, sparse, _) = value.into_parts();
        unsafe { Self::from_parts_unchecked(dense, sparse) }
    }
}

fn extend_sparse<K, S>(sparse: &mut Vec<S>, new_len: usize)
where
    K: Key,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    let old_len = sparse.len();
    if old_len >= new_len {
        return;
    }

    let epoch = Default::default();
    let item = S::vacant(epoch);
    sparse.resize(new_len, item);
}

generate_entry_types!(EpochSparseSet<K, V>);
