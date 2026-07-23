use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr::NonNull,
};

use crate::traits::{Soa, SoaContext};

type Inner<'ctx, 'a, T> = crate::traits::SlicesMut<'ctx, 'a, T>;

/// Type wrapper for [mutable slices](SoaContext::SlicesMut)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct SlicesMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    inner: Inner<'static, 'a, T>,
    phantom: PhantomData<&'ctx ()>,
}

impl<'ctx, 'a, T> SlicesMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    /// Creates self from the [mutable slices](SoaContext::SlicesMut).
    #[inline]
    pub fn new(inner: Inner<'ctx, 'a, T>) -> Self {
        Self {
            inner: unsafe { transmute::<Inner<'_, '_, T>, Inner<'_, '_, T>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [mutable slices](SoaContext::SlicesMut).
    #[inline]
    pub fn as_inner(&self) -> &Inner<'_, 'a, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [mutable slices](SoaContext::SlicesMut).
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut Inner<'_, 'a, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [mutable slices](SoaContext::SlicesMut).
    #[inline]
    pub fn into_inner(self) -> Inner<'ctx, 'a, T> {
        let Self { inner, .. } = self;
        T::Context::upcast_mut_slices(inner)
    }
}

impl<'a, T> Debug for SlicesMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("Refs").field(inner).finish()
    }
}

impl<'ctx, 'a, T> Default for SlicesMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    Inner<'ctx, 'a, T>: Default,
{
    #[inline]
    fn default() -> Self {
        let inner = Default::default();
        Self::new(inner)
    }
}

impl<'a, T> Clone for SlicesMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;

        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<'a, T> Copy for SlicesMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Copy,
{
}

impl<'a, T> PartialEq for SlicesMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, .. } = self;
        inner == &other.inner
    }
}

impl<'a, T> Eq for SlicesMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Eq,
{
}

impl<'a, T> PartialOrd for SlicesMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { inner, .. } = self;
        inner.partial_cmp(&other.inner)
    }
}

impl<'a, T> Ord for SlicesMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { inner, .. } = self;
        inner.cmp(&other.inner)
    }
}

impl<'a, T> Hash for SlicesMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, .. } = self;
        inner.hash(state);
    }
}
