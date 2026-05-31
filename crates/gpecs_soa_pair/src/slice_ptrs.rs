use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr, slice,
};

use gpecs_ptr::slice::{CastMut, ConstSliceItemPtr};
use gpecs_soa::{
    traits::{RawSoa, RawSoaContext, SlicePtrs, Soa, SoaContext},
    wrapper,
};

use crate::{KeyValueMutPtrs, KeyValueMutSlicePtrs, KeyValuePtrs, KeyValueSlices};

pub struct KeyValueSlicePtrs<'ctx, K, V, P = *const K>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    key: P,
    len: usize,
    values: wrapper::SlicePtrs<'ctx, V>,
}

impl<'ctx, K, V, P> KeyValueSlicePtrs<'ctx, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    #[inline]
    #[track_caller]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(context: &'ctx V::Context, keys: *const [K], values: SlicePtrs<'ctx, V>) -> Self {
        let keys_len = keys.len();
        let values_len = context.slice_ptrs_len(&values);
        assert_eq!(keys_len, values_len);

        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn new_unchecked(keys: *const [K], values: SlicePtrs<'ctx, V>) -> Self {
        let key = unsafe { P::from_slice(keys, 0) };
        let len = keys.len();
        let values = wrapper::SlicePtrs::new(values);
        Self { key, len, values }
    }

    #[inline]
    pub fn from_parts(
        context: &'ctx V::Context,
        ptrs: KeyValuePtrs<'ctx, K, V, P>,
        len: usize,
    ) -> Self {
        let (key, value) = ptrs.into_parts();

        let values = context.slice_ptrs_from_raw_parts(value, len);
        let values = wrapper::SlicePtrs::new(values);
        Self { key, len, values }
    }

    #[inline]
    pub fn into_parts(self) -> (*const [K], SlicePtrs<'ctx, V>) {
        let Self { key, len, values } = self;

        let keys = ptr::slice_from_raw_parts(key.as_raw_ptr(), len);
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
    pub fn cast_mut(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueMutSlicePtrs<'ctx, K, V, CastMut<P>> {
        let Self { key, len, values } = self;

        let value = context.slice_ptrs_as_ptrs(values.into_inner());
        let value = context.ptrs_cast_mut(value);
        let ptrs = KeyValueMutPtrs::new(key.cast_mut(), value);
        KeyValueMutSlicePtrs::from_parts(context, ptrs, len)
    }

    #[inline]
    pub fn into_ptrs(self, context: &'ctx V::Context) -> KeyValuePtrs<'ctx, K, V, P> {
        let Self { key, values, .. } = self;

        let value = context.slice_ptrs_as_ptrs(values.into_inner());
        KeyValuePtrs::new(key, value)
    }
}

impl<'ctx, 'a, K, V, P> KeyValueSlicePtrs<'ctx, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    #[inline]
    pub unsafe fn as_ref_unchecked(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueSlices<'ctx, 'a, K, V, P> {
        let Self { key, len, values } = self;

        let keys = unsafe { slice::from_raw_parts(key.as_raw_ptr(), len) };
        let values = unsafe { context.slice_ptrs_to_slices(values.into_inner()) };
        unsafe { KeyValueSlices::new_unchecked(keys, values) }
    }
}

impl<'ctx, K, V, P> From<KeyValueSlicePtrs<'ctx, K, V, P>> for (*const [K], SlicePtrs<'ctx, V>)
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    #[inline]
    fn from(value: KeyValueSlicePtrs<'ctx, K, V, P>) -> Self {
        value.into_parts()
    }
}

impl<K, V, P> Debug for KeyValueSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx> SlicePtrs<'ctx, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = ptr::slice_from_raw_parts(key.as_raw_ptr(), len);
        f.debug_struct("KeyValueSlicePtrs")
            .field("keys", &keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V, P> PartialEq for KeyValueSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K> + PartialEq,
    for<'ctx> SlicePtrs<'ctx, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, len, values } = self;

        let other = (&other.key, &other.len, &other.values);
        (key, len, values) == other
    }
}

impl<K, V, P> Eq for KeyValueSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K> + Eq,
    for<'ctx> SlicePtrs<'ctx, V>: Eq,
{
}

impl<K, V, P> PartialOrd for KeyValueSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K> + PartialOrd,
    for<'ctx> SlicePtrs<'ctx, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, len, values } = self;

        let other = (&other.key, &other.len, &other.values);
        (key, len, values).partial_cmp(&other)
    }
}

impl<K, V, P> Ord for KeyValueSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K> + Ord,
    for<'ctx> SlicePtrs<'ctx, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, len, values } = self;

        let other = (&other.key, &other.len, &other.values);
        (key, len, values).cmp(&other)
    }
}

impl<K, V, P> Hash for KeyValueSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K> + Hash,
    for<'ctx> SlicePtrs<'ctx, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, len, values } = self;
        (key, len, values).hash(state);
    }
}

impl<K, V, P> Clone for KeyValueSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
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

impl<K, V, P> Copy for KeyValueSlicePtrs<'_, K, V, P>
where
    V: RawSoa + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx> SlicePtrs<'ctx, V>: Copy,
{
}
