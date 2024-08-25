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
        _ => {
            let capacity_in_bytes = multi_vec_len_in_bytes::<T, U, V>(capacity);
            capacity_in_bytes / size_of::<usize>() + 1
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
    let len = match capacity {
        0 => 0,
        _ => {
            let capacity_in_bytes = multi_vec_len_in_bytes::<T, U, V>(capacity);
            capacity_in_bytes / size_of::<usize>() + 1
        }
    };
    ptr::slice_from_raw_parts_mut(data, len) as *mut _
}

#[inline(always)]
pub(crate) unsafe fn align_cast_then_advance<T>(ptr: *mut u8, len: usize) -> (*mut T, *mut u8) {
    let offset = ptr.align_offset(align_of::<T>());
    let ptr = ptr.add(offset);

    let t_ptr = ptr.cast::<T>();
    debug_assert!(t_ptr.is_aligned());

    let ptr = ptr.add(len * size_of::<T>());
    (t_ptr, ptr)
}

#[inline(always)]
const fn align_up<T>(addr: usize) -> usize {
    let align = align_of::<T>();
    (addr + align - 1) & !(align - 1)
}

#[inline]
pub(crate) const fn multi_vec_len_in_bytes<T, U, V>(len: usize) -> usize {
    let mut len_in_bytes = size_of::<usize>();

    len_in_bytes = align_up::<T>(len_in_bytes) + (len * size_of::<T>());
    len_in_bytes = align_up::<U>(len_in_bytes) + (len * size_of::<U>());
    len_in_bytes = align_up::<V>(len_in_bytes) + (len * size_of::<V>());

    len_in_bytes
}

pub(crate) struct MultiVecPtrs<T, U, V> {
    pub(crate) start: *mut usize,
    pub(crate) t_ptr: *mut T,
    pub(crate) u_ptr: *mut U,
    pub(crate) v_ptr: *mut V,
    pub(crate) end: *mut u8,
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
