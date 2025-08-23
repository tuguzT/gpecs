use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr::NonNull,
};

use crate::traits::Soa;

/// Type wrapper for [mutable pointers](Soa::MutPtrs)
/// to each field of [`Fields`](Soa::Fields)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct MutPtrs<'context, T>
where
    T: Soa + ?Sized,
{
    inner: T::MutPtrs<'static>,
    phantom: PhantomData<&'context ()>,
}

impl<'context, T> MutPtrs<'context, T>
where
    T: Soa + ?Sized,
{
    /// Creates self from the [mutable pointers](Soa::MutPtrs)
    /// to each field of [`Fields`](Soa::Fields).
    #[inline]
    pub fn new(inner: T::MutPtrs<'context>) -> Self {
        Self {
            inner: unsafe { transmute::<T::MutPtrs<'_>, T::MutPtrs<'_>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [mutable pointers](Soa::MutPtrs)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn as_inner(&self) -> &T::MutPtrs<'context> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [mutable pointers](Soa::MutPtrs)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut T::MutPtrs<'context> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [mutable pointers](Soa::MutPtrs)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn into_inner(self) -> T::MutPtrs<'context> {
        let Self { inner, .. } = self;
        T::upcast_mut_ptrs(inner)
    }
}

impl<T> Debug for MutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::MutPtrs<'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("MutPtrs").field(inner).finish()
    }
}

impl<T> Default for MutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::MutPtrs<'any>: Default,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<T> Clone for MutPtrs<'_, T>
where
    T: Soa + ?Sized,
{
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;
        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<T> Copy for MutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::MutPtrs<'any>: Copy,
{
}

impl<T> PartialEq for MutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::MutPtrs<'any>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<T> Eq for MutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::MutPtrs<'any>: Eq,
{
}

impl<T> PartialOrd for MutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::MutPtrs<'any>: PartialOrd,
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

impl<T> Ord for MutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::MutPtrs<'any>: Ord,
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

impl<T> Hash for MutPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::MutPtrs<'any>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}
