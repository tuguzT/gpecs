use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr::NonNull,
};

use gpecs_ptr::slice::{
    CastConst, MutSliceItemPtr, NonNullAsPtr, NonNullSliceItemPtr, SliceItemPtr,
};
use gpecs_soa::{
    traits::{MutPtrs, NonNullPtrs, RawSoa, RawSoaContext},
    wrapper,
};

use crate::{KeyValueMutPtrs, KeyValuePtrs};

pub struct KeyValueNonNullPtrs<'ctx, K, V, P = NonNull<K>>
where
    V: RawSoa + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
{
    key: P,
    value: wrapper::NonNullPtrs<'ctx, V>,
}

impl<'ctx, K, V, P> KeyValueNonNullPtrs<'ctx, K, V, P>
where
    V: RawSoa + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
{
    #[inline]
    pub fn new(key: P, value: NonNullPtrs<'ctx, V>) -> Self {
        let value = wrapper::NonNullPtrs::new(value);
        Self { key, value }
    }

    #[inline]
    pub unsafe fn new_unchecked(
        context: &'ctx V::Context,
        key: NonNullAsPtr<P>,
        value: MutPtrs<'ctx, V>,
    ) -> Self {
        let key = unsafe { P::from_slice(NonNull::new_unchecked(key.slice()), key.index()) };
        let value = unsafe { context.ptrs_to_nonnull(value) };
        Self::new(key, value)
    }

    #[inline]
    pub fn into_parts(self) -> (P, NonNullPtrs<'ctx, V>) {
        let Self { key, value } = self;
        (key, value.into_inner())
    }

    #[inline]
    pub fn into_ptrs(
        self,
        context: &'ctx V::Context,
    ) -> KeyValuePtrs<'ctx, K, V, CastConst<NonNullAsPtr<P>>> {
        let (key, value) = self.into_parts();

        let key = key.as_ptr().cast_const();
        let value = context.nonnull_to_ptrs(value);
        let value = context.ptrs_cast_const(value);
        KeyValuePtrs::new(key, value)
    }

    #[inline]
    pub fn into_mut_ptrs(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueMutPtrs<'ctx, K, V, NonNullAsPtr<P>> {
        let (key, value) = self.into_parts();

        let key = key.as_ptr();
        let value = context.nonnull_to_ptrs(value);
        KeyValueMutPtrs::new(key, value)
    }
}

impl<'ctx, K, V, P> From<(P, NonNullPtrs<'ctx, V>)> for KeyValueNonNullPtrs<'ctx, K, V, P>
where
    V: RawSoa + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
{
    #[inline]
    fn from(value: (P, NonNullPtrs<'ctx, V>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'ctx, K, V, P> From<KeyValueNonNullPtrs<'ctx, K, V, P>> for (P, NonNullPtrs<'ctx, V>)
where
    V: RawSoa + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
{
    #[inline]
    fn from(value: KeyValueNonNullPtrs<'ctx, K, V, P>) -> Self {
        value.into_parts()
    }
}

impl<K, V, P> Debug for KeyValueNonNullPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: NonNullSliceItemPtr<Item = K> + Debug,
    for<'ctx> NonNullPtrs<'ctx, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;

        f.debug_struct("KeyValueNonNullPtrs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<K, V, P> PartialEq for KeyValueNonNullPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: NonNullSliceItemPtr<Item = K> + PartialEq,
    for<'ctx> NonNullPtrs<'ctx, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value) == other
    }
}

impl<K, V, P> Eq for KeyValueNonNullPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: NonNullSliceItemPtr<Item = K> + Eq,
    for<'ctx> NonNullPtrs<'ctx, V>: Eq,
{
}

impl<K, V, P> PartialOrd for KeyValueNonNullPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: NonNullSliceItemPtr<Item = K> + PartialOrd,
    for<'ctx> NonNullPtrs<'ctx, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value).partial_cmp(&other)
    }
}

impl<K, V, P> Ord for KeyValueNonNullPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: NonNullSliceItemPtr<Item = K> + Ord,
    for<'ctx> NonNullPtrs<'ctx, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value } = self;

        let other = (&other.key, &other.value);
        (key, value).cmp(&other)
    }
}

impl<K, V, P> Hash for KeyValueNonNullPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: NonNullSliceItemPtr<Item = K> + Hash,
    for<'ctx> NonNullPtrs<'ctx, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;
        (key, value).hash(state);
    }
}

impl<K, V, P> Clone for KeyValueNonNullPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;

        let value = value.clone();
        Self { key, value }
    }
}

impl<K, V, P> Copy for KeyValueNonNullPtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
    for<'ctx> NonNullPtrs<'ctx, V>: Copy,
{
}
