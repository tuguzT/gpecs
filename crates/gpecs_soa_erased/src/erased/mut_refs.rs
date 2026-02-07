use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    mem::MaybeUninit,
    slice,
};

use crate::{
    erased::{
        CovariantFieldDescriptors, ErasedSoaMutPtrs, ErasedSoaMutPtrsIter, ErasedSoaPtrs,
        ErasedSoaRefsIter,
        error::{ErasedSoaIntoValueError, ErasedSoaPtrsError},
    },
    field::{ErasedFieldRef, ErasedFieldRefMut},
    slice_item_ptr::{CastConstPtr, MutSliceItemPtr},
    soa::{
        field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOwned},
        traits::{AllocSoa, RefsMut, Soa, SoaContext},
    },
    storage::AddressableUnit,
};

pub struct ErasedSoaRefsMut<'a, D, P, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<&'a mut [MaybeUninit<A>]>,
    ptrs: ErasedSoaMutPtrs<D, P, A>,
}

impl<'a, D, P, A> ErasedSoaRefsMut<'a, D, P, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: &'a mut [MaybeUninit<A>],
        capacity: usize,
        offset: usize,
    ) -> Self {
        let ptrs =
            unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, buffer, capacity, offset) };
        unsafe { Self::from_mut_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_mut_ptrs(ptrs: ErasedSoaMutPtrs<D, P, A>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, &'a mut [MaybeUninit<A>], usize, usize) {
        let Self { ptrs, .. } = self;
        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();

        let buffer = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        (descriptors, buffer, capacity, offset)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D, P, A> {
        let Self { ptrs, .. } = self;
        ptrs.cast_const()
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> ErasedSoaMutPtrs<D, P, A> {
        let Self { ptrs, .. } = self;
        ptrs
    }
}

impl<'a, D, P, A> ErasedSoaRefsMut<'a, D, P, A>
where
    A: AddressableUnit,
    D: FieldDescriptorsOwned,
{
    #[inline]
    pub fn new(
        descriptors: D,
        buffer: &'a mut [MaybeUninit<A>],
        capacity: usize,
        offset: usize,
    ) -> Result<Self, ErasedSoaPtrsError> {
        let ptrs = ErasedSoaMutPtrs::new(descriptors, buffer, capacity, offset)?;

        let me = unsafe { Self::from_mut_ptrs(ptrs) };
        Ok(me)
    }
}

impl<'a, D, P> ErasedSoaRefsMut<'a, D, P, u8>
where
    D: FieldDescriptorsOwned,
{
    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<RefsMut<'_, 'a, T>, ErasedSoaIntoValueError<Self>>
    where
        T: AllocSoa + Soa<'a> + ?Sized,
    {
        let Self { ptrs, .. } = self;

        let result = unsafe { ptrs.try_into::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_mut_ptrs(ptrs) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;

        let refs = unsafe { context.mut_ptrs_to_mut_refs(ptrs) };
        Ok(refs)
    }
}

impl<D, P, A> ErasedSoaRefsMut<'_, D, P, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> &[MaybeUninit<A>] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> &mut [MaybeUninit<A>] {
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

impl<'a, D, P, A> ErasedSoaRefsMut<'_, D, P, A>
where
    A: AddressableUnit,
    P: MutSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    pub fn iter(
        &'a self,
    ) -> ErasedSoaRefsIter<'a, FieldDescriptorsIter<'a, D>, CastConstPtr<P>, A> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaRefsIter::from_ptrs(ptrs) }
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> ErasedSoaRefsMutIter<'a, FieldDescriptorsIter<'a, D>, P, A> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.iter_mut();
        unsafe { ErasedSoaRefsMutIter::from_ptrs(ptrs) }
    }
}

impl<D, P, A> Debug for ErasedSoaRefsMut<'_, D, P, A>
where
    A: AddressableUnit,
    D: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptrs, .. } = self;
        f.debug_struct("ErasedSoaRefsMut")
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<'a, D, P, A> IntoIterator for &'a ErasedSoaRefsMut<'_, D, P, A>
where
    A: AddressableUnit,
    P: MutSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Item = ErasedFieldRef<'a, CastConstPtr<P>>;
    type IntoIter = ErasedSoaRefsIter<'a, FieldDescriptorsIter<'a, D>, CastConstPtr<P>, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P, A> IntoIterator for &'a mut ErasedSoaRefsMut<'_, D, P, A>
where
    A: AddressableUnit,
    P: MutSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Item = ErasedFieldRefMut<'a, P>;
    type IntoIter = ErasedSoaRefsMutIter<'a, FieldDescriptorsIter<'a, D>, P, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, D, P, A> IntoIterator for ErasedSoaRefsMut<'a, D, P, A>
where
    A: AddressableUnit,
    P: MutSliceItemPtr<Item = MaybeUninit<A>>,
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    type Item = ErasedFieldRefMut<'a, P>;
    type IntoIter = ErasedSoaRefsMutIter<'a, D::IntoIter, P, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaRefsMutIter::from_ptrs(ptrs) }
    }
}

impl<'a, D, P, A> FieldDescriptors<'a> for ErasedSoaRefsMut<'_, D, P, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }
}

impl<D, P, A> CovariantFieldDescriptors for ErasedSoaRefsMut<'_, D, P, A>
where
    A: AddressableUnit,
    D: CovariantFieldDescriptors + ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        D::upcast_field_descriptors(from)
    }
}

pub struct ErasedSoaRefsMutIter<'a, D, P, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<&'a mut [MaybeUninit<A>]>,
    ptrs: ErasedSoaMutPtrsIter<D, P, A>,
}

impl<D, P, A> ErasedSoaRefsMutIter<'_, D, P, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub(super) unsafe fn from_ptrs(ptrs: ErasedSoaMutPtrsIter<D, P, A>) -> Self {
        let phantom = PhantomData;
        Self { phantom, ptrs }
    }
}

impl<D, P, A> ErasedSoaRefsMutIter<'_, D, P, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> &[MaybeUninit<A>] {
        let Self { ptrs, .. } = self;

        let buffer = ptrs.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> &mut [MaybeUninit<A>] {
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

impl<'a, D, P, A> ErasedSoaRefsMutIter<'_, D, P, A>
where
    A: AddressableUnit,
    P: MutSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    pub(super) fn entries(&'a self) -> ErasedSoaRefsMutIter<'a, FieldDescriptorsIter<'a, D>, P, A> {
        let Self { ptrs, .. } = self;

        let ptrs = ptrs.entries();
        unsafe { ErasedSoaRefsMutIter::from_ptrs(ptrs) }
    }
}

impl<D, P, A> Debug for ErasedSoaRefsMutIter<'_, D, P, A>
where
    A: AddressableUnit,
    P: MutSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptorsOwned + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<'a, D, P, A> Iterator for ErasedSoaRefsMutIter<'a, D, P, A>
where
    A: AddressableUnit,
    P: MutSliceItemPtr<Item = MaybeUninit<A>>,
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
    type Item = ErasedFieldRefMut<'a, P>;

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

impl<D, P, A> ExactSizeIterator for ErasedSoaRefsMutIter<'_, D, P, A>
where
    A: AddressableUnit,
    P: MutSliceItemPtr<Item = MaybeUninit<A>>,
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D, P, A> FusedIterator for ErasedSoaRefsMutIter<'_, D, P, A>
where
    A: AddressableUnit,
    P: MutSliceItemPtr<Item = MaybeUninit<A>>,
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
}

impl<'a, D, P, A> FieldDescriptors<'a> for ErasedSoaRefsMutIter<'_, D, P, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { ptrs, .. } = self;
        ptrs.field_descriptors()
    }
}

impl<D, P, A> CovariantFieldDescriptors for ErasedSoaRefsMutIter<'_, D, P, A>
where
    A: AddressableUnit,
    D: CovariantFieldDescriptors + ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        D::upcast_field_descriptors(from)
    }
}
