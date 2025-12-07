use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr, slice,
};

use crate::{
    erased::{
        ErasedSoaPtrs, ErasedSoaPtrsIter, ErasedSoaRefs, ErasedSoaRefsMut,
        assert::debug_assert_descriptors,
        error::{ErasedSoaIntoValueError, ErasedSoaPtrsError, check_offset, check_sufficient_len},
    },
    error::{check_layout, check_len},
    field::ErasedFieldMutPtr,
    soa::{
        field::{FieldDescriptor, buffer_layout},
        traits::{MutPtrs, RawSoa, RawSoaContext},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaMutPtrs<D>
where
    D: ?Sized,
{
    ptr: *mut u8,
    capacity: usize,
    offset: usize,
    descriptors: D,
}

impl<D> ErasedSoaMutPtrs<D> {
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        ptr: *mut u8,
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
    pub fn into_parts(self) -> (D, *mut u8, usize, usize) {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
        } = self;
        (descriptors, ptr, capacity, offset)
    }

    #[inline]
    pub fn cast_const(self) -> ErasedSoaPtrs<D> {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
        } = self;

        let ptr = ptr.cast_const();
        unsafe { ErasedSoaPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
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

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSoaRefs<'a, D> {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
        } = self;
        unsafe { ErasedSoaRefs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedSoaRefsMut<'a, D> {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
        } = self;
        unsafe { ErasedSoaRefsMut::new_unchecked(descriptors, ptr, capacity, offset) }
    }
}

impl<D> ErasedSoaMutPtrs<D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn new(
        descriptors: D,
        buffer: *mut [u8],
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
    ) -> Result<MutPtrs<'_, T>, ErasedSoaIntoValueError<Self>>
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
            let ptrs = context.ptrs_from_buffer_mut(ptr, capacity);
            let ptrs = context.ptrs_add_mut(ptrs, offset);
            Ok(ptrs)
        }
    }
}

impl<D> ErasedSoaMutPtrs<D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
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

impl<D> ErasedSoaMutPtrs<D>
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
    pub unsafe fn offset_from<A>(&self, origin: &ErasedSoaPtrs<A>) -> isize
    where
        A: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self {
            ref descriptors,
            ptr,
            capacity,
            offset,
        } = *self;

        assert_eq!(ptr.cast_const(), origin.as_ptr());
        assert_eq!(capacity, origin.capacity());
        debug_assert_descriptors(descriptors.as_ref(), origin.field_descriptors());

        unsafe { (offset - origin.offset()).try_into().unwrap_unchecked() }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap<A>(&self, with: &ErasedSoaMutPtrs<A>)
    where
        A: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = self;
        debug_assert_descriptors(descriptors.as_ref(), with.field_descriptors());

        itertools::zip_eq(self, with).for_each(|(this, with)| unsafe { this.swap(with) });
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from<A>(&self, from: &ErasedSoaPtrs<A>, count: usize)
    where
        A: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = self;
        debug_assert_descriptors(descriptors.as_ref(), from.field_descriptors());

        itertools::zip_eq(self, from)
            .for_each(|(this, from)| unsafe { this.copy_from(from, count) });
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_rev<A>(&self, from: &ErasedSoaPtrs<A>, count: usize)
    where
        A: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = self;
        debug_assert_descriptors(descriptors.as_ref(), from.field_descriptors());

        #[inline]
        #[track_caller]
        #[expect(clippy::items_after_statements)]
        fn rec(
            iter: &mut itertools::ZipEq<
                ErasedSoaMutPtrsIter<slice::Iter<'_, FieldDescriptor>>,
                ErasedSoaPtrsIter<slice::Iter<'_, FieldDescriptor>>,
            >,
            count: usize,
        ) {
            let Some((to, from)) = iter.next() else {
                return;
            };
            rec(iter, count);
            unsafe { to.copy_from(from, count) }
        }

        let mut iter = itertools::zip_eq(self, from);
        rec(&mut iter, count);
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping<A>(&self, from: &ErasedSoaPtrs<A>, count: usize)
    where
        A: AsRef<[FieldDescriptor]> + ?Sized,
    {
        let Self { descriptors, .. } = self;
        debug_assert_descriptors(descriptors.as_ref(), from.field_descriptors());

        itertools::zip_eq(self, from)
            .for_each(|(this, from)| unsafe { this.copy_from_nonoverlapping(from, count) });
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaMutPtrsIter<slice::Iter<'_, FieldDescriptor>> {
        let Self {
            ref descriptors,
            ptr,
            capacity,
            offset,
        } = *self;

        ErasedSoaMutPtrsIter {
            descriptors: descriptors.as_ref().iter(),
            ptr,
            capacity,
            offset,
        }
    }
}

impl<'a, D> IntoIterator for &'a ErasedSoaMutPtrs<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldMutPtr;
    type IntoIter = ErasedSoaMutPtrsIter<slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D> IntoIterator for ErasedSoaMutPtrs<D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldMutPtr;
    type IntoIter = ErasedSoaMutPtrsIter<D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            ptr,
            capacity,
            offset,
        } = self;

        ErasedSoaMutPtrsIter {
            descriptors: descriptors.into_iter(),
            ptr,
            capacity,
            offset,
        }
    }
}

#[derive(Clone)]
pub struct ErasedSoaMutPtrsIter<D>
where
    D: ?Sized,
{
    ptr: *mut u8,
    capacity: usize,
    offset: usize,
    descriptors: D,
}

impl<D> ErasedSoaMutPtrsIter<D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
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

impl<D> ErasedSoaMutPtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }

    #[inline]
    pub fn field_descriptors_iter(&self) -> ErasedSoaMutPtrsIter<slice::Iter<'_, FieldDescriptor>> {
        let Self {
            ref descriptors,
            ptr,
            capacity,
            offset,
        } = *self;

        ErasedSoaMutPtrsIter {
            descriptors: descriptors.as_ref().iter(),
            ptr,
            capacity,
            offset,
        }
    }
}

impl<D> Debug for ErasedSoaMutPtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            ref descriptors,
            ptr,
            capacity,
            offset,
        } = *self;

        let entries = ErasedSoaMutPtrsIter {
            descriptors: descriptors.as_ref().iter(),
            ptr,
            capacity,
            offset,
        };
        f.debug_list().entries(entries).finish()
    }
}

impl<D> Iterator for ErasedSoaMutPtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldMutPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut descriptors,
            ref mut ptr,
            capacity,
            offset,
        } = *self;

        let &desc = descriptors.next()?.as_ref();
        let buffer = ptr::slice_from_raw_parts_mut(*ptr, desc.layout().size());
        let field_ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) };

        let item = unsafe { field_ptr.add(offset) };
        *ptr = unsafe { field_ptr.add(capacity) }.as_mut_ptr();

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

impl<D> ExactSizeIterator for ErasedSoaMutPtrsIter<D>
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

impl<D> FusedIterator for ErasedSoaMutPtrsIter<D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
