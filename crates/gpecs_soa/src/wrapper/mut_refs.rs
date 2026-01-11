use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr::NonNull,
};

use crate::traits::Soa;

/// Type wrapper for [mutable references](Soa::RefsMut)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct RefsMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    inner: T::RefsMut<'static>,
    phantom: PhantomData<&'ctx ()>,
}

impl<'ctx, 'a, T> RefsMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    /// Creates self from the [mutable references](Soa::RefsMut).
    #[inline]
    pub fn new(inner: T::RefsMut<'ctx>) -> Self {
        Self {
            inner: unsafe { transmute::<T::RefsMut<'_>, T::RefsMut<'_>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [mutable references](Soa::RefsMut).
    #[inline]
    pub fn as_inner(&self) -> &T::RefsMut<'_> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [mutable references](Soa::RefsMut).
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut T::RefsMut<'_> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [mutable references](Soa::RefsMut).
    #[inline]
    pub fn into_inner(self) -> T::RefsMut<'ctx> {
        let Self { inner, .. } = self;
        T::upcast_refs_mut(inner)
    }
}

impl<'a, T> Debug for RefsMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> T::RefsMut<'ctx>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("Refs").field(inner).finish()
    }
}

impl<'a, T> Default for RefsMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> T::RefsMut<'ctx>: Default,
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
    for<'ctx> T::RefsMut<'ctx>: Clone,
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
    for<'ctx> T::RefsMut<'ctx>: Copy,
{
}

impl<'a, T> PartialEq for RefsMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> T::RefsMut<'ctx>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<'a, T> Eq for RefsMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> T::RefsMut<'ctx>: Eq,
{
}

impl<'a, T> PartialOrd for RefsMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> T::RefsMut<'ctx>: PartialOrd,
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
    for<'ctx> T::RefsMut<'ctx>: Ord,
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
    for<'ctx> T::RefsMut<'ctx>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}
