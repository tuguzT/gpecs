use alloc::vec::Vec;
use core::{
    cmp,
    fmt::{self, Debug, Display},
    hash::{self, Hash},
    ops::{Deref, DerefMut, Index, IndexMut},
};

use crate::{
    arena,
    assert::{
        check_dense_index_bounds, check_equal_key, check_key_bounds, unwrap_dense,
        unwrap_dense_index_mut, unwrap_sparse_item_mut,
    },
    entry::generate_entry_types,
    error::TryReserveError,
    item::{SparseItem, SparseItemKind},
    iter::{Drain, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, Values, ValuesMut},
    key::{Epoch, Key},
    pair::{
        KeyValueMutPtrs, KeyValuePair, KeyValuePtrs, KeyValueRefs, KeyValueSlices,
        KeyValueSlicesMut, KeyValueVecs,
    },
    soa::{
        mem::replace as soa_replace,
        slice::{SoaSlice, SoaSlices, SoaSlicesMut},
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
    sparse: Vec<SparseItem<K::Epoch>>,
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
    pub fn as_sparse_slice(&self) -> &[SparseItem<K::Epoch>] {
        let Self { sparse, .. } = self;
        sparse.as_slice()
    }

    #[inline]
    pub fn into_sparse_vec(self) -> Vec<SparseItem<K::Epoch>> {
        let Self { sparse, .. } = self;
        sparse
    }

    #[inline]
    pub fn as_sparse_ptr(&self) -> *const SparseItem<K::Epoch> {
        let Self { sparse, .. } = self;
        sparse.as_ptr()
    }

    #[inline]
    pub fn as_view(&self) -> EpochSparseView<'_, K, V> {
        let Self { dense, sparse } = self;
        EpochSparseView::new(dense, sparse)
    }

    #[inline]
    pub fn as_mut_view(&mut self) -> EpochSparseViewMut<'_, K, V> {
        let Self { dense, sparse } = self;
        EpochSparseViewMut::new(dense, sparse)
    }

    #[inline]
    pub fn into_parts(self) -> (SoaVec<KeyValuePair<K, V>>, Vec<SparseItem<K::Epoch>>) {
        let Self { dense, sparse } = self;
        (dense, sparse)
    }

    pub fn from_parts(
        dense: SoaVec<KeyValuePair<K, V>>,
        mut sparse: Vec<SparseItem<K::Epoch>>,
    ) -> Self {
        sparse.clear();
        for (dense_index, KeyValueRefs { key, .. }) in dense.iter().enumerate() {
            let sparse_index = key.sparse_index();
            let epoch = key.epoch();
            let item = SparseItem::occupied(dense_index, epoch);

            extend_sparse(&mut sparse, sparse_index.saturating_add(1));
            sparse[sparse_index] = item;
        }

        Self { dense, sparse }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let Self { dense, sparse } = self;

        let sparse_index = key.sparse_index();
        extend_sparse(sparse, sparse_index.saturating_add(1));

        let sparse_item = sparse.index_mut(sparse_index);
        if key.epoch() < sparse_item.epoch {
            return None;
        }

        if let SparseItemKind::Occupied { dense_index } = sparse_item.kind {
            let (context, dense) = dense.slices_mut().into_slices_with_context();
            let dense = SoaSlicesMut::<KeyValuePair<K, V>>::new(context, dense);
            let (dense_key, dense_value) = unwrap_dense(dense, dense_index).into();

            let value = soa_replace(context, dense_value, value);
            sparse_item.epoch = key.epoch();
            *dense_key = key;

            return Some(value);
        }

        dense.push((key, value).into());
        *sparse_item = SparseItem::occupied(dense.len() - 1, key.epoch());

        None
    }

    pub fn try_insert(&mut self, key: K, value: V) -> Result<Option<V>, TryReserveError> {
        let Self { dense, sparse } = self;

        let sparse_index = key.sparse_index();

        let new_sparse_len = sparse_index.saturating_add(1);
        sparse.try_reserve(new_sparse_len.saturating_sub(sparse.len()))?;
        extend_sparse(sparse, new_sparse_len);

        let sparse_item = sparse.index_mut(sparse_index);
        if key.epoch() < sparse_item.epoch {
            return Ok(None);
        }

        if let SparseItemKind::Occupied { dense_index } = sparse_item.kind {
            let (context, dense) = dense.slices_mut().into_slices_with_context();
            let dense = SoaSlicesMut::<KeyValuePair<K, V>>::new(context, dense);
            let (dense_key, dense_value) = unwrap_dense(dense, dense_index).into();

            let value = soa_replace(context, dense_value, value);
            sparse_item.epoch = key.epoch();
            *dense_key = key;

            return Ok(Some(value));
        }

        dense.try_reserve(1)?;
        dense.push((key, value).into());
        *sparse_item = SparseItem::occupied(dense.len() - 1, key.epoch());

        Ok(None)
    }

    pub fn push(&mut self, value: V) -> K {
        let Self { sparse, .. } = self;

        let (sparse_index, epoch) = sparse
            .iter()
            .enumerate()
            .find(|(_, item)| item.is_vacant())
            .map(|(sparse_index, item)| (sparse_index, item.epoch))
            .unwrap_or((self.sparse.len(), Default::default()));
        let key = K::new(sparse_index, epoch);

        self.insert(key, value);
        key
    }

    pub fn try_push(&mut self, value: V) -> Result<K, TryReserveError> {
        let Self { sparse, .. } = self;

        let (sparse_index, epoch) = sparse
            .iter()
            .enumerate()
            .find(|(_, item)| item.is_vacant())
            .map(|(sparse_index, item)| (sparse_index, item.epoch))
            .unwrap_or((self.sparse.len(), Default::default()));
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

        let sparse_index = key.sparse_index();
        let dense_index = sparse
            .get(sparse_index)
            .take_if(|item| item.epoch == key.epoch())
            .and_then(SparseItem::dense_index)?;
        check_dense_index_bounds(dense_index, dense.len());

        let (dense_key, value) = dense.swap_remove(dense_index).into();
        check_equal_key(key, dense_key);

        if let Some(KeyValueRefs { key, .. }) = dense.get(dense_index) {
            let sparse_index = key.sparse_index();
            let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);
            if let Some(swapped_dense_index) = sparse_item.dense_index_mut() {
                *swapped_dense_index = dense_index;
            }
        }
        sparse[sparse_index] = SparseItem::vacant(0, key.epoch().next());

        Some(value)
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        let Self { dense, sparse } = self;

        let sparse_index = key.sparse_index();
        let dense_index = sparse
            .get(sparse_index)
            .take_if(|item| item.epoch == key.epoch())
            .and_then(SparseItem::dense_index)?;
        check_dense_index_bounds(dense_index, dense.len());

        let (dense_key, value) = dense.remove(dense_index).into();
        check_equal_key(key, dense_key);

        for KeyValueRefs { key, .. } in dense.iter().skip(dense_index) {
            let sparse_index = key.sparse_index();
            let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);
            let dense_index = unwrap_dense_index_mut(sparse_item.kind_mut());
            *dense_index -= 1;
        }
        sparse[sparse_index] = SparseItem::vacant(0, key.epoch().next());

        Some(value)
    }

    pub fn pop(&mut self) -> Option<KeyValuePair<K, V>> {
        let Self { dense, sparse } = self;

        let pair = dense.pop()?;

        let sparse_index = pair.key.sparse_index();
        check_key_bounds(sparse_index, sparse.len());
        sparse[sparse_index] = SparseItem::vacant(0, pair.key.epoch().next());

        Some(pair)
    }

    #[inline]
    pub fn invalidate_epoch(&mut self, key: K) -> Option<K> {
        let mut view_mut = self.as_mut_view();
        view_mut.invalidate_epoch(key)
    }

    pub fn truncate(&mut self, dense_len: usize, sparse_len: usize) {
        for dense_index in (dense_len..self.len()).rev() {
            let (&key, _) = self.dense.deref().index(dense_index).into();
            self.remove(key);
        }
        self.dense.truncate(dense_len);

        for sparse_index in sparse_len..self.sparse_len() {
            let epoch = self.sparse[sparse_index].epoch;
            let key = K::new(sparse_index, epoch.next());
            self.remove(key);
        }
        self.sparse.truncate(sparse_len);
    }

    #[inline]
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        let Self { dense, sparse } = self;

        for KeyValueRefs { key, .. } in dense.iter() {
            let sparse_index = key.sparse_index();
            sparse[sparse_index] = SparseItem::vacant(0, key.epoch().next());
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
            let (&mut key, value) = dense.deref_mut().index_mut(curr).into();
            if !f(key, value) {
                let sparse_index = key.sparse_index();
                sparse[sparse_index] = SparseItem::vacant(0, key.epoch().next());
                continue;
            }

            dense.swap(curr, last);

            let sparse_index = key.sparse_index();
            let sparse_item = unwrap_sparse_item_mut(sparse, sparse_index);
            let dense_index = unwrap_dense_index_mut(sparse_item.kind_mut());
            *dense_index -= curr - last;

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
    fn index(&self, key: K) -> V::Refs<'_>
    where
        K: Display,
    {
        let view = self.as_view();
        view.into_index(key)
    }

    #[inline]
    fn index_mut(&mut self, key: K) -> V::RefsMut<'_>
    where
        K: Display,
    {
        let view_mut = self.as_mut_view();
        view_mut.into_index_mut(key)
    }

    #[inline]
    pub fn get_with_key(&self, sparse_index: usize) -> Option<(K, V::Refs<'_>)> {
        let view = self.as_view();
        view.into_get_with_key(sparse_index)
    }

    #[inline]
    pub fn get_mut_with_key(&mut self, sparse_index: usize) -> Option<(K, V::RefsMut<'_>)> {
        let view_mut = self.as_mut_view();
        view_mut.into_get_mut_with_key(sparse_index)
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: usize) -> Option<K::Epoch> {
        let view = self.as_view();
        view.get_epoch(sparse_index)
    }

    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let view = self.as_view();
        view.contains_key(key)
    }

    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        let Self { dense, sparse } = self;

        let sparse_index = key.sparse_index();
        let Some(dense_index) = sparse
            .get(sparse_index)
            .take_if(|item| item.epoch == key.epoch())
            .and_then(SparseItem::dense_index)
        else {
            let entry = VacantEntry::new(key, self);
            return Entry::Vacant(entry);
        };

        check_dense_index_bounds(dense_index, dense.len());
        let entry = OccupiedEntry::new(key, dense_index, self);
        Entry::Occupied(entry)
    }

    #[inline]
    pub fn clear(&mut self) {
        let Self { dense, sparse } = self;

        for KeyValueRefs { key, .. } in dense.iter() {
            let sparse_index = key.sparse_index();
            sparse[sparse_index] = SparseItem::vacant(0, key.epoch().next());
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
        let view = self.as_view();
        view.keys()
    }

    #[inline]
    pub fn into_keys(self) -> IntoKeys<K, V> {
        let Self { dense, .. } = self;
        IntoKeys::new(dense.into_iter())
    }

    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        let view = self.as_view();
        view.values()
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        let view_mut = self.as_mut_view();
        view_mut.into_values_mut()
    }

    #[inline]
    pub fn into_values(self) -> IntoValues<K, V> {
        let Self { dense, .. } = self;
        IntoValues::new(dense.into_iter())
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        let view = self.as_view();
        view.iter()
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
    K::Epoch: Debug,
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
    K::Epoch: Hash,
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
    K: Key + Display,
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
    K: Key + Display,
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

// TODO impl AsMut for this type?
impl<K, V> AsRef<SoaSlice<KeyValuePair<K, V>>> for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
{
    #[inline]
    fn as_ref(&self) -> &SoaSlice<KeyValuePair<K, V>> {
        let Self { dense, .. } = self;
        dense
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
    K: Key,
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
            me.insert(key, value);
        }

        me
    }
}

impl<K, V> FromIterator<(K, V)> for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
    V::Context: Default,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        iter.into_iter().map(KeyValuePair::from).collect()
    }
}

impl<K, V> FromIterator<V> for EpochSparseSet<K, V>
where
    K: Key,
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
    K: Key,
    V: Soa,
{
    fn extend<I: IntoIterator<Item = KeyValuePair<K, V>>>(&mut self, iter: I) {
        let mut iter = iter.into_iter();
        while let Some(KeyValuePair { key, value }) = iter.next() {
            if self.len() == self.capacity() {
                let (lower, _) = iter.size_hint();
                self.reserve(lower.saturating_add(1), 0);
            }
            self.insert(key, value);
        }
    }
}

impl<K, V> Extend<(K, V)> for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
{
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        self.extend(iter.into_iter().map(KeyValuePair::from))
    }
}

impl<K, V> Extend<V> for EpochSparseSet<K, V>
where
    K: Key,
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
            self.insert(key, value);
        }
    }
}

impl<K, V> From<arena::EpochSparseArena<K, V>> for EpochSparseSet<K, V>
where
    K: Key,
    V: Soa,
{
    #[inline]
    fn from(value: arena::EpochSparseArena<K, V>) -> Self {
        let (dense, sparse) = value.into_parts();
        Self { dense, sparse }
    }
}

fn extend_sparse<E>(sparse: &mut Vec<SparseItem<E>>, new_len: usize)
where
    E: Epoch,
{
    let old_len = sparse.len();
    if old_len >= new_len {
        return;
    }

    let epoch = Default::default();
    let item = SparseItem::vacant(0, epoch);
    sparse.resize(new_len, item);
}

generate_entry_types!(EpochSparseSet<K, V>);

#[cfg(test)]
mod tests {
    use core::{mem::forget, ops::Not};

    use crate::prelude::*;

    type Key = EpochKey;

    #[test]
    fn empty() {
        let sparse_set = SparseSet::<(i32,)>::new();
        assert!(sparse_set.is_empty());
    }

    #[test]
    fn with_capacity() {
        let sparse_set = SparseSet::<(i32,)>::with_capacity(10, 10);
        assert!(sparse_set.is_empty());
        assert!(sparse_set.capacity() >= 10);
        assert!(sparse_set.sparse_capacity() >= 10);
    }

    #[test]
    fn empty_parts() {
        let sparse_set = SparseSet::<(i32,)>::new();

        let (dense, sparse) = sparse_set.into_parts();
        assert_eq!(dense.len(), 0);
        assert_eq!(sparse.len(), 0);

        let sparse_set = SparseSet::from_parts(dense, sparse);
        assert_eq!(sparse_set.len(), 0);
    }

    #[test]
    fn empty_keys() {
        let sparse_set = SparseSet::<(i32,)>::new();

        let keys = sparse_set.keys();
        assert_eq!(keys.len(), 0);
        assert_eq!(keys.as_slice(), &[]);
    }

    #[test]
    fn empty_into_keys() {
        let sparse_set = SparseSet::<(i32,)>::new();

        let keys = sparse_set.into_keys();
        assert_eq!(keys.len(), 0);
        assert_eq!(keys.as_slice(), &[]);
    }

    #[test]
    fn empty_values() {
        let sparse_set = SparseSet::<(i32,)>::new();

        let values = sparse_set.values();
        assert_eq!(values.len(), 0);
        assert_eq!(values.as_slice(), ([].as_slice(),));
    }

    #[test]
    fn empty_values_mut() {
        let mut sparse_set = SparseSet::<(i32,)>::new();
        let values_mut = sparse_set.values_mut();

        assert_eq!(values_mut.len(), 0);
        assert_eq!(values_mut.into_slice(), ([].as_mut_slice(),));
    }

    #[test]
    fn empty_into_values() {
        let sparse_set = SparseSet::<(i32,)>::new();

        let values = sparse_set.into_values();
        assert_eq!(values.len(), 0);
        assert_eq!(values.as_slice(), ([].as_slice(),));
    }

    #[test]
    fn empty_iter() {
        let sparse_set = SparseSet::<(i32,)>::new();

        let iter = sparse_set.iter();
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.as_keys_slice(), &[]);
        assert_eq!(iter.as_values_slice(), ([].as_slice(),));
    }

    #[test]
    fn empty_iter_mut() {
        let mut sparse_set = SparseSet::<(i32,)>::new();
        let iter_mut = sparse_set.iter_mut();

        assert_eq!(iter_mut.len(), 0);
        assert_eq!(iter_mut.as_keys_slice(), &[]);
        assert_eq!(iter_mut.into_values_slice(), ([].as_mut_slice(),));
    }

    #[test]
    fn empty_into_iter() {
        let sparse_set = SparseSet::<(i32,)>::new();
        let into_iter = sparse_set.into_iter();

        assert_eq!(into_iter.len(), 0);
        assert_eq!(into_iter.as_keys_slice(), &[]);
        assert_eq!(into_iter.as_values_slice(), ([].as_slice(),));
    }

    #[test]
    fn empty_insert_one() {
        let mut sparse_set = SparseSet::new();
        let previous = sparse_set.insert(0, (42,));
        assert_eq!(previous, None);

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert!(sparse_set.contains_key(0));
    }

    #[test]
    fn with_capacity_insert_one() {
        let mut sparse_set = SparseSet::with_capacity(10, 10);
        let previous = sparse_set.insert(0, (42,));
        assert_eq!(previous, None);

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert!(sparse_set.contains_key(0));
    }

    #[test]
    fn empty_insert_one_mutate() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));
        *sparse_set.index_mut(0).0 = 43;

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some((&43,)));
        assert!(sparse_set.contains_key(0));
    }

    #[test]
    fn with_capacity_insert_one_mutate() {
        let mut sparse_set = SparseSet::with_capacity(10, 10);
        sparse_set.insert(0, (42,));
        *sparse_set.index_mut(0).0 = 43;

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some((&43,)));
        assert!(sparse_set.contains_key(0));
    }

    #[test]
    fn empty_insert_far() {
        let mut sparse_set = SparseSet::new();

        let (key, value) = (3, (42,));
        sparse_set.insert(key, value);

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), Some((&value.0,)));
        assert!(sparse_set.contains_key(key));

        let (key, value) = (6, (69,));
        sparse_set.insert(key, value);

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(key), Some((&value.0,)));
        assert!(sparse_set.contains_key(key));
    }

    #[test]
    fn empty_insert_far_remove() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(3, (42,));
        sparse_set.insert(1, (69,));

        let key = 3;
        let value = sparse_set.remove(key).unwrap();

        assert_eq!(value, (42,));
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());

        let key = 1;
        let value = sparse_set.remove(key).unwrap();

        assert_eq!(value, (69,));
        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());
    }

    #[test]
    fn empty_push() {
        let mut sparse_set = SparseSet::new();

        let key = sparse_set.push((42,));
        assert_eq!(key, 0);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(key), Some((&42,)));
        assert!(sparse_set.contains_key(key));
    }

    #[test]
    fn empty_pop() {
        let mut sparse_set = SparseSet::<(i32,)>::new();

        let popped = sparse_set.pop();
        assert_eq!(popped, None);
        assert_eq!(sparse_set.len(), 0);
    }

    #[test]
    fn one_item_insert_remove_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));

        let removed = sparse_set.remove(0);
        assert_eq!(removed, Some((42,)));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(0), None);
        assert!(sparse_set.contains_key(0).not());
    }

    #[test]
    fn one_item_insert_remove_one_epoch() {
        let mut sparse_set = EpochSparseSet::new();

        let key = Key::new(0, 1);
        sparse_set.insert(key, (42,));

        let removed = sparse_set.remove(key);
        assert_eq!(removed, Some((42,)));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());

        assert_eq!(
            sparse_set.get_epoch(key.sparse_index()),
            Some(key.epoch().next()),
        );
        let key = Key::new(0, key.epoch().next());
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());
    }

    #[test]
    fn one_item_insert_swap_remove_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));

        let removed = sparse_set.swap_remove(0);
        assert_eq!(removed, Some((42,)));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(0), None);
        assert!(sparse_set.contains_key(0).not());
    }

    #[test]
    fn one_item_insert_swap_remove_one_epoch() {
        let mut sparse_set = EpochSparseSet::new();

        let key = Key::new(0, 1);
        sparse_set.insert(key, (42,));

        let removed = sparse_set.swap_remove(key);
        assert_eq!(removed, Some((42,)));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());

        assert_eq!(
            sparse_set.get_epoch(key.sparse_index()),
            Some(key.epoch().next()),
        );
        let key = Key::new(0, key.epoch().next());
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());
    }

    #[test]
    fn one_item_push_remove_one() {
        let mut sparse_set = SparseSet::new();
        let key = sparse_set.push((42,));

        let removed = sparse_set.remove(key);
        assert_eq!(removed, Some((42,)));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());
    }

    #[test]
    fn one_item_push_remove_one_epoch() {
        let mut sparse_set = EpochSparseSet::<Key, _>::new();
        let key = sparse_set.push((42,));

        let removed = sparse_set.remove(key);
        assert_eq!(removed, Some((42,)));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());

        assert_eq!(
            sparse_set.get_epoch(key.sparse_index()),
            Some(key.epoch().next()),
        );
        let key = Key::new(0, key.epoch().next());
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());
    }

    #[test]
    fn one_item_push_swap_remove_one() {
        let mut sparse_set = SparseSet::new();
        let key = sparse_set.push((42,));

        let removed = sparse_set.swap_remove(key);
        assert_eq!(removed, Some((42,)));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());
    }

    #[test]
    fn one_item_push_swap_remove_one_epoch() {
        let mut sparse_set = EpochSparseSet::<Key, _>::new();
        let key = sparse_set.push((42,));

        let removed = sparse_set.swap_remove(key);
        assert_eq!(removed, Some((42,)));

        assert_eq!(sparse_set.len(), 0);
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());

        assert_eq!(
            sparse_set.get_epoch(key.sparse_index()),
            Some(key.epoch().next()),
        );
        let key = Key::new(0, key.epoch().next());
        assert_eq!(sparse_set.get(key), None);
        assert!(sparse_set.contains_key(key).not());
    }

    #[test]
    fn one_item_swap() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));

        sparse_set.swap(0, 0);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.as_slices(), ([42].as_slice(),));
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert!(sparse_set.contains_key(0));

        sparse_set.swap(0, 1);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.as_slices(), ([42].as_slice(),));
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert!(sparse_set.contains_key(0));
    }

    #[test]
    fn one_item_swap_keys() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));

        sparse_set.swap_keys(0, 0);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.as_slices(), ([42].as_slice(),));
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert!(sparse_set.contains_key(0));

        sparse_set.swap_keys(0, 1);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.as_slices(), ([42].as_slice(),));
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert!(sparse_set.contains_key(0));
    }

    #[test]
    fn one_item_parts() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, (42,));

        let (dense, sparse) = sparse_set.into_parts();
        let (keys, values) = dense.as_slices().into();
        assert_eq!(keys, &[2]);
        assert_eq!(values, ([42].as_slice(),));
        assert_eq!(
            sparse,
            &[
                SparseItem::vacant(0, ()),
                SparseItem::vacant(0, ()),
                SparseItem::occupied(0, ()),
            ]
        );

        let sparse_set = SparseSet::from_parts(dense, sparse);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.as_slices(), ([42].as_slice(),));
        assert_eq!(sparse_set.as_keys_slice(), &[2]);
        assert_eq!(sparse_set.get(2), Some((&42,)));
    }

    #[test]
    fn one_item_keys() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));

        let keys = sparse_set.keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys.as_slice(), &[0]);
    }

    #[test]
    fn one_item_into_keys() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));

        let keys = sparse_set.into_keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys.as_slice(), &[0]);
    }

    #[test]
    fn one_item_values() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));

        let values = sparse_set.values();
        assert_eq!(values.len(), 1);
        assert_eq!(values.as_slice(), ([42].as_slice(),));
    }

    #[test]
    fn one_item_values_mut() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));

        let values_mut = sparse_set.values_mut();
        assert_eq!(values_mut.len(), 1);
        assert_eq!(values_mut.into_slice(), ([42].as_mut_slice(),));
    }

    #[test]
    fn one_item_into_values() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));

        let values = sparse_set.into_values();
        assert_eq!(values.len(), 1);
        assert_eq!(values.as_slice(), ([42].as_slice(),));
    }

    #[test]
    fn one_item_iter() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));

        let iter = sparse_set.iter();
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.as_keys_slice(), &[0]);
        assert_eq!(iter.as_values_slice(), ([42].as_slice(),));
    }

    #[test]
    fn one_item_iter_mut() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));

        let iter_mut = sparse_set.iter_mut();
        assert_eq!(iter_mut.len(), 1);
        assert_eq!(iter_mut.as_keys_slice(), &[0]);
        assert_eq!(iter_mut.into_values_slice(), ([42].as_mut_slice(),));
    }

    #[test]
    fn one_item_into_iter() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));

        let into_iter = sparse_set.into_iter();
        assert_eq!(into_iter.len(), 1);
        assert_eq!(into_iter.as_keys_slice(), &[0]);
        assert_eq!(into_iter.as_values_slice(), ([42].as_slice(),));
    }

    #[test]
    fn two_items_insert_first() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));
        sparse_set.insert(1, (69,));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert_eq!(sparse_set.get(1), Some((&69,)));

        let previous = sparse_set.insert(0, (34,));
        assert_eq!(previous, Some((42,)));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some((&34,)));
        assert_eq!(sparse_set.get(1), Some((&69,)));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_insert_first_epoch() {
        let mut sparse_set = EpochSparseSet::new();

        let first_key = Key::new(0, 3);
        sparse_set.insert(first_key, (42,));

        let second_key = Key::new(1, 0);
        sparse_set.insert(second_key, (69,));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(first_key), Some((&42,)));
        assert_eq!(sparse_set.get(second_key), Some((&69,)));

        let first_key = Key::new(first_key.sparse_index(), first_key.epoch().next());
        let previous = sparse_set.insert(first_key, (34,));
        assert_eq!(previous, Some((42,)));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(first_key), Some((&34,)));
        assert_eq!(sparse_set.get(second_key), Some((&69,)));
        assert!(sparse_set.contains_key(first_key));
        assert!(sparse_set.contains_key(second_key));
    }

    #[test]
    fn two_items_insert_second() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));
        sparse_set.insert(1, (69,));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert_eq!(sparse_set.get(1), Some((&69,)));

        let previous = sparse_set.insert(1, (34,));
        assert_eq!(previous, Some((69,)));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert_eq!(sparse_set.get(1), Some((&34,)));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_remove_first() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));
        sparse_set.insert(1, (69,));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert_eq!(sparse_set.get(1), Some((&69,)));

        let removed = sparse_set.remove(0);
        assert_eq!(removed, Some((42,)));

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), None);
        assert_eq!(sparse_set.get(1), Some((&69,)));
        assert!(sparse_set.contains_key(0).not());
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_swap_remove_first() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));
        sparse_set.insert(1, (69,));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert_eq!(sparse_set.get(1), Some((&69,)));

        let removed = sparse_set.swap_remove(0);
        assert_eq!(removed, Some((42,)));

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), None);
        assert_eq!(sparse_set.get(1), Some((&69,)));
        assert!(sparse_set.contains_key(0).not());
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_remove_second() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));
        sparse_set.insert(1, (69,));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert_eq!(sparse_set.get(1), Some((&69,)));

        let removed = sparse_set.remove(1);
        assert_eq!(removed, Some((69,)));

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert_eq!(sparse_set.get(1), None);
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1).not());
    }

    #[test]
    fn two_items_swap_remove_second() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));
        sparse_set.insert(1, (69,));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert_eq!(sparse_set.get(1), Some((&69,)));

        let removed = sparse_set.swap_remove(1);
        assert_eq!(removed, Some((69,)));

        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert_eq!(sparse_set.get(1), None);
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1).not());
    }

    #[test]
    fn two_items_remove_one_insert_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));
        sparse_set.insert(1, (69,));

        let removed = sparse_set.remove(0);
        assert_eq!(removed, Some((42,)));
        assert_eq!(sparse_set.get(0), None);

        sparse_set.insert(0, (34,));
        assert_eq!(sparse_set.get(0), Some((&34,)));
        assert_eq!(sparse_set.get(1), Some((&69,)));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_swap_remove_one_insert_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));
        sparse_set.insert(1, (69,));

        let removed = sparse_set.swap_remove(0);
        assert_eq!(removed, Some((42,)));
        assert_eq!(sparse_set.get(0), None);

        sparse_set.insert(0, (34,));
        assert_eq!(sparse_set.get(0), Some((&34,)));
        assert_eq!(sparse_set.get(1), Some((&69,)));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_remove_one_push_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));
        sparse_set.insert(1, (69,));

        let removed = sparse_set.remove(0);
        assert_eq!(removed, Some((42,)));
        assert_eq!(sparse_set.get(0), None);

        let key = sparse_set.push((34,));
        assert_eq!(key, 0);

        assert_eq!(sparse_set.get(0), Some((&34,)));
        assert_eq!(sparse_set.get(1), Some((&69,)));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_swap_remove_one_push_one() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));
        sparse_set.insert(1, (69,));

        let removed = sparse_set.swap_remove(0);
        assert_eq!(removed, Some((42,)));
        assert_eq!(sparse_set.get(0), None);

        let key = sparse_set.push((34,));
        assert_eq!(key, 0);

        assert_eq!(sparse_set.get(0), Some((&34,)));
        assert_eq!(sparse_set.get(1), Some((&69,)));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1));
    }

    #[test]
    fn two_items_swap() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));
        sparse_set.insert(1, (69,));

        sparse_set.swap(0, 0);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.as_slices(), ([42, 69].as_slice(),));
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert_eq!(sparse_set.get(1), Some((&69,)));

        sparse_set.swap(0, 1);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.as_slices(), ([69, 42].as_slice(),));
        assert_eq!(sparse_set.get(0), Some((&69,)));
        assert_eq!(sparse_set.get(1), Some((&42,)));

        sparse_set.swap(1, 1);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.as_slices(), ([69, 42].as_slice(),));
        assert_eq!(sparse_set.get(0), Some((&69,)));
        assert_eq!(sparse_set.get(1), Some((&42,)));
    }

    #[test]
    fn two_items_swap_keys() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (42,));
        sparse_set.insert(1, (69,));

        sparse_set.swap_keys(0, 0);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.as_slices(), ([42, 69].as_slice(),));
        assert_eq!(sparse_set.get(0), Some((&42,)));
        assert_eq!(sparse_set.get(1), Some((&69,)));

        sparse_set.swap_keys(0, 1);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.as_slices(), ([42, 69].as_slice(),));
        assert_eq!(sparse_set.get(0), Some((&69,)));
        assert_eq!(sparse_set.get(1), Some((&42,)));

        sparse_set.swap_keys(1, 1);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.as_slices(), ([42, 69].as_slice(),));
        assert_eq!(sparse_set.get(0), Some((&69,)));
        assert_eq!(sparse_set.get(1), Some((&42,)));
    }

    #[test]
    fn two_items_insert_pop() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(5, (42,));
        sparse_set.insert(2, (69,));

        let popped = sparse_set.pop();
        assert_eq!(popped, Some((2, (69,)).into()));
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(5), Some((&42,)));
        assert_eq!(sparse_set.get(2), None);
    }

    #[test]
    fn two_items_push_pop() {
        let mut sparse_set = SparseSet::new();
        let first_key = sparse_set.push((42,));
        let second_key = sparse_set.push((69,));

        let popped = sparse_set.pop();
        assert_eq!(popped, Some((second_key, (69,)).into()));
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(first_key), Some((&42,)));
        assert_eq!(sparse_set.get(second_key), None);
    }

    #[test]
    fn two_items_insert_pop_epoch() {
        let mut sparse_set = EpochSparseSet::new();

        let first_key = Key::new(5, 1);
        sparse_set.insert(first_key, (42,));

        let second_key = Key::new(2, 0);
        sparse_set.insert(second_key, (69,));

        let popped = sparse_set.pop();
        assert_eq!(popped, Some((second_key, (69,)).into()));
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(first_key), Some((&42,)));
        assert_eq!(sparse_set.get(second_key), None);

        assert_eq!(
            sparse_set.get_epoch(second_key.sparse_index()),
            Some(second_key.epoch().next()),
        );
    }

    #[test]
    fn two_items_push_pop_epoch() {
        let mut sparse_set = EpochSparseSet::<Key, _>::new();
        let first_key = sparse_set.push((42,));
        let second_key = sparse_set.push((69,));

        let popped = sparse_set.pop();
        assert_eq!(popped, Some((second_key, (69,)).into()));
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.get(first_key), Some((&42,)));
        assert_eq!(sparse_set.get(second_key), None);

        assert_eq!(
            sparse_set.get_epoch(second_key.sparse_index()),
            Some(second_key.epoch().next()),
        );
    }

    #[test]
    fn two_items_invalidate_epoch() {
        let mut sparse_set = EpochSparseSet::new();

        let first_key = Key::new(5, 1);
        sparse_set.insert(first_key, (42,));

        let second_key = Key::new(2, 0);
        sparse_set.insert(second_key, (69,));

        let new_first_key = sparse_set
            .invalidate_epoch(first_key)
            .expect("first key should be present");
        assert_eq!(new_first_key.sparse_index(), first_key.sparse_index());
        assert_eq!(new_first_key.epoch(), &first_key.epoch().next());
        assert_eq!(new_first_key, Key::new(5, 2));
        assert_eq!(sparse_set.get(first_key), None);
        assert_eq!(sparse_set.get(new_first_key), Some((&42,)));

        let new_second_key = sparse_set
            .invalidate_epoch(second_key)
            .expect("second key should be present");
        assert_eq!(new_second_key.sparse_index(), second_key.sparse_index());
        assert_eq!(new_second_key.epoch(), &second_key.epoch().next());
        assert_eq!(new_second_key, Key::new(2, 1));
        assert_eq!(sparse_set.get(second_key), None);
        assert_eq!(sparse_set.get(new_second_key), Some((&69,)));
    }

    #[test]
    fn three_items_insert_remove_middle() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(5, (69,));

        let removed = sparse_set.remove(2);
        assert_eq!(removed, Some((34,)));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(2), None);
        assert_eq!(sparse_set.get(1), Some((&42,)));
        assert_eq!(sparse_set.get(5), Some((&69,)));
        assert!(sparse_set.contains_key(2).not());
        assert!(sparse_set.contains_key(1));
        assert!(sparse_set.contains_key(5));
    }

    #[test]
    fn three_items_push_remove_middle() {
        let mut sparse_set = SparseSet::new();
        let first_key = sparse_set.push((34,));
        let middle_key = sparse_set.push((42,));
        let last_key = sparse_set.push((69,));

        let removed = sparse_set.remove(middle_key);
        assert_eq!(removed, Some((42,)));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(first_key), Some((&34,)));
        assert_eq!(sparse_set.get(middle_key), None);
        assert_eq!(sparse_set.get(last_key), Some((&69,)));
        assert!(sparse_set.contains_key(first_key));
        assert!(sparse_set.contains_key(middle_key).not());
        assert!(sparse_set.contains_key(last_key));
    }

    #[test]
    fn three_items_swap_remove_middle() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(0, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(2, (69,));

        let removed = sparse_set.swap_remove(1);
        assert_eq!(removed, Some((42,)));

        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.get(0), Some((&34,)));
        assert_eq!(sparse_set.get(1), None);
        assert_eq!(sparse_set.get(2), Some((&69,)));
        assert!(sparse_set.contains_key(0));
        assert!(sparse_set.contains_key(1).not());
        assert!(sparse_set.contains_key(2));
    }

    #[test]
    fn three_items_parts() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(5, (69,));

        let (mut dense, sparse) = sparse_set.into_parts();
        let (keys, values) = dense.as_slices().into();
        assert_eq!(keys, &[2, 1, 5]);
        assert_eq!(values, ([34, 42, 69].as_slice(),));
        assert_eq!(
            sparse,
            &[
                SparseItem::vacant(0, ()),
                SparseItem::occupied(1, ()),
                SparseItem::occupied(0, ()),
                SparseItem::vacant(0, ()),
                SparseItem::vacant(0, ()),
                SparseItem::occupied(2, ()),
            ]
        );

        dense.swap_remove(0);
        let sparse_set = SparseSet::from_parts(dense, sparse);
        assert_eq!(sparse_set.len(), 2);
        assert_eq!(sparse_set.as_slices(), ([69, 42].as_slice(),));
        assert_eq!(sparse_set.as_keys_slice(), &[5, 1]);
        assert_eq!(sparse_set.get(5), Some((&69,)));
    }

    #[test]
    fn three_items_keys() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(5, (69,));

        let keys = sparse_set.keys();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys.as_slice(), &[2, 1, 5]);
    }

    #[test]
    fn three_items_into_keys() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(5, (69,));

        let keys = sparse_set.into_keys();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys.as_slice(), &[2, 1, 5]);
    }

    #[test]
    fn three_items_values() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(5, (69,));

        let values = sparse_set.values();
        assert_eq!(values.len(), 3);
        assert_eq!(values.as_slice(), ([34, 42, 69].as_slice(),));
    }

    #[test]
    fn three_items_values_mut() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(5, (69,));

        let values_mut = sparse_set.values_mut();
        assert_eq!(values_mut.len(), 3);
        assert_eq!(values_mut.into_slice(), ([34, 42, 69].as_mut_slice(),));
    }

    #[test]
    fn three_items_into_values() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(5, (69,));

        let values = sparse_set.into_values();
        assert_eq!(values.len(), 3);
        assert_eq!(values.as_slice(), ([34, 42, 69].as_slice(),));
    }

    #[test]
    fn three_items_iter() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(5, (69,));

        let iter = sparse_set.iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.as_keys_slice(), &[2, 1, 5]);
        assert_eq!(iter.as_values_slice(), ([34, 42, 69].as_slice(),));
    }

    #[test]
    fn three_items_iter_mut() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(5, (69,));

        let iter_mut = sparse_set.iter_mut();
        assert_eq!(iter_mut.len(), 3);
        assert_eq!(iter_mut.as_keys_slice(), &[2, 1, 5]);
        assert_eq!(iter_mut.into_values_slice(), ([34, 42, 69].as_mut_slice(),));
    }

    #[test]
    fn three_items_into_iter() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(5, (69,));

        let into_iter = sparse_set.into_iter();
        assert_eq!(into_iter.len(), 3);
        assert_eq!(into_iter.as_keys_slice(), &[2, 1, 5]);
        assert_eq!(into_iter.as_values_slice(), ([34, 42, 69].as_slice(),));
    }

    #[test]
    fn five_items_remove_insert() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(4, (34,));
        sparse_set.insert(2, (42,));
        sparse_set.insert(1, (69,));
        sparse_set.insert(6, (228,));
        sparse_set.insert(0, (666,));

        let key = 1;
        let value = sparse_set.remove(key).unwrap();
        assert_eq!(value, (69,));

        let key = 6;
        let value = sparse_set.remove(key).unwrap();
        assert_eq!(value, (228,));

        let key = 4;
        let value = sparse_set.remove(key).unwrap();
        assert_eq!(value, (34,));

        let key = 0;
        let value = sparse_set.remove(key).unwrap();
        assert_eq!(value, (666,));

        let key = 3;
        let value = (0,);
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some((&value.0,)));
        assert!(sparse_set.contains_key(key));

        let key = 2;
        let value = (1,);
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, Some((42,)));
        assert_eq!(sparse_set.get(key), Some((&value.0,)));
        assert!(sparse_set.contains_key(key));

        let key = 4;
        let value = (10,);
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some((&value.0,)));
        assert!(sparse_set.contains_key(key));
    }

    #[test]
    fn five_items_swap_remove_insert() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(4, (34,));
        sparse_set.insert(2, (42,));
        sparse_set.insert(1, (69,));
        sparse_set.insert(6, (228,));
        sparse_set.insert(0, (666,));

        let key = 1;
        let value = sparse_set.swap_remove(key).unwrap();
        assert_eq!(value, (69,));

        let key = 6;
        let value = sparse_set.swap_remove(key).unwrap();
        assert_eq!(value, (228,));

        let key = 4;
        let value = sparse_set.swap_remove(key).unwrap();
        assert_eq!(value, (34,));

        let key = 0;
        let value = sparse_set.swap_remove(key).unwrap();
        assert_eq!(value, (666,));

        let key = 3;
        let value = (0,);
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some((&value.0,)));
        assert!(sparse_set.contains_key(key));

        let key = 2;
        let value = (1,);
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, Some((42,)));
        assert_eq!(sparse_set.get(key), Some((&value.0,)));
        assert!(sparse_set.contains_key(key));

        let key = 4;
        let value = (10,);
        let previous = sparse_set.insert(key, value);

        assert_eq!(previous, None);
        assert_eq!(sparse_set.get(key), Some((&value.0,)));
        assert!(sparse_set.contains_key(key));
    }

    #[test]
    fn five_items_remove_push() {
        let mut sparse_set = SparseSet::new();
        let _key0 = sparse_set.push((34,));
        let key1 = sparse_set.push((42,));
        let key2 = sparse_set.push((69,));
        let key3 = sparse_set.push((228,));
        let key4 = sparse_set.push((666,));

        let value = sparse_set.remove(key1).unwrap();
        assert_eq!(value, (42,));

        let value = sparse_set.remove(key3).unwrap();
        assert_eq!(value, (228,));

        let value = sparse_set.remove(key4).unwrap();
        assert_eq!(value, (666,));

        let value = sparse_set.remove(key2).unwrap();
        assert_eq!(value, (69,));

        let value = (0,);
        let key = sparse_set.push(value);
        assert_eq!(sparse_set.get(key), Some((&value.0,)));
        assert!(sparse_set.contains_key(key));

        let value = (1,);
        let key = sparse_set.push(value);
        assert_eq!(sparse_set.get(key), Some((&value.0,)));
        assert!(sparse_set.contains_key(key));

        let value = (10,);
        let key = sparse_set.push(value);
        assert_eq!(sparse_set.get(key), Some((&value.0,)));
        assert!(sparse_set.contains_key(key));
    }

    #[test]
    fn five_items_swap_remove_push() {
        let mut sparse_set = SparseSet::new();
        let _key0 = sparse_set.push((34,));
        let key1 = sparse_set.push((42,));
        let key2 = sparse_set.push((69,));
        let key3 = sparse_set.push((228,));
        let key4 = sparse_set.push((666,));

        let value = sparse_set.swap_remove(key1).unwrap();
        assert_eq!(value, (42,));

        let value = sparse_set.swap_remove(key3).unwrap();
        assert_eq!(value, (228,));

        let value = sparse_set.swap_remove(key4).unwrap();
        assert_eq!(value, (666,));

        let value = sparse_set.swap_remove(key2).unwrap();
        assert_eq!(value, (69,));

        let value = (0,);
        let key = sparse_set.push(value);
        assert_eq!(sparse_set.get(key), Some((&value.0,)));
        assert!(sparse_set.contains_key(key));

        let value = (1,);
        let key = sparse_set.push(value);
        assert_eq!(sparse_set.get(key), Some((&value.0,)));
        assert!(sparse_set.contains_key(key));

        let value = (10,);
        let key = sparse_set.push(value);
        assert_eq!(sparse_set.get(key), Some((&value.0,)));
        assert!(sparse_set.contains_key(key));
    }

    #[test]
    fn five_items_retain() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(8, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(4, (69,));
        sparse_set.insert(3, (228,));
        sparse_set.insert(6, (666,));

        sparse_set.retain(|key, _| key % 2 == 0);
        assert_eq!(sparse_set.len(), 3);
        assert_eq!(sparse_set.keys().as_slice(), &[8, 4, 6]);
        assert_eq!(sparse_set.values().as_slice(), ([34, 69, 666].as_slice(),));

        assert_eq!(sparse_set.get(8), Some((&34,)));
        assert_eq!(sparse_set.get(1), None);
        assert_eq!(sparse_set.get(4), Some((&69,)));
        assert_eq!(sparse_set.get(3), None);
        assert_eq!(sparse_set.get(6), Some((&666,)));

        sparse_set.retain(|_, (value,)| *value % 2 == 1);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.keys().as_slice(), &[4]);
        assert_eq!(sparse_set.values().as_slice(), ([69].as_slice(),));

        assert_eq!(sparse_set.get(8), None);
        assert_eq!(sparse_set.get(1), None);
        assert_eq!(sparse_set.get(4), Some((&69,)));
        assert_eq!(sparse_set.get(3), None);
        assert_eq!(sparse_set.get(6), None);
    }

    #[test]
    fn five_items_drain() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(8, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(4, (69,));
        sparse_set.insert(3, (228,));
        sparse_set.insert(6, (666,));

        let drain = sparse_set.drain();
        assert_eq!(drain.as_keys_slice(), &[8, 1, 4, 3, 6]);
        assert_eq!(
            drain.as_values_slice(),
            ([34, 42, 69, 228, 666].as_slice(),),
        );

        forget(drain);
        assert_eq!(sparse_set.len(), 0);
        assert_ne!(sparse_set.sparse_len(), 0);
        assert_eq!(sparse_set.keys().as_slice(), &[]);
        assert_eq!(sparse_set.values().as_slice(), ([].as_slice(),));
    }

    #[test]
    fn five_items_insert_truncate() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(8, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(4, (69,));
        sparse_set.insert(3, (228,));
        sparse_set.insert(6, (666,));

        sparse_set.truncate(usize::MAX, 5);
        assert_eq!(sparse_set.sparse_len(), 5);
        assert_eq!(sparse_set.keys().as_slice(), &[1, 4, 3]);
        assert_eq!(sparse_set.values().as_slice(), ([42, 69, 228].as_slice(),));

        assert_eq!(sparse_set.get(1), Some((&42,)));
        assert_eq!(sparse_set.get(4), Some((&69,)));
        assert_eq!(sparse_set.get(3), Some((&228,)));

        sparse_set.truncate(1, usize::MAX);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.keys().as_slice(), &[1]);
        assert_eq!(sparse_set.values().as_slice(), ([42].as_slice(),));

        assert_eq!(sparse_set.get(1), Some((&42,)));
    }

    #[test]
    fn five_items_push_truncate() {
        let mut sparse_set = SparseSet::new();
        let key0 = sparse_set.push((34,));
        let key1 = sparse_set.push((42,));
        let key2 = sparse_set.push((69,));
        let key3 = sparse_set.push((228,));
        let key4 = sparse_set.push((666,));

        sparse_set.truncate(usize::MAX, 3);
        assert_eq!(sparse_set.sparse_len(), 3);
        assert_eq!(sparse_set.as_keys_slice(), &[key0, key1, key2]);
        assert_eq!(sparse_set.as_slices(), ([34, 42, 69].as_slice(),));

        assert_eq!(sparse_set.get(key0), Some((&34,)));
        assert_eq!(sparse_set.get(key1), Some((&42,)));
        assert_eq!(sparse_set.get(key2), Some((&69,)));
        assert_eq!(sparse_set.get(key3), None);
        assert_eq!(sparse_set.get(key4), None);

        sparse_set.truncate(1, usize::MAX);
        assert_eq!(sparse_set.len(), 1);
        assert_eq!(sparse_set.as_keys_slice(), &[key0]);
        assert_eq!(sparse_set.as_slices(), ([34].as_slice(),));

        assert_eq!(sparse_set.get(key0), Some((&34,)));
    }

    #[test]
    fn five_items_sort() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(8, (42,));
        sparse_set.insert(1, (228,));
        sparse_set.insert(4, (69,));
        sparse_set.insert(3, (666,));
        sparse_set.insert(6, (34,));

        sparse_set.sort();
        assert_eq!(sparse_set.keys().as_slice(), &[6, 8, 4, 1, 3]);
        assert_eq!(
            sparse_set.values().as_slice(),
            ([34, 42, 69, 228, 666].as_slice(),),
        );

        assert_eq!(sparse_set.get(8), Some((&42,)));
        assert_eq!(sparse_set.get(1), Some((&228,)));
        assert_eq!(sparse_set.get(4), Some((&69,)));
        assert_eq!(sparse_set.get(3), Some((&666,)));
        assert_eq!(sparse_set.get(6), Some((&34,)));
    }

    #[test]
    fn five_items_sort_keys() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(8, (42,));
        sparse_set.insert(1, (228,));
        sparse_set.insert(4, (69,));
        sparse_set.insert(3, (666,));
        sparse_set.insert(6, (34,));

        sparse_set.sort_keys();
        assert_eq!(sparse_set.keys().as_slice(), &[1, 3, 4, 6, 8]);
        assert_eq!(
            sparse_set.values().as_slice(),
            ([228, 666, 69, 34, 42].as_slice(),),
        );

        assert_eq!(sparse_set.get(8), Some((&42,)));
        assert_eq!(sparse_set.get(1), Some((&228,)));
        assert_eq!(sparse_set.get(4), Some((&69,)));
        assert_eq!(sparse_set.get(3), Some((&666,)));
        assert_eq!(sparse_set.get(6), Some((&34,)));
    }

    #[test]
    fn five_items_sort_by() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(8, (42,));
        sparse_set.insert(1, (228,));
        sparse_set.insert(4, (69,));
        sparse_set.insert(3, (666,));
        sparse_set.insert(6, (34,));

        sparse_set.sort_by(|(_, (a,)), (_, (b,))| Ord::cmp(b, a));
        assert_eq!(sparse_set.keys().as_slice(), &[3, 1, 4, 8, 6]);
        assert_eq!(
            sparse_set.values().as_slice(),
            ([666, 228, 69, 42, 34].as_slice(),),
        );

        assert_eq!(sparse_set.get(8), Some((&42,)));
        assert_eq!(sparse_set.get(1), Some((&228,)));
        assert_eq!(sparse_set.get(4), Some((&69,)));
        assert_eq!(sparse_set.get(3), Some((&666,)));
        assert_eq!(sparse_set.get(6), Some((&34,)));
    }

    #[test]
    fn five_items_entry() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(8, (42,));
        sparse_set.insert(1, (228,));
        sparse_set.insert(4, (69,));
        sparse_set.insert(3, (666,));
        sparse_set.insert(6, (34,));

        let entry = sparse_set.entry(0);
        assert_eq!(entry.key(), 0);
        assert_eq!(entry.get(), None);

        let entry = entry.and_modify(|(value,)| *value += 1);
        assert_eq!(entry.key(), 0);
        assert_eq!(entry.get(), None);

        let entry = entry.replace_key(1);
        assert_eq!(entry.key(), 1);
        assert_eq!(entry.get(), Some((&228,)));

        let value = entry.and_modify(|(value,)| *value += 1).or_insert((47,));
        assert_eq!(value, (&mut 229,));
    }

    #[test]
    fn from_keys_values_iter() {
        let keys = [3, 10, 5, 10, 1, usize::MAX];
        let values = [(34,), (42,), (69,), (228,), (666,)];

        let sparse_set: SparseSet<(_,)> = keys.into_iter().zip(values).collect();
        assert_eq!(sparse_set.len(), 4);
        assert_eq!(sparse_set.keys().as_slice(), &[3, 10, 5, 1]);
        assert_eq!(
            sparse_set.values().as_slice(),
            ([34, 228, 69, 666].as_slice(),)
        );

        assert_eq!(sparse_set.get(3), Some((&34,)));
        assert_eq!(sparse_set.get(10), Some((&228,)));
        assert_eq!(sparse_set.get(5), Some((&69,)));
        assert_eq!(sparse_set.get(1), Some((&666,)));
    }

    #[test]
    #[should_panic(expected = "capacity overflow")]
    fn from_keys_values_iter_too_large_key() {
        let keys = [3, 10, 5, 10, 1, usize::MAX];
        let values = [(34,), (42,), (69,), (228,), (666,), (999,)];

        let sparse_set: SparseSet<(_,)> = keys.into_iter().zip(values).collect();
        assert_eq!(sparse_set.len(), 4);
        assert_eq!(sparse_set.keys().as_slice(), &[3, 10, 5, 1, usize::MAX]);
        assert_eq!(
            sparse_set.values().as_slice(),
            ([34, 228, 69, 666, 999].as_slice(),),
        );

        assert_eq!(sparse_set.get(3), Some((&34,)));
        assert_eq!(sparse_set.get(10), Some((&228,)));
        assert_eq!(sparse_set.get(5), Some((&69,)));
        assert_eq!(sparse_set.get(1), Some((&666,)));
        assert_eq!(sparse_set.get(usize::MAX), Some((&999,)));
    }

    #[test]
    fn from_values_iter() {
        let values = [(34,), (42,), (69,), (228,), (666,)];
        let sparse_set: SparseSet<_> = values.into_iter().collect();

        assert_eq!(sparse_set.len(), 5);
        assert_eq!(sparse_set.keys().as_slice(), &[0, 1, 2, 3, 4]);
        assert_eq!(
            sparse_set.values().as_slice(),
            ([34, 42, 69, 228, 666].as_slice(),),
        );

        assert_eq!(sparse_set.get(0), Some((&34,)));
        assert_eq!(sparse_set.get(1), Some((&42,)));
        assert_eq!(sparse_set.get(2), Some((&69,)));
        assert_eq!(sparse_set.get(3), Some((&228,)));
        assert_eq!(sparse_set.get(4), Some((&666,)));
    }

    #[test]
    fn extend_keys_values() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(5, (69,));

        let keys = [3, 0, 2, 8];
        let values = [(228,), (666,), (42,), (69,)];
        sparse_set.extend(keys.into_iter().zip(values));

        assert_eq!(sparse_set.keys().as_slice(), &[2, 1, 5, 3, 0, 8]);
        assert_eq!(
            sparse_set.values().as_slice(),
            ([42, 42, 69, 228, 666, 69].as_slice(),),
        );

        assert_eq!(sparse_set.get(2), Some((&42,)));
        assert_eq!(sparse_set.get(1), Some((&42,)));
        assert_eq!(sparse_set.get(5), Some((&69,)));
        assert_eq!(sparse_set.get(3), Some((&228,)));
        assert_eq!(sparse_set.get(0), Some((&666,)));
        assert_eq!(sparse_set.get(8), Some((&69,)));
    }

    #[test]
    fn extend_values() {
        let mut sparse_set = SparseSet::new();
        sparse_set.insert(2, (34,));
        sparse_set.insert(1, (42,));
        sparse_set.insert(4, (69,));

        let values = [(228,), (666,), (201,)];
        sparse_set.extend(values);

        assert_eq!(sparse_set.keys().as_slice(), &[2, 1, 4, 0, 3, 5]);
        assert_eq!(
            sparse_set.values().as_slice(),
            ([34, 42, 69, 228, 666, 201].as_slice(),),
        );

        assert_eq!(sparse_set.get(2), Some((&34,)));
        assert_eq!(sparse_set.get(1), Some((&42,)));
        assert_eq!(sparse_set.get(4), Some((&69,)));
        assert_eq!(sparse_set.get(0), Some((&228,)));
        assert_eq!(sparse_set.get(3), Some((&666,)));
        assert_eq!(sparse_set.get(5), Some((&201,)));
    }

    #[test]
    fn from_arena() {
        let mut sparse_arena = SparseArena::new();
        sparse_arena.insert(2, (34,));
        sparse_arena.insert(1, (42,));
        sparse_arena.insert(5, (69,));

        let sparse_set = SparseSet::from(sparse_arena);
        assert_eq!(sparse_set.len(), 3);
        assert_eq!(sparse_set.keys().as_slice(), &[2, 1, 5]);
        assert_eq!(sparse_set.values().as_slice(), ([34, 42, 69].as_slice(),));

        assert_eq!(sparse_set.get(2), Some((&34,)));
        assert_eq!(sparse_set.get(1), Some((&42,)));
        assert_eq!(sparse_set.get(5), Some((&69,)));
    }
}
