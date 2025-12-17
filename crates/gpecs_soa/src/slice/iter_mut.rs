use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use crate::{
    slice::{RawIter, RawIterMut},
    traits::{MutPtrs, Ptrs, RawSoa, SliceMutPtrs, SlicePtrs, Soa},
};

#[repr(transparent)]
pub struct IterMut<'c, 'a, T>
where
    T: RawSoa + ?Sized + 'a,
{
    inner: RawIterMut<'c, T>,
    phantom: PhantomData<&'a ()>,
}

impl<'c, T> IterMut<'c, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub unsafe fn from_parts(context: &'c T::Context, slices: SliceMutPtrs<'c, T>) -> Self {
        Self {
            inner: RawIterMut::new(context, slices),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &'c T::Context {
        let Self { inner, .. } = self;
        inner.context()
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'c, T> {
        let (_, ptrs) = self.as_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&'c T::Context, Ptrs<'c, T>) {
        let Self { inner, .. } = self;
        inner.as_ptrs_with_context()
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'c, T> {
        let (_, ptrs) = self.as_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&'c T::Context, MutPtrs<'c, T>) {
        let Self { inner, .. } = self;
        inner.as_mut_ptrs_with_context()
    }

    #[inline]
    pub fn into_ptrs(self) -> Ptrs<'c, T> {
        let (_, ptrs) = self.into_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'c T::Context, Ptrs<'c, T>) {
        let Self { inner, .. } = self;
        inner.into_ptrs_with_context()
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> MutPtrs<'c, T> {
        let (_, ptrs) = self.into_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_mut_ptrs_with_context(self) -> (&'c T::Context, MutPtrs<'c, T>) {
        let Self { inner, .. } = self;
        inner.into_mut_ptrs_with_context()
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'c, T> {
        let (_, slices) = self.as_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&'c T::Context, SlicePtrs<'c, T>) {
        let Self { inner, .. } = self;
        inner.as_slice_ptrs_with_context()
    }

    #[inline]
    pub fn as_slice_mut_ptrs(&mut self) -> SliceMutPtrs<'c, T> {
        let (_, slices) = self.as_slice_mut_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_mut_ptrs_with_context(&mut self) -> (&'c T::Context, SliceMutPtrs<'c, T>) {
        let Self { inner, .. } = self;
        inner.as_slice_mut_ptrs_with_context()
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> SlicePtrs<'c, T> {
        let (_, slices) = self.into_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(self) -> (&'c T::Context, SlicePtrs<'c, T>) {
        let Self { inner, .. } = self;
        inner.into_slice_ptrs_with_context()
    }

    #[inline]
    pub fn into_slice_mut_ptrs(self) -> SliceMutPtrs<'c, T> {
        let (_, slices) = self.into_slice_mut_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn into_slice_mut_ptrs_with_context(self) -> (&'c T::Context, SliceMutPtrs<'c, T>) {
        let Self { inner, .. } = self;
        inner.into_slice_mut_ptrs_with_context()
    }

    #[inline]
    pub fn as_raw_iter(&self) -> &RawIterMut<'c, T> {
        let Self { inner, .. } = self;
        inner
    }

    #[inline]
    pub fn into_raw_iter(self) -> RawIter<'c, T> {
        let Self { inner, .. } = self;
        inner.cast_const()
    }

    #[inline]
    pub fn as_raw_iter_mut(&mut self) -> &mut RawIterMut<'c, T> {
        let Self { inner, .. } = self;
        inner
    }

    #[inline]
    pub fn into_raw_iter_mut(self) -> RawIterMut<'c, T> {
        let Self { inner, .. } = self;
        inner
    }
}

impl<'c, 'a, T> IterMut<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    pub fn new(context: &'c T::Context, slices: T::SlicesMut<'c, 'a>) -> Self {
        let slices = T::slices_mut_as_slice_ptrs(context, slices);
        unsafe { Self::from_parts(context, slices) }
    }

    #[inline]
    pub fn into_slices(self) -> T::SlicesMut<'c, 'a> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    #[doc(alias = "into_parts")]
    pub fn into_slices_with_context(self) -> (&'c T::Context, T::SlicesMut<'c, 'a>) {
        let Self { inner, .. } = self;

        let (context, slices) = inner.into_slice_mut_ptrs_with_context();
        let slices = unsafe { T::slice_mut_ptrs_to_slices(context, slices) };
        (context, slices)
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_, '_> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&'c T::Context, T::Slices<'_, '_>) {
        let Self { inner, .. } = self;

        let (context, slices) = inner.as_slice_ptrs_with_context();
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        let slices = T::upcast_slices(slices);
        (context, slices)
    }
}

impl<T, U> AsRef<[U]> for IterMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Into<&'any [U]>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices().into()
    }
}

impl<T> Debug for IterMut<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'c, 'any> T::Slices<'c, 'any>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("IterMut").field(&slices).finish()
    }
}

impl<'c, 'a, T> Iterator for IterMut<'c, 'a, T>
where
    T: Soa + ?Sized,
{
    type Item = T::RefsMut<'c, 'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        let context = inner.context();

        let f = |ptrs| unsafe { T::ptrs_to_refs_mut(context, ptrs) };
        inner.next().map(f)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = IterMut::len(self);
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for IterMut<'_, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        let context = inner.context();

        let f = |ptrs| unsafe { T::ptrs_to_refs_mut(context, ptrs) };
        inner.next_back().map(f)
    }
}

impl<T> ExactSizeIterator for IterMut<'_, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        IterMut::len(self)
    }
}

impl<T> FusedIterator for IterMut<'_, '_, T> where T: Soa + ?Sized {}
