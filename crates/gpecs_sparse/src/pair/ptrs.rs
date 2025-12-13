use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    pair::{KeyValueMutPtrs, KeyValuePair, KeyValueRefs},
    soa::{
        traits::{Ptrs, RawSoa, RawSoaContext, Soa, SoaCloneToUninit, SoaRead},
        wrapper,
    },
};

pub struct KeyValuePtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    pub key: *const K,
    pub value: wrapper::Ptrs<'context, V>,
}

impl<'context, K, V> KeyValuePtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub fn new(key: *const K, value: Ptrs<'context, V>) -> Self {
        let value = wrapper::Ptrs::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn dangling(context: &'context V::Context) -> Self {
        let key = ptr::dangling();
        let value = context.ptrs_dangling();
        Self::new(key, value)
    }

    #[inline]
    pub fn into_parts(self) -> (*const K, Ptrs<'context, V>) {
        let Self { key, value } = self;
        (key, value.into_inner())
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
}

impl<K, V> KeyValuePtrs<'_, K, V>
where
    K: Clone,
    V: SoaCloneToUninit + ?Sized,
{
    #[inline]
    pub unsafe fn clone_to_uninit(self, context: &V::Context, dst: KeyValueMutPtrs<'_, K, V>) {
        let Self { key, value } = self;
        let value = value.into_inner();

        let KeyValueMutPtrs {
            key: dst_key,
            value: dst_value,
        } = dst;
        let dst_value = dst_value.into_inner();

        unsafe {
            dst_key.write((&*key).clone());
            V::clone_to_uninit(context, value, dst_value);
        }
    }
}

impl<'context, K, V> KeyValuePtrs<'context, K, V>
where
    V: Soa + ?Sized,
{
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

impl<'context, K, V> From<(*const K, Ptrs<'context, V>)> for KeyValuePtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: (*const K, Ptrs<'context, V>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'context, K, V> From<KeyValuePtrs<'context, K, V>> for (*const K, Ptrs<'context, V>)
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: KeyValuePtrs<'context, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for KeyValuePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> Ptrs<'c, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("KeyValuePtrs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<K, V> PartialEq for KeyValuePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> Ptrs<'c, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<K, V> Eq for KeyValuePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> Ptrs<'c, V>: Eq,
{
}

impl<K, V> PartialOrd for KeyValuePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> Ptrs<'c, V>: PartialOrd,
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

impl<K, V> Ord for KeyValuePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> Ptrs<'c, V>: Ord,
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

impl<K, V> Hash for KeyValuePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> Ptrs<'c, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<K, V> Clone for KeyValuePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
{
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;
        let value = value.clone();
        Self { key, value }
    }
}

impl<K, V> Copy for KeyValuePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'c> Ptrs<'c, V>: Copy,
{
}
