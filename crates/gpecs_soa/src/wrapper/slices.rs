use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr::NonNull,
};

use crate::traits::{Soa, SoaContext};

type Inner<'ctx, 'a, T> = crate::traits::Slices<'ctx, 'a, T>;

/// Type wrapper for [slices](SoaContext::Slices)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct Slices<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    inner: Inner<'static, 'a, T>,
    phantom: PhantomData<&'ctx ()>,
}

impl<'ctx, 'a, T> Slices<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    /// Creates self from the [slices](SoaContext::Slices).
    #[inline]
    pub fn new(inner: Inner<'ctx, 'a, T>) -> Self {
        Self {
            inner: unsafe { transmute::<Inner<'_, '_, T>, Inner<'_, '_, T>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [slices](SoaContext::Slices).
    #[inline]
    pub fn as_inner(&self) -> &Inner<'_, 'a, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [slices](SoaContext::Slices).
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut Inner<'_, 'a, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [slices](SoaContext::Slices).
    #[inline]
    pub fn into_inner(self) -> Inner<'ctx, 'a, T> {
        let Self { inner, .. } = self;
        T::Context::upcast_slices(inner)
    }
}

impl<'a, T> Debug for Slices<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("Slices").field(inner).finish()
    }
}

impl<'ctx, 'a, T> Default for Slices<'ctx, 'a, T>
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

impl<'a, T> Clone for Slices<'_, 'a, T>
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

impl<'a, T> Copy for Slices<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Copy,
{
}

impl<'a, T> PartialEq for Slices<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, .. } = self;
        inner == &other.inner
    }
}

impl<'a, T> Eq for Slices<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Eq,
{
}

impl<'a, T> PartialOrd for Slices<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { inner, .. } = self;
        inner.partial_cmp(&other.inner)
    }
}

impl<'a, T> Ord for Slices<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { inner, .. } = self;
        inner.cmp(&other.inner)
    }
}

impl<'a, T> Hash for Slices<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, .. } = self;
        inner.hash(state);
    }
}
