use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    pair::{KeyValueMutPtrs, KeyValuePtrs, KeyValueSlicePtrs, KeyValueSlices, KeyValueSlicesMut},
    soa::{
        traits::{RawSoa, RawSoaContext, SliceMutPtrs, Soa},
        wrapper,
    },
};

pub struct KeyValueSliceMutPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    keys: *mut [K],
    values: wrapper::SliceMutPtrs<'context, V>,
}

impl<'context, K, V> KeyValueSliceMutPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    #[track_caller]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        context: &'context V::Context,
        keys: *mut [K],
        values: SliceMutPtrs<'context, V>,
    ) -> Self {
        let keys_len = keys.len();
        let values_len = context.slice_mut_ptrs_len(&values);
        assert_eq!(keys_len, values_len);

        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn new_unchecked(keys: *mut [K], values: SliceMutPtrs<'context, V>) -> Self {
        let values = wrapper::SliceMutPtrs::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn from_raw_parts(
        context: &'context V::Context,
        ptrs: KeyValueMutPtrs<'context, K, V>,
        len: usize,
    ) -> Self {
        let KeyValueMutPtrs { key, value } = ptrs;

        let keys = ptr::slice_from_raw_parts_mut(key, len);
        let values = context.slice_mut_ptrs_from_raw_parts(value.into_inner(), len);
        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_parts(self) -> (*mut [K], SliceMutPtrs<'context, V>) {
        let Self { keys, values } = self;
        (keys, values.into_inner())
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        context.slice_mut_ptrs_len(values.as_inner())
    }

    #[inline]
    pub fn cast_const(self, context: &'context V::Context) -> KeyValueSlicePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let keys = keys.cast_const();
        let values = context.slice_ptrs_cast_const(values.into_inner());
        unsafe { KeyValueSlicePtrs::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.cast_const().cast(); // should be `keys.as_ptr()` but it's unstable
        let values = context.slice_ptrs_cast_const(values.into_inner());
        let value = context.slice_ptrs_as_ptrs(values);
        KeyValuePtrs::new(key, value)
    }

    #[inline]
    pub fn into_mut_ptrs(self, context: &'context V::Context) -> KeyValueMutPtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.cast(); // should be `keys.as_ptr()` but it's unstable
        let value = context.slice_mut_ptrs_as_ptrs(values.into_inner());
        KeyValueMutPtrs::new(key, value)
    }

    #[inline]
    pub unsafe fn drop_in_place(self, context: &V::Context) {
        let Self { keys, values } = self;

        unsafe {
            ptr::drop_in_place(keys);
            context.slices_drop_in_place(values.into_inner());
        }
    }
}

impl<'context, K, V> KeyValueSliceMutPtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub unsafe fn deref<'a>(
        self,
        context: &'context V::Context,
    ) -> KeyValueSlices<'context, 'a, K, V> {
        let Self { keys, values } = self;

        let keys = unsafe { &*keys };
        let values = context.slice_ptrs_cast_const(values.into_inner());
        let values = unsafe { V::slice_ptrs_to_slices(context, values) };
        unsafe { KeyValueSlices::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(
        self,
        context: &'context V::Context,
    ) -> KeyValueSlicesMut<'context, 'a, K, V> {
        let Self { keys, values } = self;

        let keys = unsafe { &mut *keys };
        let values = unsafe { V::slice_mut_ptrs_to_slices(context, values.into_inner()) };
        unsafe { KeyValueSlicesMut::new_unchecked(keys, values) }
    }
}

impl<'context, K, V> From<KeyValueSliceMutPtrs<'context, K, V>>
    for (*mut [K], SliceMutPtrs<'context, V>)
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueSliceMutPtrs<'context, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for KeyValueSliceMutPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SliceMutPtrs<'c, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;
        f.debug_struct("KeyValueSliceMutPtrs")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> PartialEq for KeyValueSliceMutPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SliceMutPtrs<'c, V>: PartialEq,
{
    #[expect(ambiguous_wide_pointer_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<K, V> Eq for KeyValueSliceMutPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SliceMutPtrs<'c, V>: Eq,
{
}

impl<K, V> PartialOrd for KeyValueSliceMutPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SliceMutPtrs<'c, V>: PartialOrd,
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

impl<K, V> Ord for KeyValueSliceMutPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SliceMutPtrs<'c, V>: Ord,
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

impl<K, V> Hash for KeyValueSliceMutPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SliceMutPtrs<'c, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}

impl<K, V> Clone for KeyValueSliceMutPtrs<'_, K, V>
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

impl<K, V> Copy for KeyValueSliceMutPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> SliceMutPtrs<'c, V>: Copy,
{
}
