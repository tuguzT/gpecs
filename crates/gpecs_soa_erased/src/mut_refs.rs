use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use crate::{
    CovariantFieldLayouts, ErasedSoaMutPtrs, ErasedSoaMutPtrsIter, ErasedSoaRefs,
    ErasedSoaRefsIter,
    data::{ErasedMutRef, ErasedRef},
    error::{DowncastError, PtrsError},
    ptr::slice::{CastConst, MutSliceItemPtr},
    soa::{
        field::{FieldLayouts, FieldLayoutsIter, FieldLayoutsOutput, FieldLayoutsOwned},
        layout::WithLayout,
        traits::{AllocSoa, RefsMut, Soa, SoaContext},
    },
};

pub struct ErasedSoaMutRefs<'a, D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    phantom: PhantomData<&'a mut [P::Item]>,
    ptrs: ErasedSoaMutPtrs<D, P>,
}

impl<'a, D, P> ErasedSoaMutRefs<'a, D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn new_unchecked(
        layouts: D,
        buffer: &'a mut [P::Item],
        capacity: usize,
        offset: usize,
    ) -> Self {
        let ptrs = unsafe { ErasedSoaMutPtrs::new_unchecked(layouts, buffer, capacity, offset) };
        unsafe { Self::from_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaMutPtrs<D, P>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, &'a mut [P::Item], usize, usize) {
        let Self { ptrs, .. } = self;
        let (layouts, buffer, capacity, offset) = ptrs.into_parts();

        let buffer = unsafe { buffer.as_mut_unchecked() };
        (layouts, buffer, capacity, offset)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaMutPtrs<D, P> {
        let Self { ptrs, .. } = self;
        ptrs
    }

    #[inline]
    pub unsafe fn map_layouts<N, F>(self, f: F) -> ErasedSoaMutRefs<'a, N, P>
    where
        F: FnOnce(D) -> N,
    {
        let Self { ptrs, .. } = self;

        let ptrs = unsafe { ptrs.map_layouts(f) };
        unsafe { ErasedSoaMutRefs::from_ptrs(ptrs) }
    }
}

impl<'a, D, P> ErasedSoaMutRefs<'a, D, P>
where
    D: FieldLayoutsOwned,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn new(
        layouts: D,
        buffer: &'a mut [P::Item],
        capacity: usize,
        offset: usize,
    ) -> Result<Self, PtrsError> {
        let ptrs = ErasedSoaMutPtrs::new(layouts, buffer, capacity, offset)?;

        let me = unsafe { Self::from_ptrs(ptrs) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<T>(
        self,
        context: &T::Context,
    ) -> Result<RefsMut<'_, 'a, T>, DowncastError<Self>>
    where
        T: AllocSoa + Soa<'a> + ?Sized,
    {
        let Self { ptrs, .. } = self;

        let result = unsafe { ptrs.downcast::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;

        let refs = unsafe { context.mut_ptrs_to_mut_refs(ptrs) };
        Ok(refs)
    }
}

impl<D, P> ErasedSoaMutRefs<'_, D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_buffer();
        unsafe { buffer.as_ref_unchecked() }
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> &mut [P::Item] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_mut_buffer();
        unsafe { buffer.as_mut_unchecked() }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.offset()
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { ptrs, .. } = self;
        ptrs.layouts()
    }
}

impl<'a, D, P> ErasedSoaMutRefs<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaRefsIter<'a, FieldLayoutsIter<'a, D>, CastConst<P>> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> ErasedSoaMutRefsIter<'a, FieldLayoutsIter<'a, D>, P> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter_mut();
        unsafe { ErasedSoaMutRefsIter::from_ptrs(ptrs) }
    }
}

impl<D, P> Debug for ErasedSoaMutRefs<'_, D, P>
where
    D: Debug + ?Sized,
    P: MutSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_struct("ErasedSoaRefsMut")
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaMutRefs<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedRef<'a, CastConst<P>>;
    type IntoIter = ErasedSoaRefsIter<'a, FieldLayoutsIter<'a, D>, CastConst<P>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for &'a mut ErasedSoaMutRefs<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedMutRef<'a, P>;
    type IntoIter = ErasedSoaMutRefsIter<'a, FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, D, P> IntoIterator for ErasedSoaMutRefs<'a, D, P>
where
    D: IntoIterator<Item: WithLayout>,
    P: MutSliceItemPtr,
{
    type Item = ErasedMutRef<'a, P>;
    type IntoIter = ErasedSoaMutRefsIter<'a, D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaMutRefsIter::from_ptrs(ptrs) }
    }
}

impl<'a, D, P> From<ErasedSoaMutRefs<'a, D, P>> for ErasedSoaRefs<'a, D, CastConst<P>>
where
    P: MutSliceItemPtr,
{
    #[inline]
    fn from(refs: ErasedSoaMutRefs<'a, D, P>) -> Self {
        let (layouts, buffer, capacity, offset) = refs.into_parts();
        unsafe { Self::new_unchecked(layouts, buffer, capacity, offset) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedSoaMutRefs<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedSoaMutRefs<'_, D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}

pub struct ErasedSoaMutRefsIter<'a, D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    phantom: PhantomData<&'a mut [P::Item]>,
    ptrs: ErasedSoaMutPtrsIter<D, P>,
}

impl<D, P> ErasedSoaMutRefsIter<'_, D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_ptrs(ptrs: ErasedSoaMutPtrsIter<D, P>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }
}

impl<D, P> ErasedSoaMutRefsIter<'_, D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_buffer();
        unsafe { buffer.as_ref_unchecked() }
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> &mut [P::Item] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_mut_buffer();
        unsafe { buffer.as_mut_unchecked() }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.offset()
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { ptrs, .. } = self;
        ptrs.layouts()
    }
}

impl<'a, D, P> ErasedSoaMutRefsIter<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaMutRefsIter<'a, FieldLayoutsIter<'a, D>, P> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaMutRefsIter::from_ptrs(ptrs) }
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaMutRefsIter<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedMutRef<'a, P>;
    type IntoIter = ErasedSoaMutRefsIter<'a, FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> Debug for ErasedSoaMutRefsIter<'_, D, P>
where
    D: FieldLayoutsOwned + ?Sized,
    P: MutSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<'a, D, P> Iterator for ErasedSoaMutRefsIter<'a, D, P>
where
    D: Iterator<Item: WithLayout> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedMutRef<'a, P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ptrs, .. } = self;

        let item = unsafe { ptrs.next()?.as_mut_unchecked() };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ptrs, .. } = self;
        ptrs.size_hint()
    }
}

impl<D, P> ExactSizeIterator for ErasedSoaMutRefsIter<'_, D, P>
where
    D: ExactSizeIterator<Item: WithLayout> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D, P> FusedIterator for ErasedSoaMutRefsIter<'_, D, P>
where
    D: FusedIterator<Item: WithLayout> + ?Sized,
    P: MutSliceItemPtr,
{
}

impl<'a, D, P> FieldLayouts<'a> for ErasedSoaMutRefsIter<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedSoaMutRefsIter<'_, D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}
