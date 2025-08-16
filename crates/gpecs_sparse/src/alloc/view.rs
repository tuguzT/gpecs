use core::cmp;

use crate::{
    assert::unwrap_dense_from_sparse_index, key::Key, soa::traits::Soa, view::EpochSparseViewMut,
};

impl<K, V> EpochSparseViewMut<'_, '_, K, V>
where
    K: Key,
    V: Soa + ?Sized,
{
    #[inline]
    pub fn sort(&mut self)
    where
        for<'ca, 'any> V::Refs<'ca, 'any>: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_cached_key(|&key| {
                let sparse_index = key.sparse_index();
                unwrap_dense_from_sparse_index::<K, _>(sparse_index, values.clone(), sparse)
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
        F: FnMut((K, V::Refs<'_, '_>), (K, V::Refs<'_, '_>)) -> cmp::Ordering,
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
            });
        });
    }

    #[inline]
    pub fn sort_by_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, V::Refs<'_, '_>)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_key(|&key| {
                let sparse_index = key.sparse_index();
                let value =
                    unwrap_dense_from_sparse_index::<K, _>(sparse_index, values.clone(), sparse);
                f((key, value))
            });
        });
    }

    #[inline]
    pub fn sort_by_cached_key<T, F>(&mut self, mut f: F)
    where
        F: FnMut((K, V::Refs<'_, '_>)) -> T,
        T: Ord,
    {
        self.sort_impl(|keys, values, sparse| {
            keys.sort_by_cached_key(|&key| {
                let sparse_index = key.sparse_index();
                let value =
                    unwrap_dense_from_sparse_index::<K, _>(sparse_index, values.clone(), sparse);
                f((key, value))
            });
        });
    }
}
