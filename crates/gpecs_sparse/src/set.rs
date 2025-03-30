use alloc::vec::Vec;
use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::{Index, IndexMut},
};

use crate::{
    arena,
    assert::{
        check_dense_index_bounds, check_equal_key, check_key_bounds, unwrap_dense,
        unwrap_dense_index_mut, unwrap_into_index, unwrap_into_usize, unwrap_sparse_item_mut,
    },
    entry::generate_entry_types,
    error::{
        InvalidKeyError, TooLargeSparseIndexError, TooSmallSparseIndexError, TryInvalidKeyError,
        TryReserveError,
    },
    item::{SparseItem, SparseItemKind},
    iter::{Drain, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, Values, ValuesMut},
    key::{Epoch, Key},
    pair::{
        KeyValueMutPtrs, KeyValuePair, KeyValuePtrs, KeyValueRefs, KeyValueSlices,
        KeyValueSlicesMut, KeyValueVecs,
    },
    soa::{
        mem::replace as soa_replace,
        slice::{SoaSlices, SoaSlicesMut},
        traits::Soa,
        vec::SoaVec,
    },
    view::{EpochSparseView, EpochSparseViewMut},
};

pub type SparseSet<T> = EpochSparseSet<usize, T>;

pub struct EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
{
    dense: SoaVec<KeyValuePair<K, V>>,
    sparse: Vec<SparseItem<K>>,
}

impl<K, V> EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
{
    #[inline]
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
    pub fn with_context(context: V::Context) -> Self {
        Self {
            dense: SoaVec::with_context(context),
            sparse: Vec::new(),
        }
    }

    #[inline]
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
    pub fn with_context_and_capacity(context: V::Context, dense: usize, sparse: usize) -> Self {
        Self {
            dense: SoaVec::with_context_and_capacity(context, dense),
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
    pub fn as_slices(&self) -> V::Slices<'_> {
        let Self { dense, .. } = self;

        let KeyValueSlices { values, .. } = dense.as_slices();
        values
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> V::SlicesMut<'_> {
        let Self { dense, .. } = self;

        let KeyValueSlicesMut { values, .. } = dense.as_mut_slices();
        values
    }

    #[inline]
    pub fn slices(&self) -> SoaSlices<'_, V> {
        let Self { dense, .. } = self;

        let (context, slices) = dense.slices().into_slices_with_context();
        let KeyValueSlices { values, .. } = slices;
        SoaSlices::new(context, values)
    }

    #[inline]
    pub fn slices_mut(&mut self) -> SoaSlicesMut<'_, V> {
        let Self { dense, .. } = self;

        let (context, slices) = dense.slices_mut().into_slices_with_context();
        let KeyValueSlicesMut { values, .. } = slices;
        SoaSlicesMut::new(context, values)
    }

    #[inline]
    pub fn into_vecs(self) -> (V::Context, V::Vecs) {
        let Self { dense, .. } = self;

        let (context, KeyValueVecs { values, .. }) = dense.into_vecs();
        (context, values)
    }

    #[inline]
    pub fn as_ptrs(&self) -> V::Ptrs {
        let Self { dense, .. } = self;

        let KeyValuePtrs { value, .. } = dense.as_ptrs();
        value
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> V::MutPtrs {
        let Self { dense, .. } = self;

        let KeyValueMutPtrs { value, .. } = dense.as_mut_ptrs();
        value
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { dense, .. } = self;

        let KeyValueSlices { keys, .. } = dense.as_slices();
        keys
    }

    #[inline]
    pub fn into_keys_vec(self) -> Vec<K> {
        let Self { dense, .. } = self;

        let (_, KeyValueVecs { keys, .. }) = dense.into_vecs();
        keys
    }

    #[inline]
    pub fn as_keys_ptr(&self) -> *const K {
        let Self { dense, .. } = self;

        let KeyValuePtrs { key, .. } = dense.as_ptrs();
        key
    }

    #[inline]
    pub fn as_sparse_slice(&self) -> &[SparseItem<K>] {
        let Self { sparse, .. } = self;
        sparse.as_slice()
    }

    #[inline]
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
    pub fn as_view(&self) -> EpochSparseView<'_, K, V> {
        let Self { dense, sparse } = self;
        EpochSparseView::new(dense.slices(), sparse)
    }

    #[inline]
    pub fn as_mut_view(&mut self) -> EpochSparseViewMut<'_, K, V> {
        let Self { dense, sparse } = self;
        EpochSparseViewMut::new(dense.slices_mut(), sparse)
    }

    #[inline]
    pub fn into_parts(self) -> (SoaVec<KeyValuePair<K, V>>, Vec<SparseItem<K>>) {
        let Self { dense, sparse } = self;
        (dense, sparse)
    }

    pub fn from_parts(
        dense: SoaVec<KeyValuePair<K, V>>,
        mut sparse: Vec<SparseItem<K>>,
    ) -> Result<Self, InvalidKeyError<K>> {
        sparse.clear();
        for (dense_index, KeyValueRefs { key, .. }) in dense.slices().into_iter().enumerate() {
            let sparse_index = key
                .sparse_index()
                .try_into()
                .map_err(TooLargeSparseIndexError::new)?;
            let epoch = key.epoch();

            let dense_index = dense_index
                .try_into()
                .map_err(TooSmallSparseIndexError::new)?;
            let item = SparseItem::occupied(dense_index, epoch);

            extend_sparse(&mut sparse, sparse_index.saturating_add(1));
            sparse[sparse_index] = item;
        }

        Ok(Self { dense, sparse })
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, InvalidKeyError<K>> {
        let Self { dense, sparse } = self;

        let sparse_index = key
            .sparse_index()
            .try_into()
            .map_err(TooLargeSparseIndexError::new)?;
        extend_sparse(sparse, sparse_index.saturating_add(1));

        let sparse_item = sparse.index_mut(sparse_index);
        if key.epoch() < sparse_item.epoch {
            return Ok(None);
        }

        if let SparseItemKind::Occupied { dense_index } = sparse_item.kind {
            let (context, dense) = dense.slices_mut().into_slices_with_context();
            let dense = SoaSlicesMut::<KeyValuePair<K, V>>::new(context, dense);

            let dense_index = unwrap_into_usize(dense_index);
            let (dense_key, dense_value) = unwrap_dense(dense, dense_index).into();

            let value = soa_replace(context, dense_value, value);
            sparse_item.epoch = key.epoch();
            *dense_key = key;

            return Ok(Some(value));
        }

        let dense_index = dense
            .len()
            .try_into()
            .map_err(TooSmallSparseIndexError::new)?;
        dense.push(KeyValuePair { key, value });
        *sparse_item = SparseItem::occupied(dense_index, key.epoch());

        Ok(None)
    }

    pub fn try_insert(&mut self, key: K, value: V) -> Result<Option<V>, TryInvalidKeyError<K>> {
        let Self { dense, sparse } = self;

        let sparse_index = key
            .sparse_index()
            .try_into()
            .map_err(TooLargeSparseIndexError::new)?;

        let new_sparse_len = sparse_index.saturating_add(1);
        sparse
            .try_reserve(new_sparse_len.saturating_sub(sparse.len()))
            .map_err(TryReserveError::Sparse)?;
        extend_sparse(sparse, new_sparse_len);

        let sparse_item = sparse.index_mut(sparse_index);
        if key.epoch() < sparse_item.epoch {
            return Ok(None);
        }

        if let SparseItemKind::Occupied { dense_index } = sparse_item.kind {
            let (context, dense) = dense.slices_mut().into_slices_with_context();
            let dense = SoaSlicesMut::<KeyValuePair<K, V>>::new(context, dense);

            let dense_index = unwrap_into_usize(dense_index);
            let (dense_key, dense_value) = unwrap_dense(dense, dense_index).into();

            let value = soa_replace(context, dense_value, value);
            sparse_item.epoch = key.epoch();
            *dense_key = key;

            return Ok(Some(value));
        }

        let dense_index = dense
            .len()
            .try_into()
            .map_err(TooSmallSparseIndexError::new)?;
        dense.try_reserve(1).map_err(TryReserveError::Dense)?;
        dense.push(KeyValuePair { key, value });
        *sparse_item = SparseItem::occupied(dense_index, key.epoch());

        Ok(None)
    }

    pub fn push(&mut self, value: V) -> Result<K, InvalidKeyError<K>> {
        let Self { sparse, .. } = self;

        let (sparse_index, epoch) = sparse
            .iter()
            .enumerate()
            .find(|(_, item)| item.is_vacant())
            .map(|(sparse_index, item)| (sparse_index, item.epoch))
            .unwrap_or((self.sparse.len(), Default::default()));
        let sparse_index = sparse_index
            .try_into()
            .map_err(TooSmallSparseIndexError::new)?;
        let key = K::new(sparse_index, epoch);

        self.insert(key, value)?;
        Ok(key)
    }

    pub fn try_push(&mut self, value: V) -> Result<K, TryInvalidKeyError<K>> {
        let Self { sparse, .. } = self;

        let (sparse_index, epoch) = sparse
            .iter()
            .enumerate()
            .find(|(_, item)| item.is_vacant())
            .map(|(sparse_index, item)| (sparse_index, item.epoch))
            .unwrap_or((self.sparse.len(), Default::default()));
        let sparse_index = sparse_index
            .try_into()
            .map_err(TooSmallSparseIndexError::new)?;
        let key = K::new(sparse_index, epoch);

        self.try_insert(key, value)?;
        Ok(key)
    }

    #[inline]
    pub fn swap(&mut self, first_key: K, second_key: K) {
        let mut view_mut = self.as_mut_view();
        view_mut.swap(first_key, second_key)
    }

    #[inline]
    pub fn swap_keys(&mut self, first_key: K, second_key: K) {
        let mut view_mut = self.as_mut_view();
        view_mut.swap_keys(first_key, second_key)
    }

    pub fn swap_remove(&mut self, key: K) -> Option<V> {
        let Self { dense, sparse } = self;

        let sparse_index = key.sparse_index().try_into().ok()?;
        let dense_index = *sparse
            .get(sparse_index)
            .take_if(|item| item.epoch == key.epoch())
            .and_then(SparseItem::dense_index)?;
        let dense_index_usize = unwrap_into_usize(dense_index);
        check_dense_index_bounds(dense_index_usize, dense.len());

        let (dense_key, value) = dense.swap_remove(dense_index_usize).into();
        check_equal_key(key, dense_key);

        if let Some(KeyValueRefs { key, .. }) = dense.slices().into_get(dense_index_usize) {
            let sparse_index = unwrap_into_usize(key.sparse_index());
            let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);
            if let Some(swapped_dense_index) = sparse_item.dense_index_mut() {
                *swapped_dense_index = dense_index;
            }
        }
        sparse[sparse_index] = SparseItem::vacant(unwrap_into_index(0), key.epoch().next());

        Some(value)
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        let Self { dense, sparse } = self;

        let sparse_index = key.sparse_index().try_into().ok()?;
        let dense_index = sparse
            .get(sparse_index)
            .take_if(|item| item.epoch == key.epoch())
            .and_then(SparseItem::dense_index)?;
        let dense_index = unwrap_into_usize(*dense_index);
        check_dense_index_bounds(dense_index, dense.len());

        let (dense_key, value) = dense.remove(dense_index).into();
        check_equal_key(key, dense_key);

        for KeyValueRefs { key, .. } in dense.slices().into_iter().skip(dense_index) {
            let sparse_index = unwrap_into_usize(key.sparse_index());
            let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);
            let dense_index = unwrap_dense_index_mut(sparse_item.kind_mut());
            *dense_index = unwrap_into_index(unwrap_into_usize(*dense_index) - 1);
        }
        sparse[sparse_index] = SparseItem::vacant(unwrap_into_index(0), key.epoch().next());

        Some(value)
    }

    pub fn pop(&mut self) -> Option<(K, V)> {
        let Self { dense, sparse } = self;

        let KeyValuePair { key, value } = dense.pop()?;

        let sparse_index = unwrap_into_usize(key.sparse_index());
        check_key_bounds(sparse_index, sparse.len());

        sparse[sparse_index] = SparseItem::vacant(unwrap_into_index(0), key.epoch().next());

        Some((key, value))
    }

    #[inline]
    pub fn invalidate_epoch(&mut self, key: K) -> Option<K> {
        let mut view_mut = self.as_mut_view();
        view_mut.invalidate_epoch(key)
    }

    #[inline]
    pub fn replace_key(&mut self, key: K) -> Option<K> {
        let mut view_mut = self.as_mut_view();
        view_mut.replace_key(key)
    }

    pub fn truncate(&mut self, dense_len: usize, sparse_len: usize) {
        for dense_index in (dense_len..self.len()).rev() {
            let (&key, _) = self.dense.slices().into_index(dense_index).into();
            self.remove(key);
        }
        self.dense.truncate(dense_len);

        for sparse_index in sparse_len..self.sparse_len() {
            let epoch = self.sparse[sparse_index].epoch;
            let key = K::new(unwrap_into_index(sparse_index), epoch.next());
            self.remove(key);
        }
        self.sparse.truncate(sparse_len);
    }

    #[inline]
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        let Self { dense, sparse } = self;

        for KeyValueRefs { key, .. } in dense.slices() {
            let sparse_index = unwrap_into_usize(key.sparse_index());
            sparse[sparse_index] = SparseItem::vacant(unwrap_into_index(0), key.epoch().next());
        }

        Drain::new(dense.drain(..))
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(K, V::RefsMut<'_>) -> bool,
    {
        let old_len = self.len();
        let Self { dense, sparse } = self;

        let mut last = 0;
        for curr in 0..old_len {
            let (&mut key, value) = dense.slices_mut().into_index_mut(curr).into();
            if !f(key, value) {
                let sparse_index = unwrap_into_usize(key.sparse_index());
                sparse[sparse_index] = SparseItem::vacant(unwrap_into_index(0), key.epoch().next());
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
        for<'a> V::Refs<'a>: Ord,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort()
    }

    #[inline]
    pub fn sort_keys(&mut self) {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_keys()
    }

    #[inline]
    pub fn sort_by<F>(&mut self, f: F)
    where
        F: FnMut((K, V::Refs<'_>), (K, V::Refs<'_>)) -> cmp::Ordering,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_by(f)
    }

    #[inline]
    pub fn sort_by_key<T, F>(&mut self, f: F)
    where
        F: FnMut((K, V::Refs<'_>)) -> T,
        T: Ord,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_by_key(f)
    }

    #[inline]
    pub fn sort_by_cached_key<T, F>(&mut self, f: F)
    where
        F: FnMut((K, V::Refs<'_>)) -> T,
        T: Ord,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_by_cached_key(f)
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        for<'any> V::Refs<'any>: Ord,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_unstable()
    }

    #[inline]
    pub fn sort_keys_unstable(&mut self) {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_keys_unstable()
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, f: F)
    where
        F: FnMut((K, V::Refs<'_>), (K, V::Refs<'_>)) -> cmp::Ordering,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_unstable_by(f)
    }

    #[inline]
    pub fn sort_unstable_by_key<T, F>(&mut self, f: F)
    where
        F: FnMut((K, V::Refs<'_>)) -> T,
        T: Ord,
    {
        let mut view_mut = self.as_mut_view();
        view_mut.sort_unstable_by_key(f)
    }

    #[inline]
    pub fn get(&self, key: K) -> Option<V::Refs<'_>> {
        let view = self.as_view();
        view.into_get(key)
    }

    #[inline]
    pub fn get_mut(&mut self, key: K) -> Option<V::RefsMut<'_>> {
        let view_mut = self.as_mut_view();
        view_mut.into_get_mut(key)
    }

    #[inline]
    pub fn index(&self, key: K) -> V::Refs<'_>
    where
        K: Debug,
    {
        let view = self.as_view();
        view.into_index(key)
    }

    #[inline]
    pub fn index_mut(&mut self, key: K) -> V::RefsMut<'_>
    where
        K: Debug,
    {
        let view_mut = self.as_mut_view();
        view_mut.into_index_mut(key)
    }

    #[inline]
    pub fn get_with_key(&self, sparse_index: K::SparseIndex) -> Option<(K, V::Refs<'_>)> {
        let view = self.as_view();
        view.into_get_with_key(sparse_index)
    }

    #[inline]
    pub fn get_mut_with_key(
        &mut self,
        sparse_index: K::SparseIndex,
    ) -> Option<(K, V::RefsMut<'_>)> {
        let view_mut = self.as_mut_view();
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

    pub fn entry(&mut self, key: K) -> Result<Entry<'_, K, V>, TooLargeSparseIndexError<K>> {
        let Self { dense, sparse } = self;

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
        let Self { dense, sparse } = self;

        for KeyValueRefs { key, .. } in dense.slices() {
            let sparse_index = unwrap_into_usize(key.sparse_index());
            sparse[sparse_index] = SparseItem::vacant(unwrap_into_index(0), key.epoch().next());
        }
        dense.clear();
    }

    #[inline]
    pub fn clear_sparse(&mut self) {
        let Self { dense, sparse } = self;

        sparse.clear();
        dense.clear();
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        let Self { dense, .. } = self;
        let inner = dense.slices().into_iter();
        Keys::new(inner)
    }

    #[inline]
    pub fn into_keys(self) -> IntoKeys<K, V> {
        let Self { dense, .. } = self;
        let inner = dense.into_iter();
        IntoKeys::new(inner)
    }

    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        let Self { dense, .. } = self;
        let inner = dense.slices().into_iter();
        Values::new(inner)
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        let view_mut = self.as_mut_view();
        view_mut.into_values_mut()
    }

    #[inline]
    pub fn into_values(self) -> IntoValues<K, V> {
        let Self { dense, .. } = self;
        let inner = dense.into_iter();
        IntoValues::new(inner)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        let view = self.as_view();
        view.into_iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        let view_mut = self.as_mut_view();
        view_mut.into_iter()
    }
}

impl<K, V> Debug for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
    SparseItem<K>: Debug,
    SoaVec<KeyValuePair<K, V>>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EpochSparseSet")
            .field("dense", &self.dense)
            .field("sparse", &self.sparse)
            .finish()
    }
}

impl<K, V> Default for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
    V::Context: Default,
{
    fn default() -> Self {
        Self {
            dense: Default::default(),
            sparse: Default::default(),
        }
    }
}

impl<K, V> PartialEq for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
    SoaVec<KeyValuePair<K, V>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.dense == other.dense && self.sparse == other.sparse
    }
}

impl<K, V> Eq for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
    SoaVec<KeyValuePair<K, V>>: Eq,
{
}

impl<K, V> PartialOrd for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
    SoaVec<KeyValuePair<K, V>>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.dense.partial_cmp(&other.dense) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.sparse.partial_cmp(&other.sparse)
    }
}

impl<K, V> Ord for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
    SoaVec<KeyValuePair<K, V>>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.dense.cmp(&other.dense) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.sparse.cmp(&other.sparse)
    }
}

impl<K, V> Hash for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
    SparseItem<K>: Hash,
    SoaVec<KeyValuePair<K, V>>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.dense.hash(state);
        self.sparse.hash(state);
    }
}

impl<K, V> Clone for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
    SoaVec<KeyValuePair<K, V>>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            dense: self.dense.clone(),
            sparse: self.sparse.clone(),
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

impl<T, K, V> Index<K> for EpochSparseSet<K, V>
where
    K: Key + Debug,
    for<'a> V: Soa<Refs<'a> = &'a T> + 'a,
{
    type Output = T;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        EpochSparseSet::index(self, key)
    }
}

impl<T, K, V> IndexMut<K> for EpochSparseSet<K, V>
where
    K: Key + Debug,
    for<'a> V: Soa<Refs<'a> = &'a T, RefsMut<'a> = &'a mut T> + 'a,
{
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        EpochSparseSet::index_mut(self, key)
    }
}

impl<T, K, V> AsRef<[T]> for EpochSparseSet<K, V>
where
    K: Key,
    for<'a> V: Soa<Slices<'a> = &'a [T]> + 'a,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slices()
    }
}

impl<T, K, V> AsMut<[T]> for EpochSparseSet<K, V>
where
    K: Key,
    for<'a> V: Soa<SlicesMut<'a> = &'a mut [T]> + 'a,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slices()
    }
}

impl<K, V> AsRef<EpochSparseSet<K, V>> for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
{
    #[inline]
    fn as_ref(&self) -> &EpochSparseSet<K, V> {
        self
    }
}

impl<K, V> AsMut<EpochSparseSet<K, V>> for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
{
    #[inline]
    fn as_mut(&mut self) -> &mut EpochSparseSet<K, V> {
        self
    }
}

impl<'a, K, V> IntoIterator for &'a EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
{
    type Item = (&'a K, V::Refs<'a>);

    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
{
    type Item = (&'a K, V::RefsMut<'a>);

    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V> IntoIterator for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
{
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { dense, .. } = self;
        IntoIter::new(dense.into_iter())
    }
}

impl<K, V> FromIterator<KeyValuePair<K, V>> for EpochSparseSet<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa,
    V::Context: Default,
{
    fn from_iter<I: IntoIterator<Item = KeyValuePair<K, V>>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let iter_len = {
            let (lower, upper) = iter.size_hint();
            upper.unwrap_or(lower)
        };

        let mut me = Self::with_capacity(iter_len, iter_len);
        for KeyValuePair { key, value } in iter {
            me.insert(key, value).unwrap();
        }

        me
    }
}

impl<K, V> FromIterator<(K, V)> for EpochSparseSet<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa,
    V::Context: Default,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        iter.into_iter().map(KeyValuePair::from).collect()
    }
}

impl<K, V> FromIterator<V> for EpochSparseSet<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa,
    V::Context: Default,
{
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        let dense: SoaVec<_> = iter
            .into_iter()
            .enumerate()
            .map(|(sparse_index, value)| KeyValuePair {
                key: K::new(sparse_index, Default::default()),
                value,
            })
            .collect();
        let len = dense.len();

        let sparse = (0..len)
            .map(|dense_index| SparseItem::occupied(dense_index, Default::default()))
            .collect();

        Self { dense, sparse }
    }
}

impl<K, V> Extend<KeyValuePair<K, V>> for EpochSparseSet<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa,
{
    fn extend<I: IntoIterator<Item = KeyValuePair<K, V>>>(&mut self, iter: I) {
        let mut iter = iter.into_iter();
        while let Some(KeyValuePair { key, value }) = iter.next() {
            if self.len() == self.capacity() {
                let (lower, _) = iter.size_hint();
                self.reserve(lower.saturating_add(1), 0);
            }
            self.insert(key, value).unwrap();
        }
    }
}

impl<K, V> Extend<(K, V)> for EpochSparseSet<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa,
{
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        self.extend(iter.into_iter().map(KeyValuePair::from))
    }
}

impl<K, V> Extend<V> for EpochSparseSet<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa,
{
    fn extend<I: IntoIterator<Item = V>>(&mut self, iter: I) {
        // I could have used `push` here, but it would search for a vacant sparse item
        // multiple times from the beginning of a sparse
        let mut maybe_vacant_keys = (0..self.sparse.len()).fuse();

        let mut iter = iter.into_iter();
        while let Some(value) = iter.next() {
            if self.len() == self.capacity() {
                let (lower, _) = iter.size_hint();
                self.reserve(lower.saturating_add(1), lower.saturating_add(1));
            }
            let sparse_index = maybe_vacant_keys
                .find(|&key| self.sparse[key].is_vacant())
                .unwrap_or(self.sparse.len());
            let key = K::new(sparse_index, Default::default());
            self.insert(key, value).unwrap();
        }
    }
}

impl<K, V> From<arena::EpochSparseArena<K, V>> for EpochSparseSet<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa,
{
    #[inline]
    fn from(value: arena::EpochSparseArena<K, V>) -> Self {
        let (dense, sparse) = value.into_parts();
        Self { dense, sparse }
    }
}

fn extend_sparse<K>(sparse: &mut Vec<SparseItem<K>>, new_len: usize)
where
    K: Key,
{
    let old_len = sparse.len();
    if old_len >= new_len {
        return;
    }

    let epoch = Default::default();
    let item = SparseItem::vacant(unwrap_into_index(0), epoch);
    sparse.resize(new_len, item);
}

generate_entry_types!(EpochSparseSet<K, V>);
