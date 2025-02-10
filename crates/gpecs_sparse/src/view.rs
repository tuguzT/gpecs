use core::{
    cmp,
    fmt::{self, Debug, Display},
    hash::{self, Hash},
    ops::{Index, IndexMut},
};

use gpecs_soa::slice::SoaSlice;

use crate::{
    algo::{
        sparse_contains_key, sparse_get, sparse_get_epoch, sparse_get_mut, sparse_get_mut_with_key,
        sparse_get_with_key, sparse_index, sparse_index_mut, sparse_swap, sparse_swap_keys,
    },
    assert::{
        check_equal_key, unwrap_dense_index, unwrap_dense_index_mut, unwrap_dense_key,
        unwrap_dense_key_mut, unwrap_sparse_item, unwrap_sparse_items_pair_mut,
        unwrap_value_from_sparse_index,
    },
    item::SparseItem,
    iter::{Iter, IterMut, Keys, Values, ValuesMut},
    key::{Epoch, Key},
};

pub type SparseView<'a, T> = EpochSparseView<'a, usize, T>;

pub struct EpochSparseView<'a, K, V>
where
    K: Key,
{
    dense: &'a SoaSlice<(K, V)>,
    sparse: &'a [SparseItem<K::Epoch>],
}

impl<'a, K, V> EpochSparseView<'a, K, V>
where
    K: Key,
{
    #[inline]
    pub(crate) fn new(dense: &'a SoaSlice<(K, V)>, sparse: &'a [SparseItem<K::Epoch>]) -> Self {
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
    pub fn as_slice(&self) -> &'a [V] {
        let Self { dense, .. } = self;

        let (_, values) = dense.as_slices();
        values
    }

    #[inline]
    pub fn as_ptr(&self) -> *const V {
        let Self { dense, .. } = self;

        let (_, values) = dense.as_ptrs();
        values
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &'a [K] {
        let Self { dense, .. } = self;

        let (keys, _) = dense.as_slices();
        keys
    }

    #[inline]
    pub fn as_keys_ptr(&self) -> *const K {
        let Self { dense, .. } = self;

        let (keys, _) = dense.as_ptrs();
        keys
    }

    #[inline]
    pub fn as_sparse_slice(&self) -> &'a [SparseItem<K::Epoch>] {
        let Self { sparse, .. } = self;
        sparse
    }

    #[inline]
    pub fn as_sparse_ptr(&self) -> *const SparseItem<K::Epoch> {
        let Self { sparse, .. } = self;
        sparse.as_ptr()
    }

    #[inline]
    pub fn get(&self, key: K) -> Option<&'a V> {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_slices();
        sparse_get(dense_keys, dense_values, sparse, key)
    }

    #[inline]
    pub fn get_with_key(&self, sparse_index: usize) -> Option<(K, &'a V)> {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_slices();
        sparse_get_with_key(dense_keys, dense_values, sparse, sparse_index)
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: usize) -> Option<K::Epoch> {
        let Self { dense, sparse } = self;

        let (dense_keys, _) = dense.as_slices();
        sparse_get_epoch(dense_keys, sparse, sparse_index)
    }

    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let Self { dense, sparse } = self;

        let (dense_keys, _) = dense.as_slices();
        sparse_contains_key(dense_keys, sparse, key)
    }

    #[inline]
    pub fn keys(&self) -> Keys<'a, K, V> {
        let Self { dense, .. } = self;
        Keys::new(dense.iter())
    }

    #[inline]
    pub fn values(&self) -> Values<'a, K, V> {
        let Self { dense, .. } = self;
        Values::new(dense.iter())
    }

    #[inline]
    pub fn iter(&self) -> Iter<'a, K, V> {
        let Self { dense, .. } = self;
        Iter::new(dense.iter())
    }

    #[inline]
    pub fn into_index(self, key: K) -> &'a V
    where
        K: Display,
    {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_slices();
        sparse_index(dense_keys, dense_values, sparse, key)
    }
}

impl<'a, K, V> Debug for EpochSparseView<'a, K, V>
where
    K: Key,
    K::Epoch: Debug,
    SoaSlice<(K, V)>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EpochSparseView")
            .field("dense", &self.dense)
            .field("sparse", &self.sparse)
            .finish()
    }
}

impl<K, V> Default for EpochSparseView<'_, K, V>
where
    K: Key,
{
    #[inline]
    fn default() -> Self {
        Self {
            dense: Default::default(),
            sparse: Default::default(),
        }
    }
}

impl<K, V> Clone for EpochSparseView<'_, K, V>
where
    K: Key,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V> Copy for EpochSparseView<'_, K, V> where K: Key {}

impl<'a, K, V> PartialEq for EpochSparseView<'a, K, V>
where
    K: Key,
    SoaSlice<(K, V)>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.dense == other.dense && self.sparse == other.sparse
    }
}

impl<'a, K, V> Eq for EpochSparseView<'a, K, V>
where
    K: Key,
    SoaSlice<(K, V)>: Eq,
{
}

impl<'a, K, V> PartialOrd for EpochSparseView<'a, K, V>
where
    K: Key,
    SoaSlice<(K, V)>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.dense.partial_cmp(&other.dense) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.sparse.partial_cmp(&other.sparse)
    }
}

impl<'a, K, V> Ord for EpochSparseView<'a, K, V>
where
    K: Key,
    SoaSlice<(K, V)>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.dense.cmp(&other.dense) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.sparse.cmp(&other.sparse)
    }
}

impl<'a, K, V> Hash for EpochSparseView<'a, K, V>
where
    K: Key,
    K::Epoch: Hash,
    SoaSlice<(K, V)>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.dense.hash(state);
        self.sparse.hash(state);
    }
}

impl<K, V> Index<K> for EpochSparseView<'_, K, V>
where
    K: Key + Display,
{
    type Output = V;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_slices();
        sparse_index(dense_keys, dense_values, sparse, key)
    }
}

impl<K, V> AsRef<[V]> for EpochSparseView<'_, K, V>
where
    K: Key,
{
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_slice()
    }
}

impl<'a, K, V> AsRef<EpochSparseView<'a, K, V>> for EpochSparseView<'a, K, V>
where
    K: Key,
{
    #[inline]
    fn as_ref(&self) -> &EpochSparseView<'a, K, V> {
        self
    }
}

impl<'a, K, V> IntoIterator for &EpochSparseView<'a, K, V>
where
    K: Key,
{
    type Item = (&'a K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for EpochSparseView<'a, K, V>
where
    K: Key,
{
    type Item = (&'a K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub type SparseViewMut<'a, T> = EpochSparseViewMut<'a, usize, T>;

pub struct EpochSparseViewMut<'a, K, V>
where
    K: Key,
{
    dense: &'a mut SoaSlice<(K, V)>,
    sparse: &'a mut [SparseItem<K::Epoch>],
}

impl<'a, K, V> EpochSparseViewMut<'a, K, V>
where
    K: Key,
{
    #[inline]
    pub(crate) fn new(
        dense: &'a mut SoaSlice<(K, V)>,
        sparse: &'a mut [SparseItem<K::Epoch>],
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
    pub fn as_slice(&self) -> &[V] {
        let Self { dense, .. } = self;

        let (_, values) = dense.as_slices();
        values
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [V] {
        let Self { dense, .. } = self;

        let (_, values) = dense.as_mut_slices();
        values
    }

    #[inline]
    pub fn into_slice(self) -> &'a mut [V] {
        let Self { dense, .. } = self;

        let (_, values) = dense.as_mut_slices();
        values
    }

    #[inline]
    pub fn as_ptr(&self) -> *const V {
        let Self { dense, .. } = self;

        let (_, values) = dense.as_ptrs();
        values
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut V {
        let Self { dense, .. } = self;

        let (_, values) = dense.as_mut_ptrs();
        values
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { dense, .. } = self;

        let (keys, _) = dense.as_slices();
        keys
    }

    #[inline]
    pub fn into_keys_slice(self) -> &'a [K] {
        let Self { dense, .. } = self;

        let (keys, _) = dense.as_slices();
        keys
    }

    #[inline]
    pub fn as_keys_ptr(&self) -> *const K {
        let Self { dense, .. } = self;

        let (keys, _) = dense.as_ptrs();
        keys
    }

    #[inline]
    pub fn as_sparse_slice(&self) -> &[SparseItem<K::Epoch>] {
        let Self { sparse, .. } = self;
        sparse
    }

    #[inline]
    pub fn into_sparse_slice(self) -> &'a [SparseItem<K::Epoch>] {
        let Self { sparse, .. } = self;
        sparse
    }

    #[inline]
    pub fn as_sparse_ptr(&self) -> *const SparseItem<K::Epoch> {
        let Self { sparse, .. } = self;
        sparse.as_ptr()
    }

    #[inline]
    pub fn swap(&mut self, first_key: K, second_key: K) {
        let Self { dense, sparse } = self;

        let (_, dense_values) = dense.as_mut_slices();
        sparse_swap(dense_values, sparse, first_key, second_key)
    }

    #[inline]
    pub fn swap_keys(&mut self, first_key: K, second_key: K) {
        let Self { dense, sparse } = self;

        let (dense_keys, _) = dense.as_mut_slices();
        sparse_swap_keys(dense_keys, sparse, first_key, second_key)
    }

    pub fn invalidate_epoch(&mut self, key: K) -> Option<K> {
        let Self { dense, sparse } = self;

        let sparse_index = key.sparse_index();
        let sparse_item = sparse
            .get_mut(sparse_index)
            .take_if(|item| item.epoch == key.epoch())?;
        let dense_index = sparse_item.dense_index()?;

        let (dense_keys, _) = dense.as_mut_slices();
        let dense_key = unwrap_dense_key_mut(dense_keys, dense_index);
        check_equal_key(key, *dense_key);

        sparse_item.epoch = sparse_item.epoch.next();
        *dense_key = K::new(sparse_index, sparse_item.epoch);

        Some(*dense_key)
    }

    #[inline]
    pub fn sort(&mut self)
    where
        V: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_cached_key(|&key| {
                let sparse_index = key.sparse_index();
                unwrap_value_from_sparse_index(sparse_index, values, sparse)
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
        F: FnMut((K, &V), (K, &V)) -> cmp::Ordering,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by(|&lhs_key, &rhs_key| {
                let lhs_index = lhs_key.sparse_index();
                let lhs_value = unwrap_value_from_sparse_index(lhs_index, values, sparse);
                let lhs = (lhs_key, lhs_value);

                let rhs_index = rhs_key.sparse_index();
                let rhs_value = unwrap_value_from_sparse_index(rhs_index, values, sparse);
                let rhs = (rhs_key, rhs_value);

                f(lhs, rhs)
            })
        });
    }

    #[inline]
    pub fn sort_by_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, &V)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_key(|&key| {
                let sparse_index = key.sparse_index();
                let value = unwrap_value_from_sparse_index(sparse_index, values, sparse);
                f((key, value))
            })
        });
    }

    #[inline]
    pub fn sort_by_cached_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, &V)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_cached_key(|&key| {
                let sparse_index = key.sparse_index();
                let value = unwrap_value_from_sparse_index(sparse_index, values, sparse);
                f((key, value))
            })
        });
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        V: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by_key(|&key| {
                let sparse_index = key.sparse_index();
                unwrap_value_from_sparse_index(sparse_index, values, sparse)
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
        F: FnMut((K, &V), (K, &V)) -> cmp::Ordering,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by(|&lhs_key, &rhs_key| {
                let lhs_index = lhs_key.sparse_index();
                let lhs_value = unwrap_value_from_sparse_index(lhs_index, values, sparse);
                let lhs = (lhs_key, lhs_value);

                let rhs_index = rhs_key.sparse_index();
                let rhs_value = unwrap_value_from_sparse_index(rhs_index, values, sparse);
                let rhs = (rhs_key, rhs_value);

                f(lhs, rhs)
            })
        });
    }

    #[inline]
    pub fn sort_unstable_by_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, &V)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_unstable_by_key(|&key| {
                let sparse_index = key.sparse_index();
                let value = unwrap_value_from_sparse_index(sparse_index, values, sparse);
                f((key, value))
            })
        });
    }

    // Implementation was borrowed from the links below:
    // https://skypjack.github.io/2019-09-25-ecs-baf-part-5/#:~:text=Mixing%20in%2Dplace%20sorting%20and%20permutations
    // https://github.com/skypjack/entt/blob/8b0ef2b94234def2053c9a8a2591f4a5e87cf0ea/src/entt/entity/sparse_set.hpp#L964
    fn sort_impl<SortKeys>(&mut self, sort_keys: SortKeys)
    where
        SortKeys: FnOnce(&mut [K], &[V], &[SparseItem<K::Epoch>]),
    {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_mut_slices();
        sort_keys(dense_keys, dense_values, sparse);

        for pos in 0..dense_keys.len() {
            let mut curr = pos;
            let mut next = {
                let sparse_index = unwrap_dense_key(dense_keys, curr).sparse_index();
                let sparse_item = unwrap_sparse_item(sparse, sparse_index);
                unwrap_dense_index(sparse_item.kind())
            };

            while curr != next {
                let (curr_item, next_item) = {
                    let first_index = unwrap_dense_key(dense_keys, curr).sparse_index();
                    let second_index = unwrap_dense_key(dense_keys, next).sparse_index();
                    unwrap_sparse_items_pair_mut(sparse, first_index, second_index)
                };
                let curr_dense_index = unwrap_dense_index_mut(curr_item.kind_mut());
                let next_dense_index = unwrap_dense_index_mut(next_item.kind_mut());

                dense_values.swap(*curr_dense_index, *next_dense_index);

                *curr_dense_index = curr;
                curr = next;
                next = *next_dense_index;
            }
        }
    }

    #[inline]
    pub fn get(&self, key: K) -> Option<&V> {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_slices();
        sparse_get(dense_keys, dense_values, sparse, key)
    }

    #[inline]
    pub fn into_get(self, key: K) -> Option<&'a V> {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_slices();
        sparse_get(dense_keys, dense_values, sparse, key)
    }

    #[inline]
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_mut_slices();
        sparse_get_mut(dense_keys, dense_values, sparse, key)
    }

    #[inline]
    pub fn into_get_mut(self, key: K) -> Option<&'a mut V> {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_mut_slices();
        sparse_get_mut(dense_keys, dense_values, sparse, key)
    }

    #[inline]
    pub fn get_with_key(&self, sparse_index: usize) -> Option<(K, &V)> {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_slices();
        sparse_get_with_key(dense_keys, dense_values, sparse, sparse_index)
    }

    #[inline]
    pub fn get_mut_with_key(&mut self, sparse_index: usize) -> Option<(K, &mut V)> {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_mut_slices();
        sparse_get_mut_with_key(dense_keys, dense_values, sparse, sparse_index)
    }

    #[inline]
    pub fn into_get_mut_with_key(self, sparse_index: usize) -> Option<(K, &'a mut V)> {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_mut_slices();
        sparse_get_mut_with_key(dense_keys, dense_values, sparse, sparse_index)
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: usize) -> Option<K::Epoch> {
        let Self { dense, sparse } = self;

        let (dense_keys, _) = dense.as_slices();
        sparse_get_epoch(dense_keys, sparse, sparse_index)
    }

    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let Self { dense, sparse } = self;

        let (dense_keys, _) = dense.as_slices();
        sparse_contains_key(dense_keys, sparse, key)
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        let Self { dense, .. } = self;
        Keys::new(dense.iter())
    }

    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        let Self { dense, .. } = self;
        Values::new(dense.iter())
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        let Self { dense, .. } = self;
        ValuesMut::new(dense.iter_mut())
    }

    #[inline]
    pub fn into_values_mut(self) -> ValuesMut<'a, K, V> {
        let Self { dense, .. } = self;
        ValuesMut::new(dense.iter_mut())
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        let Self { dense, .. } = self;
        Iter::new(dense.iter())
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        let Self { dense, .. } = self;
        IterMut::new(dense.iter_mut())
    }

    #[inline]
    pub fn into_index(self, key: K) -> &'a V
    where
        K: Display,
    {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_slices();
        sparse_index(dense_keys, dense_values, sparse, key)
    }

    #[inline]
    pub fn into_index_mut(self, key: K) -> &'a mut V
    where
        K: Display,
    {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_mut_slices();
        sparse_index_mut(dense_keys, dense_values, sparse, key)
    }
}

impl<'a, K, V> Debug for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    K::Epoch: Debug,
    SoaSlice<(K, V)>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EpochSparseViewMut")
            .field("dense", &self.dense)
            .field("sparse", &self.sparse)
            .finish()
    }
}

impl<K, V> Default for EpochSparseViewMut<'_, K, V>
where
    K: Key,
{
    #[inline]
    fn default() -> Self {
        Self {
            dense: Default::default(),
            sparse: Default::default(),
        }
    }
}

impl<'a, K, V> PartialEq for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    SoaSlice<(K, V)>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.dense == other.dense && self.sparse == other.sparse
    }
}

impl<'a, K, V> Eq for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    SoaSlice<(K, V)>: Eq,
{
}

impl<'a, K, V> PartialOrd for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    SoaSlice<(K, V)>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.dense.partial_cmp(&other.dense) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.sparse.partial_cmp(&other.sparse)
    }
}

impl<'a, K, V> Ord for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    SoaSlice<(K, V)>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.dense.cmp(&other.dense) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.sparse.cmp(&other.sparse)
    }
}

impl<'a, K, V> Hash for EpochSparseViewMut<'a, K, V>
where
    K: Key,
    K::Epoch: Hash,
    SoaSlice<(K, V)>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.dense.hash(state);
        self.sparse.hash(state);
    }
}

impl<K, V> Index<K> for EpochSparseViewMut<'_, K, V>
where
    K: Key + Display,
{
    type Output = V;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_slices();
        sparse_index(dense_keys, dense_values, sparse, key)
    }
}

impl<K, V> IndexMut<K> for EpochSparseViewMut<'_, K, V>
where
    K: Key + Display,
{
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        let Self { dense, sparse } = self;

        let (dense_keys, dense_values) = dense.as_mut_slices();
        sparse_index_mut(dense_keys, dense_values, sparse, key)
    }
}

impl<K, V> AsRef<[V]> for EpochSparseViewMut<'_, K, V>
where
    K: Key,
{
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_slice()
    }
}

impl<K, V> AsMut<[V]> for EpochSparseViewMut<'_, K, V>
where
    K: Key,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [V] {
        self.as_mut_slice()
    }
}

impl<'a, K, V> AsRef<EpochSparseViewMut<'a, K, V>> for EpochSparseViewMut<'a, K, V>
where
    K: Key,
{
    #[inline]
    fn as_ref(&self) -> &EpochSparseViewMut<'a, K, V> {
        self
    }
}

impl<'a, K, V> AsMut<EpochSparseViewMut<'a, K, V>> for EpochSparseViewMut<'a, K, V>
where
    K: Key,
{
    #[inline]
    fn as_mut(&mut self) -> &mut EpochSparseViewMut<'a, K, V> {
        self
    }
}

impl<'a, K, V> IntoIterator for &'a EpochSparseViewMut<'_, K, V>
where
    K: Key,
{
    type Item = (&'a K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut EpochSparseViewMut<'_, K, V>
where
    K: Key,
{
    type Item = (&'a K, &'a mut V);

    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, K, V> IntoIterator for EpochSparseViewMut<'a, K, V>
where
    K: Key,
{
    type Item = (&'a K, &'a mut V);

    type IntoIter = IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { dense, .. } = self;
        IterMut::new(dense.iter_mut())
    }
}

impl<'a, K, V> From<EpochSparseViewMut<'a, K, V>> for EpochSparseView<'a, K, V>
where
    K: Key,
{
    #[inline]
    fn from(value: EpochSparseViewMut<'a, K, V>) -> Self {
        let EpochSparseViewMut { dense, sparse } = value;
        Self::new(dense, sparse)
    }
}
