use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr, slice,
};

use crate::{
    error::InsufficientAlignError,
    field::{
        ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldSlice, ErasedFieldSliceMutPtr,
        ErasedFieldSlicePtr,
        error::{ErasedFieldIntoValueError, ErasedFieldSlicePtrError},
    },
    fmt::SliceUpperHex,
    soa::field::FieldDescriptor,
    storage::AddressableUnit,
};

pub struct ErasedFieldSliceMut<'a, A>
where
    A: AddressableUnit,
{
    ptr: ErasedFieldSliceMutPtr<A>,
    phantom: PhantomData<&'a mut [A]>,
}

impl<'a, A> ErasedFieldSliceMut<'a, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub fn new(
        desc: FieldDescriptor,
        buffer: &'a mut [A],
        len: usize,
    ) -> Result<Self, ErasedFieldSlicePtrError> {
        let ptr = ErasedFieldSliceMutPtr::new(desc, buffer, len)?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &'a mut [A], len: usize) -> Self {
        let ptr = unsafe { ErasedFieldSliceMutPtr::new_unchecked(desc, buffer, len) };
        unsafe { Self::from_ptr(ptr) }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedFieldSliceMutPtr<A>) -> Self {
        let phantom = PhantomData;
        Self { ptr, phantom }
    }

    #[inline]
    pub unsafe fn try_into<T>(self) -> Result<&'a mut [T], ErasedFieldIntoValueError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_ptr(ptr) };
        let buffer = <*mut [T]>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&[T], ErasedFieldIntoValueError<&Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let buffer = <*mut [T]>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn cast_mut<T>(&mut self) -> Result<&mut [T], ErasedFieldIntoValueError<&mut Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let buffer = <*mut [T]>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { ptr, .. } = self;
        ptr.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { ptr, .. } = self;
        ptr.descriptor()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[A] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const A {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn as_field_slice_ptr(&self) -> ErasedFieldSlicePtr<A> {
        let Self { ptr, .. } = self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr<A> {
        let Self { ptr, .. } = self;
        ptr.as_field_ptr()
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> &mut [A] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_mut_buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut A {
        let Self { ptr, .. } = self;
        ptr.as_mut_ptr()
    }

    #[inline]
    pub fn as_mut_field_slice_ptr(&mut self) -> ErasedFieldSliceMutPtr<A> {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_mut_field_ptr(&mut self) -> ErasedFieldMutPtr<A> {
        let Self { ptr, .. } = self;
        ptr.as_mut_field_ptr()
    }

    #[inline]
    pub fn into_buffer(self) -> &'a mut [A] {
        let (_, buffer, _) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, &'a mut [A], usize) {
        let Self { ptr, .. } = self;
        let (desc, buffer, len) = ptr.into_parts();
        let buffer = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        (desc, buffer, len)
    }
}

impl<A> Debug for ErasedFieldSliceMut<'_, A>
where
    A: AddressableUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &SliceUpperHex(self.as_buffer());
        let len = &self.len();
        f.debug_struct("ErasedFieldSliceMut")
            .field("desc", desc)
            .field("buffer", buffer)
            .field("len", len)
            .finish()
    }
}

impl<A> AsRef<[A]> for ErasedFieldSliceMut<'_, A>
where
    A: AddressableUnit,
{
    #[inline]
    fn as_ref(&self) -> &[A] {
        self.as_buffer()
    }
}

impl<A> AsMut<[A]> for ErasedFieldSliceMut<'_, A>
where
    A: AddressableUnit,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [A] {
        self.as_mut_buffer()
    }
}

impl<'a, T, A> TryFrom<&'a mut [T]> for ErasedFieldSliceMut<'a, A>
where
    A: AddressableUnit,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(slice: &'a mut [T]) -> Result<Self, Self::Error> {
        let ptr = ptr::from_mut(slice).try_into()?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }
}

impl<'a, A> From<ErasedFieldSliceMut<'a, A>> for ErasedFieldSlice<'a, A>
where
    A: AddressableUnit,
{
    fn from(value: ErasedFieldSliceMut<'a, A>) -> Self {
        let (desc, buffer, len) = value.into_parts();
        unsafe { ErasedFieldSlice::new_unchecked(desc, buffer, len) }
    }
}
