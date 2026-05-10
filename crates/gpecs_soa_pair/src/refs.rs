use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use gpecs_soa::{
    traits::{Refs, Soa, SoaContext, SoaOwned},
    wrapper,
};

use crate::KeyValuePtrs;

pub struct KeyValueRefs<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    pub key: &'a K,
    pub value: wrapper::Refs<'ctx, 'a, V>,
}

impl<'ctx, 'a, K, V> KeyValueRefs<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn new(key: &'a K, value: Refs<'ctx, 'a, V>) -> Self {
        let value = wrapper::Refs::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn into_parts(self) -> (&'a K, Refs<'ctx, 'a, V>) {
        let Self { key, value } = self;
        (key, value.into_inner())
    }

    #[inline]
    pub fn into_ptrs(self, context: &'ctx V::Context) -> KeyValuePtrs<'ctx, K, V> {
        let Self { key, value } = self;

        let key = ptr::from_ref(key);
        let value = context.refs_as_ptrs(value.into_inner());
        KeyValuePtrs::new(key, value)
    }
}

impl<'ctx, 'a, K, V> From<(&'a K, Refs<'ctx, 'a, V>)> for KeyValueRefs<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    fn from(value: (&'a K, Refs<'ctx, 'a, V>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'ctx, 'a, K, V> From<KeyValueRefs<'ctx, 'a, K, V>> for (&'a K, Refs<'ctx, 'a, V>)
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    fn from(value: KeyValueRefs<'ctx, 'a, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for KeyValueRefs<'_, '_, K, V>
where
    K: Debug,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: Debug,
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
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<K, V> Eq for KeyValueRefs<'_, '_, K, V>
where
    K: Eq,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: Eq,
{
}

impl<K, V> PartialOrd for KeyValueRefs<'_, '_, K, V>
where
    K: PartialOrd,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: PartialOrd,
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
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: Ord,
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
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;

        key.hash(state);
        value.hash(state);
    }
}

impl<K, V> Clone for KeyValueRefs<'_, '_, K, V>
where
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: Clone,
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
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: Copy,
{
}
