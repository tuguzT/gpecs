use core::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use crate::{
    erased::{
        CovariantFieldDescriptors, ErasedSoaMutPtrs, ErasedSoaMutPtrsIter, ErasedSoaPtrs,
        ErasedSoaSlicePtrs, ErasedSoaSlicePtrsIter, ErasedSoaSlices, ErasedSoaSlicesMut,
        error::{ErasedSoaIntoValueError, ErasedSoaSlicePtrsError, check_offset, check_offset_len},
    },
    error::{check_ptr_align, check_sufficient_align, check_sufficient_len},
    field::{ErasedFieldSliceMutPtr, ErasedFieldSlicePtr, field_slice_from_raw_parts_mut},
    soa::{
        field::{
            FieldDescriptor, FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOwned,
            buffer_offsets,
        },
        traits::{AllocSoa, RawSoaContext, SliceMutPtrs},
    },
    storage::AddressableUnit,
};

pub struct ErasedSoaSliceMutPtrs<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    len: usize,
    ptrs: ErasedSoaMutPtrs<D, A>,
}

impl<D, A> ErasedSoaSliceMutPtrs<D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: *mut [MaybeUninit<A>],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        let ptrs =
            unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, buffer, capacity, offset) };
        unsafe { Self::from_mut_ptrs(ptrs, len) }
    }

    #[inline]
    pub unsafe fn from_mut_ptrs(ptrs: ErasedSoaMutPtrs<D, A>, len: usize) -> Self {
        Self { len, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *mut [MaybeUninit<A>], usize, usize, usize) {
        let Self { ptrs, len } = self;
        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();
        (descriptors, buffer, capacity, offset, len)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D, A> {
        let Self { ptrs, .. } = self;
        ptrs.cast_const()
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> ErasedSoaMutPtrs<D, A> {
        let Self { ptrs, .. } = self;
        ptrs
    }

    #[inline]
    pub fn cast_const(self) -> ErasedSoaSlicePtrs<D, A> {
        let Self { ptrs, len } = self;
        unsafe { ErasedSoaSlicePtrs::from_ptrs(ptrs.cast_const(), len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSoaSlices<'a, D, A> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedSoaSlicesMut<'a, D, A> {
        unsafe { ErasedSoaSlicesMut::from_mut_ptrs(self) }
    }
}

impl<D, A> ErasedSoaSliceMutPtrs<D, A>
where
    A: AddressableUnit,
    D: FieldDescriptorsOwned,
{
    #[inline]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        descriptors: D,
        buffer: *mut [MaybeUninit<A>],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Result<Self, ErasedSoaSlicePtrsError> {
        let mut offsets = buffer_offsets(descriptors.field_descriptors(), capacity);
        offsets.by_ref().try_for_each(|offset| {
            let desc = offset?.field_descriptor;
            check_sufficient_align(desc.layout(), Layout::new::<A>())
                .map_err(ErasedSoaSlicePtrsError::from)
        })?;

        let layout = offsets.into_layout();
        check_sufficient_len(buffer.len() * size_of::<A>(), layout.size())?;
        check_ptr_align(buffer.cast(), layout)?;
        check_offset(offset, capacity)?;
        check_offset_len(offset, len, capacity)?;

        let me = unsafe { Self::new_unchecked(descriptors, buffer, capacity, offset, len) };
        Ok(me)
    }
}

impl<D> ErasedSoaSliceMutPtrs<D, u8>
where
    D: FieldDescriptorsOwned,
{
    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<SliceMutPtrs<'_, T>, ErasedSoaIntoValueError<Self>>
    where
        T: AllocSoa + ?Sized,
    {
        let Self { ptrs, len } = self;

        let result = unsafe { ptrs.try_into::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_mut_ptrs(ptrs, len) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;

        let slices = context.mut_slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<D, A> ErasedSoaSliceMutPtrs<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [MaybeUninit<A>] {
        let Self { ptrs, .. } = self;
        ptrs.as_buffer()
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> *mut [MaybeUninit<A>] {
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

impl<'a, D, A> ErasedSoaSliceMutPtrs<D, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaSlicePtrsIter<FieldDescriptorsIter<'a, D>, A> {
        let Self { ref ptrs, len } = *self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaSlicePtrsIter::new_unchecked(ptrs, len) }
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> ErasedSoaSliceMutPtrsIter<FieldDescriptorsIter<'a, D>, A> {
        let Self { ref mut ptrs, len } = *self;

        let ptrs = ptrs.iter_mut();
        unsafe { ErasedSoaSliceMutPtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<D, A> Debug for ErasedSoaSliceMutPtrs<D, A>
where
    A: AddressableUnit,
    D: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, ptrs } = self;
        f.debug_struct("ErasedSoaSliceMutPtrs")
            .field("len", len)
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<D, A> Clone for ErasedSoaSliceMutPtrs<D, A>
where
    A: AddressableUnit,
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { len, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { len, ptrs }
    }
}

impl<D, A> Copy for ErasedSoaSliceMutPtrs<D, A>
where
    A: AddressableUnit,
    D: Copy,
{
}

impl<'a, D, A> IntoIterator for &'a ErasedSoaSliceMutPtrs<D, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Item = ErasedFieldSlicePtr<A>;
    type IntoIter = ErasedSoaSlicePtrsIter<FieldDescriptorsIter<'a, D>, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, A> IntoIterator for &'a mut ErasedSoaSliceMutPtrs<D, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Item = ErasedFieldSliceMutPtr<A>;
    type IntoIter = ErasedSoaSliceMutPtrsIter<FieldDescriptorsIter<'a, D>, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<D, A> IntoIterator for ErasedSoaSliceMutPtrs<D, A>
where
    A: AddressableUnit,
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    type Item = ErasedFieldSliceMutPtr<A>;
    type IntoIter = ErasedSoaSliceMutPtrsIter<D::IntoIter, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, len } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaSliceMutPtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<'a, D, A> FieldDescriptors<'a> for ErasedSoaSliceMutPtrs<D, A>
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

impl<D, A> CovariantFieldDescriptors for ErasedSoaSliceMutPtrs<D, A>
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

#[inline]
pub unsafe fn slice_from_raw_parts_mut<D, A>(
    data: ErasedSoaMutPtrs<D, A>,
    len: usize,
) -> ErasedSoaSliceMutPtrs<D, A>
where
    A: AddressableUnit,
{
    unsafe { ErasedSoaSliceMutPtrs::from_mut_ptrs(data, len) }
}

pub struct ErasedSoaSliceMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    len: usize,
    ptrs: ErasedSoaMutPtrsIter<D, A>,
}

impl<D, A> ErasedSoaSliceMutPtrsIter<D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub(super) unsafe fn new_unchecked(ptrs: ErasedSoaMutPtrsIter<D, A>, len: usize) -> Self {
        Self { len, ptrs }
    }
}

impl<D, A> ErasedSoaSliceMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [MaybeUninit<A>] {
        let Self { ptrs, .. } = self;
        ptrs.as_buffer()
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> *mut [MaybeUninit<A>] {
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

impl<'a, D, A> ErasedSoaSliceMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    pub(super) fn entries(&'a self) -> ErasedSoaSliceMutPtrsIter<FieldDescriptorsIter<'a, D>, A> {
        let Self { ref ptrs, len } = *self;

        let ptrs = ptrs.entries();
        unsafe { ErasedSoaSliceMutPtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<D, A> Debug for ErasedSoaSliceMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: FieldDescriptorsOwned + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D, A> Clone for ErasedSoaSliceMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { len, ref ptrs } = *self;

        let ptrs = ptrs.clone();
        Self { len, ptrs }
    }
}

impl<D, A> Iterator for ErasedSoaSliceMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
    type Item = ErasedFieldSliceMutPtr<A>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ref mut ptrs, len } = *self;

        let data = ptrs.next()?;
        let item = unsafe { field_slice_from_raw_parts_mut(data, len) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ptrs, .. } = self;
        ptrs.size_hint()
    }
}

impl<D, A> ExactSizeIterator for ErasedSoaSliceMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D, A> FusedIterator for ErasedSoaSliceMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
}

impl<'a, D, A> FieldDescriptors<'a> for ErasedSoaSliceMutPtrsIter<D, A>
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

impl<D, A> CovariantFieldDescriptors for ErasedSoaSliceMutPtrsIter<D, A>
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
