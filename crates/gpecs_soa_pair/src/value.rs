use core::alloc::Layout;

use gpecs_soa::layout::WithLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    #[inline]
    pub fn map_key<N>(self, f: impl FnOnce(K) -> N) -> KeyValuePair<N, V> {
        let Self { key, value } = self;

        let key = f(key);
        KeyValuePair { key, value }
    }

    #[inline]
    pub fn map_value<N>(self, f: impl FnOnce(V) -> N) -> KeyValuePair<K, N> {
        let Self { key, value } = self;

        let value = f(value);
        KeyValuePair { key, value }
    }

    #[inline]
    pub fn into_parts(self) -> (K, V) {
        let Self { key, value } = self;
        (key, value)
    }

    #[inline]
    pub fn into_key(self) -> K {
        let Self { key, .. } = self;
        key
    }

    #[inline]
    pub fn into_value(self) -> V {
        let Self { value, .. } = self;
        value
    }
}

impl<K, V> KeyValuePair<K, V>
where
    V: ?Sized,
{
    #[inline]
    pub const fn as_key(&self) -> &K {
        let Self { key, .. } = self;
        key
    }

    #[inline]
    pub const fn as_value(&self) -> &V {
        let Self { value, .. } = self;
        value
    }

    #[inline]
    pub const fn as_mut_key(&mut self) -> &mut K {
        let Self { key, .. } = self;
        key
    }

    #[inline]
    pub const fn as_mut_value(&mut self) -> &mut V {
        let Self { value, .. } = self;
        value
    }
}

impl<K, V> From<(K, V)> for KeyValuePair<K, V> {
    #[inline]
    fn from(pair: (K, V)) -> Self {
        let (key, value) = pair;
        Self { key, value }
    }
}

impl<K, V> From<KeyValuePair<K, V>> for (K, V) {
    #[inline]
    fn from(pair: KeyValuePair<K, V>) -> Self {
        pair.into_parts()
    }
}

impl<K, V> WithLayout for KeyValuePair<K, V>
where
    V: WithLayout + ?Sized,
{
    #[inline]
    fn layout(&self) -> Layout {
        let Self { value, .. } = self;
        value.layout()
    }
}
