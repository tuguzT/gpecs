use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr::NonNull,
};

use crate::{
    item::{DenseMutPtrs, DensePtrs},
    soa::{
        traits::{MutPtrs, NonNullPtrs, RawSoa, RawSoaContext},
        wrapper,
    },
};

pub struct DenseNonNullPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    pub key: NonNull<K>,
    pub value: wrapper::NonNullPtrs<'context, V>,
}

impl<'context, K, V> DenseNonNullPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub fn new(key: NonNull<K>, value: NonNullPtrs<'context, V>) -> Self {
        let value = wrapper::NonNullPtrs::new(value);
        Self { key, value }
    }

    #[inline]
    pub unsafe fn new_unchecked(
        context: &'context V::Context,
        key: *mut K,
        value: MutPtrs<'context, V>,
    ) -> Self {
        let key = unsafe { NonNull::new_unchecked(key) };
        let value = unsafe { context.ptrs_to_nonnull(value) };
        Self::new(key, value)
    }

    #[inline]
    pub fn into_parts(self) -> (NonNull<K>, NonNullPtrs<'context, V>) {
        let Self { key, value } = self;
        (key, value.into_inner())
    }

    #[inline]
    pub fn into_ptrs(self, context: &'context V::Context) -> DensePtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = key.as_ptr().cast_const();
        let value = context.nonnull_to_ptrs(value.into_inner());
        let value = context.ptrs_cast_const(value);
        DensePtrs::new(key, value)
    }

    #[inline]
    pub fn into_mut_ptrs(self, context: &'context V::Context) -> DenseMutPtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = key.as_ptr();
        let value = context.nonnull_to_ptrs(value.into_inner());
        DenseMutPtrs::new(key, value)
    }
}

impl<'context, K, V> From<(NonNull<K>, NonNullPtrs<'context, V>)>
    for DenseNonNullPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: (NonNull<K>, NonNullPtrs<'context, V>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'context, K, V> From<DenseNonNullPtrs<'context, K, V>>
    for (NonNull<K>, NonNullPtrs<'context, V>)
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: DenseNonNullPtrs<'context, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for DenseNonNullPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> NonNullPtrs<'c, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("DenseNonNullPtrs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<K, V> PartialEq for DenseNonNullPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> NonNullPtrs<'c, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<K, V> Eq for DenseNonNullPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> NonNullPtrs<'c, V>: Eq,
{
}

impl<K, V> PartialOrd for DenseNonNullPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> NonNullPtrs<'c, V>: PartialOrd,
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

impl<K, V> Ord for DenseNonNullPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> NonNullPtrs<'c, V>: Ord,
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

impl<K, V> Hash for DenseNonNullPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> NonNullPtrs<'c, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<K, V> Clone for DenseNonNullPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;
        let value = value.clone();
        Self { key, value }
    }
}

impl<K, V> Copy for DenseNonNullPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> NonNullPtrs<'c, V>: Copy,
{
}
