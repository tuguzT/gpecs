use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    slice,
};

use gpecs_ptr::slice::ConstSliceItemPtr;
use gpecs_soa::{
    traits::{Refs, Soa, SoaContext, SoaOwned},
    wrapper,
};

use crate::KeyValuePtrs;

pub struct KeyValueRefs<'ctx, 'a, K, V, P = *const K>
where
    K: 'a,
    V: Soa<'a> + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    key: P,
    value: wrapper::Refs<'ctx, 'a, V>,
}

impl<'ctx, 'a, K, V, P> KeyValueRefs<'ctx, 'a, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    #[inline]
    pub fn new(key: &'a K, value: Refs<'ctx, 'a, V>) -> Self {
        let key = unsafe { P::from_slice(slice::from_ref(key), 0) };
        unsafe { Self::from_parts(key, value) }
    }

    #[inline]
    pub unsafe fn from_parts(key: P, value: Refs<'ctx, 'a, V>) -> Self {
        let value = wrapper::Refs::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn into_parts(self) -> (&'a K, Refs<'ctx, 'a, V>) {
        let Self { key, value } = self;

        let key = unsafe { key.as_ref_unchecked() };
        (key, value.into_inner())
    }

    #[inline]
    pub fn into_ptrs(self, context: &'ctx V::Context) -> KeyValuePtrs<'ctx, K, V, P> {
        let Self { key, value } = self;

        let value = context.refs_as_ptrs(value.into_inner());
        KeyValuePtrs::new(key, value)
    }
}

impl<'ctx, 'a, K, V, P> From<(&'a K, Refs<'ctx, 'a, V>)> for KeyValueRefs<'ctx, 'a, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    #[inline]
    fn from(value: (&'a K, Refs<'ctx, 'a, V>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'ctx, 'a, K, V, P> From<KeyValueRefs<'ctx, 'a, K, V, P>> for (&'a K, Refs<'ctx, 'a, V>)
where
    V: Soa<'a> + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
{
    #[inline]
    fn from(value: KeyValueRefs<'ctx, 'a, K, V, P>) -> Self {
        value.into_parts()
    }
}

impl<K, V, P> Debug for KeyValueRefs<'_, '_, K, V, P>
where
    K: Debug,
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;

        let key = unsafe { key.as_ref_unchecked() };
        f.debug_struct("KeyValueRefs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<K, V, P> PartialEq for KeyValueRefs<'_, '_, K, V, P>
where
    K: PartialEq,
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;

        let key = unsafe { key.as_ref_unchecked() };
        let other_key = unsafe { other.key.as_ref_unchecked() };

        let other = (other_key, &other.value);
        (key, value) == other
    }
}

impl<K, V, P> Eq for KeyValueRefs<'_, '_, K, V, P>
where
    K: Eq,
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: Eq,
{
}

impl<K, V, P> PartialOrd for KeyValueRefs<'_, '_, K, V, P>
where
    K: PartialOrd,
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value } = self;

        let key = unsafe { key.as_ref_unchecked() };
        let other_key = unsafe { other.key.as_ref_unchecked() };

        let other = (other_key, &other.value);
        (key, value).partial_cmp(&other)
    }
}

impl<K, V, P> Ord for KeyValueRefs<'_, '_, K, V, P>
where
    K: Ord,
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value } = self;

        let key = unsafe { key.as_ref_unchecked() };
        let other_key = unsafe { other.key.as_ref_unchecked() };

        let other = (other_key, &other.value);
        (key, value).cmp(&other)
    }
}

impl<K, V, P> Hash for KeyValueRefs<'_, '_, K, V, P>
where
    K: Hash,
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;

        let key = unsafe { key.as_ref_unchecked() };
        (key, value).hash(state);
    }
}

impl<K, V, P> Clone for KeyValueRefs<'_, '_, K, V, P>
where
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { key, ref value } = *self;

        let value = value.clone();
        Self { key, value }
    }
}

impl<K, V, P> Copy for KeyValueRefs<'_, '_, K, V, P>
where
    V: SoaOwned + ?Sized,
    P: ConstSliceItemPtr<Item = K>,
    for<'ctx, 'a> Refs<'ctx, 'a, V>: Copy,
{
}
