use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    mem::MaybeUninit,
    slice,
};

use crate::{
    CovariantFieldDescriptors, ErasedSoaMutPtrs, ErasedSoaMutPtrsIter, ErasedSoaPtrs,
    ErasedSoaRefsIter,
    data::{ErasedMutRef, ErasedRef},
    error::{DowncastError, PtrsError},
    ptr::slice::{CastConstPtr, MutSliceItemPtr},
    soa::{
        field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOwned},
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
        descriptors: D,
        buffer: &'a mut [P::Item],
        capacity: usize,
        offset: usize,
    ) -> Self {
        let ptrs =
            unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, buffer, capacity, offset) };
        unsafe { Self::from_mut_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_mut_ptrs(ptrs: ErasedSoaMutPtrs<D, P>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, &'a mut [P::Item], usize, usize) {
        let Self { ptrs, .. } = self;
        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();

        let buffer = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        (descriptors, buffer, capacity, offset)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D, CastConstPtr<P>> {
        let Self { ptrs, .. } = self;
        ptrs.cast_const()
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> ErasedSoaMutPtrs<D, P> {
        let Self { ptrs, .. } = self;
        ptrs
    }
}

impl<'a, D, P> ErasedSoaMutRefs<'a, D, P>
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
    ) -> Result<Self, PtrsError> {
        let ptrs = ErasedSoaMutPtrs::new(descriptors, buffer, capacity, offset)?;

        let me = unsafe { Self::from_mut_ptrs(ptrs) };
        Ok(me)
    }
}

impl<'a, D, P> ErasedSoaMutRefs<'a, D, P>
where
    D: FieldDescriptorsOwned,
    P: MutSliceItemPtr<Item = MaybeUninit<u8>>,
{
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
        let into_self = |ptrs| unsafe { Self::from_mut_ptrs(ptrs) };
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
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> &mut [P::Item] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_mut_buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
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
}

impl<'a, D, P, U> ErasedSoaMutRefs<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaRefsIter<'a, FieldDescriptorsIter<'a, D>, CastConstPtr<P>> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> ErasedSoaMutRefsIter<'a, FieldDescriptorsIter<'a, D>, P> {
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

impl<'a, D, P, U> IntoIterator for &'a ErasedSoaMutRefs<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Item = ErasedRef<'a, CastConstPtr<P>>;
    type IntoIter = ErasedSoaRefsIter<'a, FieldDescriptorsIter<'a, D>, CastConstPtr<P>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P, U> IntoIterator for &'a mut ErasedSoaMutRefs<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Item = ErasedMutRef<'a, P>;
    type IntoIter = ErasedSoaMutRefsIter<'a, FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, D, P, U> IntoIterator for ErasedSoaMutRefs<'a, D, P>
where
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
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

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaMutRefs<'_, D, P>
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

impl<D, P> CovariantFieldDescriptors for ErasedSoaMutRefs<'_, D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        D::upcast_field_descriptors(from)
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
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> &mut [P::Item] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_mut_buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
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
}

impl<'a, D, P, U> ErasedSoaMutRefsIter<'_, D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub(super) fn entries(&'a self) -> ErasedSoaMutRefsIter<'a, FieldDescriptorsIter<'a, D>, P> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.entries();
        unsafe { ErasedSoaMutRefsIter::from_ptrs(ptrs) }
    }
}

impl<D, P, U> Debug for ErasedSoaMutRefsIter<'_, D, P>
where
    D: FieldDescriptorsOwned + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
    U: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<'a, D, P, U> Iterator for ErasedSoaMutRefsIter<'a, D, P>
where
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Item = ErasedMutRef<'a, P>;

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

impl<D, P, U> ExactSizeIterator for ErasedSoaMutRefsIter<'_, D, P>
where
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D, P, U> FusedIterator for ErasedSoaMutRefsIter<'_, D, P>
where
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaMutRefsIter<'_, D, P>
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

impl<D, P> CovariantFieldDescriptors for ErasedSoaMutRefsIter<'_, D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        D::upcast_field_descriptors(from)
    }
}
