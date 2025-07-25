use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
};

use crate::traits::Soa;

/// Type wrapper for [non-null pointers](Soa::NonNullPtrs)
/// to each field of [`Fields`](Soa::Fields)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct NonNullPtrs<'context, T>
where
    T: Soa + ?Sized,
{
    inner: T::NonNullPtrs<'static>,
    phantom: PhantomData<&'context ()>,
}

impl<'context, T> NonNullPtrs<'context, T>
where
    T: Soa + ?Sized,
{
    /// Creates self from the [non-null pointers](Soa::NonNullPtrs)
    /// to each field of [`Fields`](Soa::Fields).
    #[inline]
    pub fn new(inner: T::NonNullPtrs<'context>) -> Self {
        Self {
            inner: unsafe { transmute::<T::NonNullPtrs<'_>, T::NonNullPtrs<'_>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves the [non-null pointers](Soa::NonNullPtrs)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn into_inner(self) -> T::NonNullPtrs<'context> {
        let Self { inner, .. } = self;
        T::upcast_nonnull_ptrs(inner)
    }
}

impl<T> Debug for NonNullPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::NonNullPtrs<'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("NonNullPtrs").field(inner).finish()
    }
}

impl<T> Default for NonNullPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::NonNullPtrs<'any>: Default,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: Default::default(),
        }
    }
}

impl<T> Clone for NonNullPtrs<'_, T>
where
    T: Soa + ?Sized,
{
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;
        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<T> Copy for NonNullPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::NonNullPtrs<'any>: Copy,
{
}

impl<T> PartialEq for NonNullPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::NonNullPtrs<'any>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<T> Eq for NonNullPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::NonNullPtrs<'any>: Eq,
{
}

impl<T> PartialOrd for NonNullPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::NonNullPtrs<'any>: PartialOrd,
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

impl<T> Ord for NonNullPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::NonNullPtrs<'any>: Ord,
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

impl<T> Hash for NonNullPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::NonNullPtrs<'any>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}
