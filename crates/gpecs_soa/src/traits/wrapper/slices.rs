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
/// to each field of [`Fields`](Soa::Fields)
/// which is covariant over generic lifetimes.
#[repr(transparent)]
pub struct Slices<'context, 'a, T>
where
    T: Soa + ?Sized + 'a,
{
    inner: T::Slices<'static, 'a>,
    phantom: PhantomData<&'context ()>,
}

impl<'context, 'a, T> Slices<'context, 'a, T>
where
    T: Soa + ?Sized,
{
    /// Creates self from the [slices](Soa::Slices)
    /// to each field of [`Fields`](Soa::Fields).
    #[inline]
    pub fn new(inner: T::Slices<'context, 'a>) -> Self {
        Self {
            inner: unsafe { transmute::<T::Slices<'_, '_>, T::Slices<'_, '_>>(inner) },
            phantom: PhantomData,
        }
    }

    /// Retrieves a reference of [slices](Soa::Slices)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn as_inner(&self) -> &T::Slices<'context, 'a> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [slices](Soa::Slices)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut T::Slices<'context, 'a> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [slices](Soa::Slices)
    /// to each field of [`Fields`](Soa::Fields) from self.
    #[inline]
    pub fn into_inner(self) -> T::Slices<'context, 'a> {
        let Self { inner, .. } = self;
        T::upcast_slices(inner)
    }
}

impl<'a, T> Debug for Slices<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'any> T::Slices<'any, 'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner, .. } = self;
        f.debug_tuple("Slices").field(inner).finish()
    }
}

impl<'a, T> Default for Slices<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'any> T::Slices<'any, 'a>: Default,
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
    for<'any> T::Slices<'any, 'a>: Clone,
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
    for<'any> T::Slices<'any, 'a>: Copy,
{
}

impl<'a, T> PartialEq for Slices<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'any> T::Slices<'any, 'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { inner, phantom } = self;
        *inner == other.inner && *phantom == other.phantom
    }
}

impl<'a, T> Eq for Slices<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'any> T::Slices<'any, 'a>: Eq,
{
}

impl<'a, T> PartialOrd for Slices<'_, 'a, T>
where
    T: Soa + ?Sized,
    for<'any> T::Slices<'any, 'a>: PartialOrd,
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
    for<'any> T::Slices<'any, 'a>: Ord,
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
    for<'any> T::Slices<'any, 'a>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { inner, phantom } = self;
        inner.hash(state);
        phantom.hash(state);
    }
}
