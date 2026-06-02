use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem, ptr,
};

use gpecs_ptr::slice::{CastConst, MutSliceItemPtr};
use gpecs_soa::{
    traits::{RawSoa, RawSoaContext, SliceMutPtrs, Soa, SoaContext},
    wrapper,
};

use crate::{KeyValueMutPtrs, KeyValueMutSlices, KeyValuePtrs, KeyValueSlicePtrs, KeyValueSlices};

pub struct KeyValueMutSlicePtrs<'ctx, K, V, P = *mut K>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    key: P,
    len: usize,
    values: wrapper::SliceMutPtrs<'ctx, V>,
}

impl<'ctx, K, V, P> KeyValueMutSlicePtrs<'ctx, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    #[inline]
    #[track_caller]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(context: &'ctx V::Context, keys: *mut [K], values: SliceMutPtrs<'ctx, V>) -> Self {
        let keys_len = keys.len();
        let values_len = context.mut_slice_ptrs_len(&values);
        assert_eq!(keys_len, values_len);

        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn new_unchecked(keys: *mut [K], values: SliceMutPtrs<'ctx, V>) -> Self {
        let key = unsafe { P::from_slice(keys, 0) };
        let len = keys.len();
        unsafe { Self::from_parts(key, len, values) }
    }

    #[inline]
    pub fn from_ptrs(
        context: &'ctx V::Context,
        ptrs: KeyValueMutPtrs<'ctx, K, V, P>,
        len: usize,
    ) -> Self {
        let (key, value) = ptrs.into_parts();
        let values = context.mut_slice_ptrs_from_raw_parts(value, len);
        unsafe { Self::from_parts(key, len, values) }
    }

    #[inline]
    pub unsafe fn from_parts(key: P, len: usize, values: SliceMutPtrs<'ctx, V>) -> Self {
        let values = wrapper::SliceMutPtrs::new(values);
        Self { key, len, values }
    }

    #[inline]
    pub fn into_parts(self) -> (*mut [K], SliceMutPtrs<'ctx, V>) {
        let Self { key, len, values } = self;

        let keys = ptr::slice_from_raw_parts_mut(key.as_mut_raw_ptr(), len);
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

    #[inline]
    pub fn cast_const(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueSlicePtrs<'ctx, K, V, CastConst<P>> {
        let Self { key, len, values } = self;

        let key = key.cast_const();
        let values = context.slice_ptrs_cast_const(values.into_inner());
        unsafe { KeyValueSlicePtrs::from_parts(key, len, values) }
    }

    #[inline]
    pub fn into_ptrs(self, context: &'ctx V::Context) -> KeyValuePtrs<'ctx, K, V, CastConst<P>> {
        let Self { key, values, .. } = self;

        let key = key.cast_const();
        let values = context.slice_ptrs_cast_const(values.into_inner());
        let value = context.slice_ptrs_as_ptrs(values);
        KeyValuePtrs::new(key, value)
    }

    #[inline]
    pub fn into_mut_ptrs(self, context: &'ctx V::Context) -> KeyValueMutPtrs<'ctx, K, V, P> {
        let Self { key, values, .. } = self;

        let value = context.mut_slice_ptrs_as_ptrs(values.into_inner());
        KeyValueMutPtrs::new(key, value)
    }

    #[inline]
    pub unsafe fn drop_in_place(self, context: &V::Context) {
        let Self { key, len, values } = self;

        if mem::needs_drop::<K>() {
            for i in 0..len {
                unsafe { key.add(i).drop_in_place() }
            }
        }
        unsafe { context.slices_drop_in_place(values.into_inner()) }
    }
}

impl<'ctx, 'a, K, V, P> KeyValueMutSlicePtrs<'ctx, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    #[inline]
    pub unsafe fn as_ref_unchecked(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueSlices<'ctx, 'a, K, V, CastConst<P>> {
        unsafe { self.cast_const(context).as_ref_unchecked(context) }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueMutSlices<'ctx, 'a, K, V, P> {
        let Self { key, len, values } = self;

        let values = unsafe { context.mut_slice_ptrs_to_mut_slices(values.into_inner()) };
        unsafe { KeyValueMutSlices::from_parts(key, len, values) }
    }
}

impl<'ctx, K, V, P> From<KeyValueMutSlicePtrs<'ctx, K, V, P>> for (*mut [K], SliceMutPtrs<'ctx, V>)
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    #[inline]
    fn from(value: KeyValueMutSlicePtrs<'ctx, K, V, P>) -> Self {
        value.into_parts()
    }
}

impl<K, V, P> Debug for KeyValueMutSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx> SliceMutPtrs<'ctx, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = ptr::slice_from_raw_parts_mut(key.as_mut_raw_ptr(), len);
        f.debug_struct("KeyValueMutSlicePtrs")
            .field("keys", &keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V, P> PartialEq for KeyValueMutSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K> + PartialEq,
    for<'ctx> SliceMutPtrs<'ctx, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, len, values } = self;

        let other = (&other.key, &other.len, &other.values);
        (key, len, values) == other
    }
}

impl<K, V, P> Eq for KeyValueMutSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K> + Eq,
    for<'ctx> SliceMutPtrs<'ctx, V>: Eq,
{
}

impl<K, V, P> PartialOrd for KeyValueMutSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K> + PartialOrd,
    for<'ctx> SliceMutPtrs<'ctx, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, len, values } = self;

        let other = (&other.key, &other.len, &other.values);
        (key, len, values).partial_cmp(&other)
    }
}

impl<K, V, P> Ord for KeyValueMutSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K> + Ord,
    for<'ctx> SliceMutPtrs<'ctx, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, len, values } = self;

        let other = (&other.key, &other.len, &other.values);
        (key, len, values).cmp(&other)
    }
}

impl<K, V, P> Hash for KeyValueMutSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K> + Hash,
    for<'ctx> SliceMutPtrs<'ctx, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, len, values } = self;
        (key, len, values).hash(state);
    }
}

impl<K, V, P> Clone for KeyValueMutSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K>,
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

impl<K, V, P> Copy for KeyValueMutSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx> SliceMutPtrs<'ctx, V>: Copy,
{
}
