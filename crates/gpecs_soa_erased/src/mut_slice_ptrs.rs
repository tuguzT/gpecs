use core::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use crate::{
    CovariantFieldDescriptors, ErasedSoaMutPtrs, ErasedSoaMutPtrsIter, ErasedSoaMutSlices,
    ErasedSoaPtrs, ErasedSoaSlicePtrs, ErasedSoaSlicePtrsIter, ErasedSoaSlices,
    data::{ErasedMutSlicePtr, ErasedSlicePtr},
    error::{DowncastError, SlicePtrsError, check_offset, check_offset_len},
    error::{check_ptr_align, check_sufficient_align, check_sufficient_len},
    ptr::slice::{CastConstPtr, MutSliceItemPtr},
    soa::{
        field::{
            FieldDescriptor, FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOwned,
            buffer_offsets,
        },
        traits::{AllocSoa, RawSoaContext, SliceMutPtrs},
    },
};

pub struct ErasedSoaMutSlicePtrs<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    len: usize,
    ptrs: ErasedSoaMutPtrs<D, P>,
}

impl<D, P> ErasedSoaMutSlicePtrs<D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: *mut [P::Item],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        let ptrs =
            unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, buffer, capacity, offset) };
        unsafe { Self::from_mut_ptrs(ptrs, len) }
    }

    #[inline]
    pub unsafe fn from_mut_ptrs(ptrs: ErasedSoaMutPtrs<D, P>, len: usize) -> Self {
        Self { len, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *mut [P::Item], usize, usize, usize) {
        let Self { ptrs, len } = self;
        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();
        (descriptors, buffer, capacity, offset, len)
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

    #[inline]
    pub fn cast_const(self) -> ErasedSoaSlicePtrs<D, CastConstPtr<P>> {
        let Self { ptrs, len } = self;
        let ptrs = ptrs.cast_const();
        unsafe { ErasedSoaSlicePtrs::from_ptrs(ptrs, len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSoaSlices<'a, D, CastConstPtr<P>> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedSoaMutSlices<'a, D, P> {
        unsafe { ErasedSoaMutSlices::from_mut_ptrs(self) }
    }
}

impl<D, P> ErasedSoaMutSlicePtrs<D, P>
where
    D: FieldDescriptorsOwned,
    P: MutSliceItemPtr,
{
    #[inline]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        descriptors: D,
        buffer: *mut [P::Item],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Result<Self, SlicePtrsError> {
        let mut offsets = buffer_offsets(descriptors.field_descriptors(), capacity);
        offsets.by_ref().try_for_each(|offset| {
            check_sufficient_align(offset?.desc.layout(), Layout::new::<P::Item>())
                .map_err(SlicePtrsError::from)
        })?;

        let layout = offsets.into_layout();
        check_sufficient_len(buffer.len() * size_of::<P::Item>(), layout.size())?;
        check_ptr_align(buffer.cast(), layout)?;
        check_offset(offset, capacity)?;
        check_offset_len(offset, len, capacity)?;

        let me = unsafe { Self::new_unchecked(descriptors, buffer, capacity, offset, len) };
        Ok(me)
    }
}

impl<D, P> ErasedSoaMutSlicePtrs<D, P>
where
    D: FieldDescriptorsOwned,
    P: MutSliceItemPtr<Item = MaybeUninit<u8>>,
{
    #[inline]
    pub unsafe fn downcast<T>(
        self,
        context: &T::Context,
    ) -> Result<SliceMutPtrs<'_, T>, DowncastError<Self>>
    where
        T: AllocSoa + ?Sized,
    {
        let Self { ptrs, len } = self;

        let result = unsafe { ptrs.downcast::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_mut_ptrs(ptrs, len) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;

        let slices = context.mut_slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<D, P> ErasedSoaMutSlicePtrs<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { ptrs, .. } = self;
        ptrs.as_buffer()
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> *mut [P::Item] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut_buffer()
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
    pub fn len(&self) -> usize {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, D, P, U> ErasedSoaMutSlicePtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaSlicePtrsIter<FieldDescriptorsIter<'a, D>, CastConstPtr<P>> {
        let Self { ref ptrs, len } = *self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaSlicePtrsIter::new_unchecked(ptrs, len) }
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> ErasedSoaMutSlicePtrsIter<FieldDescriptorsIter<'a, D>, P> {
        let Self { ref mut ptrs, len } = *self;

        let ptrs = ptrs.iter_mut();
        unsafe { ErasedSoaMutSlicePtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<D, P> Debug for ErasedSoaMutSlicePtrs<D, P>
where
    D: Debug + ?Sized,
    P: MutSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, ptrs } = self;
        f.debug_struct("ErasedSoaSliceMutPtrs")
            .field("len", len)
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<D, P> Clone for ErasedSoaMutSlicePtrs<D, P>
where
    D: Clone,
    P: MutSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { len, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { len, ptrs }
    }
}

impl<D, P> Copy for ErasedSoaMutSlicePtrs<D, P>
where
    D: Copy,
    P: MutSliceItemPtr,
{
}

impl<'a, D, P, U> IntoIterator for &'a ErasedSoaMutSlicePtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Item = ErasedSlicePtr<CastConstPtr<P>>;
    type IntoIter = ErasedSoaSlicePtrsIter<FieldDescriptorsIter<'a, D>, CastConstPtr<P>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P, U> IntoIterator for &'a mut ErasedSoaMutSlicePtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Item = ErasedMutSlicePtr<P>;
    type IntoIter = ErasedSoaMutSlicePtrsIter<FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<D, P, U> IntoIterator for ErasedSoaMutSlicePtrs<D, P>
where
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Item = ErasedMutSlicePtr<P>;
    type IntoIter = ErasedSoaMutSlicePtrsIter<D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, len } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaMutSlicePtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaMutSlicePtrs<D, P>
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

impl<D, P> CovariantFieldDescriptors for ErasedSoaMutSlicePtrs<D, P>
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

pub struct ErasedSoaMutSlicePtrsIter<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    len: usize,
    ptrs: ErasedSoaMutPtrsIter<D, P>,
}

impl<D, P> ErasedSoaMutSlicePtrsIter<D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn new_unchecked(ptrs: ErasedSoaMutPtrsIter<D, P>, len: usize) -> Self {
        Self { len, ptrs }
    }
}

impl<D, P> ErasedSoaMutSlicePtrsIter<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { ptrs, .. } = self;
        ptrs.as_buffer()
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> *mut [P::Item] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut_buffer()
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

impl<'a, D, P, U> ErasedSoaMutSlicePtrsIter<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub(super) fn entries(&'a self) -> ErasedSoaMutSlicePtrsIter<FieldDescriptorsIter<'a, D>, P> {
        let Self { ref ptrs, len } = *self;

        let ptrs = ptrs.entries();
        unsafe { ErasedSoaMutSlicePtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<D, P, U> Debug for ErasedSoaMutSlicePtrsIter<D, P>
where
    D: FieldDescriptorsOwned + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>> + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D, P> Clone for ErasedSoaMutSlicePtrsIter<D, P>
where
    D: Clone,
    P: MutSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { len, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { len, ptrs }
    }
}

impl<D, P, U> Iterator for ErasedSoaMutSlicePtrsIter<D, P>
where
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Item = ErasedMutSlicePtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ref mut ptrs, len } = *self;

        let data = ptrs.next()?;
        let item = unsafe { ErasedMutSlicePtr::from_parts(data, len) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ptrs, .. } = self;
        ptrs.size_hint()
    }
}

impl<D, P, U> ExactSizeIterator for ErasedSoaMutSlicePtrsIter<D, P>
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

impl<D, P, U> FusedIterator for ErasedSoaMutSlicePtrsIter<D, P>
where
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaMutSlicePtrsIter<D, P>
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

impl<D, P> CovariantFieldDescriptors for ErasedSoaMutSlicePtrsIter<D, P>
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
