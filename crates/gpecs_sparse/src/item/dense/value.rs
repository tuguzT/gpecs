#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct DenseItem<K, V>
where
    V: ?Sized,
{
    pub key: K,
    pub value: V,
}

impl<K, V> DenseItem<K, V> {
    #[inline]
    pub const fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

impl<K, V> From<(K, V)> for DenseItem<K, V> {
    #[inline]
    fn from(value: (K, V)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<K, V> From<DenseItem<K, V>> for (K, V) {
    #[inline]
    fn from(value: DenseItem<K, V>) -> Self {
        let DenseItem { key, value } = value;
        (key, value)
    }
}
