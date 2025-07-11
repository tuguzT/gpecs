use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
};

use crate::traits::Soa;

#[repr(transparent)]
pub struct Ptrs<'context, T>
where
    T: Soa,
{
    inner: T::Ptrs<'static>,
    phantom: PhantomData<&'context ()>,
}

impl<'context, T> Ptrs<'context, T>
where
    T: Soa,
{
    #[inline]
    pub fn new(inner: T::Ptrs<'context>) -> Self {
        Self {
            inner: unsafe { transmute(inner) },
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_inner(self) -> T::Ptrs<'context> {
        let Self { inner, .. } = self;
        T::upcast_ptrs(inner)
    }
}

impl<T> Debug for Ptrs<'_, T>
where
    T: Soa,
    for<'any> T::Ptrs<'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("Ptrs").field(inner).finish()
    }
}

impl<T> Default for Ptrs<'_, T>
where
    T: Soa,
    for<'any> T::Ptrs<'any>: Default,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: Default::default(),
        }
    }
}

impl<T> Clone for Ptrs<'_, T>
where
    T: Soa,
{
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;

        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<T> Copy for Ptrs<'_, T>
where
    T: Soa,
    for<'any> T::Ptrs<'any>: Copy,
{
}

impl<T> PartialEq for Ptrs<'_, T>
where
    T: Soa,
    for<'any> T::Ptrs<'any>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<T> Eq for Ptrs<'_, T>
where
    T: Soa,
    for<'any> T::Ptrs<'any>: Eq,
{
}

impl<T> PartialOrd for Ptrs<'_, T>
where
    T: Soa,
    for<'any> T::Ptrs<'any>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { inner, phantom } = self;

        match inner.partial_cmp(&other.inner) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        phantom.partial_cmp(&other.phantom)
    }
}

impl<T> Ord for Ptrs<'_, T>
where
    T: Soa,
    for<'any> T::Ptrs<'any>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { inner, phantom } = self;

        match inner.cmp(&other.inner) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        phantom.cmp(&other.phantom)
    }
}

impl<T> Hash for Ptrs<'_, T>
where
    T: Soa,
    for<'any> T::Ptrs<'any>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;

        inner.hash(state);
        phantom.hash(state);
    }
}
