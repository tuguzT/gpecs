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
    marker: PhantomData<&'ctx ()>,
}

impl<'ctx, 'a, T> SlicesMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    /// Creates self from the [mutable slices](SoaContext::SlicesMut).
    #[inline]
    pub fn new(inner: Inner<'ctx, 'a, T>) -> Self {
        // SAFETY: internal layout should not change even if lifetime changes: https://github.com/rust-lang/rust/pull/101520#issuecomment-1252016235
        let inner = unsafe { transmute::<Inner<'ctx, 'a, T>, Inner<'static, 'a, T>>(inner) };
        let marker = PhantomData;
        Self { inner, marker }
    }

    /// Retrieves a reference of [mutable slices](SoaContext::SlicesMut).
    #[inline]
    pub fn as_inner(&self) -> &Inner<'ctx, 'a, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [mutable slices](SoaContext::SlicesMut).
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut Inner<'ctx, 'a, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [mutable slices](SoaContext::SlicesMut).
    #[inline]
    pub fn into_inner(self) -> Inner<'ctx, 'a, T> {
        let Self { inner, .. } = self;
        T::Context::mut_slices_upcast(inner)
    }
}

impl<'ctx, 'a, T> Debug for SlicesMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    Inner<'ctx, 'a, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.as_inner();
        f.debug_tuple("SlicesMut").field(inner).finish()
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

impl<'ctx, 'a, T> Clone for SlicesMut<'ctx, 'a, T>
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

impl<'a, T> Copy for SlicesMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
    for<'ctx> Inner<'ctx, 'a, T>: Copy,
{
}

impl<'ctx, 'a, T> PartialEq for SlicesMut<'ctx, 'a, T>
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

impl<'ctx, 'a, T> Eq for SlicesMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    Inner<'ctx, 'a, T>: Eq,
{
}

impl<'ctx, 'a, T> PartialOrd for SlicesMut<'ctx, 'a, T>
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

impl<'ctx, 'a, T> Ord for SlicesMut<'ctx, 'a, T>
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

impl<'ctx, 'a, T> Hash for SlicesMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
    Inner<'ctx, 'a, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let inner = self.as_inner();
        inner.hash(state);
    }
}
