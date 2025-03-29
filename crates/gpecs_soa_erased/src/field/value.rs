use alloc::boxed::Box;
use core::{
    fmt::{self, Debug},
    mem::ManuallyDrop,
    ptr, slice,
};

use crate::{
    align::{Align, Aligned, Unaligned},
    assert::{check_same_len, validate_layout},
    byte::ErasedByte,
    erased::error::ErasedSoaError,
    soa::traits::FieldDescriptor,
};

use super::{
    assert::check_layout, error::IntoValueError, ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldRef,
    ErasedFieldRefMut,
};

pub struct ErasedField<A>
where
    A: Align,
{
    desc: FieldDescriptor,
    buffer: Box<[ErasedByte<A>]>,
}

impl<A> ErasedField<A>
where
    A: Align,
{
    #[inline]
    #[track_caller]
    pub fn new<B>(desc: FieldDescriptor, buffer: B) -> Result<Self, ErasedSoaError>
    where
        B: AsRef<[u8]>,
    {
        if A::IS_ALIGNED {
            validate_layout::<A>(desc.layout())?;
        }
        check_same_len(buffer.as_ref().len(), desc.layout().size())?;

        let me = unsafe { Self::actual_new(desc, buffer) };
        Ok(me)
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked<B>(desc: FieldDescriptor, buffer: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer).expect("incorrect inputs");
        }
        unsafe { Self::actual_new(desc, buffer) }
    }

    #[inline]
    unsafe fn actual_new<B>(desc: FieldDescriptor, buffer: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        let buffer_len = buffer.as_ref().len().div_ceil(size_of::<ErasedByte<A>>());
        let mut r#box = Box::new_uninit_slice(buffer_len);
        unsafe {
            ptr::copy_nonoverlapping(
                buffer.as_ref().as_ptr(),
                r#box.as_mut_ptr().cast(),
                desc.layout().size(),
            );
        }

        let buffer = unsafe { r#box.assume_init() };
        Self { desc, buffer }
    }

    #[inline]
    pub fn from<T>(value: T) -> Self {
        let value = ManuallyDrop::new(value);
        let desc = FieldDescriptor::of::<T>();
        let data = ptr::from_ref(&value).cast();
        let buffer = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
        unsafe { Self::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub unsafe fn into<T>(self) -> Result<T, IntoValueError<Self>> {
        let me = check_layout::<T, _>(self.desc.layout(), self)?;
        let Self { buffer, .. } = me;

        let src = buffer.as_ptr().cast();
        if A::IS_ALIGNED {
            Ok(unsafe { ptr::read(src) })
        } else {
            Ok(unsafe { ptr::read_unaligned(src) })
        }
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { desc, .. } = *self;
        desc
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        let Self { ref desc, buffer } = self;

        let data = buffer.as_ptr().cast();
        unsafe { slice::from_raw_parts(data, desc.layout().size()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { buffer, .. } = self;
        buffer.as_ptr().cast()
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        let Self { ref desc, buffer } = self;

        let data = buffer.as_mut_ptr().cast();
        unsafe { slice::from_raw_parts_mut(data, desc.layout().size()) }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { buffer, .. } = self;
        buffer.as_mut_ptr().cast()
    }

    #[inline]
    pub fn into_buffer(self) -> Box<[u8]> {
        let (_, buffer) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, Box<[u8]>) {
        let Self { desc, buffer } = self;

        let data = buffer.as_ptr().cast();
        let buffer = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
        (desc, buffer.into())
    }
}

impl<Fields> ErasedField<Aligned<Fields>> {
    #[inline]
    #[track_caller]
    pub unsafe fn cast<T>(&self) -> Result<&T, IntoValueError<&Self>> {
        let me = check_layout::<T, _>(self.desc.layout(), self)?;
        let Self { buffer, .. } = me;

        let ptr = buffer.as_ptr().cast();
        Ok(unsafe { &*ptr })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn cast_mut<T>(&mut self) -> Result<&mut T, IntoValueError<&mut Self>> {
        let me = check_layout::<T, _>(self.desc.layout(), self)?;
        let Self { buffer, .. } = me;

        let ptr = buffer.as_mut_ptr().cast();
        Ok(unsafe { &mut *ptr })
    }

    #[inline]
    pub fn as_field_ref(&self) -> ErasedFieldRef<'_> {
        let Self { desc, .. } = *self;
        let buffer = self.buffer();
        unsafe { ErasedFieldRef::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { desc, .. } = *self;
        let buffer = ptr::from_ref(self.buffer());
        unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn as_field_ref_mut(&mut self) -> ErasedFieldRefMut<'_> {
        let Self { desc, .. } = *self;
        let buffer = self.buffer_mut();
        unsafe { ErasedFieldRefMut::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn as_field_ptr_mut(&mut self) -> ErasedFieldMutPtr {
        let Self { desc, .. } = *self;
        let buffer = ptr::from_mut(self.buffer_mut());
        unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn into_unaligned(self) -> ErasedField<Unaligned> {
        let (desc, buffer) = self.into_parts();
        unsafe { ErasedField::new_unchecked(desc, &buffer) }
    }
}

impl ErasedField<Unaligned> {
    #[inline]
    pub fn into_aligned<F>(self) -> ErasedField<Aligned<F>> {
        let (desc, buffer) = self.into_parts();
        unsafe { ErasedField::new_unchecked(desc, &buffer) }
    }
}

impl<A> Debug for ErasedField<A>
where
    A: Align,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { desc, .. } = self;
        let buffer = &self.buffer();
        let aligned = &A::IS_ALIGNED;
        f.debug_struct("ErasedField")
            .field("desc", desc)
            .field("buffer", buffer)
            .field("aligned", aligned)
            .finish()
    }
}

impl<A> AsRef<[u8]> for ErasedField<A>
where
    A: Align,
{
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.buffer()
    }
}

impl<A> AsMut<[u8]> for ErasedField<A>
where
    A: Align,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.buffer_mut()
    }
}
