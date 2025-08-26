use crate::{
    pair::{KeyValueRefs, KeyValueRefsMut},
    soa::traits::Soa,
};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct KeyValuePair<K, V>
where
    V: ?Sized,
{
    pub key: K,
    pub value: V,
}

impl<K, V> KeyValuePair<K, V> {
    #[inline]
    pub const fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

impl<K, V> KeyValuePair<K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn as_refs<'a>(&'a self, context: &'a V::Context) -> KeyValueRefs<'a, 'a, K, V> {
        let Self { key, value } = self;

        let value = V::value_as_refs(context, value);
        KeyValueRefs::new(key, value)
    }

    #[inline]
    pub fn as_refs_mut<'a>(&'a mut self, context: &'a V::Context) -> KeyValueRefsMut<'a, 'a, K, V> {
        let Self { key, value } = self;

        let value = V::mut_value_as_refs(context, value);
        KeyValueRefsMut::new(key, value)
    }
}

impl<K, V> From<(K, V)> for KeyValuePair<K, V> {
    #[inline]
    fn from(value: (K, V)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<K, V> From<KeyValuePair<K, V>> for (K, V) {
    #[inline]
    fn from(value: KeyValuePair<K, V>) -> Self {
        let KeyValuePair { key, value } = value;
        (key, value)
    }
}
