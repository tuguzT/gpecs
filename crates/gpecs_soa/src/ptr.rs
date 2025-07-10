use core::{
    alloc::{Layout, LayoutError},
    mem::{ManuallyDrop, MaybeUninit, offset_of},
    ptr,
};

use crate::{
    slice::SoaSlice,
    traits::{Soa, SoaTrustedFields},
};

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn slice_from_raw_parts<T>(
    data: *const BufferData<T>,
    len: usize,
    capacity: usize,
) -> *const SoaSlice<T>
where
    T: SoaTrustedFields,
{
    let context = unsafe { &*data.ptr_to_context() };
    let len = len_for_inner::<T>(context, len, capacity);
    ptr::slice_from_raw_parts(data, len) as _
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn slice_from_raw_parts_mut<T>(
    data: *mut BufferData<T>,
    len: usize,
    capacity: usize,
) -> *mut SoaSlice<T>
where
    T: SoaTrustedFields,
{
    let context = unsafe { &*data.ptr_to_context() };
    let len = len_for_inner::<T>(context, len, capacity);
    ptr::slice_from_raw_parts_mut(data, len) as _
}

#[inline]
fn len_for_inner<T>(context: &T::Context, len: usize, capacity: usize) -> usize
where
    T: Soa,
{
    if !should_allocate::<T>(context, capacity) {
        return len;
    }

    let capacity_in_bytes = buffer_layout::<T>(context, capacity)
        .expect("layout size should not exceed `isize::MAX`")
        .size();
    capacity_in_bytes / size_of::<BufferData<T>>()
}

/// Special type which is used to properly allocate a buffer in memory
/// with respect to the size and alignment of
/// [`Fields`](`Soa::Fields`) and [`Context`](`Soa::Context`) associated types of `T`.
pub union BufferData<T>
where
    T: Soa,
{
    _len_align: [usize; 0],
    _fields: ManuallyDrop<MaybeUninit<T::Fields>>,
    _context: ManuallyDrop<MaybeUninit<T::Context>>,
}

pub trait SoaSlicePtr<T>: Copy + private_slice_ptr::Sealed
where
    T: SoaTrustedFields,
{
    #[allow(clippy::missing_safety_doc)]
    unsafe fn context<'a>(self) -> &'a T::Context;

    #[allow(clippy::missing_safety_doc)]
    unsafe fn len(self) -> usize;

    #[allow(clippy::missing_safety_doc)]
    #[inline(always)]
    unsafe fn is_empty(self) -> bool {
        unsafe { self.len() == 0 }
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe fn capacity(self) -> usize;

    fn as_ptr(self) -> *const BufferData<T>;
}

impl<T> SoaSlicePtr<T> for *const SoaSlice<T>
where
    T: SoaTrustedFields,
{
    #[inline]
    unsafe fn context<'a>(self) -> &'a <T as Soa>::Context {
        let buffer = self.as_ptr();
        unsafe { &*buffer.ptr_to_context() }
    }

    #[inline]
    unsafe fn len(self) -> usize {
        let buffer_layout = slice_buffer_layout(self);
        match buffer_layout.size() {
            0 => self.into_inner().len(),
            _ => unsafe { ptr::read(self.as_ptr().ptr_to_len()) },
        }
    }

    #[inline]
    unsafe fn capacity(self) -> usize {
        let buffer_layout = slice_buffer_layout(self);
        let context = unsafe { self.context() };
        capacity_from::<T>(context, buffer_layout)
    }

    #[inline]
    fn as_ptr(self) -> *const BufferData<T> {
        let buffer = self.into_inner();
        buffer as *const BufferData<T> // should be `<*const [BufferData<T>]>::as_ptr(buffer)` but it's unstable
    }
}

pub trait SoaSlicePtrMut<T>: Copy + private_slice_ptr::Sealed
where
    T: SoaTrustedFields,
{
    #[allow(clippy::missing_safety_doc)]
    unsafe fn context<'a>(self) -> &'a T::Context;

    #[allow(clippy::missing_safety_doc)]
    unsafe fn len(self) -> usize;

    #[allow(clippy::missing_safety_doc)]
    #[inline(always)]
    unsafe fn is_empty(self) -> bool {
        unsafe { self.len() == 0 }
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe fn capacity(self) -> usize;

    fn as_mut_ptr(self) -> *mut BufferData<T>;
}

impl<T> SoaSlicePtrMut<T> for *mut SoaSlice<T>
where
    T: SoaTrustedFields,
{
    #[inline]
    unsafe fn context<'a>(self) -> &'a <T as Soa>::Context {
        let buffer = self.as_mut_ptr();
        unsafe { &*buffer.ptr_to_context_mut() }
    }

    #[inline]
    unsafe fn len(self) -> usize {
        let buffer_layout = slice_buffer_layout(self);
        match buffer_layout.size() {
            0 => self.into_inner().len(),
            _ => unsafe { ptr::read(self.as_mut_ptr().ptr_to_len_mut()) },
        }
    }

    #[inline]
    unsafe fn capacity(self) -> usize {
        let buffer_layout = slice_buffer_layout(self);
        let context = unsafe { self.context() };
        capacity_from::<T>(context, buffer_layout)
    }

    #[inline]
    fn as_mut_ptr(self) -> *mut BufferData<T> {
        let buffer = self.into_inner_mut();
        buffer as *mut BufferData<T> // should be `<*mut [BufferData<T>]>::as_mut_ptr(buffer)` but it's unstable
    }
}

fn slice_buffer_layout<T>(ptr: *const SoaSlice<T>) -> Layout
where
    T: SoaTrustedFields,
{
    let buffer = ptr.into_inner();

    let size = buffer.len() * size_of::<BufferData<T>>();
    let align = align_of::<BufferData<T>>();
    Layout::from_size_align(size, align).expect("layout size should not exceed `isize::MAX`")
}

trait SoaSlicePtrIntoInner<T>: Copy
where
    T: SoaTrustedFields,
{
    fn into_inner(self) -> *const [BufferData<T>];
}

impl<T> SoaSlicePtrIntoInner<T> for *const SoaSlice<T>
where
    T: SoaTrustedFields,
{
    #[inline(always)]
    fn into_inner(self) -> *const [BufferData<T>] {
        self as *const [BufferData<T>]
    }
}

trait SoaSlicePtrIntoInnerMut<T>: Copy
where
    T: SoaTrustedFields,
{
    fn into_inner_mut(self) -> *mut [BufferData<T>];
}

impl<T> SoaSlicePtrIntoInnerMut<T> for *mut SoaSlice<T>
where
    T: SoaTrustedFields,
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
    _fields_align: [T::Fields; 0],
    context: T::Context,
    len: usize,
}

const _: () = {
    const fn assert_safety_preconditions<T>()
    where
        T: Soa,
    {
        assert!(
            offset_of!(BufferPrefix<T>, context) == 0,
            "context should be located at the beginning of the buffer prefix",
        );
        assert!(
            align_of::<BufferData<T>>() == align_of::<BufferPrefix<T>>(),
            "alignment of buffer data and prefix should be the same",
        );
    }

    assert_safety_preconditions::<()>();
    assert_safety_preconditions::<(u8, u8, u8)>();
    assert_safety_preconditions::<(u8, u32, u16)>();
    assert_safety_preconditions::<(u128,)>();
};

pub(crate) trait BufferDataPtr<T>: Copy
where
    T: Soa,
{
    unsafe fn ptr_to_len(self) -> *const usize;
    fn ptr_to_context(self) -> *const T::Context;
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

    #[inline(always)]
    fn ptr_to_context(self) -> *const T::Context {
        self.cast()
    }
}

pub(crate) trait BufferDataPtrMut<T>: Copy
where
    T: Soa,
{
    unsafe fn ptr_to_len_mut(self) -> *mut usize;
    fn ptr_to_context_mut(self) -> *mut T::Context;
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

    #[inline(always)]
    fn ptr_to_context_mut(self) -> *mut T::Context {
        self.cast()
    }
}

mod private_slice_ptr {
    use super::{SoaSlice, SoaTrustedFields};

    pub trait Sealed {}

    impl<T> Sealed for *const SoaSlice<T> where T: SoaTrustedFields {}
    impl<T> Sealed for *mut SoaSlice<T> where T: SoaTrustedFields {}
}

#[inline]
#[track_caller]
pub(crate) fn is_zst<T>(context: &T::Context) -> bool
where
    T: Soa,
{
    let packed_size = T::field_descriptors(context)
        .into_iter()
        .map(|desc| desc.as_ref().layout().size())
        .sum::<usize>();
    size_of::<T::Fields>() == 0 || packed_size == 0
}

#[inline]
#[track_caller]
pub(crate) fn is_context_zst<T>() -> bool
where
    T: Soa,
{
    size_of::<T::Context>() == 0
}

#[inline]
pub(crate) fn should_allocate<T>(context: &T::Context, capacity: usize) -> bool
where
    T: Soa,
{
    let should_not_allocate = is_context_zst::<T>() && (is_zst::<T>(context) || capacity == 0);
    !should_not_allocate
}

#[inline]
fn buffer_layout_not_padded<T>(context: &T::Context, capacity: usize) -> Result<Layout, LayoutError>
where
    T: Soa,
{
    if is_zst::<T>(context) || capacity == 0 {
        if is_context_zst::<T>() {
            return Ok(Layout::new::<()>());
        }
        return Ok(Layout::new::<BufferPrefix<T>>());
    }

    let layout = T::buffer_layout(context, capacity)?;
    let prefix_layout = Layout::new::<BufferPrefix<T>>();
    let (layout, _) = prefix_layout.extend(layout)?;

    Ok(layout)
}

#[inline]
pub(crate) fn buffer_layout<T>(context: &T::Context, capacity: usize) -> Result<Layout, LayoutError>
where
    T: Soa,
{
    if is_zst::<T>(context) || capacity == 0 {
        if is_context_zst::<T>() {
            return Ok(Layout::new::<()>());
        }
        let item_layout = Layout::new::<BufferData<T>>();
        let size = size_of::<BufferPrefix<T>>().div_ceil(item_layout.size()) * item_layout.size();
        let layout = Layout::from_size_align(size, item_layout.align())?.pad_to_align();
        return Ok(layout);
    }

    let required = buffer_layout_not_padded::<T>(context, capacity)?.pad_to_align();
    let capacity_in_bytes = required.size();

    let item_layout = Layout::new::<BufferData<T>>();
    let size = capacity_in_bytes.div_ceil(item_layout.size()) * item_layout.size();
    let layout = Layout::from_size_align(size, item_layout.align())?.pad_to_align();

    Ok(layout)
}

#[inline]
fn capacity_from_not_padded<T>(context: &T::Context, buffer_layout: Layout) -> usize
where
    T: Soa,
{
    let size_of_prefix = size_of::<BufferPrefix<T>>();
    if is_zst::<T>(context) || buffer_layout.size() < size_of_prefix {
        return 0;
    }

    let size = buffer_layout.size() - size_of_prefix;
    let buffer_layout = Layout::from_size_align(size, buffer_layout.align())
        .expect("layout size should not exceed `isize::MAX`");
    T::capacity_from(context, buffer_layout)
}

#[inline]
pub(crate) fn capacity_from<T>(context: &T::Context, buffer_layout: Layout) -> usize
where
    T: Soa,
{
    let size_of_prefix = size_of::<BufferPrefix<T>>();
    if is_zst::<T>(context) || buffer_layout.size() < size_of_prefix {
        return 0;
    }

    let item_layout = Layout::new::<BufferData<T>>();
    let size = buffer_layout.size().div_ceil(item_layout.size()) * item_layout.size();
    let buffer_layout = Layout::from_size_align(size, item_layout.align())
        .expect("layout size should not exceed `isize::MAX`")
        .pad_to_align();

    capacity_from_not_padded::<T>(context, buffer_layout)
}

#[inline]
pub(crate) unsafe fn ptrs<T>(
    context: &T::Context,
    ptr: *mut BufferData<T>,
    capacity: usize,
) -> Result<T::MutPtrs<'_>, LayoutError>
where
    T: Soa,
{
    if is_zst::<T>(context) || capacity == 0 {
        return Ok(T::ptrs_dangling(context));
    }

    let layout = T::buffer_layout(context, capacity)?;
    let prefix_layout = Layout::new::<BufferPrefix<T>>();
    let (_, offset_from_prefix) = prefix_layout.extend(layout)?;

    let buffer = unsafe { ptr.cast::<u8>().add(offset_from_prefix) };
    let ptrs = unsafe { T::ptrs_from_buffer(context, buffer, capacity) };
    Ok(ptrs)
}

#[cfg(test)]
#[allow(clippy::identity_op)]
mod tests {
    use core::alloc::Layout;

    use crate::ptr::{BufferData, BufferPrefix, should_allocate};

    use super::{buffer_layout_not_padded, capacity_from_not_padded};

    #[test]
    #[cfg_attr(miri, ignore)]
    fn u8_u8_u8_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u8, u8)>(&Default::default(), capacity)
                .unwrap()
                .size()
        };
        let u8 = size_of::<u8>();
        let prefix = Layout::new::<BufferPrefix<(u8, u8, u8)>>()
            .align_to(align_of::<u8>())
            .unwrap()
            .pad_to_align()
            .size();

        assert_eq!(
            to_capacity_in_bytes(0),
            should_allocate::<(u8, u8, u8)>(&Default::default(), 0)
                .then_some(prefix)
                .unwrap_or_default(),
        );
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
    #[cfg_attr(miri, ignore)]
    fn u8_u8_u8_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u8, u8, u8)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u8, u8, u8)>(&Default::default(), buffer_layout)
        };
        let u8 = size_of::<u8>();
        let prefix = Layout::new::<BufferPrefix<(u8, u8, u8)>>()
            .align_to(align_of::<u8>())
            .unwrap()
            .pad_to_align()
            .size();

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
    #[cfg_attr(miri, ignore)]
    fn u16_u16_u16_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u16, u16, u16)>(&Default::default(), capacity)
                .unwrap()
                .size()
        };
        let u16 = size_of::<u16>();
        let prefix = Layout::new::<BufferPrefix<(u16, u16, u16)>>()
            .align_to(align_of::<u16>())
            .unwrap()
            .pad_to_align()
            .size();

        assert_eq!(
            to_capacity_in_bytes(0),
            should_allocate::<(u16, u16, u16)>(&Default::default(), 0)
                .then_some(prefix)
                .unwrap_or_default(),
        );
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
    #[cfg_attr(miri, ignore)]
    fn u16_u16_u16_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u16, u16, u16)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u16, u16, u16)>(&Default::default(), buffer_layout)
        };
        let u16 = size_of::<u16>();
        let prefix = Layout::new::<BufferPrefix<(u16, u16, u16)>>()
            .align_to(align_of::<u16>())
            .unwrap()
            .pad_to_align()
            .size();

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
    #[cfg_attr(miri, ignore)]
    fn u32_u32_u32_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u32, u32, u32)>(&Default::default(), capacity)
                .unwrap()
                .size()
        };
        let u32 = size_of::<u32>();
        let prefix = Layout::new::<BufferPrefix<(u32, u32, u32)>>()
            .align_to(align_of::<u32>())
            .unwrap()
            .pad_to_align()
            .size();

        assert_eq!(
            to_capacity_in_bytes(0),
            should_allocate::<(u32, u32, u32)>(&Default::default(), 0)
                .then_some(prefix)
                .unwrap_or_default(),
        );
        assert_eq!(to_capacity_in_bytes(1), prefix + 3 * u32 * 1);
        assert_eq!(to_capacity_in_bytes(2), prefix + 3 * u32 * 2);
        assert_eq!(to_capacity_in_bytes(3), prefix + 3 * u32 * 3);
        assert_eq!(to_capacity_in_bytes(4), prefix + 3 * u32 * 4);
        assert_eq!(to_capacity_in_bytes(5), prefix + 3 * u32 * 5);
        assert_eq!(to_capacity_in_bytes(6), prefix + 3 * u32 * 6);
        assert_eq!(to_capacity_in_bytes(7), prefix + 3 * u32 * 7);
        assert_eq!(to_capacity_in_bytes(8), prefix + 3 * u32 * 8);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn u32_u32_u32_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u32, u32, u32)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u32, u32, u32)>(&Default::default(), buffer_layout)
        };
        let u32 = size_of::<u32>();
        let prefix = Layout::new::<BufferPrefix<(u32, u32, u32)>>()
            .align_to(align_of::<u32>())
            .unwrap()
            .pad_to_align()
            .size();

        for capacity_in_bytes in 0..(prefix + 3 * u32 * 1) {
            assert_eq!(to_capacity(capacity_in_bytes), 0);
        }

        assert_eq!(1, to_capacity(prefix + 3 * u32 * 1));
        assert_eq!(1, to_capacity(prefix + 3 * u32 * 1 + 1));
        assert_eq!(1, to_capacity(prefix + 3 * u32 * 2 - 1));

        assert_eq!(2, to_capacity(prefix + 3 * u32 * 2));
        assert_eq!(2, to_capacity(prefix + 3 * u32 * 2 + 1));
        assert_eq!(2, to_capacity(prefix + 3 * u32 * 3 - 1));

        assert_eq!(3, to_capacity(prefix + 3 * u32 * 3));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn u64_u64_u64_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u64, u64, u64)>(&Default::default(), capacity)
                .unwrap()
                .size()
        };
        let u64 = size_of::<u64>();
        let prefix = Layout::new::<BufferPrefix<(u64, u64, u64)>>()
            .align_to(align_of::<u64>())
            .unwrap()
            .pad_to_align()
            .size();

        assert_eq!(
            to_capacity_in_bytes(0),
            should_allocate::<(u64, u64, u64)>(&Default::default(), 0)
                .then_some(prefix)
                .unwrap_or_default(),
        );
        assert_eq!(to_capacity_in_bytes(1), prefix + 3 * u64 * 1);
        assert_eq!(to_capacity_in_bytes(2), prefix + 3 * u64 * 2);
        assert_eq!(to_capacity_in_bytes(3), prefix + 3 * u64 * 3);
        assert_eq!(to_capacity_in_bytes(4), prefix + 3 * u64 * 4);
        assert_eq!(to_capacity_in_bytes(5), prefix + 3 * u64 * 5);
        assert_eq!(to_capacity_in_bytes(6), prefix + 3 * u64 * 6);
        assert_eq!(to_capacity_in_bytes(7), prefix + 3 * u64 * 7);
        assert_eq!(to_capacity_in_bytes(8), prefix + 3 * u64 * 8);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn u64_u64_u64_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u64, u64, u64)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u64, u64, u64)>(&Default::default(), buffer_layout)
        };
        let u64 = size_of::<u64>();
        let prefix = Layout::new::<BufferPrefix<(u64, u64, u64)>>()
            .align_to(align_of::<u64>())
            .unwrap()
            .pad_to_align()
            .size();

        for capacity_in_bytes in 0..(prefix + 3 * u64 * 1) {
            assert_eq!(to_capacity(capacity_in_bytes), 0);
        }

        assert_eq!(1, to_capacity(prefix + 3 * u64 * 1));
        assert_eq!(1, to_capacity(prefix + 3 * u64 * 1 + 1));
        assert_eq!(1, to_capacity(prefix + 3 * u64 * 2 - 1));

        assert_eq!(2, to_capacity(prefix + 3 * u64 * 2));
        assert_eq!(2, to_capacity(prefix + 3 * u64 * 2 + 1));
        assert_eq!(2, to_capacity(prefix + 3 * u64 * 3 - 1));

        assert_eq!(3, to_capacity(prefix + 3 * u64 * 3));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    #[rustfmt::skip::macros(assert_eq)]
    fn u8_u16_u32_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u16, u32)>(&Default::default(), capacity)
                .unwrap()
                .size()
        };
        let u8 = size_of::<u8>();
        let u16 = size_of::<u16>();
        let u32 = size_of::<u32>();
        let prefix = Layout::new::<BufferPrefix<(u8, u16, u32)>>()
            .align_to(align_of::<u32>())
            .unwrap()
            .pad_to_align()
            .size();

        assert_eq!(
            to_capacity_in_bytes(0),
            should_allocate::<(u8, u16, u32)>(&Default::default(), 0)
                .then_some(prefix)
                .unwrap_or_default(),
        );
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
    #[cfg_attr(miri, ignore)]
    #[rustfmt::skip::macros(assert_eq)]
    fn u8_u16_u32_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u8, u16, u32)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u8, u16, u32)>(&Default::default(), buffer_layout)
        };
        let u8 = size_of::<u8>();
        let u16 = size_of::<u16>();
        let u32 = size_of::<u32>();
        let prefix = Layout::new::<BufferPrefix<(u8, u16, u32)>>()
            .align_to(align_of::<u32>())
            .unwrap()
            .pad_to_align()
            .size();

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
    #[cfg_attr(miri, ignore)]
    fn u32_u16_u8_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u32, u16, u8)>(&Default::default(), capacity)
                .unwrap()
                .size()
        };
        let efficient_to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u16, u32)>(&Default::default(), capacity)
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
    #[cfg_attr(miri, ignore)]
    fn u32_u16_u8_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u32, u16, u8)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u32, u16, u8)>(&Default::default(), buffer_layout)
        };
        let efficient_to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u8, u16, u32)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u8, u16, u32)>(&Default::default(), buffer_layout)
        };

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes),
            );
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn u8_u16_u8_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u16, u8)>(&Default::default(), capacity)
                .unwrap()
                .size()
        };
        let efficient_to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u8, u16)>(&Default::default(), capacity)
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
    #[cfg_attr(miri, ignore)]
    fn u8_u16_u8_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u8, u16, u8)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u8, u16, u8)>(&Default::default(), buffer_layout)
        };
        let efficient_to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u8, u8, u16)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u8, u8, u16)>(&Default::default(), buffer_layout)
        };

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes),
            );
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn u16_u8_u16_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u16, u8, u16)>(&Default::default(), capacity)
                .unwrap()
                .size()
        };
        let efficient_to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u16, u16)>(&Default::default(), capacity)
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
    #[cfg_attr(miri, ignore)]
    fn u16_u8_u16_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u16, u8, u16)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u16, u8, u16)>(&Default::default(), buffer_layout)
        };
        let efficient_to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u8, u16, u16)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u8, u16, u16)>(&Default::default(), buffer_layout)
        };

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes),
            );
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn u16_u8_u32_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u16, u8, u32)>(&Default::default(), capacity)
                .unwrap()
                .size()
        };
        let efficient_to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u8, u16, u32)>(&Default::default(), capacity)
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
    #[cfg_attr(miri, ignore)]
    fn u16_u8_u32_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u16, u8, u32)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u16, u8, u32)>(&Default::default(), buffer_layout)
        };
        let efficient_to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u8, u16, u32)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u8, u16, u32)>(&Default::default(), buffer_layout)
        };

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes),
            );
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn u16_u32_u16_to_capacity_in_bytes() {
        let to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u16, u32, u16)>(&Default::default(), capacity)
                .unwrap()
                .size()
        };
        let efficient_to_capacity_in_bytes = |capacity| {
            buffer_layout_not_padded::<(u16, u16, u32)>(&Default::default(), capacity)
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
    #[cfg_attr(miri, ignore)]
    fn u16_u32_u16_to_capacity() {
        let to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u16, u32, u16)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u16, u32, u16)>(&Default::default(), buffer_layout)
        };
        let efficient_to_capacity = |capacity_in_bytes| {
            let align = align_of::<BufferData<(u16, u16, u32)>>();
            let buffer_layout = Layout::from_size_align(capacity_in_bytes, align).unwrap();
            capacity_from_not_padded::<(u16, u16, u32)>(&Default::default(), buffer_layout)
        };

        for capacity_in_bytes in 0..128 {
            assert_eq!(
                to_capacity(capacity_in_bytes),
                efficient_to_capacity(capacity_in_bytes),
            );
        }
    }
}
