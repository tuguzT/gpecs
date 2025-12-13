use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    pair::{KeyValueMutPtrs, KeyValueRefs},
    soa::{traits::Soa, wrapper},
};

pub struct KeyValueRefsMut<'context, 'a, K, V>
where
    V: Soa + ?Sized + 'a,
{
    pub key: &'a mut K,
    pub value: wrapper::RefsMut<'context, 'a, V>,
}

impl<'context, 'a, K, V> KeyValueRefsMut<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn new(key: &'a mut K, value: V::RefsMut<'context, 'a>) -> Self {
        let value = wrapper::RefsMut::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn into_parts(self) -> (&'a mut K, V::RefsMut<'context, 'a>) {
        let Self { key, value } = self;
        (key, value.into_inner())
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> KeyValueMutPtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = ptr::from_mut(key);
        let value = V::refs_mut_as_ptrs(context, value.into_inner());
        KeyValueMutPtrs::new(key, value)
    }

    #[inline]
    pub fn into_refs(self, context: &'context V::Context) -> KeyValueRefs<'context, 'a, K, V> {
        let Self { key, value } = self;

        let key = &*key;
        let value = V::refs_mut_as_refs(context, value.into_inner());
        KeyValueRefs::new(key, value)
    }
}

impl<'context, 'a, K, V> From<(&'a mut K, V::RefsMut<'context, 'a>)>
    for KeyValueRefsMut<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: (&'a mut K, V::RefsMut<'context, 'a>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'context, 'a, K, V> From<KeyValueRefsMut<'context, 'a, K, V>>
    for (&'a mut K, V::RefsMut<'context, 'a>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueRefsMut<'context, 'a, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for KeyValueRefsMut<'_, '_, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    for<'c, 'a> V::RefsMut<'c, 'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("KeyValueRefsMut")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<K, V> PartialEq for KeyValueRefsMut<'_, '_, K, V>
where
    K: PartialEq,
    V: Soa + ?Sized,
    for<'c, 'a> V::RefsMut<'c, 'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<K, V> Eq for KeyValueRefsMut<'_, '_, K, V>
where
    K: Eq,
    V: Soa + ?Sized,
    for<'c, 'a> V::RefsMut<'c, 'a>: Eq,
{
}

impl<K, V> PartialOrd for KeyValueRefsMut<'_, '_, K, V>
where
    K: PartialOrd,
    V: Soa + ?Sized,
    for<'c, 'a> V::RefsMut<'c, 'a>: PartialOrd,
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

impl<K, V> Ord for KeyValueRefsMut<'_, '_, K, V>
where
    K: Ord,
    V: Soa + ?Sized,
    for<'c, 'a> V::RefsMut<'c, 'a>: Ord,
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

impl<K, V> Hash for KeyValueRefsMut<'_, '_, K, V>
where
    K: Hash,
    V: Soa + ?Sized,
    for<'c, 'a> V::RefsMut<'c, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}
