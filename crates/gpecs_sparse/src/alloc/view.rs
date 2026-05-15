use core::cmp;

use crate::{
    assert::unwrap_dense_from_sparse_index,
    item::SparseItem,
    key::Key,
    soa::traits::{Refs, SoaOwned},
    view::EpochSparseViewMut,
};

impl<K, V, S> EpochSparseViewMut<'_, '_, K, V, S>
where
    K: Key,
    V: SoaOwned + ?Sized,
    S: SparseItem<Index = K::SparseIndex, Epoch = K::Epoch>,
{
    #[inline]
    pub fn sort(&mut self)
    where
        for<'ctx, 'a> Refs<'ctx, 'a, V>: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_cached_key(|&key| {
                let sparse_index = key.sparse_index();
                let dense = values.clone();
                unwrap_dense_from_sparse_index::<K, _>(sparse_index, dense, sparse)
            });
        });
    }

    #[inline]
    pub fn sort_keys(&mut self) {
        self.sort_impl(|keys, _, _| keys.sort());
    }

    #[inline]
    pub fn sort_by<F>(&mut self, mut f: F)
    where
        for<'a> F: FnMut((K, Refs<'_, 'a, V>), (K, Refs<'_, 'a, V>)) -> cmp::Ordering,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by(|&lhs_key, &rhs_key| {
                let dense = values.clone();
                let lhs_index = lhs_key.sparse_index();
                let lhs_value = unwrap_dense_from_sparse_index::<K, _>(lhs_index, dense, sparse);
                let lhs = (lhs_key, lhs_value);

                let dense = values.clone();
                let rhs_index = rhs_key.sparse_index();
                let rhs_value = unwrap_dense_from_sparse_index::<K, _>(rhs_index, dense, sparse);
                let rhs = (rhs_key, rhs_value);

                f(lhs, rhs)
            });
        });
    }

    #[inline]
    pub fn sort_by_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, Refs<'_, '_, V>)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_key(|&key| {
                let sparse_index = key.sparse_index();
                let dense = values.clone();
                let value = unwrap_dense_from_sparse_index::<K, _>(sparse_index, dense, sparse);
                f((key, value))
            });
        });
    }

    #[inline]
    pub fn sort_by_cached_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, Refs<'_, '_, V>)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_cached_key(|&key| {
                let sparse_index = key.sparse_index();
                let dense = values.clone();
                let value = unwrap_dense_from_sparse_index::<K, _>(sparse_index, dense, sparse);
                f((key, value))
            });
        });
    }
}
