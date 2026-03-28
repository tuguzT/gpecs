use core::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
    ptr,
};

use crate::{
    CovariantFieldDescriptors, ErasedSoa, ErasedSoaMutPtrs, ErasedSoaRefs,
    assert::{assert_descriptors, check_downcast},
    dangling::{Dangling, dangling},
    data::ErasedPtr,
    error::{
        DowncastError, FromFieldsDescriptorsError, InsufficientAlignError, PtrsError, check_offset,
        check_ptr_align, check_sufficient_align, check_sufficient_len,
    },
    layout::bytes_to_items,
    ptr::slice::{CastMutPtr, ConstSliceItemPtr},
    soa::{
        field::{
            BufferOffset, BufferOffsets, FieldDescriptor, FieldDescriptors, FieldDescriptorsIter,
            FieldDescriptorsOutput, FieldDescriptorsOwned, buffer_offsets,
        },
        traits::{AllocSoa, AllocSoaContext, Ptrs, RawSoaContext},
    },
    storage::AlignedStorageFromLayout,
};

pub struct ErasedSoaPtrs<D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    buffer: *const [P::Item],
    capacity: usize,
    offset: usize,
    descriptors: D,
}

impl<D, P> ErasedSoaPtrs<D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: *const [P::Item],
        capacity: usize,
        offset: usize,
    ) -> Self {
        Self {
            buffer,
            capacity,
            offset,
            descriptors,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *const [P::Item], usize, usize) {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;
        (descriptors, buffer, capacity, offset)
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedSoaMutPtrs<D, CastMutPtr<P>> {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;

        let buffer = buffer.cast_mut();
        unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, buffer, capacity, offset) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSoaRefs<'a, D, P> {
        unsafe { ErasedSoaRefs::from_ptrs(self) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { offset, .. } = self;

        let offset = unsafe { offset.unchecked_add(count) };
        Self { offset, ..self }
    }
}

impl<D, P> ErasedSoaPtrs<D, P>
where
    D: FieldDescriptorsOwned,
    P: ConstSliceItemPtr,
{
    #[inline]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        descriptors: D,
        buffer: *const [P::Item],
        capacity: usize,
        offset: usize,
    ) -> Result<Self, PtrsError> {
        check_offset(offset, capacity)?;

        let mut offsets = buffer_offsets(descriptors.field_descriptors(), capacity);
        offsets.by_ref().try_for_each(|offset| {
            check_sufficient_align(offset?.desc.layout(), Layout::new::<P::Item>())
                .map_err(PtrsError::from)
        })?;

        let layout = offsets.into_layout();
        check_ptr_align(buffer.cast(), layout)?;

        let buffer_layout = Layout::array::<P::Item>(buffer.len())?;
        check_sufficient_len(buffer_layout.size(), layout.size())?;

        let me = unsafe { Self::new_unchecked(descriptors, buffer, capacity, offset) };
        Ok(me)
    }

    #[inline]
    pub fn dangling(descriptors: D) -> Result<Self, InsufficientAlignError> {
        let Dangling { addr, capacity } = dangling::<_, P::Item>(descriptors.field_descriptors())?;

        let data = ptr::without_provenance(addr);
        let buffer = ptr::slice_from_raw_parts(data, 0);

        let me = unsafe { Self::new_unchecked(descriptors, buffer, capacity, 0) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn downcast<T>(
        self,
        context: &T::Context,
    ) -> Result<Ptrs<'_, T>, DowncastError<Self>>
    where
        T: AllocSoa + ?Sized,
    {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
        } = self;

        let actual = descriptors.field_descriptors();
        let expected = context.field_descriptors();
        if let Err(error) = check_downcast(actual, expected, capacity) {
            return Err(DowncastError::new(self, error));
        }

        let ptrs = unsafe { context.ptrs_from_buffer(buffer.cast(), capacity) };
        let ptrs = unsafe { context.ptrs_add(ptrs, offset) };
        Ok(ptrs)
    }
}

impl<D, P> ErasedSoaPtrs<D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { capacity, .. } = *self;
        capacity
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { offset, .. } = *self;
        offset
    }

    #[inline]
    pub fn descriptors(&self) -> &D {
        let Self { descriptors, .. } = self;
        descriptors
    }
}

impl<'a, D, P> ErasedSoaPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<'e, E>(&'a self, origin: &'e ErasedSoaPtrs<E, P>) -> isize
    where
        E: FieldDescriptors<'e> + ?Sized,
    {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
        } = *self;

        assert_eq!(buffer, origin.buffer);
        assert_eq!(capacity, origin.capacity);
        assert_descriptors(descriptors.field_descriptors(), origin.field_descriptors());

        let offset = offset.cast_signed();
        let origin_offset = origin.offset().cast_signed();
        offset.wrapping_sub(origin_offset)
    }

    #[inline]
    pub fn iter(&'a self) -> ErasedSoaPtrsIter<FieldDescriptorsIter<'a, D>, P> {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
        } = *self;

        let descriptors = descriptors.field_descriptors().into_iter();
        let inner = buffer_offsets(descriptors, capacity);
        unsafe { ErasedSoaPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, P, U> ErasedSoaPtrs<D, P>
where
    D: FieldDescriptorsOwned + Clone,
    P: ConstSliceItemPtr<Item = MaybeUninit<U>>,
    U: Copy,
{
    #[inline]
    pub unsafe fn read<T>(
        &self,
    ) -> Result<ErasedSoa<T, D, P::Ptrs>, FromFieldsDescriptorsError<T::Error>>
    where
        T: AlignedStorageFromLayout<Item = U>,
    {
        let fields = self.iter().map(|ptr| unsafe { ptr.deref() });
        let descriptors = self.descriptors().clone();
        ErasedSoa::try_from_fields_descriptors(fields, descriptors)
    }
}

impl<D, P> Debug for ErasedSoaPtrs<D, P>
where
    D: Debug + ?Sized,
    P: ConstSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            buffer,
            capacity,
            offset,
            descriptors,
        } = self;

        f.debug_struct("ErasedSoaPtrs")
            .field("buffer", buffer)
            .field("capacity", capacity)
            .field("offset", offset)
            .field("descriptors", &descriptors)
            .finish()
    }
}

impl<D, P> Clone for ErasedSoaPtrs<D, P>
where
    D: Clone,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
        } = *self;

        let descriptors = descriptors.clone();
        unsafe { Self::new_unchecked(descriptors, buffer, capacity, offset) }
    }
}

impl<D, P> Copy for ErasedSoaPtrs<D, P>
where
    D: Copy,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedPtr<P>;
    type IntoIter = ErasedSoaPtrsIter<FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> IntoIterator for ErasedSoaPtrs<D, P>
where
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
    P: ConstSliceItemPtr,
{
    type Item = ErasedPtr<P>;
    type IntoIter = ErasedSoaPtrsIter<D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;

        let descriptors = descriptors.into_iter();
        let inner = buffer_offsets(descriptors, capacity);
        unsafe { ErasedSoaPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { descriptors, .. } = self;
        descriptors.field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedSoaPtrs<D, P>
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

pub struct ErasedSoaPtrsIter<D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    buffer: *const [P::Item],
    offset: usize,
    inner: BufferOffsets<D>,
}

impl<D, P> ErasedSoaPtrsIter<D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn new_unchecked(
        inner: BufferOffsets<D>,
        buffer: *const [P::Item],
        offset: usize,
    ) -> Self {
        Self {
            buffer,
            offset,
            inner,
        }
    }
}

impl<D, P> ErasedSoaPtrsIter<D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { inner, .. } = self;
        inner.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { offset, .. } = *self;
        offset
    }

    #[inline]
    pub fn descriptors(&self) -> &D {
        let Self { inner, .. } = self;
        inner.as_inner()
    }

    #[inline]
    unsafe fn field_ptr_from_buffer_offset(&self, offset: BufferOffset) -> ErasedPtr<P> {
        let BufferOffset { desc, offset, .. } = offset;
        let index = bytes_to_items::<P::Item>(offset);

        let Self { buffer, offset, .. } = *self;
        let ptr = unsafe { P::from_slice(buffer, index) };
        unsafe { ErasedPtr::from_parts(desc.layout(), ptr).add(offset) }
    }
}

impl<D, P> ErasedSoaPtrsIter<D, P>
where
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn next_unchecked(&mut self) -> ErasedPtr<P> {
        let Self { inner, .. } = self;

        let offset = unsafe { inner.next_unchecked() };
        unsafe { self.field_ptr_from_buffer_offset(offset) }
    }
}

impl<'a, D, P> ErasedSoaPtrsIter<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub(super) fn entries(&'a self) -> ErasedSoaPtrsIter<FieldDescriptorsIter<'a, D>, P> {
        let Self {
            ref inner,
            buffer,
            offset,
        } = *self;

        let layout = inner.layout();
        let capacity = inner.capacity();
        let fields = inner.as_inner().field_descriptors().into_iter();

        let inner = unsafe { BufferOffsets::from_parts(layout, capacity, fields) };
        unsafe { ErasedSoaPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, P> Debug for ErasedSoaPtrsIter<D, P>
where
    D: FieldDescriptorsOwned + ?Sized,
    P: ConstSliceItemPtr + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D, P> Clone for ErasedSoaPtrsIter<D, P>
where
    D: Clone,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            ref inner,
            buffer,
            offset,
        } = *self;

        let inner = inner.clone();
        unsafe { Self::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, P> Iterator for ErasedSoaPtrsIter<D, P>
where
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedPtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;

        let offset = inner.next()?;
        let offset = unsafe { offset.unwrap_unchecked() };
        let item = unsafe { self.field_ptr_from_buffer_offset(offset) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }
}

impl<D, P> ExactSizeIterator for ErasedSoaPtrsIter<D, P>
where
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }
}

impl<D, P> FusedIterator for ErasedSoaPtrsIter<D, P>
where
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaPtrsIter<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { inner, .. } = self;
        inner.as_inner().field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedSoaPtrsIter<D, P>
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
