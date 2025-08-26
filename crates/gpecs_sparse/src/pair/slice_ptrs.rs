use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    pair::{KeyValuePtrs, KeyValueSliceMutPtrs, KeyValueSlices},
    soa::{traits::Soa, wrapper::SlicePtrs},
};

pub struct KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    keys: *const [K],
    values: SlicePtrs<'context, V>,
}

impl<'context, K, V> KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    #[track_caller]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        context: &'context V::Context,
        keys: *const [K],
        values: V::SlicePtrs<'context>,
    ) -> Self {
        let keys_len = keys.len();
        let values_len = V::slice_ptrs_len(context, &values);
        assert_eq!(keys_len, values_len);

        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn new_unchecked(keys: *const [K], values: V::SlicePtrs<'context>) -> Self {
        let values = SlicePtrs::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn from_raw_parts(
        context: &'context V::Context,
        ptrs: KeyValuePtrs<'context, K, V>,
        len: usize,
    ) -> Self {
        let KeyValuePtrs { key, value } = ptrs;

        let keys = ptr::slice_from_raw_parts(key, len);
        let values = SlicePtrs::new(V::slices_from_raw_parts(context, value.into_inner(), len));
        Self { keys, values }
    }

    #[inline]
    pub fn into_parts(self) -> (*const [K], SlicePtrs<'context, V>) {
        let Self { keys, values } = self;
        (keys, values)
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        V::slice_ptrs_len(context, values.as_inner())
    }

    #[inline]
    pub fn cast_mut(self, context: &'context V::Context) -> KeyValueSliceMutPtrs<'context, K, V> {
        let Self { keys, values } = self;

        let keys = keys.cast_mut();
        let values = V::slice_ptrs_cast_mut(context, values.into_inner());
        unsafe { KeyValueSliceMutPtrs::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.cast(); // should be `keys.as_ptr()` but it's unstable
        let value = V::slice_ptrs_as_ptrs(context, values.into_inner());
        KeyValuePtrs::new(key, value)
    }

    #[inline]
    pub unsafe fn deref<'a>(
        self,
        context: &'context V::Context,
    ) -> KeyValueSlices<'context, 'a, K, V> {
        let Self { keys, values } = self;

        let keys = unsafe { &*keys };
        let values = unsafe { V::slice_ptrs_to_slices(context, values.into_inner()) };
        unsafe { KeyValueSlices::new_unchecked(keys, values) }
    }
}

impl<'context, K, V> From<KeyValueSlicePtrs<'context, K, V>>
    for (*const [K], SlicePtrs<'context, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueSlicePtrs<'context, K, V>) -> Self {
        value.into_parts()
    }
}

impl<'context, K, V> Debug for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SlicePtrs<'context, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;
        f.debug_struct("KeyValueSlicePtrs")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<'context, K, V> PartialEq for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SlicePtrs<'context, V>: PartialEq,
{
    #[expect(ambiguous_wide_pointer_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<'context, K, V> Eq for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SlicePtrs<'context, V>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SlicePtrs<'context, V>: PartialOrd,
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

impl<'context, K, V> Ord for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SlicePtrs<'context, V>: Ord,
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

impl<'context, K, V> Hash for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SlicePtrs<'context, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}

impl<K, V> Clone for KeyValueSlicePtrs<'_, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { keys, ref values } = *self;
        let values = values.clone();
        Self { keys, values }
    }
}

impl<'context, K, V> Copy for KeyValueSlicePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    SlicePtrs<'context, V>: Copy,
{
}
