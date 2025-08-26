use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    pair::{
        KeyValueMutPtrs, KeyValuePtrs, KeyValueSliceMutPtrs, KeyValueSlicePtrs, KeyValueSlices,
    },
    soa::{traits::Soa, wrapper::SlicesMut},
};

pub struct KeyValueSlicesMut<'context, 'a, K, V>
where
    V: Soa + ?Sized + 'a,
{
    keys: &'a mut [K],
    values: SlicesMut<'context, 'a, V>,
}

impl<'context, 'a, K, V> KeyValueSlicesMut<'context, 'a, K, V>
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
        let values = SlicesMut::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        V::slices_mut_len(context, values.as_inner())
    }

    #[inline]
    pub fn into_parts(self) -> (&'a mut [K], SlicesMut<'context, 'a, V>) {
        let Self { keys, values } = self;
        (keys, values)
    }

    #[inline]
    pub fn into_slice_ptrs(
        self,
        context: &'context V::Context,
    ) -> KeyValueSlicePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let keys = ptr::from_ref(keys);
        let values = V::slices_mut_as_slices(context, values.into_inner());
        let values = V::slices_as_slice_ptrs(context, values);
        unsafe { KeyValueSlicePtrs::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_slice_mut_ptrs(
        self,
        context: &'context V::Context,
    ) -> KeyValueSliceMutPtrs<'context, K, V> {
        let Self { keys, values } = self;

        let keys = ptr::from_mut(keys);
        let values = V::slices_mut_as_slice_ptrs(context, values.into_inner());
        unsafe { KeyValueSliceMutPtrs::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.as_ptr();
        let values = V::slices_mut_as_slices(context, values.into_inner());
        let value = V::slices_as_ptrs(context, values);
        KeyValuePtrs::new(key, value)
    }

    #[inline]
    pub fn into_mut_ptrs(self, context: &'context V::Context) -> KeyValueMutPtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.as_mut_ptr();
        let value = V::slices_mut_as_ptrs(context, values.into_inner());
        KeyValueMutPtrs::new(key, value)
    }

    #[inline]
    pub fn into_slices(self, context: &'context V::Context) -> KeyValueSlices<'context, 'a, K, V> {
        let Self { keys, values } = self;

        let keys = &*keys;
        let values = V::slices_mut_as_slices(context, values.into_inner());
        unsafe { KeyValueSlices::new_unchecked(keys, values) }
    }
}

impl<'context, 'a, K, V> From<KeyValueSlicesMut<'context, 'a, K, V>>
    for (&'a mut [K], SlicesMut<'context, 'a, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueSlicesMut<'context, 'a, K, V>) -> Self {
        value.into_parts()
    }
}

impl<'context, 'a, K, V> Debug for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    SlicesMut<'context, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;
        f.debug_struct("KeyValueSlicesMut")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<'context, 'a, K, V> Default for KeyValueSlicesMut<'context, 'a, K, V>
where
    V: Soa + ?Sized,
    SlicesMut<'context, 'a, V>: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            keys: Default::default(),
            values: SlicesMut::default(),
        }
    }
}

impl<'context, 'a, K, V> PartialEq for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: PartialEq,
    V: Soa + ?Sized,
    SlicesMut<'context, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<'context, 'a, K, V> Eq for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: Eq,
    V: Soa + ?Sized,
    SlicesMut<'context, 'a, V>: Eq,
{
}

impl<'context, 'a, K, V> PartialOrd for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: PartialOrd,
    V: Soa + ?Sized,
    SlicesMut<'context, 'a, V>: PartialOrd,
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

impl<'context, 'a, K, V> Ord for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: Ord,
    V: Soa + ?Sized,
    SlicesMut<'context, 'a, V>: Ord,
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

impl<'context, 'a, K, V> Hash for KeyValueSlicesMut<'context, 'a, K, V>
where
    K: Hash,
    V: Soa + ?Sized,
    SlicesMut<'context, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}
