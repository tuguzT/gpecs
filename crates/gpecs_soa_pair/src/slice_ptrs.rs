use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use gpecs_soa::{
    traits::{RawSoa, RawSoaContext, SlicePtrs, Soa, SoaContext},
    wrapper,
};

use crate::{KeyValueMutSlicePtrs, KeyValuePtrs, KeyValueSlices};

pub struct KeyValueSlicePtrs<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    keys: *const [K],
    values: wrapper::SlicePtrs<'ctx, V>,
}

impl<'ctx, K, V> KeyValueSlicePtrs<'ctx, K, V>
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
        ptrs: KeyValuePtrs<'ctx, K, V>,
        len: usize,
    ) -> Self {
        let KeyValuePtrs { key, value } = ptrs;

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
    pub fn cast_mut(self, context: &'ctx V::Context) -> KeyValueMutSlicePtrs<'ctx, K, V> {
        let Self { keys, values } = self;

        let keys = keys.cast_mut();
        let values = context.slice_ptrs_cast_mut(values.into_inner());
        unsafe { KeyValueMutSlicePtrs::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'ctx V::Context) -> KeyValuePtrs<'ctx, K, V> {
        let Self { keys, values } = self;

        let key = keys.cast(); // should be `keys.as_ptr()` but it's unstable
        let value = context.slice_ptrs_as_ptrs(values.into_inner());
        KeyValuePtrs::new(key, value)
    }
}

impl<'ctx, 'a, K, V> KeyValueSlicePtrs<'ctx, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    pub unsafe fn as_ref_unchecked(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueSlices<'ctx, 'a, K, V> {
        let Self { keys, values } = self;

        let keys = unsafe { keys.as_ref_unchecked() };
        let values = unsafe { context.slice_ptrs_to_slices(values.into_inner()) };
        unsafe { KeyValueSlices::new_unchecked(keys, values) }
    }
}

impl<'ctx, K, V> From<KeyValueSlicePtrs<'ctx, K, V>> for (*const [K], SlicePtrs<'ctx, V>)
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueSlicePtrs<'ctx, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for KeyValueSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> SlicePtrs<'ctx, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;

        f.debug_struct("KeyValueSlicePtrs")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> PartialEq for KeyValueSlicePtrs<'_, K, V>
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

impl<K, V> Eq for KeyValueSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> SlicePtrs<'ctx, V>: Eq,
{
}

impl<K, V> PartialOrd for KeyValueSlicePtrs<'_, K, V>
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

impl<K, V> Ord for KeyValueSlicePtrs<'_, K, V>
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

impl<K, V> Hash for KeyValueSlicePtrs<'_, K, V>
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

impl<K, V> Clone for KeyValueSlicePtrs<'_, K, V>
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

impl<K, V> Copy for KeyValueSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> SlicePtrs<'ctx, V>: Copy,
{
}
