use core::{alloc::Layout, ptr};

use crate::{
    data::{
        ErasedMutSlicePtr, ErasedPtr, ErasedSlice,
        error::{DataError, DowncastError, TryFromSlicePtrError, check_downcast},
    },
    error::{check_len, check_ptr_align, check_sufficient_align},
    layout::{self, bytes_to_items},
    ptr::slice::{CastMutPtr, ConstSliceItemPtr},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedSlicePtr<T> {
    len: usize,
    ptr: ErasedPtr<T>,
}

impl<T> ErasedSlicePtr<T> {
    #[inline]
    pub unsafe fn from_parts(ptr: ErasedPtr<T>, len: usize) -> Self {
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
    pub fn field_ptr(self) -> ErasedPtr<T> {
        let Self { ptr, .. } = self;
        ptr
    }

    #[inline]
    pub fn into_parts(self) -> (ErasedPtr<T>, usize) {
        let Self { ptr, len } = self;
        (ptr, len)
    }
}

impl<T> ErasedSlicePtr<T>
where
    T: ConstSliceItemPtr,
{
    #[inline]
    #[expect(
        clippy::not_unsafe_ptr_arg_deref,
        reason = "`T::from_slice` should not dereference input buffer"
    )]
    pub fn new(layout: Layout, buffer: *const [T::Item], len: usize) -> Result<Self, DataError> {
        check_ptr_align(buffer.cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let buffer_layout = Layout::array::<T::Item>(buffer.len())?;
        let (expected_layout, _) = layout::repeat(layout, len)?;
        check_len(buffer_layout.size(), expected_layout.size())?;

        let ptr = unsafe { T::from_slice(buffer, 0) };
        let ptr = unsafe { ErasedPtr::from_parts(layout, ptr) };

        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }

    #[inline]
    pub fn downcast<V>(self) -> Result<*const [V], DowncastError<Self>> {
        let layout = self.layout();
        let Self { ptr, len, .. } = check_downcast::<V, _>(layout, self)?;

        let data = ptr.as_ptr().cast();
        let slice = ptr::slice_from_raw_parts(data, len);
        Ok(slice)
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedMutSlicePtr<CastMutPtr<T>> {
        let Self { ptr, len } = self;
        let ptr = ptr.cast_mut();
        unsafe { ErasedMutSlicePtr::from_parts(ptr, len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSlice<'a, T> {
        unsafe { ErasedSlice::from_ptr(self) }
    }

    #[inline]
    pub fn as_buffer(self) -> *const [T::Item] {
        let Self { ptr, len } = self;

        let buffer = ptr.as_buffer();
        let len = buffer.len().wrapping_mul(len);
        ptr::slice_from_raw_parts(buffer.cast(), len)
    }

    #[inline]
    pub fn as_ptr(self) -> *const T::Item {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }
}

impl<T, V> TryFrom<*const [V]> for ErasedSlicePtr<T>
where
    T: ConstSliceItemPtr,
{
    type Error = TryFromSlicePtrError;

    #[inline]
    fn try_from(ptr: *const [V]) -> Result<Self, Self::Error> {
        let layout = Layout::new::<V>();
        check_ptr_align(ptr.cast(), layout)?;
        check_sufficient_align(layout, Layout::new::<T::Item>())?;

        let len = ptr.len();
        let buffer_len = bytes_to_items::<T::Item>(Layout::array::<V>(len)?.size());
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), buffer_len);

        let ptr = unsafe { T::from_slice(buffer, 0) };
        let ptr = unsafe { ErasedPtr::from_parts(layout, ptr) };
        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }
}

impl<T, V> TryFrom<ErasedSlicePtr<T>> for *const [V]
where
    T: ConstSliceItemPtr,
{
    type Error = DowncastError<ErasedSlicePtr<T>>;

    #[inline]
    fn try_from(ptr: ErasedSlicePtr<T>) -> Result<Self, Self::Error> {
        ptr.downcast()
    }
}
