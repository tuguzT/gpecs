use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr::NonNull,
};

use crate::traits::{RawSoa, RawSoaContext};

type Inner<'ctx, T> = crate::traits::Ptrs<'ctx, T>;

/// Type wrapper for [pointers](RawSoaContext::Ptrs)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct Ptrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    inner: Inner<'static, T>,
    phantom: PhantomData<&'ctx ()>,
}

impl<'ctx, T> Ptrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    /// Creates self from the [pointers](RawSoaContext::Ptrs).
    #[inline]
    pub fn new(inner: Inner<'ctx, T>) -> Self {
        Self {
            inner: unsafe { transmute::<Inner<'_, T>, Inner<'_, T>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [pointers](RawSoaContext::Ptrs).
    #[inline]
    pub fn as_inner(&self) -> &Inner<'_, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [pointers](RawSoaContext::Ptrs).
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut Inner<'_, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [pointers](RawSoaContext::Ptrs).
    #[inline]
    pub fn into_inner(self) -> Inner<'ctx, T> {
        let Self { inner, .. } = self;
        T::Context::upcast_ptrs(inner)
    }
}

impl<T> Debug for Ptrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("Ptrs").field(inner).finish()
    }
}

impl<'ctx, T> Default for Ptrs<'ctx, T>
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

impl<T> Clone for Ptrs<'_, T>
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

impl<T> Copy for Ptrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Copy,
{
}

impl<T> PartialEq for Ptrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, .. } = self;
        inner == &other.inner
    }
}

impl<T> Eq for Ptrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Eq,
{
}

impl<T> PartialOrd for Ptrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { inner, .. } = self;
        inner.partial_cmp(&other.inner)
    }
}

impl<T> Ord for Ptrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { inner, .. } = self;
        inner.cmp(&other.inner)
    }
}

impl<T> Hash for Ptrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Inner<'ctx, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, .. } = self;
        inner.hash(state);
    }
}
