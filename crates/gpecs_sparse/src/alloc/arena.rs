use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::{Index, IndexMut},
    ptr,
};
use core_alloc::vec::Vec;

use crate::{
    assert::{
        check_dense_index_bounds, check_equal_key, check_key_bounds, unwrap_dense,
        unwrap_dense_index_mut, unwrap_into_index, unwrap_into_usize, unwrap_next_vacant,
        unwrap_next_vacant_mut, unwrap_sparse_item, unwrap_sparse_item_mut,
    },
    error::{
        FromPartsError, TooLargeSparseIndexError, TooSmallSparseIndexError, TryModifyError,
        TryModifyErrorKind, TryReserveError,
    },
    item::{
        DenseContext, DenseItem, DenseMutPtrs, DensePtrs, DenseRefs, SparseItem, SparseItemKind,
    },
    iter::{
        Drain, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, RawIter, RawIterMut, RawKeys,
        RawValues, RawValuesMut, Values, ValuesMut,
    },
    key::{Epoch, Key},
    soa::{
        mem::replace as soa_replace,
        slice::{SoaSlices, SoaSlicesMut},
        traits::{MutPtrs, Ptrs, RawSoaContext, Soa, SoaRead, SoaWrite},
        vec::SoaVec,
    },
    view::{EpochSparseView, EpochSparseViewMut, EpochSparseViewMutPtr, EpochSparseViewPtr},
};

use super::{
    access::{TryInsertAccess, drop_old_then_write},
    assert::{try_entry_failed, try_insert_failed, try_push_failed},
    entry::generate_entry_types,
    set,
};

pub type SparseArena<T> = EpochSparseArena<usize, T>;

pub struct EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    dense: SoaVec<DenseItem<K, V>>,
    sparse: Vec<SparseItem<K>>,
    sparse_vacant_head: usize,
}

impl<K, V> EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
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
        let context = DenseContext::from_inner(context);
        Self {
            dense: SoaVec::with_context(context),
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
        let context = DenseContext::from_inner(context);
        Self {
            dense: SoaVec::with_context_and_capacity(context, dense),
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
    pub fn as_slices(&self) -> V::Slices<'_, '_> {
        let Self { dense, .. } = self;

        let (_, values) = dense.as_slices().into_parts();
        values
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> V::SlicesMut<'_, '_> {
        let Self { dense, .. } = self;

        let (_, values) = dense.as_mut_slices().into_parts();
        values
    }

    #[inline]
    pub fn slices(&self) -> SoaSlices<'_, '_, V> {
        let Self { dense, .. } = self;

        let (context, slices) = dense.slices().into_slices_with_context();
        let (_, values) = slices.into_parts();
        SoaSlices::new(context.as_inner(), values)
    }

    #[inline]
    pub fn slices_mut(&mut self) -> SoaSlicesMut<'_, '_, V> {
        let Self { dense, .. } = self;

        let (context, slices) = dense.slices_mut().into_slices_with_context();
        let (_, values) = slices.into_parts();
        SoaSlicesMut::new(context.as_inner(), values)
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'_, V> {
        let Self { dense, .. } = self;

        let DensePtrs { value, .. } = dense.as_ptrs();
        value.into_inner()
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'_, V> {
        let Self { dense, .. } = self;

        let DenseMutPtrs { value, .. } = dense.as_mut_ptrs();
        value.into_inner()
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { dense, .. } = self;

        let (keys, _) = dense.as_slices().into_parts();
        keys
    }

    #[inline]
    pub unsafe fn as_keys_slice_mut(&mut self) -> &mut [K] {
        let Self { dense, .. } = self;

        let (keys, _) = dense.as_mut_slices().into_parts();
        keys
    }

    #[inline]
    pub fn as_keys_ptr(&self) -> *const K {
        let Self { dense, .. } = self;

        let DensePtrs { key, .. } = dense.as_ptrs();
        key
    }

    #[inline]
    pub unsafe fn as_keys_ptr_mut(&mut self) -> *mut K {
        let Self { dense, .. } = self;

        let DenseMutPtrs { key, .. } = dense.as_mut_ptrs();
        key
    }

    #[inline]
    pub fn as_sparse_slice(&self) -> &[SparseItem<K>] {
        let Self { sparse, .. } = self;
        sparse.as_slice()
    }

    #[inline]
    pub unsafe fn as_sparse_slice_mut(&mut self) -> &mut [SparseItem<K>] {
        let Self { sparse, .. } = self;
        sparse.as_mut_slice()
    }

    #[inline]
    #[must_use]
    pub fn into_sparse_vec(self) -> Vec<SparseItem<K>> {
        let Self { sparse, .. } = self;
        sparse
    }

    #[inline]
    pub fn as_sparse_ptr(&self) -> *const SparseItem<K> {
        let Self { sparse, .. } = self;
        sparse.as_ptr()
    }

    #[inline]
    pub unsafe fn as_sparse_ptr_mut(&mut self) -> *mut SparseItem<K> {
        let Self { sparse, .. } = self;
        sparse.as_mut_ptr()
    }

    #[inline]
    pub fn as_view_ptr(&self) -> EpochSparseViewPtr<'_, K, V> {
        let Self { dense, sparse, .. } = self;
        unsafe { EpochSparseViewPtr::from_parts(dense.slice_ptrs(), sparse.as_slice()) }
    }

    #[inline]
    pub fn as_view(&self) -> EpochSparseView<'_, '_, K, V> {
        unsafe { self.as_view_ptr().deref() }
    }

    #[inline]
    pub fn as_view_mut_ptr(&mut self) -> EpochSparseViewMutPtr<'_, K, V> {
        let Self { dense, sparse, .. } = self;
        unsafe { EpochSparseViewMutPtr::from_parts(dense.slice_mut_ptrs(), sparse.as_mut_slice()) }
    }

    #[inline]
    pub fn as_view_mut(&mut self) -> EpochSparseViewMut<'_, '_, K, V> {
        unsafe { self.as_view_mut_ptr().deref_mut() }
    }

    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (SoaVec<DenseItem<K, V>>, Vec<SparseItem<K>>) {
        let Self { dense, sparse, .. } = self;
        (dense, sparse)
    }

    #[inline]
    #[track_caller]
    pub fn from_parts(
        dense: SoaVec<DenseItem<K, V>>,
        mut sparse: Vec<SparseItem<K>>,
    ) -> Result<Self, FromPartsError<K>> {
        let _ = EpochSparseView::new(dense.slices(), sparse.as_slice())?;

        sparse.clear();
        let mut sparse_vacant_head = 0;
        for (dense_index, DenseRefs { key, .. }) in dense.slices().iter().enumerate() {
            let sparse_index = key
                .sparse_index()
                .try_into()
                .map_err(TooLargeSparseIndexError::new)?;
            let epoch = key.epoch();

            let dense_index = dense_index
                .try_into()
                .map_err(TooSmallSparseIndexError::new)?;
            let item = SparseItem::occupied(dense_index, epoch);

            if sparse_index >= sparse.len() {
                let new_len = sparse_index.saturating_add(1);
                extend_sparse(&mut sparse, new_len, &mut sparse_vacant_head)?;
            } else {
                let sparse_item = unwrap_sparse_item(sparse.as_slice(), sparse_index);
                let next_vacant = (*unwrap_next_vacant(sparse_item.kind()))
                    .try_into()
                    .map_err(TooLargeSparseIndexError::new)?;
                remove_from_vacant_list(
                    &mut sparse,
                    &mut sparse_vacant_head,
                    sparse_index,
                    next_vacant,
                );
            }
            sparse[sparse_index] = item;
        }

        Ok(Self {
            dense,
            sparse,
            sparse_vacant_head,
        })
    }

    #[inline]
    pub fn swap(&mut self, first_key: K, second_key: K) {
        let mut view_mut = self.as_view_mut();
        view_mut.swap(first_key, second_key);
    }

    #[inline]
    pub fn swap_keys(&mut self, first_key: K, second_key: K) {
        let mut view_mut = self.as_view_mut();
        view_mut.swap_keys(first_key, second_key);
    }

    #[inline]
    pub fn invalidate_epoch(&mut self, key: K) -> Option<K> {
        let mut view_mut = self.as_view_mut();
        view_mut.invalidate_epoch(key)
    }

    #[inline]
    pub fn replace_key(&mut self, key: K) -> Option<K> {
        let mut view_mut = self.as_view_mut();
        view_mut.replace_key(key)
    }

    pub fn truncate(&mut self, dense_len: usize, sparse_len: usize) {
        for dense_index in (dense_len..self.len()).rev() {
            let (&key, _) = self.dense.slices().into_index(dense_index).into();

            self.remove_into(key, |context, src| {
                let Some(value) = src else { return };
                let value = V::Context::upcast_ptrs(value);
                let value = context.ptrs_cast_mut(value);
                unsafe { context.ptrs_drop_in_place(value) }
            });
        }
        self.dense.truncate(dense_len);

        for sparse_index in sparse_len..self.sparse_len() {
            let epoch = self.sparse[sparse_index].epoch;
            let key = K::new(unwrap_into_index(sparse_index), epoch.next());

            self.remove_into(key, |context, src| {
                let Some(value) = src else { return };
                let value = V::Context::upcast_ptrs(value);
                let value = context.ptrs_cast_mut(value);
                unsafe { context.ptrs_drop_in_place(value) }
            });
        }
        self.sparse.truncate(sparse_len);
    }

    #[inline]
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        for DenseRefs { key, .. } in dense.slices() {
            let sparse_index = unwrap_into_usize(key.sparse_index());

            let next_vacant = unwrap_into_index(*sparse_vacant_head);
            sparse[sparse_index] = SparseItem::vacant(next_vacant, key.epoch().next());
            *sparse_vacant_head = sparse_index;
        }

        Drain::new(dense.drain(..))
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(K, V::RefsMut<'_, '_>) -> bool,
    {
        let old_len = self.len();
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        let mut last = 0;
        for curr in 0..old_len {
            let (&mut key, value) = dense.slices_mut().into_index_mut(curr).into();
            if !f(key, value) {
                let sparse_index = unwrap_into_usize(key.sparse_index());

                let next_vacant = unwrap_into_index(*sparse_vacant_head);
                sparse[sparse_index] = SparseItem::vacant(next_vacant, key.epoch().next());
                *sparse_vacant_head = sparse_index;
                continue;
            }

            dense.slices_mut().swap(curr, last);

            let sparse_index = unwrap_into_usize(key.sparse_index());
            let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);
            let dense_index = unwrap_dense_index_mut(sparse_item.kind_mut());
            *dense_index = unwrap_into_index(unwrap_into_usize(*dense_index) - (curr - last));

            last += 1;
        }

        dense.truncate(last);
    }

    #[inline]
    pub fn sort(&mut self)
    where
        for<'c, 'any> V::Refs<'c, 'any>: Ord,
    {
        let mut view_mut = self.as_view_mut();
        view_mut.sort();
    }

    #[inline]
    pub fn sort_keys(&mut self) {
        let mut view_mut = self.as_view_mut();
        view_mut.sort_keys();
    }

    #[inline]
    pub fn sort_by<F>(&mut self, f: F)
    where
        F: FnMut((K, V::Refs<'_, '_>), (K, V::Refs<'_, '_>)) -> cmp::Ordering,
    {
        let mut view_mut = self.as_view_mut();
        view_mut.sort_by(f);
    }

    #[inline]
    pub fn sort_by_key<T, F>(&mut self, f: F)
    where
        F: FnMut((K, V::Refs<'_, '_>)) -> T,
        T: Ord,
    {
        let mut view_mut = self.as_view_mut();
        view_mut.sort_by_key(f);
    }

    #[inline]
    pub fn sort_by_cached_key<T, F>(&mut self, f: F)
    where
        F: FnMut((K, V::Refs<'_, '_>)) -> T,
        T: Ord,
    {
        let mut view_mut = self.as_view_mut();
        view_mut.sort_by_cached_key(f);
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        for<'c, 'any> V::Refs<'c, 'any>: Ord,
    {
        let mut view_mut = self.as_view_mut();
        view_mut.sort_unstable();
    }

    #[inline]
    pub fn sort_keys_unstable(&mut self) {
        let mut view_mut = self.as_view_mut();
        view_mut.sort_keys_unstable();
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, f: F)
    where
        F: FnMut((K, V::Refs<'_, '_>), (K, V::Refs<'_, '_>)) -> cmp::Ordering,
    {
        let mut view_mut = self.as_view_mut();
        view_mut.sort_unstable_by(f);
    }

    #[inline]
    pub fn sort_unstable_by_key<T, F>(&mut self, f: F)
    where
        F: FnMut((K, V::Refs<'_, '_>)) -> T,
        T: Ord,
    {
        let mut view_mut = self.as_view_mut();
        view_mut.sort_unstable_by_key(f);
    }

    #[inline]
    pub fn get(&self, key: K) -> Option<V::Refs<'_, '_>> {
        let view = self.as_view();
        view.into_get(key)
    }

    #[inline]
    pub fn get_mut(&mut self, key: K) -> Option<V::RefsMut<'_, '_>> {
        let view_mut = self.as_view_mut();
        view_mut.into_get_mut(key)
    }

    #[inline]
    pub fn index(&self, key: K) -> V::Refs<'_, '_>
    where
        K: Debug,
    {
        let view = self.as_view();
        view.into_index(key)
    }

    #[inline]
    pub fn index_mut(&mut self, key: K) -> V::RefsMut<'_, '_>
    where
        K: Debug,
    {
        let view_mut = self.as_view_mut();
        view_mut.into_index_mut(key)
    }

    #[inline]
    pub fn get_with_key(&self, sparse_index: K::SparseIndex) -> Option<(K, V::Refs<'_, '_>)> {
        let view = self.as_view();
        view.into_get_with_key(sparse_index)
    }

    #[inline]
    pub fn get_mut_with_key(
        &mut self,
        sparse_index: K::SparseIndex,
    ) -> Option<(K, V::RefsMut<'_, '_>)> {
        let view_mut = self.as_view_mut();
        view_mut.into_get_mut_with_key(sparse_index)
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
    #[track_caller]
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        self.try_entry(key)
            .unwrap_or_else(|error| try_entry_failed(error))
    }

    #[inline]
    pub fn try_entry(&mut self, key: K) -> Result<Entry<'_, K, V>, TooLargeSparseIndexError<K>> {
        let Self { dense, sparse, .. } = self;

        let sparse_index: usize = key
            .sparse_index()
            .try_into()
            .map_err(TooLargeSparseIndexError::new)?;
        let Some(dense_index) = sparse
            .get(sparse_index)
            .take_if(|item| item.epoch == key.epoch())
            .and_then(SparseItem::dense_index)
        else {
            let entry = VacantEntry::new(key, self);
            return Ok(Entry::Vacant(entry));
        };

        let dense_index = unwrap_into_usize(*dense_index);
        check_dense_index_bounds(dense_index, dense.len());
        let entry = OccupiedEntry::new(key, dense_index, self);
        Ok(Entry::Occupied(entry))
    }

    #[inline]
    pub fn clear(&mut self) {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        for DenseRefs { key, .. } in dense.slices() {
            let sparse_index = unwrap_into_usize(key.sparse_index());
            let next_vacant = unwrap_into_index(*sparse_vacant_head);
            sparse[sparse_index] = SparseItem::vacant(next_vacant, key.epoch().next());
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

    #[inline]
    pub fn raw_keys(&self) -> RawKeys<'_, K, V> {
        let view = self.as_view();
        view.into_raw_keys()
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_, '_, K, V> {
        let view = self.as_view();
        view.into_keys()
    }

    #[inline]
    #[must_use]
    #[expect(clippy::unnecessary_to_owned, reason = "false positive")]
    pub fn into_keys(self) -> IntoKeys<K, V> {
        let Self { dense, .. } = self;
        let (keys, _) = dense.as_slices().into_parts();
        let inner = keys.to_vec().into_iter();
        IntoKeys::new(inner)
    }

    #[inline]
    pub fn raw_values(&self) -> RawValues<'_, K, V> {
        let view = self.as_view();
        view.into_raw_values()
    }

    #[inline]
    pub fn values(&self) -> Values<'_, '_, K, V> {
        let view = self.as_view();
        view.into_values()
    }

    #[inline]
    pub fn raw_values_mut(&mut self) -> RawValuesMut<'_, K, V> {
        let view_mut = self.as_view_mut();
        view_mut.into_raw_values_mut()
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, '_, K, V> {
        let view_mut = self.as_view_mut();
        view_mut.into_values_mut()
    }

    #[inline]
    pub fn raw_iter(&self) -> RawIter<'_, K, V> {
        let view = self.as_view();
        view.into_raw_iter()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, '_, K, V> {
        let view = self.as_view();
        view.into_iter()
    }

    #[inline]
    pub fn raw_iter_mut(&mut self) -> RawIterMut<'_, K, V> {
        let view_mut = self.as_view_mut();
        view_mut.into_raw_iter_mut()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, '_, K, V> {
        let view_mut = self.as_view_mut();
        view_mut.into_iter()
    }

    pub fn swap_remove_into<F, R>(&mut self, key: K, f: F) -> R
    where
        F: FnOnce(&V::Context, Option<Ptrs<'_, V>>) -> R,
    {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;
        let context = dense.context();

        let Some(sparse_index) = key.sparse_index().try_into().ok() else {
            return f(context, None);
        };

        let dense_index = sparse
            .get(sparse_index)
            .take_if(|item| item.epoch == key.epoch())
            .and_then(SparseItem::dense_index)
            .copied();
        let Some(dense_index) = dense_index else {
            return f(context, None);
        };
        let dense_index_usize = unwrap_into_usize(dense_index);
        check_dense_index_bounds(dense_index_usize, dense.len());

        let result = dense.swap_remove_into(dense_index_usize, |context, src| {
            let dense_key = unsafe { ptr::read(src.key) };
            check_equal_key(key, dense_key);
            f(context, Some(src.value.into_inner()))
        });

        if let Some(DenseRefs { key, .. }) = dense.slices().into_get(dense_index_usize) {
            let sparse_index = unwrap_into_usize(key.sparse_index());
            let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);
            match sparse_item.kind_mut() {
                SparseItemKind::Occupied { dense_index: index } => *index = dense_index,
                SparseItemKind::Vacant { next_vacant } => *next_vacant = dense_index,
            }
        }
        let next_vacant = unwrap_into_index(*sparse_vacant_head);
        sparse[sparse_index] = SparseItem::vacant(next_vacant, key.epoch().next());
        *sparse_vacant_head = sparse_index;

        result
    }

    pub fn remove_into<F, R>(&mut self, key: K, f: F) -> R
    where
        F: FnOnce(&V::Context, Option<Ptrs<'_, V>>) -> R,
    {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;
        let context = dense.context();

        let Some(sparse_index) = key.sparse_index().try_into().ok() else {
            return f(context, None);
        };

        let dense_index = sparse
            .get(sparse_index)
            .take_if(|item| item.epoch == key.epoch())
            .and_then(SparseItem::dense_index)
            .copied();
        let Some(dense_index) = dense_index else {
            return f(context, None);
        };
        let dense_index = unwrap_into_usize(dense_index);
        check_dense_index_bounds(dense_index, dense.len());

        let result = dense.remove_into(dense_index, |context, src| {
            let dense_key = unsafe { ptr::read(src.key) };
            check_equal_key(key, dense_key);
            f(context, Some(src.value.into_inner()))
        });

        for DenseRefs { key, .. } in dense.slices().into_iter().skip(dense_index) {
            let sparse_index = unwrap_into_usize(key.sparse_index());
            let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);
            let dense_index = unwrap_dense_index_mut(sparse_item.kind_mut());
            *dense_index = unwrap_into_index(unwrap_into_usize(*dense_index) - 1);
        }
        let next_vacant = unwrap_into_index(*sparse_vacant_head);
        sparse[sparse_index] = SparseItem::vacant(next_vacant, key.epoch().next());
        *sparse_vacant_head = sparse_index;

        result
    }

    pub fn pop_into<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&V::Context, Option<(K, Ptrs<'_, V>)>) -> R,
    {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        dense.pop_into(|context, src| {
            let Some(DensePtrs { key, value }) = src else {
                return f(context, None);
            };
            let key = unsafe { ptr::read(key) };

            let sparse_index = unwrap_into_usize(key.sparse_index());
            check_key_bounds(sparse_index, sparse.len());

            let result = f(context, Some((key, value.into_inner())));

            let next_vacant = unwrap_into_index(*sparse_vacant_head);
            sparse[sparse_index] = SparseItem::vacant(next_vacant, key.epoch().next());
            *sparse_vacant_head = sparse_index;

            result
        })
    }

    #[inline]
    #[track_caller]
    pub fn insert_from<F, R>(&mut self, key: K, f: F) -> R
    where
        F: FnOnce(&V::Context, Option<TryInsertAccess<V>>) -> R,
    {
        self.try_insert_from(key, |context, dst| {
            let dst = dst.unwrap_or_else(|error| try_insert_failed(error));
            f(context, dst)
        })
    }

    pub fn try_insert_from<F, R>(&mut self, key: K, f: F) -> R
    where
        F: FnOnce(&V::Context, Result<Option<TryInsertAccess<V>>, TryModifyErrorKind<K>>) -> R,
    {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;
        let context = dense.context();

        let sparse_index: usize = match key.sparse_index().try_into() {
            Ok(sparse_index) => sparse_index,
            Err(error) => {
                let error = TooLargeSparseIndexError::new(error).into();
                return f(context, Err(error));
            }
        };

        match sparse.get_mut(sparse_index) {
            Some(sparse_item) if key.epoch() >= sparse_item.epoch => match sparse_item.kind {
                SparseItemKind::Occupied { dense_index } => {
                    let (context, dense) = dense.slices_mut().into_slices_with_context();
                    let dense = SoaSlicesMut::<DenseItem<K, V>>::new(context, dense);

                    let dense_index = unwrap_into_usize(dense_index);
                    let (dense_key, dense_value) = unwrap_dense(dense, dense_index).into();
                    let result = f(context, Ok(Some(TryInsertAccess::read_write(dense_value))));

                    sparse_item.epoch = key.epoch();
                    *dense_key = key;

                    result
                }
                SparseItemKind::Vacant { next_vacant } => {
                    let next_vacant = unwrap_into_usize(next_vacant);
                    remove_from_vacant_list(sparse, sparse_vacant_head, sparse_index, next_vacant);

                    let dense_index = match dense.len().try_into() {
                        Ok(dense_index) => dense_index,
                        Err(error) => {
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

                        sparse[sparse_index] = SparseItem::occupied(dense_index, key.epoch());
                        unsafe { ptr::write(key_ptr, key) }

                        result
                    })
                }
            },
            Some(_) => f(context, Ok(None)),
            None => {
                let new_sparse_len = sparse_index.saturating_add(1);
                let additional = new_sparse_len.saturating_sub(sparse.len());
                if let Err(error) = sparse.try_reserve(additional) {
                    let error = TryReserveError::Sparse(error).into();
                    return f(context, Err(error));
                }
                if let Err(error) = extend_sparse(sparse, new_sparse_len, sparse_vacant_head) {
                    let error = error.into();
                    return f(context, Err(error));
                }

                let dense_index: K::SparseIndex = match dense.len().try_into() {
                    Ok(dense_index) => dense_index,
                    Err(error) => {
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

                    sparse[sparse_index] = SparseItem::occupied(dense_index, key.epoch());
                    unsafe { ptr::write(key_ptr, key) }

                    result
                })
            }
        }
    }

    #[inline]
    #[track_caller]
    pub fn push_from<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&V::Context, K, MutPtrs<'_, V>) -> R,
    {
        self.try_push_from(|context, dst| {
            let (key, dst) = dst.unwrap_or_else(|error| try_insert_failed(error));
            f(context, key, dst)
        })
    }

    pub fn try_push_from<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&V::Context, Result<(K, MutPtrs<'_, V>), TryModifyErrorKind<K>>) -> R,
    {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;
        let context = dense.context();

        if let Some(sparse_item) = sparse.get_mut(*sparse_vacant_head) {
            let next_vacant = match (*unwrap_next_vacant(sparse_item.kind())).try_into() {
                Ok(next_vacant) => next_vacant,
                Err(error) => {
                    let error = TooLargeSparseIndexError::new(error).into();
                    return f(context, Err(error));
                }
            };

            let sparse_index = match (*sparse_vacant_head).try_into() {
                Ok(sparse_index) => sparse_index,
                Err(error) => {
                    let error = TooSmallSparseIndexError::new(error).into();
                    return f(context, Err(error));
                }
            };
            let key = K::new(sparse_index, sparse_item.epoch);

            let dense_index = match dense.len().try_into() {
                Ok(dense_index) => dense_index,
                Err(error) => {
                    let error = TooSmallSparseIndexError::new(error).into();
                    return f(context, Err(error));
                }
            };
            let sparse_item_kind = SparseItemKind::occupied(dense_index);

            if let Err(error) = dense.try_reserve(1) {
                let context = dense.context();
                let error = TryReserveError::Dense(error).into();
                return f(context, Err(error));
            }

            return dense.push_from(|context, dst| {
                let (key_ptr, value_ptrs) = dst.into();
                let result = f(context, Ok((key, value_ptrs)));

                sparse_item.kind = sparse_item_kind;
                *sparse_vacant_head = next_vacant;
                unsafe { ptr::write(key_ptr, key) }

                result
            });
        }

        let sparse_index = match (*sparse_vacant_head).try_into() {
            Ok(sparse_index) => sparse_index,
            Err(error) => {
                let error = TooSmallSparseIndexError::new(error).into();
                return f(context, Err(error));
            }
        };
        let key = K::new(sparse_index, Default::default());

        let dense_index = match dense.len().try_into() {
            Ok(dense_index) => dense_index,
            Err(error) => {
                let error = TooSmallSparseIndexError::new(error).into();
                return f(context, Err(error));
            }
        };
        let sparse_item = SparseItem::occupied(dense_index, Default::default());

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
            unsafe { ptr::write(key_ptr, key) }

            result
        })
    }
}

impl<K, V> EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + SoaRead,
{
    #[inline]
    pub fn swap_remove(&mut self, key: K) -> Option<V> {
        self.swap_remove_into(key, |context, src| unsafe { V::read(context, src?) }.into())
    }

    #[inline]
    pub fn remove(&mut self, key: K) -> Option<V> {
        self.remove_into(key, |context, src| unsafe { V::read(context, src?) }.into())
    }

    #[inline]
    pub fn pop(&mut self) -> Option<(K, V)> {
        self.pop_into(|context, src| {
            let (key, value) = src?;
            let value = unsafe { V::read(context, value) };
            (key, value).into()
        })
    }

    #[inline]
    #[must_use]
    pub fn into_values(self) -> IntoValues<K, V> {
        let Self { dense, .. } = self;
        let inner = dense.into_iter();
        IntoValues::new(inner)
    }
}

impl<K, V> EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + SoaWrite,
{
    #[inline]
    #[track_caller]
    pub fn push(&mut self, value: V) -> K {
        self.try_push(value)
            .unwrap_or_else(|error| try_push_failed(error.kind))
    }

    #[inline]
    pub fn try_push(&mut self, value: V) -> Result<K, TryModifyError<K, V>> {
        self.try_push_from(|context, dst| match dst {
            Ok((key, dst)) => {
                unsafe { V::write(context, dst, value) }
                Ok(key)
            }
            Err(error) => Err(TryModifyError::new(error, value)),
        })
    }
}

impl<K, V> EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + SoaRead + SoaWrite,
{
    #[inline]
    #[track_caller]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.try_insert(key, value)
            .unwrap_or_else(|error| try_insert_failed(error.kind))
    }

    #[inline]
    pub fn try_insert(&mut self, key: K, value: V) -> Result<Option<V>, TryModifyError<K, V>> {
        self.try_insert_from(key, |context, dst| match dst {
            Ok(Some(TryInsertAccess::ReadWrite(dst))) => {
                let value = soa_replace(context, dst.into_inner(), value);
                Ok(Some(value))
            }
            Ok(Some(TryInsertAccess::WriteOnly(dst))) => {
                unsafe { V::write(context, dst.into_inner(), value) }
                Ok(None)
            }
            Ok(None) => Ok(None),
            Err(error) => Err(TryModifyError::new(error, value)),
        })
    }
}

impl<K, V> Debug for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
    K::SparseIndex: Debug,
    SparseItem<K>: Debug,
    SoaVec<DenseItem<K, V>>: Debug,
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

impl<K, V> Default for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
    V::Context: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> PartialEq for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaVec<DenseItem<K, V>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        *dense == other.dense
            && *sparse == other.sparse
            && *sparse_vacant_head == other.sparse_vacant_head
    }
}

impl<K, V> Eq for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaVec<DenseItem<K, V>>: Eq,
{
}

impl<K, V> PartialOrd for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaVec<DenseItem<K, V>>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        match dense.partial_cmp(&other.dense) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match sparse.partial_cmp(&other.sparse) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        sparse_vacant_head.partial_cmp(&other.sparse_vacant_head)
    }
}

impl<K, V> Ord for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaVec<DenseItem<K, V>>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        match dense.cmp(&other.dense) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match sparse.cmp(&other.sparse) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        sparse_vacant_head.cmp(&other.sparse_vacant_head)
    }
}

impl<K, V> Hash for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
    K::SparseIndex: Hash,
    SparseItem<K>: Hash,
    SoaVec<DenseItem<K, V>>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        dense.hash(state);
        sparse.hash(state);
        sparse_vacant_head.hash(state);
    }
}

impl<K, V> Clone for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaVec<DenseItem<K, V>>: Clone,
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

impl<T, K, V> Index<K> for EpochSparseArena<K, V>
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

impl<T, K, V> IndexMut<K> for EpochSparseArena<K, V>
where
    K: Key + Debug,
    V: Soa + ?Sized,
    for<'c, 'any> V: Soa<Refs<'c, 'any> = &'any T, RefsMut<'c, 'any> = &'any mut T> + 'any,
{
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        Self::index_mut(self, key)
    }
}

impl<T, K, V> AsRef<[T]> for EpochSparseArena<K, V>
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

impl<T, K, V> AsMut<[T]> for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
    for<'c, 'any> V::SlicesMut<'c, 'any>: Into<&'any mut [T]>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slices().into()
    }
}

impl<K, V> AsRef<Self> for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<K, V> AsMut<Self> for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'r, K, V> IntoIterator for &'r EpochSparseArena<K, V>
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

impl<'r, K, V> IntoIterator for &'r mut EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    type Item = (&'r K, V::RefsMut<'r, 'r>);
    type IntoIter = IterMut<'r, 'r, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V> IntoIterator for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + SoaRead,
{
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { dense, .. } = self;
        IntoIter::new(dense.into_iter())
    }
}

impl<K, V> FromIterator<DenseItem<K, V>> for EpochSparseArena<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa + SoaWrite,
    V::Context: Default,
{
    fn from_iter<I: IntoIterator<Item = DenseItem<K, V>>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let iter_len = {
            let (lower, upper) = iter.size_hint();
            upper.unwrap_or(lower)
        };

        let mut me = Self::with_capacity(iter_len, iter_len);
        for DenseItem { key, value } in iter {
            me.insert_from(key, |context, dst| unsafe {
                drop_old_then_write(context, dst, value);
            });
        }

        me
    }
}

impl<K, V> FromIterator<(K, V)> for EpochSparseArena<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa + SoaWrite,
    V::Context: Default,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        iter.into_iter().map(DenseItem::from).collect()
    }
}

impl<K, V> FromIterator<V> for EpochSparseArena<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa + SoaWrite,
    V::Context: Default,
{
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        let dense: SoaVec<_> = iter
            .into_iter()
            .enumerate()
            .map(|(sparse_index, value)| {
                let key = K::new(sparse_index, Default::default());
                DenseItem { key, value }
            })
            .collect();
        let len = dense.len();

        let sparse = (0..len)
            .map(|dense_index| SparseItem::occupied(dense_index, Default::default()))
            .collect();
        let sparse_vacant_head = len;

        Self {
            dense,
            sparse,
            sparse_vacant_head,
        }
    }
}

impl<K, V> Extend<DenseItem<K, V>> for EpochSparseArena<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa + SoaWrite,
{
    fn extend<I: IntoIterator<Item = DenseItem<K, V>>>(&mut self, iter: I) {
        let mut iter = iter.into_iter();
        while let Some(DenseItem { key, value }) = iter.next() {
            if self.len() == self.capacity() {
                let (lower, _) = iter.size_hint();
                self.reserve(lower.saturating_add(1), 0);
            }
            self.insert_from(key, |context, dst| unsafe {
                drop_old_then_write(context, dst, value);
            });
        }
    }
}

impl<K, V> Extend<(K, V)> for EpochSparseArena<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa + SoaWrite,
{
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        self.extend(iter.into_iter().map(DenseItem::from));
    }
}

impl<K, V> Extend<V> for EpochSparseArena<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa + SoaWrite,
{
    fn extend<I: IntoIterator<Item = V>>(&mut self, iter: I) {
        let mut iter = iter.into_iter();
        while let Some(value) = iter.next() {
            if self.len() == self.capacity() {
                let (lower, _) = iter.size_hint();
                self.reserve(lower.saturating_add(1), lower.saturating_add(1));
            }
            self.push(value);
        }
    }
}

impl<K, V> From<set::EpochSparseSet<K, V>> for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: set::EpochSparseSet<K, V>) -> Self {
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
fn last_vacant_item<K>(
    sparse: &mut [SparseItem<K>],
    first_vacant: usize,
) -> Option<&mut SparseItem<K>>
where
    K: Key,
{
    let old_len = sparse.len();
    if first_vacant >= old_len {
        return None;
    }

    let mut last_vacant = first_vacant;
    for _ in 0..old_len {
        let last_vacant_item = unwrap_sparse_item_mut(sparse, last_vacant);
        let next_vacant = unwrap_next_vacant(last_vacant_item.kind_mut());
        let next_vacant = unwrap_into_usize(*next_vacant);
        if next_vacant == old_len {
            // should be `Some(last_vacant_item)`, but the lack of Polonius strikes again
            return Some(unwrap_sparse_item_mut(sparse, last_vacant));
        }
        last_vacant = next_vacant;
    }

    last_vacant_item_failed()
}

fn extend_sparse<K>(
    sparse: &mut Vec<SparseItem<K>>,
    new_len: usize,
    sparse_vacant_head: &mut usize,
) -> Result<(), TooSmallSparseIndexError<K>>
where
    K: Key,
{
    let old_len = sparse.len();
    if old_len >= new_len {
        return Ok(());
    }

    let max_vacant = new_len.try_into().map_err(TooSmallSparseIndexError::new)?;
    if let Some(last_vacant_item) = last_vacant_item(sparse, *sparse_vacant_head) {
        let next_vacant = unwrap_next_vacant_mut(last_vacant_item.kind_mut());
        *next_vacant = max_vacant;
    }

    let mut next_vacant = if *sparse_vacant_head < old_len {
        *sparse_vacant_head
    } else {
        new_len
    };
    let mut current_vacant = old_len;
    sparse.resize_with(new_len, || {
        let epoch = Default::default();
        let item = SparseItem::vacant(unwrap_into_index(next_vacant), epoch);
        next_vacant = current_vacant;
        current_vacant += 1;
        item
    });

    let last_sparse_item = sparse
        .last()
        .expect("sparse should contain at least one item");
    let next_vacant = unwrap_next_vacant(last_sparse_item.kind());
    *sparse_vacant_head = unwrap_into_usize(*next_vacant);

    Ok(())
}

fn remove_from_vacant_list<K>(
    sparse: &mut [SparseItem<K>],
    sparse_vacant_head: &mut usize,
    sparse_index: usize,
    next_vacant: usize,
) where
    K: Key,
{
    let vacant_to_fix = {
        let mut result = None;
        let mut next_vacant = *sparse_vacant_head;
        while next_vacant != sparse_index {
            result = Some(next_vacant);

            let vacant_item = unwrap_sparse_item(sparse, next_vacant);
            next_vacant = unwrap_into_usize(*unwrap_next_vacant(vacant_item.kind()));
        }
        result
    };

    match vacant_to_fix {
        Some(vacant_to_fix) => {
            let vacant_item = unwrap_sparse_item_mut(sparse, vacant_to_fix);
            let next_vacant_mut = unwrap_next_vacant_mut(vacant_item.kind_mut());
            *next_vacant_mut = unwrap_into_index(next_vacant);
        }
        None => *sparse_vacant_head = next_vacant,
    }
}

generate_entry_types!(EpochSparseArena<K, V>);
