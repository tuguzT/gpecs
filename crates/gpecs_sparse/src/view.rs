use core::{
    cmp,
    fmt::Display,
    ops::{Index, IndexMut},
};

use crate::{
    algo::{
        sparse_contains_key, sparse_get, sparse_get_epoch, sparse_get_mut, sparse_get_mut_with_key,
        sparse_get_with_key, sparse_index, sparse_index_mut, sparse_swap, sparse_swap_keys,
    },
    assert::{
        check_equal_key, check_kv_same_len, unwrap_dense_index, unwrap_dense_index_mut,
        unwrap_dense_key, unwrap_dense_key_mut, unwrap_sparse_item, unwrap_sparse_items_pair_mut,
        unwrap_value_from_sparse_index,
    },
    item::SparseItem,
    iter::{Iter, IterMut, Keys, Values, ValuesMut},
    key::{Epoch, Key},
};

pub type SparseView<'a, T> = EpochSparseView<'a, usize, T>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EpochSparseView<'a, K, V>
where
    K: Key,
{
    dense_keys: &'a [K],
    dense_values: &'a [V],
    sparse: &'a [SparseItem<K::Epoch>],
}

impl<'a, K, V> EpochSparseView<'a, K, V>
where
    K: Key,
{
    #[inline]
    pub(crate) const fn new(
        dense_keys: &'a [K],
        dense_values: &'a [V],
        sparse: &'a [SparseItem<K::Epoch>],
    ) -> Self {
        check_kv_same_len(dense_keys.len(), dense_values.len());
        Self {
            dense_keys,
            dense_values,
            sparse,
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        let Self { dense_keys, .. } = self;
        dense_keys.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub const fn sparse_len(&self) -> usize {
        let Self { sparse, .. } = self;
        sparse.len()
    }

    #[inline]
    pub const fn sparse_is_empty(&self) -> bool {
        self.sparse_len() == 0
    }

    #[inline]
    pub const fn as_slice(&self) -> &'a [V] {
        let Self { dense_values, .. } = self;
        dense_values
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const V {
        let Self { dense_values, .. } = self;
        dense_values.as_ptr()
    }

    #[inline]
    pub const fn as_keys_slice(&self) -> &'a [K] {
        let Self { dense_keys, .. } = self;
        dense_keys
    }

    #[inline]
    pub const fn as_keys_ptr(&self) -> *const K {
        let Self { dense_keys, .. } = self;
        dense_keys.as_ptr()
    }

    #[inline]
    pub const fn as_sparse_slice(&self) -> &'a [SparseItem<K::Epoch>] {
        let Self { sparse, .. } = self;
        sparse
    }

    #[inline]
    pub const fn as_sparse_ptr(&self) -> *const SparseItem<K::Epoch> {
        let Self { sparse, .. } = self;
        sparse.as_ptr()
    }

    #[inline]
    pub fn get(&self, key: K) -> Option<&'a V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_get(dense_keys, dense_values, sparse, key)
    }

    #[inline]
    pub fn get_with_key(&self, sparse_index: usize) -> Option<(K, &'a V)> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_get_with_key(dense_keys, dense_values, sparse, sparse_index)
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: usize) -> Option<K::Epoch> {
        let Self {
            sparse, dense_keys, ..
        } = self;

        sparse_get_epoch(dense_keys, sparse, sparse_index)
    }

    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let Self {
            dense_keys, sparse, ..
        } = self;

        sparse_contains_key(dense_keys, sparse, key)
    }

    #[inline]
    pub fn keys(&self) -> Keys<'a, K, V> {
        let Self { dense_keys, .. } = self;

        let keys = dense_keys.iter();
        Keys::new(keys)
    }

    #[inline]
    pub fn values(&self) -> Values<'a, K, V> {
        let Self { dense_values, .. } = self;

        let values = dense_values.iter();
        Values::new(values)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'a, K, V> {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        let keys = dense_keys.iter();
        let values = dense_values.iter();
        Iter::new(keys, values)
    }

    #[inline]
    pub fn into_index(self, key: K) -> &'a V
    where
        K: Display,
    {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_index(dense_keys, dense_values, sparse, key)
    }
}

impl<'a, K, V> Default for EpochSparseView<'a, K, V>
where
    K: Key,
{
    #[inline]
    fn default() -> Self {
        Self {
            dense_keys: Default::default(),
            dense_values: Default::default(),
            sparse: Default::default(),
        }
    }
}

impl<'a, K, V> Clone for EpochSparseView<'a, K, V>
where
    K: Key,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, K, V> Copy for EpochSparseView<'a, K, V> where K: Key {}

impl<'a, K, V> Index<K> for EpochSparseView<'a, K, V>
where
    K: Key + Display,
{
    type Output = V;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_index(dense_keys, dense_values, sparse, key)
    }
}

impl<'a, K, V> AsRef<[V]> for EpochSparseView<'a, K, V>
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EpochSparseViewMut<'a, K, V>
where
    K: Key,
{
    dense_keys: &'a mut [K],
    dense_values: &'a mut [V],
    sparse: &'a mut [SparseItem<K::Epoch>],
}

impl<'a, K, V> EpochSparseViewMut<'a, K, V>
where
    K: Key,
{
    #[inline]
    pub(crate) fn new(
        dense_keys: &'a mut [K],
        dense_values: &'a mut [V],
        sparse: &'a mut [SparseItem<K::Epoch>],
    ) -> Self {
        check_kv_same_len(dense_keys.len(), dense_values.len());
        Self {
            dense_keys,
            dense_values,
            sparse,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { dense_keys, .. } = self;
        dense_keys.len()
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
        let Self { dense_values, .. } = self;
        dense_values
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [V] {
        let Self { dense_values, .. } = self;
        dense_values
    }

    #[inline]
    pub fn into_slice(self) -> &'a mut [V] {
        let Self { dense_values, .. } = self;
        dense_values
    }

    #[inline]
    pub fn as_ptr(&self) -> *const V {
        let Self { dense_values, .. } = self;
        dense_values.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut V {
        let Self { dense_values, .. } = self;
        dense_values.as_mut_ptr()
    }

    #[inline]
    pub fn as_keys_slice(&self) -> &[K] {
        let Self { dense_keys, .. } = self;
        dense_keys
    }

    #[inline]
    pub fn into_keys_slice(self) -> &'a [K] {
        let Self { dense_keys, .. } = self;
        dense_keys
    }

    #[inline]
    pub fn as_keys_ptr(&self) -> *const K {
        let Self { dense_keys, .. } = self;
        dense_keys.as_ptr()
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
    #[allow(clippy::type_complexity)]
    pub fn into_parts(self) -> (&'a [K], &'a mut [V], &'a [SparseItem<K::Epoch>]) {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        (dense_keys, dense_values, sparse)
    }

    #[inline]
    pub fn swap(&mut self, first_key: K, second_key: K) {
        let Self {
            dense_values,
            sparse,
            ..
        } = self;

        sparse_swap(dense_values, sparse, first_key, second_key)
    }

    #[inline]
    pub fn swap_keys(&mut self, first_key: K, second_key: K) {
        let Self {
            dense_keys, sparse, ..
        } = self;

        sparse_swap_keys(dense_keys, sparse, first_key, second_key)
    }

    pub fn invalidate_epoch(&mut self, key: K) -> Option<K> {
        let Self {
            dense_keys, sparse, ..
        } = self;

        let sparse_index = key.sparse_index();
        let sparse_item = sparse
            .get_mut(sparse_index)
            .take_if(|item| item.epoch == key.epoch())?;
        let dense_index = sparse_item.dense_index()?;

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
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

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
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_get(dense_keys, dense_values, sparse, key)
    }

    #[inline]
    pub fn into_get(self, key: K) -> Option<&'a V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_get(dense_keys, dense_values, sparse, key)
    }

    #[inline]
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_get_mut(dense_keys, dense_values, sparse, key)
    }

    #[inline]
    pub fn into_get_mut(self, key: K) -> Option<&'a mut V> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_get_mut(dense_keys, dense_values, sparse, key)
    }

    #[inline]
    pub fn get_with_key(&self, sparse_index: usize) -> Option<(K, &V)> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_get_with_key(dense_keys, dense_values, sparse, sparse_index)
    }

    #[inline]
    pub fn get_mut_with_key(&mut self, sparse_index: usize) -> Option<(K, &mut V)> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_get_mut_with_key(dense_keys, dense_values, sparse, sparse_index)
    }

    #[inline]
    pub fn into_get_mut_with_key(self, sparse_index: usize) -> Option<(K, &'a mut V)> {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_get_mut_with_key(dense_keys, dense_values, sparse, sparse_index)
    }

    #[inline]
    pub fn get_epoch(&self, sparse_index: usize) -> Option<K::Epoch> {
        let Self {
            sparse, dense_keys, ..
        } = self;

        sparse_get_epoch(dense_keys, sparse, sparse_index)
    }

    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let Self {
            dense_keys, sparse, ..
        } = self;

        sparse_contains_key(dense_keys, sparse, key)
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        let Self { dense_keys, .. } = self;

        let keys = dense_keys.iter();
        Keys::new(keys)
    }

    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        let Self { dense_values, .. } = self;

        let values = dense_values.iter();
        Values::new(values)
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        let Self { dense_values, .. } = self;

        let values = dense_values.iter_mut();
        ValuesMut::new(values)
    }

    #[inline]
    pub fn into_values_mut(self) -> ValuesMut<'a, K, V> {
        let Self { dense_values, .. } = self;

        let values = dense_values.iter_mut();
        ValuesMut::new(values)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        let keys = dense_keys.iter();
        let values = dense_values.iter();
        Iter::new(keys, values)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        let keys = dense_keys.iter();
        let values = dense_values.iter_mut();
        IterMut::new(keys, values)
    }

    #[inline]
    pub fn into_index(self, key: K) -> &'a V
    where
        K: Display,
    {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_index(dense_keys, dense_values, sparse, key)
    }

    #[inline]
    pub fn into_index_mut(self, key: K) -> &'a mut V
    where
        K: Display,
    {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_index_mut(dense_keys, dense_values, sparse, key)
    }
}

impl<'a, K, V> Default for EpochSparseViewMut<'a, K, V>
where
    K: Key,
{
    #[inline]
    fn default() -> Self {
        Self {
            dense_keys: Default::default(),
            dense_values: Default::default(),
            sparse: Default::default(),
        }
    }
}

impl<'a, K, V> Index<K> for EpochSparseViewMut<'a, K, V>
where
    K: Key + Display,
{
    type Output = V;

    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_index(dense_keys, dense_values, sparse, key)
    }
}

impl<'a, K, V> IndexMut<K> for EpochSparseViewMut<'a, K, V>
where
    K: Key + Display,
{
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        let Self {
            dense_keys,
            dense_values,
            sparse,
        } = self;

        sparse_index_mut(dense_keys, dense_values, sparse, key)
    }
}

impl<'a, K, V> AsRef<[V]> for EpochSparseViewMut<'a, K, V>
where
    K: Key,
{
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_slice()
    }
}

impl<'a, K, V> AsMut<[V]> for EpochSparseViewMut<'a, K, V>
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
        let Self {
            dense_keys,
            dense_values,
            ..
        } = self;

        let keys = dense_keys.iter();
        let values = dense_values.iter_mut();
        IterMut::new(keys, values)
    }
}

impl<'a, K, V> From<EpochSparseViewMut<'a, K, V>> for EpochSparseView<'a, K, V>
where
    K: Key,
{
    #[inline]
    fn from(value: EpochSparseViewMut<'a, K, V>) -> Self {
        let (dense_keys, dense_values, sparse) = value.into_parts();
        Self::new(dense_keys, dense_values, sparse)
    }
}
