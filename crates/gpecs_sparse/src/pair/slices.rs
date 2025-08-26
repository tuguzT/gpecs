use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    pair::{KeyValuePtrs, KeyValueSlicePtrs},
    soa::{traits::Soa, wrapper::Slices},
};

pub struct KeyValueSlices<'context, 'a, K, V>
where
    V: Soa + ?Sized + 'a,
{
    keys: &'a [K],
    values: Slices<'context, 'a, V>,
}

impl<'context, 'a, K, V> KeyValueSlices<'context, 'a, K, V>
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
        let values = Slices::new(values);
        Self { keys, values }
    }

    #[inline]
    pub fn len(&self, context: &V::Context) -> usize {
        let Self { values, .. } = self;
        V::slices_len(context, values.as_inner())
    }

    #[inline]
    pub fn into_parts(self) -> (&'a [K], Slices<'context, 'a, V>) {
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
        let values = V::slices_as_slice_ptrs(context, values.into_inner());
        unsafe { KeyValueSlicePtrs::new_unchecked(keys, values) }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { keys, values } = self;

        let key = keys.as_ptr();
        let value = V::slices_as_ptrs(context, values.into_inner());
        KeyValuePtrs::new(key, value)
    }
}

impl<'context, 'a, K, V> From<KeyValueSlices<'context, 'a, K, V>>
    for (&'a [K], Slices<'context, 'a, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueSlices<'context, 'a, K, V>) -> Self {
        value.into_parts()
    }
}

impl<'context, 'a, K, V> Debug for KeyValueSlices<'context, 'a, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { keys, values } = self;
        f.debug_struct("KeyValueSlices")
            .field("keys", keys)
            .field("values", values)
            .finish()
    }
}

impl<'context, 'a, K, V> Default for KeyValueSlices<'context, 'a, K, V>
where
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            keys: Default::default(),
            values: Slices::default(),
        }
    }
}

impl<'context, 'a, K, V> PartialEq for KeyValueSlices<'context, 'a, K, V>
where
    K: PartialEq,
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { keys, values } = self;
        *keys == other.keys && *values == other.values
    }
}

impl<'context, 'a, K, V> Eq for KeyValueSlices<'context, 'a, K, V>
where
    K: Eq,
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: Eq,
{
}

impl<'context, 'a, K, V> PartialOrd for KeyValueSlices<'context, 'a, K, V>
where
    K: PartialOrd,
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: PartialOrd,
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

impl<'context, 'a, K, V> Ord for KeyValueSlices<'context, 'a, K, V>
where
    K: Ord,
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: Ord,
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

impl<'context, 'a, K, V> Hash for KeyValueSlices<'context, 'a, K, V>
where
    K: Hash,
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { keys, values } = self;
        keys.hash(state);
        values.hash(state);
    }
}

impl<'context, 'a, K, V> Clone for KeyValueSlices<'context, 'a, K, V>
where
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { keys, ref values } = *self;
        let values = values.clone();
        Self { keys, values }
    }
}

impl<'context, 'a, K, V> Copy for KeyValueSlices<'context, 'a, K, V>
where
    V: Soa + ?Sized,
    Slices<'context, 'a, V>: Copy,
{
}
