use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
};

use crate::traits::Soa;

/// Type wrapper for [mutable slice pointers](Soa::SliceMutPtrs)
/// to each field of [`Fields`](Soa::Fields)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct SliceMutPtrs<'context, T>
where
    T: Soa + ?Sized,
{
    inner: T::SliceMutPtrs<'static>,
    phantom: PhantomData<&'context ()>,
}

impl<'context, T> SliceMutPtrs<'context, T>
where
    T: Soa + ?Sized,
{
    /// Creates self from the [mutable slice pointers](Soa::SliceMutPtrs)
    /// to each field of [`Fields`](Soa::Fields).
    #[inline]
    pub fn new(inner: T::SliceMutPtrs<'context>) -> Self {
        Self {
            inner: unsafe { transmute::<T::SliceMutPtrs<'_>, T::SliceMutPtrs<'_>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves the [mutable slice pointers](Soa::SliceMutPtrs)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn into_inner(self) -> T::SliceMutPtrs<'context> {
        let Self { inner, .. } = self;
        T::upcast_slice_mut_ptrs(inner)
    }
}

impl<T> Debug for SliceMutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SliceMutPtrs<'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("SliceMutPtrs").field(inner).finish()
    }
}

impl<T> Default for SliceMutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SliceMutPtrs<'any>: Default,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: Default::default(),
        }
    }
}

impl<T> Clone for SliceMutPtrs<'_, T>
where
    T: Soa + ?Sized,
{
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;
        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<T> Copy for SliceMutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SliceMutPtrs<'any>: Copy,
{
}

impl<T> PartialEq for SliceMutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SliceMutPtrs<'any>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<T> Eq for SliceMutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SliceMutPtrs<'any>: Eq,
{
}

impl<T> PartialOrd for SliceMutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SliceMutPtrs<'any>: PartialOrd,
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

impl<T> Ord for SliceMutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SliceMutPtrs<'any>: Ord,
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

impl<T> Hash for SliceMutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SliceMutPtrs<'any>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}
