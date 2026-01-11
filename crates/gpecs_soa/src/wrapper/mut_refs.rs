use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr::NonNull,
};

use crate::traits::{Soa, SoaContext};

type Inner<'ctx, 'a, T> = crate::traits::RefsMut<'ctx, 'a, T>;

/// Type wrapper for [mutable references](SoaContext::RefsMut)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct RefsMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    inner: Inner<'static, 'a, T>,
    phantom: PhantomData<&'ctx ()>,
}

impl<'ctx, 'a, T> RefsMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    /// Creates self from the [mutable references](SoaContext::RefsMut).
    #[inline]
    pub fn new(inner: Inner<'ctx, 'a, T>) -> Self {
        Self {
            inner: unsafe { transmute::<Inner<'_, '_, T>, Inner<'_, '_, T>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [mutable references](SoaContext::RefsMut).
    #[inline]
    pub fn as_inner(&self) -> &Inner<'_, 'a, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [mutable references](SoaContext::RefsMut).
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut Inner<'_, 'a, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [mutable references](SoaContext::RefsMut).
    #[inline]
    pub fn into_inner(self) -> Inner<'ctx, 'a, T> {
        let Self { inner, .. } = self;
        T::Context::upcast_mut_refs(inner)
    }
}

impl<'a, T> Debug for RefsMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("Refs").field(inner).finish()
    }
}

impl<'a, T> Default for RefsMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Default,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Clone for RefsMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Clone,
{
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;
        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<'a, T> Copy for RefsMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Copy,
{
}

impl<'a, T> PartialEq for RefsMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<'a, T> Eq for RefsMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Eq,
{
}

impl<'a, T> PartialOrd for RefsMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: PartialOrd,
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

impl<'a, T> Ord for RefsMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Ord,
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

impl<'a, T> Hash for RefsMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}
