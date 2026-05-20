use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr,
};

use gpecs_soa::{
    traits::{MutPtrs, RawSoa, RawSoaContext, Soa, SoaContext, SoaWrite, WriteSoaContext},
    wrapper,
};

use crate::{KeyValueMutRefs, KeyValuePair, KeyValuePtrs, KeyValueRefs};

pub struct KeyValueMutPtrs<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    key: *mut K,
    value: wrapper::MutPtrs<'ctx, V>,
}

impl<'ctx, K, V> KeyValueMutPtrs<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    pub fn new(key: *mut K, value: MutPtrs<'ctx, V>) -> Self {
        let value = wrapper::MutPtrs::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn dangling(context: &'ctx V::Context) -> Self {
        let key = ptr::dangling_mut();
        let value = context.ptrs_dangling_mut();
        Self::new(key, value)
    }

    #[inline]
    pub fn into_parts(self) -> (*mut K, MutPtrs<'ctx, V>) {
        let Self { key, value } = self;
        (key, value.into_inner())
    }

    #[inline]
    pub fn cast_const(self, context: &'ctx V::Context) -> KeyValuePtrs<'ctx, K, V> {
        let (key, value) = self.into_parts();

        let key = key.cast_const();
        let value = context.ptrs_cast_const(value);
        KeyValuePtrs::new(key, value)
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, context: &'ctx V::Context, offset: usize) -> Self {
        let (key, value) = self.into_parts();

        let key = unsafe { key.add(offset) };
        let value = unsafe { context.ptrs_add_mut(value, offset) };
        Self::new(key, value)
    }

    #[inline]
    pub unsafe fn offset_from(self, context: &V::Context, origin: KeyValuePtrs<'_, K, V>) -> isize {
        let (key, value) = self.into_parts();
        let (origin_key, origin_value) = origin.into_parts();

        let key_offset = unsafe { key.offset_from(origin_key) };
        let values_offset = unsafe { context.ptrs_offset_from_mut(value, origin_value) };
        assert_eq!(key_offset, values_offset);

        key_offset
    }

    #[inline]
    pub unsafe fn swap(self, context: &V::Context, with: KeyValueMutPtrs<'_, K, V>) {
        let (key, value) = self.into_parts();
        let (with_key, with_value) = with.into_parts();

        unsafe {
            ptr::swap(key, with_key);
            context.ptrs_swap(value, with_value);
        }
    }

    #[inline]
    pub unsafe fn copy_from_forward(
        self,
        context: &V::Context,
        from: KeyValuePtrs<'_, K, V>,
        len: usize,
    ) {
        let (dst_key, dst_value) = self.into_parts();
        let (src_key, src_value) = from.into_parts();

        unsafe {
            ptr::copy(src_key, dst_key, len);
            context.ptrs_copy_forward(src_value, dst_value, len);
        }
    }

    #[inline]
    pub unsafe fn copy_from_backward(
        self,
        context: &V::Context,
        from: KeyValuePtrs<'_, K, V>,
        len: usize,
    ) {
        let (dst_key, dst_value) = self.into_parts();
        let (src_key, src_value) = from.into_parts();

        unsafe {
            context.ptrs_copy_backward(src_value, dst_value, len);
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
        let (dst_key, dst_value) = self.into_parts();
        let (src_key, src_value) = from.into_parts();

        unsafe {
            ptr::copy_nonoverlapping(src_key, dst_key, len);
            context.ptrs_copy_nonoverlapping(src_value, dst_value, len);
        }
    }

    #[inline]
    pub unsafe fn drop_in_place(self, context: &V::Context) {
        let (key, value) = self.into_parts();

        unsafe {
            ptr::drop_in_place(key);
            context.ptrs_drop_in_place(value);
        }
    }

    #[inline]
    pub unsafe fn write<W>(self, context: &V::Context, value: KeyValuePair<K, W>)
    where
        V: SoaWrite<W>,
    {
        let (key_ptr, value_ptr) = self.into_parts();
        let (key, value) = value.into_parts();

        unsafe {
            ptr::write(key_ptr, key);
            context.write(value_ptr, value);
        }
    }
}

impl<'ctx, 'a, K, V> KeyValueMutPtrs<'ctx, K, V>
where
    V: Soa<'a> + ?Sized,
{
    #[inline]
    pub unsafe fn as_ref_unchecked(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueRefs<'ctx, 'a, K, V> {
        unsafe { self.cast_const(context).as_ref_unchecked(context) }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueMutRefs<'ctx, 'a, K, V> {
        let (key, value) = self.into_parts();

        let key = unsafe { key.as_mut_unchecked() };
        let value = unsafe { context.mut_ptrs_to_mut_refs(value) };
        KeyValueMutRefs::new(key, value)
    }
}

impl<'ctx, K, V> From<(*mut K, MutPtrs<'ctx, V>)> for KeyValueMutPtrs<'ctx, K, V>
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: (*mut K, MutPtrs<'ctx, V>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'ctx, K, V> From<KeyValueMutPtrs<'ctx, K, V>> for (*mut K, MutPtrs<'ctx, V>)
where
    V: RawSoa + ?Sized,
{
    #[inline]
    fn from(value: KeyValueMutPtrs<'ctx, K, V>) -> Self {
        value.into_parts()
    }
}

impl<K, V> Debug for KeyValueMutPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;

        f.debug_struct("KeyValueMutPtrs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<K, V> PartialEq for KeyValueMutPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value) == other
    }
}

impl<K, V> Eq for KeyValueMutPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, V>: Eq,
{
}

impl<K, V> PartialOrd for KeyValueMutPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value).partial_cmp(&other)
    }
}

impl<K, V> Ord for KeyValueMutPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value).cmp(&other)
    }
}

impl<K, V> Hash for KeyValueMutPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        (key, value).hash(state);
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

impl<K, V> Copy for KeyValueMutPtrs<'_, K, V>
where
    V: RawSoa + ?Sized,
    for<'ctx> MutPtrs<'ctx, V>: Copy,
{
}
