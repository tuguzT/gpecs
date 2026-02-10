use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr, slice,
};

use crate::{
    error::InsufficientAlignError,
    field::{
        ErasedFieldPtr, ErasedFieldSlicePtr,
        error::{ErasedFieldIntoValueError, ErasedFieldSlicePtrError},
    },
    fmt::SliceUpperHex,
    slice_item_ptr::ConstSliceItemPtr,
    soa::field::FieldDescriptor,
    storage::AddressableUnit,
};

pub struct ErasedFieldSlice<'a, T>
where
    T: ConstSliceItemPtr,
{
    ptr: ErasedFieldSlicePtr<T>,
    phantom: PhantomData<&'a [T::Item]>,
}

impl<T> ErasedFieldSlice<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedFieldSlicePtr<T>) -> Self {
        let phantom = PhantomData;
        Self { ptr, phantom }
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
    pub fn as_field_slice_ptr(&self) -> ErasedFieldSlicePtr<T> {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr<T> {
        let Self { ptr, .. } = self;
        ptr.field_ptr()
    }
}

impl<'a, T, U> ErasedFieldSlice<'a, T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
    U: AddressableUnit,
{
    #[inline]
    pub fn new(
        desc: FieldDescriptor,
        buffer: &'a [U],
        len: usize,
    ) -> Result<Self, ErasedFieldSlicePtrError> {
        let ptr = ErasedFieldSlicePtr::new(desc, buffer, len)?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn try_into<V>(self) -> Result<&'a [V], ErasedFieldIntoValueError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_ptr(ptr) };
        let buffer = <*const [V]>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub unsafe fn cast<V>(&self) -> Result<&[V], ErasedFieldIntoValueError<&Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let buffer = <*const [V]>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let slice = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        Ok(slice)
    }

    #[inline]
    pub fn as_uninit_buffer(&self) -> &[MaybeUninit<U>] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_uninit_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn byte_offset(self) -> usize {
        let Self { ptr, .. } = self;
        ptr.byte_offset()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[U] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const U {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn into_buffer(self) -> &'a [U] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }
}

impl<T, U> Debug for ErasedFieldSlice<'_, T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
    U: AddressableUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &SliceUpperHex(self.as_buffer());
        let len = &self.len();
        f.debug_struct("ErasedFieldSlice")
            .field("desc", desc)
            .field("buffer", buffer)
            .field("len", len)
            .finish()
    }
}

impl<T> Clone for ErasedFieldSlice<'_, T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ErasedFieldSlice<'_, T> where T: ConstSliceItemPtr {}

impl<T, U> AsRef<[U]> for ErasedFieldSlice<'_, T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
    U: AddressableUnit,
{
    #[inline]
    fn as_ref(&self) -> &[U] {
        self.as_buffer()
    }
}

impl<'a, T, U, V> TryFrom<&'a [V]> for ErasedFieldSlice<'a, T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
    U: AddressableUnit,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(slice: &'a [V]) -> Result<Self, Self::Error> {
        let ptr = ptr::from_ref(slice).try_into()?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }
}
