use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr::NonNull,
};

use gpecs_ptr::slice::{ConstPtr, MutPtr, MutSlicePtrAtIndex, NonNullSlicePtrAtIndex};
use gpecs_soa::{
    traits::{SlicesMut, Soa, SoaContext, SoaOwned},
    wrapper,
};

use crate::{
    KeyValueMutPtrs, KeyValueMutSlicePtrs, KeyValueNonNullPtrs, KeyValueNonNullSlicePtrs,
    KeyValuePtrs, KeyValueSlicePtrs, KeyValueSlices,
};

pub struct KeyValueMutSlices<'ctx, 'a, K, V, P = *mut K>
where
    K: 'a,
    V: Soa<'a> + ?Sized,
    P: NonNullSlicePtrAtIndex<Item = K>,
{
    key: P,
    len: usize,
    values: wrapper::SlicesMut<'ctx, 'a, V>,
}

impl<'ctx, 'a, K, V, P> KeyValueMutSlices<'ctx, 'a, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: NonNullSlicePtrAtIndex<Item = K>,
{
    #[inline]
    #[track_caller]
    pub fn new(
        context: &'ctx V::Context,
        keys: &'a mut [K],
        values: SlicesMut<'ctx, 'a, V>,
    ) -> Self {
        let keys_len = keys.len();
        let values_len = context.mut_slices_len(&values);
        assert_eq!(keys_len, values_len);

        unsafe { Self::from_parts(keys, values) }
    }

    #[inline]
    pub unsafe fn from_parts(keys: &'a mut [K], values: SlicesMut<'ctx, 'a, V>) -> Self {
        let keys = NonNull::from_mut(keys);
        let key = unsafe { P::from_parts(keys, 0) };
        let len = keys.len();
        unsafe { Self::from_raw_parts(key, len, values) }
    }

    #[inline]
    pub unsafe fn from_raw_parts(key: P, len: usize, values: SlicesMut<'ctx, 'a, V>) -> Self {
        let values = wrapper::SlicesMut::new(values);
        Self { key, len, values }
    }

    #[inline]
    pub unsafe fn from_mut_ptrs(
        context: &'ctx V::Context,
        data: KeyValueMutPtrs<'ctx, K, V, MutPtr<P::Ptrs>>,
        len: usize,
    ) -> Self {
        let (key, values) = data.into_parts();
        let key = unsafe { key.cast_nonnull_unchecked() };
        let values = unsafe { context.mut_slices_from_raw_parts(values, len) };
        unsafe { Self::from_raw_parts(key, len, values) }
    }

    #[inline]
    pub unsafe fn from_nonnull_ptrs(
        context: &'ctx V::Context,
        data: KeyValueNonNullPtrs<'ctx, K, V, P>,
        len: usize,
    ) -> Self {
        let (key, values) = data.into_parts();
        let values = unsafe { context.mut_slices_from_parts(values, len) };
        unsafe { Self::from_raw_parts(key, len, values) }
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
    pub fn into_raw_parts(self) -> (P, usize, SlicesMut<'ctx, 'a, V>) {
        let Self { key, len, values } = self;
        (key, len, values.into_inner())
    }

    #[inline]
    pub fn into_parts(self) -> (&'a mut [K], SlicesMut<'ctx, 'a, V>) {
        let Self { key, len, values } = self;

        let keys = unsafe { Self::keys(key, len) };
        (keys, values.into_inner())
    }

    #[inline]
    pub fn into_slice_ptrs(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueSlicePtrs<'ctx, K, V, ConstPtr<P::Ptrs>> {
        let Self { key, len, values } = self;

        let key = key.cast_const();
        let values = context.mut_slices_as_slice_ptrs(values.into_inner());
        unsafe { KeyValueSlicePtrs::from_raw_parts(key, len, values) }
    }

    #[inline]
    pub fn into_mut_slice_ptrs(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueMutSlicePtrs<'ctx, K, V, MutPtr<P::Ptrs>> {
        let Self { key, len, values } = self;

        let key = key.cast_mut();
        let values = context.mut_slices_as_mut_slice_ptrs(values.into_inner());
        unsafe { KeyValueMutSlicePtrs::from_raw_parts(key, len, values) }
    }

    #[inline]
    pub fn into_nonnull_slice_ptrs(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueNonNullSlicePtrs<'ctx, K, V, P> {
        let Self { key, len, values } = self;

        let values = context.mut_slices_as_nonnull_slice_ptrs(values.into_inner());
        unsafe { KeyValueNonNullSlicePtrs::from_raw_parts(key, len, values) }
    }

    #[inline]
    pub fn into_ptrs(
        self,
        context: &'ctx V::Context,
    ) -> KeyValuePtrs<'ctx, K, V, ConstPtr<P::Ptrs>> {
        let Self { key, values, .. } = self;

        let key = key.cast_const();
        let values = context.mut_slices_as_ptrs(values.into_inner());
        KeyValuePtrs::new(key, values)
    }

    #[inline]
    pub fn into_mut_ptrs(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueMutPtrs<'ctx, K, V, MutPtr<P::Ptrs>> {
        let Self { key, values, .. } = self;

        let key = key.cast_mut();
        let values = context.mut_slices_as_mut_ptrs(values.into_inner());
        KeyValueMutPtrs::new(key, values)
    }

    #[inline]
    pub fn into_nonnull_ptrs(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueNonNullPtrs<'ctx, K, V, P> {
        let Self { key, values, .. } = self;

        let values = context.mut_slices_as_nonnull_ptrs(values.into_inner());
        KeyValueNonNullPtrs::new(key, values)
    }

    #[inline]
    pub fn into_slices(self, context: &'ctx V::Context) -> KeyValueSlices<'ctx, 'a, K, V, P> {
        let Self { key, len, values } = self;

        let values = context.mut_slices_as_slices(values.into_inner());
        unsafe { KeyValueSlices::from_raw_parts(key, len, values) }
    }

    unsafe fn keys<'b>(key: P, len: usize) -> &'b mut [K] {
        let mut keys = NonNull::slice_from_raw_parts(key.as_ptr(), len);
        unsafe { keys.as_mut() }
    }
}

impl<K, V, P> Debug for KeyValueMutSlices<'_, '_, K, V, P>
where
    K: Debug,
    V: SoaOwned + ?Sized,
    P: NonNullSlicePtrAtIndex<Item = K>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { Self::keys(key, len) };
        f.debug_struct("KeyValueMutSlices")
            .field("keys", &keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V, P> Default for KeyValueMutSlices<'_, '_, K, V, P>
where
    V: SoaOwned + ?Sized,
    P: NonNullSlicePtrAtIndex<Item = K>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Default,
{
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = SlicesMut::<V>::default();
        unsafe { Self::from_parts(keys, values) }
    }
}

impl<K, V, P> PartialEq for KeyValueMutSlices<'_, '_, K, V, P>
where
    K: PartialEq,
    V: SoaOwned + ?Sized,
    P: NonNullSlicePtrAtIndex<Item = K>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { Self::keys(key, len) };
        let other_keys = unsafe { Self::keys(other.key, other.len) };

        let other = (other_keys, &other.values);
        (keys, values) == other
    }
}

impl<K, V, P> Eq for KeyValueMutSlices<'_, '_, K, V, P>
where
    K: Eq,
    V: SoaOwned + ?Sized,
    P: NonNullSlicePtrAtIndex<Item = K>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Eq,
{
}

impl<K, V, P> PartialOrd for KeyValueMutSlices<'_, '_, K, V, P>
where
    K: PartialOrd,
    V: SoaOwned + ?Sized,
    P: NonNullSlicePtrAtIndex<Item = K>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { Self::keys(key, len) };
        let other_keys = unsafe { Self::keys(other.key, other.len) };

        let other = (other_keys, &other.values);
        (keys, values).partial_cmp(&other)
    }
}

impl<K, V, P> Ord for KeyValueMutSlices<'_, '_, K, V, P>
where
    K: Ord,
    V: SoaOwned + ?Sized,
    P: NonNullSlicePtrAtIndex<Item = K>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { Self::keys(key, len) };
        let other_keys = unsafe { Self::keys(other.key, other.len) };

        let other = (other_keys, &other.values);
        (keys, values).cmp(&other)
    }
}

impl<K, V, P> Hash for KeyValueMutSlices<'_, '_, K, V, P>
where
    K: Hash,
    V: SoaOwned + ?Sized,
    P: NonNullSlicePtrAtIndex<Item = K>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { Self::keys(key, len) };
        (keys, values).hash(state);
    }
}
