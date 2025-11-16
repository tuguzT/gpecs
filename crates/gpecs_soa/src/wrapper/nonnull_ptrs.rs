use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr::NonNull,
};

use crate::traits::{NonNullPtrs as Inner, Soa, SoaContext};

/// Type wrapper for [non-null pointers](SoaContext::NonNullPtrs)
/// to each field of [`Fields`](SoaContext::Fields)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct NonNullPtrs<'context, T>
where
    T: Soa + ?Sized,
{
    inner: Inner<'static, T>,
    phantom: PhantomData<&'context ()>,
}

impl<'context, T> NonNullPtrs<'context, T>
where
    T: Soa + ?Sized,
{
    /// Creates self from the [non-null pointers](SoaContext::NonNullPtrs)
    /// to each field of [`Fields`](SoaContext::Fields).
    #[inline]
    pub fn new(inner: Inner<'context, T>) -> Self {
        Self {
            inner: unsafe { transmute::<Inner<'_, T>, Inner<'_, T>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [non-null pointers](SoaContext::NonNullPtrs)
    /// to each field of [`Fields`](SoaContext::Fields) from self.
    #[inline]
    pub fn as_inner(&self) -> &Inner<'_, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [non-null pointers](SoaContext::NonNullPtrs)
    /// to each field of [`Fields`](SoaContext::Fields) from self.
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut Inner<'_, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [non-null pointers](SoaContext::NonNullPtrs)
    /// to each field of [`Fields`](SoaContext::Fields) from self.
    #[inline]
    pub fn into_inner(self) -> Inner<'context, T> {
        let Self { inner, .. } = self;
        T::Context::upcast_nonnull_ptrs(inner)
    }
}

impl<T> Debug for NonNullPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> Inner<'any, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("NonNullPtrs").field(inner).finish()
    }
}

impl<T> Default for NonNullPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> Inner<'any, T>: Default,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: PhantomData,
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
    for<'any> Inner<'any, T>: Copy,
{
}

impl<T> PartialEq for NonNullPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> Inner<'any, T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<T> Eq for NonNullPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> Inner<'any, T>: Eq,
{
}

impl<T> PartialOrd for NonNullPtrs<'_, T>
where
    T: Soa + ?Sized,
    for<'any> Inner<'any, T>: PartialOrd,
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
    for<'any> Inner<'any, T>: Ord,
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
    for<'any> Inner<'any, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}
