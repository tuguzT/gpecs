use core::{alloc::Layout, mem::MaybeUninit, ptr};

use crate::{
    bytes_to_items::item_count,
    error::{InsufficientAlignError, check_ptr_align, check_sufficient_align},
    field::{
        ErasedFieldMutPtr, ErasedFieldSlice, ErasedFieldSliceMut, ErasedFieldSlicePtr,
        error::{DowncastError, SlicePtrError, check_downcast, check_slice_len},
    },
    ptr::slice::{CastConstPtr, MutSliceItemPtr},
    soa::field::FieldDescriptor,
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldSliceMutPtr<T> {
    len: usize,
    ptr: ErasedFieldMutPtr<T>,
}

impl<T> ErasedFieldSliceMutPtr<T> {
    #[inline]
    pub unsafe fn from_parts(ptr: ErasedFieldMutPtr<T>, len: usize) -> Self {
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
    pub fn field_ptr(self) -> ErasedFieldMutPtr<T> {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (ErasedFieldMutPtr<T>, usize) {
        let Self { ptr, len } = self;
        (ptr, len)
    }
}

impl<T> ErasedFieldSliceMutPtr<T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    pub fn cast_const(self) -> ErasedFieldSlicePtr<CastConstPtr<T>> {
        let Self { ptr, len } = self;
        let ptr = ptr.cast_const();
        unsafe { ErasedFieldSlicePtr::from_parts(ptr, len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldSlice<'a, CastConstPtr<T>> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedFieldSliceMut<'a, T> {
        unsafe { ErasedFieldSliceMut::from_ptr(self) }
    }
}

impl<T, U> ErasedFieldSliceMutPtr<T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub fn new(desc: FieldDescriptor, buffer: *mut [U], len: usize) -> Result<Self, SlicePtrError> {
        check_sufficient_align(desc.layout(), Layout::new::<U>())?;
        check_slice_len(buffer.len() * size_of::<U>(), desc.layout().size(), len)?;
        check_ptr_align(buffer.cast(), desc.layout())?;

        let buffer = ptr::slice_from_raw_parts_mut(buffer.cast(), buffer.len());
        let ptr = unsafe { T::from_slice(buffer, 0) };
        let ptr = unsafe { ErasedFieldMutPtr::from_parts(desc, ptr) };

        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<V>(self) -> Result<*mut [V], DowncastError<Self>> {
        let layout = self.descriptor().layout();
        let Self { ptr, len, .. } = check_downcast::<V, _>(layout, self)?;

        let data = ptr.as_mut_ptr().cast();
        let slice = ptr::slice_from_raw_parts_mut(data, len);
        Ok(slice)
    }

    #[inline]
    pub fn as_uninit_buffer(self) -> *const [MaybeUninit<U>] {
        let Self { ptr, .. } = self;
        ptr.as_uninit_buffer()
    }

    #[inline]
    pub fn as_mut_uninit_buffer(self) -> *mut [MaybeUninit<U>] {
        let Self { ptr, .. } = self;
        ptr.as_mut_uninit_buffer()
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
    pub fn as_mut_buffer(self) -> *mut [U] {
        let Self { ptr, len } = self;
        let buffer = ptr.as_mut_buffer();
        ptr::slice_from_raw_parts_mut(buffer.cast(), len * buffer.len())
    }

    #[inline]
    pub fn as_ptr(self) -> *const U {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(self) -> *mut U {
        let Self { ptr, .. } = self;
        ptr.as_mut_ptr()
    }
}

impl<T, U, V> TryFrom<*mut [V]> for ErasedFieldSliceMutPtr<T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(ptr: *mut [V]) -> Result<Self, Self::Error> {
        let desc = FieldDescriptor::of::<V>();
        check_sufficient_align(desc.layout(), Layout::new::<U>())?;

        let len = ptr.len();
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast(), item_count::<U>(desc) * len);

        let ptr = unsafe { MutSliceItemPtr::from_slice(buffer, 0) };
        let ptr = unsafe { ErasedFieldMutPtr::from_parts(desc, ptr) };
        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }
}

impl<T, U, V> TryFrom<ErasedFieldSliceMutPtr<T>> for *mut [V]
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = DowncastError<ErasedFieldSliceMutPtr<T>>;

    #[inline]
    fn try_from(ptr: ErasedFieldSliceMutPtr<T>) -> Result<Self, Self::Error> {
        ptr.downcast()
    }
}
