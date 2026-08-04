use core::{
    mem::offset_of,
    ptr::{self, NonNull},
};

pub use gpecs_soa_core::ptr::*;

use crate::{
    buffer::{
        BufferData, BufferPrefix, buffer_is_dangling, buffer_layout, fields_are_zst, ptr_to_data,
        ptr_to_data_mut,
    },
    slice::SoaSlice,
    traits::{AllocSoa, AllocSoaTrusted},
};

#[inline]
pub unsafe fn slice_from_raw_parts<T>(
    data: *const BufferData<T>,
    len: usize,
    capacity: usize,
) -> *const SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    let context = unsafe { data.context() };
    let len = len_for_inner::<T>(context, len, capacity);
    ptr::slice_from_raw_parts(data, len) as _
}

#[inline]
pub unsafe fn slice_from_raw_parts_mut<T>(
    data: *mut BufferData<T>,
    len: usize,
    capacity: usize,
) -> *mut SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    let context = unsafe { data.context() };
    let len = len_for_inner::<T>(context, len, capacity);
    ptr::slice_from_raw_parts_mut(data, len) as _
}

#[inline]
fn len_for_inner<T>(context: &T::Context, len: usize, capacity: usize) -> usize
where
    T: AllocSoa + ?Sized,
{
    if buffer_is_dangling::<T>(context, capacity) {
        return len;
    }

    let capacity_in_bytes = buffer_layout::<T>(context, capacity)
        .expect("layout size should not exceed `isize::MAX`")
        .size();
    capacity_in_bytes / size_of::<BufferData<T>>()
}

pub trait SoaSlicePtr<T>: Copy + private::Sealed
where
    T: AllocSoaTrusted + ?Sized,
{
    fn as_ptr(self) -> *const BufferData<T>;

    unsafe fn context<'a>(self) -> &'a T::Context;

    unsafe fn len(self) -> usize;

    #[inline]
    unsafe fn is_empty(self) -> bool {
        unsafe { self.len() == 0 }
    }

    unsafe fn capacity(self) -> usize;
}

impl<T> SoaSlicePtr<T> for *const SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    fn as_ptr(self) -> *const BufferData<T> {
        let buffer = self.into_inner();
        buffer.cast::<BufferData<T>>() // should be `<*const [BufferData<T>]>::as_ptr(buffer)` but it's unstable
    }

    #[inline]
    unsafe fn context<'a>(self) -> &'a T::Context {
        let buffer = self.as_ptr();
        unsafe { buffer.context() }
    }

    #[inline]
    unsafe fn len(self) -> usize {
        if slice_is_dangling(self) {
            return self.into_inner().len();
        }
        unsafe { self.as_ptr().len() }
    }

    #[inline]
    unsafe fn capacity(self) -> usize {
        let context = unsafe { self.context() };
        if fields_are_zst::<T>(context) {
            return usize::MAX;
        }
        if slice_is_dangling(self) {
            return 0;
        }
        unsafe { self.as_ptr().capacity() }
    }
}

pub trait SoaSlicePtrMut<T>: Copy + private::Sealed
where
    T: AllocSoaTrusted + ?Sized,
{
    fn as_mut_ptr(self) -> *mut BufferData<T>;

    unsafe fn context<'a>(self) -> &'a T::Context;

    unsafe fn len(self) -> usize;

    #[inline]
    unsafe fn is_empty(self) -> bool {
        unsafe { self.len() == 0 }
    }

    unsafe fn capacity(self) -> usize;
}

impl<T> SoaSlicePtrMut<T> for *mut SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    fn as_mut_ptr(self) -> *mut BufferData<T> {
        let buffer = self.into_inner_mut();
        buffer.cast::<BufferData<T>>() // should be `<*mut [BufferData<T>]>::as_mut_ptr(buffer)` but it's unstable
    }

    #[inline]
    unsafe fn context<'a>(self) -> &'a T::Context {
        let buffer = self.as_mut_ptr();
        unsafe { buffer.context() }
    }

    #[inline]
    unsafe fn len(self) -> usize {
        if slice_is_dangling(self) {
            return self.into_inner_mut().len();
        }
        unsafe { self.as_mut_ptr().len() }
    }

    #[inline]
    unsafe fn capacity(self) -> usize {
        let context = unsafe { self.context() };
        if fields_are_zst::<T>(context) {
            return usize::MAX;
        }
        if slice_is_dangling(self) {
            return 0;
        }
        unsafe { self.as_mut_ptr().capacity() }
    }
}

fn slice_is_dangling<T>(ptr: *const SoaSlice<T>) -> bool
where
    T: AllocSoaTrusted + ?Sized,
{
    ptr.into_inner().len() == 0 || size_of::<BufferData<T>>() == 0
}

pub trait BufferDataPtr<T>: Copy + private::Sealed
where
    T: AllocSoa + ?Sized,
{
    unsafe fn ptr_to_context(self) -> *const T::Context;
    unsafe fn ptr_to_len(self) -> *const usize;
    unsafe fn ptr_to_capacity(self) -> *const usize;
    unsafe fn ptr_to_data(self) -> *const u8;

    #[inline]
    unsafe fn context<'a>(self) -> &'a T::Context {
        let context = unsafe { self.ptr_to_context() };
        let context = unsafe { NonNull::new_unchecked(context.cast_mut()) };
        unsafe { context.as_ref() }
    }

    #[inline]
    unsafe fn len(self) -> usize {
        let len = unsafe { self.ptr_to_len() };
        unsafe { ptr::read(len) }
    }

    #[inline]
    unsafe fn is_empty(self) -> bool {
        unsafe { self.len() == 0 }
    }

    #[inline]
    unsafe fn capacity(self) -> usize {
        let capacity = unsafe { self.ptr_to_capacity() };
        unsafe { ptr::read(capacity) }
    }
}

impl<T> BufferDataPtr<T> for *const BufferData<T>
where
    T: AllocSoa + ?Sized,
{
    #[inline]
    unsafe fn ptr_to_context(self) -> *const T::Context {
        self.cast()
    }

    #[inline]
    unsafe fn ptr_to_len(self) -> *const usize {
        let prefix = self.cast::<u8>();
        let len = unsafe { prefix.add(offset_of!(BufferPrefix<T>, len)) };
        len.cast()
    }

    #[inline]
    unsafe fn ptr_to_capacity(self) -> *const usize {
        let prefix = self.cast::<u8>();
        let capacity = unsafe { prefix.add(offset_of!(BufferPrefix<T>, capacity)) };
        capacity.cast()
    }

    #[inline]
    unsafe fn ptr_to_data(self) -> *const u8 {
        let context = unsafe { self.context() };
        let capacity = unsafe { self.capacity() };
        unsafe { ptr_to_data(context, self, capacity).unwrap_unchecked() }
    }
}

pub trait BufferDataPtrMut<T>: BufferDataPtr<T>
where
    T: AllocSoa + ?Sized,
{
    unsafe fn ptr_to_context_mut(self) -> *mut T::Context;
    unsafe fn ptr_to_len_mut(self) -> *mut usize;
    unsafe fn ptr_to_capacity_mut(self) -> *mut usize;
    unsafe fn ptr_to_data_mut(self) -> *mut u8;
}

impl<T> BufferDataPtr<T> for *mut BufferData<T>
where
    T: AllocSoa + ?Sized,
{
    #[inline]
    unsafe fn ptr_to_context(self) -> *const T::Context {
        unsafe { self.cast_const().ptr_to_context() }
    }

    #[inline]
    unsafe fn ptr_to_len(self) -> *const usize {
        unsafe { self.cast_const().ptr_to_len() }
    }

    #[inline]
    unsafe fn ptr_to_capacity(self) -> *const usize {
        unsafe { self.cast_const().ptr_to_capacity() }
    }

    #[inline]
    unsafe fn ptr_to_data(self) -> *const u8 {
        unsafe { self.cast_const().ptr_to_data() }
    }
}

impl<T> BufferDataPtrMut<T> for *mut BufferData<T>
where
    T: AllocSoa + ?Sized,
{
    #[inline]
    unsafe fn ptr_to_context_mut(self) -> *mut T::Context {
        self.cast()
    }

    #[inline]
    unsafe fn ptr_to_len_mut(self) -> *mut usize {
        let prefix = self.cast::<u8>();
        let len = unsafe { prefix.add(offset_of!(BufferPrefix<T>, len)) };
        len.cast()
    }

    #[inline]
    unsafe fn ptr_to_capacity_mut(self) -> *mut usize {
        let prefix = self.cast::<u8>();
        let capacity = unsafe { prefix.add(offset_of!(BufferPrefix<T>, capacity)) };
        capacity.cast()
    }

    #[inline]
    unsafe fn ptr_to_data_mut(self) -> *mut u8 {
        let context = unsafe { self.context() };
        let capacity = unsafe { self.capacity() };
        unsafe { ptr_to_data_mut(context, self, capacity).unwrap_unchecked() }
    }
}

trait SoaSlicePtrIntoInner<T>: Copy
where
    T: AllocSoaTrusted + ?Sized,
{
    fn into_inner(self) -> *const [BufferData<T>];
}

impl<T> SoaSlicePtrIntoInner<T> for *const SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    fn into_inner(self) -> *const [BufferData<T>] {
        self as *const [BufferData<T>]
    }
}

trait SoaSlicePtrIntoInnerMut<T>: Copy
where
    T: AllocSoaTrusted + ?Sized,
{
    fn into_inner_mut(self) -> *mut [BufferData<T>];
}

impl<T> SoaSlicePtrIntoInnerMut<T> for *mut SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    fn into_inner_mut(self) -> *mut [BufferData<T>] {
        self as *mut [BufferData<T>]
    }
}

mod private {
    use crate::{
        buffer::BufferData,
        slice::SoaSlice,
        traits::{AllocSoa, AllocSoaTrusted},
    };

    pub trait Sealed {}

    impl<T> Sealed for *const SoaSlice<T> where T: AllocSoaTrusted + ?Sized {}
    impl<T> Sealed for *mut SoaSlice<T> where T: AllocSoaTrusted + ?Sized {}

    impl<T> Sealed for *const BufferData<T> where T: AllocSoa + ?Sized {}
    impl<T> Sealed for *mut BufferData<T> where T: AllocSoa + ?Sized {}
}
