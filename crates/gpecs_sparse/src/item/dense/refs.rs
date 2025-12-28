use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    item::DensePtrs,
    soa::{traits::Soa, wrapper},
};

pub struct DenseRefs<'ctx, 'a, K, V>
where
    V: Soa + ?Sized + 'a,
{
    pub key: &'a K,
    pub value: wrapper::Refs<'ctx, 'a, V>,
}

impl<'ctx, 'a, K, V> DenseRefs<'ctx, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn new(key: &'a K, value: V::Refs<'ctx, 'a>) -> Self {
        let value = wrapper::Refs::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn into_parts(self) -> (&'a K, V::Refs<'ctx, 'a>) {
        let Self { key, value } = self;
        (key, value.into_inner())
    }

    #[inline]
    pub fn into_ptrs(self, context: &'ctx V::Context) -> DensePtrs<'ctx, K, V> {
        let Self { key, value } = self;

        let key = ptr::from_ref(key);
        let value = V::refs_as_ptrs(context, value.into_inner());
        DensePtrs::new(key, value)
    }
}

impl<'ctx, 'a, K, V> From<(&'a K, V::Refs<'ctx, 'a>)> for DenseRefs<'ctx, 'a, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: (&'a K, V::Refs<'ctx, 'a>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'ctx, 'a, K, V> From<DenseRefs<'ctx, 'a, K, V>> for (&'a K, V::Refs<'ctx, 'a>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: DenseRefs<'ctx, 'a, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for DenseRefs<'_, '_, K, V>
where
    K: Debug,
    V: Soa + ?Sized,
    for<'ctx, 'a> V::Refs<'ctx, 'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("DenseRefs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<K, V> PartialEq for DenseRefs<'_, '_, K, V>
where
    K: PartialEq,
    V: Soa + ?Sized,
    for<'ctx, 'a> V::Refs<'ctx, 'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<K, V> Eq for DenseRefs<'_, '_, K, V>
where
    K: Eq,
    V: Soa + ?Sized,
    for<'ctx, 'a> V::Refs<'ctx, 'a>: Eq,
{
}

impl<K, V> PartialOrd for DenseRefs<'_, '_, K, V>
where
    K: PartialOrd,
    V: Soa + ?Sized,
    for<'ctx, 'a> V::Refs<'ctx, 'a>: PartialOrd,
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

impl<K, V> Ord for DenseRefs<'_, '_, K, V>
where
    K: Ord,
    V: Soa + ?Sized,
    for<'ctx, 'a> V::Refs<'ctx, 'a>: Ord,
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

impl<K, V> Hash for DenseRefs<'_, '_, K, V>
where
    K: Hash,
    V: Soa + ?Sized,
    for<'ctx, 'a> V::Refs<'ctx, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<K, V> Clone for DenseRefs<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'ctx, 'a> V::Refs<'ctx, 'a>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;
        let value = value.clone();
        Self { key, value }
    }
}

impl<K, V> Copy for DenseRefs<'_, '_, K, V>
where
    V: Soa + ?Sized,
    for<'ctx, 'a> V::Refs<'ctx, 'a>: Copy,
{
}
