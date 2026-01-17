use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr, slice,
};

use crate::{
    error::InsufficientAlignError,
    field::{
        ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldRef,
        error::{ErasedFieldIntoValueError, ErasedFieldPtrError},
    },
    fmt::SliceUpperHex,
    soa::field::FieldDescriptor,
    storage::AddressableUnit,
};

pub struct ErasedFieldRefMut<'a, A>
where
    A: AddressableUnit,
{
    ptr: ErasedFieldMutPtr<A>,
    phantom: PhantomData<&'a mut [A]>,
}

impl<'a, A> ErasedFieldRefMut<'a, A>
where
    A: AddressableUnit,
{
    #[inline]
    pub fn new(desc: FieldDescriptor, buffer: &'a mut [A]) -> Result<Self, ErasedFieldPtrError> {
        let ptr = ErasedFieldMutPtr::new(desc, buffer)?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &'a mut [A]) -> Self {
        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) };
        unsafe { Self::from_ptr(ptr) }
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedFieldMutPtr<A>) -> Self {
        let phantom = PhantomData;
        Self { ptr, phantom }
    }

    #[inline]
    pub fn try_from<T>(r#ref: &'a mut T) -> Result<Self, InsufficientAlignError> {
        let ptr = ptr::from_mut(r#ref).try_into()?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn try_into<T>(self) -> Result<&'a mut T, ErasedFieldIntoValueError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_ptr(ptr) };
        let ptr = <*mut T>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let r#ref = unsafe { ptr.as_mut().unwrap_unchecked() };
        Ok(r#ref)
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&T, ErasedFieldIntoValueError<&Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let ptr = <*mut T>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let r#ref = unsafe { ptr.as_ref().unwrap_unchecked() };
        Ok(r#ref)
    }

    #[inline]
    pub unsafe fn cast_mut<T>(&mut self) -> Result<&mut T, ErasedFieldIntoValueError<&mut Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let ptr = <*mut T>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let r#ref = unsafe { ptr.as_mut().unwrap_unchecked() };
        Ok(r#ref)
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { ptr, .. } = self;
        ptr.descriptor()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[A] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_mut_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const A {
        let Self { ptr, .. } = self;
        ptr.as_mut_ptr().cast_const()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr<A> {
        let Self { ptr, .. } = *self;
        ptr.cast_const()
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
    pub fn as_mut_field_ptr(&mut self) -> ErasedFieldMutPtr<A> {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn into_buffer(self) -> &'a mut [A] {
        let (_, buffer) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, &'a mut [A]) {
        let Self { ptr, .. } = self;
        let (desc, buffer) = ptr.into_parts();
        let buffer = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        (desc, buffer)
    }
}

impl<A> Debug for ErasedFieldRefMut<'_, A>
where
    A: AddressableUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &SliceUpperHex(self.as_buffer());
        f.debug_struct("ErasedFieldRefMut")
            .field("desc", desc)
            .field("buffer", buffer)
            .finish()
    }
}

impl<A> AsRef<[A]> for ErasedFieldRefMut<'_, A>
where
    A: AddressableUnit,
{
    #[inline]
    fn as_ref(&self) -> &[A] {
        self.as_buffer()
    }
}

impl<A> AsMut<[A]> for ErasedFieldRefMut<'_, A>
where
    A: AddressableUnit,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [A] {
        self.as_mut_buffer()
    }
}

impl<'a, A> From<ErasedFieldRefMut<'a, A>> for ErasedFieldRef<'a, A>
where
    A: AddressableUnit,
{
    #[inline]
    fn from(value: ErasedFieldRefMut<'a, A>) -> Self {
        let ptr = value.as_field_ptr();
        unsafe { ErasedFieldRef::from_ptr(ptr) }
    }
}
