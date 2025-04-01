use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem::swap,
    ops::{Index, IndexMut},
};

use crate::{
    algo::{
        sparse_contains_key, sparse_get, sparse_get_epoch, sparse_get_mut, sparse_get_mut_with_key,
        sparse_get_with_key, sparse_index, sparse_index_mut,
    },
    assert::{
        check_compatible_key, check_equal_epoch, check_equal_key, unwrap_dense,
        unwrap_dense_from_sparse_index, unwrap_dense_index, unwrap_dense_index_mut,
        unwrap_dense_pair, unwrap_into_index, unwrap_into_usize, unwrap_sparse_item,
        unwrap_sparse_items_pair_mut,
    },
    item::SparseItem,
    iter::{Iter, IterMut, Keys, Values, ValuesMut},
    key::{Epoch, Key},
    pair::{KeyValueMutPtrs, KeyValuePair, KeyValuePtrs, KeyValueSlices, KeyValueSlicesMut},
    soa::{
        mem::swap as soa_swap,
        slice::{Iter as SoaIter, SoaSlices, SoaSlicesMut},
        traits::Soa,
    },
};

pub type SparseView<'a, T> = EpochSparseView<'a, usize, T>;

pub struct EpochSparseView<'a, K, V>
where
    K: Key,
    V: Soa,
{
    dense: SoaSlices<'a, KeyValuePair<K, V>>,
    sparse: &'a [SparseItem<K>],
}

impl<'a, K, V> EpochSparseView<'a, K, V>
where
    K: Key,
    V: Soa,
{
    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn new_unchecked(
        dense: SoaSlices<'a, KeyValuePair<K, V>>,
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
    pub fn as_slices(&self) -> V::Slices<'_> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&V::Context, V::Slices<'_>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.as_slices_with_context();
        let KeyValueSlices { values, .. } = slices;
        (context, values)
    }

    #[inline]
    pub fn into_slices(self) -> V::Slices<'a> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'a V::Context, V::Slices<'a>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.into_slices_with_context();
        let KeyValueSlices { values, .. } = slices;
        (context, values)
    }

    #[inline]
    pub fn as_ptrs(&self) -> V::Ptrs {
        let Self { dense, .. } = self;

        let KeyValuePtrs { value, .. } = dense.as_ptrs();
        value
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &'a [K] {
        let Self { dense, .. } = self;

        let KeyValueSlices { keys, .. } = dense.clone().into_slices();
        keys
    }

    #[inline]
    pub fn as_keys_ptr(&self) -> *const K {
        let Self { dense, .. } = self;

        let KeyValuePtrs { key, .. } = dense.as_ptrs();
        key
    }

    #[inline]
    pub fn as_sparse_slice(&self) -> &'a [SparseItem<K>] {
        let Self { sparse, .. } = self;
        sparse
    }

    #[inline]
    pub fn as_sparse_ptr(&self) -> *const SparseItem<K> {
        let Self { sparse, .. } = self;
        sparse.as_ptr()
    }

    #[inline]
    pub fn into_parts(self) -> (SoaSlices<'a, KeyValuePair<K, V>>, &'a [SparseItem<K>]) {
        let Self { dense, sparse } = self;
        (dense, sparse)
    }

    #[inline]
    pub fn get(&self, key: K) -> Option<V::Refs<'_>> {
        let (_, refs) = self.get_with_context(key);
        refs
    }

    #[inline]
    pub fn get_with_context(&self, key: K) -> (&V::Context, Option<V::Refs<'_>>) {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.iter_with_context();
        (context, sparse_get(dense, sparse, key))
    }

    #[inline]
    pub fn into_get(self, key: K) -> Option<V::Refs<'a>> {
        let (_, refs) = self.into_get_with_context(key);
        refs
    }

    #[inline]
    pub fn into_get_with_context(self, key: K) -> (&'a V::Context, Option<V::Refs<'a>>) {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.into_iter_with_context();
        (context, sparse_get(dense, sparse, key))
    }

    #[inline]
    pub fn get_with_key(&self, sparse_index: K::SparseIndex) -> Option<(K, V::Refs<'_>)> {
        let Self { dense, sparse } = self;
        sparse_get_with_key(dense, sparse, sparse_index)
    }

    #[inline]
    pub fn into_get_with_key(self, sparse_index: K::SparseIndex) -> Option<(K, V::Refs<'a>)> {
        let Self { dense, sparse } = self;
        sparse_get_with_key(dense, sparse, sparse_index)
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: K::SparseIndex) -> Option<K::Epoch> {
        let Self { dense, sparse } = self;

        let KeyValueSlices { keys, .. } = dense.as_slices();
        sparse_get_epoch(keys, sparse, sparse_index)
    }

    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let Self { dense, sparse } = self;

        let KeyValueSlices { keys, .. } = dense.as_slices();
        sparse_contains_key(keys, sparse, key)
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        let Self { dense, .. } = self;
        let inner = dense.iter();
        Keys::new(inner)
    }

    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        let Self { dense, .. } = self;
        let inner = dense.iter();
        Values::new(inner)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        let Self { dense, .. } = self;
        let inner = dense.iter();
        Iter::new(inner)
    }

    #[inline]
    #[track_caller]
    pub fn index(&self, key: K) -> V::Refs<'_>
    where
        K: Debug,
    {
        let (_, refs) = self.index_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_with_context(&self, key: K) -> (&V::Context, V::Refs<'_>)
    where
        K: Debug,
    {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.iter_with_context();
        (context, sparse_index(dense, sparse, key))
    }

    #[inline]
    #[track_caller]
    pub fn into_index(self, key: K) -> V::Refs<'a>
    where
        K: Debug,
    {
        let (_, refs) = self.into_index_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_with_context(self, key: K) -> (&'a V::Context, V::Refs<'a>)
    where
        K: Debug,
    {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.into_iter_with_context();
        (context, sparse_index(dense, sparse, key))
    }
}

impl<'a, K, V> Debug for EpochSparseView<'a, K, V>
where
    K: Key,
    V: Soa,
    SparseItem<K>: Debug,
    SoaSlices<'a, KeyValuePair<K, V>>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { dense, sparse } = self;
        f.debug_struct("EpochSparseView")
            .field("dense", dense)
            .field("sparse", sparse)
            .finish()
    }
}

impl<'a, K, V> From<&'a V::Context> for EpochSparseView<'a, K, V>
where
    K: Key,
    V: Soa,
{
    fn from(context: &'a V::Context) -> Self {
        Self {
            dense: context.into(),
            sparse: Default::default(),
        }
    }
}

impl<K, V> Clone for EpochSparseView<'_, K, V>
where
    K: Key,
    V: Soa,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { dense, sparse } = self;
        let dense = dense.clone();
        Self { dense, sparse }
    }
}

impl<K, V> Copy for EpochSparseView<'_, K, V>
where
    K: Key,
    V: Soa,
    V::Ptrs: Copy,
{
}

impl<'a, K, V> PartialEq for EpochSparseView<'a, K, V>
where
    K: Key,
    V: Soa,
    SoaSlices<'a, KeyValuePair<K, V>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { dense, sparse } = self;
        *dense == other.dense && *sparse == other.sparse
    }
}

impl<'a, K, V> Eq for EpochSparseView<'a, K, V>
where
    K: Key,
    V: Soa,
    SoaSlices<'a, KeyValuePair<K, V>>: Eq,
{
}

impl<'a, K, V> PartialOrd for EpochSparseView<'a, K, V>
where
    K: Key,
    V: Soa,
    SoaSlices<'a, KeyValuePair<K, V>>: PartialOrd,
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

impl<'a, K, V> Ord for EpochSparseView<'a, K, V>
where
    K: Key,
    V: Soa,
    SoaSlices<'a, KeyValuePair<K, V>>: Ord,
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

impl<'a, K, V> Hash for EpochSparseView<'a, K, V>
where
    K: Key,
    V: Soa,
    SparseItem<K>: Hash,
    SoaSlices<'a, KeyValuePair<K, V>>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { dense, sparse } = self;
        dense.hash(state);
        sparse.hash(state);
    }
}

impl<T, K, V> Index<K> for EpochSparseView<'_, K, V>
where
    K: Key + Debug,
    for<'a> V: Soa<Refs<'a> = &'a T> + 'a,
{
    type Output = T;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        EpochSparseView::index(self, key)
    }
}

impl<T, K, V> AsRef<[T]> for EpochSparseView<'_, K, V>
where
    K: Key,
    for<'a> V: Soa<Slices<'a> = &'a [T]> + 'a,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slices()
    }
}

impl<'a, K, V> AsRef<EpochSparseView<'a, K, V>> for EpochSparseView<'a, K, V>
where
    K: Key,
    V: Soa,
{
    #[inline]
    fn as_ref(&self) -> &EpochSparseView<'a, K, V> {
        self
    }
}

impl<'a, K, V> IntoIterator for &'a EpochSparseView<'_, K, V>
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

impl<'a, K, V> IntoIterator for EpochSparseView<'a, K, V>
where
    K: Key,
    V: Soa,
{
    type Item = (&'a K, V::Refs<'a>);

    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { dense, .. } = self;
        let inner = dense.into_iter();
        Iter::new(inner)
    }
}

pub type SparseViewMut<'a, T> = EpochSparseViewMut<'a, usize, T>;

pub struct EpochSparseViewMut<'a, K, V>
where
    K: Key,
    V: Soa,
{
    dense: SoaSlicesMut<'a, KeyValuePair<K, V>>,
    sparse: &'a mut [SparseItem<K>],
}

impl<'a, K, V> EpochSparseViewMut<'a, K, V>
where
    K: Key,
    V: Soa,
{
    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn new_unchecked(
        dense: SoaSlicesMut<'a, KeyValuePair<K, V>>,
        sparse: &'a mut [SparseItem<K>],
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
    pub fn as_slices(&self) -> V::Slices<'_> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&V::Context, V::Slices<'_>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.as_slices_with_context();
        let KeyValueSlices { values, .. } = slices;
        (context, values)
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> V::SlicesMut<'_> {
        let (_, slices) = self.as_mut_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_mut_slices_with_context(&mut self) -> (&V::Context, V::SlicesMut<'_>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.as_mut_slices_with_context();
        let KeyValueSlicesMut { values, .. } = slices;
        (context, values)
    }

    #[inline]
    pub fn into_slices(self) -> V::SlicesMut<'a> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    pub fn into_slices_with_context(self) -> (&'a V::Context, V::SlicesMut<'a>) {
        let Self { dense, .. } = self;

        let (context, slices) = dense.into_slices_with_context();
        let KeyValueSlicesMut { values, .. } = slices;
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
    #[allow(unsafe_code)]
    pub unsafe fn as_keys_slice_mut(&mut self) -> &mut [K] {
        let Self { dense, .. } = self;

        let KeyValueSlicesMut { keys, .. } = dense.as_mut_slices();
        keys
    }

    #[inline]
    pub fn into_keys_slice(self) -> &'a [K] {
        let Self { dense, .. } = self;

        let KeyValueSlicesMut { keys, .. } = dense.into_slices();
        keys
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn into_keys_slice_mut(self) -> &'a mut [K] {
        let Self { dense, .. } = self;

        let KeyValueSlicesMut { keys, .. } = dense.into_slices();
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
        sparse
    }

    #[inline]
    #[allow(unsafe_code)]
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
    #[allow(unsafe_code)]
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
    #[allow(unsafe_code)]
    pub unsafe fn as_sparse_ptr_mut(&mut self) -> *mut SparseItem<K> {
        let Self { sparse, .. } = self;
        sparse.as_mut_ptr()
    }

    #[inline]
    pub fn into_parts(
        self,
    ) -> (
        SoaSlicesMut<'a, KeyValuePair<K, V>>,
        &'a mut [SparseItem<K>],
    ) {
        let Self { dense, sparse } = self;
        (dense, sparse)
    }

    #[inline]
    pub fn swap(&mut self, first_key: K, second_key: K) {
        let Self { dense, sparse } = self;

        let (context, slices) = dense.as_mut_slices_with_context();
        let KeyValueSlicesMut { values, .. } = slices;
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
        soa_swap::<V>(context, first_value, second_value);
    }

    #[inline]
    pub fn swap_keys(&mut self, first_key: K, second_key: K) {
        let Self { dense, sparse } = self;
        let KeyValueSlicesMut { keys, .. } = dense.as_mut_slices();

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

        let KeyValueSlicesMut { keys, .. } = dense.as_mut_slices();
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

        let KeyValueSlicesMut { keys, .. } = dense.as_mut_slices();
        let dense_key: &mut K = unwrap_dense(keys, dense_index);
        check_compatible_key(key, *dense_key);

        *dense_key = key;
        Some(*dense_key)
    }

    #[inline]
    pub fn sort(&mut self)
    where
        for<'any> V::Refs<'any>: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_cached_key(|&key| {
                let sparse_index = key.sparse_index();
                unwrap_dense_from_sparse_index::<K, _>(sparse_index, values.clone(), sparse)
            })
        });
    }

    #[inline]
    pub fn sort_keys(&mut self) {
        self.sort_impl(|keys, _, _| keys.sort());
    }

    #[inline]
    pub fn sort_by<F>(&mut self, mut f: F)
    where
        F: FnMut((K, V::Refs<'_>), (K, V::Refs<'_>)) -> cmp::Ordering,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by(|&lhs_key, &rhs_key| {
                let lhs_index = lhs_key.sparse_index();
                let lhs_value =
                    unwrap_dense_from_sparse_index::<K, _>(lhs_index, values.clone(), sparse);
                let lhs = (lhs_key, lhs_value);

                let rhs_index = rhs_key.sparse_index();
                let rhs_value =
                    unwrap_dense_from_sparse_index::<K, _>(rhs_index, values.clone(), sparse);
                let rhs = (rhs_key, rhs_value);

                f(lhs, rhs)
            })
        });
    }

    #[inline]
    pub fn sort_by_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, V::Refs<'_>)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_key(|&key| {
                let sparse_index = key.sparse_index();
                let value =
                    unwrap_dense_from_sparse_index::<K, _>(sparse_index, values.clone(), sparse);
                f((key, value))
            })
        });
    }

    #[inline]
    pub fn sort_by_cached_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, V::Refs<'_>)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_cached_key(|&key| {
                let sparse_index = key.sparse_index();
                let value =
                    unwrap_dense_from_sparse_index::<K, _>(sparse_index, values.clone(), sparse);
                f((key, value))
            })
        });
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        for<'any> V::Refs<'any>: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by_key(|&key| {
                let sparse_index = key.sparse_index();
                unwrap_dense_from_sparse_index::<K, _>(sparse_index, values.clone(), sparse)
            })
        });
    }

    #[inline]
    pub fn sort_keys_unstable(&mut self) {
        self.sort_impl(|keys, _, _| keys.sort_unstable());
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, mut f: F)
    where
        F: FnMut((K, V::Refs<'_>), (K, V::Refs<'_>)) -> cmp::Ordering,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by(|&lhs_key, &rhs_key| {
                let lhs_index = lhs_key.sparse_index();
                let lhs_value =
                    unwrap_dense_from_sparse_index::<K, _>(lhs_index, values.clone(), sparse);
                let lhs = (lhs_key, lhs_value);

                let rhs_index = rhs_key.sparse_index();
                let rhs_value =
                    unwrap_dense_from_sparse_index::<K, _>(rhs_index, values.clone(), sparse);
                let rhs = (rhs_key, rhs_value);

                f(lhs, rhs)
            })
        });
    }

    #[inline]
    pub fn sort_unstable_by_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, V::Refs<'_>)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by_key(|&key| {
                let sparse_index = key.sparse_index();
                let value =
                    unwrap_dense_from_sparse_index::<K, _>(sparse_index, values.clone(), sparse);
                f((key, value))
            })
        });
    }

    // Implementation was borrowed from the links below:
    // https://skypjack.github.io/2019-09-25-ecs-baf-part-5/#:~:text=Mixing%20in%2Dplace%20sorting%20and%20permutations
    // https://github.com/skypjack/entt/blob/8b0ef2b94234def2053c9a8a2591f4a5e87cf0ea/src/entt/entity/sparse_set.hpp#L964
    fn sort_impl<SortKeys>(&mut self, sort_keys: SortKeys)
    where
        SortKeys: FnOnce(&mut [K], SoaIter<V>, &[SparseItem<K>]),
    {
        let Self { dense, sparse } = self;

        let (context, slices) = dense.as_mut_slices_with_context();
        let KeyValueSlicesMut { keys, values } = slices;
        let mut values = SoaSlicesMut::new(context, values);

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
    pub fn get(&self, key: K) -> Option<V::Refs<'_>> {
        let (_, refs) = self.get_with_context(key);
        refs
    }

    #[inline]
    pub fn get_with_context(&self, key: K) -> (&V::Context, Option<V::Refs<'_>>) {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.iter_with_context();
        (context, sparse_get(dense, sparse, key))
    }

    #[inline]
    pub fn into_get(self, key: K) -> Option<V::Refs<'a>> {
        let (_, refs) = self.into_get_with_context(key);
        refs
    }

    #[inline]
    pub fn into_get_with_context(self, key: K) -> (&'a V::Context, Option<V::Refs<'a>>) {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.into_iter_with_context();
        (context, sparse_get(dense, sparse, key))
    }

    #[inline]
    pub fn get_mut(&mut self, key: K) -> Option<V::RefsMut<'_>> {
        let (_, refs) = self.get_mut_with_context(key);
        refs
    }

    #[inline]
    pub fn get_mut_with_context(&mut self, key: K) -> (&V::Context, Option<V::RefsMut<'_>>) {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.iter_mut_with_context();
        (context, sparse_get_mut(dense, sparse, key))
    }

    #[inline]
    pub fn into_get_mut(self, key: K) -> Option<V::RefsMut<'a>> {
        let (_, refs) = self.into_get_mut_with_context(key);
        refs
    }

    #[inline]
    pub fn into_get_mut_with_context(self, key: K) -> (&'a V::Context, Option<V::RefsMut<'a>>) {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.into_iter_mut_with_context();
        (context, sparse_get_mut(dense, sparse, key))
    }

    #[inline]
    pub fn get_with_key(&self, sparse_index: K::SparseIndex) -> Option<(K, V::Refs<'_>)> {
        let Self { dense, sparse } = self;
        sparse_get_with_key(dense, sparse, sparse_index)
    }

    #[inline]
    pub fn into_get_with_key(self, sparse_index: K::SparseIndex) -> Option<(K, V::Refs<'a>)> {
        let Self { dense, sparse } = self;
        let (_, dense) = dense.into_iter_with_context();
        sparse_get_with_key(dense, sparse, sparse_index)
    }

    #[inline]
    pub fn get_mut_with_key(
        &mut self,
        sparse_index: K::SparseIndex,
    ) -> Option<(K, V::RefsMut<'_>)> {
        let Self { dense, sparse } = self;
        sparse_get_mut_with_key(dense, sparse, sparse_index)
    }

    #[inline]
    pub fn into_get_mut_with_key(
        self,
        sparse_index: K::SparseIndex,
    ) -> Option<(K, V::RefsMut<'a>)> {
        let Self { dense, sparse } = self;
        sparse_get_mut_with_key(dense, sparse, sparse_index)
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: K::SparseIndex) -> Option<K::Epoch> {
        let Self { dense, sparse } = self;

        let KeyValueSlices { keys, .. } = dense.as_slices();
        sparse_get_epoch(keys, sparse, sparse_index)
    }

    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let Self { dense, sparse } = self;

        let KeyValueSlices { keys, .. } = dense.as_slices();
        sparse_contains_key(keys, sparse, key)
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        let Self { dense, .. } = self;
        let inner = dense.iter();
        Keys::new(inner)
    }

    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        let Self { dense, .. } = self;
        let inner = dense.iter();
        Values::new(inner)
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        let Self { dense, .. } = self;
        let inner = dense.iter_mut();
        ValuesMut::new(inner)
    }

    #[inline]
    pub fn into_values_mut(self) -> ValuesMut<'a, K, V> {
        let Self { dense, .. } = self;
        let inner = dense.into_iter();
        ValuesMut::new(inner)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        let Self { dense, .. } = self;
        let inner = dense.iter();
        Iter::new(inner)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        let Self { dense, .. } = self;
        let inner = dense.iter_mut();
        IterMut::new(inner)
    }

    #[inline]
    #[track_caller]
    pub fn index(&self, key: K) -> V::Refs<'_>
    where
        K: Debug,
    {
        let (_, refs) = self.index_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_with_context(&self, key: K) -> (&V::Context, V::Refs<'_>)
    where
        K: Debug,
    {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.iter_with_context();
        (context, sparse_index(dense, sparse, key))
    }

    #[inline]
    #[track_caller]
    pub fn into_index(self, key: K) -> V::Refs<'a>
    where
        K: Debug,
    {
        let (_, refs) = self.into_index_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_with_context(self, key: K) -> (&'a V::Context, V::Refs<'a>)
    where
        K: Debug,
    {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.into_iter_with_context();
        (context, sparse_index(dense, sparse, key))
    }

    #[inline]
    #[track_caller]
    pub fn index_mut(&mut self, key: K) -> V::RefsMut<'_>
    where
        K: Debug,
    {
        let (_, refs) = self.index_mut_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn index_mut_with_context(&mut self, key: K) -> (&V::Context, V::RefsMut<'_>)
    where
        K: Debug,
    {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.iter_mut_with_context();
        (context, sparse_index_mut(dense, sparse, key))
    }

    #[inline]
    pub fn into_index_mut(self, key: K) -> V::RefsMut<'a>
    where
        K: Debug,
    {
        let (_, refs) = self.into_index_mut_with_context(key);
        refs
    }

    #[inline]
    #[track_caller]
    pub fn into_index_mut_with_context(self, key: K) -> (&'a V::Context, V::RefsMut<'a>)
    where
        K: Debug,
    {
        let Self { dense, sparse } = self;
        let (context, dense) = dense.into_iter_mut_with_context();
        (context, sparse_index_mut(dense, sparse, key))
    }
}

impl<'a, K, V> Debug for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    V: Soa,
    SparseItem<K>: Debug,
    SoaSlicesMut<'a, KeyValuePair<K, V>>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { dense, sparse } = self;
        f.debug_struct("EpochSparseViewMut")
            .field("dense", dense)
            .field("sparse", sparse)
            .finish()
    }
}

impl<'a, K, V> From<&'a V::Context> for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    V: Soa,
{
    fn from(context: &'a V::Context) -> Self {
        Self {
            dense: context.into(),
            sparse: Default::default(),
        }
    }
}

impl<'a, K, V> PartialEq for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    V: Soa,
    SoaSlicesMut<'a, KeyValuePair<K, V>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { dense, sparse } = self;
        *dense == other.dense && *sparse == other.sparse
    }
}

impl<'a, K, V> Eq for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    V: Soa,
    SoaSlicesMut<'a, KeyValuePair<K, V>>: Eq,
{
}

impl<'a, K, V> PartialOrd for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    V: Soa,
    SoaSlicesMut<'a, KeyValuePair<K, V>>: PartialOrd,
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

impl<'a, K, V> Ord for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    V: Soa,
    SoaSlicesMut<'a, KeyValuePair<K, V>>: Ord,
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

impl<'a, K, V> Hash for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    V: Soa,
    SparseItem<K>: Hash,
    SoaSlicesMut<'a, KeyValuePair<K, V>>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { dense, sparse } = self;
        dense.hash(state);
        sparse.hash(state);
    }
}

impl<T, K, V> Index<K> for EpochSparseViewMut<'_, K, V>
where
    K: Key + Debug,
    for<'a> V: Soa<Refs<'a> = &'a T> + 'a,
{
    type Output = T;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        EpochSparseViewMut::index(self, key)
    }
}

impl<T, K, V> IndexMut<K> for EpochSparseViewMut<'_, K, V>
where
    K: Key + Debug,
    for<'a> V: Soa<Refs<'a> = &'a T, RefsMut<'a> = &'a mut T> + 'a,
{
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        self.index_mut(key)
    }
}

impl<T, K, V> AsRef<[T]> for EpochSparseViewMut<'_, K, V>
where
    K: Key,
    for<'a> V: Soa<Slices<'a> = &'a [T]> + 'a,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slices()
    }
}

impl<T, K, V> AsMut<[T]> for EpochSparseViewMut<'_, K, V>
where
    K: Key,
    for<'a> V: Soa<SlicesMut<'a> = &'a mut [T]> + 'a,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slices()
    }
}

impl<'a, K, V> AsRef<EpochSparseViewMut<'a, K, V>> for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    V: Soa,
{
    #[inline]
    fn as_ref(&self) -> &EpochSparseViewMut<'a, K, V> {
        self
    }
}

impl<'a, K, V> AsMut<EpochSparseViewMut<'a, K, V>> for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    V: Soa,
{
    #[inline]
    fn as_mut(&mut self) -> &mut EpochSparseViewMut<'a, K, V> {
        self
    }
}

impl<'a, K, V> IntoIterator for &'a EpochSparseViewMut<'_, K, V>
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

impl<'a, K, V> IntoIterator for &'a mut EpochSparseViewMut<'_, K, V>
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

impl<'a, K, V> IntoIterator for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    V: Soa,
{
    type Item = (&'a K, V::RefsMut<'a>);

    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { dense, .. } = self;
        let inner = dense.into_iter();
        IterMut::new(inner)
    }
}

impl<'a, K, V> From<EpochSparseViewMut<'a, K, V>> for EpochSparseView<'a, K, V>
where
    K: Key,
    V: Soa,
{
    #[inline]
    #[allow(unsafe_code)]
    fn from(value: EpochSparseViewMut<'a, K, V>) -> Self {
        let EpochSparseViewMut { dense, sparse } = value;
        unsafe { Self::new_unchecked(dense.into(), sparse) }
    }
}
