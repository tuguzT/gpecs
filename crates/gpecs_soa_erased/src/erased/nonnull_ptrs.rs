use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr::{self, NonNull},
    slice,
};

use crate::{
    erased::{
        assert::debug_assert_descriptors,
        error::{ErasedSoaIntoValueError, ErasedSoaPtrsError, check_offset, check_sufficient_len},
    },
    error::{check_layout, check_len},
    field::ErasedFieldNonNullPtr,
    soa::{
        field::{FieldDescriptor, buffer_layout},
        traits::{NonNullPtrs, RawSoa, RawSoaContext},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaNonNullPtrs<D>
where
    D: ?Sized,
{
    ptr: NonNull<u8>,
    capacity: usize,
    offset: usize,
    descriptors: D,
}

impl<D> ErasedSoaNonNullPtrs<D> {
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        ptr: NonNull<u8>,
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
    pub fn into_parts(self) -> (D, NonNull<u8>, usize, usize) {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
        } = self;
        (descriptors, ptr, capacity, offset)
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { offset, .. } = self;
        Self {
            offset: unsafe { offset.unchecked_add(count) },
            ..self
        }
    }
}

impl<D> ErasedSoaNonNullPtrs<D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn new(
        descriptors: D,
        buffer: NonNull<[u8]>,
        capacity: usize,
        offset: usize,
    ) -> Result<Self, ErasedSoaPtrsError> {
        let layout = buffer_layout(descriptors.as_ref(), capacity)?;
        check_sufficient_len(buffer.len(), layout.size())?;
        check_offset(offset, capacity)?;

        let ptr = buffer.cast();
        let me = unsafe { Self::new_unchecked(descriptors, ptr, capacity, offset) };
        Ok(me)
    }

    #[inline]
    pub fn dangling(descriptors: D) -> Self {
        let addr = descriptors
            .as_ref()
            .iter()
            .map(|desc| desc.layout().align())
            .max()
            .unwrap_or(1);
        let ptr = ptr::without_provenance_mut(addr);
        let ptr = unsafe { NonNull::new_unchecked(ptr) };

        let packed_size = descriptors
            .as_ref()
            .iter()
            .map(|desc| desc.layout().size())
            .sum::<usize>();
        let capacity = match packed_size {
            0 => usize::MAX,
            _ => 0,
        };

        Self {
            descriptors,
            ptr,
            capacity,
            offset: 0,
        }
    }

    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<NonNullPtrs<'_, T>, ErasedSoaIntoValueError<Self>>
    where
        T: RawSoa,
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

        unsafe {
            let ptrs = context.ptrs_from_buffer_mut(ptr.as_ptr(), capacity);
            let ptrs = context.ptrs_add_mut(ptrs, offset);
            let ptrs = context.ptrs_to_nonnull(ptrs);
            Ok(ptrs)
        }
    }
}

impl<D> ErasedSoaNonNullPtrs<D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_ptr(&self) -> NonNull<u8> {
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

impl<D> ErasedSoaNonNullPtrs<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }

    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<A>(&self, origin: &ErasedSoaNonNullPtrs<A>) -> isize
    where
        A: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self {
            ref descriptors,
            ptr,
            capacity,
            offset,
        } = *self;

        assert_eq!(ptr, origin.as_ptr());
        assert_eq!(capacity, origin.capacity());
        debug_assert_descriptors(descriptors.as_ref(), origin.field_descriptors());

        unsafe { (offset - origin.offset()).try_into().unwrap_unchecked() }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap<A>(&mut self, with: &mut ErasedSoaNonNullPtrs<A>)
    where
        A: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = &self;
        debug_assert_descriptors(descriptors.as_ref(), with.field_descriptors());

        itertools::zip_eq(self.iter(), with.iter()).for_each(|(me, with)| unsafe { me.swap(with) });
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from<A>(&mut self, from: &ErasedSoaNonNullPtrs<A>, count: usize)
    where
        A: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = &self;
        debug_assert_descriptors(descriptors.as_ref(), from.field_descriptors());

        itertools::zip_eq(self.iter(), from)
            .for_each(|(me, from)| unsafe { me.copy_from(from, count) });
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_rev<A>(&mut self, from: &ErasedSoaNonNullPtrs<A>, count: usize)
    where
        A: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = &self;
        debug_assert_descriptors(descriptors.as_ref(), from.field_descriptors());

        #[inline]
        #[track_caller]
        #[expect(clippy::items_after_statements)]
        fn rec(
            iter: &mut itertools::ZipEq<
                ErasedSoaNonNullPtrsIter<slice::Iter<'_, FieldDescriptor>>,
                ErasedSoaNonNullPtrsIter<slice::Iter<'_, FieldDescriptor>>,
            >,
            count: usize,
        ) {
            let Some((to, from)) = iter.next() else {
                return;
            };
            rec(iter, count);
            unsafe { to.copy_from(from, count) }
        }

        let mut iter = itertools::zip_eq(self.iter(), from);
        rec(&mut iter, count);
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping<A>(
        &mut self,
        from: &ErasedSoaNonNullPtrs<A>,
        count: usize,
    ) where
        A: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = &self;
        debug_assert_descriptors(descriptors.as_ref(), from.field_descriptors());

        itertools::zip_eq(self.iter(), from)
            .for_each(|(me, from)| unsafe { me.copy_from_nonoverlapping(from, count) });
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaNonNullPtrsIter<slice::Iter<'_, FieldDescriptor>> {
        let Self {
            ref descriptors,
            ptr,
            capacity,
            offset,
        } = *self;

        let descriptors = descriptors.as_ref().iter();
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(ptr, capacity, offset, descriptors) }
    }
}

impl<'a, D> IntoIterator for &'a ErasedSoaNonNullPtrs<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldNonNullPtr;
    type IntoIter = ErasedSoaNonNullPtrsIter<slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> IntoIterator for ErasedSoaNonNullPtrs<D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldNonNullPtr;
    type IntoIter = ErasedSoaNonNullPtrsIter<D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
        } = self;

        let descriptors = descriptors.into_iter();
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(ptr, capacity, offset, descriptors) }
    }
}

#[derive(Clone)]
pub struct ErasedSoaNonNullPtrsIter<D>
where
    D: ?Sized,
{
    ptr: NonNull<u8>,
    capacity: usize,
    offset: usize,
    descriptors: D,
}

impl<D> ErasedSoaNonNullPtrsIter<D> {
    #[inline]
    pub(super) unsafe fn new_unchecked(
        ptr: NonNull<u8>,
        capacity: usize,
        offset: usize,
        descriptors: D,
    ) -> Self {
        Self {
            ptr,
            capacity,
            offset,
            descriptors,
        }
    }
}

impl<D> ErasedSoaNonNullPtrsIter<D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_ptr(&self) -> NonNull<u8> {
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

impl<D> ErasedSoaNonNullPtrsIter<D>
where
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
    ) -> ErasedSoaNonNullPtrsIter<slice::Iter<'_, FieldDescriptor>> {
        let Self {
            ref descriptors,
            ptr,
            capacity,
            offset,
        } = *self;

        let descriptors = descriptors.as_ref().iter();
        unsafe { ErasedSoaNonNullPtrsIter::new_unchecked(ptr, capacity, offset, descriptors) }
    }
}

impl<D> Debug for ErasedSoaNonNullPtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.debug_entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D> Iterator for ErasedSoaNonNullPtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldNonNullPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut descriptors,
            ref mut ptr,
            capacity,
            offset,
        } = *self;

        let &desc = descriptors.next()?.as_ref();
        let buffer = ptr::slice_from_raw_parts_mut(ptr.as_ptr(), desc.layout().size());
        let buffer = unsafe { NonNull::new_unchecked(buffer) };
        let field_ptr = unsafe { ErasedFieldNonNullPtr::new_unchecked(desc, buffer) };

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

impl<D> ExactSizeIterator for ErasedSoaNonNullPtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + ExactSizeIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { descriptors, .. } = self;
        descriptors.len()
    }
}

impl<D> FusedIterator for ErasedSoaNonNullPtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
