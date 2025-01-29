use core::{
    alloc::{Layout, LayoutError},
    ptr,
};

use crate::{slice::SoaSlice, soa::Soa};

#[allow(clippy::missing_safety_doc)]
#[inline]
pub fn slice_from_raw_parts<T>(data: *const u8, capacity: usize) -> *const SoaSlice<T>
where
    T: Soa,
{
    let len_in_bytes = buffer_layout::<T>(capacity)
        .expect("layout size should not exceed `isize::MAX`")
        .size();
    slice_from_len_in_bytes(data, len_in_bytes)
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub fn slice_from_raw_parts_mut<T>(data: *mut u8, capacity: usize) -> *mut SoaSlice<T>
where
    T: Soa,
{
    let len_in_bytes = buffer_layout::<T>(capacity)
        .expect("layout size should not exceed `isize::MAX`")
        .size();
    slice_from_len_in_bytes_mut(data, len_in_bytes)
}

#[inline(always)]
pub(crate) fn slice_from_len_in_bytes<T>(data: *const u8, len_in_bytes: usize) -> *const SoaSlice<T>
where
    T: Soa,
{
    ptr::slice_from_raw_parts(data, len_in_bytes) as *const _
}

#[inline(always)]
pub(crate) fn slice_from_len_in_bytes_mut<T>(data: *mut u8, len_in_bytes: usize) -> *mut SoaSlice<T>
where
    T: Soa,
{
    ptr::slice_from_raw_parts_mut(data, len_in_bytes) as *mut _
}

#[repr(transparent)]
pub(crate) struct BufferAlign<T>
where
    T: Soa,
{
    align: [(usize, T); 0],
}

#[inline]
fn buffer_layout_unaligned<T>(len: usize) -> Result<Layout, LayoutError>
where
    T: Soa,
{
    if T::min_size_of_components() == 0 || len == 0 {
        return Ok(Layout::new::<()>());
    }

    let initial = Layout::new::<usize>();
    let (layout, _) = T::buffer_layout_unaligned(initial, len)?;
    Ok(layout)
}

#[inline]
pub(crate) fn buffer_layout<T>(len: usize) -> Result<Layout, LayoutError>
where
    T: Soa,
{
    let unaligned = buffer_layout_unaligned::<T>(len)?;
    Ok(unaligned.pad_to_align())
}

#[inline]
fn unaligned_to_len<T>(len_in_bytes: usize) -> usize
where
    T: Soa,
{
    if T::min_size_of_components() == 0 || len_in_bytes < size_of::<usize>() {
        return 0;
    }

    let max_len = (len_in_bytes - size_of::<usize>()) / T::min_size_of_components();

    let mut len = max_len;
    while {
        // this variable is not inlined (in debug builds) only for better debugging experience
        let to_len_in_bytes = buffer_layout_unaligned::<T>(len)
            .expect("layout size should not exceed `isize::MAX`")
            .size();
        to_len_in_bytes > len_in_bytes
    } {
        len -= 1;
    }
    len
}

#[inline]
pub(crate) fn to_len<T>(len_in_bytes: usize) -> usize
where
    T: Soa,
{
    let layout = Layout::from_size_align(len_in_bytes, align_of::<BufferAlign<T>>())
        .expect("layout should be valid");
    let aligned_len = layout.pad_to_align().size();
    unaligned_to_len::<T>(aligned_len)
}

#[inline]
pub(crate) unsafe fn ptrs<T>(ptr: *mut u8, len: usize) -> T::MutPtrs
where
    T: Soa,
{
    if T::min_size_of_components() == 0 || len == 0 {
        return T::ptrs_dangling();
    }

    let initial = Layout::new::<usize>();
    unsafe { T::ptrs(ptr, initial, len) }
}

#[cfg(test)]
#[allow(clippy::identity_op)]
mod tests {
    use core::alloc::Layout;

    use super::{buffer_layout_unaligned, unaligned_to_len};

    #[test]
    fn u8_u8_u8_to_len_in_bytes() {
        let to_len_in_bytes = |len| buffer_layout_unaligned::<(u8, u8, u8)>(len).unwrap().size();
        let usize = size_of::<usize>();
        let u8 = size_of::<u8>();

        assert_eq!(to_len_in_bytes(0), 0);
        assert_eq!(to_len_in_bytes(1), usize + 3 * u8 * 1);
        assert_eq!(to_len_in_bytes(2), usize + 3 * u8 * 2);
        assert_eq!(to_len_in_bytes(3), usize + 3 * u8 * 3);
        assert_eq!(to_len_in_bytes(4), usize + 3 * u8 * 4);
        assert_eq!(to_len_in_bytes(5), usize + 3 * u8 * 5);
        assert_eq!(to_len_in_bytes(6), usize + 3 * u8 * 6);
        assert_eq!(to_len_in_bytes(7), usize + 3 * u8 * 7);
        assert_eq!(to_len_in_bytes(8), usize + 3 * u8 * 8);
        assert_eq!(to_len_in_bytes(9), usize + 3 * u8 * 9);
    }

    #[test]
    fn u8_u8_u8_to_len() {
        let to_len = unaligned_to_len::<(u8, u8, u8)>;
        let usize = size_of::<usize>();
        let u8 = size_of::<u8>();

        for len_in_bytes in 0..(usize + 3 * u8 * 1) {
            assert_eq!(to_len(len_in_bytes), 0);
        }

        assert_eq!(1, to_len(usize + 3 * u8 * 1));
        assert_eq!(1, to_len(usize + 3 * u8 * 1 + 1));
        assert_eq!(1, to_len(usize + 3 * u8 * 2 - 1));

        assert_eq!(2, to_len(usize + 3 * u8 * 2));
        assert_eq!(2, to_len(usize + 3 * u8 * 2 + 1));
        assert_eq!(2, to_len(usize + 3 * u8 * 3 - 1));

        assert_eq!(3, to_len(usize + 3 * u8 * 3));
        assert_eq!(3, to_len(usize + 3 * u8 * 3 + 1));
        assert_eq!(3, to_len(usize + 3 * u8 * 4 - 1));

        assert_eq!(4, to_len(usize + 3 * u8 * 4));
    }

    #[test]
    fn u16_u16_u16_to_len_in_bytes() {
        let to_len_in_bytes = |len| {
            buffer_layout_unaligned::<(u16, u16, u16)>(len)
                .unwrap()
                .size()
        };
        let usize = size_of::<usize>();
        let u16 = size_of::<u16>();

        assert_eq!(to_len_in_bytes(0), 0);
        assert_eq!(to_len_in_bytes(1), usize + 3 * u16 * 1);
        assert_eq!(to_len_in_bytes(2), usize + 3 * u16 * 2);
        assert_eq!(to_len_in_bytes(3), usize + 3 * u16 * 3);
        assert_eq!(to_len_in_bytes(4), usize + 3 * u16 * 4);
        assert_eq!(to_len_in_bytes(5), usize + 3 * u16 * 5);
        assert_eq!(to_len_in_bytes(6), usize + 3 * u16 * 6);
        assert_eq!(to_len_in_bytes(7), usize + 3 * u16 * 7);
        assert_eq!(to_len_in_bytes(8), usize + 3 * u16 * 8);
        assert_eq!(to_len_in_bytes(9), usize + 3 * u16 * 9);
    }

    #[test]
    fn u16_u16_u16_to_len() {
        let to_len = unaligned_to_len::<(u16, u16, u16)>;
        let usize = size_of::<usize>();
        let u16 = size_of::<u16>();

        for len_in_bytes in 0..(usize + 3 * u16 * 1) {
            assert_eq!(to_len(len_in_bytes), 0);
        }

        assert_eq!(1, to_len(usize + 3 * u16 * 1));
        assert_eq!(1, to_len(usize + 3 * u16 * 1 + 1));
        assert_eq!(1, to_len(usize + 3 * u16 * 2 - 1));

        assert_eq!(2, to_len(usize + 3 * u16 * 2));
        assert_eq!(2, to_len(usize + 3 * u16 * 2 + 1));
        assert_eq!(2, to_len(usize + 3 * u16 * 3 - 1));

        assert_eq!(3, to_len(usize + 3 * u16 * 3));
    }

    #[test]
    fn u32_u32_u32_to_len_in_bytes() {
        let to_len_in_bytes = |len| {
            buffer_layout_unaligned::<(u32, u32, u32)>(len)
                .unwrap()
                .size()
        };
        let u32 = size_of::<u32>();
        let aligned_len = Layout::new::<usize>()
            .align_to(align_of::<u32>())
            .unwrap()
            .pad_to_align()
            .size();

        assert_eq!(to_len_in_bytes(0), 0);
        assert_eq!(to_len_in_bytes(1), aligned_len + 3 * u32 * 1);
        assert_eq!(to_len_in_bytes(2), aligned_len + 3 * u32 * 2);
        assert_eq!(to_len_in_bytes(3), aligned_len + 3 * u32 * 3);
        assert_eq!(to_len_in_bytes(4), aligned_len + 3 * u32 * 4);
        assert_eq!(to_len_in_bytes(5), aligned_len + 3 * u32 * 5);
        assert_eq!(to_len_in_bytes(6), aligned_len + 3 * u32 * 6);
        assert_eq!(to_len_in_bytes(7), aligned_len + 3 * u32 * 7);
        assert_eq!(to_len_in_bytes(8), aligned_len + 3 * u32 * 8);
    }

    #[test]
    fn u32_u32_u32_to_len() {
        let to_len = unaligned_to_len::<(u32, u32, u32)>;
        let u32 = size_of::<u32>();
        let aligned_len = Layout::new::<usize>()
            .align_to(align_of::<u32>())
            .unwrap()
            .pad_to_align()
            .size();

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
    fn u64_u64_u64_to_len_in_bytes() {
        let to_len_in_bytes = |len| {
            buffer_layout_unaligned::<(u64, u64, u64)>(len)
                .unwrap()
                .size()
        };
        let u64 = size_of::<u64>();
        let aligned_len = Layout::new::<usize>()
            .align_to(align_of::<u64>())
            .unwrap()
            .pad_to_align()
            .size();

        assert_eq!(to_len_in_bytes(0), 0);
        assert_eq!(to_len_in_bytes(1), aligned_len + 3 * u64 * 1);
        assert_eq!(to_len_in_bytes(2), aligned_len + 3 * u64 * 2);
        assert_eq!(to_len_in_bytes(3), aligned_len + 3 * u64 * 3);
        assert_eq!(to_len_in_bytes(4), aligned_len + 3 * u64 * 4);
        assert_eq!(to_len_in_bytes(5), aligned_len + 3 * u64 * 5);
        assert_eq!(to_len_in_bytes(6), aligned_len + 3 * u64 * 6);
        assert_eq!(to_len_in_bytes(7), aligned_len + 3 * u64 * 7);
        assert_eq!(to_len_in_bytes(8), aligned_len + 3 * u64 * 8);
    }

    #[test]
    fn u64_u64_u64_to_len() {
        let to_len = unaligned_to_len::<(u64, u64, u64)>;
        let u64 = size_of::<u64>();
        let aligned_len = Layout::new::<usize>()
            .align_to(align_of::<u64>())
            .unwrap()
            .pad_to_align()
            .size();

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
    #[rustfmt::skip::macros(assert_eq)]
    fn u8_u16_u32_to_len_in_bytes() {
        let to_len_in_bytes = |len| {
            buffer_layout_unaligned::<(u8, u16, u32)>(len)
                .unwrap()
                .size()
        };
        let u8 = size_of::<u8>();
        let u16 = size_of::<u16>();
        let u32 = size_of::<u32>();
        let usize = size_of::<usize>();

        assert_eq!(to_len_in_bytes(0), 0);
        assert_eq!(to_len_in_bytes(1), usize + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1));
        assert_eq!(to_len_in_bytes(2), usize + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2));
        assert_eq!(to_len_in_bytes(3), usize + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3));
        assert_eq!(to_len_in_bytes(4), usize + (u8 * 4) + 0 + (u16 * 4) + 0 + (u32 * 4));
        assert_eq!(to_len_in_bytes(5), usize + (u8 * 5) + 1 + (u16 * 5) + 0 + (u32 * 5));
        assert_eq!(to_len_in_bytes(6), usize + (u8 * 6) + 0 + (u16 * 6) + 2 + (u32 * 6));
        assert_eq!(to_len_in_bytes(7), usize + (u8 * 7) + 1 + (u16 * 7) + 2 + (u32 * 7));
        assert_eq!(to_len_in_bytes(8), usize + (u8 * 8) + 0 + (u16 * 8) + 0 + (u32 * 8));
    }

    #[test]
    #[rustfmt::skip::macros(assert_eq)]
    fn u8_u16_u32_to_len() {
        let to_len = unaligned_to_len::<(u8, u16, u32)>;
        let u8 = size_of::<u8>();
        let u16 = size_of::<u16>();
        let u32 = size_of::<u32>();
        let usize = size_of::<usize>();

        for len_in_bytes in 0..(usize + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1)) {
            assert_eq!(to_len(len_in_bytes), 0);
        }

        assert_eq!(1, to_len(usize + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1)));
        assert_eq!(1, to_len(usize + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1) + 1));
        assert_eq!(1, to_len(usize + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2) - 1));

        assert_eq!(2, to_len(usize + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2)));
        assert_eq!(2, to_len(usize + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2) + 1));
        assert_eq!(2, to_len(usize + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3) - 1));

        assert_eq!(3, to_len(usize + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3)));
    }

    #[test]
    fn u32_u16_u8_to_len_in_bytes() {
        let to_len_in_bytes = |len| {
            buffer_layout_unaligned::<(u32, u16, u8)>(len)
                .unwrap()
                .size()
        };
        let efficient_to_len_in_bytes = |len| {
            buffer_layout_unaligned::<(u8, u16, u32)>(len)
                .unwrap()
                .size()
        };

        assert_eq!(to_len_in_bytes(0), efficient_to_len_in_bytes(0));
        assert_eq!(to_len_in_bytes(1), efficient_to_len_in_bytes(1));
        assert_eq!(to_len_in_bytes(2), efficient_to_len_in_bytes(2));
        assert_eq!(to_len_in_bytes(3), efficient_to_len_in_bytes(3));
        assert_eq!(to_len_in_bytes(4), efficient_to_len_in_bytes(4));
        assert_eq!(to_len_in_bytes(5), efficient_to_len_in_bytes(5));
        assert_eq!(to_len_in_bytes(6), efficient_to_len_in_bytes(6));
        assert_eq!(to_len_in_bytes(7), efficient_to_len_in_bytes(7));
        assert_eq!(to_len_in_bytes(8), efficient_to_len_in_bytes(8));
    }

    #[test]
    fn u32_u16_u8_to_len() {
        let to_len = unaligned_to_len::<(u32, u16, u8)>;
        let efficient_to_len = unaligned_to_len::<(u8, u16, u32)>;

        for len_in_bytes in 0..128 {
            assert_eq!(to_len(len_in_bytes), efficient_to_len(len_in_bytes));
        }
    }

    #[test]
    fn u8_u16_u8_to_len_in_bytes() {
        let to_len_in_bytes = |len| {
            buffer_layout_unaligned::<(u8, u16, u8)>(len)
                .unwrap()
                .size()
        };
        let efficient_to_len_in_bytes = |len| {
            buffer_layout_unaligned::<(u8, u8, u16)>(len)
                .unwrap()
                .size()
        };

        assert_eq!(to_len_in_bytes(0), efficient_to_len_in_bytes(0));
        assert_eq!(to_len_in_bytes(1), efficient_to_len_in_bytes(1));
        assert_eq!(to_len_in_bytes(2), efficient_to_len_in_bytes(2));
        assert_eq!(to_len_in_bytes(3), efficient_to_len_in_bytes(3));
        assert_eq!(to_len_in_bytes(4), efficient_to_len_in_bytes(4));
        assert_eq!(to_len_in_bytes(5), efficient_to_len_in_bytes(5));
        assert_eq!(to_len_in_bytes(6), efficient_to_len_in_bytes(6));
        assert_eq!(to_len_in_bytes(7), efficient_to_len_in_bytes(7));
        assert_eq!(to_len_in_bytes(8), efficient_to_len_in_bytes(8));
    }

    #[test]
    fn u8_u16_u8_to_len() {
        let to_len = unaligned_to_len::<(u8, u16, u8)>;
        let efficient_to_len = unaligned_to_len::<(u8, u8, u16)>;

        for len_in_bytes in 0..128 {
            assert_eq!(to_len(len_in_bytes), efficient_to_len(len_in_bytes));
        }
    }

    #[test]
    fn u16_u8_u16_to_len_in_bytes() {
        let to_len_in_bytes = |len| {
            buffer_layout_unaligned::<(u16, u8, u16)>(len)
                .unwrap()
                .size()
        };
        let efficient_to_len_in_bytes = |len| {
            buffer_layout_unaligned::<(u8, u16, u16)>(len)
                .unwrap()
                .size()
        };

        assert_eq!(to_len_in_bytes(0), efficient_to_len_in_bytes(0));
        assert_eq!(to_len_in_bytes(1), efficient_to_len_in_bytes(1));
        assert_eq!(to_len_in_bytes(2), efficient_to_len_in_bytes(2));
        assert_eq!(to_len_in_bytes(3), efficient_to_len_in_bytes(3));
        assert_eq!(to_len_in_bytes(4), efficient_to_len_in_bytes(4));
        assert_eq!(to_len_in_bytes(5), efficient_to_len_in_bytes(5));
        assert_eq!(to_len_in_bytes(6), efficient_to_len_in_bytes(6));
        assert_eq!(to_len_in_bytes(7), efficient_to_len_in_bytes(7));
        assert_eq!(to_len_in_bytes(8), efficient_to_len_in_bytes(8));
    }

    #[test]
    fn u16_u8_u16_to_len() {
        let to_len = unaligned_to_len::<(u16, u8, u16)>;
        let efficient_to_len = unaligned_to_len::<(u8, u16, u16)>;

        for len_in_bytes in 0..128 {
            assert_eq!(to_len(len_in_bytes), efficient_to_len(len_in_bytes));
        }
    }

    #[test]
    fn u16_u8_u32_to_len_in_bytes() {
        let to_len_in_bytes = |len| {
            buffer_layout_unaligned::<(u16, u8, u32)>(len)
                .unwrap()
                .size()
        };
        let efficient_to_len_in_bytes = |len| {
            buffer_layout_unaligned::<(u8, u16, u32)>(len)
                .unwrap()
                .size()
        };

        assert_eq!(to_len_in_bytes(0), efficient_to_len_in_bytes(0));
        assert_eq!(to_len_in_bytes(1), efficient_to_len_in_bytes(1));
        assert_eq!(to_len_in_bytes(2), efficient_to_len_in_bytes(2));
        assert_eq!(to_len_in_bytes(3), efficient_to_len_in_bytes(3));
        assert_eq!(to_len_in_bytes(4), efficient_to_len_in_bytes(4));
        assert_eq!(to_len_in_bytes(5), efficient_to_len_in_bytes(5));
        assert_eq!(to_len_in_bytes(6), efficient_to_len_in_bytes(6));
        assert_eq!(to_len_in_bytes(7), efficient_to_len_in_bytes(7));
        assert_eq!(to_len_in_bytes(8), efficient_to_len_in_bytes(8));
    }

    #[test]
    fn u16_u8_u32_to_len() {
        let to_len = unaligned_to_len::<(u16, u8, u32)>;
        let efficient_to_len = unaligned_to_len::<(u8, u16, u32)>;

        for len_in_bytes in 0..128 {
            assert_eq!(to_len(len_in_bytes), efficient_to_len(len_in_bytes));
        }
    }

    #[test]
    fn u16_u32_u16_to_len_in_bytes() {
        let to_len_in_bytes = |len| {
            buffer_layout_unaligned::<(u16, u32, u16)>(len)
                .unwrap()
                .size()
        };
        let efficient_to_len_in_bytes = |len| {
            buffer_layout_unaligned::<(u16, u16, u32)>(len)
                .unwrap()
                .size()
        };

        assert_eq!(to_len_in_bytes(0), efficient_to_len_in_bytes(0));
        assert_eq!(to_len_in_bytes(1), efficient_to_len_in_bytes(1));
        assert_eq!(to_len_in_bytes(2), efficient_to_len_in_bytes(2));
        assert_eq!(to_len_in_bytes(3), efficient_to_len_in_bytes(3));
        assert_eq!(to_len_in_bytes(4), efficient_to_len_in_bytes(4));
        assert_eq!(to_len_in_bytes(5), efficient_to_len_in_bytes(5));
        assert_eq!(to_len_in_bytes(6), efficient_to_len_in_bytes(6));
        assert_eq!(to_len_in_bytes(7), efficient_to_len_in_bytes(7));
        assert_eq!(to_len_in_bytes(8), efficient_to_len_in_bytes(8));
    }

    #[test]
    fn u16_u32_u16_to_len() {
        let to_len = unaligned_to_len::<(u16, u32, u16)>;
        let efficient_to_len = unaligned_to_len::<(u16, u16, u32)>;

        for len_in_bytes in 0..128 {
            assert_eq!(to_len(len_in_bytes), efficient_to_len(len_in_bytes));
        }
    }
}
