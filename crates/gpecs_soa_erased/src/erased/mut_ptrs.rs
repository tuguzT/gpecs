use core::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr, slice,
};

use crate::{
    erased::{
        ErasedSoaPtrs, ErasedSoaPtrsIter, ErasedSoaRefs, ErasedSoaRefsMut,
        assert::debug_assert_eq_descriptors,
        error::{ErasedSoaIntoValueError, ErasedSoaPtrsError, check_offset},
    },
    error::{
        InsufficientAlignError, check_layout, check_len, check_ptr_align, check_sufficient_align,
        check_sufficient_len,
    },
    field::{ErasedFieldMutPtr, ErasedFieldPtr},
    soa::{
        field::{BufferOffset, BufferOffsets, FieldDescriptor, buffer_offsets},
        traits::{AllocSoa, AllocSoaContext, MutPtrs, RawSoaContext, WithFieldDescriptors},
    },
    storage::AddressableUnit,
};

pub struct ErasedSoaMutPtrs<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    buffer: *mut [A],
    capacity: usize,
    offset: usize,
    descriptors: D,
}

impl<D, A> ErasedSoaMutPtrs<D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: *mut [A],
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
    pub fn into_parts(self) -> (D, *mut [A], usize, usize) {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;
        (descriptors, buffer, capacity, offset)
    }

    #[inline]
    pub fn cast_const(self) -> ErasedSoaPtrs<D, A> {
        let Self {
            descriptors,
            buffer: ptr,
            capacity,
            offset,
        } = self;

        let ptr = ptr.cast_const();
        unsafe { ErasedSoaPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSoaRefs<'a, D, A> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedSoaRefsMut<'a, D, A> {
        unsafe { ErasedSoaRefsMut::from_mut_ptrs(self) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { offset, .. } = self;

        let offset = unsafe { offset.unchecked_add(count) };
        Self { offset, ..self }
    }
}

impl<D, A> ErasedSoaMutPtrs<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        descriptors: D,
        buffer: *mut [A],
        capacity: usize,
        offset: usize,
    ) -> Result<Self, ErasedSoaPtrsError> {
        let mut offsets = buffer_offsets(descriptors.as_ref(), capacity);
        offsets.by_ref().try_for_each(|offset| {
            let desc = offset?.field_descriptor;
            check_sufficient_align(desc.layout(), Layout::new::<A>())
                .map_err(ErasedSoaPtrsError::from)
        })?;

        let layout = offsets.layout();
        check_sufficient_len(buffer.len() * size_of::<A>(), layout.size())?;
        check_ptr_align(buffer.cast(), layout)?;
        check_offset(offset, capacity)?;

        let me = unsafe { Self::new_unchecked(descriptors, buffer, capacity, offset) };
        Ok(me)
    }

    #[inline]
    pub fn dangling(descriptors: D) -> Result<Self, InsufficientAlignError> {
        let mut packed_size = 0;
        let addr = descriptors.as_ref().iter().try_fold(1, |max_align, desc| {
            let layout = desc.layout();
            check_sufficient_align(layout, Layout::new::<A>())?;

            packed_size += layout.size().div_ceil(size_of::<A>());
            Ok(usize::max(max_align, layout.align()))
        })?;

        let data = ptr::without_provenance_mut(addr);
        let buffer = ptr::slice_from_raw_parts_mut(data, 0);
        let capacity = match packed_size {
            0 => usize::MAX,
            _ => 0,
        };

        let me = unsafe { Self::new_unchecked(descriptors, buffer, capacity, 0) };
        Ok(me)
    }
}

impl<D> ErasedSoaMutPtrs<D, u8>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<MutPtrs<'_, T>, ErasedSoaIntoValueError<Self>>
    where
        T: AllocSoa + ?Sized,
    {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
        } = self;
        let descriptors = descriptors.as_ref();

        let result = context
            .field_descriptors()
            .into_iter()
            .zip(&self)
            .try_fold(0, |len, (desc, slice)| {
                check_layout(slice.descriptor().layout(), desc.as_ref().layout())?;
                Ok(len + 1)
            })
            .and_then(|len| {
                check_len(len, descriptors.len())?;
                Ok(())
            });
        if let Err(error) = result {
            return Err(ErasedSoaIntoValueError::new(self, error));
        }

        let ptrs = unsafe { context.ptrs_from_buffer_mut(buffer.cast(), capacity) };
        let ptrs = unsafe { context.ptrs_add_mut(ptrs, offset) };
        Ok(ptrs)
    }
}

impl<D, A> ErasedSoaMutPtrs<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [A] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> *mut [A] {
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
}

impl<D, A> ErasedSoaMutPtrs<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<E>(&self, origin: &ErasedSoaPtrs<E, A>) -> isize
    where
        E: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
        } = *self;

        assert_eq!(buffer.cast_const(), origin.as_buffer());
        assert_eq!(capacity, origin.capacity());
        debug_assert_eq_descriptors(descriptors.as_ref(), origin.field_descriptors());

        unsafe { (offset - origin.offset()).try_into().unwrap_unchecked() }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap<E>(&mut self, with: &mut ErasedSoaMutPtrs<E, A>)
    where
        E: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = &self;
        debug_assert_eq_descriptors(descriptors.as_ref(), with.field_descriptors());

        // TODO: rewrite this loop without zip
        for (this, with) in itertools::zip_eq(self, with) {
            unsafe { this.swap(with) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from<E>(&mut self, from: &ErasedSoaPtrs<E, A>, count: usize)
    where
        E: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = &self;
        debug_assert_eq_descriptors(descriptors.as_ref(), from.field_descriptors());

        for (this, from) in itertools::zip_eq(self, from) {
            unsafe { this.copy_from(from, count) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_rev<E>(&mut self, from: &ErasedSoaPtrs<E, A>, count: usize)
    where
        E: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = &self;
        debug_assert_eq_descriptors(descriptors.as_ref(), from.field_descriptors());

        #[inline]
        #[expect(clippy::items_after_statements)]
        fn rec<A, I>(iter: I, count: usize)
        where
            A: AddressableUnit,
            I: IntoIterator<Item = (ErasedFieldMutPtr<A>, ErasedFieldPtr<A>)>,
        {
            let mut iter = iter.into_iter();
            let Some((to, from)) = iter.next() else {
                return;
            };

            rec(iter, count);
            unsafe { to.copy_from(from, count) }
        }

        rec(itertools::zip_eq(self, from), count);
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping<E>(&mut self, from: &ErasedSoaPtrs<E, A>, count: usize)
    where
        E: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = &self;
        debug_assert_eq_descriptors(descriptors.as_ref(), from.field_descriptors());

        for (this, from) in itertools::zip_eq(self, from) {
            unsafe { this.copy_from_nonoverlapping(from, count) }
        }
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaPtrsIter<slice::Iter<'_, FieldDescriptor>, A> {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
        } = *self;

        let descriptors = descriptors.as_ref().iter();
        let inner = buffer_offsets(descriptors, capacity);
        unsafe { ErasedSoaPtrsIter::new_unchecked(inner, buffer, offset) }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedSoaMutPtrsIter<slice::Iter<'_, FieldDescriptor>, A> {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
        } = *self;

        let descriptors = descriptors.as_ref().iter();
        let inner = buffer_offsets(descriptors, capacity);
        unsafe { ErasedSoaMutPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, A> Debug for ErasedSoaMutPtrs<D, A>
where
    A: AddressableUnit,
    D: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            buffer,
            capacity,
            offset,
            descriptors,
        } = self;

        f.debug_struct("ErasedSoaMutPtrs")
            .field("buffer", buffer)
            .field("capacity", capacity)
            .field("offset", offset)
            .field("descriptors", &descriptors)
            .finish()
    }
}

impl<D, A> Clone for ErasedSoaMutPtrs<D, A>
where
    A: AddressableUnit,
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            buffer,
            capacity,
            offset,
            ref descriptors,
        } = *self;

        let descriptors = descriptors.clone();
        unsafe { Self::new_unchecked(descriptors, buffer, capacity, offset) }
    }
}

impl<D, A> Copy for ErasedSoaMutPtrs<D, A>
where
    A: AddressableUnit,
    D: Copy,
{
}

impl<'a, D, A> IntoIterator for &'a ErasedSoaMutPtrs<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldPtr<A>;
    type IntoIter = ErasedSoaPtrsIter<slice::Iter<'a, FieldDescriptor>, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, A> IntoIterator for &'a mut ErasedSoaMutPtrs<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldMutPtr<A>;
    type IntoIter = ErasedSoaMutPtrsIter<slice::Iter<'a, FieldDescriptor>, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<D, A> IntoIterator for ErasedSoaMutPtrs<D, A>
where
    A: AddressableUnit,
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldMutPtr<A>;
    type IntoIter = ErasedSoaMutPtrsIter<D::IntoIter, A>;

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
        unsafe { ErasedSoaMutPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

pub struct ErasedSoaMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    buffer: *mut [A],
    offset: usize,
    inner: BufferOffsets<D>,
}

impl<D, A> ErasedSoaMutPtrsIter<D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub(super) unsafe fn new_unchecked(
        inner: BufferOffsets<D>,
        buffer: *mut [A],
        offset: usize,
    ) -> Self {
        Self {
            buffer,
            offset,
            inner,
        }
    }
}

impl<D, A> ErasedSoaMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [A] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> *mut [A] {
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
}

impl<D, A> ErasedSoaMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { inner, .. } = self;
        inner.as_inner().as_ref()
    }

    #[inline]
    pub(super) fn debug_entries(
        &self,
    ) -> ErasedSoaMutPtrsIter<slice::Iter<'_, FieldDescriptor>, A> {
        let Self {
            ref inner,
            buffer,
            offset,
        } = *self;

        let layout = inner.layout();
        let capacity = inner.capacity();
        let fields = inner.as_inner().as_ref().iter();

        let inner = unsafe { BufferOffsets::from_parts(layout, capacity, fields) };
        unsafe { ErasedSoaMutPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, A> Debug for ErasedSoaMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.debug_entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D, A> Clone for ErasedSoaMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: Clone,
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

impl<D, A> Iterator for ErasedSoaMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldMutPtr<A>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut inner,
            buffer,
            offset,
        } = *self;

        let field_ptr = {
            let BufferOffset {
                field_descriptor: desc,
                offset,
                ..
            } = inner
                .next()?
                .expect("buffer layout should have been checked way earlier");

            let offset = offset.div_ceil(size_of::<A>());
            let len = desc.layout().size().div_ceil(size_of::<A>());
            let data = unsafe { buffer.cast::<A>().add(offset) };

            let buffer = ptr::slice_from_raw_parts_mut(data, len);
            unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) }
        };

        let item = unsafe { field_ptr.add(offset) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }
}

impl<D, A> ExactSizeIterator for ErasedSoaMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: ExactSizeIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }
}

impl<D, A> FusedIterator for ErasedSoaMutPtrsIter<D, A>
where
    A: AddressableUnit,
    D: FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
