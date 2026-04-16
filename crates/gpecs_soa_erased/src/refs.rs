use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    slice,
};

use crate::{
    CovariantFieldDescriptors, ErasedSoaPtrs, ErasedSoaPtrsIter,
    data::ErasedRef,
    error::{DowncastError, PtrsError},
    ptr::slice::ConstSliceItemPtr,
    soa::{
        field::{
            FieldDescriptor, FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOutput,
            FieldDescriptorsOwned,
        },
        traits::{AllocSoa, Refs, Soa, SoaContext},
    },
};

pub struct ErasedSoaRefs<'a, D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    phantom: PhantomData<&'a [P::Item]>,
    ptrs: ErasedSoaPtrs<D, P>,
}

impl<'a, D, P> ErasedSoaRefs<'a, D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: &'a [P::Item],
        capacity: usize,
        offset: usize,
    ) -> Self {
        let ptrs = unsafe { ErasedSoaPtrs::new_unchecked(descriptors, buffer, capacity, offset) };
        unsafe { Self::from_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaPtrs<D, P>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, &'a [P::Item], usize, usize) {
        let Self { ptrs, .. } = self;
        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();

        let buffer = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        (descriptors, buffer, capacity, offset)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D, P> {
        let Self { ptrs, .. } = self;
        ptrs
    }

    #[inline]
    pub unsafe fn map_descriptors<N, F>(self, f: F) -> ErasedSoaRefs<'a, N, P>
    where
        F: FnOnce(D) -> N,
    {
        let Self { ptrs, .. } = self;

        let ptrs = unsafe { ptrs.map_descriptors(f) };
        unsafe { ErasedSoaRefs::from_ptrs(ptrs) }
    }
}

impl<'a, D, P> ErasedSoaRefs<'a, D, P>
where
    D: FieldDescriptorsOwned,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn new(
        descriptors: D,
        buffer: &'a [P::Item],
        capacity: usize,
        offset: usize,
    ) -> Result<Self, PtrsError> {
        let ptrs = ErasedSoaPtrs::new(descriptors, buffer, capacity, offset)?;
        let me = unsafe { Self::from_ptrs(ptrs) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<T>(
        self,
        context: &T::Context,
    ) -> Result<Refs<'_, 'a, T>, DowncastError<Self>>
    where
        T: AllocSoa + Soa<'a> + ?Sized,
    {
        let Self { ptrs, .. } = self;

        let result = unsafe { ptrs.downcast::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;

        let refs = unsafe { context.ptrs_to_refs(ptrs) };
        Ok(refs)
    }
}

impl<D, P> ErasedSoaRefs<'_, D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
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
    pub fn descriptors(&self) -> &D {
        let Self { ptrs, .. } = self;
        ptrs.descriptors()
    }
}

impl<'a, D, P> ErasedSoaRefs<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaRefsIter<'a, FieldDescriptorsIter<'a, D>, P> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }
}

impl<D, P> Debug for ErasedSoaRefs<'_, D, P>
where
    D: Debug + ?Sized,
    P: ConstSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_struct("ErasedSoaRefs")
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<D, P> Clone for ErasedSoaRefs<'_, D, P>
where
    D: Clone,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { phantom, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { phantom, ptrs }
    }
}

impl<D, P> Copy for ErasedSoaRefs<'_, D, P>
where
    D: Copy,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaRefs<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedRef<'a, P>;
    type IntoIter = ErasedSoaRefsIter<'a, FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for ErasedSoaRefs<'a, D, P>
where
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
    P: ConstSliceItemPtr,
{
    type Item = ErasedRef<'a, P>;
    type IntoIter = ErasedSoaRefsIter<'a, D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaRefs<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedSoaRefs<'_, D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}

pub struct ErasedSoaRefsIter<'a, D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    phantom: PhantomData<&'a [P::Item]>,
    ptrs: ErasedSoaPtrsIter<D, P>,
}

impl<D, P> ErasedSoaRefsIter<'_, D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_ptrs(ptrs: ErasedSoaPtrsIter<D, P>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }
}

impl<D, P> ErasedSoaRefsIter<'_, D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
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
    pub fn descriptors(&self) -> &D {
        let Self { ptrs, .. } = self;
        ptrs.descriptors()
    }
}

impl<'a, D, P> ErasedSoaRefsIter<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaRefsIter<'a, FieldDescriptorsIter<'a, D>, P> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaRefsIter<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedRef<'a, P>;
    type IntoIter = ErasedSoaRefsIter<'a, FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> Debug for ErasedSoaRefsIter<'_, D, P>
where
    D: FieldDescriptorsOwned + ?Sized,
    P: ConstSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<D, P> Clone for ErasedSoaRefsIter<'_, D, P>
where
    D: Clone,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { phantom, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { phantom, ptrs }
    }
}

impl<'a, D, P> Iterator for ErasedSoaRefsIter<'a, D, P>
where
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedRef<'a, P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ptrs, .. } = self;

        let item = unsafe { ptrs.next()?.deref() };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ptrs, .. } = self;
        ptrs.size_hint()
    }
}

impl<D, P> ExactSizeIterator for ErasedSoaRefsIter<'_, D, P>
where
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D, P> FusedIterator for ErasedSoaRefsIter<'_, D, P>
where
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaRefsIter<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedSoaRefsIter<'_, D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}
