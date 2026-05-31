use core::{
    alloc::Layout,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
};

use gpecs_ptr::slice::{CoreSliceItemPtrs, SliceItemPtrs};
use gpecs_soa::layout::WithLayout;

pub struct KeyValuePair<K, V, P = CoreSliceItemPtrs<K>>
where
    V: ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    phantom: PhantomData<P>,
    key: K,
    value: V,
}

impl<K, V, P> KeyValuePair<K, V, P>
where
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    pub const fn new(key: K, value: V) -> Self {
        let phantom = PhantomData;
        Self {
            phantom,
            key,
            value,
        }
    }

    #[inline]
    pub fn map_key<N>(self, f: impl FnOnce(K) -> N) -> KeyValuePair<N, V> {
        let Self { key, value, .. } = self;

        let key = f(key);
        KeyValuePair::new(key, value)
    }

    #[inline]
    pub fn map_value<N>(self, f: impl FnOnce(V) -> N) -> KeyValuePair<K, N> {
        let Self { key, value, .. } = self;

        let value = f(value);
        KeyValuePair::new(key, value)
    }

    #[inline]
    pub fn into_parts(self) -> (K, V) {
        let Self { key, value, .. } = self;
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

impl<K, V, P> KeyValuePair<K, V, P>
where
    V: ?Sized,
    P: SliceItemPtrs<Item = K>,
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

impl<K, V, P> From<(K, V)> for KeyValuePair<K, V, P>
where
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    fn from(pair: (K, V)) -> Self {
        let (key, value) = pair;
        Self::new(key, value)
    }
}

impl<K, V, P> From<KeyValuePair<K, V, P>> for (K, V)
where
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    fn from(pair: KeyValuePair<K, V, P>) -> Self {
        pair.into_parts()
    }
}

impl<K, V, P> Debug for KeyValuePair<K, V, P>
where
    K: Debug,
    V: Debug + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value, .. } = self;

        f.debug_struct("KeyValuePair")
            .field("key", key)
            .field("value", &value)
            .finish()
    }
}

impl<K, V, P> Clone for KeyValuePair<K, V, P>
where
    K: Clone,
    V: Clone,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { key, value, .. } = self;

        let key = key.clone();
        let value = value.clone();
        Self::new(key, value)
    }
}

impl<K, V, P> Copy for KeyValuePair<K, V, P>
where
    K: Copy,
    V: Copy,
    P: SliceItemPtrs<Item = K>,
{
}

impl<K, V, P> PartialEq for KeyValuePair<K, V, P>
where
    K: PartialEq,
    V: PartialEq + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value, .. } = self;

        let other = (&other.key, &other.value);
        (key, value) == other
    }
}

impl<K, V, P> Eq for KeyValuePair<K, V, P>
where
    K: Eq,
    V: Eq + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
}

impl<K, V, P> PartialOrd for KeyValuePair<K, V, P>
where
    K: PartialOrd,
    V: PartialOrd + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value, .. } = self;

        let other = (&other.key, &other.value);
        (key, value).partial_cmp(&other)
    }
}

impl<K, V, P> Ord for KeyValuePair<K, V, P>
where
    K: Ord,
    V: Ord + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value, .. } = self;

        let other = (&other.key, &other.value);
        (key, value).cmp(&other)
    }
}

impl<K, V, P> Hash for KeyValuePair<K, V, P>
where
    K: Hash,
    V: Hash + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value, .. } = self;
        (key, value).hash(state);
    }
}

impl<K, V, P> WithLayout for KeyValuePair<K, V, P>
where
    V: WithLayout + ?Sized,
    P: SliceItemPtrs<Item = K>,
{
    #[inline]
    fn layout(&self) -> Layout {
        let Self { value, .. } = self;
        value.layout()
    }
}
