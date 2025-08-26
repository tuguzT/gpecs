use core::{
    alloc::{Layout, LayoutError},
    mem::offset_of,
    ptr::{self, NonNull},
};

use crate::{
    layout::{BufferData, BufferPrefix, buffer_layout, is_zst, should_allocate},
    slice::SoaSlice,
    traits::{Soa, SoaTrustedFields},
};

#[inline]
pub unsafe fn slice_from_raw_parts<T>(
    data: *const BufferData<T>,
    len: usize,
    capacity: usize,
) -> *const SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
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
    T: SoaTrustedFields + ?Sized,
{
    let context = unsafe { data.context() };
    let len = len_for_inner::<T>(context, len, capacity);
    ptr::slice_from_raw_parts_mut(data, len) as _
}

#[inline]
fn len_for_inner<T>(context: &T::Context, len: usize, capacity: usize) -> usize
where
    T: Soa + ?Sized,
{
    if !should_allocate::<T>(context, capacity) {
        return len;
    }

    let capacity_in_bytes = buffer_layout::<T>(context, capacity)
        .expect("layout size should not exceed `isize::MAX`")
        .size();
    capacity_in_bytes / size_of::<BufferData<T>>()
}

pub trait SoaSlicePtr<T>: Copy + private::Sealed
where
    T: SoaTrustedFields + ?Sized,
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
    T: SoaTrustedFields + ?Sized,
{
    #[inline]
    fn as_ptr(self) -> *const BufferData<T> {
        let buffer = self.into_inner();
        buffer.cast::<BufferData<T>>() // should be `<*const [BufferData<T>]>::as_ptr(buffer)` but it's unstable
    }

    #[inline]
    unsafe fn context<'a>(self) -> &'a <T as Soa>::Context {
        let buffer = self.as_ptr();
        unsafe { buffer.context() }
    }

    #[inline]
    unsafe fn len(self) -> usize {
        let buffer_layout = slice_buffer_layout(self);
        match buffer_layout.size() {
            0 => self.into_inner().len(),
            _ => unsafe { self.as_ptr().len() },
        }
    }

    #[inline]
    unsafe fn capacity(self) -> usize {
        let context = unsafe { self.context() };
        if is_zst::<T>(context) {
            return usize::MAX;
        }

        let buffer_layout = slice_buffer_layout(self);
        match buffer_layout.size() {
            0 => 0,
            _ => unsafe { self.as_ptr().capacity() },
        }
    }
}

pub trait SoaSlicePtrMut<T>: Copy + private::Sealed
where
    T: SoaTrustedFields + ?Sized,
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
    T: SoaTrustedFields + ?Sized,
{
    #[inline]
    fn as_mut_ptr(self) -> *mut BufferData<T> {
        let buffer = self.into_inner_mut();
        buffer.cast::<BufferData<T>>() // should be `<*mut [BufferData<T>]>::as_mut_ptr(buffer)` but it's unstable
    }

    #[inline]
    unsafe fn context<'a>(self) -> &'a <T as Soa>::Context {
        let buffer = self.as_mut_ptr();
        unsafe { buffer.context() }
    }

    #[inline]
    unsafe fn len(self) -> usize {
        let buffer_layout = slice_buffer_layout(self);
        match buffer_layout.size() {
            0 => self.into_inner_mut().len(),
            _ => unsafe { self.as_mut_ptr().len() },
        }
    }

    #[inline]
    unsafe fn capacity(self) -> usize {
        let context = unsafe { self.context() };
        if is_zst::<T>(context) {
            return usize::MAX;
        }

        let buffer_layout = slice_buffer_layout(self);
        match buffer_layout.size() {
            0 => 0,
            _ => unsafe { self.as_mut_ptr().capacity() },
        }
    }
}

fn slice_buffer_layout<T>(ptr: *const SoaSlice<T>) -> Layout
where
    T: SoaTrustedFields + ?Sized,
{
    let buffer = ptr.into_inner();

    let size = buffer.len() * size_of::<BufferData<T>>();
    let align = align_of::<BufferData<T>>();
    Layout::from_size_align(size, align).expect("layout size should not exceed `isize::MAX`")
}

pub trait BufferDataPtr<T>: Copy + private::Sealed
where
    T: Soa + ?Sized,
{
    fn ptr_to_context(self) -> *const T::Context;
    unsafe fn ptr_to_len(self) -> *const usize;
    unsafe fn ptr_to_capacity(self) -> *const usize;
    unsafe fn ptr_to_data(self) -> *const u8;

    #[inline]
    unsafe fn context<'a>(self) -> &'a T::Context {
        let context = self.ptr_to_context();
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
    T: Soa + ?Sized,
{
    #[inline]
    fn ptr_to_context(self) -> *const T::Context {
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
    T: Soa + ?Sized,
{
    fn ptr_to_context_mut(self) -> *mut T::Context;
    unsafe fn ptr_to_len_mut(self) -> *mut usize;
    unsafe fn ptr_to_capacity_mut(self) -> *mut usize;
    unsafe fn ptr_to_data_mut(self) -> *mut u8;
}

impl<T> BufferDataPtr<T> for *mut BufferData<T>
where
    T: Soa + ?Sized,
{
    #[inline]
    fn ptr_to_context(self) -> *const T::Context {
        self.cast_const().ptr_to_context()
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
    T: Soa + ?Sized,
{
    #[inline]
    fn ptr_to_context_mut(self) -> *mut T::Context {
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
    T: SoaTrustedFields + ?Sized,
{
    fn into_inner(self) -> *const [BufferData<T>];
}

impl<T> SoaSlicePtrIntoInner<T> for *const SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
{
    #[inline]
    fn into_inner(self) -> *const [BufferData<T>] {
        self as *const [BufferData<T>]
    }
}

trait SoaSlicePtrIntoInnerMut<T>: Copy
where
    T: SoaTrustedFields + ?Sized,
{
    fn into_inner_mut(self) -> *mut [BufferData<T>];
}

impl<T> SoaSlicePtrIntoInnerMut<T> for *mut SoaSlice<T>
where
    T: SoaTrustedFields + ?Sized,
{
    #[inline]
    fn into_inner_mut(self) -> *mut [BufferData<T>] {
        self as *mut [BufferData<T>]
    }
}

mod private {
    use crate::{
        layout::BufferData,
        slice::SoaSlice,
        traits::{Soa, SoaTrustedFields},
    };

    pub trait Sealed {}

    impl<T> Sealed for *const SoaSlice<T> where T: SoaTrustedFields + ?Sized {}
    impl<T> Sealed for *mut SoaSlice<T> where T: SoaTrustedFields + ?Sized {}

    impl<T> Sealed for *const BufferData<T> where T: Soa + ?Sized {}
    impl<T> Sealed for *mut BufferData<T> where T: Soa + ?Sized {}
}

#[inline]
pub(crate) unsafe fn ptrs_from_buffer<T>(
    context: &T::Context,
    ptr: *const BufferData<T>,
    capacity: usize,
) -> T::Ptrs<'_>
where
    T: Soa + ?Sized,
{
    if is_zst::<T>(context) || capacity == 0 {
        return T::ptrs_dangling(context);
    }

    let buffer = unsafe { ptr_to_data(context, ptr, capacity).unwrap_unchecked() };
    unsafe { T::ptrs_from_buffer(context, buffer, capacity) }
}

#[inline]
pub(crate) unsafe fn ptrs_from_buffer_mut<T>(
    context: &T::Context,
    ptr: *mut BufferData<T>,
    capacity: usize,
) -> T::MutPtrs<'_>
where
    T: Soa + ?Sized,
{
    if is_zst::<T>(context) || capacity == 0 {
        return T::ptrs_dangling_mut(context);
    }

    let buffer = unsafe { ptr_to_data_mut(context, ptr, capacity).unwrap_unchecked() };
    unsafe { T::ptrs_from_buffer_mut(context, buffer, capacity) }
}

unsafe fn ptr_to_data<T>(
    context: &T::Context,
    ptr: *const BufferData<T>,
    capacity: usize,
) -> Result<*const u8, LayoutError>
where
    T: Soa + ?Sized,
{
    let layout = T::buffer_layout(context, capacity)?;
    let prefix_layout = Layout::new::<BufferPrefix<T>>();
    let (_, offset_from_prefix) = prefix_layout.extend(layout)?;

    let buffer = unsafe { ptr.cast::<u8>().add(offset_from_prefix) };
    Ok(buffer)
}

unsafe fn ptr_to_data_mut<T>(
    context: &T::Context,
    ptr: *mut BufferData<T>,
    capacity: usize,
) -> Result<*mut u8, LayoutError>
where
    T: Soa + ?Sized,
{
    let layout = T::buffer_layout(context, capacity)?;
    let prefix_layout = Layout::new::<BufferPrefix<T>>();
    let (_, offset_from_prefix) = prefix_layout.extend(layout)?;

    let buffer = unsafe { ptr.cast::<u8>().add(offset_from_prefix) };
    Ok(buffer)
}
