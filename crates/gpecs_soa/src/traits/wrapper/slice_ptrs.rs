use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr,
};

use crate::traits::Soa;

/// Type wrapper for [slice pointers](Soa::SlicePtrs)
/// to each field of [`Fields`](Soa::Fields)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct SlicePtrs<'context, T>
where
    T: Soa + ?Sized,
{
    inner: T::SlicePtrs<'static>,
    phantom: PhantomData<&'context ()>,
}

impl<'context, T> SlicePtrs<'context, T>
where
    T: Soa + ?Sized,
{
    /// Creates self from the [slice pointers](Soa::SlicePtrs)
    /// to each field of [`Fields`](Soa::Fields).
    #[inline]
    pub fn new(inner: T::SlicePtrs<'context>) -> Self {
        Self {
            inner: unsafe { transmute::<T::SlicePtrs<'_>, T::SlicePtrs<'_>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [slice pointers](Soa::SlicePtrs)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn as_inner(&self) -> &T::SlicePtrs<'context> {
        let Self { inner, .. } = self;
        unsafe { &*ptr::from_ref(inner).cast() }
    }

    /// Retrieves a mutable reference of [slice pointers](Soa::SlicePtrs)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut T::SlicePtrs<'context> {
        let Self { inner, .. } = self;
        unsafe { &mut *ptr::from_mut(inner).cast() }
    }

    /// Retrieves the [slice pointers](Soa::SlicePtrs)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn into_inner(self) -> T::SlicePtrs<'context> {
        let Self { inner, .. } = self;
        T::upcast_slice_ptrs(inner)
    }
}

impl<T> Debug for SlicePtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SlicePtrs<'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("SlicePtrs").field(inner).finish()
    }
}

impl<T> Default for SlicePtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SlicePtrs<'any>: Default,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<T> Clone for SlicePtrs<'_, T>
where
    T: Soa + ?Sized,
{
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;
        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<T> Copy for SlicePtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SlicePtrs<'any>: Copy,
{
}

impl<T> PartialEq for SlicePtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SlicePtrs<'any>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<T> Eq for SlicePtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SlicePtrs<'any>: Eq,
{
}

impl<T> PartialOrd for SlicePtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SlicePtrs<'any>: PartialOrd,
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

impl<T> Ord for SlicePtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SlicePtrs<'any>: Ord,
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

impl<T> Hash for SlicePtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> T::SlicePtrs<'any>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}
