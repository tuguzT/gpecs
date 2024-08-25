use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr::{self, addr_of, addr_of_mut, NonNull},
    slice,
};

use crate::{multi_vec_len_in_bytes, multi_vec_ptrs, MultiVecPtrs};

pub struct MultiSlice<T, U, V> {
    phantom: PhantomData<(NonNull<T>, NonNull<U>, NonNull<V>)>,
    len: usize,
    data: [usize],
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
        self.data.len() * size_of::<usize>()
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        let size_of_all = size_of::<T>() + size_of::<U>() + size_of::<V>();
        match self.capacity_in_bytes().checked_div(size_of_all) {
            Some(capacity) => capacity,
            None => usize::MAX,
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const usize {
        self.data.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut usize {
        self.data.as_mut_ptr()
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const T, *const U, *const V) {
        unsafe {
            let len_in_bytes = self.capacity() * size_of::<T>();
            let ptr = addr_of!(*self).cast_mut().cast();

            let MultiVecPtrs {
                start,
                t_ptr,
                u_ptr,
                v_ptr,
                end,
            } = multi_vec_ptrs::<T, U, V>(ptr, len_in_bytes);
            debug_assert_eq!(ptr, start.cast());
            debug_assert_eq!(
                end.offset_from(ptr) as usize,
                self.capacity_in_bytes() + size_of::<usize>(),
            );

            (t_ptr, u_ptr, v_ptr)
        }
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*mut T, *mut U, *mut V) {
        unsafe {
            let len_in_bytes = self.capacity() * size_of::<T>();
            let ptr = addr_of_mut!(*self).cast();

            let MultiVecPtrs {
                start,
                t_ptr,
                u_ptr,
                v_ptr,
                end,
            } = multi_vec_ptrs::<T, U, V>(ptr, len_in_bytes);
            debug_assert_eq!(ptr, start.cast());
            debug_assert_eq!(
                end.offset_from(ptr) as usize,
                self.capacity_in_bytes() + size_of::<usize>(),
            );

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
            .field("capacity", &self.capacity())
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

#[allow(clippy::missing_safety_doc)]
#[inline]
pub fn slice_from_raw_parts<T, U, V>(
    data: *const usize,
    capacity: usize,
) -> *const MultiSlice<T, U, V> {
    let (data, len) = match capacity {
        0 => ([0].as_ptr(), 0),
        _ => {
            let capacity_in_bytes = multi_vec_len_in_bytes::<T, U, V>(capacity);
            let len = capacity_in_bytes.saturating_sub(size_of::<usize>()) / size_of::<usize>();
            (data, len)
        }
    };
    ptr::slice_from_raw_parts(data, len) as *const _
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub fn slice_from_raw_parts_mut<T, U, V>(
    data: *mut usize,
    capacity: usize,
) -> *mut MultiSlice<T, U, V> {
    let (data, len) = match capacity {
        0 => ([0].as_mut_ptr(), 0),
        _ => {
            let capacity_in_bytes = multi_vec_len_in_bytes::<T, U, V>(capacity);
            let len = capacity_in_bytes.saturating_sub(size_of::<usize>()) / size_of::<usize>();
            (data, len)
        }
    };
    ptr::slice_from_raw_parts(data, len) as *mut _
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
