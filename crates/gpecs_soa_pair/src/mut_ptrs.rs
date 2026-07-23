use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use gpecs_ptr::slice::{CastConst, MutSliceItemPtr, SliceItemPtr};
use gpecs_soa::{
    traits::{
        AllocSoaContext, MutPtrs, RawSoa, RawSoaContext, Soa, SoaContext, SoaWrite, WriteSoaContext,
    },
    wrapper,
};

use crate::{KeyValueMutRefs, KeyValuePair, KeyValuePtrs, KeyValueRefs};

pub struct KeyValueMutPtrs<'ctx, K, V, P = *mut K>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    key: P,
    value: wrapper::MutPtrs<'ctx, V>,
}

impl<'ctx, K, V, P> KeyValueMutPtrs<'ctx, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    #[inline]
    pub fn new(key: P, value: MutPtrs<'ctx, V>) -> Self {
        let value = wrapper::MutPtrs::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn dangling(context: &'ctx V::Context) -> Self {
        let key = P::dangling();
        let value = context.ptrs_dangling_mut();
        Self::new(key, value)
    }

    #[inline]
    pub fn into_parts(self) -> (P, MutPtrs<'ctx, V>) {
        let Self { key, value } = self;
        (key, value.into_inner())
    }

    #[inline]
    pub fn cast_const(self, context: &'ctx V::Context) -> KeyValuePtrs<'ctx, K, V, CastConst<P>> {
        let (key, value) = self.into_parts();

        let key = key.cast_const();
        let value = context.ptrs_cast_const(value);
        KeyValuePtrs::new(key, value)
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, context: &'ctx V::Context, count: usize) -> Self {
        let (key, value) = self.into_parts();

        let key = unsafe { key.add(count) };
        let value = unsafe { context.ptrs_add_mut(value, count) };
        Self::new(key, value)
    }

    #[inline]
    pub unsafe fn offset_from(
        self,
        context: &V::Context,
        origin: KeyValuePtrs<'_, K, V, CastConst<P>>,
    ) -> isize {
        let (key, value) = self.into_parts();
        let (origin_key, origin_value) = origin.into_parts();

        let key_offset = unsafe { key.cast_const().offset_from(origin_key) };
        let values_offset = unsafe { context.ptrs_offset_from_mut(value, origin_value) };
        assert_eq!(key_offset, values_offset);

        key_offset
    }

    #[inline]
    pub unsafe fn swap_nonoverlapping(
        self,
        context: &V::Context,
        with: KeyValueMutPtrs<'_, K, V, P>,
        count: usize,
    ) {
        let (key, value) = self.into_parts();
        let (with_key, with_value) = with.into_parts();

        unsafe {
            key.swap_nonoverlapping(with_key, count);
            context.ptrs_swap_nonoverlapping(value, with_value, count);
        }
    }

    #[inline]
    pub unsafe fn copy_from_nonoverlapping(
        self,
        context: &V::Context,
        from: KeyValuePtrs<'_, K, V, CastConst<P>>,
        count: usize,
    ) {
        let (dst_key, dst_value) = self.into_parts();
        let (src_key, src_value) = from.into_parts();

        unsafe {
            dst_key.copy_from_nonoverlapping(src_key, count);
            context.ptrs_copy_nonoverlapping(src_value, dst_value, count);
        }
    }

    #[inline]
    pub unsafe fn drop_in_place(self, context: &V::Context) {
        let (key, value) = self.into_parts();

        unsafe {
            key.drop_in_place();
            context.ptrs_drop_in_place(value);
        }
    }

    #[inline]
    pub unsafe fn write<W>(self, context: &V::Context, value: KeyValuePair<K, W, P::Ptrs>)
    where
        V: SoaWrite<W>,
    {
        let (key_ptr, value_ptr) = self.into_parts();
        let (key, value) = value.into_parts();

        unsafe {
            key_ptr.write(key);
            context.write(value_ptr, value);
        }
    }
}

impl<K, V, P> KeyValueMutPtrs<'_, K, V, P>
where
    V: RawSoa<Context: AllocSoaContext<V>> + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    #[inline]
    pub unsafe fn copy_from_forward(
        self,
        context: &V::Context,
        from: KeyValuePtrs<'_, K, V, CastConst<P>>,
        count: usize,
    ) {
        let (dst_key, dst_value) = self.into_parts();
        let (src_key, src_value) = from.into_parts();

        unsafe {
            dst_key.copy_from(src_key, count);
            context.ptrs_copy_forward(src_value, dst_value, count);
        }
    }

    #[inline]
    pub unsafe fn copy_from_backward(
        self,
        context: &V::Context,
        from: KeyValuePtrs<'_, K, V, CastConst<P>>,
        count: usize,
    ) {
        let (dst_key, dst_value) = self.into_parts();
        let (src_key, src_value) = from.into_parts();

        unsafe {
            context.ptrs_copy_backward(src_value, dst_value, count);
            dst_key.copy_from(src_key, count);
        }
    }
}

impl<'ctx, 'a, K, V, P> KeyValueMutPtrs<'ctx, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    #[inline]
    pub unsafe fn as_ref_unchecked(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueRefs<'ctx, 'a, K, V, CastConst<P>> {
        unsafe { self.cast_const(context).as_ref_unchecked(context) }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueMutRefs<'ctx, 'a, K, V, P> {
        let (key, value) = self.into_parts();

        let value = unsafe { context.mut_ptrs_to_mut_refs(value) };
        unsafe { KeyValueMutRefs::from_parts(key, value) }
    }
}

impl<'ctx, K, V, P> From<(P, MutPtrs<'ctx, V>)> for KeyValueMutPtrs<'ctx, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    #[inline]
    fn from(value: (P, MutPtrs<'ctx, V>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'ctx, K, V, P> From<KeyValueMutPtrs<'ctx, K, V, P>> for (P, MutPtrs<'ctx, V>)
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    #[inline]
    fn from(value: KeyValueMutPtrs<'ctx, K, V, P>) -> Self {
        value.into_parts()
    }
}

impl<K, V, P> Debug for KeyValueMutPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K> + Debug,
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

impl<K, V, P> PartialEq for KeyValueMutPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K> + PartialEq,
    for<'ctx> MutPtrs<'ctx, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value) == other
    }
}

impl<K, V, P> Eq for KeyValueMutPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K> + Eq,
    for<'ctx> MutPtrs<'ctx, V>: Eq,
{
}

impl<K, V, P> PartialOrd for KeyValueMutPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K> + PartialOrd,
    for<'ctx> MutPtrs<'ctx, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value).partial_cmp(&other)
    }
}

impl<K, V, P> Ord for KeyValueMutPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K> + Ord,
    for<'ctx> MutPtrs<'ctx, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value).cmp(&other)
    }
}

impl<K, V, P> Hash for KeyValueMutPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K> + Hash,
    for<'ctx> MutPtrs<'ctx, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        (key, value).hash(state);
    }
}

impl<K, V, P> Clone for KeyValueMutPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;

        let value = value.clone();
        Self { key, value }
    }
}

impl<K, V, P> Copy for KeyValueMutPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx> MutPtrs<'ctx, V>: Copy,
{
}
