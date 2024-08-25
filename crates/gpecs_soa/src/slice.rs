use core::{
    marker::PhantomData,
    ptr::{self, addr_of, addr_of_mut, NonNull},
    slice,
};

use crate::{multi_vec_ptrs, MultiVecPtrs};

pub struct MultiSlice<T, U, V> {
    phantom: PhantomData<(NonNull<T>, NonNull<U>, NonNull<V>)>,
    len: usize,
    data: [u8],
}

impl<T, U, V> MultiSlice<T, U, V> {
    #[inline]
    pub const fn len(&self) -> usize {
        match self.capacity_in_bytes() {
            0 => 0,
            _ => self.len,
        }
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub const fn capacity_in_bytes(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        let size_of_all = size_of::<T>() + size_of::<U>() + size_of::<V>();
        match self
            .capacity_in_bytes()
            .saturating_sub(size_of::<usize>())
            .checked_div(size_of_all)
        {
            Some(capacity) => capacity,
            None => usize::MAX,
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const T, *const U, *const V) {
        unsafe {
            let len = self.capacity();
            let ptr = addr_of!(*self).cast_mut().cast();

            let MultiVecPtrs {
                start,
                t_ptr,
                u_ptr,
                v_ptr,
                end,
            } = multi_vec_ptrs::<T, U, V>(ptr, len);
            debug_assert_eq!(ptr, start.cast());
            debug_assert_eq!(end.offset_from(ptr) as usize, self.capacity_in_bytes());

            (t_ptr, u_ptr, v_ptr)
        }
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*mut T, *mut U, *mut V) {
        unsafe {
            let len = self.capacity();
            let ptr = addr_of_mut!(*self).cast();

            let MultiVecPtrs {
                start,
                t_ptr,
                u_ptr,
                v_ptr,
                end,
            } = multi_vec_ptrs::<T, U, V>(ptr, len);
            debug_assert_eq!(ptr, start.cast());
            debug_assert_eq!(end.offset_from(ptr) as usize, self.capacity_in_bytes());

            (t_ptr, u_ptr, v_ptr)
        }
    }

    #[inline]
    pub fn as_slices(&self) -> (&[T], &[U], &[V]) {
        let (t_data, u_data, v_data) = self.as_ptrs();
        let len = self.len();

        unsafe {
            let t_slice = slice::from_raw_parts(t_data, len);
            let u_slice = slice::from_raw_parts(u_data, len);
            let v_slice = slice::from_raw_parts(v_data, len);
            (t_slice, u_slice, v_slice)
        }
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [U], &mut [V]) {
        let (t_data, u_data, v_data) = self.as_mut_ptrs();
        let len = self.len();

        unsafe {
            let t_slice = slice::from_raw_parts_mut(t_data, len);
            let u_slice = slice::from_raw_parts_mut(u_data, len);
            let v_slice = slice::from_raw_parts_mut(v_data, len);
            (t_slice, u_slice, v_slice)
        }
    }
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub fn slice_from_raw_parts<T, U, V>(
    data: *const u8,
    capacity: usize,
) -> *const MultiSlice<T, U, V> {
    ptr::slice_from_raw_parts(data, capacity) as _
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub fn slice_from_raw_parts_mut<T, U, V>(
    data: *mut u8,
    capacity: usize,
) -> *mut MultiSlice<T, U, V> {
    ptr::slice_from_raw_parts_mut(data, capacity) as _
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn from_raw_parts<'slice, T, U, V>(
    data: *const u8,
    capacity: usize,
) -> &'slice MultiSlice<T, U, V> {
    &*slice_from_raw_parts(data, capacity)
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn from_raw_parts_mut<'slice, T, U, V>(
    data: *mut u8,
    capacity: usize,
) -> &'slice mut MultiSlice<T, U, V> {
    &mut *slice_from_raw_parts_mut(data, capacity)
}
