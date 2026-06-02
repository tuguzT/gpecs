use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    slice,
};

use gpecs_ptr::slice::{CastConst, MutSliceItemPtr};
use gpecs_soa::{
    traits::{SlicesMut, Soa, SoaContext, SoaOwned},
    wrapper,
};

use crate::{KeyValueMutSlicePtrs, KeyValueSlicePtrs, KeyValueSlices};

pub struct KeyValueMutSlices<'ctx, 'a, K, V, P = *mut K>
where
    K: 'a,
    V: Soa<'a> + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    key: P,
    len: usize,
    values: wrapper::SlicesMut<'ctx, 'a, V>,
}

impl<'ctx, 'a, K, V, P> KeyValueMutSlices<'ctx, 'a, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: MutSliceItemPtr<Item = K>,
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

        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn new_unchecked(keys: &'a mut [K], values: SlicesMut<'ctx, 'a, V>) -> Self {
        let key = unsafe { P::from_slice(keys, 0) };
        let len = keys.len();
        unsafe { Self::from_parts(key, len, values) }
    }

    #[inline]
    pub unsafe fn from_parts(key: P, len: usize, values: SlicesMut<'ctx, 'a, V>) -> Self {
        let values = wrapper::SlicesMut::new(values);
        Self { key, len, values }
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
    pub fn into_parts(self) -> (&'a mut [K], SlicesMut<'ctx, 'a, V>) {
        let Self { key, len, values } = self;

        let keys = unsafe { slice::from_raw_parts_mut(key.as_mut_raw_ptr(), len) };
        (keys, values.into_inner())
    }

    #[inline]
    pub fn into_slice_ptrs(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueSlicePtrs<'ctx, K, V, CastConst<P>> {
        let Self { key, len, values } = self;

        let key = key.cast_const();
        let values = context.mut_slices_as_slices(values.into_inner());
        let values = context.slices_as_slice_ptrs(values);
        unsafe { KeyValueSlicePtrs::from_parts(key, len, values) }
    }

    #[inline]
    pub fn into_mut_slice_ptrs(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueMutSlicePtrs<'ctx, K, V, P> {
        let Self { key, len, values } = self;

        let values = context.mut_slices_as_mut_slice_ptrs(values.into_inner());
        unsafe { KeyValueMutSlicePtrs::from_parts(key, len, values) }
    }

    #[inline]
    pub fn into_slices(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueSlices<'ctx, 'a, K, V, CastConst<P>> {
        let Self { key, len, values } = self;

        let key = key.cast_const();
        let values = context.mut_slices_as_slices(values.into_inner());
        unsafe { KeyValueSlices::from_parts(key, len, values) }
    }
}

impl<'ctx, 'a, K, V, P> From<KeyValueMutSlices<'ctx, 'a, K, V, P>>
    for (&'a mut [K], SlicesMut<'ctx, 'a, V>)
where
    V: Soa<'a> + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    #[inline]
    fn from(value: KeyValueMutSlices<'ctx, 'a, K, V, P>) -> Self {
        value.into_parts()
    }
}

impl<K, V, P> Debug for KeyValueMutSlices<'_, '_, K, V, P>
where
    K: Debug,
    V: SoaOwned + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { slice::from_raw_parts(key.as_mut_raw_ptr(), len) };
        f.debug_struct("KeyValueMutSlices")
            .field("keys", &keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V, P> Default for KeyValueMutSlices<'_, '_, K, V, P>
where
    V: SoaOwned + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Default,
{
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = SlicesMut::<V>::default();
        unsafe { Self::new_unchecked(keys, values) }
    }
}

impl<K, V, P> PartialEq for KeyValueMutSlices<'_, '_, K, V, P>
where
    K: PartialEq,
    V: SoaOwned + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { slice::from_raw_parts(key.as_mut_raw_ptr(), len) };
        let other_keys = unsafe { slice::from_raw_parts(other.key.as_mut_raw_ptr(), other.len) };

        let other = (other_keys, &other.values);
        (keys, values) == other
    }
}

impl<K, V, P> Eq for KeyValueMutSlices<'_, '_, K, V, P>
where
    K: Eq,
    V: SoaOwned + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Eq,
{
}

impl<K, V, P> PartialOrd for KeyValueMutSlices<'_, '_, K, V, P>
where
    K: PartialOrd,
    V: SoaOwned + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { slice::from_raw_parts(key.as_mut_raw_ptr(), len) };
        let other_keys = unsafe { slice::from_raw_parts(other.key.as_mut_raw_ptr(), other.len) };

        let other = (other_keys, &other.values);
        (keys, values).partial_cmp(&other)
    }
}

impl<K, V, P> Ord for KeyValueMutSlices<'_, '_, K, V, P>
where
    K: Ord,
    V: SoaOwned + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { slice::from_raw_parts(key.as_mut_raw_ptr(), len) };
        let other_keys = unsafe { slice::from_raw_parts(other.key.as_mut_raw_ptr(), other.len) };

        let other = (other_keys, &other.values);
        (keys, values).cmp(&other)
    }
}

impl<K, V, P> Hash for KeyValueMutSlices<'_, '_, K, V, P>
where
    K: Hash,
    V: SoaOwned + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx, 'a> SlicesMut<'ctx, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { slice::from_raw_parts(key.as_mut_raw_ptr(), len) };
        (keys, values).hash(state);
    }
}
