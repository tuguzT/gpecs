use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::{Index, IndexMut},
    ptr, slice,
};
use core_alloc::vec::Vec;

use gpecs_ptr::slice::{CoreSliceItemPtrs, MutSliceItemPtr, SliceItemPtr, SliceItemPtrs};

use crate::{
    algo::{check_parts, dense_keys, sparse_item_by_epoch},
    assert::{
        assert_dense_index_bounds, assert_equal_key, assert_key_bounds, assert_occupied,
        assert_vacant, unwrap_dense, unwrap_dense_index, unwrap_into_index, unwrap_into_usize,
        unwrap_next_vacant, unwrap_sparse_item, unwrap_sparse_item_mut,
    },
    error::{
        FromPartsError, TooLargeSparseIndexError, TooSmallSparseIndexError, TryModifyError,
        TryModifyErrorKind, TryReserveError,
    },
    item::{
        ArenaSparseItem, DefaultSparseItem, KeyValueMutPtrs, KeyValueMutSlicePtrs,
        KeyValueMutSlices, KeyValuePair, KeyValuePtrs, KeyValueSlicePtrs, KeyValueSlices,
        SparseIndexKind,
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
    assert::{try_entry_failed, try_insert_failed, try_push_failed},
    entry::generate_entry_types,
    set,
};

pub type SparseArena<T, S = DefaultSparseItem<usize>, P = CoreSliceItemPtrs<usize>> =
    EpochSparseArena<usize, T, S, P>;

pub struct EpochSparseArena<K, V, S = DefaultSparseItem<K>, P = CoreSliceItemPtrs<K>>
where
    K: Key,
    V: AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    dense: SoaVec<KeyValuePair<K, V, P>>,
    sparse: Vec<S>,
    sparse_vacant_head: usize,
}

impl<K, V, S, P> EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
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
            sparse_vacant_head: 0,
        }
    }

    #[inline]
    #[must_use]
    pub fn with_context(context: V::Context) -> Self {
        Self {
            dense: SoaVec::with_context(context.into()),
            sparse: Vec::new(),
            sparse_vacant_head: 0,
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
            sparse_vacant_head: 0,
        }
    }

    #[inline]
    #[must_use]
    pub fn with_context_and_capacity(context: V::Context, dense: usize, sparse: usize) -> Self {
        Self {
            dense: SoaVec::with_context_and_capacity(context.into(), dense),
            sparse: Vec::with_capacity(sparse),
            sparse_vacant_head: 0,
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
        dense: SoaVec<KeyValuePair<K, V, P>>,
        mut sparse: Vec<S>,
    ) -> Result<Self, FromPartsError<K>> {
        check_parts(dense.slices(), sparse.as_slice())?;

        sparse.clear();
        let mut sparse_vacant_head = 0;
        for (dense_index, key) in dense_keys(dense.slices()).iter().enumerate() {
            let sparse_index = key
                .sparse_index()
                .try_into()
                .map_err(TooLargeSparseIndexError::new)?;
            let epoch = key.epoch();

            let dense_index = dense_index
                .try_into()
                .map_err(TooSmallSparseIndexError::new)?;
            let item = S::occupied(epoch, dense_index);

            if sparse_index >= sparse.len() {
                let new_len = sparse_index.saturating_add(1);
                extend_sparse(&mut sparse, new_len, &mut sparse_vacant_head)?;
            } else {
                let sparse_item = unwrap_sparse_item(sparse.as_slice(), sparse_index);
                let next_vacant = unwrap_next_vacant(sparse_item)
                    .try_into()
                    .map_err(TooLargeSparseIndexError::new)?;
                remove_from_vacant_list::<K, S>(
                    &mut sparse,
                    &mut sparse_vacant_head,
                    sparse_index,
                    next_vacant,
                );
            }
            sparse[sparse_index] = item;
        }

        let me = unsafe { Self::from_parts_unchecked(dense, sparse, sparse_vacant_head) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_parts_unchecked(
        dense: SoaVec<KeyValuePair<K, V, P>>,
        sparse: Vec<S>,
        sparse_vacant_head: usize,
    ) -> Self {
        Self {
            dense,
            sparse,
            sparse_vacant_head,
        }
    }

    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (SoaVec<KeyValuePair<K, V, P>>, Vec<S>, usize) {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;
        (dense, sparse, sparse_vacant_head)
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
        let Self { dense, sparse, .. } = self;

        dense.reserve(additional_dense);
        sparse.reserve(additional_sparse);
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional_dense: usize, additional_sparse: usize) {
        let Self { dense, sparse, .. } = self;

        dense.reserve_exact(additional_dense);
        sparse.reserve_exact(additional_sparse);
    }

    #[inline]
    pub fn try_reserve(
        &mut self,
        additional_dense: usize,
        additional_sparse: usize,
    ) -> Result<(), TryReserveError> {
        let Self { dense, sparse, .. } = self;

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
        let Self { dense, sparse, .. } = self;

        dense.try_reserve_exact(additional_dense)?;
        sparse.try_reserve_exact(additional_sparse)?;
        Ok(())
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        let Self { dense, sparse, .. } = self;

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

        let Self { dense, sparse, .. } = self;
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
    pub fn as_view_ptr(&self) -> EpochSparseViewPtr<'_, K, V, S, P> {
        let Self { dense, sparse, .. } = self;
        unsafe { EpochSparseViewPtr::from_parts(dense.slice_ptrs(), sparse.as_slice()) }
    }

    #[inline]
    pub fn as_mut_view_ptr(&mut self) -> EpochSparseViewMutPtr<'_, K, V, S, P> {
        let Self { dense, sparse, .. } = self;
        unsafe { EpochSparseViewMutPtr::from_parts(dense.mut_slice_ptrs(), sparse.as_mut_slice()) }
    }

    #[inline]
    pub fn as_view(&self) -> EpochSparseView<'_, '_, K, V, S, P> {
        unsafe { self.as_view_ptr().as_ref_unchecked() }
    }

    #[inline]
    pub fn as_mut_view(&mut self) -> EpochSparseViewMut<'_, '_, K, V, S, P> {
        unsafe { self.as_mut_view_ptr().as_mut_unchecked() }
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
        let Self { dense, sparse, .. } = self;

        let (context, dense) = dense.as_ptrs_with_context();
        let sparse = sparse.as_ptr();
        (context, dense, sparse)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (KeyValueMutPtrs<'_, K, V, P::Mut>, *mut S) {
        let (_, dense, sparse) = self.as_mut_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(
        &mut self,
    ) -> (&V::Context, KeyValueMutPtrs<'_, K, V, P::Mut>, *mut S) {
        let Self { dense, sparse, .. } = self;

        let (context, dense) = dense.as_mut_ptrs_with_context();
        let sparse = sparse.as_mut_ptr();
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
    pub fn as_mut_key_ptr(&mut self) -> P::Mut {
        let (_, key) = self.as_mut_key_ptr_with_context();
        key
    }

    #[inline]
    pub fn as_mut_key_ptr_with_context(&mut self) -> (&V::Context, P::Mut) {
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
    pub fn as_mut_dense_ptrs(&mut self) -> KeyValueMutPtrs<'_, K, V, P::Mut> {
        let (_, dense) = self.as_mut_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_mut_dense_ptrs_with_context(
        &mut self,
    ) -> (&V::Context, KeyValueMutPtrs<'_, K, V, P::Mut>) {
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
        let Self { dense, sparse, .. } = self;

        let (context, dense) = dense.as_slice_ptrs_with_context();
        let sparse = ptr::from_ref(sparse.as_slice());
        (context, dense, sparse)
    }

    #[inline]
    pub fn as_mut_slice_ptrs(&mut self) -> (KeyValueMutSlicePtrs<'_, K, V, P::Mut>, *mut [S]) {
        let (_, dense, sparse) = self.as_mut_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub fn as_mut_slice_ptrs_with_context(
        &mut self,
    ) -> (
        &V::Context,
        KeyValueMutSlicePtrs<'_, K, V, P::Mut>,
        *mut [S],
    ) {
        let Self { dense, sparse, .. } = self;

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
    pub fn as_mut_dense_slice_ptrs(&mut self) -> KeyValueMutSlicePtrs<'_, K, V, P::Mut> {
        let (_, dense) = self.as_mut_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn as_mut_dense_slice_ptrs_with_context(
        &mut self,
    ) -> (&V::Context, KeyValueMutSlicePtrs<'_, K, V, P::Mut>) {
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
    pub unsafe fn get_mut_with_key_unchecked(
        &mut self,
        sparse_index: K::SparseIndex,
    ) -> (P::Mut, MutPtrs<'_, V>) {
        let (_, key, value) = unsafe { self.get_mut_with_key_unchecked_with_context(sparse_index) };
        (key, value)
    }

    #[inline]
    pub unsafe fn get_mut_with_key_unchecked_with_context(
        &mut self,
        sparse_index: K::SparseIndex,
    ) -> (&V::Context, P::Mut, MutPtrs<'_, V>) {
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
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        for key in dense_keys(dense.slices()) {
            let sparse_index = unwrap_into_usize(key.sparse_index());
            let next_vacant = unwrap_into_index(*sparse_vacant_head);
            sparse[sparse_index] = S::with_next_vacant(key.epoch().next(), next_vacant);
            *sparse_vacant_head = sparse_index;
        }
        dense.clear();
    }

    #[inline]
    pub fn clear_sparse(&mut self) {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        sparse.clear();
        *sparse_vacant_head = 0;
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
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

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
            let (src_key, src_value) = src.into_parts();
            let dense_key = unsafe { src_key.read() };
            assert_equal_key(key, dense_key);
            f(context, Some(src_value))
        });

        // SAFETY: `swap_remove_into()` neither shrinks nor enlarges the dense buffer
        let keys: &[K] = unsafe { slice::from_raw_parts(keys.cast(), keys.len() - 1) };
        if let Some(key) = keys.get(dense_index_usize) {
            let sparse_index = unwrap_into_usize(key.sparse_index());
            let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);

            let (_, kind) = sparse_item.index();
            match kind {
                SparseIndexKind::NextVacant => {
                    assert_vacant(sparse_item);
                    *sparse_item = S::with_next_vacant(sparse_item.epoch(), dense_index);
                }
                SparseIndexKind::Dense => {
                    assert_occupied(sparse_item);
                    *sparse_item = S::occupied(sparse_item.epoch(), dense_index);
                }
            }
        }
        let next_vacant = unwrap_into_index(*sparse_vacant_head);
        sparse[sparse_index] = S::with_next_vacant(key.epoch().next(), next_vacant);
        *sparse_vacant_head = sparse_index;

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
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

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
            let (src_key, src_value) = src.into_parts();
            let dense_key = unsafe { src_key.read() };
            assert_equal_key(key, dense_key);
            f(context, Some(src_value))
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
        let next_vacant = unwrap_into_index(*sparse_vacant_head);
        sparse[sparse_index] = S::with_next_vacant(key.epoch().next(), next_vacant);
        *sparse_vacant_head = sparse_index;

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
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        dense.pop_into(|context, src| {
            let Some(src) = src else {
                return f(context, None);
            };
            let (key, value) = src.into_parts();

            let key = unsafe { key.read() };
            let sparse_index = unwrap_into_usize(key.sparse_index());
            assert_key_bounds(sparse_index, sparse.len());

            let result = f(context, Some((key, value)));

            let next_vacant = unwrap_into_index(*sparse_vacant_head);
            sparse[sparse_index] = S::with_next_vacant(key.epoch().next(), next_vacant);
            *sparse_vacant_head = sparse_index;

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
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        let sparse_index: usize = match key.sparse_index().try_into() {
            Ok(sparse_index) => sparse_index,
            Err(error) => {
                let context = dense.context();
                let error = TooLargeSparseIndexError::new(error).into();
                return f(context, Err(error));
            }
        };

        match sparse.get_mut(sparse_index) {
            Some(sparse_item) if key.epoch() >= sparse_item.epoch() => {
                let (index, kind) = sparse_item.index();
                match kind {
                    SparseIndexKind::NextVacant => {
                        assert_vacant(sparse_item);

                        let next_vacant = unwrap_into_usize(index);
                        remove_from_vacant_list::<K, S>(
                            sparse,
                            sparse_vacant_head,
                            sparse_index,
                            next_vacant,
                        );

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

                        dense.push_from(|context, dst| {
                            let (key_ptr, value_ptrs) = dst.into();
                            let result =
                                f(context, Ok(Some(TryInsertAccess::write_only(value_ptrs))));

                            sparse[sparse_index] = S::occupied(key.epoch(), dense_index);
                            unsafe { key_ptr.write(key) }

                            result
                        })
                    }
                    SparseIndexKind::Dense => {
                        assert_occupied(sparse_item);

                        let (context, dense) = dense.mut_slice_ptrs().into_iter_with_context();

                        let dense_index = unwrap_into_usize(index);
                        let (dense_key, dense_value) = unwrap_dense(dense, dense_index).into();

                        let access = unsafe { TryInsertAccess::read_write_unchecked(dense_value) };
                        let result = f(context, Ok(Some(access)));

                        *sparse_item = S::occupied(key.epoch(), index);
                        unsafe { ptr::replace(dense_key.as_mut_raw_ptr(), key) };

                        result
                    }
                }
            }
            Some(_) => f(dense.context(), Ok(None)),
            None => {
                let new_sparse_len = sparse_index.saturating_add(1);
                let additional = new_sparse_len.saturating_sub(sparse.len());
                if let Err(error) = sparse.try_reserve(additional) {
                    let context = dense.context();
                    let error = TryReserveError::Sparse(error).into();
                    return f(context, Err(error));
                }
                if let Err(error) = extend_sparse(sparse, new_sparse_len, sparse_vacant_head) {
                    let context = dense.context();
                    let error = error.into();
                    return f(context, Err(error));
                }

                let dense_index: K::SparseIndex = match dense.len().try_into() {
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

                dense.push_from(|context, dst| {
                    let (key_ptr, value_ptrs) = dst.into();
                    let result = f(context, Ok(Some(TryInsertAccess::write_only(value_ptrs))));

                    sparse[sparse_index] = S::occupied(key.epoch(), dense_index);
                    unsafe { key_ptr.write(key) }

                    result
                })
            }
        }
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
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        if let Some(sparse_item) = sparse.get_mut(*sparse_vacant_head) {
            let next_vacant = match unwrap_next_vacant(sparse_item).try_into() {
                Ok(next_vacant) => next_vacant,
                Err(error) => {
                    let context = dense.context();
                    let error = TooLargeSparseIndexError::new(error).into();
                    return f(context, Err(error));
                }
            };

            let sparse_index = match (*sparse_vacant_head).try_into() {
                Ok(sparse_index) => sparse_index,
                Err(error) => {
                    let context = dense.context();
                    let error = TooSmallSparseIndexError::new(error).into();
                    return f(context, Err(error));
                }
            };
            let key = K::new(sparse_index, sparse_item.epoch());

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

            return dense.push_from(|context, dst| {
                let (key_ptr, value_ptrs) = dst.into();
                let result = f(context, Ok((key, value_ptrs)));

                *sparse_item = S::occupied(sparse_item.epoch(), dense_index);
                *sparse_vacant_head = next_vacant;
                unsafe { key_ptr.write(key) }

                result
            });
        }

        let sparse_index = match (*sparse_vacant_head).try_into() {
            Ok(sparse_index) => sparse_index,
            Err(error) => {
                let context = dense.context();
                let error = TooSmallSparseIndexError::new(error).into();
                return f(context, Err(error));
            }
        };
        let key = K::new(sparse_index, Default::default());

        let dense_index = match dense.len().try_into() {
            Ok(dense_index) => dense_index,
            Err(error) => {
                let context = dense.context();
                let error = TooSmallSparseIndexError::new(error).into();
                return f(context, Err(error));
            }
        };
        let sparse_item = S::occupied(Default::default(), dense_index);

        if let Err(error) = dense.try_reserve(1) {
            let context = dense.context();
            let error = TryReserveError::Dense(error).into();
            return f(context, Err(error));
        }
        if let Err(error) = sparse.try_reserve(1) {
            let context = dense.context();
            let error = TryReserveError::Sparse(error).into();
            return f(context, Err(error));
        }

        let new_sparse_vacant_head = dense.len() + 1;
        dense.push_from(|context, dst| {
            let (key_ptr, value_ptrs) = dst.into();
            let result = f(context, Ok((key, value_ptrs)));

            sparse.push(sparse_item);
            *sparse_vacant_head = new_sparse_vacant_head;
            unsafe { key_ptr.write(key) }

            result
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

impl<K, V, S> EpochSparseArena<K, V, S>
where
    K: Key,
    V: AllocSoa + ?Sized,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    #[track_caller]
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V, S> {
        self.try_entry(key)
            .unwrap_or_else(|error| try_entry_failed(error))
    }

    #[inline]
    pub fn try_entry(&mut self, key: K) -> Result<Entry<'_, K, V, S>, TooLargeSparseIndexError<K>> {
        let Self { dense, sparse, .. } = self;

        let sparse_index = key
            .sparse_index()
            .try_into()
            .map_err(TooLargeSparseIndexError::new)?;
        let Some(dense_index) = sparse_item_by_epoch::<K, _>(sparse, sparse_index, key.epoch())
            .copied()
            .and_then(S::dense_index)
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

impl<K, V, S, P> EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
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
    pub fn raw_values(&self) -> RawValues<'_, K, V, P> {
        let (_, iter) = self.raw_values_with_context();
        iter
    }

    #[inline]
    pub fn raw_values_with_context(&self) -> (&V::Context, RawValues<'_, K, V, P>) {
        let view = self.as_view();
        view.into_raw_values_with_context()
    }

    #[inline]
    pub fn raw_values_mut(&mut self) -> RawValuesMut<'_, K, V, P> {
        let (_, iter) = self.raw_values_mut_with_context();
        iter
    }

    #[inline]
    pub fn raw_values_mut_with_context(&mut self) -> (&V::Context, RawValuesMut<'_, K, V, P>) {
        let view_mut = self.as_mut_view();
        view_mut.into_raw_values_mut_with_context()
    }

    #[inline]
    pub fn raw_iter(&self) -> RawIter<'_, K, V, P> {
        let (_, iter) = self.raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn raw_iter_with_context(&self) -> (&V::Context, RawIter<'_, K, V, P>) {
        let view = self.as_view();
        view.into_raw_iter_with_context()
    }

    #[inline]
    pub fn raw_iter_mut(&mut self) -> RawIterMut<'_, K, V, P> {
        let (_, iter) = self.raw_iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn raw_iter_mut_with_context(&mut self) -> (&V::Context, RawIterMut<'_, K, V, P>) {
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
    pub fn drain<'a, R>(&'a mut self) -> Drain<'a, K, V, R, P>
    where
        V: SoaRead<'a, R>,
    {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        for key in dense_keys(dense.slices()) {
            let sparse_index = unwrap_into_usize(key.sparse_index());

            let next_vacant = unwrap_into_index(*sparse_vacant_head);
            sparse[sparse_index] = S::with_next_vacant(key.epoch().next(), next_vacant);
            *sparse_vacant_head = sparse_index;
        }

        Drain::new(dense.drain(..))
    }

    #[inline]
    #[must_use]
    pub fn into_values<R>(self) -> IntoValues<K, V, R, P>
    where
        V: SoaReadOwned<R>,
    {
        let Self { dense, .. } = self;
        let inner = dense.into_items();
        IntoValues::new(inner)
    }

    #[inline]
    pub fn into_items<R>(self) -> IntoIter<K, V, R, P>
    where
        V: SoaReadOwned<R>,
    {
        let Self { dense, .. } = self;
        let inner = dense.into_items();
        IntoIter::new(inner)
    }
}

impl<'a, K, V, S, P> EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
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
    pub unsafe fn as_mut_slices(
        &'a mut self,
    ) -> (KeyValueMutSlices<'a, 'a, K, V, P::Mut>, &'a mut [S]) {
        let (_, dense, sparse) = unsafe { self.as_mut_slices_with_context() };
        (dense, sparse)
    }

    #[inline]
    #[expect(clippy::type_complexity)]
    pub unsafe fn as_mut_slices_with_context(
        &'a mut self,
    ) -> (
        &'a V::Context,
        KeyValueMutSlices<'a, 'a, K, V, P::Mut>,
        &'a mut [S],
    ) {
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
    pub unsafe fn as_mut_dense_slices(&'a mut self) -> KeyValueMutSlices<'a, 'a, K, V, P::Mut> {
        let (_, dense) = unsafe { self.as_mut_dense_slices_with_context() };
        dense
    }

    #[inline]
    pub unsafe fn as_mut_dense_slices_with_context(
        &'a mut self,
    ) -> (&'a V::Context, KeyValueMutSlices<'a, 'a, K, V, P::Mut>) {
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
    pub fn values(&'a self) -> Values<'a, 'a, K, V, P> {
        let (_, iter) = self.values_with_context();
        iter
    }

    #[inline]
    pub fn values_with_context(&'a self) -> (&'a V::Context, Values<'a, 'a, K, V, P>) {
        let view = self.as_view();
        view.into_values_with_context()
    }

    #[inline]
    pub fn values_mut(&'a mut self) -> ValuesMut<'a, 'a, K, V, P> {
        let (_, iter) = self.values_mut_with_context();
        iter
    }

    #[inline]
    pub fn values_mut_with_context(&'a mut self) -> (&'a V::Context, ValuesMut<'a, 'a, K, V, P>) {
        let view_mut = self.as_mut_view();
        view_mut.into_values_mut_with_context()
    }

    #[inline]
    pub fn iter(&'a self) -> Iter<'a, 'a, K, V, P> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&'a self) -> (&'a V::Context, Iter<'a, 'a, K, V, P>) {
        let view = self.as_view();
        view.into_iter_with_context()
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> IterMut<'a, 'a, K, V, P> {
        let (_, iter) = self.iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn iter_mut_with_context(&'a mut self) -> (&'a V::Context, IterMut<'a, 'a, K, V, P>) {
        let view_mut = self.as_mut_view();
        view_mut.into_iter_with_context()
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
    pub fn par_iter_mut(&'a mut self) -> crate::iter::ParIterMut<'a, 'a, K, V, P> {
        let (_, iter) = self.par_iter_mut_with_context();
        iter
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_iter_mut_with_context(
        &'a mut self,
    ) -> (&'a V::Context, crate::iter::ParIterMut<'a, 'a, K, V, P>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.mut_slices_with_context();
        let iter = crate::iter::ParIterMut::new(slices.into_par_iter());
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

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_values_mut(&'a mut self) -> crate::iter::ParValuesMut<'a, 'a, K, V, P> {
        let (_, iter) = self.par_values_mut_with_context();
        iter
    }

    #[inline]
    #[cfg(feature = "rayon")]
    pub fn par_values_mut_with_context(
        &'a mut self,
    ) -> (&'a V::Context, crate::iter::ParValuesMut<'a, 'a, K, V, P>) {
        let (context, inner) = self.par_iter_mut_with_context();
        let values = crate::iter::ParValuesMut::new(inner);
        (context, values)
    }
}

impl<K, V, S, P> EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: SoaOwned + AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(K, RefsMut<'_, '_, V>) -> bool,
    {
        let old_len = self.len();
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        let mut last = 0;
        for curr in 0..old_len {
            let (&mut key, value) = dense.mut_slices().into_index_mut(curr).into();
            if !f(key, value) {
                let sparse_index = unwrap_into_usize(key.sparse_index());

                let next_vacant = unwrap_into_index(*sparse_vacant_head);
                sparse[sparse_index] = S::with_next_vacant(key.epoch().next(), next_vacant);
                *sparse_vacant_head = sparse_index;
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

impl<K, V, S, P> Debug for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Debug,
    SoaVec<KeyValuePair<K, V, P>>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        f.debug_struct("EpochSparseArena")
            .field("dense", dense)
            .field("sparse", sparse)
            .field("sparse_vacant_head", sparse_vacant_head)
            .finish()
    }
}

impl<K, V, S, P> Default for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Default,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, S, P> PartialEq for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + PartialEq,
    SoaVec<KeyValuePair<K, V, P>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        let other = (&other.dense, &other.sparse, &other.sparse_vacant_head);
        (dense, sparse, sparse_vacant_head) == other
    }
}

impl<K, V, S, P> Eq for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Eq,
    SoaVec<KeyValuePair<K, V, P>>: Eq,
{
}

impl<K, V, S, P> PartialOrd for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + PartialOrd,
    SoaVec<KeyValuePair<K, V, P>>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        let other = (&other.dense, &other.sparse, &other.sparse_vacant_head);
        (dense, sparse, sparse_vacant_head).partial_cmp(&other)
    }
}

impl<K, V, S, P> Ord for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Ord,
    SoaVec<KeyValuePair<K, V, P>>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        let other = (&other.dense, &other.sparse, &other.sparse_vacant_head);
        (dense, sparse, sparse_vacant_head).cmp(&other)
    }
}

impl<K, V, S, P> Hash for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch> + Hash,
    SoaVec<KeyValuePair<K, V, P>>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;
        (dense, sparse, sparse_vacant_head).hash(state);
    }
}

impl<K, V, S, P> Clone for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    SoaVec<KeyValuePair<K, V, P>>: Clone,
{
    fn clone(&self) -> Self {
        let Self {
            ref dense,
            ref sparse,
            sparse_vacant_head,
        } = *self;

        Self {
            dense: dense.clone(),
            sparse: sparse.clone(),
            sparse_vacant_head,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;
        let Self {
            dense: source_dense,
            sparse: source_sparse,
            sparse_vacant_head: source_sparse_vacant_head,
        } = source;

        dense.clone_from(source_dense);
        sparse.clone_from(source_sparse);
        sparse_vacant_head.clone_from(source_sparse_vacant_head);
    }
}

impl<T, K, V, S, P> Index<K> for EpochSparseArena<K, V, S, P>
where
    K: Key + Debug,
    V: SoaOwned + AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx, 'a> V::Context: SoaContext<'a, V, Refs<'ctx> = &'a T>,
{
    type Output = T;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        Self::index(self, key)
    }
}

impl<T, K, V, S, P> IndexMut<K> for EpochSparseArena<K, V, S, P>
where
    K: Key + Debug,
    V: SoaOwned + AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx, 'a> V::Context: SoaContext<'a, V, Refs<'ctx> = &'a T, RefsMut<'ctx> = &'a mut T>,
{
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        Self::index_mut(self, key)
    }
}

impl<T, K, V, S, P> AsRef<[T]> for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: SoaOwned + AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Into<&'a [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_value_slices().into()
    }
}

impl<T, K, V, S, P> AsMut<[T]> for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: SoaOwned + AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Into<&'a mut [T]>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_value_slices().into()
    }
}

impl<K, V, S, P> AsRef<Self> for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<K, V, S, P> AsMut<Self> for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'a, K, V, S, P> IntoIterator for &'a EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (&'a K, Refs<'a, 'a, V>);
    type IntoIter = Iter<'a, 'a, K, V, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, S, P> IntoIterator for &'a mut EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (&'a K, RefsMut<'a, 'a, V>);
    type IntoIter = IterMut<'a, 'a, K, V, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V, S, P> IntoIterator for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + SoaReadOwned<V>,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V, V, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_items()
    }
}

#[cfg(feature = "rayon")]
impl<'a, K, V, S, P> rayon::iter::IntoParallelIterator for &'a EpochSparseArena<K, V, S, P>
where
    K: Key + Sync,
    V: AllocSoa + Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
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
impl<'a, K, V, S, P> rayon::iter::IntoParallelIterator for &'a mut EpochSparseArena<K, V, S, P>
where
    K: Key + Send + Sync,
    V: AllocSoa + Soa<'a> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Sync,
    V::Fields: Send,
    RefsMut<'a, 'a, V>: Send,
{
    type Item = (&'a K, RefsMut<'a, 'a, V>);
    type Iter = crate::iter::ParIterMut<'a, 'a, K, V, P>;

    #[inline]
    fn into_par_iter(self) -> Self::Iter {
        self.par_iter_mut()
    }
}

impl<K, V, S, P, W> FromIterator<(K, W)> for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + SoaWrite<W> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
    V::Context: Default,
{
    fn from_iter<T: IntoIterator<Item = (K, W)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let iter_len = {
            let (lower, upper) = iter.size_hint();
            upper.unwrap_or(lower)
        };

        let mut me = Self::with_capacity(iter_len, iter_len);
        for (key, value) in iter {
            me.insert_from(key, |context, dst| {
                let Some(dst) = dst else { return };
                unsafe { dst.drop_in_place_then_write(context, value) }
            });
        }

        me
    }
}

impl<K, V, S, P, W> Extend<(K, W)> for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + SoaWrite<W> + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    fn extend<I: IntoIterator<Item = (K, W)>>(&mut self, iter: I) {
        let mut iter = iter.into_iter();
        while let Some((key, value)) = iter.next() {
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

impl<K, V, S, P> From<set::EpochSparseSet<K, V, S, P>> for EpochSparseArena<K, V, S, P>
where
    K: Key,
    V: AllocSoa + ?Sized,
    P: SliceItemPtrs<Item = K>,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    fn from(value: set::EpochSparseSet<K, V, S, P>) -> Self {
        let (dense, sparse) = value.into_parts();
        Self::from_parts(dense, sparse).unwrap_or_else(|_| {
            unreachable!("creation of sparse arena from valid parts should not fail")
        })
    }
}

#[cold]
#[track_caller]
#[inline(never)]
const fn last_vacant_item_failed() -> ! {
    panic!("list of vacant items should not contain any cycles")
}

#[track_caller]
fn last_vacant_item<K, S>(sparse: &mut [S], first_vacant: usize) -> Option<&mut S>
where
    K: Key,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    let old_len = sparse.len();
    if first_vacant >= old_len {
        return None;
    }

    let mut last_vacant = first_vacant;
    for _ in 0..old_len {
        let last_vacant_item = unwrap_sparse_item_mut(sparse, last_vacant);
        let next_vacant = unwrap_next_vacant(last_vacant_item);
        let next_vacant = unwrap_into_usize(next_vacant);
        if next_vacant == old_len {
            // should be `Some(last_vacant_item)`, but the lack of Polonius strikes again
            return Some(unwrap_sparse_item_mut(sparse, last_vacant));
        }
        last_vacant = next_vacant;
    }

    last_vacant_item_failed()
}

fn extend_sparse<K, S>(
    sparse: &mut Vec<S>,
    new_len: usize,
    sparse_vacant_head: &mut usize,
) -> Result<(), TooSmallSparseIndexError<K>>
where
    K: Key,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    let old_len = sparse.len();
    if old_len >= new_len {
        return Ok(());
    }

    let max_vacant = new_len.try_into().map_err(TooSmallSparseIndexError::new)?;
    if let Some(last_vacant_item) = last_vacant_item::<K, S>(sparse, *sparse_vacant_head) {
        let _next_vacant = unwrap_next_vacant(last_vacant_item);
        *last_vacant_item = S::with_next_vacant(last_vacant_item.epoch(), max_vacant);
    }

    let mut next_vacant = if *sparse_vacant_head < old_len {
        *sparse_vacant_head
    } else {
        new_len
    };
    let mut current_vacant = old_len;
    sparse.resize_with(new_len, || {
        let item = S::with_next_vacant(Default::default(), unwrap_into_index(next_vacant));
        next_vacant = current_vacant;
        current_vacant += 1;
        item
    });

    let last_sparse_item = sparse
        .last()
        .expect("sparse should contain at least one item");
    let next_vacant = unwrap_next_vacant(last_sparse_item);
    *sparse_vacant_head = unwrap_into_usize(next_vacant);

    Ok(())
}

fn remove_from_vacant_list<K, S>(
    sparse: &mut [S],
    sparse_vacant_head: &mut usize,
    sparse_index: usize,
    next_vacant: usize,
) where
    K: Key,
    S: ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    let vacant_to_fix = {
        let mut result = None;
        let mut next_vacant = *sparse_vacant_head;
        while next_vacant != sparse_index {
            result = Some(next_vacant);

            let vacant_item = unwrap_sparse_item(sparse, next_vacant);
            next_vacant = unwrap_into_usize(unwrap_next_vacant(vacant_item));
        }
        result
    };

    match vacant_to_fix {
        Some(vacant_to_fix) => {
            let vacant_item = unwrap_sparse_item_mut(sparse, vacant_to_fix);
            let _next_vacant = unwrap_next_vacant(vacant_item);

            let next_vacant = unwrap_into_index(next_vacant);
            *vacant_item = S::with_next_vacant(vacant_item.epoch(), next_vacant);
        }
        None => *sparse_vacant_head = next_vacant,
    }
}

generate_entry_types!(EpochSparseArena<K, V, S>, ArenaSparseItem<Index = K::SparseIndex, Epoch = K::Epoch>);
