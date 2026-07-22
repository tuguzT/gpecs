use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use gpecs_ptr::slice::{CastMut, ConstSliceItemPtr, MutSliceItemPtr};
use gpecs_soa::{
    traits::{
        CloneToUninitSoaContext, Ptrs, RawSoa, RawSoaContext, ReadSoaContext, Soa,
        SoaCloneToUninit, SoaContext, SoaRead,
    },
    wrapper,
};

use crate::{KeyValueMutPtrs, KeyValuePair, KeyValueRefs};

pub struct KeyValuePtrs<'ctx, K, V, P = *const K>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    key: P,
    value: wrapper::Ptrs<'ctx, V>,
}

impl<'ctx, K, V, P> KeyValuePtrs<'ctx, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    #[inline]
    pub fn new(key: P, value: Ptrs<'ctx, V>) -> Self {
        let value = wrapper::Ptrs::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn dangling(context: &'ctx V::Context) -> Self {
        let key = P::dangling();
        let value = context.ptrs_dangling();
        Self::new(key, value)
    }

    #[inline]
    pub fn into_parts(self) -> (P, Ptrs<'ctx, V>) {
        let Self { key, value } = self;
        (key, value.into_inner())
    }

    #[inline]
    pub fn cast_mut(self, context: &'ctx V::Context) -> KeyValueMutPtrs<'ctx, K, V, CastMut<P>> {
        let (key, value) = self.into_parts();

        let key = key.cast_mut();
        let value = context.ptrs_cast_mut(value);
        KeyValueMutPtrs::new(key, value)
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, context: &'ctx V::Context, count: usize) -> Self {
        let (key, value) = self.into_parts();

        let key = unsafe { key.add(count) };
        let value = unsafe { context.ptrs_add(value, count) };
        Self::new(key, value)
    }

    #[inline]
    pub unsafe fn offset_from(
        self,
        context: &V::Context,
        origin: KeyValuePtrs<'_, K, V, P>,
    ) -> isize {
        let (key, value) = self.into_parts();
        let (origin_key, origin_value) = origin.into_parts();

        let key_offset = unsafe { key.offset_from(origin_key) };
        let values_offset = unsafe { context.ptrs_offset_from(value, origin_value) };
        assert_eq!(key_offset, values_offset);

        key_offset
    }

    #[inline]
    pub unsafe fn read<R>(self, context: &'ctx V::Context) -> KeyValuePair<K, R, P::Ptrs>
    where
        V: SoaRead<'ctx, R>,
    {
        let (key, value) = self.into_parts();

        let key = unsafe { key.read() };
        let value = unsafe { context.read(value) };
        KeyValuePair::new(key, value)
    }
}

impl<K, V, P> KeyValuePtrs<'_, K, V, P>
where
    K: Clone,
    V: SoaCloneToUninit + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    #[inline]
    pub unsafe fn clone_to_uninit(
        self,
        context: &V::Context,
        dst: KeyValueMutPtrs<'_, K, V, CastMut<P>>,
    ) {
        let (key, value) = self.into_parts();
        let (dst_key, dst_value) = dst.into_parts();

        unsafe {
            dst_key.write(key.as_ref_unchecked().clone());
            context.clone_to_uninit(value, dst_value);
        }
    }
}

impl<'ctx, 'a, K, V, P> KeyValuePtrs<'ctx, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    #[inline]
    pub unsafe fn as_ref_unchecked(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueRefs<'ctx, 'a, K, V, P> {
        let (key, value) = self.into_parts();

        let value = unsafe { context.ptrs_to_refs(value) };
        unsafe { KeyValueRefs::from_parts(key, value) }
    }
}

impl<'ctx, K, V, P> From<(P, Ptrs<'ctx, V>)> for KeyValuePtrs<'ctx, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    #[inline]
    fn from(value: (P, Ptrs<'ctx, V>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'ctx, K, V, P> From<KeyValuePtrs<'ctx, K, V, P>> for (P, Ptrs<'ctx, V>)
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    #[inline]
    fn from(value: KeyValuePtrs<'ctx, K, V, P>) -> Self {
        value.into_parts()
    }
}

impl<K, V, P> Debug for KeyValuePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K> + Debug,
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

impl<K, V, P> PartialEq for KeyValuePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K> + PartialEq,
    for<'ctx> Ptrs<'ctx, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value) == other
    }
}

impl<K, V, P> Eq for KeyValuePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K> + Eq,
    for<'ctx> Ptrs<'ctx, V>: Eq,
{
}

impl<K, V, P> PartialOrd for KeyValuePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K> + PartialOrd,
    for<'ctx> Ptrs<'ctx, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value).partial_cmp(&other)
    }
}

impl<K, V, P> Ord for KeyValuePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K> + Ord,
    for<'ctx> Ptrs<'ctx, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value).cmp(&other)
    }
}

impl<K, V, P> Hash for KeyValuePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K> + Hash,
    for<'ctx> Ptrs<'ctx, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        (key, value).hash(state);
    }
}

impl<K, V, P> Clone for KeyValuePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;

        let value = value.clone();
        Self { key, value }
    }
}

impl<K, V, P> Copy for KeyValuePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx> Ptrs<'ctx, V>: Copy,
{
}
