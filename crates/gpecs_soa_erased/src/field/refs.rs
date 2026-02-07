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
        ErasedFieldPtr,
        error::{ErasedFieldIntoValueError, ErasedFieldPtrError},
    },
    fmt::SliceUpperHex,
    slice_item_ptr::ConstSliceItemPtr,
    soa::field::FieldDescriptor,
    storage::AddressableUnit,
};

pub struct ErasedFieldRef<'a, T>
where
    T: ConstSliceItemPtr,
{
    ptr: ErasedFieldPtr<T>,
    phantom: PhantomData<&'a [T::Item]>,
}

impl<'a, T, A> ErasedFieldRef<'a, T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    pub fn new(desc: FieldDescriptor, buffer: &'a [A]) -> Result<Self, ErasedFieldPtrError> {
        let ptr = ErasedFieldPtr::new(desc, buffer)?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: ErasedFieldPtr<T>) -> Self {
        let phantom = PhantomData;
        Self { ptr, phantom }
    }

    #[inline]
    pub fn try_from<V>(r#ref: &'a V) -> Result<Self, InsufficientAlignError> {
        let ptr = ptr::from_ref(r#ref).try_into()?;
        let me = unsafe { Self::from_ptr(ptr) };
        Ok(me)
    }

    #[inline]
    pub unsafe fn try_into<V>(self) -> Result<&'a V, ErasedFieldIntoValueError<Self>> {
        let Self { ptr, .. } = self;
        let into_self = |ptr| unsafe { Self::from_ptr(ptr) };
        let ptr = <*const V>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let r#ref = unsafe { ptr.as_ref().unwrap_unchecked() };
        Ok(r#ref)
    }

    #[inline]
    pub unsafe fn cast<V>(&self) -> Result<&V, ErasedFieldIntoValueError<&Self>> {
        let Self { ptr, .. } = *self;
        let into_self = |_| self;
        let ptr = <*const V>::try_from(ptr).map_err(|err| err.map_value(into_self))?;
        let r#ref = unsafe { ptr.as_ref().unwrap_unchecked() };
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
        let buffer = ptr.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const A {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr<T> {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn into_buffer(self) -> &'a [A] {
        let Self { ptr, .. } = self;
        let buffer = ptr.as_buffer();
        unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) }
    }
}

impl<T, A> Debug for ErasedFieldRef<'_, T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    A: AddressableUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = &self.descriptor();
        let buffer = &SliceUpperHex(self.as_buffer());
        f.debug_struct("ErasedFieldRef")
            .field("desc", desc)
            .field("buffer", buffer)
            .finish()
    }
}

impl<T, A> Clone for ErasedFieldRef<'_, T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, A> Copy for ErasedFieldRef<'_, T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    A: AddressableUnit,
{
}

impl<T, A> AsRef<[A]> for ErasedFieldRef<'_, T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<A>>,
    A: AddressableUnit,
{
    #[inline]
    fn as_ref(&self) -> &[A] {
        self.as_buffer()
    }
}
