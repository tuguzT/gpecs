use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use crate::{
    slice::{RawIter, RawIterMut},
    traits::{
        MutPtrs, Ptrs, RawSoa, RefsMut, SliceMutPtrs, SlicePtrs, Slices, SlicesMut, Soa, SoaContext,
    },
};

#[repr(transparent)]
pub struct IterMut<'ctx, 'a, T>
where
    T: RawSoa + ?Sized + 'a,
{
    inner: RawIterMut<'ctx, T>,
    phantom: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl<'ctx, T> IterMut<'ctx, '_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub unsafe fn from_parts(context: &'ctx T::Context, slices: SliceMutPtrs<'ctx, T>) -> Self {
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
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'ctx, T> {
        let (_, ptrs) = self.as_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&'ctx T::Context, MutPtrs<'ctx, T>) {
        let Self { inner, .. } = self;
        inner.as_mut_ptrs_with_context()
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
    pub fn into_mut_ptrs(self) -> MutPtrs<'ctx, T> {
        let (_, ptrs) = self.into_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_mut_ptrs_with_context(self) -> (&'ctx T::Context, MutPtrs<'ctx, T>) {
        let Self { inner, .. } = self;
        inner.into_mut_ptrs_with_context()
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
    pub fn as_mut_slice_ptrs(&mut self) -> SliceMutPtrs<'ctx, T> {
        let (_, slices) = self.as_mut_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_mut_slice_ptrs_with_context(&mut self) -> (&'ctx T::Context, SliceMutPtrs<'ctx, T>) {
        let Self { inner, .. } = self;
        inner.as_mut_slice_ptrs_with_context()
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
    pub fn into_mut_slice_ptrs(self) -> SliceMutPtrs<'ctx, T> {
        let (_, slices) = self.into_mut_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn into_mut_slice_ptrs_with_context(self) -> (&'ctx T::Context, SliceMutPtrs<'ctx, T>) {
        let Self { inner, .. } = self;
        inner.into_mut_slice_ptrs_with_context()
    }

    #[inline]
    pub fn as_raw_iter(&self) -> &RawIterMut<'ctx, T> {
        let Self { inner, .. } = self;
        inner
    }

    #[inline]
    pub fn into_raw_iter(self) -> RawIter<'ctx, T> {
        let Self { inner, .. } = self;
        inner.cast_const()
    }

    #[inline]
    pub fn as_raw_iter_mut(&mut self) -> &mut RawIterMut<'ctx, T> {
        let Self { inner, .. } = self;
        inner
    }

    #[inline]
    pub fn into_raw_iter_mut(self) -> RawIterMut<'ctx, T> {
        let Self { inner, .. } = self;
        inner
    }
}

impl<'ctx, 'a, T> IterMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    #[inline]
    pub fn new(context: &'ctx T::Context, slices: SlicesMut<'ctx, 'a, T>) -> Self {
        let slices = context.mut_slices_as_mut_slice_ptrs(slices);
        unsafe { Self::from_parts(context, slices) }
    }

    #[inline]
    pub fn into_slices(self) -> SlicesMut<'ctx, 'a, T> {
        let (_, slices) = self.into_slices_with_context();
        slices
    }

    #[inline]
    #[doc(alias = "into_parts")]
    pub fn into_slices_with_context(self) -> (&'ctx T::Context, SlicesMut<'ctx, 'a, T>) {
        let Self { inner, .. } = self;

        let (context, slices) = inner.into_mut_slice_ptrs_with_context();
        let slices = unsafe { context.mut_slice_ptrs_to_mut_slices(slices) };
        (context, slices)
    }
}

impl<'a, T> IterMut<'_, '_, T>
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

impl<T, U> AsRef<[U]> for IterMut<'_, '_, T>
where
    T: ?Sized,
    for<'a> T: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Into<&'a [U]>,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_slices().into()
    }
}

impl<T> Debug for IterMut<'_, '_, T>
where
    T: ?Sized,
    for<'a> T: Soa<'a>,
    for<'ctx, 'a> Slices<'ctx, 'a, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slices();
        f.debug_tuple("IterMut").field(&slices).finish()
    }
}

impl<'ctx, 'a, T> Iterator for IterMut<'ctx, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    type Item = RefsMut<'ctx, 'a, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        let context = inner.context();

        let f = |ptrs| unsafe { context.mut_ptrs_to_mut_refs(ptrs) };
        inner.next().map(f)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = IterMut::len(self);
        (len, Some(len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;
        let context = inner.context();

        let f = |ptrs| unsafe { context.mut_ptrs_to_mut_refs(ptrs) };
        inner.next_back().map(f)
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'_, 'a, T>
where
    T: Soa<'a> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        IterMut::len(self)
    }
}

impl<'a, T> FusedIterator for IterMut<'_, 'a, T> where T: Soa<'a> + ?Sized {}
