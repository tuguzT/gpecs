use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    mem::MaybeUninit,
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
    slice_item_ptr::{CastConstPtr, MutSliceItemPtr},
    soa::field::FieldDescriptor,
    storage::AddressableUnit,
};

pub struct ErasedFieldSliceMut<'a, T>
where
    T: MutSliceItemPtr,
{
    ptr: ErasedFieldSliceMutPtr<T>,
    phantom: PhantomData<&'a mut [T::Item]>,
}

impl<'a, T, A> ErasedFieldSliceMut<'a, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<A>>,
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
    pub unsafe fn from_ptr(ptr: ErasedFieldSliceMutPtr<T>) -> Self {
        let phantom = PhantomData;
        Self { ptr, phantom }
    }

    #[inline]
    pub unsafe fn try_into<V>(self) -> Result<&'a mut [V], ErasedFieldIntoValueError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_ptr(ptr) };
        let buffer = <*mut [V]>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn cast<V>(&self) -> Result<&[V], ErasedFieldIntoValueError<&Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let buffer = <*mut [V]>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn cast_mut<V>(&mut self) -> Result<&mut [V], ErasedFieldIntoValueError<&mut Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let buffer = <*mut [V]>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
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
    pub fn byte_offset(self) -> usize {
        let Self { ptr, .. } = self;
        ptr.byte_offset()
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
    pub fn as_field_slice_ptr(&self) -> ErasedFieldSlicePtr<CastConstPtr<T>> {
        let Self { ptr, .. } = self;
        ptr.cast_const()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr<CastConstPtr<T>> {
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
    pub fn as_mut_field_slice_ptr(&mut self) -> ErasedFieldSliceMutPtr<T> {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_mut_field_ptr(&mut self) -> ErasedFieldMutPtr<T> {
        let Self { ptr, .. } = self;
        ptr.as_mut_field_ptr()
    }

    #[inline]
    pub fn into_buffer(self) -> &'a mut [A] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_mut_buffer();
        unsafe { slice::from_raw_parts_mut(buffer.cast(), buffer.len()) }
    }
}

impl<T, A> Debug for ErasedFieldSliceMut<'_, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<A>>,
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

impl<T, A> AsRef<[A]> for ErasedFieldSliceMut<'_, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    fn as_ref(&self) -> &[A] {
        self.as_buffer()
    }
}

impl<T, A> AsMut<[A]> for ErasedFieldSliceMut<'_, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [A] {
        self.as_mut_buffer()
    }
}

impl<'a, T, V, A> TryFrom<&'a mut [V]> for ErasedFieldSliceMut<'a, T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<A>>,
    A: AddressableUnit,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(slice: &'a mut [V]) -> Result<Self, Self::Error> {
        let ptr = ptr::from_mut(slice).try_into()?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }
}

impl<'a, T, A> From<ErasedFieldSliceMut<'a, T>> for ErasedFieldSlice<'a, CastConstPtr<T>>
where
    T: MutSliceItemPtr<Item = MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    fn from(value: ErasedFieldSliceMut<'a, T>) -> Self {
        let ptr = value.as_field_slice_ptr();
        unsafe { ErasedFieldSlice::from_ptr(ptr) }
    }
}
