use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    item::{DenseMutPtrs, DenseRefs},
    soa::{
        traits::{RefsMut, Soa, SoaContext, SoaOwned},
        wrapper,
    },
};

pub struct DenseRefsMut<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    pub key: &'a mut K,
    pub value: wrapper::RefsMut<'ctx, 'a, V>,
}

impl<'ctx, 'a, K, V> DenseRefsMut<'ctx, 'a, K, V>
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
    pub fn into_ptrs(self, context: &'ctx V::Context) -> DenseMutPtrs<'ctx, K, V> {
        let Self { key, value } = self;

        let key = ptr::from_mut(key);
        let value = context.mut_refs_as_mut_ptrs(value.into_inner());
        DenseMutPtrs::new(key, value)
    }

    #[inline]
    pub fn into_refs(self, context: &'ctx V::Context) -> DenseRefs<'ctx, 'a, K, V> {
        let Self { key, value } = self;

        let key = &*key;
        let value = context.mut_refs_as_refs(value.into_inner());
        DenseRefs::new(key, value)
    }
}

impl<'ctx, 'a, K, V> From<(&'a mut K, RefsMut<'ctx, 'a, V>)> for DenseRefsMut<'ctx, 'a, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    fn from(value: (&'a mut K, RefsMut<'ctx, 'a, V>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'ctx, 'a, K, V> From<DenseRefsMut<'ctx, 'a, K, V>> for (&'a mut K, RefsMut<'ctx, 'a, V>)
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    fn from(value: DenseRefsMut<'ctx, 'a, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for DenseRefsMut<'_, '_, K, V>
where
    K: Debug,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("DenseRefsMut")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<K, V> PartialEq for DenseRefsMut<'_, '_, K, V>
where
    K: PartialEq,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<K, V> Eq for DenseRefsMut<'_, '_, K, V>
where
    K: Eq,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: Eq,
{
}

impl<K, V> PartialOrd for DenseRefsMut<'_, '_, K, V>
where
    K: PartialOrd,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: PartialOrd,
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

impl<K, V> Ord for DenseRefsMut<'_, '_, K, V>
where
    K: Ord,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: Ord,
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

impl<K, V> Hash for DenseRefsMut<'_, '_, K, V>
where
    K: Hash,
    V: SoaOwned + ?Sized,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}
