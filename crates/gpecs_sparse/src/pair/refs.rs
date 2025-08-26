use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    pair::KeyValuePtrs,
    soa::{traits::Soa, wrapper::Refs},
};

pub struct KeyValueRefs<'context, 'a, K, V>
where
    V: Soa + ?Sized + 'a,
{
    pub key: &'a K,
    pub value: Refs<'context, 'a, V>,
}

impl<'context, 'a, K, V> KeyValueRefs<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn new(key: &'a K, value: V::Refs<'context, 'a>) -> Self {
        let value = Refs::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = ptr::from_ref(key);
        let value = V::refs_as_ptrs(context, value.into_inner());
        KeyValuePtrs::new(key, value)
    }
}

impl<'context, 'a, K, V> From<(&'a K, Refs<'context, 'a, V>)> for KeyValueRefs<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: (&'a K, Refs<'context, 'a, V>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, 'a, K, V> From<KeyValueRefs<'context, 'a, K, V>> for (&'a K, Refs<'context, 'a, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueRefs<'context, 'a, K, V>) -> Self {
        let KeyValueRefs { key, value } = value;
        (key, value)
    }
}

impl<'context, 'a, K, V> Debug for KeyValueRefs<'context, 'a, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("KeyValueRefs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<'context, 'a, K, V> PartialEq for KeyValueRefs<'context, 'a, K, V>
where
    K: PartialEq,
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, 'a, K, V> Eq for KeyValueRefs<'context, 'a, K, V>
where
    K: Eq,
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: Eq,
{
}

impl<'context, 'a, K, V> PartialOrd for KeyValueRefs<'context, 'a, K, V>
where
    K: PartialOrd,
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: PartialOrd,
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

impl<'context, 'a, K, V> Ord for KeyValueRefs<'context, 'a, K, V>
where
    K: Ord,
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: Ord,
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

impl<'context, 'a, K, V> Hash for KeyValueRefs<'context, 'a, K, V>
where
    K: Hash,
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<'context, 'a, K, V> Clone for KeyValueRefs<'context, 'a, K, V>
where
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;
        let value = value.clone();
        Self { key, value }
    }
}

impl<'context, 'a, K, V> Copy for KeyValueRefs<'context, 'a, K, V>
where
    V: Soa + ?Sized,
    Refs<'context, 'a, V>: Copy,
{
}
