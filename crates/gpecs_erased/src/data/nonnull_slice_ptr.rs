use core::{alloc::Layout, ptr::NonNull};

use gpecs_ptr::slice::{NonNullAsMutPtr, NonNullAsPtr};

use crate::{
    data::{
        ErasedMutSlicePtr, ErasedNonNullPtr, ErasedSlicePtr,
        error::{DataError, DowncastError, UpcastSliceError, check_downcast},
    },
    error::{check_len, check_ptr_align, check_sufficient_align},
    layout::bytes_to_items,
    ptr::slice::NonNullSliceItemPtr,
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedNonNullSlicePtr<T> {
    len: usize,
    ptr: ErasedNonNullPtr<T>,
}

impl<T> ErasedNonNullSlicePtr<T> {
    #[inline]
    pub unsafe fn from_parts(ptr: ErasedNonNullPtr<T>, len: usize) -> Self {
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
    pub fn layout(self) -> Layout {
        let Self { ptr, .. } = self;
        ptr.layout()
    }

    #[inline]
    pub fn field_ptr(self) -> ErasedNonNullPtr<T> {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (ErasedNonNullPtr<T>, usize) {
        let Self { ptr, len } = self;
        (ptr, len)
    }
}

impl<T> ErasedNonNullSlicePtr<T>
where
    T: NonNullSliceItemPtr,
{
    #[inline]
    pub fn new(layout: Layout, buffer: NonNull<[T::Item]>, len: usize) -> Result<Self, DataError> {
        check_ptr_align(buffer.as_ptr().cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let buffer_layout = Layout::array::<T::Item>(buffer.len())?;
        let expected_layout = layout.pad_to_align().repeat_packed(len)?;
        check_len(buffer_layout.size(), expected_layout.size())?;

        let ptr = unsafe { T::from_slice(buffer, 0) };
        let ptr = unsafe { ErasedNonNullPtr::from_parts(layout, ptr) };

        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }

    #[inline]
    pub fn upcast<V>(ptr: NonNull<[V]>) -> Result<Self, UpcastSliceError> {
        let layout = Layout::new::<V>();
        check_ptr_align(ptr.as_ptr().cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let len = ptr.len();
        let buffer_len = bytes_to_items::<T::Item>(Layout::array::<V>(len)?.size());
        let buffer = NonNull::slice_from_raw_parts(ptr.cast(), buffer_len);

        let ptr = unsafe { NonNullSliceItemPtr::from_slice(buffer, 0) };
        let ptr = unsafe { ErasedNonNullPtr::from_parts(layout, ptr) };
        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<V>(self) -> Result<NonNull<[V]>, DowncastError<Self>> {
        let layout = self.layout();
        let Self { ptr, len, .. } = check_downcast::<V, _>(layout, self)?;

        let data = ptr.as_ptr().cast();
        let slice = NonNull::slice_from_raw_parts(data, len);
        Ok(slice)
    }

    #[inline]
    pub fn into_slice_ptr(self) -> ErasedSlicePtr<NonNullAsPtr<T>> {
        let Self { ptr, len } = self;
        let ptr = ptr.into_ptr();
        unsafe { ErasedSlicePtr::from_parts(ptr, len) }
    }

    #[inline]
    pub fn into_mut_slice_ptr(self) -> ErasedMutSlicePtr<NonNullAsMutPtr<T>> {
        let Self { ptr, len } = self;
        let ptr = ptr.into_mut_ptr();
        unsafe { ErasedMutSlicePtr::from_parts(ptr, len) }
    }

    #[inline]
    pub fn as_buffer(self) -> NonNull<[T::Item]> {
        let Self { ptr, len } = self;

        let buffer = ptr.as_buffer();
        let len = buffer.len().wrapping_mul(len);
        NonNull::slice_from_raw_parts(buffer.cast(), len)
    }

    #[inline]
    pub fn as_ptr(self) -> NonNull<T::Item> {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }
}
