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
    marker: PhantomData<&'ctx ()>,
}

impl<'ctx, 'a, T> Slices<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    /// Creates self from the [slices](SoaContext::Slices).
    #[inline]
    pub fn new(inner: Inner<'ctx, 'a, T>) -> Self {
        // SAFETY: internal layout should not change even if lifetime changes: https://github.com/rust-lang/rust/pull/101520#issuecomment-1252016235
        let inner = unsafe { transmute::<Inner<'ctx, 'a, T>, Inner<'static, 'a, T>>(inner) };
        let marker = PhantomData;
        Self { inner, marker }
    }

    /// Retrieves a reference of [slices](SoaContext::Slices).
    #[inline]
    pub fn as_inner(&self) -> &Inner<'ctx, 'a, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [slices](SoaContext::Slices).
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut Inner<'ctx, 'a, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [slices](SoaContext::Slices).
    #[inline]
    pub fn into_inner(self) -> Inner<'ctx, 'a, T> {
        let Self { inner, .. } = self;
        T::Context::slices_upcast(inner)
    }
}

impl<'ctx, 'a, T> Debug for Slices<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    Inner<'ctx, 'a, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.as_inner();
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

impl<'ctx, 'a, T> Clone for Slices<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    Inner<'ctx, 'a, T>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let inner = self.as_inner().clone();
        Self::new(inner)
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        let inner = self.as_inner_mut();
        let source = source.as_inner();
        inner.clone_from(source);
    }
}

impl<'a, T> Copy for Slices<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Copy,
{
}

impl<'ctx, 'a, T> PartialEq for Slices<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    Inner<'ctx, 'a, T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let inner = self.as_inner();
        let other = other.as_inner();
        inner.eq(other)
    }
}

impl<'ctx, 'a, T> Eq for Slices<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    Inner<'ctx, 'a, T>: Eq,
{
}

impl<'ctx, 'a, T> PartialOrd for Slices<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    Inner<'ctx, 'a, T>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let inner = self.as_inner();
        let other = other.as_inner();
        inner.partial_cmp(other)
    }
}

impl<'ctx, 'a, T> Ord for Slices<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    Inner<'ctx, 'a, T>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let inner = self.as_inner();
        let other = other.as_inner();
        inner.cmp(other)
    }
}

impl<'ctx, 'a, T> Hash for Slices<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    Inner<'ctx, 'a, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let inner = self.as_inner();
        inner.hash(state);
    }
}
