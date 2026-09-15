use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    mem::transmute,
    ptr::NonNull,
};

use crate::traits::{RawSoa, RawSoaContext};

type Inner<'ctx, T> = crate::traits::SlicePtrs<'ctx, T>;

/// Type wrapper for [slice pointers](RawSoaContext::SlicePtrs)
/// which is covariant over generic lifetime.
#[repr(transparent)]
pub struct SlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    inner: Inner<'static, T>,
    marker: PhantomData<&'ctx ()>,
}

impl<'ctx, T> SlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    /// Creates self from the [slice pointers](RawSoaContext::SlicePtrs).
    #[inline]
    pub fn new(inner: Inner<'ctx, T>) -> Self {
        // SAFETY: internal layout should not change even if lifetime changes: https://github.com/rust-lang/rust/pull/101520#issuecomment-1252016235
        let inner = unsafe { transmute::<Inner<'ctx, T>, Inner<'static, T>>(inner) };
        let marker = PhantomData;
        Self { inner, marker }
    }

    /// Retrieves a reference of [slice pointers](RawSoaContext::SlicePtrs).
    #[inline]
    pub fn as_inner(&self) -> &Inner<'ctx, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_ref(inner).cast().as_ref() }
    }

    /// Retrieves a mutable reference of [slice pointers](RawSoaContext::SlicePtrs).
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut Inner<'ctx, T> {
        let Self { inner, .. } = self;
        unsafe { NonNull::from_mut(inner).cast().as_mut() }
    }

    /// Retrieves the [slice pointers](RawSoaContext::SlicePtrs).
    #[inline]
    pub fn into_inner(self) -> Inner<'ctx, T> {
        let Self { inner, .. } = self;
        T::Context::slice_ptrs_upcast(inner)
    }
}

impl<'ctx, T> Debug for SlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
    Inner<'ctx, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.as_inner();
        f.debug_tuple("SlicePtrs").field(inner).finish()
    }
}

impl<'ctx, T> Default for SlicePtrs<'ctx, T>
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

impl<T> Clone for SlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
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

impl<T> Copy for SlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    Inner<'static, T>: Copy,
{
}

impl<'ctx, T> PartialEq for SlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
    Inner<'ctx, T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let inner = self.as_inner();
        let other = other.as_inner();
        inner.eq(other)
    }
}

impl<'ctx, T> Eq for SlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
    Inner<'ctx, T>: Eq,
{
}

impl<'ctx, T> PartialOrd for SlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
    Inner<'ctx, T>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let inner = self.as_inner();
        let other = other.as_inner();
        inner.partial_cmp(other)
    }
}

impl<'ctx, T> Ord for SlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
    Inner<'ctx, T>: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let inner = self.as_inner();
        let other = other.as_inner();
        inner.cmp(other)
    }
}

impl<'ctx, T> Hash for SlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
    Inner<'ctx, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let inner = self.as_inner();
        inner.hash(state);
    }
}
