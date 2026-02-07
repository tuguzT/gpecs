use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Range,
    ptr, slice,
};

use crate::{
    error::InsufficientAlignError,
    field::{
        ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldRef,
        error::{ErasedFieldIntoValueError, ErasedFieldPtrError},
    },
    fmt::SliceUpperHex,
    slice_item_ptr::{CastConstPtr, MutSliceItemPtr},
    soa::field::FieldDescriptor,
    storage::AddressableUnit,
};

pub struct ErasedFieldRefMut<'a, T>
where
    T: MutSliceItemPtr,
{
    ptr: ErasedFieldMutPtr<T>,
    phantom: PhantomData<&'a mut [T::Item]>,
}

impl<'a, T, A> ErasedFieldRefMut<'a, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    pub fn new(desc: FieldDescriptor, buffer: &'a mut [A]) -> Result<Self, ErasedFieldPtrError> {
        let ptr = ErasedFieldMutPtr::new(desc, buffer)?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedFieldMutPtr<T>) -> Self {
        let phantom = PhantomData;
        Self { ptr, phantom }
    }

    #[inline]
    pub fn try_from<V>(r#ref: &'a mut V) -> Result<Self, InsufficientAlignError> {
        let ptr = ptr::from_mut(r#ref).try_into()?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn try_into<V>(self) -> Result<&'a mut V, ErasedFieldIntoValueError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_ptr(ptr) };
        let ptr = <*mut V>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let r#ref = unsafe { ptr.as_mut().unwrap_unchecked() };
        Ok(r#ref)
    }

    #[inline]
    pub unsafe fn cast<V>(&self) -> Result<&V, ErasedFieldIntoValueError<&Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let ptr = <*mut V>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let r#ref = unsafe { ptr.as_ref().unwrap_unchecked() };
        Ok(r#ref)
    }

    #[inline]
    pub unsafe fn cast_mut<V>(&mut self) -> Result<&mut V, ErasedFieldIntoValueError<&mut Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let ptr = <*mut V>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let r#ref = unsafe { ptr.as_mut().unwrap_unchecked() };
        Ok(r#ref)
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { ptr, .. } = self;
        ptr.descriptor()
    }

    #[inline]
    pub fn as_uninit_buffer(&self) -> &[MaybeUninit<A>] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_uninit_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_mut_uninit_buffer(&mut self) -> &mut [MaybeUninit<A>] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_mut_uninit_buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn byte_offset(&self) -> usize {
        let Self { ptr, .. } = self;
        ptr.byte_offset()
    }

    #[inline]
    pub fn buffer_init_range(&self) -> Range<usize> {
        let Self { ptr, .. } = self;
        ptr.buffer_init_range()
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
    pub fn as_field_ptr(&self) -> ErasedFieldPtr<CastConstPtr<T>> {
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
    pub fn as_mut_field_ptr(&mut self) -> ErasedFieldMutPtr<T> {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn into_buffer(self) -> &'a mut [A] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_mut_buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
    }
}

impl<T, A> Debug for ErasedFieldRefMut<'_, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<A>>,
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

impl<T, A> AsRef<[A]> for ErasedFieldRefMut<'_, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    fn as_ref(&self) -> &[A] {
        self.as_buffer()
    }
}

impl<T, A> AsMut<[A]> for ErasedFieldRefMut<'_, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [A] {
        self.as_mut_buffer()
    }
}

impl<'a, T, A> From<ErasedFieldRefMut<'a, T>> for ErasedFieldRef<'a, CastConstPtr<T>>
where
    T: MutSliceItemPtr<Item = MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    fn from(value: ErasedFieldRefMut<'a, T>) -> Self {
        let ptr = value.as_field_ptr();
        unsafe { ErasedFieldRef::from_ptr(ptr) }
    }
}
