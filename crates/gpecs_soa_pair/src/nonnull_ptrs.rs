use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr::NonNull,
};

use gpecs_ptr::slice::{
    ConstSliceItemPtr, MutSliceItemPtr, NonNullAsMutPtr, NonNullAsPtr, NonNullSliceItemPtr,
    SliceItemPtr,
};
use gpecs_soa::{
    traits::{
        NonNullPtrs, SoaAllocContext, SoaCloneToUninit, SoaCloneToUninitContext, SoaRaw,
        SoaRawContext, SoaRead, SoaReadContext, SoaWrite, SoaWriteContext,
    },
    wrapper,
};

use crate::{KeyValueMutPtrs, KeyValuePair, KeyValuePtrs};

pub struct KeyValueNonNullPtrs<'ctx, K, V, P = NonNull<K>>
where
    V: SoaRaw + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
{
    key: P,
    value: wrapper::NonNullPtrs<'ctx, V>,
}

impl<'ctx, K, V, P> KeyValueNonNullPtrs<'ctx, K, V, P>
where
    V: SoaRaw + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
{
    #[inline]
    pub fn new(key: P, value: NonNullPtrs<'ctx, V>) -> Self {
        let value = wrapper::NonNullPtrs::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn dangling(context: &'ctx V::Context) -> Self {
        let key = P::dangling();
        let value = context.nonnull_ptrs_dangling();
        Self::new(key, value)
    }

    #[inline]
    pub unsafe fn from_ptrs(
        context: &'ctx V::Context,
        ptrs: KeyValuePtrs<'ctx, K, V, NonNullAsPtr<P>>,
    ) -> Self {
        let (key, value) = ptrs.into_parts();
        let key = unsafe {
            let slice = NonNull::new_unchecked(key.slice().cast_mut());
            P::from_slice(slice, key.index())
        };
        let value = unsafe { context.nonnull_ptrs_from_ptrs(value) };
        Self::new(key, value)
    }

    #[inline]
    pub unsafe fn from_mut_ptrs(
        context: &'ctx V::Context,
        ptrs: KeyValueMutPtrs<'ctx, K, V, NonNullAsMutPtr<P>>,
    ) -> Self {
        let (key, value) = ptrs.into_parts();
        let key = unsafe {
            let slice = NonNull::new_unchecked(key.slice());
            P::from_slice(slice, key.index())
        };
        let value = unsafe { context.nonnull_ptrs_from_mut_ptrs(value) };
        Self::new(key, value)
    }

    #[inline]
    pub fn into_parts(self) -> (P, NonNullPtrs<'ctx, V>) {
        let Self { key, value } = self;
        (key, value.into_inner())
    }

    #[inline]
    pub fn into_ptrs(self, context: &'ctx V::Context) -> KeyValuePtrs<'ctx, K, V, NonNullAsPtr<P>> {
        let (key, value) = self.into_parts();

        let key = key.as_ptr();
        let value = context.nonnull_ptrs_as_ptrs(value);
        KeyValuePtrs::new(key, value)
    }

    #[inline]
    pub fn into_mut_ptrs(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueMutPtrs<'ctx, K, V, NonNullAsMutPtr<P>> {
        let (key, value) = self.into_parts();

        let key = key.as_mut_ptr();
        let value = context.nonnull_ptrs_as_mut_ptrs(value);
        KeyValueMutPtrs::new(key, value)
    }
    #[inline]
    #[must_use]
    pub unsafe fn add(self, context: &'ctx V::Context, count: usize) -> Self {
        let (key, value) = self.into_parts();

        let key = unsafe { key.add(count) };
        let value = unsafe { context.nonnull_ptrs_add(value, count) };
        Self::new(key, value)
    }

    #[inline]
    pub unsafe fn offset_from(
        self,
        context: &V::Context,
        origin: KeyValueNonNullPtrs<'_, K, V, P>,
    ) -> isize {
        let (key, value) = self.into_parts();
        let (origin_key, origin_value) = origin.into_parts();

        let key_offset = unsafe { key.offset_from(origin_key) };
        let value_offset = unsafe { context.nonnull_ptrs_offset_from(value, origin_value) };
        assert_eq!(key_offset, value_offset);

        key_offset
    }

    #[inline]
    pub unsafe fn swap_nonoverlapping(
        self,
        context: &V::Context,
        with: KeyValueNonNullPtrs<'_, K, V, P>,
        count: usize,
    ) {
        let (key, value) = self.into_parts();
        let (with_key, with_value) = with.into_parts();

        unsafe {
            key.swap_nonoverlapping(with_key, count);
            context.nonnull_ptrs_swap_nonoverlapping(value, with_value, count);
        }
    }

    #[inline]
    pub unsafe fn copy_from_nonoverlapping(
        self,
        context: &V::Context,
        from: KeyValueNonNullPtrs<'_, K, V, P>,
        count: usize,
    ) {
        let (dst_key, dst_value) = self.into_parts();
        let (src_key, src_value) = from.into_parts();

        unsafe {
            dst_key.copy_from_nonoverlapping(src_key, count);
            context.nonnull_ptrs_copy_nonoverlapping(src_value, dst_value, count);
        }
    }

    #[inline]
    pub unsafe fn drop_in_place(self, context: &V::Context) {
        let (key, value) = self.into_parts();

        unsafe {
            key.drop_in_place();
            context.nonnull_ptrs_drop_in_place(value);
        }
    }

    #[inline]
    pub unsafe fn read<R>(self, context: &'ctx V::Context) -> KeyValuePair<K, R, P::Ptrs>
    where
        V: SoaRead<'ctx, R>,
    {
        let (key, value) = self.into_parts();

        let key = unsafe { key.read() };
        let value = unsafe { context.nonnull_ptrs_read(value) };
        KeyValuePair::new(key, value)
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
            context.nonnull_ptrs_write(value_ptr, value);
        }
    }
}

impl<K, V, P> KeyValueNonNullPtrs<'_, K, V, P>
where
    K: Clone,
    V: SoaCloneToUninit + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
{
    #[inline]
    pub unsafe fn clone_to_uninit(
        self,
        context: &V::Context,
        dst: KeyValueNonNullPtrs<'_, K, V, P>,
    ) {
        let (key, value) = self.into_parts();
        let (dst_key, dst_value) = dst.into_parts();

        unsafe {
            dst_key.write(key.as_ref().clone());
            context.nonnull_ptrs_clone_to_uninit(value, dst_value);
        }
    }
}

impl<K, V, P> KeyValueNonNullPtrs<'_, K, V, P>
where
    V: SoaRaw<Context: SoaAllocContext<V>> + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
{
    #[inline]
    pub unsafe fn copy_from_forward(
        self,
        context: &V::Context,
        from: KeyValueNonNullPtrs<'_, K, V, P>,
        count: usize,
    ) {
        let (dst_key, dst_value) = self.into_parts();
        let (src_key, src_value) = from.into_parts();

        unsafe {
            dst_key.copy_from(src_key, count);
            context.nonnull_ptrs_copy_forward(src_value, dst_value, count);
        }
    }

    #[inline]
    pub unsafe fn copy_from_backward(
        self,
        context: &V::Context,
        from: KeyValueNonNullPtrs<'_, K, V, P>,
        count: usize,
    ) {
        let (dst_key, dst_value) = self.into_parts();
        let (src_key, src_value) = from.into_parts();

        unsafe {
            context.nonnull_ptrs_copy_backward(src_value, dst_value, count);
            dst_key.copy_from(src_key, count);
        }
    }
}

impl<K, V, P> Debug for KeyValueNonNullPtrs<'_, K, V, P>
where
    V: SoaRaw + ?Sized,
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
    V: SoaRaw + ?Sized,
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
    V: SoaRaw + ?Sized,
    P: NonNullSliceItemPtr<Item = K> + Eq,
    for<'ctx> NonNullPtrs<'ctx, V>: Eq,
{
}

impl<K, V, P> PartialOrd for KeyValueNonNullPtrs<'_, K, V, P>
where
    V: SoaRaw + ?Sized,
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
    V: SoaRaw + ?Sized,
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
    V: SoaRaw + ?Sized,
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
    V: SoaRaw + ?Sized,
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
    V: SoaRaw + ?Sized,
    P: NonNullSliceItemPtr<Item = K>,
    for<'ctx> NonNullPtrs<'ctx, V>: Copy,
{
}
