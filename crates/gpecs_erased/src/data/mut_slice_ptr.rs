use core::{alloc::Layout, ptr};

use crate::{
    data::{
        ErasedMutPtr, ErasedMutSlice, ErasedSlice, ErasedSlicePtr,
        error::{DataError, DowncastError, TryFromSlicePtrError, check_downcast},
    },
    error::{check_len, check_ptr_align, check_sufficient_align},
    layout::{self, bytes_to_items},
    ptr::slice::{CastConst, MutSliceItemPtr},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedMutSlicePtr<T> {
    len: usize,
    ptr: ErasedMutPtr<T>,
}

impl<T> ErasedMutSlicePtr<T> {
    #[inline]
    pub unsafe fn from_parts(ptr: ErasedMutPtr<T>, len: usize) -> Self {
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
    pub fn field_ptr(self) -> ErasedMutPtr<T> {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (ErasedMutPtr<T>, usize) {
        let Self { ptr, len } = self;
        (ptr, len)
    }
}

impl<T> ErasedMutSlicePtr<T>
where
    T: MutSliceItemPtr,
{
    #[inline]
    #[expect(
        clippy::not_unsafe_ptr_arg_deref,
        reason = "`T::from_slice` should not dereference input buffer"
    )]
    pub fn new(layout: Layout, buffer: *mut [T::Item], len: usize) -> Result<Self, DataError> {
        check_ptr_align(buffer.cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let buffer_layout = Layout::array::<T::Item>(buffer.len())?;
        let (expected_layout, _) = layout::repeat(layout, len)?;
        check_len(buffer_layout.size(), expected_layout.size())?;

        let ptr = unsafe { T::from_slice(buffer, 0) };
        let ptr = unsafe { ErasedMutPtr::from_parts(layout, ptr) };

        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<V>(self) -> Result<*mut [V], DowncastError<Self>> {
        let layout = self.layout();
        let Self { ptr, len, .. } = check_downcast::<V, _>(layout, self)?;

        let data = ptr.as_mut_ptr().cast();
        let slice = ptr::slice_from_raw_parts_mut(data, len);
        Ok(slice)
    }

    #[inline]
    pub fn cast_const(self) -> ErasedSlicePtr<CastConst<T>> {
        let Self { ptr, len } = self;
        let ptr = ptr.cast_const();
        unsafe { ErasedSlicePtr::from_parts(ptr, len) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedSlice<'a, CastConst<T>> {
        unsafe { self.cast_const().as_ref_unchecked() }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked<'a>(self) -> ErasedMutSlice<'a, T> {
        unsafe { ErasedMutSlice::from_ptr(self) }
    }

    #[inline]
    pub fn as_buffer(self) -> *const [T::Item] {
        let Self { ptr, len } = self;

        let buffer = ptr.as_buffer();
        let len = buffer.len().wrapping_mul(len);
        ptr::slice_from_raw_parts(buffer.cast(), len)
    }

    #[inline]
    pub fn as_mut_buffer(self) -> *mut [T::Item] {
        let Self { ptr, len } = self;

        let buffer = ptr.as_mut_buffer();
        let len = buffer.len().wrapping_mul(len);
        ptr::slice_from_raw_parts_mut(buffer.cast(), len)
    }

    #[inline]
    pub fn as_ptr(self) -> *const T::Item {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(self) -> *mut T::Item {
        let Self { ptr, .. } = self;
        ptr.as_mut_ptr()
    }
}

impl<T, V> TryFrom<*mut [V]> for ErasedMutSlicePtr<T>
where
    T: MutSliceItemPtr,
{
    type Error = TryFromSlicePtrError;

    #[inline]
    fn try_from(ptr: *mut [V]) -> Result<Self, Self::Error> {
        let layout = Layout::new::<V>();
        check_ptr_align(ptr.cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let len = ptr.len();
        let buffer_len = bytes_to_items::<T::Item>(Layout::array::<V>(len)?.size());
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast(), buffer_len);

        let ptr = unsafe { MutSliceItemPtr::from_slice(buffer, 0) };
        let ptr = unsafe { ErasedMutPtr::from_parts(layout, ptr) };
        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }
}

impl<T, V> TryFrom<ErasedMutSlicePtr<T>> for *mut [V]
where
    T: MutSliceItemPtr,
{
    type Error = DowncastError<ErasedMutSlicePtr<T>>;

    #[inline]
    fn try_from(ptr: ErasedMutSlicePtr<T>) -> Result<Self, Self::Error> {
        ptr.downcast()
    }
}
