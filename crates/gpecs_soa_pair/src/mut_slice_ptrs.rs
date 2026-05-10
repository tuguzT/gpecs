use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use gpecs_soa::{
    traits::{RawSoa, RawSoaContext, SliceMutPtrs, Soa, SoaContext},
    wrapper,
};

use crate::{KeyValueMutPtrs, KeyValueMutSlices, KeyValuePtrs, KeyValueSlicePtrs, KeyValueSlices};

pub struct KeyValueMutSlicePtrs<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    keys: *mut [K],
    values: wrapper::SliceMutPtrs<'ctx, V>,
}

impl<'ctx, K, V> KeyValueMutSlicePtrs<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    #[track_caller]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(context: &'ctx V::Context, keys: *mut [K], values: SliceMutPtrs<'ctx, V>) -> Self {
        let keys_len = keys.len();
        let values_len = context.mut_slice_ptrs_len(&values);
        assert_eq!(keys_len, values_len);

        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn new_unchecked(keys: *mut [K], values: SliceMutPtrs<'ctx, V>) -> Self {
        let values = wrapper::SliceMutPtrs::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn from_raw_parts(
        context: &'ctx V::Context,
        ptrs: KeyValueMutPtrs<'ctx, K, V>,
        len: usize,
    ) -> Self {
        let KeyValueMutPtrs { key, value } = ptrs;

        let keys = ptr::slice_from_raw_parts_mut(key, len);
        let values = context.mut_slice_ptrs_from_raw_parts(value.into_inner(), len);
        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_parts(self) -> (*mut [K], SliceMutPtrs<'ctx, V>) {
        let Self { keys, values } = self;
        (keys, values.into_inner())
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        context.mut_slice_ptrs_len(values.as_inner())
    }

    #[inline]
    pub fn cast_const(self, context: &'ctx V::Context) -> KeyValueSlicePtrs<'ctx, K, V> {
        let Self { keys, values } = self;

        let keys = keys.cast_const();
        let values = context.slice_ptrs_cast_const(values.into_inner());
        unsafe { KeyValueSlicePtrs::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'ctx V::Context) -> KeyValuePtrs<'ctx, K, V> {
        let Self { keys, values } = self;

        let key = keys.cast_const().cast(); // should be `keys.as_ptr()` but it's unstable
        let values = context.slice_ptrs_cast_const(values.into_inner());
        let value = context.slice_ptrs_as_ptrs(values);
        KeyValuePtrs::new(key, value)
    }

    #[inline]
    pub fn into_mut_ptrs(self, context: &'ctx V::Context) -> KeyValueMutPtrs<'ctx, K, V> {
        let Self { keys, values } = self;

        let key = keys.cast(); // should be `keys.as_ptr()` but it's unstable
        let value = context.mut_slice_ptrs_as_ptrs(values.into_inner());
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

impl<'ctx, 'a, K, V> KeyValueMutSlicePtrs<'ctx, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    pub unsafe fn as_ref_unchecked(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueSlices<'ctx, 'a, K, V> {
        unsafe { self.cast_const(context).as_ref_unchecked(context) }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueMutSlices<'ctx, 'a, K, V> {
        let Self { keys, values } = self;

        let keys = unsafe { keys.as_mut_unchecked() };
        let values = unsafe { context.mut_slice_ptrs_to_mut_slices(values.into_inner()) };
        unsafe { KeyValueMutSlices::new_unchecked(keys, values) }
    }
}

impl<'ctx, K, V> From<KeyValueMutSlicePtrs<'ctx, K, V>> for (*mut [K], SliceMutPtrs<'ctx, V>)
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueMutSlicePtrs<'ctx, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for KeyValueMutSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> SliceMutPtrs<'ctx, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;

        f.debug_struct("KeyValueMutSlicePtrs")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V> PartialEq for KeyValueMutSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> SliceMutPtrs<'ctx, V>: PartialEq,
{
    #[expect(ambiguous_wide_pointer_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<K, V> Eq for KeyValueMutSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> SliceMutPtrs<'ctx, V>: Eq,
{
}

impl<K, V> PartialOrd for KeyValueMutSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> SliceMutPtrs<'ctx, V>: PartialOrd,
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

impl<K, V> Ord for KeyValueMutSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> SliceMutPtrs<'ctx, V>: Ord,
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

impl<K, V> Hash for KeyValueMutSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> SliceMutPtrs<'ctx, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;

        keys.hash(state);
        values.hash(state);
    }
}

impl<K, V> Clone for KeyValueMutSlicePtrs<'_, K, V>
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

impl<K, V> Copy for KeyValueMutSlicePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> SliceMutPtrs<'ctx, V>: Copy,
{
}
