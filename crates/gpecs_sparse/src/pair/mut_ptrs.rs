use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use crate::{
    pair::{KeyValuePair, KeyValuePtrs, KeyValueRefs, KeyValueRefsMut},
    soa::{
        traits::{MutPtrs, RawSoa, RawSoaContext, Soa, SoaWrite},
        wrapper::MutPtrs as MutPtrsWrapper,
    },
};

pub struct KeyValueMutPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    pub key: *mut K,
    pub value: MutPtrsWrapper<'context, V>,
}

impl<'context, K, V> KeyValueMutPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub fn new(key: *mut K, value: MutPtrs<'context, V>) -> Self {
        let value = MutPtrsWrapper::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn dangling(context: &'context V::Context) -> Self {
        let key = ptr::dangling_mut();
        let value = context.ptrs_dangling_mut();
        Self::new(key, value)
    }

    #[inline]
    pub fn cast_const(self, context: &'context V::Context) -> KeyValuePtrs<'context, K, V> {
        let Self { key, value } = self;

        let key = key.cast_const();
        let value = context.ptrs_cast_const(value.into_inner());
        KeyValuePtrs::new(key, value)
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, context: &'context V::Context, offset: usize) -> Self {
        let Self { key, value } = self;

        let key = unsafe { key.add(offset) };
        let value = unsafe { context.ptrs_add_mut(value.into_inner(), offset) };
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
        let values_offset = unsafe { context.ptrs_offset_from_mut(value, origin_value) };
        assert_eq!(key_offset, values_offset);

        key_offset
    }

    #[inline]
    pub unsafe fn swap(self, context: &V::Context, with: KeyValueMutPtrs<'_, K, V>) {
        let Self {
            key: this_key,
            value: this_value,
        } = self;
        let KeyValueMutPtrs {
            key: with_key,
            value: with_value,
        } = with;

        unsafe {
            ptr::swap(this_key, with_key);
            context.ptrs_swap(this_value.into_inner(), with_value.into_inner());
        }
    }

    #[inline]
    pub unsafe fn copy_from(self, context: &V::Context, from: KeyValuePtrs<'_, K, V>, len: usize) {
        let Self {
            key: dst_key,
            value: dst_value,
        } = self;
        let KeyValuePtrs {
            key: src_key,
            value: src_value,
        } = from;

        unsafe {
            ptr::copy(src_key, dst_key, len);
            context.ptrs_copy(src_value.into_inner(), dst_value.into_inner(), len);
        }
    }

    #[inline]
    pub unsafe fn copy_from_rev(
        self,
        context: &V::Context,
        from: KeyValuePtrs<'_, K, V>,
        len: usize,
    ) {
        let Self {
            key: dst_key,
            value: dst_value,
        } = self;
        let KeyValuePtrs {
            key: src_key,
            value: src_value,
        } = from;

        unsafe {
            context.ptrs_copy_rev(src_value.into_inner(), dst_value.into_inner(), len);
            ptr::copy(src_key, dst_key, len);
        }
    }

    #[inline]
    pub unsafe fn copy_from_nonoverlapping(
        self,
        context: &V::Context,
        from: KeyValuePtrs<'_, K, V>,
        len: usize,
    ) {
        let Self {
            key: dst_key,
            value: dst_value,
        } = self;
        let KeyValuePtrs {
            key: src_key,
            value: src_value,
        } = from;

        let src_value = src_value.into_inner();
        let dst_value = dst_value.into_inner();
        unsafe {
            ptr::copy_nonoverlapping(src_key, dst_key, len);
            context.ptrs_copy_nonoverlapping(src_value, dst_value, len);
        }
    }

    #[inline]
    pub unsafe fn drop_in_place(self, context: &V::Context) {
        let Self { key, value } = self;

        unsafe {
            ptr::drop_in_place(key);
            context.ptrs_drop_in_place(value.into_inner());
        }
    }
}

impl<'context, K, V> KeyValueMutPtrs<'context, K, V>
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
        let value = context.ptrs_cast_const(value.into_inner());
        let value = unsafe { V::ptrs_to_refs(context, value) };
        KeyValueRefs::new(key, value)
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(
        self,
        context: &'context V::Context,
    ) -> KeyValueRefsMut<'context, 'a, K, V> {
        let Self { key, value } = self;

        let key = unsafe { &mut *key };
        let value = unsafe { V::ptrs_to_refs_mut(context, value.into_inner()) };
        KeyValueRefsMut::new(key, value)
    }
}

impl<K, V> KeyValueMutPtrs<'_, K, V>
where
    V: SoaWrite,
{
    #[inline]
    pub unsafe fn write(self, context: &V::Context, value: KeyValuePair<K, V>) {
        let Self {
            key: key_ptr,
            value: value_ptr,
        } = self;
        let KeyValuePair { key, value } = value;

        unsafe {
            ptr::write(key_ptr, key);
            V::write(context, value_ptr.into_inner(), value);
        }
    }
}

impl<'context, K, V> From<(*mut K, MutPtrsWrapper<'context, V>)> for KeyValueMutPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: (*mut K, MutPtrsWrapper<'context, V>)) -> Self {
        let (key, value) = value;
        Self { key, value }
    }
}

impl<'context, K, V> From<KeyValueMutPtrs<'context, K, V>> for (*mut K, MutPtrsWrapper<'context, V>)
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueMutPtrs<'context, K, V>) -> Self {
        let KeyValueMutPtrs { key, value } = value;
        (key, value)
    }
}

impl<'context, K, V> Debug for KeyValueMutPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
    MutPtrsWrapper<'context, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;
        f.debug_struct("KeyValueMutPtrs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<'context, K, V> PartialEq for KeyValueMutPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
    MutPtrsWrapper<'context, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;
        *key == other.key && *value == other.value
    }
}

impl<'context, K, V> Eq for KeyValueMutPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
    MutPtrsWrapper<'context, V>: Eq,
{
}

impl<'context, K, V> PartialOrd for KeyValueMutPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
    MutPtrsWrapper<'context, V>: PartialOrd,
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

impl<'context, K, V> Ord for KeyValueMutPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
    MutPtrsWrapper<'context, V>: Ord,
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

impl<'context, K, V> Hash for KeyValueMutPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
    MutPtrsWrapper<'context, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        key.hash(state);
        value.hash(state);
    }
}

impl<K, V> Clone for KeyValueMutPtrs<'_, K, V>
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

impl<'context, K, V> Copy for KeyValueMutPtrs<'context, K, V>
where
    V: RawSoa + ?Sized,
    MutPtrsWrapper<'context, V>: Copy,
{
}
