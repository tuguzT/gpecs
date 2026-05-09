//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

use core::{
    borrow::{Borrow, BorrowMut},
    cmp,
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr,
};

#[derive(Debug, Default, Clone, Copy, Eq, Ord, Hash)]
#[repr(transparent)]
pub struct Identity<T>(pub T)
where
    T: ?Sized;

impl<T> Identity<T>
where
    T: ?Sized,
{
    #[inline]
    pub const fn from_inner_ref(inner: &T) -> &Self {
        // SAFETY: Self is `#[repr(transparent)]` over `T`.
        unsafe { (ptr::from_ref(inner) as *const Self).as_ref_unchecked() }
    }

    #[inline]
    pub const fn from_inner_mut(inner: &mut T) -> &mut Self {
        // SAFETY: Self is `#[repr(transparent)]` over `T`.
        unsafe { (ptr::from_mut(inner) as *mut Self).as_mut_unchecked() }
    }

    #[inline]
    pub const fn as_inner(&self) -> &T {
        let Self(inner) = self;
        inner
    }

    #[inline]
    pub const fn as_inner_mut(&mut self) -> &mut T {
        let Self(inner) = self;
        inner
    }
}

impl<T> Identity<T> {
    #[inline]
    pub const fn from_inner(inner: T) -> Self {
        Self(inner)
    }

    #[inline]
    pub fn into_inner(self) -> T {
        let Self(inner) = self;
        inner
    }
}

pub trait IdentityPtr<T: ?Sized>: private::Sealed {
    #[expect(clippy::wrong_self_convention, reason = "method of pointer type")]
    fn as_inner_ptr(self) -> *const T;
}

impl<T> IdentityPtr<T> for *const Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn as_inner_ptr(self) -> *const T {
        self as _
    }
}

pub trait IdentityMutPtr<T: ?Sized>: private::Sealed {
    #[expect(clippy::wrong_self_convention, reason = "method of pointer type")]
    fn as_inner_mut_ptr(self) -> *mut T;
}

impl<T> IdentityMutPtr<T> for *mut Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn as_inner_mut_ptr(self) -> *mut T {
        self as _
    }
}

pub trait IdentitySlicePtr<T>: private::Sealed {
    #[expect(clippy::wrong_self_convention, reason = "method of pointer type")]
    fn as_inner_ptr(self) -> *const [T];
}

impl<T> IdentitySlicePtr<T> for *const [Identity<T>] {
    #[inline]
    fn as_inner_ptr(self) -> *const [T] {
        self as _
    }
}

pub trait IdentitySliceMutPtr<T>: private::Sealed {
    #[expect(clippy::wrong_self_convention, reason = "method of pointer type")]
    fn as_inner_mut_ptr(self) -> *mut [T];
}

impl<T> IdentitySliceMutPtr<T> for *mut [Identity<T>] {
    #[inline]
    fn as_inner_mut_ptr(self) -> *mut [T] {
        self as _
    }
}

pub trait IdentitySlice<T>: private::Sealed {
    fn as_inner(&self) -> &[T];
    fn as_inner_mut(&mut self) -> &mut [T];
}

impl<T> IdentitySlice<T> for [Identity<T>] {
    #[inline]
    fn as_inner(&self) -> &[T] {
        let inner = ptr::from_ref(self).as_inner_ptr();
        unsafe { inner.as_ref_unchecked() }
    }

    #[inline]
    fn as_inner_mut(&mut self) -> &mut [T] {
        let inner = ptr::from_mut(self).as_inner_mut_ptr();
        unsafe { inner.as_mut_unchecked() }
    }
}

pub trait AsIdentitySlice<T>: private::Sealed {
    fn as_identity_slice(&self) -> &[Identity<T>];
    fn as_identity_slice_mut(&mut self) -> &mut [Identity<T>];
}

impl<T> AsIdentitySlice<T> for [T] {
    #[inline]
    fn as_identity_slice(&self) -> &[Identity<T>] {
        let inner = ptr::from_ref(self) as *const [_];
        unsafe { inner.as_ref_unchecked() }
    }

    #[inline]
    fn as_identity_slice_mut(&mut self) -> &mut [Identity<T>] {
        let inner = ptr::from_mut(self) as *mut [_];
        unsafe { inner.as_mut_unchecked() }
    }
}

mod private {
    use super::Identity;

    pub trait Sealed {}

    impl<T> Sealed for *const Identity<T> where T: ?Sized {}

    impl<T> Sealed for *mut Identity<T> where T: ?Sized {}

    impl<T> Sealed for *const [Identity<T>] {}

    impl<T> Sealed for *mut [Identity<T>] {}

    impl<T> Sealed for [T] {}
}

impl<T> From<T> for Identity<T> {
    #[inline]
    fn from(inner: T) -> Self {
        Self::from_inner(inner)
    }
}

impl<T, U> PartialEq<Identity<U>> for Identity<T>
where
    T: PartialEq<U> + ?Sized,
    U: ?Sized,
{
    #[inline]
    fn eq(&self, other: &Identity<U>) -> bool {
        self.as_inner() == other.as_inner()
    }
}

impl<T, U> PartialOrd<Identity<U>> for Identity<T>
where
    T: PartialOrd<U> + ?Sized,
    U: ?Sized,
{
    #[inline]
    fn partial_cmp(&self, other: &Identity<U>) -> Option<cmp::Ordering> {
        let this = self.as_inner();
        let other = other.as_inner();
        this.partial_cmp(other)
    }
}

impl<T> Deref for Identity<T>
where
    T: ?Sized,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.as_inner()
    }
}

impl<T> DerefMut for Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.as_inner_mut()
    }
}

impl<T> AsRef<T> for Identity<T>
where
    T: ?Sized,
    <Self as Deref>::Target: AsRef<T>,
{
    #[inline]
    fn as_ref(&self) -> &T {
        (**self).as_ref()
    }
}

impl<T> AsRef<Self> for Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T> AsMut<T> for Identity<T>
where
    T: ?Sized,
    <Self as Deref>::Target: AsMut<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        (**self).as_mut()
    }
}

impl<T> AsMut<Self> for Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T> Borrow<T> for Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn borrow(&self) -> &T {
        self
    }
}

impl<T> BorrowMut<T> for Identity<T>
where
    T: ?Sized,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<T, A> FromIterator<A> for Identity<T>
where
    T: FromIterator<A>,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        let inner = T::from_iter(iter);
        Self::from_inner(inner)
    }
}

impl<T> IntoIterator for Identity<T>
where
    T: IntoIterator,
{
    type Item = T::Item;
    type IntoIter = T::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

impl<T, A> Extend<A> for Identity<T>
where
    T: Extend<A>,
{
    #[inline]
    fn extend<I: IntoIterator<Item = A>>(&mut self, iter: I) {
        self.as_inner_mut().extend(iter);
    }
}

impl<T, Idx> Index<Idx> for Identity<T>
where
    T: Index<Idx>,
{
    type Output = T::Output;

    #[inline]
    fn index(&self, index: Idx) -> &Self::Output {
        self.as_inner().index(index)
    }
}

impl<T, Idx> IndexMut<Idx> for Identity<T>
where
    T: IndexMut<Idx>,
{
    #[inline]
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        self.as_inner_mut().index_mut(index)
    }
}
