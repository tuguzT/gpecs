use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr,
};

use crate::traits::Soa;

/// Type wrapper for [references](Soa::Refs)
/// to each field of [`Fields`](Soa::Fields)
/// which is covariant over generic lifetimes.
#[repr(transparent)]
pub struct Refs<'context, 'a, T>
where
    T: Soa + ?Sized + 'a,
{
    inner: T::Refs<'static, 'a>,
    phantom: PhantomData<&'context ()>,
}

impl<'context, 'a, T> Refs<'context, 'a, T>
where
    T: Soa + ?Sized,
{
    /// Creates self from the [references](Soa::Refs)
    /// to each field of [`Fields`](Soa::Fields).
    #[inline]
    pub fn new(inner: T::Refs<'context, 'a>) -> Self {
        Self {
            inner: unsafe { transmute::<T::Refs<'_, '_>, T::Refs<'_, '_>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [references](Soa::Refs)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn as_inner(&self) -> &T::Refs<'context, 'a> {
        let Self { inner, .. } = self;
        unsafe { &*ptr::from_ref(inner).cast() }
    }

    /// Retrieves a mutable reference of [references](Soa::Refs)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut T::Refs<'context, 'a> {
        let Self { inner, .. } = self;
        unsafe { &mut *ptr::from_mut(inner).cast() }
    }

    /// Retrieves the [references](Soa::Refs)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn into_inner(self) -> T::Refs<'context, 'a> {
        let Self { inner, .. } = self;
        T::upcast_refs(inner)
    }
}

impl<'a, T> Debug for Refs<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'any> T::Refs<'any, 'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("Refs").field(inner).finish()
    }
}

impl<'a, T> Default for Refs<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'any> T::Refs<'any, 'a>: Default,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Clone for Refs<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'any> T::Refs<'any, 'a>: Clone,
{
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;
        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<'a, T> Copy for Refs<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'any> T::Refs<'any, 'a>: Copy,
{
}

impl<'a, T> PartialEq for Refs<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'any> T::Refs<'any, 'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<'a, T> Eq for Refs<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'any> T::Refs<'any, 'a>: Eq,
{
}

impl<'a, T> PartialOrd for Refs<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'any> T::Refs<'any, 'a>: PartialOrd,
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

impl<'a, T> Ord for Refs<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'any> T::Refs<'any, 'a>: Ord,
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

impl<'a, T> Hash for Refs<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'any> T::Refs<'any, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}
