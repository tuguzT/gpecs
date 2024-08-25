//! Nothing too special for now...

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod slice;
pub mod vec;

#[inline]
unsafe fn align_cast_then_advance<T>(ptr: *mut u8, len: usize) -> (*mut T, *mut u8) {
    let offset = ptr.align_offset(align_of::<T>());
    let ptr = ptr.add(offset);

    let t_ptr = ptr.cast::<T>();
    debug_assert!(t_ptr.is_aligned());

    let ptr = ptr.add(len * size_of::<T>());
    (t_ptr, ptr)
}

#[inline]
fn multi_vec_len_in_bytes<T, U, V>(len: usize) -> usize {
    let ptr = core::ptr::null_mut();
    unsafe {
        let MultiVecPtrs { start, end, .. } = multi_vec_ptrs::<T, U, V>(ptr, len);
        end.offset_from(start.cast()) as usize
    }
}

struct MultiVecPtrs<T, U, V> {
    start: *mut usize,
    t_ptr: *mut T,
    u_ptr: *mut U,
    v_ptr: *mut V,
    end: *mut u8,
}

#[inline]
unsafe fn multi_vec_ptrs<T, U, V>(ptr: *mut u8, len: usize) -> MultiVecPtrs<T, U, V> {
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
