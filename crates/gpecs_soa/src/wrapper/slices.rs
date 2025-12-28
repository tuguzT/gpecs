use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr::NonNull,
};

use crate::traits::Soa;

/// Type wrapper for [slices](Soa::Slices)
/// which is covariant over generic lifetimes.
#[repr(transparent)]
pub struct Slices<'ctx, 'a, T>
where
    T: Soa + ?Sized + 'a,
{
    inner: T::Slices<'static, 'a>,
    phantom: PhantomData<&'ctx ()>,
}

impl<'ctx, 'a, T> Slices<'ctx, 'a, T>
where
    T: Soa + ?Sized,
{
    /// Creates self from the [slices](Soa::Slices).
    #[inline]
    pub fn new(inner: T::Slices<'ctx, 'a>) -> Self {
        Self {
            inner: unsafe { transmute::<T::Slices<'_, '_>, T::Slices<'_, '_>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [slices](Soa::Slices).
    #[inline]
    pub fn as_inner(&self) -> &T::Slices<'_, '_> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [slices](Soa::Slices).
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut T::Slices<'_, '_> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [slices](Soa::Slices).
    #[inline]
    pub fn into_inner(self) -> T::Slices<'ctx, 'a> {
        let Self { inner, .. } = self;
        T::upcast_slices(inner)
    }
}

impl<'a, T> Debug for Slices<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'ctx> T::Slices<'ctx, 'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("Slices").field(inner).finish()
    }
}

impl<'a, T> Default for Slices<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'ctx> T::Slices<'ctx, 'a>: Default,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Clone for Slices<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'ctx> T::Slices<'ctx, 'a>: Clone,
{
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;
        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<'a, T> Copy for Slices<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'ctx> T::Slices<'ctx, 'a>: Copy,
{
}

impl<'a, T> PartialEq for Slices<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'ctx> T::Slices<'ctx, 'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<'a, T> Eq for Slices<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'ctx> T::Slices<'ctx, 'a>: Eq,
{
}

impl<'a, T> PartialOrd for Slices<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'ctx> T::Slices<'ctx, 'a>: PartialOrd,
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

impl<'a, T> Ord for Slices<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'ctx> T::Slices<'ctx, 'a>: Ord,
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

impl<'a, T> Hash for Slices<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'ctx> T::Slices<'ctx, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}
