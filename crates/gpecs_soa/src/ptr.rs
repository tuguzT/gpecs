use core::ptr::{self, NonNull};

use crate::slice::MultiSlice;

#[allow(clippy::missing_safety_doc)]
#[inline]
pub const fn slice_from_raw_parts<T, U, V>(
    data: *const u8,
    capacity: usize,
) -> *const MultiSlice<T, U, V> {
    let len_in_bytes = to_len_in_bytes::<T, U, V>(capacity);
    slice_from_len_in_bytes(data, len_in_bytes)
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub fn slice_from_raw_parts_mut<T, U, V>(
    data: *mut u8,
    capacity: usize,
) -> *mut MultiSlice<T, U, V> {
    let len_in_bytes = to_len_in_bytes::<T, U, V>(capacity);
    slice_from_len_in_bytes_mut(data, len_in_bytes)
}

#[inline(always)]
pub(crate) const fn slice_from_len_in_bytes<T, U, V>(
    data: *const u8,
    len_in_bytes: usize,
) -> *const MultiSlice<T, U, V> {
    ptr::slice_from_raw_parts(data, len_in_bytes) as *const _
}

#[inline(always)]
pub(crate) fn slice_from_len_in_bytes_mut<T, U, V>(
    data: *mut u8,
    len_in_bytes: usize,
) -> *mut MultiSlice<T, U, V> {
    ptr::slice_from_raw_parts_mut(data, len_in_bytes) as *mut _
}

#[inline(always)]
pub(crate) unsafe fn ptr_align_up<T>(ptr: *mut u8) -> *mut u8 {
    let align = align_of::<T>();
    let offset = ptr.align_offset(align);

    unsafe { ptr.add(offset) }
}

#[inline(always)]
pub(crate) unsafe fn align_cast_then_advance<T>(ptr: *mut u8, len: usize) -> (*mut T, *mut u8) {
    let ptr = unsafe { ptr_align_up::<T>(ptr) };

    let t_ptr = ptr.cast::<T>();
    debug_assert!(t_ptr.is_aligned());

    let ptr = unsafe { t_ptr.add(len).cast() };
    (t_ptr, ptr)
}

#[inline(always)]
pub(crate) const fn align_up<T>(addr: usize) -> usize {
    let align = align_of::<T>();
    (addr + align - 1) & !(align - 1)
}

#[inline]
pub(crate) const fn to_len_in_bytes<T, U, V>(len: usize) -> usize {
    if min_size_of::<T, U, V>() == 0 || len == 0 {
        return 0;
    }

    let mut len_in_bytes = size_of::<usize>();
    len_in_bytes = align_up::<T>(len_in_bytes) + (len * size_of::<T>());
    len_in_bytes = align_up::<U>(len_in_bytes) + (len * size_of::<U>());
    len_in_bytes = align_up::<V>(len_in_bytes) + (len * size_of::<V>());

    len_in_bytes
}

#[inline]
pub(crate) const fn min_size_of<T, U, V>() -> usize {
    size_of::<T>() + size_of::<U>() + size_of::<V>()
}

#[inline]
pub(crate) const fn align_of_buffer<T, U, V>() -> usize {
    align_of::<(usize, (T, U, V))>()
}

#[inline]
pub(crate) const fn to_len<T, U, V>(len_in_bytes: usize) -> usize {
    if min_size_of::<T, U, V>() == 0 || len_in_bytes < size_of::<usize>() {
        return 0;
    }

    let max_len = (len_in_bytes - size_of::<usize>()) / min_size_of::<T, U, V>();

    let mut len = max_len;
    while {
        // this variable is not inlined (in debug builds) only for better debugging experience
        let to_len_in_bytes = to_len_in_bytes::<T, U, V>(len);
        to_len_in_bytes > len_in_bytes
    } {
        len -= 1;
    }
    len
}

#[inline]
pub(crate) unsafe fn ptrs<T, U, V>(ptr: *mut u8, len: usize) -> (*mut T, *mut U, *mut V) {
    if min_size_of::<T, U, V>() == 0 {
        return (
            NonNull::dangling().as_ptr(),
            NonNull::dangling().as_ptr(),
            NonNull::dangling().as_ptr(),
        );
    }

    let (_, ptr) = unsafe { align_cast_then_advance::<usize>(ptr.cast(), 1) };
    let (t_ptr, ptr) = unsafe { align_cast_then_advance(ptr, len) };
    let (u_ptr, ptr) = unsafe { align_cast_then_advance(ptr, len) };
    let (v_ptr, _) = unsafe { align_cast_then_advance(ptr, len) };

    (t_ptr, u_ptr, v_ptr)
}

#[cfg(test)]
#[allow(clippy::identity_op)]
#[rustfmt::skip::macros(assert_eq)]
mod tests {
    use super::{align_up, to_len, to_len_in_bytes};

    #[test]
    fn u8_u8_u8_to_buffer_len() {
        let to_buffer_len = to_len_in_bytes::<u8, u8, u8>;
        let u8 = size_of::<u8>();

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), size_of::<usize>() + 3 * u8 * 1);
        assert_eq!(to_buffer_len(2), size_of::<usize>() + 3 * u8 * 2);
        assert_eq!(to_buffer_len(3), size_of::<usize>() + 3 * u8 * 3);
        assert_eq!(to_buffer_len(4), size_of::<usize>() + 3 * u8 * 4);
        assert_eq!(to_buffer_len(5), size_of::<usize>() + 3 * u8 * 5);
        assert_eq!(to_buffer_len(6), size_of::<usize>() + 3 * u8 * 6);
        assert_eq!(to_buffer_len(7), size_of::<usize>() + 3 * u8 * 7);
        assert_eq!(to_buffer_len(8), size_of::<usize>() + 3 * u8 * 8);
        assert_eq!(to_buffer_len(9), size_of::<usize>() + 3 * u8 * 9);
    }

    #[test]
    fn u8_u8_u8_to_len() {
        let to_len = to_len::<u8, u8, u8>;
        let u8 = size_of::<u8>();

        for len_in_bytes in 0..(size_of::<usize>() + 3 * u8 * 1) {
            assert_eq!(to_len(len_in_bytes), 0);
        }

        assert_eq!(1, to_len(size_of::<usize>() + 3 * u8 * 1));
        assert_eq!(1, to_len(size_of::<usize>() + 3 * u8 * 1 + 1));
        assert_eq!(1, to_len(size_of::<usize>() + 3 * u8 * 2 - 1));

        assert_eq!(2, to_len(size_of::<usize>() + 3 * u8 * 2));
        assert_eq!(2, to_len(size_of::<usize>() + 3 * u8 * 2 + 1));
        assert_eq!(2, to_len(size_of::<usize>() + 3 * u8 * 3 - 1));

        assert_eq!(3, to_len(size_of::<usize>() + 3 * u8 * 3));
        assert_eq!(3, to_len(size_of::<usize>() + 3 * u8 * 3 + 1));
        assert_eq!(3, to_len(size_of::<usize>() + 3 * u8 * 4 - 1));

        assert_eq!(4, to_len(size_of::<usize>() + 3 * u8 * 4));
    }

    #[test]
    fn u16_u16_u16_to_buffer_len() {
        let to_buffer_len = to_len_in_bytes::<u16, u16, u16>;
        let u16 = size_of::<u16>();

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), size_of::<usize>() + 3 * u16 * 1);
        assert_eq!(to_buffer_len(2), size_of::<usize>() + 3 * u16 * 2);
        assert_eq!(to_buffer_len(3), size_of::<usize>() + 3 * u16 * 3);
        assert_eq!(to_buffer_len(4), size_of::<usize>() + 3 * u16 * 4);
        assert_eq!(to_buffer_len(5), size_of::<usize>() + 3 * u16 * 5);
        assert_eq!(to_buffer_len(6), size_of::<usize>() + 3 * u16 * 6);
        assert_eq!(to_buffer_len(7), size_of::<usize>() + 3 * u16 * 7);
        assert_eq!(to_buffer_len(8), size_of::<usize>() + 3 * u16 * 8);
        assert_eq!(to_buffer_len(9), size_of::<usize>() + 3 * u16 * 9);
    }

    #[test]
    fn u16_u16_u16_to_len() {
        let to_len = to_len::<u16, u16, u16>;
        let u16 = size_of::<u16>();

        for len_in_bytes in 0..(size_of::<usize>() + 3 * u16 * 1) {
            assert_eq!(to_len(len_in_bytes), 0);
        }

        assert_eq!(1, to_len(size_of::<usize>() + 3 * u16 * 1));
        assert_eq!(1, to_len(size_of::<usize>() + 3 * u16 * 1 + 1));
        assert_eq!(1, to_len(size_of::<usize>() + 3 * u16 * 2 - 1));

        assert_eq!(2, to_len(size_of::<usize>() + 3 * u16 * 2));
        assert_eq!(2, to_len(size_of::<usize>() + 3 * u16 * 2 + 1));
        assert_eq!(2, to_len(size_of::<usize>() + 3 * u16 * 3 - 1));

        assert_eq!(3, to_len(size_of::<usize>() + 3 * u16 * 3));
    }

    #[test]
    fn u32_u32_u32_to_buffer_len() {
        let to_buffer_len = to_len_in_bytes::<u32, u32, u32>;
        let u32 = size_of::<u32>();
        let aligned_len = align_up::<u32>(size_of::<usize>());

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), aligned_len + 3 * u32 * 1);
        assert_eq!(to_buffer_len(2), aligned_len + 3 * u32 * 2);
        assert_eq!(to_buffer_len(3), aligned_len + 3 * u32 * 3);
        assert_eq!(to_buffer_len(4), aligned_len + 3 * u32 * 4);
        assert_eq!(to_buffer_len(5), aligned_len + 3 * u32 * 5);
        assert_eq!(to_buffer_len(6), aligned_len + 3 * u32 * 6);
        assert_eq!(to_buffer_len(7), aligned_len + 3 * u32 * 7);
        assert_eq!(to_buffer_len(8), aligned_len + 3 * u32 * 8);
    }

    #[test]
    fn u32_u32_u32_to_len() {
        let to_len = to_len::<u32, u32, u32>;
        let u32 = size_of::<u32>();
        let aligned_len = align_up::<u32>(size_of::<usize>());

        for len_in_bytes in 0..(aligned_len + 3 * u32 * 1) {
            assert_eq!(to_len(len_in_bytes), 0);
        }

        assert_eq!(1, to_len(aligned_len + 3 * u32 * 1));
        assert_eq!(1, to_len(aligned_len + 3 * u32 * 1 + 1));
        assert_eq!(1, to_len(aligned_len + 3 * u32 * 2 - 1));

        assert_eq!(2, to_len(aligned_len + 3 * u32 * 2));
        assert_eq!(2, to_len(aligned_len + 3 * u32 * 2 + 1));
        assert_eq!(2, to_len(aligned_len + 3 * u32 * 3 - 1));

        assert_eq!(3, to_len(aligned_len + 3 * u32 * 3));
    }

    #[test]
    fn u64_u64_u64_to_buffer_len() {
        let to_buffer_len = to_len_in_bytes::<u64, u64, u64>;
        let u64 = size_of::<u64>();
        let aligned_len = align_up::<u64>(size_of::<usize>());

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), aligned_len + 3 * u64 * 1);
        assert_eq!(to_buffer_len(2), aligned_len + 3 * u64 * 2);
        assert_eq!(to_buffer_len(3), aligned_len + 3 * u64 * 3);
        assert_eq!(to_buffer_len(4), aligned_len + 3 * u64 * 4);
        assert_eq!(to_buffer_len(5), aligned_len + 3 * u64 * 5);
        assert_eq!(to_buffer_len(6), aligned_len + 3 * u64 * 6);
        assert_eq!(to_buffer_len(7), aligned_len + 3 * u64 * 7);
        assert_eq!(to_buffer_len(8), aligned_len + 3 * u64 * 8);
    }

    #[test]
    fn u64_u64_u64_to_len() {
        let to_len = to_len::<u64, u64, u64>;
        let u64 = size_of::<u64>();
        let aligned_len = align_up::<u64>(size_of::<usize>());

        for len_in_bytes in 0..(aligned_len + 3 * u64 * 1) {
            assert_eq!(to_len(len_in_bytes), 0);
        }

        assert_eq!(1, to_len(aligned_len + 3 * u64 * 1));
        assert_eq!(1, to_len(aligned_len + 3 * u64 * 1 + 1));
        assert_eq!(1, to_len(aligned_len + 3 * u64 * 2 - 1));

        assert_eq!(2, to_len(aligned_len + 3 * u64 * 2));
        assert_eq!(2, to_len(aligned_len + 3 * u64 * 2 + 1));
        assert_eq!(2, to_len(aligned_len + 3 * u64 * 3 - 1));

        assert_eq!(3, to_len(aligned_len + 3 * u64 * 3));
    }

    #[test]
    fn u8_u16_u8_to_buffer_len() {
        let to_buffer_len = to_len_in_bytes::<u8, u16, u8>;
        let u8 = size_of::<u8>();
        let u16 = size_of::<u16>();

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), size_of::<usize>() + (u8 * 1) + 1 + (u16 + u8) * 1);
        assert_eq!(to_buffer_len(2), size_of::<usize>() + (u8 * 2) + 0 + (u16 + u8) * 2);
        assert_eq!(to_buffer_len(3), size_of::<usize>() + (u8 * 3) + 1 + (u16 + u8) * 3);
        assert_eq!(to_buffer_len(4), size_of::<usize>() + (u8 * 4) + 0 + (u16 + u8) * 4);
        assert_eq!(to_buffer_len(5), size_of::<usize>() + (u8 * 5) + 1 + (u16 + u8) * 5);
        assert_eq!(to_buffer_len(6), size_of::<usize>() + (u8 * 6) + 0 + (u16 + u8) * 6);
    }

    #[test]
    fn u8_u16_u8_to_len() {
        let to_len = to_len::<u8, u16, u8>;
        let u8 = size_of::<u8>();
        let u16 = size_of::<u16>();

        for len_in_bytes in 0..(size_of::<usize>() + (u8 * 1) + 1 + (u16 + u8) * 1) {
            assert_eq!(to_len(len_in_bytes), 0);
        }

        assert_eq!(1, to_len(size_of::<usize>() + (u8 * 1) + 1 + (u16 + u8) * 1));
        assert_eq!(1, to_len(size_of::<usize>() + (u8 * 1) + 1 + (u16 + u8) * 1 + 1));
        assert_eq!(1, to_len(size_of::<usize>() + (u8 * 2) + 0 + (u16 + u8) * 2 - 1));

        assert_eq!(2, to_len(size_of::<usize>() + (u8 * 2) + 0 + (u16 + u8) * 2));
        assert_eq!(2, to_len(size_of::<usize>() + (u8 * 2) + 0 + (u16 + u8) * 2 + 1));
        assert_eq!(2, to_len(size_of::<usize>() + (u8 * 3) + 1 + (u16 + u8) * 3 - 1));

        assert_eq!(3, to_len(size_of::<usize>() + (u8 * 3) + 1 + (u16 + u8) * 3));
    }

    #[test]
    fn u16_u8_u16_to_buffer_len() {
        let to_buffer_len = to_len_in_bytes::<u16, u8, u16>;
        let u8 = size_of::<u8>();
        let u16 = size_of::<u16>();

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), size_of::<usize>() + (u16 + u8) * 1 + 1 + (u16 * 1));
        assert_eq!(to_buffer_len(2), size_of::<usize>() + (u16 + u8) * 2 + 0 + (u16 * 2));
        assert_eq!(to_buffer_len(3), size_of::<usize>() + (u16 + u8) * 3 + 1 + (u16 * 3));
        assert_eq!(to_buffer_len(4), size_of::<usize>() + (u16 + u8) * 4 + 0 + (u16 * 4));
        assert_eq!(to_buffer_len(5), size_of::<usize>() + (u16 + u8) * 5 + 1 + (u16 * 5));
        assert_eq!(to_buffer_len(6), size_of::<usize>() + (u16 + u8) * 6 + 0 + (u16 * 6));
        assert_eq!(to_buffer_len(7), size_of::<usize>() + (u16 + u8) * 7 + 1 + (u16 * 7));
        assert_eq!(to_buffer_len(8), size_of::<usize>() + (u16 + u8) * 8 + 0 + (u16 * 8));
    }

    #[test]
    fn u16_u8_u16_to_len() {
        let to_len = to_len::<u16, u8, u16>;
        let u8 = size_of::<u8>();
        let u16 = size_of::<u16>();

        for len_in_bytes in 0..(size_of::<usize>() + (u16 + u8) * 1 + 1 + (u16 * 1)) {
            assert_eq!(to_len(len_in_bytes), 0);
        }

        assert_eq!(1, to_len(size_of::<usize>() + (u16 + u8) * 1 + 1 + (u16 * 1)));
        assert_eq!(1, to_len(size_of::<usize>() + (u16 + u8) * 1 + 1 + (u16 * 1) + 1));
        assert_eq!(1, to_len(size_of::<usize>() + (u16 + u8) * 2 + 0 + (u16 * 2) - 1));

        assert_eq!(2, to_len(size_of::<usize>() + (u16 + u8) * 2 + 0 + (u16 * 2)));
        assert_eq!(2, to_len(size_of::<usize>() + (u16 + u8) * 2 + 0 + (u16 * 2) + 1));
        assert_eq!(2, to_len(size_of::<usize>() + (u16 + u8) * 3 + 1 + (u16 * 3) - 1));

        assert_eq!(3, to_len(size_of::<usize>() + (u16 + u8) * 3 + 1 + (u16 * 3)));
    }

    #[test]
    fn u16_u8_u32_to_buffer_len() {
        let to_buffer_len = to_len_in_bytes::<u16, u8, u32>;
        let u8 = size_of::<u8>();
        let u16 = size_of::<u16>();
        let u32 = size_of::<u32>();

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), size_of::<usize>() + (u16 + u8) * 1 + 1 + (u32 * 1));
        assert_eq!(to_buffer_len(2), size_of::<usize>() + (u16 + u8) * 2 + 2 + (u32 * 2));
        assert_eq!(to_buffer_len(3), size_of::<usize>() + (u16 + u8) * 3 + 3 + (u32 * 3));
        assert_eq!(to_buffer_len(4), size_of::<usize>() + (u16 + u8) * 4 + 0 + (u32 * 4));
        assert_eq!(to_buffer_len(5), size_of::<usize>() + (u16 + u8) * 5 + 1 + (u32 * 5));
    }

    #[test]
    fn u16_u8_u32_to_len() {
        let to_len = to_len::<u16, u8, u32>;
        let u8 = size_of::<u8>();
        let u16 = size_of::<u16>();
        let u32 = size_of::<u32>();

        for len_in_bytes in 0..(size_of::<usize>() + (u16 + u8) * 1 + 1 + (u32 * 1)) {
            assert_eq!(to_len(len_in_bytes), 0);
        }

        assert_eq!(1, to_len(size_of::<usize>() + (u16 + u8) * 1 + 1 + (u32 * 1)));
        assert_eq!(1, to_len(size_of::<usize>() + (u16 + u8) * 1 + 1 + (u32 * 1) + 1));
        assert_eq!(1, to_len(size_of::<usize>() + (u16 + u8) * 2 + 2 + (u32 * 2) - 1));

        assert_eq!(2, to_len(size_of::<usize>() + (u16 + u8) * 2 + 2 + (u32 * 2)));
        assert_eq!(2, to_len(size_of::<usize>() + (u16 + u8) * 2 + 2 + (u32 * 2) + 1));
        assert_eq!(2, to_len(size_of::<usize>() + (u16 + u8) * 3 + 3 + (u32 * 3) - 1));

        assert_eq!(3, to_len(size_of::<usize>() + (u16 + u8) * 3 + 3 + (u32 * 3)));
    }

    #[test]
    fn u16_u32_u16_to_buffer_len() {
        let to_buffer_len = to_len_in_bytes::<u16, u32, u16>;
        let u16 = size_of::<u16>();
        let u32 = size_of::<u32>();

        assert_eq!(to_buffer_len(0), 0);
        assert_eq!(to_buffer_len(1), size_of::<usize>() + (u16 + u32) * 1 + 2 + (u16 * 1));
        assert_eq!(to_buffer_len(2), size_of::<usize>() + (u16 + u32) * 2 + 0 + (u16 * 2));
        assert_eq!(to_buffer_len(3), size_of::<usize>() + (u16 + u32) * 3 + 2 + (u16 * 3));
        assert_eq!(to_buffer_len(4), size_of::<usize>() + (u16 + u32) * 4 + 0 + (u16 * 4));
        assert_eq!(to_buffer_len(5), size_of::<usize>() + (u16 + u32) * 5 + 2 + (u16 * 5));
        assert_eq!(to_buffer_len(6), size_of::<usize>() + (u16 + u32) * 6 + 0 + (u16 * 6));
        assert_eq!(to_buffer_len(7), size_of::<usize>() + (u16 + u32) * 7 + 2 + (u16 * 7));
        assert_eq!(to_buffer_len(8), size_of::<usize>() + (u16 + u32) * 8 + 0 + (u16 * 8));
    }

    #[test]
    fn u16_u32_u16_to_len() {
        let to_len = to_len::<u16, u32, u16>;
        let u16 = size_of::<u16>();
        let u32 = size_of::<u32>();

        for len_in_bytes in 0..(size_of::<usize>() + (u16 + u32) * 1 + 2 + (u16 * 1)) {
            assert_eq!(to_len(len_in_bytes), 0);
        }

        assert_eq!(1, to_len(size_of::<usize>() + (u16 + u32) * 1 + 2 + (u16 * 1)));
        assert_eq!(1, to_len(size_of::<usize>() + (u16 + u32) * 1 + 2 + (u16 * 1) + 1));
        assert_eq!(1, to_len(size_of::<usize>() + (u16 + u32) * 2 + 0 + (u16 * 2) - 1));

        assert_eq!(2, to_len(size_of::<usize>() + (u16 + u32) * 2 + 0 + (u16 * 2)));
        assert_eq!(2, to_len(size_of::<usize>() + (u16 + u32) * 2 + 0 + (u16 * 2) + 1));
        assert_eq!(2, to_len(size_of::<usize>() + (u16 + u32) * 3 + 2 + (u16 * 3) - 1));

        assert_eq!(3, to_len(size_of::<usize>() + (u16 + u32) * 3 + 2 + (u16 * 3)));
    }
}
