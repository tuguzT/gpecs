use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    item::{DenseMutPtrs, DensePtrs, DenseSliceMutPtrs, DenseSlicePtrs, DenseSlices},
    soa::{traits::Soa, wrapper},
};

pub struct DenseSlicesMut<'context, 'a, K, V>
where
    V: Soa + ?Sized + 'a,
{
    keys: &'a mut [K],
    values: wrapper::SlicesMut<'context, 'a, V>,
}

impl<'context, 'a, K, V> DenseSlicesMut<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    #[track_caller]
    pub fn new(
        context: &'context V::Context,
        keys: &'a mut [K],
        values: V::SlicesMut<'context, 'a>,
    ) -> Self {
        let keys_len = keys.len();
        let values_len = V::slices_mut_len(context, &values);
        assert_eq!(keys_len, values_len);

        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn new_unchecked(keys: &'a mut [K], values: V::SlicesMut<'context, 'a>) -> Self {
        let values = wrapper::SlicesMut::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        V::slices_mut_len(context, values.as_inner())
    }

    #[inline]
    pub fn into_parts(self) -> (&'a mut [K], V::SlicesMut<'context, 'a>) {
        let Self { keys, values } = self;
        (keys, values.into_inner())
    }

    #[inline]
    pub fn into_slice_ptrs(self, context: &'context V::Context) -> DenseSlicePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let keys = ptr::from_ref(keys);
        let values = V::slices_mut_as_slices(context, values.into_inner());
        let values = V::slices_as_slice_ptrs(context, values);
        unsafe { DenseSlicePtrs::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_slice_mut_ptrs(
        self,
        context: &'context V::Context,
    ) -> DenseSliceMutPtrs<'context, K, V> {
        let Self { keys, values } = self;

        let keys = ptr::from_mut(keys);
        let values = V::slices_mut_as_slice_ptrs(context, values.into_inner());
        unsafe { DenseSliceMutPtrs::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> DensePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.as_ptr();
        let values = V::slices_mut_as_slices(context, values.into_inner());
        let value = V::slices_as_ptrs(context, values);
        DensePtrs::new(key, value)
    }

    #[inline]
    pub fn into_mut_ptrs(self, context: &'context V::Context) -> DenseMutPtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.as_mut_ptr();
        let value = V::slices_mut_as_ptrs(context, values.into_inner());
        DenseMutPtrs::new(key, value)
    }

    #[inline]
    pub fn into_slices(self, context: &'context V::Context) -> DenseSlices<'context, 'a, K, V> {
        let Self { keys, values } = self;

        let keys = &*keys;
        let values = V::slices_mut_as_slices(context, values.into_inner());
        unsafe { DenseSlices::new_unchecked(keys, values) }
    }
}

impl<'context, 'a, K, V> From<DenseSlicesMut<'context, 'a, K, V>>
    for (&'a mut [K], V::SlicesMut<'context, 'a>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: DenseSlicesMut<'context, 'a, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for DenseSlicesMut<'_, '_, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    for<'c, 'a> V::SlicesMut<'c, 'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;
        f.debug_struct("DenseSlicesMut")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> Default for DenseSlicesMut<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'a> V::SlicesMut<'c, 'a>: Default,
{
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = V::SlicesMut::default();
        unsafe { Self::new_unchecked(keys, values) }
    }
}

impl<K, V> PartialEq for DenseSlicesMut<'_, '_, K, V>
where
    K: PartialEq,
    V: Soa + ?Sized,
    for<'c, 'a> V::SlicesMut<'c, 'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<K, V> Eq for DenseSlicesMut<'_, '_, K, V>
where
    K: Eq,
    V: Soa + ?Sized,
    for<'c, 'a> V::SlicesMut<'c, 'a>: Eq,
{
}

impl<K, V> PartialOrd for DenseSlicesMut<'_, '_, K, V>
where
    K: PartialOrd,
    V: Soa + ?Sized,
    for<'c, 'a> V::SlicesMut<'c, 'a>: PartialOrd,
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

impl<K, V> Ord for DenseSlicesMut<'_, '_, K, V>
where
    K: Ord,
    V: Soa + ?Sized,
    for<'c, 'a> V::SlicesMut<'c, 'a>: Ord,
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

impl<K, V> Hash for DenseSlicesMut<'_, '_, K, V>
where
    K: Hash,
    V: Soa + ?Sized,
    for<'c, 'a> V::SlicesMut<'c, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}
