use core::{
    alloc::LayoutError,
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    slice,
};

use crate::{
    erased::{ErasedSoaPtrs, ErasedSoaPtrsIter, error::ErasedSoaIntoValueError},
    field::ErasedFieldRef,
    soa::{field::FieldDescriptor, traits::Soa},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSoaRefs<'a, D>
where
    D: ?Sized,
{
    phantom: PhantomData<&'a [u8]>,
    inner: ErasedSoaPtrs<D>,
}

impl<D> ErasedSoaRefs<'_, D> {
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        ptr: *const u8,
        capacity: usize,
        offset: usize,
    ) -> Self {
        let ptrs = unsafe { ErasedSoaPtrs::new_unchecked(descriptors, ptr, capacity, offset) };
        unsafe { Self::from_ptrs(ptrs) }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedSoaPtrs<D>) -> Self {
        Self {
            inner: ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *const u8, usize, usize) {
        let Self { inner, .. } = self;
        inner.into_parts()
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedSoaPtrs<D> {
        let Self { inner, .. } = self;
        inner
    }
}

impl<'a, D> ErasedSoaRefs<'a, D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    #[track_caller]
    pub fn new(
        descriptors: D,
        buffer: &'a [u8],
        capacity: usize,
        offset: usize,
    ) -> Result<Self, LayoutError> {
        let ptrs = ErasedSoaPtrs::new(descriptors, buffer, capacity, offset)?;
        let me = unsafe { Self::from_ptrs(ptrs) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<T::Refs<'_, 'a>, ErasedSoaIntoValueError<Self>>
    where
        T: Soa,
    {
        let Self { inner, .. } = self;
        let result = unsafe { inner.try_into::<T>(context) };
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let ptrs = result.map_err(|err| err.map_value(into_self))?;
        let refs = unsafe { T::ptrs_to_refs(context, ptrs) };
        Ok(refs)
    }
}

impl<D> ErasedSoaRefs<'_, D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { inner, .. } = self;
        inner.as_ptr()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { inner, .. } = self;
        inner.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { inner, .. } = self;
        inner.offset()
    }
}

impl<D> ErasedSoaRefs<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { inner, .. } = self;
        inner.field_descriptors()
    }

    #[inline]
    pub fn iter(&self) -> ErasedSoaRefsIter<'_, slice::Iter<'_, FieldDescriptor>> {
        let Self { inner, .. } = self;
        ErasedSoaRefsIter {
            inner: inner.iter(),
            phantom: PhantomData,
        }
    }
}

impl<'a, D> IntoIterator for &'a ErasedSoaRefs<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = ErasedFieldRef<'a>;
    type IntoIter = ErasedSoaRefsIter<'a, slice::Iter<'a, FieldDescriptor>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for ErasedSoaRefs<'a, D>
where
    D: IntoIterator,
    D::Item: AsRef<FieldDescriptor>,
    D::IntoIter: AsRef<[FieldDescriptor]>,
{
    type Item = ErasedFieldRef<'a>;
    type IntoIter = ErasedSoaRefsIter<'a, D::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner, phantom } = self;
        ErasedSoaRefsIter {
            inner: inner.into_iter(),
            phantom,
        }
    }
}

#[derive(Clone)]
pub struct ErasedSoaRefsIter<'a, D>
where
    D: ?Sized,
{
    phantom: PhantomData<&'a [u8]>,
    inner: ErasedSoaPtrsIter<D>,
}

impl<D> ErasedSoaRefsIter<'_, D>
where
    D: ?Sized,
{
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { inner, .. } = self;
        inner.as_ptr()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { inner, .. } = self;
        inner.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { inner, .. } = self;
        inner.offset()
    }
}

impl<D> ErasedSoaRefsIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { inner, .. } = self;
        inner.field_descriptors()
    }

    #[inline]
    pub fn field_descriptors_iter(
        &self,
    ) -> ErasedSoaRefsIter<'_, slice::Iter<'_, FieldDescriptor>> {
        let Self { inner, .. } = self;
        ErasedSoaRefsIter {
            inner: inner.field_descriptors_iter(),
            phantom: PhantomData,
        }
    }
}

impl<D> Debug for ErasedSoaRefsIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.field_descriptors_iter();
        f.debug_list().entries(entries).finish()
    }
}

impl<'a, D> Iterator for ErasedSoaRefsIter<'a, D>
where
    D: AsRef<[FieldDescriptor]> + Iterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    type Item = ErasedFieldRef<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;

        let item = unsafe { inner.next()?.deref() };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }
}

impl<D> ExactSizeIterator for ErasedSoaRefsIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + ExactSizeIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }
}

impl<D> FusedIterator for ErasedSoaRefsIter<'_, D>
where
    D: AsRef<[FieldDescriptor]> + FusedIterator + ?Sized,
    D::Item: AsRef<FieldDescriptor>,
{
}
