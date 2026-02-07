use core::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use crate::{
    erased::{
        CovariantFieldDescriptors, ErasedSoaPtrs, ErasedSoaPtrsIter, ErasedSoaSliceMutPtrs,
        ErasedSoaSlices,
        error::{ErasedSoaIntoValueError, ErasedSoaSlicePtrsError, check_offset, check_offset_len},
    },
    error::{check_ptr_align, check_sufficient_align, check_sufficient_len},
    field::ErasedFieldSlicePtr,
    slice_item_ptr::ConstSliceItemPtr,
    soa::{
        field::{
            FieldDescriptor, FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOwned,
            buffer_offsets,
        },
        traits::{AllocSoa, RawSoaContext, SlicePtrs},
    },
    storage::AddressableUnit,
};

pub struct ErasedSoaSlicePtrs<D, P, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    len: usize,
    ptrs: ErasedSoaPtrs<D, P, A>,
}

impl<D, P, A> ErasedSoaSlicePtrs<D, P, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: *const [MaybeUninit<A>],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Self {
        let ptrs = unsafe { ErasedSoaPtrs::new_unchecked(descriptors, buffer, capacity, offset) };
        unsafe { Self::from_ptrs(ptrs, len) }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaPtrs<D, P, A>, len: usize) -> Self {
        Self { len, ptrs }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *const [MaybeUninit<A>], usize, usize, usize) {
        let Self { ptrs, len } = self;
        let (descriptors, buffer, capacity, offset) = ptrs.into_parts();
        (descriptors, buffer, capacity, offset, len)
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D, P, A> {
        let Self { ptrs, .. } = self;
        ptrs
    }

    #[inline]
    pub fn cast_mut<E>(self) -> ErasedSoaSliceMutPtrs<D, E, A> {
        let Self { ptrs, len } = self;
        let ptrs = ptrs.cast_mut();
        unsafe { ErasedSoaSliceMutPtrs::from_mut_ptrs(ptrs, len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSoaSlices<'a, D, P, A> {
        unsafe { ErasedSoaSlices::from_ptrs(self) }
    }
}

impl<D, P, A> ErasedSoaSlicePtrs<D, P, A>
where
    A: AddressableUnit,
    D: FieldDescriptorsOwned,
{
    #[inline]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        descriptors: D,
        buffer: *const [MaybeUninit<A>],
        capacity: usize,
        offset: usize,
        len: usize,
    ) -> Result<Self, ErasedSoaSlicePtrsError> {
        let mut offsets = buffer_offsets(descriptors.field_descriptors(), capacity);
        offsets.by_ref().try_for_each(|offset| {
            check_sufficient_align(offset?.desc.layout(), Layout::new::<A>())
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

impl<D, P> ErasedSoaSlicePtrs<D, P, u8>
where
    D: FieldDescriptorsOwned,
{
    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<SlicePtrs<'_, T>, ErasedSoaIntoValueError<Self>>
    where
        T: AllocSoa + ?Sized,
    {
        let Self { ptrs, len } = self;

        let result = unsafe { ptrs.try_into::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs, len) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;

        let slices = context.slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<D, P, A> ErasedSoaSlicePtrs<D, P, A>
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

impl<'a, D, P, A> ErasedSoaSlicePtrs<D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaSlicePtrsIter<FieldDescriptorsIter<'a, D>, P, A> {
        let Self { ref ptrs, len } = *self;

        let ptrs = ptrs.iter();
        unsafe { ErasedSoaSlicePtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<D, P, A> Debug for ErasedSoaSlicePtrs<D, P, A>
where
    A: AddressableUnit,
    D: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { len, ptrs } = self;
        f.debug_struct("ErasedSoaSlicePtrs")
            .field("len", len)
            .field("ptrs", &ptrs)
            .finish()
    }
}

impl<D, P, A> Clone for ErasedSoaSlicePtrs<D, P, A>
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

impl<D, P, A> Copy for ErasedSoaSlicePtrs<D, P, A>
where
    A: AddressableUnit,
    D: Copy,
{
}

impl<'a, D, P, A> IntoIterator for &'a ErasedSoaSlicePtrs<D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Item = ErasedFieldSlicePtr<P>;
    type IntoIter = ErasedSoaSlicePtrsIter<FieldDescriptorsIter<'a, D>, P, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P, A> IntoIterator for ErasedSoaSlicePtrs<D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    type Item = ErasedFieldSlicePtr<P>;
    type IntoIter = ErasedSoaSlicePtrsIter<D::IntoIter, P, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { ptrs, len } = self;

        let ptrs = ptrs.into_iter();
        unsafe { ErasedSoaSlicePtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<'a, D, P, A> FieldDescriptors<'a> for ErasedSoaSlicePtrs<D, P, A>
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

impl<D, P, A> CovariantFieldDescriptors for ErasedSoaSlicePtrs<D, P, A>
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

pub struct ErasedSoaSlicePtrsIter<D, P, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    len: usize,
    ptrs: ErasedSoaPtrsIter<D, P, A>,
}

impl<D, P, A> ErasedSoaSlicePtrsIter<D, P, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub(super) unsafe fn new_unchecked(ptrs: ErasedSoaPtrsIter<D, P, A>, len: usize) -> Self {
        Self { len, ptrs }
    }
}

impl<D, P, A> ErasedSoaSlicePtrsIter<D, P, A>
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

impl<'a, D, P, A> ErasedSoaSlicePtrsIter<D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: FieldDescriptors<'a> + ?Sized,
{
    #[inline]
    pub(super) fn entries(&'a self) -> ErasedSoaSlicePtrsIter<FieldDescriptorsIter<'a, D>, P, A> {
        let Self { ref ptrs, len } = *self;

        let ptrs = ptrs.entries();
        unsafe { ErasedSoaSlicePtrsIter::new_unchecked(ptrs, len) }
    }
}

impl<D, P, A> Debug for ErasedSoaSlicePtrsIter<D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>> + Debug,
    D: FieldDescriptorsOwned + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D, P, A> Clone for ErasedSoaSlicePtrsIter<D, P, A>
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

impl<D, P, A> Iterator for ErasedSoaSlicePtrsIter<D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
    type Item = ErasedFieldSlicePtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ref mut ptrs, len } = *self;

        let data = ptrs.next()?;
        let item = unsafe { ErasedFieldSlicePtr::from_parts(data, len) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { ptrs, .. } = self;
        ptrs.size_hint()
    }
}

impl<D, P, A> ExactSizeIterator for ErasedSoaSlicePtrsIter<D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { ptrs, .. } = self;
        ptrs.len()
    }
}

impl<D, P, A> FusedIterator for ErasedSoaSlicePtrsIter<D, P, A>
where
    A: AddressableUnit,
    P: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
{
}

impl<'a, D, P, A> FieldDescriptors<'a> for ErasedSoaSlicePtrsIter<D, P, A>
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

impl<D, P, A> CovariantFieldDescriptors for ErasedSoaSlicePtrsIter<D, P, A>
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
