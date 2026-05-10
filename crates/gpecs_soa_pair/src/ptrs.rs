use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use gpecs_soa::{
    traits::{
        CloneToUninitSoaContext, Ptrs, RawSoa, RawSoaContext, ReadSoaContext, Soa,
        SoaCloneToUninit, SoaContext, SoaRead,
    },
    wrapper,
};

use crate::{KeyValueMutPtrs, KeyValuePair, KeyValueRefs};

pub struct KeyValuePtrs<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    pub key: *const K,
    pub value: wrapper::Ptrs<'ctx, V>,
}

impl<'ctx, K, V> KeyValuePtrs<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub fn new(key: *const K, value: Ptrs<'ctx, V>) -> Self {
        let value = wrapper::Ptrs::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn dangling(context: &'ctx V::Context) -> Self {
        let key = ptr::dangling();
        let value = context.ptrs_dangling();
        Self::new(key, value)
    }

    #[inline]
    pub fn into_parts(self) -> (*const K, Ptrs<'ctx, V>) {
        let Self { key, value } = self;
        (key, value.into_inner())
    }

    #[inline]
    pub fn cast_mut(self, context: &'ctx V::Context) -> KeyValueMutPtrs<'ctx, K, V> {
        let Self { key, value } = self;

        let key = key.cast_mut();
        let value = context.ptrs_cast_mut(value.into_inner());
        KeyValueMutPtrs::new(key, value)
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, context: &'ctx V::Context, offset: usize) -> Self {
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
    pub unsafe fn read<R>(self, context: &'ctx V::Context) -> KeyValuePair<K, R>
    where
        V: SoaRead<'ctx, R>,
    {
        let Self { key, value } = self;

        let key = unsafe { ptr::read(key) };
        let value = unsafe { context.read(value.into_inner()) };
        KeyValuePair::new(key, value)
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
            dst_key.write(key.as_ref_unchecked().clone());
            context.clone_to_uninit(value, dst_value);
        }
    }
}

impl<'ctx, 'a, K, V> KeyValuePtrs<'ctx, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    pub unsafe fn as_ref_unchecked(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueRefs<'ctx, 'a, K, V> {
        let Self { key, value } = self;

        let key = unsafe { key.as_ref_unchecked() };
        let value = unsafe { context.ptrs_to_refs(value.into_inner()) };
        KeyValueRefs::new(key, value)
    }
}

impl<'ctx, K, V> From<(*const K, Ptrs<'ctx, V>)> for KeyValuePtrs<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: (*const K, Ptrs<'ctx, V>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'ctx, K, V> From<KeyValuePtrs<'ctx, K, V>> for (*const K, Ptrs<'ctx, V>)
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: KeyValuePtrs<'ctx, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for KeyValuePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> Ptrs<'ctx, V>: Debug,
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
    for<'ctx> Ptrs<'ctx, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value) == other
    }
}

impl<K, V> Eq for KeyValuePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> Ptrs<'ctx, V>: Eq,
{
}

impl<K, V> PartialOrd for KeyValuePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> Ptrs<'ctx, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value).partial_cmp(&other)
    }
}

impl<K, V> Ord for KeyValuePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> Ptrs<'ctx, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value).cmp(&other)
    }
}

impl<K, V> Hash for KeyValuePtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> Ptrs<'ctx, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        (key, value).hash(state);
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
    for<'ctx> Ptrs<'ctx, V>: Copy,
{
}
