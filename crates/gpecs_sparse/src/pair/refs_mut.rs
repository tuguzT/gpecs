use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    pair::{KeyValueMutPtrs, KeyValueRefs},
    soa::{traits::Soa, wrapper::RefsMut},
};

pub struct KeyValueRefsMut<'context, 'a, K, V>
where
    V: Soa + ?Sized + 'a,
{
    pub key: &'a mut K,
    pub value: RefsMut<'context, 'a, V>,
}

impl<'context, 'a, K, V> KeyValueRefsMut<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn new(key: &'a mut K, value: V::RefsMut<'context, 'a>) -> Self {
        let value = RefsMut::new(value);
        Self { key, value }
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

impl<'context, 'a, K, V> From<(&'a mut K, RefsMut<'context, 'a, V>)>
    for KeyValueRefsMut<'context, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: (&'a mut K, RefsMut<'context, 'a, V>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, 'a, K, V> From<KeyValueRefsMut<'context, 'a, K, V>>
    for (&'a mut K, RefsMut<'context, 'a, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueRefsMut<'context, 'a, K, V>) -> Self {
        let KeyValueRefsMut { key, value } = value;
        (key, value)
    }
}

impl<'context, 'a, K, V> Debug for KeyValueRefsMut<'context, 'a, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    RefsMut<'context, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("KeyValueRefsMut")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<'context, 'a, K, V> PartialEq for KeyValueRefsMut<'context, 'a, K, V>
where
    K: PartialEq,
    V: Soa + ?Sized,
    RefsMut<'context, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, 'a, K, V> Eq for KeyValueRefsMut<'context, 'a, K, V>
where
    K: Eq,
    V: Soa + ?Sized,
    RefsMut<'context, 'a, V>: Eq,
{
}

impl<'context, 'a, K, V> PartialOrd for KeyValueRefsMut<'context, 'a, K, V>
where
    K: PartialOrd,
    V: Soa + ?Sized,
    RefsMut<'context, 'a, V>: PartialOrd,
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

impl<'context, 'a, K, V> Ord for KeyValueRefsMut<'context, 'a, K, V>
where
    K: Ord,
    V: Soa + ?Sized,
    RefsMut<'context, 'a, V>: Ord,
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

impl<'context, 'a, K, V> Hash for KeyValueRefsMut<'context, 'a, K, V>
where
    K: Hash,
    V: Soa + ?Sized,
    RefsMut<'context, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}
