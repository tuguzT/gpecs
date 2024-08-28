use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{self, NonNull},
    slice,
};

use crate::ptr::{ptrs, slice_from_raw_parts, slice_from_raw_parts_mut, to_len, Ptrs};

#[repr(transparent)]
pub struct MultiSlice<T, U, V> {
    phantom: PhantomData<(T, U, V)>,
    buffer: [MaybeUninit<usize>],
}

impl<T, U, V> MultiSlice<T, U, V> {
    #[inline]
    pub const fn len(&self) -> usize {
        match self.capacity() {
            0 => 0,
            _ => unsafe { ptr::read(self.as_ptr()) },
        }
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub const fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        let buffer_len = self.buffer_len();
        to_len::<T, U, V>(buffer_len)
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const usize {
        self.buffer.as_ptr().cast()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut usize {
        self.buffer.as_mut_ptr().cast()
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const T, *const U, *const V) {
        let ptr = self.as_ptr().cast_mut();
        let len = self.capacity();

        unsafe {
            let Ptrs {
                start,
                t_ptr,
                u_ptr,
                v_ptr,
                end,
            } = ptrs::<T, U, V>(ptr, len);
            debug_assert_eq!(ptr, start);
            debug_assert_eq!(end.offset_from(start) as usize, self.buffer_len());

            (t_ptr, u_ptr, v_ptr)
        }
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*mut T, *mut U, *mut V) {
        let ptr = self.as_mut_ptr();
        let len = self.capacity();

        unsafe {
            let Ptrs {
                start,
                t_ptr,
                u_ptr,
                v_ptr,
                end,
            } = ptrs::<T, U, V>(ptr, len);
            debug_assert_eq!(ptr, start);
            debug_assert_eq!(end.offset_from(start) as usize, self.buffer_len());

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

impl<T, U, V> Debug for MultiSlice<T, U, V>
where
    T: Debug,
    U: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (t_slice, u_slice, v_slice) = self.as_slices();
        f.debug_struct("MultiSlice")
            .field("t_slice", &t_slice)
            .field("u_slice", &u_slice)
            .field("v_slice", &v_slice)
            .finish()
    }
}

impl<T, U, V> Default for &MultiSlice<T, U, V> {
    fn default() -> Self {
        let data = NonNull::dangling().as_ptr();
        unsafe { from_raw_parts(data, 0) }
    }
}

impl<T, U, V> Default for &mut MultiSlice<T, U, V> {
    fn default() -> Self {
        let data = NonNull::dangling().as_ptr();
        unsafe { from_raw_parts_mut(data, 0) }
    }
}

impl<T, U, V> Drop for MultiSlice<T, U, V> {
    fn drop(&mut self) {
        if self.is_empty() {
            return;
        }

        let (t_data, u_data, v_data) = self.as_mut_ptrs();
        let len = self.len();

        unsafe {
            let t_slice = ptr::slice_from_raw_parts_mut(t_data, len);
            let u_slice = ptr::slice_from_raw_parts_mut(u_data, len);
            let v_slice = ptr::slice_from_raw_parts_mut(v_data, len);

            ptr::drop_in_place(t_slice);
            ptr::drop_in_place(u_slice);
            ptr::drop_in_place(v_slice);
        }
    }
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn from_raw_parts<'slice, T, U, V>(
    data: *const usize,
    capacity: usize,
) -> &'slice MultiSlice<T, U, V> {
    &*slice_from_raw_parts(data, capacity)
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn from_raw_parts_mut<'slice, T, U, V>(
    data: *mut usize,
    capacity: usize,
) -> &'slice mut MultiSlice<T, U, V> {
    &mut *slice_from_raw_parts_mut(data, capacity)
}
