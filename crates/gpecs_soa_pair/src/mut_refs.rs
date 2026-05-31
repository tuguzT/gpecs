use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    slice,
};

use gpecs_ptr::slice::{CastConst, MutSliceItemPtr};
use gpecs_soa::{
    traits::{RefsMut, Soa, SoaContext, SoaOwned},
    wrapper,
};

use crate::{KeyValueMutPtrs, KeyValueRefs};

pub struct KeyValueMutRefs<'ctx, 'a, K, V, P = *mut K>
where
    K: 'a,
    V: Soa<'a> + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    key: P,
    value: wrapper::RefsMut<'ctx, 'a, V>,
}

impl<'ctx, 'a, K, V, P> KeyValueMutRefs<'ctx, 'a, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    #[inline]
    pub fn new(key: &'a mut K, value: RefsMut<'ctx, 'a, V>) -> Self {
        let key = unsafe { P::from_slice(slice::from_mut(key), 0) };
        unsafe { Self::from_parts(key, value) }
    }

    #[inline]
    pub unsafe fn from_parts(key: P, value: RefsMut<'ctx, 'a, V>) -> Self {
        let value = wrapper::RefsMut::new(value);
        Self { key, value }
    }

    #[inline]
    pub fn into_parts(self) -> (&'a mut K, RefsMut<'ctx, 'a, V>) {
        let Self { key, value } = self;

        let key = unsafe { key.as_mut_unchecked() };
        (key, value.into_inner())
    }

    #[inline]
    pub fn into_ptrs(self, context: &'ctx V::Context) -> KeyValueMutPtrs<'ctx, K, V, P> {
        let Self { key, value } = self;

        let value = context.mut_refs_as_mut_ptrs(value.into_inner());
        KeyValueMutPtrs::new(key, value)
    }

    #[inline]
    pub fn into_refs(
        self,
        context: &'ctx V::Context,
    ) -> KeyValueRefs<'ctx, 'a, K, V, CastConst<P>> {
        let Self { key, value } = self;

        let key = key.cast_const();
        let value = context.mut_refs_as_refs(value.into_inner());
        unsafe { KeyValueRefs::from_parts(key, value) }
    }
}

impl<'ctx, 'a, K, V, P> From<(&'a mut K, RefsMut<'ctx, 'a, V>)>
    for KeyValueMutRefs<'ctx, 'a, K, V, P>
where
    V: Soa<'a> + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    #[inline]
    fn from(value: (&'a mut K, RefsMut<'ctx, 'a, V>)) -> Self {
        let (key, value) = value;
        Self::new(key, value)
    }
}

impl<'ctx, 'a, K, V, P> From<KeyValueMutRefs<'ctx, 'a, K, V, P>>
    for (&'a mut K, RefsMut<'ctx, 'a, V>)
where
    V: Soa<'a> + ?Sized,
    P: MutSliceItemPtr<Item = K>,
{
    #[inline]
    fn from(value: KeyValueMutRefs<'ctx, 'a, K, V, P>) -> Self {
        value.into_parts()
    }
}

impl<K, V, P> Debug for KeyValueMutRefs<'_, '_, K, V, P>
where
    K: Debug,
    V: SoaOwned + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { key, value } = self;

        let key = unsafe { key.as_mut_unchecked() };
        f.debug_struct("KeyValueMutRefs")
            .field("key", key)
            .field("value", value)
            .finish()
    }
}

impl<K, V, P> PartialEq for KeyValueMutRefs<'_, '_, K, V, P>
where
    K: PartialEq,
    V: SoaOwned + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { key, value } = self;

        let key = unsafe { key.as_mut_unchecked() };
        let other_key = unsafe { other.key.as_mut_unchecked() };

        let other = (other_key, &other.value);
        (key, value) == other
    }
}

impl<K, V, P> Eq for KeyValueMutRefs<'_, '_, K, V, P>
where
    K: Eq,
    V: SoaOwned + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: Eq,
{
}

impl<K, V, P> PartialOrd for KeyValueMutRefs<'_, '_, K, V, P>
where
    K: PartialOrd,
    V: SoaOwned + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { key, value } = self;

        let key = unsafe { key.as_mut_unchecked() };
        let other_key = unsafe { other.key.as_mut_unchecked() };

        let other = (other_key, &other.value);
        (key, value).partial_cmp(&other)
    }
}

impl<K, V, P> Ord for KeyValueMutRefs<'_, '_, K, V, P>
where
    K: Ord,
    V: SoaOwned + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { key, value } = self;

        let key = unsafe { key.as_mut_unchecked() };
        let other_key = unsafe { other.key.as_mut_unchecked() };

        let other = (other_key, &other.value);
        (key, value).cmp(&other)
    }
}

impl<K, V, P> Hash for KeyValueMutRefs<'_, '_, K, V, P>
where
    K: Hash,
    V: SoaOwned + ?Sized,
    P: MutSliceItemPtr<Item = K>,
    for<'ctx, 'a> RefsMut<'ctx, 'a, V>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { key, value } = self;

        let key = unsafe { key.as_mut_unchecked() };
        (key, value).hash(state);
    }
}
