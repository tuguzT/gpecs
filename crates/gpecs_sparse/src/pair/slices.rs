use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    pair::{DensePtrs, DenseSlicePtrs},
    soa::{traits::Soa, wrapper},
};

pub struct DenseSlices<'context, 'a, K, V>
where
    V: Soa + ?Sized + 'a,
{
    keys: &'a [K],
    values: wrapper::Slices<'context, 'a, V>,
}

impl<'context, 'a, K, V> DenseSlices<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    #[track_caller]
    pub fn new(
        context: &'context V::Context,
        keys: &'a [K],
        values: V::Slices<'context, 'a>,
    ) -> Self {
        let keys_len = keys.len();
        let values_len = V::slices_len(context, &values);
        assert_eq!(keys_len, values_len);

        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn new_unchecked(keys: &'a [K], values: V::Slices<'context, 'a>) -> Self {
        let values = wrapper::Slices::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        V::slices_len(context, values.as_inner())
    }

    #[inline]
    pub fn into_parts(self) -> (&'a [K], V::Slices<'context, 'a>) {
        let Self { keys, values } = self;
        (keys, values.into_inner())
    }

    #[inline]
    pub fn into_slice_ptrs(self, context: &'context V::Context) -> DenseSlicePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let keys = ptr::from_ref(keys);
        let values = V::slices_as_slice_ptrs(context, values.into_inner());
        unsafe { DenseSlicePtrs::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> DensePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.as_ptr();
        let value = V::slices_as_ptrs(context, values.into_inner());
        DensePtrs::new(key, value)
    }
}

impl<'context, 'a, K, V> From<DenseSlices<'context, 'a, K, V>>
    for (&'a [K], V::Slices<'context, 'a>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: DenseSlices<'context, 'a, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for DenseSlices<'_, '_, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    for<'c, 'a> V::Slices<'c, 'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;
        f.debug_struct("DenseSlices")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> Default for DenseSlices<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'a> V::Slices<'c, 'a>: Default,
{
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = V::Slices::default();
        unsafe { Self::new_unchecked(keys, values) }
    }
}

impl<K, V> PartialEq for DenseSlices<'_, '_, K, V>
where
    K: PartialEq,
    V: Soa + ?Sized,
    for<'c, 'a> V::Slices<'c, 'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<K, V> Eq for DenseSlices<'_, '_, K, V>
where
    K: Eq,
    V: Soa + ?Sized,
    for<'c, 'a> V::Slices<'c, 'a>: Eq,
{
}

impl<K, V> PartialOrd for DenseSlices<'_, '_, K, V>
where
    K: PartialOrd,
    V: Soa + ?Sized,
    for<'c, 'a> V::Slices<'c, 'a>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { keys, values } = self;
        match keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        values.partial_cmp(&other.values)
    }
}

impl<K, V> Ord for DenseSlices<'_, '_, K, V>
where
    K: Ord,
    V: Soa + ?Sized,
    for<'c, 'a> V::Slices<'c, 'a>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { keys, values } = self;
        match keys.cmp(&other.keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        values.cmp(&other.values)
    }
}

impl<K, V> Hash for DenseSlices<'_, '_, K, V>
where
    K: Hash,
    V: Soa + ?Sized,
    for<'c, 'a> V::Slices<'c, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}

impl<K, V> Clone for DenseSlices<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'a> V::Slices<'c, 'a>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { keys, ref values } = *self;
        let values = values.clone();
        Self { keys, values }
    }
}

impl<K, V> Copy for DenseSlices<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'a> V::Slices<'c, 'a>: Copy,
{
}
