use alloc::vec::Vec;
use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ops::{Index, IndexMut},
};
use gpecs_soa::traits::SoaVecs;

use crate::{
    assert::{
        check_dense_index_bounds, check_equal_key, check_key_bounds, unwrap_dense,
        unwrap_dense_index_mut, unwrap_into_index, unwrap_into_usize, unwrap_next_vacant,
        unwrap_next_vacant_mut, unwrap_sparse_item, unwrap_sparse_item_mut,
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
    set,
    soa::{
        mem::replace as soa_replace,
        slice::{SoaSlices, SoaSlicesMut},
        traits::Soa,
        vec::SoaVec,
    },
    view::{EpochSparseView, EpochSparseViewMut},
};

pub type SparseArena<T> = EpochSparseArena<usize, T>;

pub struct EpochSparseArena<K, V>
where
    K: Key,
    V: Soa,
{
    dense: SoaVec<KeyValuePair<K, V>>,
    sparse: Vec<SparseItem<K>>,
    sparse_vacant_head: usize,
}

impl<K, V> EpochSparseArena<K, V>
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
            sparse_vacant_head: 0,
        }
    }

    #[inline]
    pub fn with_context(context: V::Context) -> Self {
        Self {
            dense: SoaVec::with_context(context),
            sparse: Vec::new(),
            sparse_vacant_head: 0,
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
            sparse_vacant_head: 0,
        }
    }

    #[inline]
    pub fn with_context_and_capacity(context: V::Context, dense: usize, sparse: usize) -> Self {
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
    #[allow(unsafe_code)]
    pub unsafe fn as_keys_slice_mut(&mut self) -> &mut [K] {
        let Self { dense, .. } = self;

        let KeyValueSlicesMut { keys, .. } = dense.as_mut_slices();
        keys
    }

    #[inline]
    pub fn as_keys_ptr(&self) -> *const K {
        let Self { dense, .. } = self;

        let KeyValuePtrs { key, .. } = dense.as_ptrs();
        key
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn as_keys_ptr_mut(&mut self) -> *mut K {
        let Self { dense, .. } = self;

        let KeyValueMutPtrs { key, .. } = dense.as_mut_ptrs();
        key
    }

    #[inline]
    pub fn as_sparse_slice(&self) -> &[SparseItem<K>] {
        let Self { sparse, .. } = self;
        sparse.as_slice()
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn as_sparse_slice_mut(&mut self) -> &mut [SparseItem<K>] {
        let Self { sparse, .. } = self;
        sparse.as_mut_slice()
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
    #[allow(unsafe_code)]
    pub unsafe fn as_sparse_ptr_mut(&mut self) -> *mut SparseItem<K> {
        let Self { sparse, .. } = self;
        sparse.as_mut_ptr()
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn as_view(&self) -> EpochSparseView<'_, K, V> {
        let Self { dense, sparse, .. } = self;
        unsafe { EpochSparseView::new_unchecked(dense.slices(), sparse) }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn as_mut_view(&mut self) -> EpochSparseViewMut<'_, K, V> {
        let Self { dense, sparse, .. } = self;
        unsafe { EpochSparseViewMut::new_unchecked(dense.slices_mut(), sparse) }
    }

    #[inline]
    pub fn into_parts(self) -> (SoaVec<KeyValuePair<K, V>>, Vec<SparseItem<K>>) {
        let Self { dense, sparse, .. } = self;
        (dense, sparse)
    }

    #[inline]
    #[track_caller]
    pub fn from_parts(
        dense: SoaVec<KeyValuePair<K, V>>,
        mut sparse: Vec<SparseItem<K>>,
    ) -> Result<Self, InvalidKeyError<K>> {
        let _view = EpochSparseView::new(dense.slices(), sparse.as_slice())?;

        sparse.clear();
        let mut sparse_vacant_head = 0;
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

            if sparse_index >= sparse.len() {
                let new_len = sparse_index.saturating_add(1);
                extend_sparse(&mut sparse, new_len, &mut sparse_vacant_head)?;
            } else {
                let sparse_item = unwrap_sparse_item(sparse.as_slice(), sparse_index);
                let next_vacant = unwrap_next_vacant(sparse_item.kind())
                    .clone()
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

    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, InvalidKeyError<K>> {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        let sparse_index = key
            .sparse_index()
            .try_into()
            .map_err(TooLargeSparseIndexError::new)?;
        match sparse.get_mut(sparse_index) {
            Some(sparse_item) if key.epoch() >= sparse_item.epoch => match sparse_item.kind {
                SparseItemKind::Occupied { dense_index } => {
                    let (context, dense) = dense.slices_mut().into_slices_with_context();
                    let dense = SoaSlicesMut::<KeyValuePair<K, V>>::new(context, dense);

                    let dense_index = unwrap_into_usize(dense_index);
                    let (dense_key, dense_value) = unwrap_dense(dense, dense_index).into();

                    let value = soa_replace(context, dense_value, value);
                    sparse_item.epoch = key.epoch();
                    *dense_key = key;

                    Ok(Some(value))
                }
                SparseItemKind::Vacant { next_vacant } => {
                    let next_vacant = unwrap_into_usize(next_vacant);
                    remove_from_vacant_list(sparse, sparse_vacant_head, sparse_index, next_vacant);

                    let dense_index = dense
                        .len()
                        .try_into()
                        .map_err(TooSmallSparseIndexError::new)?;
                    dense.push(KeyValuePair { key, value });
                    sparse[sparse_index] = SparseItem::occupied(dense_index, key.epoch());

                    Ok(None)
                }
            },
            Some(_) => Ok(None),
            None => {
                let new_sparse_len = sparse_index.saturating_add(1);
                extend_sparse(sparse, new_sparse_len, sparse_vacant_head)?;

                let dense_index = dense
                    .len()
                    .try_into()
                    .map_err(TooSmallSparseIndexError::new)?;
                dense.push(KeyValuePair { key, value });
                sparse[sparse_index] = SparseItem::occupied(dense_index, key.epoch());

                Ok(None)
            }
        }
    }

    pub fn try_insert(&mut self, key: K, value: V) -> Result<Option<V>, TryInvalidKeyError<K>> {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        let sparse_index = key
            .sparse_index()
            .try_into()
            .map_err(TooLargeSparseIndexError::new)?;
        match sparse.get_mut(sparse_index) {
            Some(sparse_item) if key.epoch() >= sparse_item.epoch => match sparse_item.kind {
                SparseItemKind::Occupied { dense_index } => {
                    let (context, dense) = dense.slices_mut().into_slices_with_context();
                    let dense = SoaSlicesMut::<KeyValuePair<K, V>>::new(context, dense);

                    let dense_index = unwrap_into_usize(dense_index);
                    let (dense_key, dense_value) = unwrap_dense(dense, dense_index).into();

                    let value = soa_replace(context, dense_value, value);
                    sparse_item.epoch = key.epoch();
                    *dense_key = key;

                    Ok(Some(value))
                }
                SparseItemKind::Vacant { next_vacant } => {
                    let next_vacant = unwrap_into_usize(next_vacant);
                    remove_from_vacant_list(sparse, sparse_vacant_head, sparse_index, next_vacant);

                    let dense_index = dense
                        .len()
                        .try_into()
                        .map_err(TooSmallSparseIndexError::new)?;
                    dense.try_reserve(1).map_err(TryReserveError::Dense)?;
                    dense.push(KeyValuePair { key, value });
                    sparse[sparse_index] = SparseItem::occupied(dense_index, key.epoch());

                    Ok(None)
                }
            },
            Some(_) => Ok(None),
            None => {
                let new_sparse_len = sparse_index.saturating_add(1);
                sparse
                    .try_reserve(new_sparse_len.saturating_sub(sparse.len()))
                    .map_err(TryReserveError::Sparse)?;
                extend_sparse(sparse, new_sparse_len, sparse_vacant_head)?;

                let dense_index = dense
                    .len()
                    .try_into()
                    .map_err(TooSmallSparseIndexError::new)?;
                dense.try_reserve(1).map_err(TryReserveError::Dense)?;
                dense.push(KeyValuePair { key, value });
                sparse[sparse_index] = SparseItem::occupied(dense_index, key.epoch());

                Ok(None)
            }
        }
    }

    pub fn push(&mut self, value: V) -> Result<K, InvalidKeyError<K>> {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        if let Some(sparse_item) = sparse.get_mut(*sparse_vacant_head) {
            let next_vacant = unwrap_next_vacant(sparse_item.kind())
                .clone()
                .try_into()
                .map_err(TooLargeSparseIndexError::new)?;

            let sparse_index = sparse_vacant_head
                .clone()
                .try_into()
                .map_err(TooSmallSparseIndexError::new)?;
            let key = K::new(sparse_index, sparse_item.epoch);

            let dense_index = dense
                .len()
                .try_into()
                .map_err(TooSmallSparseIndexError::new)?;
            let sparse_item_kind = SparseItemKind::occupied(dense_index);

            dense.push(KeyValuePair { key, value });

            sparse_item.kind = sparse_item_kind;
            *sparse_vacant_head = next_vacant;

            return Ok(key);
        }

        let sparse_index = sparse_vacant_head
            .clone()
            .try_into()
            .map_err(TooSmallSparseIndexError::new)?;
        let key = Key::new(sparse_index, Default::default());

        let dense_index = dense
            .len()
            .try_into()
            .map_err(TooSmallSparseIndexError::new)?;
        let sparse_item = SparseItem::occupied(dense_index, Default::default());

        dense.push(KeyValuePair { key, value });
        sparse.push(sparse_item);
        *sparse_vacant_head = dense.len();

        Ok(key)
    }

    pub fn try_push(&mut self, value: V) -> Result<K, TryInvalidKeyError<K>> {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        if let Some(sparse_item) = sparse.get_mut(*sparse_vacant_head) {
            let next_vacant = unwrap_next_vacant(sparse_item.kind())
                .clone()
                .try_into()
                .map_err(TooLargeSparseIndexError::new)?;

            let sparse_index = sparse_vacant_head
                .clone()
                .try_into()
                .map_err(TooSmallSparseIndexError::new)?;
            let key = K::new(sparse_index, sparse_item.epoch);

            let dense_index = dense
                .len()
                .try_into()
                .map_err(TooSmallSparseIndexError::new)?;
            let sparse_item_kind = SparseItemKind::occupied(dense_index);

            dense.try_reserve(1).map_err(TryReserveError::Dense)?;
            dense.push(KeyValuePair { key, value });

            sparse_item.kind = sparse_item_kind;
            *sparse_vacant_head = next_vacant;

            return Ok(key);
        }

        let sparse_index = sparse_vacant_head
            .clone()
            .try_into()
            .map_err(TooSmallSparseIndexError::new)?;
        let key = Key::new(sparse_index, Default::default());

        let dense_index = dense
            .len()
            .try_into()
            .map_err(TooSmallSparseIndexError::new)?;
        let sparse_item = SparseItem::occupied(dense_index, Default::default());

        dense.try_reserve(1).map_err(TryReserveError::Dense)?;
        sparse.try_reserve(1).map_err(TryReserveError::Sparse)?;

        dense.push(KeyValuePair { key, value });
        sparse.push(sparse_item);
        *sparse_vacant_head = dense.len();

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
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

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
            match sparse_item.kind_mut() {
                SparseItemKind::Occupied { dense_index: index } => *index = dense_index,
                SparseItemKind::Vacant { next_vacant } => *next_vacant = dense_index,
            }
        }
        let next_vacant = unwrap_into_index(*sparse_vacant_head);
        sparse[sparse_index] = SparseItem::vacant(next_vacant, key.epoch().next());
        *sparse_vacant_head = sparse_index;

        Some(value)
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

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
        let next_vacant = unwrap_into_index(*sparse_vacant_head);
        sparse[sparse_index] = SparseItem::vacant(next_vacant, key.epoch().next());
        *sparse_vacant_head = sparse_index;

        Some(value)
    }

    pub fn pop(&mut self) -> Option<(K, V)> {
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        let KeyValuePair { key, value } = dense.pop()?;

        let sparse_index = unwrap_into_usize(key.sparse_index());
        check_key_bounds(sparse_index, sparse.len());

        let next_vacant = unwrap_into_index(*sparse_vacant_head);
        sparse[sparse_index] = SparseItem::vacant(next_vacant, key.epoch().next());
        *sparse_vacant_head = sparse_index;

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
        let Self {
            dense,
            sparse,
            sparse_vacant_head,
        } = self;

        for KeyValueRefs { key, .. } in dense.slices() {
            let sparse_index = unwrap_into_usize(key.sparse_index());

            let next_vacant = unwrap_into_index(*sparse_vacant_head);
            sparse[sparse_index] = SparseItem::vacant(next_vacant, key.epoch().next());
            *sparse_vacant_head = sparse_index;
        }

        Drain::new(dense.drain(..))
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(K, V::RefsMut<'_>) -> bool,
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
        let Self { dense, sparse, .. } = self;

        let sparse_index = key
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

        for KeyValueRefs { key, .. } in dense.slices() {
            let sparse_index = unwrap_into_usize(key.sparse_index());
            sparse[sparse_index] =
                SparseItem::vacant(unwrap_into_index(*sparse_vacant_head), key.epoch().next());
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

impl<K, V> EpochSparseArena<K, V>
where
    K: Key,
    V: SoaVecs,
{
    #[inline]
    pub fn into_vecs(self) -> (V::Context, V::Vecs) {
        let Self { dense, .. } = self;

        let (context, KeyValueVecs { values, .. }) = dense.into_vecs();
        (context, values)
    }

    #[inline]
    pub fn into_keys_vec(self) -> Vec<K> {
        let Self { dense, .. } = self;

        let (_, KeyValueVecs { keys, .. }) = dense.into_vecs();
        keys
    }
}

impl<K, V> Debug for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa,
    K::SparseIndex: Debug,
    SparseItem<K>: Debug,
    SoaVec<KeyValuePair<K, V>>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EpochSparseArena")
            .field("dense", &self.dense)
            .field("sparse", &self.sparse)
            .field("sparse_vacant_head", &self.sparse_vacant_head)
            .finish()
    }
}

impl<K, V> Default for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa,
    V::Context: Default,
{
    fn default() -> Self {
        Self {
            dense: Default::default(),
            sparse: Default::default(),
            sparse_vacant_head: Default::default(),
        }
    }
}

impl<K, V> PartialEq for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa,
    SoaVec<KeyValuePair<K, V>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.dense == other.dense
            && self.sparse == other.sparse
            && self.sparse_vacant_head == other.sparse_vacant_head
    }
}

impl<K, V> Eq for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa,
    SoaVec<KeyValuePair<K, V>>: Eq,
{
}

impl<K, V> PartialOrd for EpochSparseArena<K, V>
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
        match self.sparse.partial_cmp(&other.sparse) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.sparse_vacant_head
            .partial_cmp(&other.sparse_vacant_head)
    }
}

impl<K, V> Ord for EpochSparseArena<K, V>
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
        match self.sparse.cmp(&other.sparse) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.sparse_vacant_head.cmp(&other.sparse_vacant_head)
    }
}

impl<K, V> Hash for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa,
    K::SparseIndex: Hash,
    SparseItem<K>: Hash,
    SoaVec<KeyValuePair<K, V>>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.dense.hash(state);
        self.sparse.hash(state);
        self.sparse_vacant_head.hash(state);
    }
}

impl<K, V> Clone for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa,
    SoaVec<KeyValuePair<K, V>>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            dense: self.dense.clone(),
            sparse: self.sparse.clone(),
            sparse_vacant_head: self.sparse_vacant_head.clone(),
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
    for<'a> V: Soa<Refs<'a> = &'a T> + 'a,
{
    type Output = T;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        EpochSparseArena::index(self, key)
    }
}

impl<T, K, V> IndexMut<K> for EpochSparseArena<K, V>
where
    K: Key + Debug,
    for<'a> V: Soa<Refs<'a> = &'a T, RefsMut<'a> = &'a mut T> + 'a,
{
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        EpochSparseArena::index_mut(self, key)
    }
}

impl<T, K, V> AsRef<[T]> for EpochSparseArena<K, V>
where
    K: Key,
    for<'a> V: Soa<Slices<'a> = &'a [T]> + 'a,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slices()
    }
}

impl<T, K, V> AsMut<[T]> for EpochSparseArena<K, V>
where
    K: Key,
    for<'a> V: Soa<SlicesMut<'a> = &'a mut [T]> + 'a,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slices()
    }
}

impl<K, V> AsRef<EpochSparseArena<K, V>> for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa,
{
    #[inline]
    fn as_ref(&self) -> &EpochSparseArena<K, V> {
        self
    }
}

impl<K, V> AsMut<EpochSparseArena<K, V>> for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa,
{
    #[inline]
    fn as_mut(&mut self) -> &mut EpochSparseArena<K, V> {
        self
    }
}

impl<'a, K, V> IntoIterator for &'a EpochSparseArena<K, V>
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

impl<'a, K, V> IntoIterator for &'a mut EpochSparseArena<K, V>
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

impl<K, V> IntoIterator for EpochSparseArena<K, V>
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

impl<K, V> FromIterator<KeyValuePair<K, V>> for EpochSparseArena<K, V>
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

impl<K, V> FromIterator<(K, V)> for EpochSparseArena<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa,
    V::Context: Default,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        iter.into_iter().map(KeyValuePair::from).collect()
    }
}

impl<K, V> FromIterator<V> for EpochSparseArena<K, V>
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
        let sparse_vacant_head = len;

        Self {
            dense,
            sparse,
            sparse_vacant_head,
        }
    }
}

impl<K, V> Extend<KeyValuePair<K, V>> for EpochSparseArena<K, V>
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

impl<K, V> Extend<(K, V)> for EpochSparseArena<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa,
{
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        self.extend(iter.into_iter().map(KeyValuePair::from))
    }
}

impl<K, V> Extend<V> for EpochSparseArena<K, V>
where
    K: Key<SparseIndex = usize>,
    V: Soa,
{
    fn extend<I: IntoIterator<Item = V>>(&mut self, iter: I) {
        let mut iter = iter.into_iter();
        while let Some(value) = iter.next() {
            if self.len() == self.capacity() {
                let (lower, _) = iter.size_hint();
                self.reserve(lower.saturating_add(1), lower.saturating_add(1));
            }
            self.push(value).unwrap();
        }
    }
}

impl<K, V> From<set::EpochSparseSet<K, V>> for EpochSparseArena<K, V>
where
    K: Key,
    V: Soa,
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
