use core::ptr;

use crate::slice::MultiSlice;

#[allow(clippy::missing_safety_doc)]
#[inline]
pub const fn slice_from_raw_parts<T, U, V>(
    data: *const usize,
    capacity: usize,
) -> *const MultiSlice<T, U, V> {
    let buffer_len = to_buffer_len::<T, U, V>(capacity);
    ptr::slice_from_raw_parts(data, buffer_len) as *const _
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub fn slice_from_raw_parts_mut<T, U, V>(
    data: *mut usize,
    capacity: usize,
) -> *mut MultiSlice<T, U, V> {
    let buffer_len = to_buffer_len::<T, U, V>(capacity);
    ptr::slice_from_raw_parts_mut(data, buffer_len) as *mut _
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
pub(crate) const fn to_buffer_len<T, U, V>(len: usize) -> usize {
    if len == 0 {
        return 0;
    }

    let mut len_in_bytes = size_of::<usize>();
    len_in_bytes = align_up::<T>(len_in_bytes) + (len * size_of::<T>());
    len_in_bytes = align_up::<U>(len_in_bytes) + (len * size_of::<U>());
    len_in_bytes = align_up::<V>(len_in_bytes) + (len * size_of::<V>());

    align_up::<usize>(len_in_bytes) / size_of::<usize>()
}

#[inline]
pub(crate) const fn to_len<T, U, V>(buffer_len: usize) -> usize {
    if buffer_len == 0 || buffer_len == 1 {
        return 0;
    }

    let max_len = {
        let size_of_all = size_of::<T>() + size_of::<U>() + size_of::<V>();
        let len_in_bytes = (buffer_len - 1) * size_of::<usize>();
        len_in_bytes / size_of_all
    };

    let mut len = max_len;
    while {
        // this variable is not inlined (in debug builds) only for better debugging experience
        let to_buffer_len = to_buffer_len::<T, U, V>(len);
        to_buffer_len > buffer_len
    } {
        len -= 1;
    }
    len
}

pub(crate) struct Ptrs<T, U, V> {
    pub start: *mut usize,
    pub t_ptr: *mut T,
    pub u_ptr: *mut U,
    pub v_ptr: *mut V,
    pub end: *mut usize,
}

#[inline]
pub(crate) unsafe fn ptrs<T, U, V>(ptr: *mut usize, len: usize) -> Ptrs<T, U, V> {
    let (start, ptr) = align_cast_then_advance(ptr.cast(), 1);
    let (t_ptr, ptr) = align_cast_then_advance(ptr, len);
    let (u_ptr, ptr) = align_cast_then_advance(ptr, len);
    let (v_ptr, ptr) = align_cast_then_advance(ptr, len);
    let end = ptr_align_up::<usize>(ptr).cast();

    Ptrs {
        start,
        t_ptr,
        u_ptr,
        v_ptr,
        end,
    }
}

#[cfg(test)]
mod tests {
    use super::{to_buffer_len, to_len};

    #[test]
    fn u8_u8_u8_to_buffer_len() {
        let to_buffer_len = to_buffer_len::<u8, u8, u8>;

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), 2);
        assert_eq!(to_buffer_len(2), 2);
        assert_eq!(to_buffer_len(3), 3);
        assert_eq!(to_buffer_len(4), 3);
        assert_eq!(to_buffer_len(5), 3);
        assert_eq!(to_buffer_len(6), 4);
        assert_eq!(to_buffer_len(7), 4);
        assert_eq!(to_buffer_len(8), 4);
        assert_eq!(to_buffer_len(9), 5);
    }

    #[test]
    fn u8_u8_u8_to_len() {
        let to_len = to_len::<u8, u8, u8>;

        assert_eq!(to_len(0), 0);
        assert_eq!(to_len(1), 0);
        assert_eq!(to_len(2), 2);
        assert_eq!(to_len(3), 5);
        assert_eq!(to_len(4), 8);
    }

    #[test]
    fn u16_u16_u16_to_buffer_len() {
        let to_buffer_len = to_buffer_len::<u16, u16, u16>;

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), 2);
        assert_eq!(to_buffer_len(2), 3);
        assert_eq!(to_buffer_len(3), 4);
        assert_eq!(to_buffer_len(4), 4);
        assert_eq!(to_buffer_len(5), 5);
        assert_eq!(to_buffer_len(6), 6);
        assert_eq!(to_buffer_len(7), 7);
        assert_eq!(to_buffer_len(8), 7);
        assert_eq!(to_buffer_len(9), 8);
    }

    #[test]
    fn u16_u16_u16_to_len() {
        let to_len = to_len::<u16, u16, u16>;

        assert_eq!(to_len(0), 0);
        assert_eq!(to_len(1), 0);
        assert_eq!(to_len(2), 1);
        assert_eq!(to_len(3), 2);
        assert_eq!(to_len(4), 4);
        assert_eq!(to_len(5), 5);
        assert_eq!(to_len(6), 6);
        assert_eq!(to_len(7), 8);
    }

    #[test]
    fn u32_u32_u32_to_buffer_len() {
        let to_buffer_len = to_buffer_len::<u32, u32, u32>;

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), 3);
        assert_eq!(to_buffer_len(2), 4);
        assert_eq!(to_buffer_len(3), 6);
        assert_eq!(to_buffer_len(4), 7);
        assert_eq!(to_buffer_len(5), 9);
        assert_eq!(to_buffer_len(6), 10);
        assert_eq!(to_buffer_len(7), 12);
        assert_eq!(to_buffer_len(8), 13);
    }

    #[test]
    fn u32_u32_u32_to_len() {
        let to_len = to_len::<u32, u32, u32>;

        assert_eq!(to_len(0), 0);
        assert_eq!(to_len(1), 0);
        assert_eq!(to_len(2), 0);
        assert_eq!(to_len(3), 1);
        assert_eq!(to_len(4), 2);
        assert_eq!(to_len(5), 2);
        assert_eq!(to_len(6), 3);
        assert_eq!(to_len(7), 4);
        assert_eq!(to_len(8), 4);
        assert_eq!(to_len(9), 5);
        assert_eq!(to_len(10), 6);
        assert_eq!(to_len(11), 6);
        assert_eq!(to_len(12), 7);
        assert_eq!(to_len(13), 8);
    }

    #[test]
    fn u64_u64_u64_to_buffer_len() {
        let to_buffer_len = to_buffer_len::<u64, u64, u64>;

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), 4);
        assert_eq!(to_buffer_len(2), 7);
        assert_eq!(to_buffer_len(3), 10);
        assert_eq!(to_buffer_len(4), 13);
        assert_eq!(to_buffer_len(5), 16);
        assert_eq!(to_buffer_len(6), 19);
        assert_eq!(to_buffer_len(7), 22);
        assert_eq!(to_buffer_len(8), 25);
    }

    #[test]
    fn u64_u64_u64_to_len() {
        let to_len = to_len::<u64, u64, u64>;

        assert_eq!(to_len(0), 0);
        assert_eq!(to_len(1), 0);
        assert_eq!(to_len(2), 0);
        assert_eq!(to_len(3), 0);
        assert_eq!(to_len(4), 1);
        assert_eq!(to_len(5), 1);
        assert_eq!(to_len(6), 1);
        assert_eq!(to_len(7), 2);
        assert_eq!(to_len(8), 2);
        assert_eq!(to_len(9), 2);
        assert_eq!(to_len(10), 3);
    }

    #[test]
    fn u8_u16_u8_to_buffer_len() {
        let to_buffer_len = to_buffer_len::<u8, u16, u8>;

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), 2);
        assert_eq!(to_buffer_len(2), 2);
        assert_eq!(to_buffer_len(3), 3);
        assert_eq!(to_buffer_len(4), 3);
        assert_eq!(to_buffer_len(5), 4);
        assert_eq!(to_buffer_len(6), 4);
        assert_eq!(to_buffer_len(7), 5);
        assert_eq!(to_buffer_len(8), 5);
        assert_eq!(to_buffer_len(9), 6);
        assert_eq!(to_buffer_len(10), 6);
    }

    #[test]
    fn u8_u16_u8_to_len() {
        let to_len = to_len::<u8, u16, u8>;

        assert_eq!(to_len(0), 0);
        assert_eq!(to_len(1), 0);
        assert_eq!(to_len(2), 2);
        assert_eq!(to_len(3), 4);
        assert_eq!(to_len(4), 6);
        assert_eq!(to_len(5), 8);
        assert_eq!(to_len(6), 10);
    }

    #[test]
    fn u16_u8_u16_to_buffer_len() {
        let to_buffer_len = to_buffer_len::<u16, u8, u16>;

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), 2);
        assert_eq!(to_buffer_len(2), 3);
        assert_eq!(to_buffer_len(3), 3);
        assert_eq!(to_buffer_len(4), 4);
        assert_eq!(to_buffer_len(5), 5);
        assert_eq!(to_buffer_len(6), 5);
        assert_eq!(to_buffer_len(7), 6);
        assert_eq!(to_buffer_len(8), 6);
        assert_eq!(to_buffer_len(9), 7);
    }

    #[test]
    fn u16_u8_u16_to_len() {
        let to_len = to_len::<u16, u8, u16>;

        assert_eq!(to_len(0), 0);
        assert_eq!(to_len(1), 0);
        assert_eq!(to_len(2), 1);
        assert_eq!(to_len(3), 3);
        assert_eq!(to_len(4), 4);
        assert_eq!(to_len(5), 6);
        assert_eq!(to_len(6), 8);
        assert_eq!(to_len(7), 9);
    }

    #[test]
    fn u16_u8_u32_to_buffer_len() {
        let to_buffer_len = to_buffer_len::<u16, u8, u32>;

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), 2);
        assert_eq!(to_buffer_len(2), 3);
        assert_eq!(to_buffer_len(3), 4);
        assert_eq!(to_buffer_len(4), 5);
        assert_eq!(to_buffer_len(5), 6);
        assert_eq!(to_buffer_len(6), 7);
        assert_eq!(to_buffer_len(7), 8);
        assert_eq!(to_buffer_len(8), 8);
        assert_eq!(to_buffer_len(9), 9);
    }

    #[test]
    fn u16_u8_u32_to_len() {
        let to_len = to_len::<u16, u8, u32>;

        assert_eq!(to_len(0), 0);
        assert_eq!(to_len(1), 0);
        assert_eq!(to_len(2), 1);
        assert_eq!(to_len(3), 2);
        assert_eq!(to_len(4), 3);
        assert_eq!(to_len(5), 4);
        assert_eq!(to_len(6), 5);
        assert_eq!(to_len(7), 6);
        assert_eq!(to_len(8), 8);
        assert_eq!(to_len(9), 9);
    }

    #[test]
    fn u16_u32_u16_to_buffer_len() {
        let to_buffer_len = to_buffer_len::<u16, u32, u16>;

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), 3);
        assert_eq!(to_buffer_len(2), 3);
        assert_eq!(to_buffer_len(3), 5);
        assert_eq!(to_buffer_len(4), 5);
        assert_eq!(to_buffer_len(5), 7);
    }

    #[test]
    fn u16_u32_u16_to_len() {
        let to_len = to_len::<u16, u32, u16>;

        assert_eq!(to_len(0), 0);
        assert_eq!(to_len(1), 0);
        assert_eq!(to_len(2), 0);
        assert_eq!(to_len(3), 2);
        assert_eq!(to_len(4), 2);
        assert_eq!(to_len(5), 4);
    }
}
