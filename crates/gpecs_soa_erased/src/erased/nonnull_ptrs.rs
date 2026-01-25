use core::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr::{self, NonNull},
    slice,
};

use crate::{
    erased::{
        assert::debug_assert_eq_descriptors,
        error::{ErasedSoaIntoValueError, ErasedSoaPtrsError, check_offset},
    },
    error::{
        InsufficientAlignError, check_layout, check_len, check_ptr_align, check_sufficient_align,
        check_sufficient_len,
    },
    field::ErasedFieldNonNullPtr,
    soa::{
        field::{FieldDescriptor, buffer_offsets},
        traits::{AllocSoa, AllocSoaContext, NonNullPtrs, RawSoaContext},
    },
    storage::AddressableUnit,
};

pub struct ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    buffer: NonNull<[A]>,
    capacity: usize,
    offset: usize,
    descriptors: D,
}

impl<D, A> ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: NonNull<[A]>,
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
    pub fn into_parts(self) -> (D, NonNull<[A]>, usize, usize) {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;
        (descriptors, buffer, capacity, offset)
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { offset, .. } = self;

        let offset = unsafe { offset.unchecked_add(count) };
        Self { offset, ..self }
    }
}

impl<D, A> ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn new(
        descriptors: D,
        buffer: NonNull<[A]>,
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
        check_ptr_align(buffer.as_ptr().cast(), layout)?;
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
        let buffer = unsafe { NonNull::new_unchecked(buffer) };
        let capacity = match packed_size {
            0 => usize::MAX,
            _ => 0,
        };

        let me = unsafe { Self::new_unchecked(descriptors, buffer, capacity, 0) };
        Ok(me)
    }
}

impl<D> ErasedSoaNonNullPtrs<D, u8>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<NonNullPtrs<'_, T>, ErasedSoaIntoValueError<Self>>
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

        unsafe {
            let ptrs = context.ptrs_from_buffer_mut(buffer.as_ptr().cast(), capacity);
            let ptrs = context.ptrs_add_mut(ptrs, offset);
            let ptrs = context.ptrs_to_nonnull(ptrs);
            Ok(ptrs)
        }
    }
}

impl<D, A> ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    #[inline]
    pub fn as_buffer(&self) -> NonNull<[A]> {
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

impl<D, A> ErasedSoaNonNullPtrs<D, A>
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
    pub unsafe fn offset_from<E>(&self, origin: &ErasedSoaNonNullPtrs<E, A>) -> isize
    where
        E: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
        } = *self;

        assert_eq!(buffer, origin.as_buffer());
        assert_eq!(capacity, origin.capacity());
        debug_assert_eq_descriptors(descriptors.as_ref(), origin.field_descriptors());

        unsafe { (offset - origin.offset).try_into().unwrap_unchecked() }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap<E>(&mut self, with: &mut ErasedSoaNonNullPtrs<E, A>)
    where
        E: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = &self;
        debug_assert_eq_descriptors(descriptors.as_ref(), with.field_descriptors());

        itertools::zip_eq(self.iter(), with.iter()).for_each(|(me, with)| unsafe { me.swap(with) });
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from<E>(&mut self, from: &ErasedSoaNonNullPtrs<E, A>, count: usize)
    where
        E: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = &self;
        debug_assert_eq_descriptors(descriptors.as_ref(), from.field_descriptors());

        itertools::zip_eq(self.iter(), from)
            .for_each(|(me, from)| unsafe { me.copy_from(from, count) });
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_rev<E>(&mut self, from: &ErasedSoaNonNullPtrs<E, A>, count: usize)
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
            I: IntoIterator<Item = (ErasedFieldNonNullPtr<A>, ErasedFieldNonNullPtr<A>)>,
        {
            let mut iter = iter.into_iter();
            let Some((to, from)) = iter.next() else {
                return;
            };

            rec(iter, count);
            unsafe { to.copy_from(from, count) }
        }

        rec(itertools::zip_eq(self.iter(), from), count);
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping<E>(
        &mut self,
        from: &ErasedSoaNonNullPtrs<E, A>,
        count: usize,
    ) where
        E: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = &self;
        debug_assert_eq_descriptors(descriptors.as_ref(), from.field_descriptors());

        itertools::zip_eq(self.iter(), from)
            .for_each(|(me, from)| unsafe { me.copy_from_nonoverlapping(from, count) });
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaNonNullPtrsIter<slice::Iter<'_, FieldDescriptor>, A> {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
        } = *self;

        let descriptors = descriptors.as_ref().iter();
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(descriptors, buffer, capacity, offset) }
    }
}

impl<D, A> Debug for ErasedSoaNonNullPtrs<D, A>
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

        f.debug_struct("ErasedSoaNonNullPtrs")
            .field("buffer", buffer)
            .field("capacity", capacity)
            .field("offset", offset)
            .field("descriptors", &descriptors)
            .finish()
    }
}

impl<D, A> Clone for ErasedSoaNonNullPtrs<D, A>
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

impl<D, A> Copy for ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: Copy,
{
}

impl<'a, D, A> IntoIterator for &'a ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldNonNullPtr<A>;
    type IntoIter = ErasedSoaNonNullPtrsIter<slice::Iter<'a, FieldDescriptor>, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, A> IntoIterator for ErasedSoaNonNullPtrs<D, A>
where
    A: AddressableUnit,
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldNonNullPtr<A>;
    type IntoIter = ErasedSoaNonNullPtrsIter<D::IntoIter, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
        } = self;

        let descriptors = descriptors.into_iter();
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(descriptors, buffer, capacity, offset) }
    }
}

pub struct ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    ptr: NonNull<A>,
    capacity: usize,
    offset: usize,
    descriptors: D,
}

impl<D, A> ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub(super) unsafe fn new_unchecked(
        descriptors: D,
        buffer: NonNull<[A]>,
        capacity: usize,
        offset: usize,
    ) -> Self {
        Self {
            ptr: buffer.cast(),
            capacity,
            offset,
            descriptors,
        }
    }
}

impl<D, A> ErasedSoaNonNullPtrsIter<D, A>
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

impl<D, A> ErasedSoaNonNullPtrsIter<D, A>
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
    pub(super) fn debug_entries(
        &self,
    ) -> ErasedSoaNonNullPtrsIter<slice::Iter<'_, FieldDescriptor>, A> {
        let Self {
            ref descriptors,
            ptr,
            capacity,
            offset,
        } = *self;

        let descriptors = descriptors.as_ref().iter();
        ErasedSoaNonNullPtrsIter {
            ptr,
            capacity,
            offset,
            descriptors,
        }
    }
}

impl<D, A> Debug for ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.debug_entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D, A> Clone for ErasedSoaNonNullPtrsIter<D, A>
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
        Self {
            ptr,
            capacity,
            offset,
            descriptors,
        }
    }
}

impl<D, A> Iterator for ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldNonNullPtr<A>;

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
            let buffer = ptr::slice_from_raw_parts_mut(ptr.as_ptr(), len);
            let buffer = unsafe { NonNull::new_unchecked(buffer) };
            unsafe { ErasedFieldNonNullPtr::new_unchecked(desc, buffer) }
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

impl<D, A> ExactSizeIterator for ErasedSoaNonNullPtrsIter<D, A>
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

impl<D, A> FusedIterator for ErasedSoaNonNullPtrsIter<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
