pub use gpecs_soa_core::ptr::*;

use crate::{slice::SoaSlice, traits::AllocSoaTrusted};

#[inline]
pub unsafe fn slice_from_raw_parts<T>(
    data: *const u8,
    len: usize,
    capacity: usize,
) -> *const SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    unsafe { SoaSlice::ptr_from_raw_parts(data, len, capacity) }
}

#[inline]
pub unsafe fn slice_from_raw_parts_mut<T>(
    data: *mut u8,
    len: usize,
    capacity: usize,
) -> *mut SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    unsafe { SoaSlice::ptr_from_raw_parts_mut(data, len, capacity) }
}

pub trait SoaSlicePtr<T>: Copy + private::Sealed
where
    T: AllocSoaTrusted + ?Sized,
{
    fn as_ptr(self) -> *const u8;

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
    fn as_ptr(self) -> *const u8 {
        SoaSlice::ptr_as_ptr(self)
    }

    #[inline]
    unsafe fn len(self) -> usize {
        unsafe { SoaSlice::ptr_len(self) }
    }

    #[inline]
    unsafe fn capacity(self) -> usize {
        unsafe { SoaSlice::ptr_capacity(self) }
    }
}

impl<T> SoaSlicePtr<T> for *mut SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    fn as_ptr(self) -> *const u8 {
        self.cast_const().as_ptr()
    }

    #[inline]
    unsafe fn len(self) -> usize {
        unsafe { self.cast_const().len() }
    }

    #[inline]
    unsafe fn capacity(self) -> usize {
        unsafe { self.cast_const().capacity() }
    }
}

pub trait SoaSlicePtrMut<T>: SoaSlicePtr<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    fn as_mut_ptr(self) -> *mut u8;
}

impl<T> SoaSlicePtrMut<T> for *mut SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    fn as_mut_ptr(self) -> *mut u8 {
        SoaSlice::ptr_as_mut_ptr(self)
    }
}

mod private {
    use crate::{slice::SoaSlice, traits::AllocSoaTrusted};

    pub trait Sealed {}

    impl<T> Sealed for *const SoaSlice<T> where T: AllocSoaTrusted + ?Sized {}
    impl<T> Sealed for *mut SoaSlice<T> where T: AllocSoaTrusted + ?Sized {}
}
