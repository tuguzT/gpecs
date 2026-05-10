use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use gpecs_soa::{
    traits::{RefsMut, Soa, SoaContext, SoaOwned},
    wrapper,
};

use crate::{KeyValueMutPtrs, KeyValueRefs};

pub struct KeyValueMutRefs<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    pub key: &'a mut K,
    pub value: wrapper::RefsMut<'ctx, 'a, V>,
}

impl<'ctx, 'a, K, V> KeyValueMutRefs<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn new(key: &'a mut K, value: RefsMut<'ctx, 'a, V>) -> Self {
        let value = wrapper::RefsMut::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn into_parts(self) -> (&'a mut K, RefsMut<'ctx, 'a, V>) {
        let Self { key, value } = self;
        (key, value.into_inner())
    }

    #[inline]
    pub fn into_ptrs(self, context: &'ctx V::Context) -> KeyValueMutPtrs<'ctx, K, V> {
        let Self { key, value } = self;

        let key = ptr::from_mut(key);
        let value = context.mut_refs_as_mut_ptrs(value.into_inner());
        KeyValueMutPtrs::new(key, value)
    }

    #[inline]
    pub fn into_refs(self, context: &'ctx V::Context) -> KeyValueRefs<'ctx, 'a, K, V> {
        let Self { key, value } = self;

        let key = &*key;
        let value = context.mut_refs_as_refs(value.into_inner());
        KeyValueRefs::new(key, value)
    }
}

impl<'ctx, 'a, K, V> From<(&'a mut K, RefsMut<'ctx, 'a, V>)> for KeyValueMutRefs<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    fn from(value: (&'a mut K, RefsMut<'ctx, 'a, V>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'ctx, 'a, K, V> From<KeyValueMutRefs<'ctx, 'a, K, V>> for (&'a mut K, RefsMut<'ctx, 'a, V>)
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    fn from(value: KeyValueMutRefs<'ctx, 'a, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for KeyValueMutRefs<'_, '_, K, V>
where
    K: Debug,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;

        f.debug_struct("KeyValueMutRefs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<K, V> PartialEq for KeyValueMutRefs<'_, '_, K, V>
where
    K: PartialEq,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value) == other
    }
}

impl<K, V> Eq for KeyValueMutRefs<'_, '_, K, V>
where
    K: Eq,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: Eq,
{
}

impl<K, V> PartialOrd for KeyValueMutRefs<'_, '_, K, V>
where
    K: PartialOrd,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value).partial_cmp(&other)
    }
}

impl<K, V> Ord for KeyValueMutRefs<'_, '_, K, V>
where
    K: Ord,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value).cmp(&other)
    }
}

impl<K, V> Hash for KeyValueMutRefs<'_, '_, K, V>
where
    K: Hash,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        (key, value).hash(state);
    }
}
