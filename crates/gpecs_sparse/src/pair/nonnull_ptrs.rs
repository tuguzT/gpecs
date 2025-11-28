use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr::NonNull,
};

use crate::{
    pair::{KeyValueMutPtrs, KeyValuePtrs},
    soa::{
        traits::{RawSoa, RawSoaContext},
        wrapper::NonNullPtrs,
    },
};

pub struct KeyValueNonNullPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    pub key: NonNull<K>,
    pub value: NonNullPtrs<'context, V>,
}

impl<'context, K, V> KeyValueNonNullPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub unsafe fn new_unchecked(
        context: &'context V::Context,
        ptrs: KeyValueMutPtrs<'context, K, V>,
    ) -> Self {
        let KeyValueMutPtrs { key, value } = ptrs;

        let key = unsafe { NonNull::new_unchecked(key) };
        let value = NonNullPtrs::new(unsafe { context.ptrs_to_nonnull(value.into_inner()) });
        Self { key, value }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = key.as_ptr().cast_const();
        let value = context.nonnull_to_ptrs(value.into_inner());
        let value = context.ptrs_cast_const(value);
        KeyValuePtrs::new(key, value)
    }

    #[inline]
    pub fn into_mut_ptrs(self, context: &'context V::Context) -> KeyValueMutPtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = key.as_ptr();
        let value = context.nonnull_to_ptrs(value.into_inner());
        KeyValueMutPtrs::new(key, value)
    }
}

impl<'context, K, V> From<(NonNull<K>, NonNullPtrs<'context, V>)>
    for KeyValueNonNullPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: (NonNull<K>, NonNullPtrs<'context, V>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, K, V> From<KeyValueNonNullPtrs<'context, K, V>>
    for (NonNull<K>, NonNullPtrs<'context, V>)
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueNonNullPtrs<'context, K, V>) -> Self {
        let KeyValueNonNullPtrs { key, value } = value;
        (key, value)
    }
}

impl<'context, K, V> Debug for KeyValueNonNullPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
    NonNullPtrs<'context, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("KeyValueNonNullPtrs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<'context, K, V> PartialEq for KeyValueNonNullPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
    NonNullPtrs<'context, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, K, V> Eq for KeyValueNonNullPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
    NonNullPtrs<'context, V>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValueNonNullPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
    NonNullPtrs<'context, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value } = self;
        match key.partial_cmp(&other.key) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        value.partial_cmp(&other.value)
    }
}

impl<'context, K, V> Ord for KeyValueNonNullPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
    NonNullPtrs<'context, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value } = self;
        match key.cmp(&other.key) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        value.cmp(&other.value)
    }
}

impl<'context, K, V> Hash for KeyValueNonNullPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
    NonNullPtrs<'context, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<K, V> Clone for KeyValueNonNullPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;
        let value = value.clone();
        Self { key, value }
    }
}

impl<'context, K, V> Copy for KeyValueNonNullPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
    NonNullPtrs<'context, V>: Copy,
{
}
