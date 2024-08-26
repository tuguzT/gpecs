use core::ptr;

use crate::slice::MultiSlice;

#[allow(clippy::missing_safety_doc)]
#[inline]
pub fn slice_from_raw_parts<T, U, V>(
    data: *const usize,
    capacity: usize,
) -> *const MultiSlice<T, U, V> {
    let len = match capacity {
        0 => 0,
        _ => multi_vec_buffer_len::<T, U, V>(capacity),
    };
    ptr::slice_from_raw_parts(data, len) as *const _
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub fn slice_from_raw_parts_mut<T, U, V>(
    data: *mut usize,
    capacity: usize,
) -> *mut MultiSlice<T, U, V> {
    let len = match capacity {
        0 => 0,
        _ => multi_vec_buffer_len::<T, U, V>(capacity),
    };
    ptr::slice_from_raw_parts_mut(data, len) as *mut _
}

#[inline(always)]
unsafe fn ptr_align_up<T>(ptr: *mut u8) -> *mut u8 {
    let align = align_of::<T>();
    let offset = ptr.align_offset(align);

    ptr.add(offset)
}

#[inline(always)]
pub(crate) unsafe fn align_cast_then_advance<T>(ptr: *mut u8, len: usize) -> (*mut T, *mut u8) {
    let ptr = ptr_align_up::<T>(ptr);

    let t_ptr = ptr.cast::<T>();
    debug_assert!(t_ptr.is_aligned());

    let ptr = t_ptr.add(len).cast();
    (t_ptr, ptr)
}

#[inline(always)]
const fn align_up<T>(addr: usize) -> usize {
    let align = align_of::<T>();
    (addr + align - 1) & !(align - 1)
}

#[inline]
pub(crate) const fn multi_vec_buffer_len<T, U, V>(len: usize) -> usize {
    let mut len_in_bytes = size_of::<usize>();
    len_in_bytes = align_up::<T>(len_in_bytes) + (len * size_of::<T>());
    len_in_bytes = align_up::<U>(len_in_bytes) + (len * size_of::<U>());
    len_in_bytes = align_up::<V>(len_in_bytes) + (len * size_of::<V>());

    align_up::<usize>(len_in_bytes) / size_of::<usize>()
}

pub(crate) struct MultiVecPtrs<T, U, V> {
    pub start: *mut usize,
    pub t_ptr: *mut T,
    pub u_ptr: *mut U,
    pub v_ptr: *mut V,
    pub end: *mut u8,
}

#[inline]
pub(crate) unsafe fn multi_vec_ptrs<T, U, V>(ptr: *mut u8, len: usize) -> MultiVecPtrs<T, U, V> {
    let (start, ptr) = align_cast_then_advance(ptr, 1);
    let (t_ptr, ptr) = align_cast_then_advance(ptr, len);
    let (u_ptr, ptr) = align_cast_then_advance(ptr, len);
    let (v_ptr, end) = align_cast_then_advance(ptr, len);

    MultiVecPtrs {
        start,
        t_ptr,
        u_ptr,
        v_ptr,
        end,
    }
}
