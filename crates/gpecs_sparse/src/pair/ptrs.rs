use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    pair::{KeyValueMutPtrs, KeyValuePair, KeyValueRefs},
    soa::{
        traits::{Ptrs, RawSoaContext, Soa, SoaRead},
        wrapper::Ptrs as PtrsWrapper,
    },
};

pub struct KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    pub key: *const K,
    pub value: PtrsWrapper<'context, V>,
}

impl<'context, K, V> KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    pub fn new(key: *const K, value: Ptrs<'context, V>) -> Self {
        let value = PtrsWrapper::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn dangling(context: &'context V::Context) -> Self {
        let key = ptr::dangling();
        let value = context.ptrs_dangling();
        Self::new(key, value)
    }

    #[inline]
    pub fn cast_mut(self, context: &'context V::Context) -> KeyValueMutPtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = key.cast_mut();
        let value = context.ptrs_cast_mut(value.into_inner());
        KeyValueMutPtrs::new(key, value)
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, context: &'context V::Context, offset: usize) -> Self {
        let Self { key, value } = self;

        let key = unsafe { key.add(offset) };
        let value = unsafe { context.ptrs_add(value.into_inner(), offset) };
        Self::new(key, value)
    }

    #[inline]
    pub unsafe fn offset_from(self, context: &V::Context, origin: KeyValuePtrs<'_, K, V>) -> isize {
        let Self { key, value } = self;
        let KeyValuePtrs {
            key: origin_key,
            value: origin_value,
        } = origin;

        let value = value.into_inner();
        let origin_value = origin_value.into_inner();

        let key_offset = unsafe { key.offset_from(origin_key) };
        let values_offset = unsafe { context.ptrs_offset_from(value, origin_value) };
        assert_eq!(key_offset, values_offset);

        key_offset
    }

    #[inline]
    pub unsafe fn deref<'a>(
        self,
        context: &'context V::Context,
    ) -> KeyValueRefs<'context, 'a, K, V> {
        let Self { key, value } = self;

        let key = unsafe { &*key };
        let value = unsafe { V::ptrs_to_refs(context, value.into_inner()) };
        KeyValueRefs::new(key, value)
    }
}

impl<K, V> KeyValuePtrs<'_, K, V>
where
    V: SoaRead,
{
    #[inline]
    pub unsafe fn read(self, context: &V::Context) -> KeyValuePair<K, V> {
        let Self { key, value } = self;

        let key = unsafe { ptr::read(key) };
        let value = unsafe { V::read(context, value.into_inner()) };
        KeyValuePair::new(key, value)
    }
}

impl<'context, K, V> From<(*const K, PtrsWrapper<'context, V>)> for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: (*const K, PtrsWrapper<'context, V>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, K, V> From<KeyValuePtrs<'context, K, V>> for (*const K, PtrsWrapper<'context, V>)
where
    V: Soa + ?Sized,
{
    #[inline]
    fn from(value: KeyValuePtrs<'context, K, V>) -> Self {
        let KeyValuePtrs { key, value } = value;
        (key, value)
    }
}

impl<'context, K, V> Debug for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    PtrsWrapper<'context, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("KeyValuePtrs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<'context, K, V> PartialEq for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    PtrsWrapper<'context, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, K, V> Eq for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    PtrsWrapper<'context, V>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    PtrsWrapper<'context, V>: PartialOrd,
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

impl<'context, K, V> Ord for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    PtrsWrapper<'context, V>: Ord,
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

impl<'context, K, V> Hash for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    PtrsWrapper<'context, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<K, V> Clone for KeyValuePtrs<'_, K, V>
where
    V: Soa + ?Sized,
{
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;
        let value = value.clone();
        Self { key, value }
    }
}

impl<'context, K, V> Copy for KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
    PtrsWrapper<'context, V>: Copy,
{
}
