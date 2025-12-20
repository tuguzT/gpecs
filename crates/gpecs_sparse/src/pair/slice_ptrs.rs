use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    pair::{DensePtrs, DenseSliceMutPtrs, DenseSlices},
    soa::{
        traits::{RawSoa, RawSoaContext, SlicePtrs, Soa},
        wrapper,
    },
};

pub struct DenseSlicePtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    keys: *const [K],
    values: wrapper::SlicePtrs<'context, V>,
}

impl<'context, K, V> DenseSlicePtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    #[track_caller]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        context: &'context V::Context,
        keys: *const [K],
        values: SlicePtrs<'context, V>,
    ) -> Self {
        let keys_len = keys.len();
        let values_len = context.slice_ptrs_len(&values);
        assert_eq!(keys_len, values_len);

        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn new_unchecked(keys: *const [K], values: SlicePtrs<'context, V>) -> Self {
        let values = wrapper::SlicePtrs::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn from_raw_parts(
        context: &'context V::Context,
        ptrs: DensePtrs<'context, K, V>,
        len: usize,
    ) -> Self {
        let DensePtrs { key, value } = ptrs;

        let keys = ptr::slice_from_raw_parts(key, len);
        let values = context.slice_ptrs_from_raw_parts(value.into_inner(), len);
        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_parts(self) -> (*const [K], SlicePtrs<'context, V>) {
        let Self { keys, values } = self;
        (keys, values.into_inner())
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        context.slice_ptrs_len(values.as_inner())
    }

    #[inline]
    pub fn cast_mut(self, context: &'context V::Context) -> DenseSliceMutPtrs<'context, K, V> {
        let Self { keys, values } = self;

        let keys = keys.cast_mut();
        let values = context.slice_ptrs_cast_mut(values.into_inner());
        unsafe { DenseSliceMutPtrs::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> DensePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.cast(); // should be `keys.as_ptr()` but it's unstable
        let value = context.slice_ptrs_as_ptrs(values.into_inner());
        DensePtrs::new(key, value)
    }
}

impl<'context, K, V> DenseSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub unsafe fn deref<'a>(
        self,
        context: &'context V::Context,
    ) -> DenseSlices<'context, 'a, K, V> {
        let Self { keys, values } = self;

        let keys = unsafe { &*keys };
        let values = unsafe { V::slice_ptrs_to_slices(context, values.into_inner()) };
        unsafe { DenseSlices::new_unchecked(keys, values) }
    }
}

impl<'context, K, V> From<DenseSlicePtrs<'context, K, V>> for (*const [K], SlicePtrs<'context, V>)
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: DenseSlicePtrs<'context, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for DenseSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SlicePtrs<'c, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;
        f.debug_struct("DenseSlicePtrs")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> PartialEq for DenseSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SlicePtrs<'c, V>: PartialEq,
{
    #[expect(ambiguous_wide_pointer_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<K, V> Eq for DenseSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SlicePtrs<'c, V>: Eq,
{
}

impl<K, V> PartialOrd for DenseSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SlicePtrs<'c, V>: PartialOrd,
{
    #[expect(ambiguous_wide_pointer_comparisons)]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { keys, values } = self;
        match keys.partial_cmp(&other.keys) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        values.partial_cmp(&other.values)
    }
}

impl<K, V> Ord for DenseSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SlicePtrs<'c, V>: Ord,
{
    #[expect(ambiguous_wide_pointer_comparisons)]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { keys, values } = self;
        match keys.cmp(&other.keys) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        values.cmp(&other.values)
    }
}

impl<K, V> Hash for DenseSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SlicePtrs<'c, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}

impl<K, V> Clone for DenseSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { keys, ref values } = *self;
        let values = values.clone();
        Self { keys, values }
    }
}

impl<K, V> Copy for DenseSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SlicePtrs<'c, V>: Copy,
{
}
