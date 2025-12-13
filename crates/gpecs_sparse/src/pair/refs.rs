use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    pair::KeyValuePtrs,
    soa::{traits::Soa, wrapper},
};

pub struct KeyValueRefs<'context, 'a, K, V>
where
    V: Soa + ?Sized + 'a,
{
    pub key: &'a K,
    pub value: wrapper::Refs<'context, 'a, V>,
}

impl<'context, 'a, K, V> KeyValueRefs<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn new(key: &'a K, value: V::Refs<'context, 'a>) -> Self {
        let value = wrapper::Refs::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn into_parts(self) -> (&'a K, V::Refs<'context, 'a>) {
        let Self { key, value } = self;
        (key, value.into_inner())
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = ptr::from_ref(key);
        let value = V::refs_as_ptrs(context, value.into_inner());
        KeyValuePtrs::new(key, value)
    }
}

impl<'context, 'a, K, V> From<(&'a K, V::Refs<'context, 'a>)> for KeyValueRefs<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: (&'a K, V::Refs<'context, 'a>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'context, 'a, K, V> From<KeyValueRefs<'context, 'a, K, V>> for (&'a K, V::Refs<'context, 'a>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueRefs<'context, 'a, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for KeyValueRefs<'_, '_, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    for<'c, 'a> V::Refs<'c, 'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("KeyValueRefs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<K, V> PartialEq for KeyValueRefs<'_, '_, K, V>
where
    K: PartialEq,
    V: Soa + ?Sized,
    for<'c, 'a> V::Refs<'c, 'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<K, V> Eq for KeyValueRefs<'_, '_, K, V>
where
    K: Eq,
    V: Soa + ?Sized,
    for<'c, 'a> V::Refs<'c, 'a>: Eq,
{
}

impl<K, V> PartialOrd for KeyValueRefs<'_, '_, K, V>
where
    K: PartialOrd,
    V: Soa + ?Sized,
    for<'c, 'a> V::Refs<'c, 'a>: PartialOrd,
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

impl<K, V> Ord for KeyValueRefs<'_, '_, K, V>
where
    K: Ord,
    V: Soa + ?Sized,
    for<'c, 'a> V::Refs<'c, 'a>: Ord,
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

impl<K, V> Hash for KeyValueRefs<'_, '_, K, V>
where
    K: Hash,
    V: Soa + ?Sized,
    for<'c, 'a> V::Refs<'c, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<K, V> Clone for KeyValueRefs<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'a> V::Refs<'c, 'a>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;
        let value = value.clone();
        Self { key, value }
    }
}

impl<K, V> Copy for KeyValueRefs<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'c, 'a> V::Refs<'c, 'a>: Copy,
{
}
