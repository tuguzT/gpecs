use core::{alloc::Layout, mem::MaybeUninit, ptr};

use crate::{
    data::{
        ErasedMutPtr, ErasedMutSlice, ErasedSlice, ErasedSlicePtr,
        error::{DowncastError, SlicePtrError, check_downcast, check_slice_len},
    },
    error::{InsufficientAlignError, check_ptr_align, check_sufficient_align},
    layout::bytes_to_items,
    ptr::slice::{CastConstPtr, MutSliceItemPtr},
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
    pub fn cast_const(self) -> ErasedSlicePtr<CastConstPtr<T>> {
        let Self { ptr, len } = self;
        let ptr = ptr.cast_const();
        unsafe { ErasedSlicePtr::from_parts(ptr, len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSlice<'a, CastConstPtr<T>> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedMutSlice<'a, T> {
        unsafe { ErasedMutSlice::from_ptr(self) }
    }
}

impl<T, U> ErasedMutSlicePtr<T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    pub fn new(layout: Layout, buffer: *mut [U], len: usize) -> Result<Self, SlicePtrError> {
        check_sufficient_align(layout, Layout::new::<U>())?;
        check_slice_len(buffer.len() * size_of::<U>(), layout.size(), len)?;
        check_ptr_align(buffer.cast(), layout)?;

        let buffer = ptr::slice_from_raw_parts_mut(buffer.cast(), buffer.len());
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

impl<T, U, V> TryFrom<*mut [V]> for ErasedMutSlicePtr<T>
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = InsufficientAlignError;

    #[inline]
    fn try_from(ptr: *mut [V]) -> Result<Self, Self::Error> {
        let layout = Layout::new::<V>();
        check_sufficient_align(layout, Layout::new::<U>())?;

        let len = ptr.len();
        let buffer_len = bytes_to_items::<U>(layout.size()) * len;
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast(), buffer_len);

        let ptr = unsafe { MutSliceItemPtr::from_slice(buffer, 0) };
        let ptr = unsafe { ErasedMutPtr::from_parts(layout, ptr) };
        let me = unsafe { Self::from_parts(ptr, len) };
        Ok(me)
    }
}

impl<T, U, V> TryFrom<ErasedMutSlicePtr<T>> for *mut [V]
where
    T: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    type Error = DowncastError<ErasedMutSlicePtr<T>>;

    #[inline]
    fn try_from(ptr: ErasedMutSlicePtr<T>) -> Result<Self, Self::Error> {
        ptr.downcast()
    }
}
