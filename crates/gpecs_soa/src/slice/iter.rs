use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use crate::{
    slice::RawIter,
    traits::{Ptrs, RawSoa, SlicePtrs, Soa},
};

#[repr(transparent)]
pub struct Iter<'ctx, 'a, T>
where
    T: RawSoa + ?Sized + 'a,
{
    inner: RawIter<'ctx, T>,
    phantom: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl<'ctx, T> Iter<'ctx, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub unsafe fn from_parts(context: &'ctx T::Context, slices: SlicePtrs<'ctx, T>) -> Self {
        Self {
            inner: RawIter::new(context, slices),
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
    pub fn context(&self) -> &'ctx T::Context {
        let Self { inner, .. } = self;
        inner.context()
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'ctx, T> {
        let (_, ptrs) = self.as_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&'ctx T::Context, Ptrs<'ctx, T>) {
        let Self { inner, .. } = self;
        inner.as_ptrs_with_context()
    }

    #[inline]
    pub fn into_ptrs(self) -> Ptrs<'ctx, T> {
        let (_, ptrs) = self.into_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'ctx T::Context, Ptrs<'ctx, T>) {
        let Self { inner, .. } = self;
        inner.into_ptrs_with_context()
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'ctx, T> {
        let (_, slices) = self.as_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&'ctx T::Context, SlicePtrs<'ctx, T>) {
        let Self { inner, .. } = self;
        inner.as_slice_ptrs_with_context()
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> SlicePtrs<'ctx, T> {
        let (_, slices) = self.into_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(self) -> (&'ctx T::Context, SlicePtrs<'ctx, T>) {
        let Self { inner, .. } = self;
        inner.into_slice_ptrs_with_context()
    }

    #[inline]
    pub fn as_raw_iter(&self) -> &RawIter<'ctx, T> {
        let Self { inner, .. } = self;
        inner
    }

    #[inline]
    pub fn as_raw_iter_mut(&mut self) -> &mut RawIter<'ctx, T> {
        let Self { inner, .. } = self;
        inner
    }

    #[inline]
    pub fn into_raw_iter(self) -> RawIter<'ctx, T> {
        let Self { inner, .. } = self;
        inner
    }
}

impl<'ctx, 'a, T> Iter<'ctx, 'a, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    pub fn new(context: &'ctx T::Context, slices: T::Slices<'ctx, 'a>) -> Self {
        let slices = T::slices_as_slice_ptrs(context, slices);
        unsafe { Self::from_parts(context, slices) }
    }

    #[inline]
    pub fn as_slices(&self) -> T::Slices<'_, '_> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&self) -> (&'ctx T::Context, T::Slices<'_, '_>) {
        let Self { inner, .. } = self;

        let (context, slices) = inner.as_slice_ptrs_with_context();
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        let slices = T::upcast_slices(slices);
        (context, slices)
    }

    #[inline]
    pub fn into_slices(self) -> T::Slices<'ctx, 'a> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    #[doc(alias = "into_parts")]
    pub fn into_slices_with_context(self) -> (&'ctx T::Context, T::Slices<'ctx, 'a>) {
        let Self { inner, .. } = self;

        let (context, slices) = inner.into_slice_ptrs_with_context();
        let slices = unsafe { T::slice_ptrs_to_slices(context, slices) };
        (context, slices)
    }
}

impl<T, U> AsRef<[U]> for Iter<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'ctx, 'a> T::Slices<'ctx, 'a>: Into<&'a [U]>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices().into()
    }
}

impl<T> Debug for Iter<'_, '_, T>
where
    T: Soa + ?Sized,
    for<'ctx, 'a> T::Slices<'ctx, 'a>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("Iter").field(&slices).finish()
    }
}

impl<T> Clone for Iter<'_, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { ref inner, phantom } = *self;

        let inner = inner.clone();
        Self { inner, phantom }
    }
}

impl<'ctx, 'a, T> Iterator for Iter<'ctx, 'a, T>
where
    T: Soa + ?Sized,
{
    type Item = T::Refs<'ctx, 'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        let context = inner.context();

        let f = |ptrs| unsafe { T::ptrs_to_refs(context, ptrs) };
        inner.next().map(f)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = Iter::len(self);
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for Iter<'_, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        let context = inner.context();

        let f = |ptrs| unsafe { T::ptrs_to_refs(context, ptrs) };
        inner.next_back().map(f)
    }
}

impl<T> ExactSizeIterator for Iter<'_, '_, T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        Iter::len(self)
    }
}

impl<T> FusedIterator for Iter<'_, '_, T> where T: Soa + ?Sized {}
