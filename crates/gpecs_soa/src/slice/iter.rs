use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use crate::{
    slice::RawIter,
    traits::{Ptrs, RawSoa, Refs, SlicePtrs, Slices, Soa, SoaContext, SoaOwned},
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
    T: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn new(context: &'ctx T::Context, slices: Slices<'ctx, 'a, T>) -> Self {
        let slices = context.slices_as_slice_ptrs(slices);
        unsafe { Self::from_parts(context, slices) }
    }

    #[inline]
    pub fn into_slices(self) -> Slices<'ctx, 'a, T> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    #[doc(alias = "into_parts")]
    pub fn into_slices_with_context(self) -> (&'ctx T::Context, Slices<'ctx, 'a, T>) {
        let Self { inner, .. } = self;

        let (context, slices) = inner.into_slice_ptrs_with_context();
        let slices = unsafe { context.slice_ptrs_to_slices(slices) };
        (context, slices)
    }
}

impl<'a, T> Iter<'_, '_, T>
where
    T: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn as_slices(&'a self) -> Slices<'a, 'a, T> {
        let (_, slices) = self.as_slices_with_context();
        slices
    }

    #[inline]
    pub fn as_slices_with_context(&'a self) -> (&'a T::Context, Slices<'a, 'a, T>) {
        let Self { inner, .. } = self;

        let (context, slices) = inner.as_slice_ptrs_with_context();
        let slices = unsafe { context.slice_ptrs_to_slices(slices) };
        let slices = T::Context::upcast_slices(slices);
        (context, slices)
    }
}

impl<T, U> AsRef<[U]> for Iter<'_, '_, T>
where
    T: SoaOwned + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Into<&'a [U]>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices().into()
    }
}

impl<T> Debug for Iter<'_, '_, T>
where
    T: SoaOwned + ?Sized,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Debug,
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
    T: Soa<'a> + ?Sized,
{
    type Item = Refs<'ctx, 'a, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        let context = inner.context();

        let f = |ptrs| unsafe { context.ptrs_to_refs(ptrs) };
        inner.next().map(f)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = Iter::len(self);
        (len, Some(len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        let context = inner.context();

        let f = |ptrs| unsafe { context.ptrs_to_refs(ptrs) };
        inner.next_back().map(f)
    }
}

impl<'a, T> ExactSizeIterator for Iter<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        Iter::len(self)
    }
}

impl<'a, T> FusedIterator for Iter<'_, 'a, T> where T: Soa<'a> + ?Sized {}
