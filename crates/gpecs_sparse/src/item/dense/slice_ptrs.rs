use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    item::{DensePtrs, DenseSliceMutPtrs, DenseSlices},
    soa::{
        traits::{RawSoa, RawSoaContext, SlicePtrs, Soa, SoaContext},
        wrapper,
    },
};

pub struct DenseSlicePtrs<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    keys: *const [K],
    values: wrapper::SlicePtrs<'ctx, V>,
}

impl<'ctx, K, V> DenseSlicePtrs<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    #[track_caller]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(context: &'ctx V::Context, keys: *const [K], values: SlicePtrs<'ctx, V>) -> Self {
        let keys_len = keys.len();
        let values_len = context.slice_ptrs_len(&values);
        assert_eq!(keys_len, values_len);

        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn new_unchecked(keys: *const [K], values: SlicePtrs<'ctx, V>) -> Self {
        let values = wrapper::SlicePtrs::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn from_raw_parts(
        context: &'ctx V::Context,
        ptrs: DensePtrs<'ctx, K, V>,
        len: usize,
    ) -> Self {
        let DensePtrs { key, value } = ptrs;

        let keys = ptr::slice_from_raw_parts(key, len);
        let values = context.slice_ptrs_from_raw_parts(value.into_inner(), len);
        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_parts(self) -> (*const [K], SlicePtrs<'ctx, V>) {
        let Self { keys, values } = self;
        (keys, values.into_inner())
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        context.slice_ptrs_len(values.as_inner())
    }

    #[inline]
    pub fn cast_mut(self, context: &'ctx V::Context) -> DenseSliceMutPtrs<'ctx, K, V> {
        let Self { keys, values } = self;

        let keys = keys.cast_mut();
        let values = context.slice_ptrs_cast_mut(values.into_inner());
        unsafe { DenseSliceMutPtrs::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'ctx V::Context) -> DensePtrs<'ctx, K, V> {
        let Self { keys, values } = self;

        let key = keys.cast(); // should be `keys.as_ptr()` but it's unstable
        let value = context.slice_ptrs_as_ptrs(values.into_inner());
        DensePtrs::new(key, value)
    }
}

impl<'ctx, 'a, K, V> DenseSlicePtrs<'ctx, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    pub unsafe fn deref(self, context: &'ctx V::Context) -> DenseSlices<'ctx, 'a, K, V> {
        let Self { keys, values } = self;

        let keys = unsafe { keys.as_ref_unchecked() };
        let values = unsafe { context.slice_ptrs_to_slices(values.into_inner()) };
        unsafe { DenseSlices::new_unchecked(keys, values) }
    }
}

impl<'ctx, K, V> From<DenseSlicePtrs<'ctx, K, V>> for (*const [K], SlicePtrs<'ctx, V>)
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: DenseSlicePtrs<'ctx, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for DenseSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> SlicePtrs<'ctx, V>: Debug,
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
    for<'ctx> SlicePtrs<'ctx, V>: PartialEq,
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
    for<'ctx> SlicePtrs<'ctx, V>: Eq,
{
}

impl<K, V> PartialOrd for DenseSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> SlicePtrs<'ctx, V>: PartialOrd,
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
    for<'ctx> SlicePtrs<'ctx, V>: Ord,
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
    for<'ctx> SlicePtrs<'ctx, V>: Hash,
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
    for<'ctx> SlicePtrs<'ctx, V>: Copy,
{
}
