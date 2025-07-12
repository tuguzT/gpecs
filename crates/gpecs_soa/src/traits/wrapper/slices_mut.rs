use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
};

use crate::traits::Soa;

/// Type wrapper for [mutable slices](Soa::SlicesMut)
/// to each field of [`Fields`](Soa::Fields)
/// which is covariant over `'context` lifetime.
#[repr(transparent)]
pub struct SlicesMut<'context, 'a, T>
where
    T: Soa + 'a,
{
    inner: T::SlicesMut<'static, 'a>,
    phantom: PhantomData<&'context ()>,
}

impl<'context, 'a, T> SlicesMut<'context, 'a, T>
where
    T: Soa,
{
    /// Creates self from the [mutable slices](Soa::SlicesMut)
    /// to each field of [`Fields`](Soa::Fields).
    #[inline]
    pub fn new(inner: T::SlicesMut<'context, 'a>) -> Self {
        Self {
            inner: unsafe { transmute(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves the [mutable slices](Soa::SlicesMut)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn into_inner(self) -> T::SlicesMut<'context, 'a> {
        let Self { inner, .. } = self;
        T::upcast_slices_mut(inner)
    }
}

impl<'a, T> Debug for SlicesMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::SlicesMut<'any, 'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("Refs").field(inner).finish()
    }
}

impl<'a, T> Default for SlicesMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::SlicesMut<'any, 'a>: Default,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: Default::default(),
        }
    }
}

impl<'a, T> Clone for SlicesMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::SlicesMut<'any, 'a>: Clone,
{
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;
        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<'a, T> Copy for SlicesMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::SlicesMut<'any, 'a>: Copy,
{
}

impl<'a, T> PartialEq for SlicesMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::SlicesMut<'any, 'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<'a, T> Eq for SlicesMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::SlicesMut<'any, 'a>: Eq,
{
}

impl<'a, T> PartialOrd for SlicesMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::SlicesMut<'any, 'a>: PartialOrd,
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

impl<'a, T> Ord for SlicesMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::SlicesMut<'any, 'a>: Ord,
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

impl<'a, T> Hash for SlicesMut<'_, 'a, T>
where
    T: Soa,
    for<'any> T::SlicesMut<'any, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}
