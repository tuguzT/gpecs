use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem::swap,
    ops::{Index, IndexMut},
    ptr, slice,
};

use crate::{
    algo::{
        sparse_contains_key, sparse_get, sparse_get_epoch, sparse_get_unchecked,
        sparse_get_with_key, sparse_index,
    },
    assert::{
        check_compatible_key, check_equal_epoch, check_equal_key, unwrap_dense,
        unwrap_dense_from_sparse_index, unwrap_dense_index, unwrap_dense_index_mut,
        unwrap_dense_pair, unwrap_into_index, unwrap_into_usize, unwrap_sparse_item,
        unwrap_sparse_items_pair_mut,
    },
    error::{
        DenseIndexMismatchError, DenseIndexOutOfBoundsError, EpochMismatchError, FromPartsError,
        OccupiedSparseItemExpectedError, SparseIndexMismatchError, SparseIndexOutOfBoundsError,
        TooLargeSparseIndexError, TooSmallSparseIndexError,
    },
    item::{SparseItem, SparseItemKind},
    iter::{
        Iter, IterMut, Keys, RawIter, RawIterMut, RawKeys, RawValues, RawValuesMut, Values,
        ValuesMut,
    },
    key::{Epoch, Key},
    pair::{
        KeyValueMutPtrs, KeyValuePair, KeyValuePairContext, KeyValuePtrs, KeyValueRefs,
        KeyValueSlicePtrs,
    },
    soa::{
        self,
        slice::{Iter as SoaIter, SoaSlicePtrs, SoaSlices, SoaSlicesMut},
        traits::{MutPtrs, Ptrs, RawSoa, SlicePtrs, Soa},
    },
};

pub type SparseViewPtr<'c, T> = EpochSparseViewPtr<'c, usize, T>;

pub struct EpochSparseViewPtr<'c, K, V>
where
    K: Key + 'c,
    V: RawSoa + ?Sized + 'c,
{
    dense: SoaSlicePtrs<'c, KeyValuePair<K, V>>,
    sparse: *const [SparseItem<K>],
}

impl<'c, K, V> EpochSparseViewPtr<'c, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    pub unsafe fn from_parts(
        dense: SoaSlicePtrs<'c, KeyValuePair<K, V>>,
        sparse: *const [SparseItem<K>],
    ) -> Self {
        Self { dense, sparse }
    }

    #[inline]
    pub fn into_parts(self) -> (SoaSlicePtrs<'c, KeyValuePair<K, V>>, *const [SparseItem<K>]) {
        let Self { dense, sparse } = self;
        (dense, sparse)
    }

    // TODO: cast_mut

    #[inline]
    pub unsafe fn deref<'a>(self) -> EpochSparseView<'c, 'a, K, V> {
        let Self { dense, sparse } = self;

        let dense = unsafe { dense.deref() };
        let sparse = unsafe { slice::from_raw_parts(sparse.cast(), sparse.len()) };
        unsafe { EpochSparseView::new_unchecked(dense, sparse) }
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
    pub fn as_ptrs(&self) -> (KeyValuePtrs<'_, K, V>, *const SparseItem<K>) {
        let (_, dense, sparse) = self.as_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_ptrs_with_context(
        &self,
    ) -> (&V::Context, KeyValuePtrs<'_, K, V>, *const SparseItem<K>) {
        let Self { ref dense, sparse } = *self;

        let (context, dense) = dense.as_ptrs_with_context();
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
    pub fn into_ptrs(self) -> (KeyValuePtrs<'c, K, V>, *const SparseItem<K>) {
        let (_, dense, sparse) = self.into_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_ptrs_with_context(
        self,
    ) -> (&'c V::Context, KeyValuePtrs<'c, K, V>, *const SparseItem<K>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_ptrs_with_context();
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
    pub fn into_dense_ptrs(self) -> KeyValuePtrs<'c, K, V> {
        let (_, dense) = self.into_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_ptrs_with_context(self) -> (&'c V::Context, KeyValuePtrs<'c, K, V>) {
        let (context, dense, _) = self.into_ptrs_with_context();
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
    pub fn as_slice_ptrs(&self) -> (KeyValueSlicePtrs<'_, K, V>, *const [SparseItem<K>]) {
        let (_, dense, sparse) = self.as_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(
        &self,
    ) -> (
        &V::Context,
        KeyValueSlicePtrs<'_, K, V>,
        *const [SparseItem<K>],
    ) {
        let Self { ref dense, sparse } = *self;
        let (context, dense) = dense.as_slice_ptrs_with_context();
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
    pub fn into_slice_ptrs(self) -> (KeyValueSlicePtrs<'c, K, V>, *const [SparseItem<K>]) {
        let (_, dense, sparse) = self.into_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(
        self,
    ) -> (
        &'c V::Context,
        KeyValueSlicePtrs<'c, K, V>,
        *const [SparseItem<K>],
    ) {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.into_slice_ptrs_with_context();
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
    pub fn into_dense_slice_ptrs(self) -> KeyValueSlicePtrs<'c, K, V> {
        let (_, dense) = self.into_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_slice_ptrs_with_context(
        self,
    ) -> (&'c V::Context, KeyValueSlicePtrs<'c, K, V>) {
        let (context, dense, _) = self.into_slice_ptrs_with_context();
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
    pub unsafe fn into_get_unchecked(self, key: K) -> Ptrs<'c, V> {
        let (_, ptrs) = unsafe { self.into_get_unchecked_with_context(key) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_with_context(self, key: K) -> (&'c V::Context, Ptrs<'c, V>) {
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
        let iter = RawKeys::from_inner(inner);
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
    pub fn into_values(self) -> RawValues<'c, K, V> {
        let (_, iter) = self.into_values_with_context();
        iter
    }

    #[inline]
    pub fn into_values_with_context(self) -> (&'c V::Context, RawValues<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_iter_with_context();
        let iter = RawValues::from_inner(inner);
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
    pub fn into_iter_with_context(self) -> (&'c V::Context, RawIter<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, iter) = dense.into_iter_with_context();
        let iter = RawIter::from_inner(iter);
        (context, iter)
    }
}

impl<'c, K, V> From<&'c V::Context> for EpochSparseViewPtr<'c, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(context: &'c V::Context) -> Self {
        let context = KeyValuePairContext::from_inner_ref(context);
        let dense = SoaSlicePtrs::from(context);
        let sparse = ptr::from_ref(&[]);
        unsafe { Self::from_parts(dense, sparse) }
    }
}

impl<K, V> Debug for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    for<'c> SlicePtrs<'c, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { dense, sparse } = self;
        f.debug_struct("EpochSparseViewPtr")
            .field("dense", dense)
            .field("sparse", sparse)
            .finish()
    }
}

impl<K, V> Clone for EpochSparseViewPtr<'_, K, V>
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

impl<K, V> Copy for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    for<'c> Ptrs<'c, V>: Copy,
{
}

impl<K, V> PartialEq for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: PartialEq,
    for<'c> Ptrs<'c, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { dense, sparse } = self;
        *dense == other.dense && ptr::eq(*sparse, other.sparse)
    }
}

impl<K, V> Eq for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: Eq,
    for<'c> Ptrs<'c, V>: Eq,
{
}

impl<K, V> PartialOrd for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: PartialOrd,
    for<'c> Ptrs<'c, V>: PartialOrd,
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

impl<K, V> Ord for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: Ord,
    for<'c> Ptrs<'c, V>: Ord,
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

impl<K, V> Hash for EpochSparseViewPtr<'_, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
    V::Context: Hash,
    for<'c> Ptrs<'c, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { dense, sparse } = self;
        dense.hash(state);
        sparse.hash(state);
    }
}

impl<'r, K, V> IntoIterator for &'r EpochSparseViewPtr<'_, K, V>
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

impl<'c, K, V> IntoIterator for EpochSparseViewPtr<'c, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    type Item = (*const K, Ptrs<'c, V>);
    type IntoIter = RawIter<'c, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}

pub type SparseView<'c, 'a, T> = EpochSparseView<'c, 'a, usize, T>;

// TODO: add support for raw SoA types

pub struct EpochSparseView<'c, 'a, K, V>
where
    K: Key + 'c + 'a,
    V: RawSoa + ?Sized + 'c + 'a,
{
    dense: SoaSlices<'c, 'a, KeyValuePair<K, V>>,
    sparse: &'a [SparseItem<K>],
}

impl<'c, 'a, K, V> EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: RawSoa + ?Sized,
{
    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(
        dense: SoaSlices<'c, 'a, KeyValuePair<K, V>>,
        sparse: &'a [SparseItem<K>],
    ) -> Self {
        Self { dense, sparse }
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
        let Self { ref dense, sparse } = *self;

        let dense = dense.slice_ptrs();
        let sparse = ptr::from_ref(sparse);
        unsafe { EpochSparseViewPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn into_view_ptr(self) -> EpochSparseViewPtr<'c, K, V> {
        let Self { dense, sparse } = self;

        let dense = dense.into_slice_ptrs();
        let sparse = ptr::from_ref(sparse);
        unsafe { EpochSparseViewPtr::from_parts(dense, sparse) }
    }

    #[inline]
    pub fn view(&self) -> EpochSparseView<'_, '_, K, V> {
        unsafe { self.as_view_ptr().deref() }
    }

    #[inline]
    pub fn as_ptrs(&self) -> (KeyValuePtrs<'_, K, V>, *const SparseItem<K>) {
        let (_, dense, sparse) = self.as_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_ptrs_with_context(
        &self,
    ) -> (&V::Context, KeyValuePtrs<'_, K, V>, *const SparseItem<K>) {
        let Self { ref dense, sparse } = *self;

        let (context, dense) = dense.as_ptrs_with_context();
        let sparse = sparse.as_ptr();
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
    pub fn into_ptrs(self) -> (KeyValuePtrs<'c, K, V>, *const SparseItem<K>) {
        let (_, dense, sparse) = self.into_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_ptrs_with_context(
        self,
    ) -> (&'c V::Context, KeyValuePtrs<'c, K, V>, *const SparseItem<K>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_ptrs_with_context();
        let sparse = sparse.as_ptr();
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
    pub fn into_dense_ptrs(self) -> KeyValuePtrs<'c, K, V> {
        let (_, dense) = self.into_dense_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_ptrs_with_context(self) -> (&'c V::Context, KeyValuePtrs<'c, K, V>) {
        let (context, dense, _) = self.into_ptrs_with_context();
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
    pub fn as_slice_ptrs(&self) -> (KeyValueSlicePtrs<'_, K, V>, *const [SparseItem<K>]) {
        let (_, dense, sparse) = self.as_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(
        &self,
    ) -> (
        &V::Context,
        KeyValueSlicePtrs<'_, K, V>,
        *const [SparseItem<K>],
    ) {
        let Self { ref dense, sparse } = *self;
        let (context, dense) = dense.as_slice_ptrs_with_context();
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
    pub fn into_slice_ptrs(self) -> (KeyValueSlicePtrs<'c, K, V>, *const [SparseItem<K>]) {
        let (_, dense, sparse) = self.into_slice_ptrs_with_context();
        (dense, sparse)
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(
        self,
    ) -> (
        &'c V::Context,
        KeyValueSlicePtrs<'c, K, V>,
        *const [SparseItem<K>],
    ) {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.into_slice_ptrs_with_context();
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
    pub fn into_dense_slice_ptrs(self) -> KeyValueSlicePtrs<'c, K, V> {
        let (_, dense) = self.into_dense_slice_ptrs_with_context();
        dense
    }

    #[inline]
    pub fn into_dense_slice_ptrs_with_context(
        self,
    ) -> (&'c V::Context, KeyValueSlicePtrs<'c, K, V>) {
        let (context, dense, _) = self.into_slice_ptrs_with_context();
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
    pub unsafe fn into_get_unchecked(self, key: K) -> Ptrs<'c, V> {
        let (_, ptrs) = unsafe { self.into_get_unchecked_with_context(key) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_with_context(self, key: K) -> (&'c V::Context, Ptrs<'c, V>) {
        let view_ptr = self.into_view_ptr();
        unsafe { view_ptr.into_get_unchecked_with_context(key) }
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
        let view_ptr = self.into_view_ptr();
        unsafe { view_ptr.into_get_with_key_unchecked_with_context(sparse_index) }
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
    pub fn into_raw_keys(self) -> RawKeys<'c, K, V> {
        let (_, iter) = self.into_raw_keys_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_keys_with_context(self) -> (&'c V::Context, RawKeys<'c, K, V>) {
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
    pub fn into_raw_values(self) -> RawValues<'c, K, V> {
        let (_, iter) = self.into_raw_values_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_values_with_context(self) -> (&'c V::Context, RawValues<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_with_context();
        let iter = RawValues::from_inner(inner);
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
    pub fn into_raw_iter(self) -> RawIter<'c, K, V> {
        let (_, iter) = self.into_raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_iter_with_context(self) -> (&'c V::Context, RawIter<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_with_context();
        let iter = RawIter::from_inner(inner);
        (context, iter)
    }
}

impl<'c, 'a, K, V> EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized, // TODO: replace with RawSoa to see which methods to move
{
    #[inline]
    pub fn new(
        dense: SoaSlices<'c, 'a, KeyValuePair<K, V>>,
        sparse: &'a [SparseItem<K>],
    ) -> Result<Self, FromPartsError<K>> {
        check_parts(&dense, sparse)?;

        let me = unsafe { Self::new_unchecked(dense, sparse) };
        Ok(me)
    }

    // TODO: rename slice methods according to pointer ones

    #[inline]
    pub fn as_slices(&self) -> V::Slices<'_, '_> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&V::Context, V::Slices<'_, '_>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.as_slices_with_context();
        let (_, values) = slices.into_parts();
        (context, values)
    }

    #[inline]
    pub fn into_slices(self) -> V::Slices<'c, 'a> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'c V::Context, V::Slices<'c, 'a>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.into_slices_with_context();
        let (_, values) = slices.into_parts();
        (context, values)
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &'a [K] {
        let Self { dense, .. } = self;

        let (keys, _) = dense.clone().into_slices().into_parts();
        keys
    }

    #[inline]
    pub fn as_sparse_slice(&self) -> &'a [SparseItem<K>] {
        let Self { sparse, .. } = self;
        sparse
    }

    #[inline]
    pub fn into_parts(self) -> (SoaSlices<'c, 'a, KeyValuePair<K, V>>, &'a [SparseItem<K>]) {
        let Self { dense, sparse } = self;
        (dense, sparse)
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
    pub fn into_get(self, key: K) -> Option<V::Refs<'c, 'a>> {
        let (_, refs) = self.into_get_with_context(key);
        refs
    }

    #[inline]
    pub fn into_get_with_context(self, key: K) -> (&'c V::Context, Option<V::Refs<'c, 'a>>) {
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
    pub fn into_index(self, key: K) -> V::Refs<'c, 'a>
    where
        K: Debug,
    {
        let (_, refs) = self.into_index_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_with_context(self, key: K) -> (&'c V::Context, V::Refs<'c, 'a>)
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
    pub fn into_get_with_key(self, sparse_index: K::SparseIndex) -> Option<(K, V::Refs<'c, 'a>)> {
        let (_, pair) = self.into_get_with_key_with_context(sparse_index);
        pair
    }

    #[inline]
    pub fn into_get_with_key_with_context(
        self,
        sparse_index: K::SparseIndex,
    ) -> (&'c V::Context, Option<(K, V::Refs<'c, 'a>)>) {
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
    pub fn into_keys(self) -> Keys<'c, 'a, K, V> {
        let (_, iter) = self.into_keys_with_context();
        iter
    }

    #[inline]
    pub fn into_keys_with_context(self) -> (&'c V::Context, Keys<'c, 'a, K, V>) {
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
    pub fn into_values(self) -> Values<'c, 'a, K, V> {
        let (_, iter) = self.into_values_with_context();
        iter
    }

    #[inline]
    pub fn into_values_with_context(self) -> (&'c V::Context, Values<'c, 'a, K, V>) {
        let (context, iter) = self.into_raw_values_with_context();
        let iter = unsafe { iter.deref() };
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
    pub fn into_iter_with_context(self) -> (&'c V::Context, Iter<'c, 'a, K, V>) {
        let (context, iter) = self.into_raw_iter_with_context();
        let iter = unsafe { iter.deref() };
        (context, iter)
    }
}

impl<'c, 'a, K, V> Debug for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SparseItem<K>: Debug,
    SoaSlices<'c, 'a, KeyValuePair<K, V>>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { dense, sparse } = self;
        f.debug_struct("EpochSparseView")
            .field("dense", dense)
            .field("sparse", sparse)
            .finish()
    }
}

impl<'c, K, V> From<&'c V::Context> for EpochSparseView<'c, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn from(context: &'c V::Context) -> Self {
        let view_ptr = EpochSparseViewPtr::from(context);
        unsafe { view_ptr.deref() }
    }
}

impl<K, V> Clone for EpochSparseView<'_, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { dense, sparse } = self;
        let dense = dense.clone();
        Self { dense, sparse }
    }
}

impl<'c, 'a, K, V> Copy for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlices<'c, 'a, KeyValuePair<K, V>>: Copy,
{
}

impl<'c, 'a, K, V> PartialEq for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlices<'c, 'a, KeyValuePair<K, V>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { dense, sparse } = self;
        *dense == other.dense && *sparse == other.sparse
    }
}

impl<'c, 'a, K, V> Eq for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlices<'c, 'a, KeyValuePair<K, V>>: Eq,
{
}

impl<'c, 'a, K, V> PartialOrd for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlices<'c, 'a, KeyValuePair<K, V>>: PartialOrd,
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

impl<'c, 'a, K, V> Ord for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlices<'c, 'a, KeyValuePair<K, V>>: Ord,
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

impl<'c, 'a, K, V> Hash for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SparseItem<K>: Hash,
    SoaSlices<'c, 'a, KeyValuePair<K, V>>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { dense, sparse } = self;
        dense.hash(state);
        sparse.hash(state);
    }
}

impl<T, K, V> Index<K> for EpochSparseView<'_, '_, K, V>
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

impl<T, K, V> AsRef<[T]> for EpochSparseView<'_, '_, K, V>
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

impl<K, V> AsRef<Self> for EpochSparseView<'_, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<'r, K, V> IntoIterator for &'r EpochSparseView<'_, '_, K, V>
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

impl<'c, 'a, K, V> IntoIterator for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    type Item = (&'a K, V::Refs<'c, 'a>);
    type IntoIter = Iter<'c, 'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}

pub type SparseViewMut<'c, 'a, T> = EpochSparseViewMut<'c, 'a, usize, T>;

pub struct EpochSparseViewMut<'c, 'a, K, V>
where
    K: Key + 'c + 'a,
    V: Soa + ?Sized + 'c + 'a,
{
    dense: SoaSlicesMut<'c, 'a, KeyValuePair<K, V>>,
    sparse: &'a mut [SparseItem<K>],
}

impl<'c, 'a, K, V> EpochSparseViewMut<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    pub fn new(
        dense: SoaSlicesMut<'c, 'a, KeyValuePair<K, V>>,
        sparse: &'a mut [SparseItem<K>],
    ) -> Result<Self, FromPartsError<K>> {
        check_parts(&dense.slices(), sparse)?;
        Ok(Self { dense, sparse })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(
        dense: SoaSlicesMut<'c, 'a, KeyValuePair<K, V>>,
        sparse: &'a mut [SparseItem<K>],
    ) -> Self {
        if cfg!(debug_assertions) {
            let Ok(view) = Self::new(dense, sparse) else {
                panic!("incorrect inputs");
            };
            return view;
        }

        Self { dense, sparse }
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
    pub fn as_slices(&self) -> V::Slices<'_, '_> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&V::Context, V::Slices<'_, '_>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.as_slices_with_context();
        let (_, values) = slices.into_parts();
        (context, values)
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> V::SlicesMut<'_, '_> {
        let (_, slices) = self.as_mut_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_mut_slices_with_context(&mut self) -> (&V::Context, V::SlicesMut<'_, '_>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.as_mut_slices_with_context();
        let (_, values) = slices.into_parts();
        (context, values)
    }

    #[inline]
    pub fn into_slices(self) -> V::SlicesMut<'c, 'a> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'c V::Context, V::SlicesMut<'c, 'a>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.into_slices_with_context();
        let (_, values) = slices.into_parts();
        (context, values)
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'_, V> {
        let Self { dense, .. } = self;

        let KeyValuePtrs { value, .. } = dense.as_ptrs();
        value.into_inner()
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'_, V> {
        let Self { dense, .. } = self;

        let KeyValueMutPtrs { value, .. } = dense.as_mut_ptrs();
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
    pub fn into_keys_slice(self) -> &'a [K] {
        let Self { dense, .. } = self;

        let (keys, _) = dense.into_slices().into_parts();
        keys
    }

    #[inline]
    pub unsafe fn into_keys_slice_mut(self) -> &'a mut [K] {
        let Self { dense, .. } = self;

        let (keys, _) = dense.into_slices().into_parts();
        keys
    }

    #[inline]
    pub fn as_keys_ptr(&self) -> *const K {
        let Self { dense, .. } = self;

        let KeyValuePtrs { key, .. } = dense.as_ptrs();
        key
    }

    #[inline]
    pub unsafe fn as_keys_ptr_mut(&mut self) -> *mut K {
        let Self { dense, .. } = self;

        let KeyValueMutPtrs { key, .. } = dense.as_mut_ptrs();
        key
    }

    #[inline]
    pub fn as_sparse_slice(&self) -> &[SparseItem<K>] {
        let Self { sparse, .. } = self;
        sparse
    }

    #[inline]
    pub unsafe fn as_sparse_slice_mut(&mut self) -> &mut [SparseItem<K>] {
        let Self { sparse, .. } = self;
        sparse
    }

    #[inline]
    pub fn into_sparse_slice(self) -> &'a [SparseItem<K>] {
        let Self { sparse, .. } = self;
        sparse
    }

    #[inline]
    pub unsafe fn into_sparse_slice_mut(self) -> &'a mut [SparseItem<K>] {
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
    pub fn into_parts(
        self,
    ) -> (
        SoaSlicesMut<'c, 'a, KeyValuePair<K, V>>,
        &'a mut [SparseItem<K>],
    ) {
        let Self { dense, sparse } = self;
        (dense, sparse)
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
        let dense_key: &mut K = unwrap_dense(keys, dense_index);
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
        let dense_key: &mut K = unwrap_dense(keys, dense_index);
        check_compatible_key(key, *dense_key);

        *dense_key = key;
        Some(*dense_key)
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        for<'ca, 'any> V::Refs<'ca, 'any>: Ord,
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
    pub fn into_get(self, key: K) -> Option<V::Refs<'c, 'a>> {
        let (_, refs) = self.into_get_with_context(key);
        refs
    }

    #[inline]
    pub fn into_get_with_context(self, key: K) -> (&'c V::Context, Option<V::Refs<'c, 'a>>) {
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
    pub fn into_get_mut(self, key: K) -> Option<V::RefsMut<'c, 'a>> {
        let (_, refs) = self.into_get_mut_with_context(key);
        refs
    }

    #[inline]
    pub fn into_get_mut_with_context(self, key: K) -> (&'c V::Context, Option<V::RefsMut<'c, 'a>>) {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let refs = sparse_get(dense.map(From::from), sparse, key);
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
    pub fn into_get_with_key(self, sparse_index: K::SparseIndex) -> Option<(K, V::Refs<'c, 'a>)> {
        let (_, pair) = self.into_get_with_key_with_context(sparse_index);
        pair
    }

    #[inline]
    pub fn into_get_with_key_with_context(
        self,
        sparse_index: K::SparseIndex,
    ) -> (&'c V::Context, Option<(K, V::Refs<'c, 'a>)>) {
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
    ) -> Option<(K, V::RefsMut<'c, 'a>)> {
        let (_, pair) = self.into_get_mut_with_key_with_context(sparse_index);
        pair
    }

    #[inline]
    pub fn into_get_mut_with_key_with_context(
        self,
        sparse_index: K::SparseIndex,
    ) -> (&'c V::Context, Option<(K, V::RefsMut<'c, 'a>)>) {
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
    pub fn into_raw_keys(self) -> RawKeys<'c, K, V> {
        let (_, iter) = self.into_raw_keys_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_keys_with_context(self) -> (&'c V::Context, RawKeys<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_with_context();
        let iter = RawKeys::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_keys(self) -> Keys<'c, 'a, K, V> {
        let (_, iter) = self.into_keys_with_context();
        iter
    }

    #[inline]
    pub fn into_keys_with_context(self) -> (&'c V::Context, Keys<'c, 'a, K, V>) {
        let (context, iter) = self.into_raw_keys_with_context();
        let iter = unsafe { iter.deref() };
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
    pub fn into_raw_values(self) -> RawValues<'c, K, V> {
        let (_, iter) = self.into_raw_values_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_values_with_context(self) -> (&'c V::Context, RawValues<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_with_context();
        let iter = RawValues::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_values(self) -> Values<'c, 'a, K, V> {
        let (_, iter) = self.into_values_with_context();
        iter
    }

    #[inline]
    pub fn into_values_with_context(self) -> (&'c V::Context, Values<'c, 'a, K, V>) {
        let (context, iter) = self.into_raw_values_with_context();
        let iter = unsafe { iter.deref() };
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
    pub fn into_raw_values_mut(self) -> RawValuesMut<'c, K, V> {
        let (_, iter) = self.into_raw_values_mut_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_values_mut_with_context(self) -> (&'c V::Context, RawValuesMut<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_mut_with_context();
        let iter = RawValuesMut::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_values_mut(self) -> ValuesMut<'c, 'a, K, V> {
        let (_, iter) = self.into_values_mut_with_context();
        iter
    }

    #[inline]
    pub fn into_values_mut_with_context(self) -> (&'c V::Context, ValuesMut<'c, 'a, K, V>) {
        let (context, iter) = self.into_raw_values_mut_with_context();
        let iter = unsafe { iter.deref_mut() };
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
    pub fn into_raw_iter(self) -> RawIter<'c, K, V> {
        let (_, iter) = self.into_raw_iter_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_iter_with_context(self) -> (&'c V::Context, RawIter<'c, K, V>) {
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
    pub fn into_raw_iter_mut(self) -> RawIterMut<'c, K, V> {
        let (_, iter) = self.into_raw_iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn into_raw_iter_mut_with_context(self) -> (&'c V::Context, RawIterMut<'c, K, V>) {
        let Self { dense, .. } = self;

        let (context, inner) = dense.into_raw_iter_mut_with_context();
        let iter = RawIterMut::from_inner(inner);
        (context, iter)
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'c V::Context, IterMut<'c, 'a, K, V>) {
        let (context, iter) = self.into_raw_iter_mut_with_context();
        let iter = unsafe { iter.deref_mut() };
        (context, iter)
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
    pub fn into_index(self, key: K) -> V::Refs<'c, 'a>
    where
        K: Debug,
    {
        let (_, refs) = self.into_index_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_with_context(self, key: K) -> (&'c V::Context, V::Refs<'c, 'a>)
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
    pub fn into_index_mut(self, key: K) -> V::RefsMut<'c, 'a>
    where
        K: Debug,
    {
        let (_, refs) = self.into_index_mut_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_mut_with_context(self, key: K) -> (&'c V::Context, V::RefsMut<'c, 'a>)
    where
        K: Debug,
    {
        let Self { dense, sparse } = self;

        let (context, dense) = dense.into_iter_with_context();
        let refs = sparse_index(dense.map(From::from), sparse, key);
        (context, refs)
    }
}

impl<'c, 'a, K, V> Debug for EpochSparseViewMut<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SparseItem<K>: Debug,
    SoaSlicesMut<'c, 'a, KeyValuePair<K, V>>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { dense, sparse } = self;
        f.debug_struct("EpochSparseViewMut")
            .field("dense", dense)
            .field("sparse", sparse)
            .finish()
    }
}

impl<'c, K, V> From<&'c V::Context> for EpochSparseViewMut<'c, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn from(context: &'c V::Context) -> Self {
        let context = KeyValuePairContext::<K, V>::from_inner_ref(context);
        Self {
            dense: context.into(),
            sparse: Default::default(),
        }
    }
}

impl<'c, 'a, K, V> PartialEq for EpochSparseViewMut<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlicesMut<'c, 'a, KeyValuePair<K, V>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { dense, sparse } = self;
        *dense == other.dense && *sparse == other.sparse
    }
}

impl<'c, 'a, K, V> Eq for EpochSparseViewMut<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlicesMut<'c, 'a, KeyValuePair<K, V>>: Eq,
{
}

impl<'c, 'a, K, V> PartialOrd for EpochSparseViewMut<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlicesMut<'c, 'a, KeyValuePair<K, V>>: PartialOrd,
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

impl<'c, 'a, K, V> Ord for EpochSparseViewMut<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SoaSlicesMut<'c, 'a, KeyValuePair<K, V>>: Ord,
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

impl<'c, 'a, K, V> Hash for EpochSparseViewMut<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
    SparseItem<K>: Hash,
    SoaSlicesMut<'c, 'a, KeyValuePair<K, V>>: Hash,
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
    for<'c, 'any> V: Soa<Refs<'c, 'any> = &'any T> + 'any,
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
    for<'c, 'any> V: Soa<Refs<'c, 'any> = &'any T, RefsMut<'c, 'any> = &'any mut T> + 'any,
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
    for<'c, 'any> V::Slices<'c, 'any>: Into<&'any [T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slices().into()
    }
}

impl<T, K, V> AsMut<[T]> for EpochSparseViewMut<'_, '_, K, V>
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

impl<K, V> AsRef<Self> for EpochSparseViewMut<'_, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<K, V> AsMut<Self> for EpochSparseViewMut<'_, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'r, K, V> IntoIterator for &'r EpochSparseViewMut<'_, '_, K, V>
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

impl<'r, K, V> IntoIterator for &'r mut EpochSparseViewMut<'_, '_, K, V>
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

impl<'c, 'a, K, V> IntoIterator for EpochSparseViewMut<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    type Item = (&'a K, V::RefsMut<'c, 'a>);
    type IntoIter = IterMut<'c, 'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}

impl<'c, 'a, K, V> From<EpochSparseViewMut<'c, 'a, K, V>> for EpochSparseView<'c, 'a, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: EpochSparseViewMut<'c, 'a, K, V>) -> Self {
        let EpochSparseViewMut { dense, sparse } = value;
        unsafe { Self::new_unchecked(dense.into(), sparse) }
    }
}

fn check_parts<K, V>(
    dense: &SoaSlices<KeyValuePair<K, V>>,
    sparse: &[SparseItem<K>],
) -> Result<(), FromPartsError<K>>
where
    K: Key,
    V: Soa + ?Sized,
{
    for (sparse_index, &SparseItem { kind, epoch }) in sparse.iter().enumerate() {
        let sparse_index = sparse_index
            .try_into()
            .map_err(TooSmallSparseIndexError::new)?;
        let Some(dense_index) = kind.dense_index().copied() else {
            continue;
        };

        let dense_index = dense_index
            .try_into()
            .map_err(TooLargeSparseIndexError::new)?;
        let KeyValueRefs { key, .. } = dense
            .get(dense_index)
            .ok_or_else(|| DenseIndexOutOfBoundsError::new(dense_index, dense.len()))?;

        let sparse_index_from_key = key.sparse_index();
        if sparse_index_from_key != sparse_index {
            let error = SparseIndexMismatchError::new(sparse_index_from_key, sparse_index);
            return Err(error.into());
        }

        let epoch_from_key = key.epoch();
        let expected_epoch = epoch;
        if epoch_from_key != expected_epoch {
            let error = EpochMismatchError::new(epoch_from_key, expected_epoch);
            return Err(error.into());
        }
    }
    for (dense_index, KeyValueRefs { key, .. }) in dense.iter().enumerate() {
        let sparse_index = key
            .sparse_index()
            .try_into()
            .map_err(TooLargeSparseIndexError::new)?;
        let sparse_item = sparse
            .get(sparse_index)
            .ok_or_else(|| SparseIndexOutOfBoundsError::new(sparse_index, sparse.len()))?;

        let dense_index = dense_index
            .try_into()
            .map_err(TooSmallSparseIndexError::new)?;
        let dense_index_from_item = match *sparse_item.kind() {
            SparseItemKind::Occupied { dense_index } => dense_index,
            SparseItemKind::Vacant { next_vacant } => {
                let error = OccupiedSparseItemExpectedError::new(next_vacant);
                return Err(error.into());
            }
        };
        if dense_index_from_item != dense_index {
            let error = DenseIndexMismatchError::new(dense_index_from_item, dense_index);
            return Err(error.into());
        }

        let epoch_from_item = sparse_item.epoch;
        let expected_epoch = key.epoch();
        if epoch_from_item != expected_epoch {
            let error = EpochMismatchError::new(epoch_from_item, expected_epoch);
            return Err(error.into());
        }
    }
    Ok(())
}
