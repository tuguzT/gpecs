use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    item::DenseSlicePtrs,
    soa::{
        traits::{Slices, Soa, SoaContext},
        wrapper,
    },
};

pub struct DenseSlices<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    keys: &'a [K],
    values: wrapper::Slices<'ctx, 'a, V>,
}

impl<'ctx, 'a, K, V> DenseSlices<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    #[track_caller]
    pub fn new(context: &'ctx V::Context, keys: &'a [K], values: Slices<'ctx, 'a, V>) -> Self {
        let keys_len = keys.len();
        let values_len = context.slices_len(&values);
        assert_eq!(keys_len, values_len);

        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn new_unchecked(keys: &'a [K], values: Slices<'ctx, 'a, V>) -> Self {
        let values = wrapper::Slices::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        context.slices_len(values.as_inner())
    }

    #[inline]
    pub fn into_parts(self) -> (&'a [K], Slices<'ctx, 'a, V>) {
        let Self { keys, values } = self;
        (keys, values.into_inner())
    }

    #[inline]
    pub fn into_slice_ptrs(self, context: &'ctx V::Context) -> DenseSlicePtrs<'ctx, K, V> {
        let Self { keys, values } = self;

        let keys = ptr::from_ref(keys);
        let values = context.slices_as_slice_ptrs(values.into_inner());
        unsafe { DenseSlicePtrs::new_unchecked(keys, values) }
    }
}

impl<'ctx, 'a, K, V> From<DenseSlices<'ctx, 'a, K, V>> for (&'a [K], Slices<'ctx, 'a, V>)
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    fn from(value: DenseSlices<'ctx, 'a, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for DenseSlices<'_, '_, K, V>
where
    K: Debug,
    V: ?Sized,
    for<'a> V: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Debug,
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
    V: ?Sized,
    for<'a> V: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Default,
{
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = Slices::<V>::default();
        unsafe { Self::new_unchecked(keys, values) }
    }
}

impl<K, V> PartialEq for DenseSlices<'_, '_, K, V>
where
    K: PartialEq,
    V: ?Sized,
    for<'a> V: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<K, V> Eq for DenseSlices<'_, '_, K, V>
where
    K: Eq,
    V: ?Sized,
    for<'a> V: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Eq,
{
}

impl<K, V> PartialOrd for DenseSlices<'_, '_, K, V>
where
    K: PartialOrd,
    V: ?Sized,
    for<'a> V: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: PartialOrd,
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
    V: ?Sized,
    for<'a> V: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Ord,
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
    V: ?Sized,
    for<'a> V: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}

impl<K, V> Clone for DenseSlices<'_, '_, K, V>
where
    V: ?Sized,
    for<'a> V: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Clone,
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
    V: ?Sized,
    for<'a> V: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Copy,
{
}
