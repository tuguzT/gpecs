use core::{alloc::Layout, mem::MaybeUninit, ptr};

use crate::{
    error::{InsufficientAlignError, check_ptr_align, check_sufficient_align},
    field::{
        ErasedFieldPtr, ErasedFieldSlice, ErasedFieldSliceMutPtr,
        error::{
            ErasedFieldIntoValueError, ErasedFieldSlicePtrError, check_into_layout, check_slice_len,
        },
    },
    slice_item_ptr::{CastMutPtr, ConstSliceItemPtr},
    soa::field::FieldDescriptor,
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldSlicePtr<T> {
    len: usize,
    ptr: ErasedFieldPtr<T>,
}

impl<T> ErasedFieldSlicePtr<T> {
    #[inline]
    pub unsafe fn from_parts(ptr: ErasedFieldPtr<T>, len: usize) -> Self {
        Self { len, ptr }
    }

    #[inline]
    pub fn len(self) -> usize {
        let Self { len, .. } = self;
        len
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn descriptor(self) -> FieldDescriptor {
        let Self { ptr, .. } = self;
        ptr.descriptor()
    }

    #[inline]
    pub fn field_ptr(self) -> ErasedFieldPtr<T> {
        let Self { ptr, .. } = self;
        ptr
    }
}

impl<T> ErasedFieldSlicePtr<T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    pub fn cast_mut(self) -> ErasedFieldSliceMutPtr<CastMutPtr<T>> {
        let Self { ptr, len } = self;
        let ptr = ptr.cast_mut();
        unsafe { ErasedFieldSliceMutPtr::from_parts(ptr, len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldSlice<'a, T> {
        unsafe { ErasedFieldSlice::from_ptr(self) }
    }
}

impl<T, U> ErasedFieldSlicePtr<T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub fn new(
        desc: FieldDescriptor,
        buffer: *const [U],
        len: usize,
    ) -> Result<Self, ErasedFieldSlicePtrError> {
        check_sufficient_align(desc.layout(), Layout::new::<U>())?;
        check_slice_len(buffer.len() * size_of::<U>(), desc.layout().size(), len)?;
        check_ptr_align(buffer.cast(), desc.layout())?;

        let buffer = ptr::slice_from_raw_parts(buffer.cast(), buffer.len());
        let ptr = unsafe { T::from_slice(buffer, 0) };
        let ptr = unsafe { ErasedFieldPtr::from_parts(desc, ptr) };

        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }

    #[inline]
    pub fn as_uninit_buffer(self) -> *const [MaybeUninit<U>] {
        let Self { ptr, .. } = self;
        ptr.as_uninit_buffer()
    }

    #[inline]
    pub fn byte_offset(self) -> usize {
        let Self { ptr, .. } = self;
        ptr.byte_offset()
    }

    #[inline]
    pub fn as_buffer(self) -> *const [U] {
        let Self { ptr, len } = self;
        let buffer = ptr.as_buffer();
        ptr::slice_from_raw_parts(buffer.cast(), len * buffer.len())
    }

    #[inline]
    pub fn as_ptr(self) -> *const U {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }
}

impl<T, U, V> TryFrom<*const [V]> for ErasedFieldSlicePtr<T>
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(ptr: *const [V]) -> Result<Self, Self::Error> {
        let desc = FieldDescriptor::of::<V>();
        check_sufficient_align(desc.layout(), Layout::new::<U>())?;

        let len = ptr.len();
        let buffer_len = desc.layout().size().div_ceil(size_of::<U>()) * len;
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), buffer_len);
        let ptr = unsafe { T::from_slice(buffer, 0) };
        let ptr = unsafe { ErasedFieldPtr::from_parts(desc, ptr) };

        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }
}

impl<T, U, V> TryFrom<ErasedFieldSlicePtr<T>> for *const [V]
where
    T: ConstSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = ErasedFieldIntoValueError<ErasedFieldSlicePtr<T>>;

    #[inline]
    fn try_from(value: ErasedFieldSlicePtr<T>) -> Result<Self, Self::Error> {
        let value = check_into_layout::<V, _>(value.descriptor().layout(), value)?;
        let ErasedFieldSlicePtr { ptr, len, .. } = value;

        let data = ptr.as_ptr().cast();
        let slice = ptr::slice_from_raw_parts(data, len);
        Ok(slice)
    }
}
