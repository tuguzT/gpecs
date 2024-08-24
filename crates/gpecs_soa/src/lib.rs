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
    let start = core::ptr::null_mut();
    let end = start;

    unsafe {
        let (_, end) = align_cast_then_advance::<T>(end, len);
        let (_, end) = align_cast_then_advance::<U>(end, len);
        let (_, end) = align_cast_then_advance::<V>(end, len);
        end.offset_from(start) as usize
    }
}
