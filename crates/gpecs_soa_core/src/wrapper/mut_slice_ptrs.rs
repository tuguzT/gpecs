use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr::NonNull,
};

use crate::traits::{RawSoa, RawSoaContext};

type Inner<'ctx, T> = crate::traits::SliceMutPtrs<'ctx, T>;

/// Type wrapper for [mutable slice pointers](RawSoaContext::SliceMutPtrs)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct SliceMutPtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    inner: Inner<'static, T>,
    phantom: PhantomData<&'ctx ()>,
}

impl<'ctx, T> SliceMutPtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    /// Creates self from the [mutable slice pointers](RawSoaContext::SliceMutPtrs).
    #[inline]
    pub fn new(inner: Inner<'ctx, T>) -> Self {
        Self {
            inner: unsafe { transmute::<Inner<'_, T>, Inner<'_, T>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [mutable slice pointers](RawSoaContext::SliceMutPtrs).
    #[inline]
    pub fn as_inner(&self) -> &Inner<'_, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [mutable slice pointers](RawSoaContext::SliceMutPtrs).
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut Inner<'_, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [mutable slice pointers](RawSoaContext::SliceMutPtrs).
    #[inline]
    pub fn into_inner(self) -> Inner<'ctx, T> {
        let Self { inner, .. } = self;
        T::Context::upcast_mut_slice_ptrs(inner)
    }
}

impl<T> Debug for SliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("SliceMutPtrs").field(inner).finish()
    }
}

impl<'ctx, T> Default for SliceMutPtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
    Inner<'ctx, T>: Default,
{
    #[inline]
    fn default() -> Self {
        let inner = Default::default();
        Self::new(inner)
    }
}

impl<T> Clone for SliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;

        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<T> Copy for SliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Copy,
{
}

impl<T> PartialEq for SliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, .. } = self;
        inner == &other.inner
    }
}

impl<T> Eq for SliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Eq,
{
}

impl<T> PartialOrd for SliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { inner, .. } = self;
        inner.partial_cmp(&other.inner)
    }
}

impl<T> Ord for SliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { inner, .. } = self;
        inner.cmp(&other.inner)
    }
}

impl<T> Hash for SliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, .. } = self;
        inner.hash(state);
    }
}
