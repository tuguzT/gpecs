use core::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr, slice,
};

use crate::{
    erased::{
        ErasedSoaMutPtrs, ErasedSoaRefs,
        assert::assert_eq_descriptors,
        error::{ErasedSoaIntoValueError, ErasedSoaPtrsError, check_offset},
    },
    error::{
        InsufficientAlignError, check_layout, check_len, check_ptr_align, check_sufficient_align,
        check_sufficient_len,
    },
    field::ErasedFieldPtr,
    soa::{
        field::{FieldDescriptor, buffer_offsets},
        traits::{AllocSoa, AllocSoaContext, Ptrs, RawSoaContext},
    },
    storage::AddressableUnit,
};

pub struct ErasedSoaPtrs<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    ptr: *const A,
    capacity: usize,
    offset: usize,
    descriptors: D,
}

impl<D, A> ErasedSoaPtrs<D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        ptr: *const A,
        capacity: usize,
        offset: usize,
    ) -> Self {
        Self {
            ptr,
            capacity,
            offset,
            descriptors,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *const A, usize, usize) {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
        } = self;
        (descriptors, ptr, capacity, offset)
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedSoaMutPtrs<D, A> {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
        } = self;

        let ptr = ptr.cast_mut();
        unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSoaRefs<'a, D, A> {
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

impl<D, A> ErasedSoaPtrs<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn new(
        descriptors: D,
        buffer: *const [A],
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

        let ptr = buffer.cast();
        let me = unsafe { Self::new_unchecked(descriptors, ptr, capacity, offset) };
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

        let ptr = ptr::without_provenance(addr);
        let capacity = match packed_size {
            0 => usize::MAX,
            _ => 0,
        };

        let me = unsafe { Self::new_unchecked(descriptors, ptr, capacity, 0) };
        Ok(me)
    }
}

impl<D> ErasedSoaPtrs<D, u8>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<Ptrs<'_, T>, ErasedSoaIntoValueError<Self>>
    where
        T: AllocSoa + ?Sized,
    {
        let Self {
            ref descriptors,
            ptr,
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

        let ptrs = unsafe { context.ptrs_from_buffer(ptr, capacity) };
        let ptrs = unsafe { context.ptrs_add(ptrs, offset) };
        Ok(ptrs)
    }
}

impl<D, A> ErasedSoaPtrs<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    #[inline]
    pub fn as_ptr(&self) -> *const A {
        let Self { ptr, .. } = *self;
        ptr
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

impl<D, A> ErasedSoaPtrs<D, A>
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
            ptr,
            capacity,
            offset,
        } = *self;

        assert_eq!(ptr, origin.ptr);
        assert_eq!(capacity, origin.capacity);
        assert_eq_descriptors(descriptors.as_ref(), origin.field_descriptors());

        unsafe { (offset - origin.offset).try_into().unwrap_unchecked() }
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaPtrsIter<slice::Iter<'_, FieldDescriptor>, A> {
        let Self {
            ref descriptors,
            ptr,
            capacity,
            offset,
        } = *self;

        let descriptors = descriptors.as_ref().iter();
        unsafe { ErasedSoaPtrsIter::new_unchecked(descriptors, ptr, capacity, offset) }
    }
}

impl<D, A> Debug for ErasedSoaPtrs<D, A>
where
    A: AddressableUnit,
    D: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            ptr,
            capacity,
            offset,
            descriptors,
        } = self;

        f.debug_struct("ErasedSoaPtrs")
            .field("ptr", ptr)
            .field("capacity", capacity)
            .field("offset", offset)
            .field("descriptors", &descriptors)
            .finish()
    }
}

impl<D, A> Clone for ErasedSoaPtrs<D, A>
where
    A: AddressableUnit,
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            ptr,
            capacity,
            offset,
            ref descriptors,
        } = *self;

        let descriptors = descriptors.clone();
        unsafe { Self::new_unchecked(descriptors, ptr, capacity, offset) }
    }
}

impl<D, A> Copy for ErasedSoaPtrs<D, A>
where
    A: AddressableUnit,
    D: Copy,
{
}

impl<'a, D, A> IntoIterator for &'a ErasedSoaPtrs<D, A>
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

impl<D, A> IntoIterator for ErasedSoaPtrs<D, A>
where
    A: AddressableUnit,
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldPtr<A>;
    type IntoIter = ErasedSoaPtrsIter<D::IntoIter, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
        } = self;

        let descriptors = descriptors.into_iter();
        unsafe { ErasedSoaPtrsIter::new_unchecked(descriptors, ptr, capacity, offset) }
    }
}

pub struct ErasedSoaPtrsIter<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    ptr: *const A,
    capacity: usize,
    offset: usize,
    descriptors: D,
}

impl<D, A> ErasedSoaPtrsIter<D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub(super) unsafe fn new_unchecked(
        descriptors: D,
        ptr: *const A,
        capacity: usize,
        offset: usize,
    ) -> Self {
        Self {
            ptr,
            capacity,
            offset,
            descriptors,
        }
    }
}

impl<D, A> ErasedSoaPtrsIter<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
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

impl<D, A> ErasedSoaPtrsIter<D, A>
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
    pub(super) fn debug_entries(&self) -> ErasedSoaPtrsIter<slice::Iter<'_, FieldDescriptor>, A> {
        let Self {
            ref descriptors,
            ptr,
            capacity,
            offset,
        } = *self;

        let descriptors = descriptors.as_ref().iter();
        unsafe { ErasedSoaPtrsIter::new_unchecked(descriptors, ptr, capacity, offset) }
    }
}

impl<D, A> Debug for ErasedSoaPtrsIter<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.debug_entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D, A> Clone for ErasedSoaPtrsIter<D, A>
where
    A: AddressableUnit,
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            ptr,
            capacity,
            offset,
            ref descriptors,
        } = *self;

        let descriptors = descriptors.clone();
        unsafe { Self::new_unchecked(descriptors, ptr, capacity, offset) }
    }
}

impl<D, A> Iterator for ErasedSoaPtrsIter<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldPtr<A>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut descriptors,
            ref mut ptr,
            capacity,
            offset,
        } = *self;

        let &desc = descriptors.next()?.as_ref();
        let field_ptr = {
            let len = desc.layout().size().div_ceil(size_of::<A>());
            let buffer = ptr::slice_from_raw_parts(*ptr, len);
            unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
        };

        let item = unsafe { field_ptr.add(offset) };
        *ptr = unsafe { field_ptr.add(capacity) }.as_ptr();

        if let [desc, ..] = descriptors.as_ref() {
            *ptr = unsafe { ptr.add(ptr.align_offset(desc.layout().align())) };
        }
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { descriptors, .. } = self;
        descriptors.size_hint()
    }
}

impl<D, A> ExactSizeIterator for ErasedSoaPtrsIter<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ExactSizeIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { descriptors, .. } = self;
        descriptors.len()
    }
}

impl<D, A> FusedIterator for ErasedSoaPtrsIter<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
