use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
};

use crate::traits::Soa;

/// Type wrapper for [mutable references](Soa::RefsMut)
/// to each field of [`Fields`](Soa::Fields)
/// which is covariant over generic lifetimes.
#[repr(transparent)]
pub struct RefsMut<'context, 'a, T>
where
    T: Soa + 'a,
{
    inner: T::RefsMut<'static, 'a>,
    phantom: PhantomData<&'context ()>,
}

impl<'context, 'a, T> RefsMut<'context, 'a, T>
where
    T: Soa,
{
    /// Creates self from the [mutable references](Soa::RefsMut)
    /// to each field of [`Fields`](Soa::Fields).
    #[inline]
    pub fn new(inner: T::RefsMut<'context, 'a>) -> Self {
        Self {
            inner: unsafe { transmute::<T::RefsMut<'_, '_>, T::RefsMut<'_, '_>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves the [mutable references](Soa::RefsMut)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn into_inner(self) -> T::RefsMut<'context, 'a> {
        let Self { inner, .. } = self;
        T::upcast_refs_mut(inner)
    }
}

impl<'a, T> Debug for RefsMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::RefsMut<'any, 'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("Refs").field(inner).finish()
    }
}

impl<'a, T> Default for RefsMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::RefsMut<'any, 'a>: Default,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: Default::default(),
        }
    }
}

impl<'a, T> Clone for RefsMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::RefsMut<'any, 'a>: Clone,
{
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;
        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<'a, T> Copy for RefsMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::RefsMut<'any, 'a>: Copy,
{
}

impl<'a, T> PartialEq for RefsMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::RefsMut<'any, 'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<'a, T> Eq for RefsMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::RefsMut<'any, 'a>: Eq,
{
}

impl<'a, T> PartialOrd for RefsMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::RefsMut<'any, 'a>: PartialOrd,
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

impl<'a, T> Ord for RefsMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::RefsMut<'any, 'a>: Ord,
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

impl<'a, T> Hash for RefsMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::RefsMut<'any, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}
