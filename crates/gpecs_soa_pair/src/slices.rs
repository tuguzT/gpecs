use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    ptr, slice,
};

use gpecs_ptr::slice::ConstSliceItemPtr;
use gpecs_soa::{
    traits::{Slices, Soa, SoaContext, SoaOwned},
    wrapper,
};

use crate::KeyValueSlicePtrs;

pub struct KeyValueSlices<'ctx, 'a, K, V, P = *const K>
where
    K: 'a,
    V: Soa<'a> + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    key: P,
    len: usize,
    values: wrapper::Slices<'ctx, 'a, V>,
}

impl<'ctx, 'a, K, V, P> KeyValueSlices<'ctx, 'a, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    #[inline]
    #[track_caller]
    pub fn new(context: &'ctx V::Context, keys: &'a [K], values: Slices<'ctx, 'a, V>) -> Self {
        let keys_len = keys.len();
        let values_len = context.slices_len(&values);
        assert_eq!(keys_len, values_len);

        unsafe { Self::new_unchecked(keys, values) }
    }

    #[inline]
    pub unsafe fn new_unchecked(keys: &'a [K], values: Slices<'ctx, 'a, V>) -> Self {
        let key = unsafe { P::from_slice(keys, 0) };
        let len = keys.len();
        let values = wrapper::Slices::new(values);
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
    pub fn into_parts(self) -> (&'a [K], Slices<'ctx, 'a, V>) {
        let Self { key, len, values } = self;

        let keys = unsafe { slice::from_raw_parts(key.as_raw_ptr(), len) };
        (keys, values.into_inner())
    }

    #[inline]
    pub fn into_slice_ptrs(self, context: &'ctx V::Context) -> KeyValueSlicePtrs<'ctx, K, V, P> {
        let Self { key, len, values } = self;

        let keys = ptr::slice_from_raw_parts(key.as_raw_ptr(), len);
        let values = context.slices_as_slice_ptrs(values.into_inner());
        unsafe { KeyValueSlicePtrs::new_unchecked(keys, values) }
    }
}

impl<'ctx, 'a, K, V, P> From<KeyValueSlices<'ctx, 'a, K, V, P>> for (&'a [K], Slices<'ctx, 'a, V>)
where
    V: Soa<'a> + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    #[inline]
    fn from(value: KeyValueSlices<'ctx, 'a, K, V, P>) -> Self {
        value.into_parts()
    }
}

impl<K, V, P> Debug for KeyValueSlices<'_, '_, K, V, P>
where
    K: Debug,
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { slice::from_raw_parts(key.as_raw_ptr(), len) };
        f.debug_struct("KeyValueSlices")
            .field("keys", &keys)
            .field("values", values)
            .finish()
    }
}

impl<K, V, P> Default for KeyValueSlices<'_, '_, K, V, P>
where
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Default,
{
    #[inline]
    fn default() -> Self {
        let keys = Default::default();
        let values = Slices::<V>::default();
        unsafe { Self::new_unchecked(keys, values) }
    }
}

impl<K, V, P> PartialEq for KeyValueSlices<'_, '_, K, V, P>
where
    K: PartialEq,
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { slice::from_raw_parts(key.as_raw_ptr(), len) };
        let other_keys = unsafe { slice::from_raw_parts(other.key.as_raw_ptr(), other.len) };

        let other = (other_keys, &other.values);
        (keys, values) == other
    }
}

impl<K, V, P> Eq for KeyValueSlices<'_, '_, K, V, P>
where
    K: Eq,
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Eq,
{
}

impl<K, V, P> PartialOrd for KeyValueSlices<'_, '_, K, V, P>
where
    K: PartialOrd,
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { slice::from_raw_parts(key.as_raw_ptr(), len) };
        let other_keys = unsafe { slice::from_raw_parts(other.key.as_raw_ptr(), other.len) };

        let other = (other_keys, &other.values);
        (keys, values).partial_cmp(&other)
    }
}

impl<K, V, P> Ord for KeyValueSlices<'_, '_, K, V, P>
where
    K: Ord,
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { slice::from_raw_parts(key.as_raw_ptr(), len) };
        let other_keys = unsafe { slice::from_raw_parts(other.key.as_raw_ptr(), other.len) };

        let other = (other_keys, &other.values);
        (keys, values).cmp(&other)
    }
}

impl<K, V, P> Hash for KeyValueSlices<'_, '_, K, V, P>
where
    K: Hash,
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self {
            key,
            len,
            ref values,
        } = *self;

        let keys = unsafe { slice::from_raw_parts(key.as_raw_ptr(), len) };
        (keys, values).hash(state);
    }
}

impl<K, V, P> Clone for KeyValueSlices<'_, '_, K, V, P>
where
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Clone,
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

impl<K, V, P> Copy for KeyValueSlices<'_, '_, K, V, P>
where
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Slices<'ctx, 'a, V>: Copy,
{
}
