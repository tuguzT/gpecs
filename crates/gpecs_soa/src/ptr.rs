use core::{
    alloc::{Layout, LayoutError},
    borrow::BorrowMut,
    convert::Infallible,
    mem::MaybeUninit,
    ptr,
};

use crate::{
    slice::{SoaSlice, SoaSliceIndex},
    traits::{IterMut, Soa},
};

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
    let buffer_layout =
        buffer_layout::<T>(capacity).expect("layout size should not exceed `isize::MAX`");
    let core_len = match buffer_layout.size() {
        0 => len,
        _ => buffer_layout.size() / size_of::<BufferData<T>>(),
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
    let buffer_layout =
        buffer_layout::<T>(capacity).expect("layout size should not exceed `isize::MAX`");
    let core_len = match buffer_layout.size() {
        0 => len,
        _ => buffer_layout.size() / size_of::<BufferData<T>>(),
    };
    ptr::slice_from_raw_parts_mut(data, core_len) as _
}

pub struct BufferData<T>
where
    T: Soa,
{
    pub never: Infallible,
    _align: [usize; 0],
    _data: MaybeUninit<T>,
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

    fn capacity_in_bytes(self) -> usize;

    fn as_ptr(self) -> *const BufferData<T>;

    #[allow(clippy::missing_safety_doc)]
    unsafe fn get_unchecked<I>(self, index: I) -> I::Ptr
    where
        I: SoaSliceIndex<SoaSlice<T>>;
}

impl<T> SoaSlicePtr<T> for *const SoaSlice<T>
where
    T: Soa,
{
    #[inline]
    unsafe fn len(self) -> usize {
        match self.capacity_in_bytes() {
            0 => self.into_inner().len(),
            _ => unsafe { ptr::read(self.as_ptr().ptr_to_len()) },
        }
    }

    #[inline]
    fn capacity(self) -> usize {
        let capacity_in_bytes = self.capacity_in_bytes();
        to_capacity::<T>(capacity_in_bytes)
    }

    #[inline]
    fn capacity_in_bytes(self) -> usize {
        let buffer = self.into_inner();
        buffer.len() * size_of::<BufferData<T>>()
    }

    #[inline]
    fn as_ptr(self) -> *const BufferData<T> {
        let buffer = self.into_inner();
        buffer as *const BufferData<T> // should be `<*const [BufferData<T>]>::as_ptr(buffer)`
    }

    #[inline]
    unsafe fn get_unchecked<I>(self, index: I) -> I::Ptr
    where
        I: SoaSliceIndex<SoaSlice<T>>,
    {
        unsafe { index.get_unchecked(self) }
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

    fn capacity_in_bytes(self) -> usize;

    fn as_mut_ptr(self) -> *mut BufferData<T>;

    #[allow(clippy::missing_safety_doc)]
    unsafe fn get_unchecked_mut<I>(self, index: I) -> I::MutPtr
    where
        I: SoaSliceIndex<SoaSlice<T>>;
}

impl<T> SoaSlicePtrMut<T> for *mut SoaSlice<T>
where
    T: Soa,
{
    #[inline]
    unsafe fn len(self) -> usize {
        match self.capacity_in_bytes() {
            0 => self.into_inner_mut().len(),
            _ => unsafe { ptr::read(self.as_mut_ptr().ptr_to_len_mut()) },
        }
    }

    #[inline]
    fn capacity(self) -> usize {
        let capacity_in_bytes = self.capacity_in_bytes();
        to_capacity::<T>(capacity_in_bytes)
    }

    #[inline]
    fn capacity_in_bytes(self) -> usize {
        let buffer = self.into_inner_mut();
        buffer.len() * size_of::<BufferData<T>>()
    }

    #[inline]
    fn as_mut_ptr(self) -> *mut BufferData<T> {
        let buffer = self.into_inner_mut();
        buffer as *mut BufferData<T> // should be `<*mut [BufferData<T>]>::as_mut_ptr(buffer)`
    }

    #[inline]
    unsafe fn get_unchecked_mut<I>(self, index: I) -> I::MutPtr
    where
        I: SoaSliceIndex<SoaSlice<T>>,
    {
        unsafe { index.get_unchecked_mut(self) }
    }
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

pub(crate) trait PtrToLen: Copy {
    fn ptr_to_len(self) -> *const usize;
}

impl<T> PtrToLen for *const BufferData<T>
where
    T: Soa,
{
    #[inline(always)]
    fn ptr_to_len(self) -> *const usize {
        self.cast()
    }
}

pub(crate) trait PtrToLenMut: Copy {
    fn ptr_to_len_mut(self) -> *mut usize;
}

impl<T> PtrToLenMut for *mut BufferData<T>
where
    T: Soa,
{
    #[inline(always)]
    fn ptr_to_len_mut(self) -> *mut usize {
        self.cast()
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
    match (T::packed_size_of(), size_of::<BufferData<T>>()) {
        (0, 0) => true,
        pair => {
            debug_assert!(
                !matches!(pair, (_, 0) | (0, _)),
                "`T::min_size_of_components()` should be `0` if and only if `T` is ZST",
            );
            false
        }
    }
}

#[inline]
pub(crate) fn actual_capacity<T>(capacity: usize) -> usize
where
    T: Soa,
{
    let capacity_in_bytes = buffer_layout::<T>(capacity)
        .expect("layout size should not exceed `isize::MAX`")
        .size();
    to_capacity::<T>(capacity_in_bytes)
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
    let (layout, _) = Layout::new::<usize>().extend(layout)?;
    Ok(layout)
}

#[inline]
pub(crate) fn buffer_layout<T>(capacity: usize) -> Result<Layout, LayoutError>
where
    T: Soa,
{
    let required = buffer_layout_not_padded::<T>(capacity)?.pad_to_align();
    let item_layout = Layout::new::<BufferData<T>>()
        .align_to(required.align())?
        .pad_to_align();
    if item_layout.size() == 0 {
        return Ok(Layout::new::<()>());
    }

    let size = required.size().div_ceil(item_layout.size()) * item_layout.size();
    let layout = Layout::from_size_align(size, item_layout.align())?.pad_to_align();
    Ok(layout)
}

#[inline]
fn to_capacity_not_padded<T>(capacity_in_bytes: usize) -> usize
where
    T: Soa,
{
    if is_zst::<T>() || capacity_in_bytes < size_of::<usize>() {
        return 0;
    }

    let max_capacity = (capacity_in_bytes - size_of::<usize>()) / T::packed_size_of();

    let mut capacity = max_capacity;
    while {
        // this variable is not inlined (in debug builds) only for better debugging experience
        let to_capacity_in_bytes = buffer_layout_not_padded::<T>(capacity)
            .expect("layout size should not exceed `isize::MAX`")
            .size();
        to_capacity_in_bytes > capacity_in_bytes
    } {
        capacity -= 1;
    }
    capacity
}

#[inline]
pub(crate) fn to_capacity<T>(capacity_in_bytes: usize) -> usize
where
    T: Soa,
{
    let item_layout = Layout::new::<BufferData<T>>();
    if item_layout.size() == 0 {
        return 0;
    }

    let size = capacity_in_bytes.div_ceil(item_layout.size()) * item_layout.size();
    let layout = Layout::from_size_align(size, item_layout.align())
        .expect("layout size should not exceed `isize::MAX`")
        .pad_to_align();
    to_capacity_not_padded::<T>(layout.size())
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

    let (layout, mut offsets) = T::buffer_layout(capacity)?;

    let (_, offset_from_len) = Layout::new::<usize>().extend(layout)?;
    for mut item in offsets.iter_mut() {
        let offset: &mut usize = item.borrow_mut();
        *offset += offset_from_len;
    }

    let ptrs = unsafe { T::ptrs(ptr, &offsets) };
    Ok(ptrs)
}

#[cfg(test)]
#[allow(clippy::identity_op)]
mod tests {
    use core::alloc::Layout;

    use super::{buffer_layout_not_padded, to_capacity_not_padded};

    #[test]
    fn u8_u8_u8_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u8, u8)>(capacity)
                .unwrap()
                .size()
        };
        let usize = size_of::<usize>();
        let u8 = size_of::<u8>();

        assert_eq!(to_capacity_in_bytes(0), 0);
        assert_eq!(to_capacity_in_bytes(1), usize + 3 * u8 * 1);
        assert_eq!(to_capacity_in_bytes(2), usize + 3 * u8 * 2);
        assert_eq!(to_capacity_in_bytes(3), usize + 3 * u8 * 3);
        assert_eq!(to_capacity_in_bytes(4), usize + 3 * u8 * 4);
        assert_eq!(to_capacity_in_bytes(5), usize + 3 * u8 * 5);
        assert_eq!(to_capacity_in_bytes(6), usize + 3 * u8 * 6);
        assert_eq!(to_capacity_in_bytes(7), usize + 3 * u8 * 7);
        assert_eq!(to_capacity_in_bytes(8), usize + 3 * u8 * 8);
        assert_eq!(to_capacity_in_bytes(9), usize + 3 * u8 * 9);
    }

    #[test]
    fn u8_u8_u8_to_capacity() {
        let to_capacity = to_capacity_not_padded::<(u8, u8, u8)>;
        let usize = size_of::<usize>();
        let u8 = size_of::<u8>();

        for capacity_in_bytes in 0..(usize + 3 * u8 * 1) {
            assert_eq!(to_capacity(capacity_in_bytes), 0);
        }

        assert_eq!(1, to_capacity(usize + 3 * u8 * 1));
        assert_eq!(1, to_capacity(usize + 3 * u8 * 1 + 1));
        assert_eq!(1, to_capacity(usize + 3 * u8 * 2 - 1));

        assert_eq!(2, to_capacity(usize + 3 * u8 * 2));
        assert_eq!(2, to_capacity(usize + 3 * u8 * 2 + 1));
        assert_eq!(2, to_capacity(usize + 3 * u8 * 3 - 1));

        assert_eq!(3, to_capacity(usize + 3 * u8 * 3));
        assert_eq!(3, to_capacity(usize + 3 * u8 * 3 + 1));
        assert_eq!(3, to_capacity(usize + 3 * u8 * 4 - 1));

        assert_eq!(4, to_capacity(usize + 3 * u8 * 4));
    }

    #[test]
    fn u16_u16_u16_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u16, u16, u16)>(capacity)
                .unwrap()
                .size()
        };
        let usize = size_of::<usize>();
        let u16 = size_of::<u16>();

        assert_eq!(to_capacity_in_bytes(0), 0);
        assert_eq!(to_capacity_in_bytes(1), usize + 3 * u16 * 1);
        assert_eq!(to_capacity_in_bytes(2), usize + 3 * u16 * 2);
        assert_eq!(to_capacity_in_bytes(3), usize + 3 * u16 * 3);
        assert_eq!(to_capacity_in_bytes(4), usize + 3 * u16 * 4);
        assert_eq!(to_capacity_in_bytes(5), usize + 3 * u16 * 5);
        assert_eq!(to_capacity_in_bytes(6), usize + 3 * u16 * 6);
        assert_eq!(to_capacity_in_bytes(7), usize + 3 * u16 * 7);
        assert_eq!(to_capacity_in_bytes(8), usize + 3 * u16 * 8);
        assert_eq!(to_capacity_in_bytes(9), usize + 3 * u16 * 9);
    }

    #[test]
    fn u16_u16_u16_to_capacity() {
        let to_capacity = to_capacity_not_padded::<(u16, u16, u16)>;
        let usize = size_of::<usize>();
        let u16 = size_of::<u16>();

        for capacity_in_bytes in 0..(usize + 3 * u16 * 1) {
            assert_eq!(to_capacity(capacity_in_bytes), 0);
        }

        assert_eq!(1, to_capacity(usize + 3 * u16 * 1));
        assert_eq!(1, to_capacity(usize + 3 * u16 * 1 + 1));
        assert_eq!(1, to_capacity(usize + 3 * u16 * 2 - 1));

        assert_eq!(2, to_capacity(usize + 3 * u16 * 2));
        assert_eq!(2, to_capacity(usize + 3 * u16 * 2 + 1));
        assert_eq!(2, to_capacity(usize + 3 * u16 * 3 - 1));

        assert_eq!(3, to_capacity(usize + 3 * u16 * 3));
    }

    #[test]
    fn u32_u32_u32_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u32, u32, u32)>(capacity)
                .unwrap()
                .size()
        };
        let u32 = size_of::<u32>();
        let aligned_bytes = Layout::new::<usize>()
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
        let to_capacity = to_capacity_not_padded::<(u32, u32, u32)>;
        let u32 = size_of::<u32>();
        let aligned_bytes = Layout::new::<usize>()
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
        let aligned_bytes = Layout::new::<usize>()
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
        let to_capacity = to_capacity_not_padded::<(u64, u64, u64)>;
        let u64 = size_of::<u64>();
        let aligned_bytes = Layout::new::<usize>()
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
        let usize = size_of::<usize>();

        assert_eq!(to_capacity_in_bytes(0), 0);
        assert_eq!(to_capacity_in_bytes(1), usize + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1));
        assert_eq!(to_capacity_in_bytes(2), usize + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2));
        assert_eq!(to_capacity_in_bytes(3), usize + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3));
        assert_eq!(to_capacity_in_bytes(4), usize + (u8 * 4) + 0 + (u16 * 4) + 0 + (u32 * 4));
        assert_eq!(to_capacity_in_bytes(5), usize + (u8 * 5) + 1 + (u16 * 5) + 0 + (u32 * 5));
        assert_eq!(to_capacity_in_bytes(6), usize + (u8 * 6) + 0 + (u16 * 6) + 2 + (u32 * 6));
        assert_eq!(to_capacity_in_bytes(7), usize + (u8 * 7) + 1 + (u16 * 7) + 2 + (u32 * 7));
        assert_eq!(to_capacity_in_bytes(8), usize + (u8 * 8) + 0 + (u16 * 8) + 0 + (u32 * 8));
    }

    #[test]
    #[rustfmt::skip::macros(assert_eq)]
    fn u8_u16_u32_to_capacity() {
        let to_capacity = to_capacity_not_padded::<(u8, u16, u32)>;
        let u8 = size_of::<u8>();
        let u16 = size_of::<u16>();
        let u32 = size_of::<u32>();
        let usize = size_of::<usize>();

        for capacity_in_bytes in 0..(usize + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1)) {
            assert_eq!(to_capacity(capacity_in_bytes), 0);
        }

        assert_eq!(1, to_capacity(usize + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1)));
        assert_eq!(1, to_capacity(usize + (u8 * 1) + 1 + (u16 * 1) + 0 + (u32 * 1) + 1));
        assert_eq!(1, to_capacity(usize + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2) - 1));

        assert_eq!(2, to_capacity(usize + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2)));
        assert_eq!(2, to_capacity(usize + (u8 * 2) + 0 + (u16 * 2) + 2 + (u32 * 2) + 1));
        assert_eq!(2, to_capacity(usize + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3) - 1));

        assert_eq!(3, to_capacity(usize + (u8 * 3) + 1 + (u16 * 3) + 2 + (u32 * 3)));
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
        let to_capacity = to_capacity_not_padded::<(u32, u16, u8)>;
        let efficient_to_capacity = to_capacity_not_padded::<(u8, u16, u32)>;

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes)
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
        let to_capacity = to_capacity_not_padded::<(u8, u16, u8)>;
        let efficient_to_capacity = to_capacity_not_padded::<(u8, u8, u16)>;

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes)
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
        let to_capacity = to_capacity_not_padded::<(u16, u8, u16)>;
        let efficient_to_capacity = to_capacity_not_padded::<(u8, u16, u16)>;

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes)
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
        let to_capacity = to_capacity_not_padded::<(u16, u8, u32)>;
        let efficient_to_capacity = to_capacity_not_padded::<(u8, u16, u32)>;

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes)
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
        let to_capacity = to_capacity_not_padded::<(u16, u32, u16)>;
        let efficient_to_capacity = to_capacity_not_padded::<(u16, u16, u32)>;

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes)
            );
        }
    }
}
