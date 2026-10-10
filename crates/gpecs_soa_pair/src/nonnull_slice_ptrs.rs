use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr::NonNull,
};

use gpecs_ptr::slice::NonNullSliceItemPtr;
use gpecs_soa::{
    traits::{SliceNonNullPtrs, SoaRaw, SoaRawContext},
    wrapper,
};

use crate::KeyValueNonNullPtrs;

pub struct KeyValueNonNullSlicePtrs<'ctx, K, V, P = NonNull<K>>
where
    V: SoaRaw + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
{
    key: P,
    len: usize,
    values: wrapper::SliceNonNullPtrs<'ctx, V>,
}

impl<'ctx, K, V, P> KeyValueNonNullSlicePtrs<'ctx, K, V, P>
where
    V: SoaRaw + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
{
    #[inline]
    #[track_caller]
    pub fn new(
        context: &'ctx V::Context,
        keys: NonNull<[K]>,
        values: SliceNonNullPtrs<'ctx, V>,
    ) -> Self {
        let keys_len = keys.len();
        let values_len = context.nonnull_slice_ptrs_len(&values);
        assert_eq!(keys_len, values_len);

        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn new_unchecked(keys: NonNull<[K]>, values: SliceNonNullPtrs<'ctx, V>) -> Self {
        let key = unsafe { P::from_slice(keys, 0) };
        let len = keys.len();
        unsafe { Self::from_parts(key, len, values) }
    }

    #[inline]
    pub fn from_ptrs(
        context: &'ctx V::Context,
        ptrs: KeyValueNonNullPtrs<'ctx, K, V, P>,
        len: usize,
    ) -> Self {
        let (key, value) = ptrs.into_parts();
        let values = context.nonnull_slice_ptrs_from_raw_parts(value, len);
        unsafe { Self::from_parts(key, len, values) }
    }

    #[inline]
    pub unsafe fn from_parts(key: P, len: usize, values: SliceNonNullPtrs<'ctx, V>) -> Self {
        let values = wrapper::SliceNonNullPtrs::new(values);
        Self { key, len, values }
    }

    #[inline]
    pub fn into_parts(self) -> (NonNull<[K]>, SliceNonNullPtrs<'ctx, V>) {
        let Self { key, len, values } = self;

        let keys = NonNull::slice_from_raw_parts(key.as_raw_ptr(), len);
        (keys, values.into_inner())
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V, P> Debug for KeyValueNonNullSlicePtrs<'_, K, V, P>
where
    V: SoaRaw + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
    for<'ctx> SliceNonNullPtrs<'ctx, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = NonNull::slice_from_raw_parts(key.as_raw_ptr(), len);
        f.debug_struct("KeyValueNonNullSlicePtrs")
            .field("keys", &keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V, P> PartialEq for KeyValueNonNullSlicePtrs<'_, K, V, P>
where
    V: SoaRaw + ?Sized,
    P: NonNullSliceItemPtr<Item = K> + PartialEq,
    for<'ctx> SliceNonNullPtrs<'ctx, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, len, values } = self;

        let other = (&other.key, &other.len, &other.values);
        (key, len, values) == other
    }
}

impl<K, V, P> Eq for KeyValueNonNullSlicePtrs<'_, K, V, P>
where
    V: SoaRaw + ?Sized,
    P: NonNullSliceItemPtr<Item = K> + Eq,
    for<'ctx> SliceNonNullPtrs<'ctx, V>: Eq,
{
}

impl<K, V, P> PartialOrd for KeyValueNonNullSlicePtrs<'_, K, V, P>
where
    V: SoaRaw + ?Sized,
    P: NonNullSliceItemPtr<Item = K> + PartialOrd,
    for<'ctx> SliceNonNullPtrs<'ctx, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, len, values } = self;

        let other = (&other.key, &other.len, &other.values);
        (key, len, values).partial_cmp(&other)
    }
}

impl<K, V, P> Ord for KeyValueNonNullSlicePtrs<'_, K, V, P>
where
    V: SoaRaw + ?Sized,
    P: NonNullSliceItemPtr<Item = K> + Ord,
    for<'ctx> SliceNonNullPtrs<'ctx, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, len, values } = self;

        let other = (&other.key, &other.len, &other.values);
        (key, len, values).cmp(&other)
    }
}

impl<K, V, P> Hash for KeyValueNonNullSlicePtrs<'_, K, V, P>
where
    V: SoaRaw + ?Sized,
    P: NonNullSliceItemPtr<Item = K> + Hash,
    for<'ctx> SliceNonNullPtrs<'ctx, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, len, values } = self;
        (key, len, values).hash(state);
    }
}

impl<K, V, P> Clone for KeyValueNonNullSlicePtrs<'_, K, V, P>
where
    V: SoaRaw + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let values = values.clone();
        Self { key, len, values }
    }
}

impl<K, V, P> Copy for KeyValueNonNullSlicePtrs<'_, K, V, P>
where
    V: SoaRaw + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
    for<'ctx> SliceNonNullPtrs<'ctx, V>: Copy,
{
}
