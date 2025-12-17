use core::{
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
};

use crate::{
    slice::{RawIter, RawIterMut, SoaSlicePtrs, SoaSlicePtrsIndex, SoaSlices, SoaSlicesMut},
    traits::{MutPtrs, Ptrs, RawSoa, RawSoaContext, SliceMutPtrs, SlicePtrs},
    wrapper,
};

pub struct SoaSliceMutPtrs<'c, T>
where
    T: RawSoa + ?Sized,
{
    ptrs: wrapper::MutPtrs<'c, T>,
    context: &'c T::Context,
    len: usize,
}

impl<'c, T> SoaSliceMutPtrs<'c, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    pub fn new(context: &'c T::Context, slices: SliceMutPtrs<'c, T>) -> Self {
        let len = context.slice_mut_ptrs_len(&slices);
        let ptrs = context.slice_mut_ptrs_as_ptrs(slices);
        unsafe { Self::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub unsafe fn from_parts(context: &'c T::Context, ptrs: MutPtrs<'c, T>, len: usize) -> Self {
        let ptrs = wrapper::MutPtrs::new(ptrs);
        Self { ptrs, context, len }
    }

    #[inline]
    pub fn into_parts(self) -> (&'c T::Context, MutPtrs<'c, T>, usize) {
        let Self { context, ptrs, len } = self;
        (context, ptrs.into_inner(), len)
    }

    #[inline]
    pub fn cast_const(self) -> SoaSlicePtrs<'c, T> {
        let (context, ptrs, len) = self.into_parts();
        let ptrs = context.ptrs_cast_const(ptrs);
        unsafe { SoaSlicePtrs::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> SoaSlices<'c, 'a, T> {
        let (context, ptrs, len) = self.into_parts();
        let ptrs = context.ptrs_cast_const(ptrs);
        unsafe { SoaSlices::from_parts(context, ptrs, len) }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> SoaSlicesMut<'c, 'a, T> {
        let (context, ptrs, len) = self.into_parts();
        unsafe { SoaSlicesMut::from_parts(context, ptrs, len) }
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
        let ptrs = context.ptrs_cast_const(ptrs);
        (context, ptrs)
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> MutPtrs<'_, T> {
        let (_, ptrs) = self.as_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn as_mut_ptrs_with_context(&mut self) -> (&T::Context, MutPtrs<'_, T>) {
        let Self {
            context, ref ptrs, ..
        } = *self;

        let ptrs = ptrs.clone().into_inner();
        (context, ptrs)
    }

    #[inline]
    pub fn into_ptrs(self) -> Ptrs<'c, T> {
        let (_, ptrs) = self.into_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_ptrs_with_context(self) -> (&'c T::Context, Ptrs<'c, T>) {
        let Self { context, ptrs, .. } = self;

        let ptrs = ptrs.into_inner();
        let ptrs = context.ptrs_cast_const(ptrs);
        (context, ptrs)
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> MutPtrs<'c, T> {
        let (_, ptrs) = self.into_mut_ptrs_with_context();
        ptrs
    }

    #[inline]
    pub fn into_mut_ptrs_with_context(self) -> (&'c T::Context, MutPtrs<'c, T>) {
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
        let ptrs = context.ptrs_cast_const(ptrs);
        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn as_slice_mut_ptrs(&mut self) -> SliceMutPtrs<'_, T> {
        let (_, slices) = self.as_slice_mut_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn as_slice_mut_ptrs_with_context(&mut self) -> (&T::Context, SliceMutPtrs<'_, T>) {
        let Self {
            context,
            len,
            ref ptrs,
        } = *self;

        let ptrs = ptrs.clone().into_inner();
        let slices = context.slice_mut_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> SlicePtrs<'c, T> {
        let (_, slices) = self.into_slice_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn into_slice_ptrs_with_context(self) -> (&'c T::Context, SlicePtrs<'c, T>) {
        let Self { context, len, ptrs } = self;

        let ptrs = ptrs.into_inner();
        let ptrs = context.ptrs_cast_const(ptrs);
        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        (context, slices)
    }

    #[inline]
    pub fn into_slice_mut_ptrs(self) -> SliceMutPtrs<'c, T> {
        let (_, slices) = self.into_slice_mut_ptrs_with_context();
        slices
    }

    #[inline]
    pub fn into_slice_mut_ptrs_with_context(self) -> (&'c T::Context, SliceMutPtrs<'c, T>) {
        let Self { context, len, ptrs } = self;

        let ptrs = ptrs.into_inner();
        let slices = context.slice_mut_ptrs_from_raw_parts(ptrs, len);
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
    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> I::MutPtrs<'_>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.get_unchecked_mut_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn get_unchecked_mut_with_context<I>(
        &mut self,
        index: I,
    ) -> (&T::Context, I::MutPtrs<'_>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (context, slices) = self.as_slice_mut_ptrs_with_context();
        let ptrs = unsafe { index.get_unchecked_mut(context, slices) };
        (context, ptrs)
    }

    #[inline]
    pub unsafe fn into_get_unchecked<I>(self, index: I) -> I::Ptrs<'c>
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
    ) -> (&'c T::Context, I::Ptrs<'c>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (context, slices) = self.into_slice_ptrs_with_context();
        let ptrs = unsafe { index.get_unchecked(context, slices) };
        (context, ptrs)
    }

    #[inline]
    pub unsafe fn into_get_unchecked_mut<I>(self, index: I) -> I::MutPtrs<'c>
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (_, ptrs) = unsafe { self.into_get_unchecked_mut_with_context(index) };
        ptrs
    }

    #[inline]
    pub unsafe fn into_get_unchecked_mut_with_context<I>(
        self,
        index: I,
    ) -> (&'c T::Context, I::MutPtrs<'c>)
    where
        I: SoaSlicePtrsIndex<T>,
    {
        let (context, slices) = self.into_slice_mut_ptrs_with_context();
        let ptrs = unsafe { index.get_unchecked_mut(context, slices) };
        (context, ptrs)
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
    pub fn iter_mut(&mut self) -> RawIterMut<'_, T> {
        let (_, iter) = self.iter_mut_with_context();
        iter
    }

    #[inline]
    pub fn iter_mut_with_context(&mut self) -> (&T::Context, RawIterMut<'_, T>) {
        let (context, slices) = self.as_slice_mut_ptrs_with_context();
        (context, RawIterMut::new(context, slices))
    }

    #[inline]
    pub fn into_iter_with_context(self) -> (&'c T::Context, RawIterMut<'c, T>) {
        let (context, slices) = self.into_slice_mut_ptrs_with_context();
        (context, RawIterMut::new(context, slices))
    }
}

impl<'c, T> From<&'c T::Context> for SoaSliceMutPtrs<'c, T>
where
    T: RawSoa + ?Sized,
{
    #[inline]
    fn from(context: &'c T::Context) -> Self {
        let ptrs = context.ptrs_dangling_mut();
        unsafe { Self::from_parts(context, ptrs, 0) }
    }
}

impl<T> Debug for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'any> SlicePtrs<'any, T>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices = self.as_slice_ptrs();
        f.debug_tuple("SoaSliceMutPtrs").field(&slices).finish()
    }
}

impl<T> Clone for SoaSliceMutPtrs<'_, T>
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

impl<T> Copy for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    for<'any> MutPtrs<'any, T>: Copy,
{
}

impl<T> PartialEq for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: PartialEq,
    for<'any> MutPtrs<'any, T>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { ptrs, context, len } = self;
        *ptrs == other.ptrs && *context == other.context && *len == other.len
    }
}

impl<T> Eq for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Eq,
    for<'any> MutPtrs<'any, T>: Eq,
{
}

impl<T> PartialOrd for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: PartialOrd,
    for<'any> MutPtrs<'any, T>: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { ptrs, context, len } = self;

        match len.partial_cmp(&other.len) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match context.partial_cmp(&other.context) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        ptrs.partial_cmp(&other.ptrs)
    }
}

impl<T> Ord for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Ord,
    for<'any> MutPtrs<'any, T>: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { ptrs, context, len } = self;

        match len.cmp(&other.len) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match context.cmp(&other.context) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        ptrs.cmp(&other.ptrs)
    }
}

impl<T> Hash for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Hash,
    for<'any> MutPtrs<'any, T>: Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { ptrs, context, len } = self;

        len.hash(state);
        context.hash(state);
        ptrs.hash(state);
    }
}

impl<'r, T> IntoIterator for &'r SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
{
    type Item = Ptrs<'r, T>;
    type IntoIter = RawIter<'r, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'r, T> IntoIterator for &'r mut SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
{
    type Item = MutPtrs<'r, T>;
    type IntoIter = RawIterMut<'r, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'c, T> IntoIterator for SoaSliceMutPtrs<'c, T>
where
    T: RawSoa + ?Sized,
{
    type Item = MutPtrs<'c, T>;
    type IntoIter = RawIterMut<'c, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (_, iter) = self.into_iter_with_context();
        iter
    }
}

unsafe impl<T> Send for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Send,
    T::Fields: Send,
{
}

unsafe impl<T> Sync for SoaSliceMutPtrs<'_, T>
where
    T: RawSoa + ?Sized,
    T::Context: Sync,
    T::Fields: Sync,
{
}
