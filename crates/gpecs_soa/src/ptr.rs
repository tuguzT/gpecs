use core::{
    alloc::{Layout, LayoutError},
    mem::{offset_of, MaybeUninit},
    ptr,
};

use crate::{slice::SoaSlice, traits::Soa};

#[allow(clippy::missing_safety_doc)]
#[inline]
pub fn slice_from_raw_parts<T>(
    data: *const BufferData<T>,
    len: usize,
    capacity: usize,
) -> *const SoaSlice<T>
where
    T: Soa,
{
    let core_len = if is_zst::<T>() || capacity == 0 {
        len
    } else {
        let buffer_layout =
            buffer_layout::<T>(capacity).expect("layout size should not exceed `isize::MAX`");
        buffer_layout.size() / size_of::<BufferData<T>>()
    };
    ptr::slice_from_raw_parts(data, core_len) as _
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub fn slice_from_raw_parts_mut<T>(
    data: *mut BufferData<T>,
    len: usize,
    capacity: usize,
) -> *mut SoaSlice<T>
where
    T: Soa,
{
    let core_len = if is_zst::<T>() || capacity == 0 {
        len
    } else {
        let buffer_layout =
            buffer_layout::<T>(capacity).expect("layout size should not exceed `isize::MAX`");
        buffer_layout.size() / size_of::<BufferData<T>>()
    };
    ptr::slice_from_raw_parts_mut(data, core_len) as _
}

/// Special type which is used internally to properly allocate a buffer in memory
/// respecting the size and alignment of [`SizeAlign`][`Soa::SizeAlign`] associated type of `T`.
pub struct BufferData<T>
where
    T: Soa,
{
    _required_align: [usize; 0],
    _size_align: MaybeUninit<T::SizeAlign>,
}

pub trait SoaSlicePtr<T>: Copy + private_slice_ptr::Sealed
where
    T: Soa,
{
    #[allow(clippy::missing_safety_doc)]
    unsafe fn len(self) -> usize;

    #[allow(clippy::missing_safety_doc)]
    #[inline(always)]
    unsafe fn is_empty(self) -> bool {
        unsafe { self.len() == 0 }
    }

    fn capacity(self) -> usize;

    fn as_ptr(self) -> *const BufferData<T>;
}

impl<T> SoaSlicePtr<T> for *const SoaSlice<T>
where
    T: Soa,
{
    #[inline]
    unsafe fn len(self) -> usize {
        match slice_buffer_layout(self).size() {
            0 => self.into_inner().len(),
            _ => unsafe { ptr::read(self.as_ptr().ptr_to_len()) },
        }
    }

    #[inline]
    fn capacity(self) -> usize {
        let buffer_layout = slice_buffer_layout(self);
        capacity_from::<T>(buffer_layout)
    }

    #[inline]
    fn as_ptr(self) -> *const BufferData<T> {
        let buffer = self.into_inner();
        buffer as *const BufferData<T> // should be `<*const [BufferData<T>]>::as_ptr(buffer)` but it's unstable
    }
}

pub trait SoaSlicePtrMut<T>: Copy + private_slice_ptr::Sealed
where
    T: Soa,
{
    #[allow(clippy::missing_safety_doc)]
    unsafe fn len(self) -> usize;

    #[allow(clippy::missing_safety_doc)]
    #[inline(always)]
    unsafe fn is_empty(self) -> bool {
        unsafe { self.len() == 0 }
    }

    fn capacity(self) -> usize;

    fn as_mut_ptr(self) -> *mut BufferData<T>;
}

impl<T> SoaSlicePtrMut<T> for *mut SoaSlice<T>
where
    T: Soa,
{
    #[inline]
    unsafe fn len(self) -> usize {
        let this = self.cast_const();
        unsafe { this.len() }
    }

    #[inline]
    fn capacity(self) -> usize {
        let this = self.cast_const();
        this.capacity()
    }

    #[inline]
    fn as_mut_ptr(self) -> *mut BufferData<T> {
        let buffer = self.into_inner_mut();
        buffer as *mut BufferData<T> // should be `<*mut [BufferData<T>]>::as_mut_ptr(buffer)` but it's unstable
    }
}

fn slice_buffer_layout<T>(ptr: *const SoaSlice<T>) -> Layout
where
    T: Soa,
{
    let buffer = ptr.into_inner();

    let size = buffer.len() * size_of::<BufferData<T>>();
    let align = align_of::<BufferData<T>>();
    Layout::from_size_align(size, align).expect("layout size should not exceed `isize::MAX`")
}

trait SoaSlicePtrIntoInner<T>: Copy
where
    T: Soa,
{
    fn into_inner(self) -> *const [BufferData<T>];
}

impl<T> SoaSlicePtrIntoInner<T> for *const SoaSlice<T>
where
    T: Soa,
{
    #[inline(always)]
    fn into_inner(self) -> *const [BufferData<T>] {
        self as *const [BufferData<T>]
    }
}

trait SoaSlicePtrIntoInnerMut<T>: Copy
where
    T: Soa,
{
    fn into_inner_mut(self) -> *mut [BufferData<T>];
}

impl<T> SoaSlicePtrIntoInnerMut<T> for *mut SoaSlice<T>
where
    T: Soa,
{
    #[inline(always)]
    fn into_inner_mut(self) -> *mut [BufferData<T>] {
        self as *mut [BufferData<T>]
    }
}

#[repr(C)]
pub(crate) struct BufferPrefix<T>
where
    T: Soa,
{
    pub len: usize,
    _align: [BufferData<T>; 0],
}

pub(crate) trait BufferDataPtr<T>: Copy
where
    T: Soa,
{
    unsafe fn ptr_to_len(self) -> *const usize;
}

impl<T> BufferDataPtr<T> for *const BufferData<T>
where
    T: Soa,
{
    #[inline]
    unsafe fn ptr_to_len(self) -> *const usize {
        let prefix = self.cast::<u8>();
        let len = unsafe { prefix.add(offset_of!(BufferPrefix<T>, len)) };
        len.cast()
    }
}

pub(crate) trait BufferDataPtrMut<T>: Copy
where
    T: Soa,
{
    unsafe fn ptr_to_len_mut(self) -> *mut usize;
}

impl<T> BufferDataPtrMut<T> for *mut BufferData<T>
where
    T: Soa,
{
    #[inline]
    unsafe fn ptr_to_len_mut(self) -> *mut usize {
        let prefix = self.cast::<u8>();
        let len = unsafe { prefix.add(offset_of!(BufferPrefix<T>, len)) };
        len.cast()
    }
}

mod private_slice_ptr {
    use super::{Soa, SoaSlice};

    pub trait Sealed {}

    impl<T> Sealed for *const SoaSlice<T> where T: Soa {}
    impl<T> Sealed for *mut SoaSlice<T> where T: Soa {}
}

#[inline]
#[track_caller]
pub(crate) fn is_zst<T>() -> bool
where
    T: Soa,
{
    size_of::<BufferData<T>>() == 0
}

#[inline]
pub(crate) fn actual_capacity<T>(capacity: usize) -> usize
where
    T: Soa,
{
    let buffer_layout =
        buffer_layout::<T>(capacity).expect("layout size should not exceed `isize::MAX`");
    capacity_from::<T>(buffer_layout)
}

#[inline]
fn buffer_layout_not_padded<T>(capacity: usize) -> Result<Layout, LayoutError>
where
    T: Soa,
{
    if is_zst::<T>() || capacity == 0 {
        return Ok(Layout::new::<()>());
    }

    let (layout, _) = T::buffer_layout(capacity)?;

    let prefix_layout = Layout::new::<BufferPrefix<T>>();
    let (layout, _) = prefix_layout.extend(layout)?;

    Ok(layout)
}

#[inline]
pub(crate) fn buffer_layout<T>(capacity: usize) -> Result<Layout, LayoutError>
where
    T: Soa,
{
    if is_zst::<T>() || capacity == 0 {
        return Ok(Layout::new::<()>());
    }

    let required = buffer_layout_not_padded::<T>(capacity)?.pad_to_align();
    let capacity_in_bytes = required.size();

    let item_layout = Layout::new::<BufferData<T>>();
    let size = capacity_in_bytes.div_ceil(item_layout.size()) * item_layout.size();
    let layout = Layout::from_size_align(size, item_layout.align())?.pad_to_align();

    Ok(layout)
}

#[inline]
fn capacity_from_not_padded<T>(buffer_layout: Layout) -> usize
where
    T: Soa,
{
    let size_of_prefix = size_of::<BufferPrefix<T>>();
    if is_zst::<T>() || buffer_layout.size() < size_of_prefix {
        return 0;
    }

    let size = buffer_layout.size() - size_of_prefix;
    let buffer_layout = Layout::from_size_align(size, buffer_layout.align())
        .expect("layout size should not exceed `isize::MAX`");
    T::capacity_from(buffer_layout)
}

#[inline]
pub(crate) fn capacity_from<T>(buffer_layout: Layout) -> usize
where
    T: Soa,
{
    let size_of_prefix = size_of::<BufferPrefix<T>>();
    if is_zst::<T>() || buffer_layout.size() < size_of_prefix {
        return 0;
    }

    let item_layout = Layout::new::<BufferData<T>>();
    let size = buffer_layout.size().div_ceil(item_layout.size()) * item_layout.size();
    let buffer_layout = Layout::from_size_align(size, item_layout.align())
        .expect("layout size should not exceed `isize::MAX`")
        .pad_to_align();

    capacity_from_not_padded::<T>(buffer_layout)
}

#[inline]
pub(crate) unsafe fn ptrs<T>(
    ptr: *mut BufferData<T>,
    capacity: usize,
) -> Result<T::MutPtrs, LayoutError>
where
    T: Soa,
{
    if is_zst::<T>() || capacity == 0 {
        return Ok(T::ptrs_dangling());
    }

    let (layout, offsets) = T::buffer_layout(capacity)?;

    let prefix_layout = Layout::new::<BufferPrefix<T>>();
    let (_, offset_from_prefix) = prefix_layout.extend(layout)?;
    let offsets = offsets
        .into_iter()
        .map(|offset| offset + offset_from_prefix);

    let ptrs = unsafe { T::ptrs(ptr.cast(), offsets) };
    Ok(ptrs)
}

#[cfg(test)]
#[allow(clippy::identity_op)]
mod tests {
    use core::alloc::Layout;

    use crate::ptr::{BufferData, BufferPrefix};

    use super::{buffer_layout_not_padded, capacity_from_not_padded};

    #[test]
    fn u8_u8_u8_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u8, u8)>(capacity)
                .unwrap()
                .size()
        };
        let prefix = size_of::<BufferPrefix<(u8, u8, u8)>>();
        let u8 = size_of::<u8>();

        assert_eq!(to_capacity_in_bytes(0), 0);
        assert_eq!(to_capacity_in_bytes(1), prefix + 3 * u8 * 1);
        assert_eq!(to_capacity_in_bytes(2), prefix + 3 * u8 * 2);
        assert_eq!(to_capacity_in_bytes(3), prefix + 3 * u8 * 3);
        assert_eq!(to_capacity_in_bytes(4), prefix + 3 * u8 * 4);
        assert_eq!(to_capacity_in_bytes(5), prefix + 3 * u8 * 5);
        assert_eq!(to_capacity_in_bytes(6), prefix + 3 * u8 * 6);
        assert_eq!(to_capacity_in_bytes(7), prefix + 3 * u8 * 7);
        assert_eq!(to_capacity_in_bytes(8), prefix + 3 * u8 * 8);
        assert_eq!(to_capacity_in_bytes(9), prefix + 3 * u8 * 9);
    }

    #[test]
    fn u8_u8_u8_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u8, u8, u8)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u8, u8, u8)>(buffer_layout)
        };
        let prefix = size_of::<BufferPrefix<(u8, u8, u8)>>();
        let u8 = size_of::<u8>();

        for capacity_in_bytes in 0..(prefix + 3 * u8 * 1) {
            assert_eq!(to_capacity(capacity_in_bytes), 0);
        }

        assert_eq!(1, to_capacity(prefix + 3 * u8 * 1));
        assert_eq!(1, to_capacity(prefix + 3 * u8 * 1 + 1));
        assert_eq!(1, to_capacity(prefix + 3 * u8 * 2 - 1));

        assert_eq!(2, to_capacity(prefix + 3 * u8 * 2));
        assert_eq!(2, to_capacity(prefix + 3 * u8 * 2 + 1));
        assert_eq!(2, to_capacity(prefix + 3 * u8 * 3 - 1));

        assert_eq!(3, to_capacity(prefix + 3 * u8 * 3));
        assert_eq!(3, to_capacity(prefix + 3 * u8 * 3 + 1));
        assert_eq!(3, to_capacity(prefix + 3 * u8 * 4 - 1));

        assert_eq!(4, to_capacity(prefix + 3 * u8 * 4));
    }

    #[test]
    fn u16_u16_u16_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u16, u16, u16)>(capacity)
                .unwrap()
                .size()
        };
        let prefix = size_of::<BufferPrefix<(u16, u16, u16)>>();
        let u16 = size_of::<u16>();

        assert_eq!(to_capacity_in_bytes(0), 0);
        assert_eq!(to_capacity_in_bytes(1), prefix + 3 * u16 * 1);
        assert_eq!(to_capacity_in_bytes(2), prefix + 3 * u16 * 2);
        assert_eq!(to_capacity_in_bytes(3), prefix + 3 * u16 * 3);
        assert_eq!(to_capacity_in_bytes(4), prefix + 3 * u16 * 4);
        assert_eq!(to_capacity_in_bytes(5), prefix + 3 * u16 * 5);
        assert_eq!(to_capacity_in_bytes(6), prefix + 3 * u16 * 6);
        assert_eq!(to_capacity_in_bytes(7), prefix + 3 * u16 * 7);
        assert_eq!(to_capacity_in_bytes(8), prefix + 3 * u16 * 8);
        assert_eq!(to_capacity_in_bytes(9), prefix + 3 * u16 * 9);
    }

    #[test]
    fn u16_u16_u16_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u16, u16, u16)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u16, u16, u16)>(buffer_layout)
        };
        let prefix = size_of::<BufferPrefix<(u16, u16, u16)>>();
        let u16 = size_of::<u16>();

        for capacity_in_bytes in 0..(prefix + 3 * u16 * 1) {
            assert_eq!(to_capacity(capacity_in_bytes), 0);
        }

        assert_eq!(1, to_capacity(prefix + 3 * u16 * 1));
        assert_eq!(1, to_capacity(prefix + 3 * u16 * 1 + 1));
        assert_eq!(1, to_capacity(prefix + 3 * u16 * 2 - 1));

        assert_eq!(2, to_capacity(prefix + 3 * u16 * 2));
        assert_eq!(2, to_capacity(prefix + 3 * u16 * 2 + 1));
        assert_eq!(2, to_capacity(prefix + 3 * u16 * 3 - 1));

        assert_eq!(3, to_capacity(prefix + 3 * u16 * 3));
    }

    #[test]
    fn u32_u32_u32_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u32, u32, u32)>(capacity)
                .unwrap()
                .size()
        };
        let u32 = size_of::<u32>();
        let aligned_bytes = Layout::new::<BufferPrefix<(u32, u32, u32)>>()
            .align_to(align_of::<u32>())
            .unwrap()
            .pad_to_align()
            .size();

        assert_eq!(to_capacity_in_bytes(0), 0);
        assert_eq!(to_capacity_in_bytes(1), aligned_bytes + 3 * u32 * 1);
        assert_eq!(to_capacity_in_bytes(2), aligned_bytes + 3 * u32 * 2);
        assert_eq!(to_capacity_in_bytes(3), aligned_bytes + 3 * u32 * 3);
        assert_eq!(to_capacity_in_bytes(4), aligned_bytes + 3 * u32 * 4);
        assert_eq!(to_capacity_in_bytes(5), aligned_bytes + 3 * u32 * 5);
        assert_eq!(to_capacity_in_bytes(6), aligned_bytes + 3 * u32 * 6);
        assert_eq!(to_capacity_in_bytes(7), aligned_bytes + 3 * u32 * 7);
        assert_eq!(to_capacity_in_bytes(8), aligned_bytes + 3 * u32 * 8);
    }

    #[test]
    fn u32_u32_u32_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u32, u32, u32)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u32, u32, u32)>(buffer_layout)
        };
        let u32 = size_of::<u32>();
        let aligned_bytes = Layout::new::<BufferPrefix<(u32, u32, u32)>>()
            .align_to(align_of::<u32>())
            .unwrap()
            .pad_to_align()
            .size();

        for capacity_in_bytes in 0..(aligned_bytes + 3 * u32 * 1) {
            assert_eq!(to_capacity(capacity_in_bytes), 0);
        }

        assert_eq!(1, to_capacity(aligned_bytes + 3 * u32 * 1));
        assert_eq!(1, to_capacity(aligned_bytes + 3 * u32 * 1 + 1));
        assert_eq!(1, to_capacity(aligned_bytes + 3 * u32 * 2 - 1));

        assert_eq!(2, to_capacity(aligned_bytes + 3 * u32 * 2));
        assert_eq!(2, to_capacity(aligned_bytes + 3 * u32 * 2 + 1));
        assert_eq!(2, to_capacity(aligned_bytes + 3 * u32 * 3 - 1));

        assert_eq!(3, to_capacity(aligned_bytes + 3 * u32 * 3));
    }

    #[test]
    fn u64_u64_u64_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u64, u64, u64)>(capacity)
                .unwrap()
                .size()
        };
        let u64 = size_of::<u64>();
        let aligned_bytes = Layout::new::<BufferPrefix<(u64, u64, u64)>>()
            .align_to(align_of::<u64>())
            .unwrap()
            .pad_to_align()
            .size();

        assert_eq!(to_capacity_in_bytes(0), 0);
        assert_eq!(to_capacity_in_bytes(1), aligned_bytes + 3 * u64 * 1);
        assert_eq!(to_capacity_in_bytes(2), aligned_bytes + 3 * u64 * 2);
        assert_eq!(to_capacity_in_bytes(3), aligned_bytes + 3 * u64 * 3);
        assert_eq!(to_capacity_in_bytes(4), aligned_bytes + 3 * u64 * 4);
        assert_eq!(to_capacity_in_bytes(5), aligned_bytes + 3 * u64 * 5);
        assert_eq!(to_capacity_in_bytes(6), aligned_bytes + 3 * u64 * 6);
        assert_eq!(to_capacity_in_bytes(7), aligned_bytes + 3 * u64 * 7);
        assert_eq!(to_capacity_in_bytes(8), aligned_bytes + 3 * u64 * 8);
    }

    #[test]
    fn u64_u64_u64_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u64, u64, u64)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u64, u64, u64)>(buffer_layout)
        };
        let u64 = size_of::<u64>();
        let aligned_bytes = Layout::new::<BufferPrefix<(u64, u64, u64)>>()
            .align_to(align_of::<u64>())
            .unwrap()
            .pad_to_align()
            .size();

        for capacity_in_bytes in 0..(aligned_bytes + 3 * u64 * 1) {
            assert_eq!(to_capacity(capacity_in_bytes), 0);
        }

        assert_eq!(1, to_capacity(aligned_bytes + 3 * u64 * 1));
        assert_eq!(1, to_capacity(aligned_bytes + 3 * u64 * 1 + 1));
        assert_eq!(1, to_capacity(aligned_bytes + 3 * u64 * 2 - 1));

        assert_eq!(2, to_capacity(aligned_bytes + 3 * u64 * 2));
        assert_eq!(2, to_capacity(aligned_bytes + 3 * u64 * 2 + 1));
        assert_eq!(2, to_capacity(aligned_bytes + 3 * u64 * 3 - 1));

        assert_eq!(3, to_capacity(aligned_bytes + 3 * u64 * 3));
    }

    #[test]
    #[rustfmt::skip::macros(assert_eq)]
    fn u8_u16_u32_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u16, u32)>(capacity)
                .unwrap()
                .size()
        };
        let u8 = size_of::<u8>();
        let u16 = size_of::<u16>();
        let u32 = size_of::<u32>();
        let prefix = size_of::<BufferPrefix<(u8, u16, u32)>>();

        assert_eq!(to_capacity_in_bytes(0), 0);
        assert_eq!(to_capacity_in_bytes(1), prefix + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1));
        assert_eq!(to_capacity_in_bytes(2), prefix + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2));
        assert_eq!(to_capacity_in_bytes(3), prefix + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3));
        assert_eq!(to_capacity_in_bytes(4), prefix + (u8 * 4) + 0 + (u16 * 4) + 0 + (u32 * 4));
        assert_eq!(to_capacity_in_bytes(5), prefix + (u8 * 5) + 1 + (u16 * 5) + 0 + (u32 * 5));
        assert_eq!(to_capacity_in_bytes(6), prefix + (u8 * 6) + 0 + (u16 * 6) + 2 + (u32 * 6));
        assert_eq!(to_capacity_in_bytes(7), prefix + (u8 * 7) + 1 + (u16 * 7) + 2 + (u32 * 7));
        assert_eq!(to_capacity_in_bytes(8), prefix + (u8 * 8) + 0 + (u16 * 8) + 0 + (u32 * 8));
    }

    #[test]
    #[rustfmt::skip::macros(assert_eq)]
    fn u8_u16_u32_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u8, u16, u32)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u8, u16, u32)>(buffer_layout)
        };
        let u8 = size_of::<u8>();
        let u16 = size_of::<u16>();
        let u32 = size_of::<u32>();
        let prefix = size_of::<BufferPrefix<(u8, u16, u32)>>();

        for capacity_in_bytes in 0..(prefix + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1)) {
            dbg!(capacity_in_bytes);
            assert_eq!(to_capacity(capacity_in_bytes), 0);
        }

        assert_eq!(1, to_capacity(prefix + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1)));
        assert_eq!(1, to_capacity(prefix + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1) + 1));
        assert_eq!(1, to_capacity(prefix + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2) - 1));

        assert_eq!(2, to_capacity(prefix + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2)));
        assert_eq!(2, to_capacity(prefix + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2) + 1));
        assert_eq!(2, to_capacity(prefix + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3) - 1));

        assert_eq!(3, to_capacity(prefix + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3)));
    }

    #[test]
    fn u32_u16_u8_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u32, u16, u8)>(capacity)
                .unwrap()
                .size()
        };
        let efficient_to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u16, u32)>(capacity)
                .unwrap()
                .size()
        };

        assert_eq!(to_capacity_in_bytes(0), efficient_to_capacity_in_bytes(0));
        assert_eq!(to_capacity_in_bytes(1), efficient_to_capacity_in_bytes(1));
        assert_eq!(to_capacity_in_bytes(2), efficient_to_capacity_in_bytes(2));
        assert_eq!(to_capacity_in_bytes(3), efficient_to_capacity_in_bytes(3));
        assert_eq!(to_capacity_in_bytes(4), efficient_to_capacity_in_bytes(4));
        assert_eq!(to_capacity_in_bytes(5), efficient_to_capacity_in_bytes(5));
        assert_eq!(to_capacity_in_bytes(6), efficient_to_capacity_in_bytes(6));
        assert_eq!(to_capacity_in_bytes(7), efficient_to_capacity_in_bytes(7));
        assert_eq!(to_capacity_in_bytes(8), efficient_to_capacity_in_bytes(8));
    }

    #[test]
    fn u32_u16_u8_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u32, u16, u8)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u32, u16, u8)>(buffer_layout)
        };
        let efficient_to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u8, u16, u32)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u8, u16, u32)>(buffer_layout)
        };

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes),
            );
        }
    }

    #[test]
    fn u8_u16_u8_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u16, u8)>(capacity)
                .unwrap()
                .size()
        };
        let efficient_to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u8, u16)>(capacity)
                .unwrap()
                .size()
        };

        assert_eq!(to_capacity_in_bytes(0), efficient_to_capacity_in_bytes(0));
        assert_eq!(to_capacity_in_bytes(1), efficient_to_capacity_in_bytes(1));
        assert_eq!(to_capacity_in_bytes(2), efficient_to_capacity_in_bytes(2));
        assert_eq!(to_capacity_in_bytes(3), efficient_to_capacity_in_bytes(3));
        assert_eq!(to_capacity_in_bytes(4), efficient_to_capacity_in_bytes(4));
        assert_eq!(to_capacity_in_bytes(5), efficient_to_capacity_in_bytes(5));
        assert_eq!(to_capacity_in_bytes(6), efficient_to_capacity_in_bytes(6));
        assert_eq!(to_capacity_in_bytes(7), efficient_to_capacity_in_bytes(7));
        assert_eq!(to_capacity_in_bytes(8), efficient_to_capacity_in_bytes(8));
    }

    #[test]
    fn u8_u16_u8_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u8, u16, u8)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u8, u16, u8)>(buffer_layout)
        };
        let efficient_to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u8, u8, u16)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u8, u8, u16)>(buffer_layout)
        };

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes),
            );
        }
    }

    #[test]
    fn u16_u8_u16_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u16, u8, u16)>(capacity)
                .unwrap()
                .size()
        };
        let efficient_to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u16, u16)>(capacity)
                .unwrap()
                .size()
        };

        assert_eq!(to_capacity_in_bytes(0), efficient_to_capacity_in_bytes(0));
        assert_eq!(to_capacity_in_bytes(1), efficient_to_capacity_in_bytes(1));
        assert_eq!(to_capacity_in_bytes(2), efficient_to_capacity_in_bytes(2));
        assert_eq!(to_capacity_in_bytes(3), efficient_to_capacity_in_bytes(3));
        assert_eq!(to_capacity_in_bytes(4), efficient_to_capacity_in_bytes(4));
        assert_eq!(to_capacity_in_bytes(5), efficient_to_capacity_in_bytes(5));
        assert_eq!(to_capacity_in_bytes(6), efficient_to_capacity_in_bytes(6));
        assert_eq!(to_capacity_in_bytes(7), efficient_to_capacity_in_bytes(7));
        assert_eq!(to_capacity_in_bytes(8), efficient_to_capacity_in_bytes(8));
    }

    #[test]
    fn u16_u8_u16_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u16, u8, u16)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u16, u8, u16)>(buffer_layout)
        };
        let efficient_to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u8, u16, u16)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u8, u16, u16)>(buffer_layout)
        };

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes),
            );
        }
    }

    #[test]
    fn u16_u8_u32_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u16, u8, u32)>(capacity)
                .unwrap()
                .size()
        };
        let efficient_to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u16, u32)>(capacity)
                .unwrap()
                .size()
        };

        assert_eq!(to_capacity_in_bytes(0), efficient_to_capacity_in_bytes(0));
        assert_eq!(to_capacity_in_bytes(1), efficient_to_capacity_in_bytes(1));
        assert_eq!(to_capacity_in_bytes(2), efficient_to_capacity_in_bytes(2));
        assert_eq!(to_capacity_in_bytes(3), efficient_to_capacity_in_bytes(3));
        assert_eq!(to_capacity_in_bytes(4), efficient_to_capacity_in_bytes(4));
        assert_eq!(to_capacity_in_bytes(5), efficient_to_capacity_in_bytes(5));
        assert_eq!(to_capacity_in_bytes(6), efficient_to_capacity_in_bytes(6));
        assert_eq!(to_capacity_in_bytes(7), efficient_to_capacity_in_bytes(7));
        assert_eq!(to_capacity_in_bytes(8), efficient_to_capacity_in_bytes(8));
    }

    #[test]
    fn u16_u8_u32_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u16, u8, u32)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u16, u8, u32)>(buffer_layout)
        };
        let efficient_to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u8, u16, u32)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u8, u16, u32)>(buffer_layout)
        };

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes),
            );
        }
    }

    #[test]
    fn u16_u32_u16_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u16, u32, u16)>(capacity)
                .unwrap()
                .size()
        };
        let efficient_to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u16, u16, u32)>(capacity)
                .unwrap()
                .size()
        };

        assert_eq!(to_capacity_in_bytes(0), efficient_to_capacity_in_bytes(0));
        assert_eq!(to_capacity_in_bytes(1), efficient_to_capacity_in_bytes(1));
        assert_eq!(to_capacity_in_bytes(2), efficient_to_capacity_in_bytes(2));
        assert_eq!(to_capacity_in_bytes(3), efficient_to_capacity_in_bytes(3));
        assert_eq!(to_capacity_in_bytes(4), efficient_to_capacity_in_bytes(4));
        assert_eq!(to_capacity_in_bytes(5), efficient_to_capacity_in_bytes(5));
        assert_eq!(to_capacity_in_bytes(6), efficient_to_capacity_in_bytes(6));
        assert_eq!(to_capacity_in_bytes(7), efficient_to_capacity_in_bytes(7));
        assert_eq!(to_capacity_in_bytes(8), efficient_to_capacity_in_bytes(8));
    }

    #[test]
    fn u16_u32_u16_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u16, u32, u16)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u16, u32, u16)>(buffer_layout)
        };
        let efficient_to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u16, u16, u32)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u16, u16, u32)>(buffer_layout)
        };

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes),
            );
        }
    }
}
