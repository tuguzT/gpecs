use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr::NonNull,
};

use crate::traits::{RawSoa, RawSoaContext};

type Inner<'ctx, T> = crate::traits::MutPtrs<'ctx, T>;

/// Type wrapper for [mutable pointers](RawSoaContext::MutPtrs)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct MutPtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    inner: Inner<'static, T>,
    phantom: PhantomData<&'ctx ()>,
}

impl<'ctx, T> MutPtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    /// Creates self from the [mutable pointers](RawSoaContext::MutPtrs).
    #[inline]
    pub fn new(inner: Inner<'ctx, T>) -> Self {
        Self {
            inner: unsafe { transmute::<Inner<'_, T>, Inner<'_, T>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [mutable pointers](RawSoaContext::MutPtrs) .
    #[inline]
    pub fn as_inner(&self) -> &Inner<'_, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [mutable pointers](RawSoaContext::MutPtrs).
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut Inner<'_, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [mutable pointers](RawSoaContext::MutPtrs).
    #[inline]
    pub fn into_inner(self) -> Inner<'ctx, T> {
        let Self { inner, .. } = self;
        T::Context::upcast_mut_ptrs(inner)
    }
}

impl<T> Debug for MutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("MutPtrs").field(inner).finish()
    }
}

impl<T> Default for MutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Default,
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
    T: RawSoa + ?Sized,
{
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;
        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<T> Copy for MutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Copy,
{
}

impl<T> PartialEq for MutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<T> Eq for MutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Eq,
{
}

impl<T> PartialOrd for MutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: PartialOrd,
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
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Ord,
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
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}
