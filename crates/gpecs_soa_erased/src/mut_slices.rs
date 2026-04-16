use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use crate::{
    CovariantFieldDescriptors, ErasedSoaMutSlicePtrs, ErasedSoaMutSlicePtrsIter, ErasedSoaSlices,
    ErasedSoaSlicesIter,
    data::{ErasedMutSlice, ErasedSlice},
    error::{DowncastError, SlicePtrsError},
    ptr::slice::{CastConst, MutSliceItemPtr},
    soa::{
        field::{
            FieldDescriptor, FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOutput,
            FieldDescriptorsOwned,
        },
        traits::{AllocSoa, SlicesMut, Soa, SoaContext},
    },
};

pub struct ErasedSoaMutSlices<'a, D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    phantom: PhantomData<&'a mut [P::Item]>,
    ptrs: ErasedSoaMutSlicePtrs<D, P>,
}

impl<'a, D, P> ErasedSoaMutSlices<'a, D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: &'a mut [P::Item],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        let ptrs = unsafe {
            ErasedSoaMutSlicePtrs::new_unchecked(descriptors, buffer, capacity, offset, len)
        };
        unsafe { Self::from_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaMutSlicePtrs<D, P>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, &'a mut [P::Item], usize, usize, usize) {
        let Self { ptrs, .. } = self;
        let (descriptors, buffer, capacity, offset, len) = ptrs.into_parts();

        let buffer = unsafe { buffer.as_mut_unchecked() };
        (descriptors, buffer, capacity, offset, len)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaMutSlicePtrs<D, P> {
        let Self { ptrs, .. } = self;
        ptrs
    }

    #[inline]
    pub unsafe fn map_descriptors<N, F>(self, f: F) -> ErasedSoaMutSlices<'a, N, P>
    where
        F: FnOnce(D) -> N,
    {
        let Self { ptrs, .. } = self;

        let ptrs = unsafe { ptrs.map_descriptors(f) };
        unsafe { ErasedSoaMutSlices::from_ptrs(ptrs) }
    }
}

impl<'a, D, P> ErasedSoaMutSlices<'a, D, P>
where
    D: FieldDescriptorsOwned,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn new(
        descriptors: D,
        buffer: &'a mut [P::Item],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Result<Self, SlicePtrsError> {
        let ptrs = ErasedSoaMutSlicePtrs::new(descriptors, buffer, capacity, offset, len)?;

        let me = unsafe { Self::from_ptrs(ptrs) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<T>(
        self,
        context: &T::Context,
    ) -> Result<SlicesMut<'_, 'a, T>, DowncastError<Self>>
    where
        T: AllocSoa + Soa<'a> + ?Sized,
    {
        let Self { ptrs, .. } = self;

        let result = unsafe { ptrs.downcast::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let slices = result.map_err(|err| err.map_value(into_self))?;

        let slices = unsafe { context.mut_slice_ptrs_to_mut_slices(slices) };
        Ok(slices)
    }
}

impl<D, P> ErasedSoaMutSlices<'_, D, P>
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
    pub fn descriptors(&self) -> &D {
        let Self { ptrs, .. } = self;
        ptrs.descriptors()
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, D, P> ErasedSoaMutSlices<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaSlicesIter<'a, FieldDescriptorsIter<'a, D>, CastConst<P>> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaSlicesIter::from_ptrs(ptrs) }
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> ErasedSoaMutSlicesIter<'a, FieldDescriptorsIter<'a, D>, P> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter_mut();
        unsafe { ErasedSoaMutSlicesIter::from_ptrs(ptrs) }
    }
}

impl<D, P> Debug for ErasedSoaMutSlices<'_, D, P>
where
    D: Debug + ?Sized,
    P: MutSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_struct("ErasedSoaSlicesMut")
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaMutSlices<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedSlice<'a, CastConst<P>>;
    type IntoIter = ErasedSoaSlicesIter<'a, FieldDescriptorsIter<'a, D>, CastConst<P>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for &'a mut ErasedSoaMutSlices<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedMutSlice<'a, P>;
    type IntoIter = ErasedSoaMutSlicesIter<'a, FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, D, P> IntoIterator for ErasedSoaMutSlices<'a, D, P>
where
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
    P: MutSliceItemPtr,
{
    type Item = ErasedMutSlice<'a, P>;
    type IntoIter = ErasedSoaMutSlicesIter<'a, D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaMutSlicesIter::from_ptrs(ptrs) }
    }
}

impl<'a, D, P> From<ErasedSoaMutSlices<'a, D, P>> for ErasedSoaSlices<'a, D, CastConst<P>>
where
    P: MutSliceItemPtr,
{
    #[inline]
    fn from(slices: ErasedSoaMutSlices<'a, D, P>) -> Self {
        let (descriptors, buffer, capacity, offset, len) = slices.into_parts();
        unsafe { Self::new_unchecked(descriptors, buffer, capacity, offset, len) }
    }
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaMutSlices<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedSoaMutSlices<'_, D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}

pub struct ErasedSoaMutSlicesIter<'a, D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    phantom: PhantomData<&'a mut [P::Item]>,
    ptrs: ErasedSoaMutSlicePtrsIter<D, P>,
}

impl<D, P> ErasedSoaMutSlicesIter<'_, D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaMutSlicePtrsIter<D, P>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }
}

impl<D, P> ErasedSoaMutSlicesIter<'_, D, P>
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
    pub fn slice_len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.slice_len()
    }

    #[inline]
    pub fn descriptors(&self) -> &D {
        let Self { ptrs, .. } = self;
        ptrs.descriptors()
    }
}

impl<'a, D, P> ErasedSoaMutSlicesIter<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaMutSlicesIter<'a, FieldDescriptorsIter<'a, D>, P> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaMutSlicesIter::from_ptrs(ptrs) }
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaMutSlicesIter<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedMutSlice<'a, P>;
    type IntoIter = ErasedSoaMutSlicesIter<'a, FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> Debug for ErasedSoaMutSlicesIter<'_, D, P>
where
    D: FieldDescriptorsOwned + ?Sized,
    P: MutSliceItemPtr<Item: Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<'a, D, P> Iterator for ErasedSoaMutSlicesIter<'a, D, P>
where
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedMutSlice<'a, P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ptrs, .. } = self;

        let item = unsafe { ptrs.next()?.deref_mut() };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ptrs, .. } = self;
        ptrs.size_hint()
    }
}

impl<D, P> ExactSizeIterator for ErasedSoaMutSlicesIter<'_, D, P>
where
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D, P> FusedIterator for ErasedSoaMutSlicesIter<'_, D, P>
where
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr,
{
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaMutSlicesIter<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedSoaMutSlicesIter<'_, D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}
