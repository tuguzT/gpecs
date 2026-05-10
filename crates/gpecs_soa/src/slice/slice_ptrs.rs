use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use crate::{
    slice::{RawIter, SoaSliceMutPtrs, SoaSlicePtrsIndex, SoaSlices},
    traits::{Ptrs, RawSoa, RawSoaContext, SlicePtrs},
    wrapper,
};

pub struct SoaSlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    ptrs: wrapper::Ptrs<'ctx, T>,
    context: &'ctx T::Context,
    len: usize,
}

impl<'ctx, T> SoaSlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub fn new(context: &'ctx T::Context, slices: SlicePtrs<'ctx, T>) -> Self {
        let len = context.slice_ptrs_len(&slices);
        let ptrs = context.slice_ptrs_as_ptrs(slices);
        unsafe { Self::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub unsafe fn from_parts(context: &'ctx T::Context, ptrs: Ptrs<'ctx, T>, len: usize) -> Self {
        let ptrs = wrapper::Ptrs::new(ptrs);
        Self { ptrs, context, len }
    }

    #[inline]
    pub fn into_parts(self) -> (&'ctx T::Context, Ptrs<'ctx, T>, usize) {
        let Self { context, ptrs, len } = self;
        (context, ptrs.into_inner(), len)
    }

    #[inline]
    pub fn cast_mut(self) -> SoaSliceMutPtrs<'ctx, T> {
        let (context, ptrs, len) = self.into_parts();
        let ptrs = context.ptrs_cast_mut(ptrs);
        unsafe { SoaSliceMutPtrs::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> SoaSlices<'ctx, 'a, T> {
        let (context, ptrs, len) = self.into_parts();
        unsafe { SoaSlices::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let Self { context, .. } = *self;
        context
    }

    #[inline]
    pub fn as_ptrs(&self) -> Ptrs<'_, T> {
        let (_, ptrs) = self.as_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_context(&self) -> (&T::Context, Ptrs<'_, T>) {
        let Self {
            context, ref ptrs, ..
        } = *self;

        let ptrs = ptrs.clone().into_inner();
        (context, ptrs)
    }

    #[inline]
    pub fn into_ptrs(self) -> Ptrs<'ctx, T> {
        let (_, ptrs) = self.into_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'ctx T::Context, Ptrs<'ctx, T>) {
        let Self { context, ptrs, .. } = self;

        let ptrs = ptrs.into_inner();
        (context, ptrs)
    }

    #[inline]
    pub fn as_slice_ptrs(&self) -> SlicePtrs<'_, T> {
        let (_, slices) = self.as_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_ptrs_with_context(&self) -> (&T::Context, SlicePtrs<'_, T>) {
        let Self {
            context,
            len,
            ref ptrs,
        } = *self;

        let ptrs = ptrs.clone().into_inner();
        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> SlicePtrs<'ctx, T> {
        let (_, slices) = self.into_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(self) -> (&'ctx T::Context, SlicePtrs<'ctx, T>) {
        let Self { context, len, ptrs } = self;

        let ptrs = ptrs.into_inner();
        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::Ptrs<'_>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.get_unchecked_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_with_context<I>(&self, index: I) -> (&T::Context, I::Ptrs<'_>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (context, slices) = self.as_slice_ptrs_with_context();
        let ptrs = unsafe { index.get_unchecked(context, slices) };
        (context, ptrs)
    }

    #[inline]
    pub unsafe fn into_get_unchecked<I>(self, index: I) -> I::Ptrs<'ctx>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.into_get_unchecked_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_with_context<I>(
        self,
        index: I,
    ) -> (&'ctx T::Context, I::Ptrs<'ctx>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (context, slices) = self.into_slice_ptrs_with_context();
        let ptrs = unsafe { index.get_unchecked(context, slices) };
        (context, ptrs)
    }

    #[inline]
    pub unsafe fn split_at_unchecked(self, mid: usize) -> (Self, Self) {
        let Self { ptrs, context, len } = self;

        let ptrs = ptrs.into_inner();
        let left = unsafe { Self::from_parts(context, ptrs.clone(), mid) };
        let right = unsafe {
            Self::from_parts(context, context.ptrs_add(ptrs, mid), len.unchecked_sub(mid))
        };
        (left, right)
    }

    #[inline]
    pub fn iter(&self) -> RawIter<'_, T> {
        let (_, iter) = self.iter_with_context();
        iter
    }

    #[inline]
    pub fn iter_with_context(&self) -> (&T::Context, RawIter<'_, T>) {
        let (context, slices) = self.as_slice_ptrs_with_context();
        (context, RawIter::new(context, slices))
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'ctx T::Context, RawIter<'ctx, T>) {
        let (context, slices) = self.into_slice_ptrs_with_context();
        (context, RawIter::new(context, slices))
    }
}

impl<'ctx, T> From<&'ctx T::Context> for SoaSlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(context: &'ctx T::Context) -> Self {
        let ptrs = context.ptrs_dangling();
        unsafe { Self::from_parts(context, ptrs, 0) }
    }
}

impl<T> Debug for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> SlicePtrs<'ctx, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slice_ptrs();
        f.debug_tuple("SoaSlicePtrs").field(&slices).finish()
    }
}

impl<T> Clone for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            ref ptrs,
            context,
            len,
        } = *self;

        let ptrs = ptrs.clone();
        Self { ptrs, context, len }
    }
}

impl<T> Copy for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'ctx> Ptrs<'ctx, T>: Copy,
{
}

impl<T> PartialEq for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: PartialEq,
    for<'ctx> Ptrs<'ctx, T>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { ptrs, context, len } = self;

        let other = (&other.len, &other.context, &other.ptrs);
        (len, context, ptrs) == other
    }
}

impl<T> Eq for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Eq,
    for<'ctx> Ptrs<'ctx, T>: Eq,
{
}

impl<T> PartialOrd for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: PartialOrd,
    for<'ctx> Ptrs<'ctx, T>: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { ptrs, context, len } = self;

        let other = (&other.len, &other.context, &other.ptrs);
        (len, context, ptrs).partial_cmp(&other)
    }
}

impl<T> Ord for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Ord,
    for<'ctx> Ptrs<'ctx, T>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { ptrs, context, len } = self;

        let other = (&other.len, &other.context, &other.ptrs);
        (len, context, ptrs).cmp(&other)
    }
}

impl<T> Hash for SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Hash,
    for<'ctx> Ptrs<'ctx, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { ptrs, context, len } = self;
        (len, context, ptrs).hash(state);
    }
}

impl<'a, T> IntoIterator for &'a SoaSlicePtrs<'_, T>
where
    T: RawSoa + ?Sized,
{
    type Item = Ptrs<'a, T>;
    type IntoIter = RawIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'ctx, T> IntoIterator for SoaSlicePtrs<'ctx, T>
where
    T: RawSoa + ?Sized,
{
    type Item = Ptrs<'ctx, T>;
    type IntoIter = RawIter<'ctx, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}
