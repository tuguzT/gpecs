use core::{
    alloc::{Layout, LayoutError},
    error::Error,
    fmt::{self, Display},
    mem::{ManuallyDrop, offset_of},
    ptr::{self, NonNull},
};

use crate::{
    field::FieldLayouts,
    layout::WithLayout,
    slice::SoaSlice,
    traits::{AllocSoa, AllocSoaContext, AllocSoaTrusted, MutPtrs, Ptrs, RawSoa, RawSoaContext},
};

#[cfg(test)]
mod tests;

/// Special type which is used to properly allocate a buffer in memory
/// with respect to the size and alignment of
/// [`Fields`](RawSoa::Fields) and [`Context`](RawSoa::Context) associated types.
pub union BufferData<T>
where
    T: AllocSoa + ?Sized,
{
    _align: ManuallyDrop<BufferAlign<T>>,
    _fields: ManuallyDrop<T::Fields>,
    _context: ManuallyDrop<T::Context>,
}

#[repr(C)]
pub struct BufferPrefix<T>
where
    T: AllocSoa + ?Sized,
{
    _align: BufferAlign<T>,
    pub context: T::Context,
    pub len: usize,
    pub capacity: usize,
}

#[repr(C)]
struct BufferAlign<T>
where
    T: AllocSoa + ?Sized,
{
    _fields: [T::Fields; 0],
    _context: [T::Context; 0],
    _len: [usize; 0],
    _capacity: [usize; 0],
}

const _: () = {
    #[cfg_attr(coverage_nightly, coverage(off))]
    const fn assert_safety_preconditions<T>()
    where
        T: AllocSoa + ?Sized,
    {
        assert!(
            size_of::<BufferAlign<T>>() == 0,
            "BufferAlign should not occupy any space",
        );
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

#[inline]
pub fn packed_size_of_fields<I>(fields: I) -> Option<usize>
where
    I: IntoIterator<Item: WithLayout>,
{
    fields
        .into_iter()
        .map(|item| item.layout().size())
        .try_fold(0, usize::checked_add)
}

#[inline]
pub fn align_of_fields<I>(fields: I) -> usize
where
    I: IntoIterator<Item: WithLayout>,
{
    fields
        .into_iter()
        .map(|item| item.layout().align())
        .max()
        .unwrap_or(1)
}

#[inline]
pub fn fields_are_zst<T>(context: &T::Context) -> bool
where
    T: AllocSoa + ?Sized,
{
    packed_size_of_fields(context.field_layouts()) == Some(0)
}

#[inline]
pub const fn context_is_zst<T>() -> bool
where
    T: RawSoa + ?Sized,
    T::Context: Sized,
{
    size_of::<T::Context>() == 0
}

#[inline]
pub fn fields_are_dangling<T>(context: &T::Context, capacity: usize) -> bool
where
    T: AllocSoa + ?Sized,
{
    capacity == 0 || fields_are_zst::<T>(context)
}

#[inline]
pub fn buffer_is_dangling<T>(context: &T::Context, capacity: usize) -> bool
where
    T: AllocSoa + ?Sized,
{
    context_is_zst::<T>() && fields_are_dangling::<T>(context, capacity)
}

#[inline]
pub fn buffer_layout<T>(context: &T::Context, capacity: usize) -> Result<Layout, LayoutError>
where
    T: AllocSoa + ?Sized,
{
    let layout = buffer_layout_inner::<T>(context, capacity)?.pad_to_align();
    Ok(layout)
}

#[inline]
fn buffer_layout_inner<T>(context: &T::Context, capacity: usize) -> Result<Layout, LayoutError>
where
    T: AllocSoa + ?Sized,
{
    let layout = if fields_are_dangling::<T>(context, capacity) {
        let prefix = size_of::<BufferPrefix<T>>();
        let size = if context_is_zst::<T>() { 0 } else { prefix };
        let align = align_of_fields(context.field_layouts());
        Layout::from_size_align(size, align)?
    } else {
        let buffer = context.buffer_layout(capacity)?;
        let prefix = Layout::new::<BufferPrefix<T>>();
        let (layout, _) = prefix.extend(buffer)?;
        layout
    };
    layout.align_to(align_of::<BufferData<T>>())
}

#[inline]
#[cfg_attr(not(feature = "alloc"), expect(unused))]
pub fn capacity_from<T>(context: &T::Context, buffer_layout: Layout) -> usize
where
    T: AllocSoa + ?Sized,
{
    let prefix = Layout::new::<BufferPrefix<T>>();
    if buffer_layout.size() <= prefix.size() || fields_are_zst::<T>(context) {
        return 0;
    }

    let align = align_of_fields(context.field_layouts());
    let next = Layout::from_size_align(0, align)
        .expect("ZST layout should be valid for any possible alignment");
    let (_, offset) = prefix
        .extend(next)
        .expect("extending prefix with ZST layout should always be possible");

    let size = buffer_layout.size() - offset;
    let buffer_layout = Layout::from_size_align(size, buffer_layout.align())
        .expect("layout with size smaller than original should always be valid");
    context.capacity_from(buffer_layout)
}

#[inline]
pub unsafe fn ptrs_from_buffer<T>(
    context: &T::Context,
    ptr: *const BufferData<T>,
    capacity: usize,
) -> Ptrs<'_, T>
where
    T: AllocSoa + ?Sized,
{
    if fields_are_dangling::<T>(context, capacity) {
        return context.ptrs_dangling();
    }

    let buffer = unsafe { ptr_to_data(context, ptr, capacity).unwrap_unchecked() };
    unsafe { context.ptrs_from_buffer(buffer, capacity) }
}

#[inline]
pub unsafe fn ptrs_from_buffer_mut<T>(
    context: &T::Context,
    ptr: *mut BufferData<T>,
    capacity: usize,
) -> MutPtrs<'_, T>
where
    T: AllocSoa + ?Sized,
{
    if fields_are_dangling::<T>(context, capacity) {
        return context.ptrs_dangling_mut();
    }

    let buffer = unsafe { ptr_to_data_mut(context, ptr, capacity).unwrap_unchecked() };
    unsafe { context.ptrs_from_buffer_mut(buffer, capacity) }
}

#[inline]
pub unsafe fn ptr_to_data<T>(
    context: &T::Context,
    ptr: *const BufferData<T>,
    capacity: usize,
) -> Result<*const u8, PtrToDataError>
where
    T: AllocSoa + ?Sized,
{
    let offset = offset_to_data::<T>(context, capacity)?;
    let data = unsafe { ptr.cast::<u8>().add(offset) };
    Ok(data)
}

#[inline]
pub unsafe fn ptr_to_data_mut<T>(
    context: &T::Context,
    ptr: *mut BufferData<T>,
    capacity: usize,
) -> Result<*mut u8, PtrToDataError>
where
    T: AllocSoa + ?Sized,
{
    let offset = offset_to_data::<T>(context, capacity)?;
    let data = unsafe { ptr.cast::<u8>().add(offset) };
    Ok(data)
}

fn offset_to_data<T>(context: &T::Context, capacity: usize) -> Result<usize, PtrToDataError>
where
    T: AllocSoa + ?Sized,
{
    if fields_are_dangling::<T>(context, capacity) {
        let error = DanglingError(());
        return Err(error.into());
    }

    let layout = context.buffer_layout(capacity)?;
    let prefix_layout = Layout::new::<BufferPrefix<T>>();
    let (_, offset) = prefix_layout.extend(layout)?;
    Ok(offset)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DanglingError(());

impl Display for DanglingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "no valid offset to data exists for dangling fields")
    }
}

impl Error for DanglingError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtrToDataError {
    Dangling(DanglingError),
    InvalidLayout(LayoutError),
}

impl From<DanglingError> for PtrToDataError {
    #[inline]
    fn from(error: DanglingError) -> Self {
        Self::Dangling(error)
    }
}

impl From<LayoutError> for PtrToDataError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl Display for PtrToDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dangling(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
        }
    }
}

impl Error for PtrToDataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Dangling(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
        }
    }
}

#[inline]
pub unsafe fn slice_from_raw_parts<T>(
    data: *const BufferData<T>,
    len: usize,
    capacity: usize,
) -> *const SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    let context = unsafe { data.context() };
    let len = len_for_inner::<T>(context, len, capacity);
    ptr::slice_from_raw_parts(data, len) as _
}

#[inline]
pub unsafe fn slice_from_raw_parts_mut<T>(
    data: *mut BufferData<T>,
    len: usize,
    capacity: usize,
) -> *mut SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    let context = unsafe { data.context() };
    let len = len_for_inner::<T>(context, len, capacity);
    ptr::slice_from_raw_parts_mut(data, len) as _
}

#[inline]
fn len_for_inner<T>(context: &T::Context, len: usize, capacity: usize) -> usize
where
    T: AllocSoa + ?Sized,
{
    if buffer_is_dangling::<T>(context, capacity) {
        return len;
    }

    let capacity_in_bytes = buffer_layout::<T>(context, capacity)
        .expect("layout size should not exceed `isize::MAX`")
        .size();
    capacity_in_bytes / size_of::<BufferData<T>>()
}

pub trait SoaSlicePtr<T>: Copy + private::Sealed
where
    T: AllocSoaTrusted + ?Sized,
{
    fn as_ptr(self) -> *const BufferData<T>;

    unsafe fn context<'a>(self) -> &'a T::Context;

    unsafe fn len(self) -> usize;

    #[inline]
    unsafe fn is_empty(self) -> bool {
        unsafe { self.len() == 0 }
    }

    unsafe fn capacity(self) -> usize;
}

impl<T> SoaSlicePtr<T> for *const SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    fn as_ptr(self) -> *const BufferData<T> {
        let buffer = self.into_inner();
        buffer.cast::<BufferData<T>>() // should be `<*const [BufferData<T>]>::as_ptr(buffer)` but it's unstable
    }

    #[inline]
    unsafe fn context<'a>(self) -> &'a T::Context {
        let buffer = self.as_ptr();
        unsafe { buffer.context() }
    }

    #[inline]
    unsafe fn len(self) -> usize {
        if slice_is_dangling(self) {
            return self.into_inner().len();
        }
        unsafe { self.as_ptr().len() }
    }

    #[inline]
    unsafe fn capacity(self) -> usize {
        let context = unsafe { self.context() };
        if fields_are_zst::<T>(context) {
            return usize::MAX;
        }
        if slice_is_dangling(self) {
            return 0;
        }
        unsafe { self.as_ptr().capacity() }
    }
}

pub trait SoaSlicePtrMut<T>: Copy + private::Sealed
where
    T: AllocSoaTrusted + ?Sized,
{
    fn as_mut_ptr(self) -> *mut BufferData<T>;

    unsafe fn context<'a>(self) -> &'a T::Context;

    unsafe fn len(self) -> usize;

    #[inline]
    unsafe fn is_empty(self) -> bool {
        unsafe { self.len() == 0 }
    }

    unsafe fn capacity(self) -> usize;
}

impl<T> SoaSlicePtrMut<T> for *mut SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    fn as_mut_ptr(self) -> *mut BufferData<T> {
        let buffer = self.into_inner_mut();
        buffer.cast::<BufferData<T>>() // should be `<*mut [BufferData<T>]>::as_mut_ptr(buffer)` but it's unstable
    }

    #[inline]
    unsafe fn context<'a>(self) -> &'a T::Context {
        let buffer = self.as_mut_ptr();
        unsafe { buffer.context() }
    }

    #[inline]
    unsafe fn len(self) -> usize {
        if slice_is_dangling(self) {
            return self.into_inner_mut().len();
        }
        unsafe { self.as_mut_ptr().len() }
    }

    #[inline]
    unsafe fn capacity(self) -> usize {
        let context = unsafe { self.context() };
        if fields_are_zst::<T>(context) {
            return usize::MAX;
        }
        if slice_is_dangling(self) {
            return 0;
        }
        unsafe { self.as_mut_ptr().capacity() }
    }
}

fn slice_is_dangling<T>(ptr: *const SoaSlice<T>) -> bool
where
    T: AllocSoaTrusted + ?Sized,
{
    ptr.into_inner().len() == 0 || size_of::<BufferData<T>>() == 0
}

pub trait BufferDataPtr<T>: Copy + private::Sealed
where
    T: AllocSoa + ?Sized,
{
    unsafe fn ptr_to_context(self) -> *const T::Context;
    unsafe fn ptr_to_len(self) -> *const usize;
    unsafe fn ptr_to_capacity(self) -> *const usize;
    unsafe fn ptr_to_data(self) -> *const u8;

    #[inline]
    unsafe fn context<'a>(self) -> &'a T::Context {
        let context = unsafe { self.ptr_to_context() };
        let context = unsafe { NonNull::new_unchecked(context.cast_mut()) };
        unsafe { context.as_ref() }
    }

    #[inline]
    unsafe fn len(self) -> usize {
        let len = unsafe { self.ptr_to_len() };
        unsafe { ptr::read(len) }
    }

    #[inline]
    unsafe fn is_empty(self) -> bool {
        unsafe { self.len() == 0 }
    }

    #[inline]
    unsafe fn capacity(self) -> usize {
        let capacity = unsafe { self.ptr_to_capacity() };
        unsafe { ptr::read(capacity) }
    }
}

impl<T> BufferDataPtr<T> for *const BufferData<T>
where
    T: AllocSoa + ?Sized,
{
    #[inline]
    unsafe fn ptr_to_context(self) -> *const T::Context {
        self.cast()
    }

    #[inline]
    unsafe fn ptr_to_len(self) -> *const usize {
        let prefix = self.cast::<u8>();
        let len = unsafe { prefix.add(offset_of!(BufferPrefix<T>, len)) };
        len.cast()
    }

    #[inline]
    unsafe fn ptr_to_capacity(self) -> *const usize {
        let prefix = self.cast::<u8>();
        let capacity = unsafe { prefix.add(offset_of!(BufferPrefix<T>, capacity)) };
        capacity.cast()
    }

    #[inline]
    unsafe fn ptr_to_data(self) -> *const u8 {
        let context = unsafe { self.context() };
        let capacity = unsafe { self.capacity() };
        unsafe { ptr_to_data(context, self, capacity).unwrap_unchecked() }
    }
}

pub trait BufferDataPtrMut<T>: BufferDataPtr<T>
where
    T: AllocSoa + ?Sized,
{
    unsafe fn ptr_to_context_mut(self) -> *mut T::Context;
    unsafe fn ptr_to_len_mut(self) -> *mut usize;
    unsafe fn ptr_to_capacity_mut(self) -> *mut usize;
    unsafe fn ptr_to_data_mut(self) -> *mut u8;
}

impl<T> BufferDataPtr<T> for *mut BufferData<T>
where
    T: AllocSoa + ?Sized,
{
    #[inline]
    unsafe fn ptr_to_context(self) -> *const T::Context {
        unsafe { self.cast_const().ptr_to_context() }
    }

    #[inline]
    unsafe fn ptr_to_len(self) -> *const usize {
        unsafe { self.cast_const().ptr_to_len() }
    }

    #[inline]
    unsafe fn ptr_to_capacity(self) -> *const usize {
        unsafe { self.cast_const().ptr_to_capacity() }
    }

    #[inline]
    unsafe fn ptr_to_data(self) -> *const u8 {
        unsafe { self.cast_const().ptr_to_data() }
    }
}

impl<T> BufferDataPtrMut<T> for *mut BufferData<T>
where
    T: AllocSoa + ?Sized,
{
    #[inline]
    unsafe fn ptr_to_context_mut(self) -> *mut T::Context {
        self.cast()
    }

    #[inline]
    unsafe fn ptr_to_len_mut(self) -> *mut usize {
        let prefix = self.cast::<u8>();
        let len = unsafe { prefix.add(offset_of!(BufferPrefix<T>, len)) };
        len.cast()
    }

    #[inline]
    unsafe fn ptr_to_capacity_mut(self) -> *mut usize {
        let prefix = self.cast::<u8>();
        let capacity = unsafe { prefix.add(offset_of!(BufferPrefix<T>, capacity)) };
        capacity.cast()
    }

    #[inline]
    unsafe fn ptr_to_data_mut(self) -> *mut u8 {
        let context = unsafe { self.context() };
        let capacity = unsafe { self.capacity() };
        unsafe { ptr_to_data_mut(context, self, capacity).unwrap_unchecked() }
    }
}

trait SoaSlicePtrIntoInner<T>: Copy
where
    T: AllocSoaTrusted + ?Sized,
{
    fn into_inner(self) -> *const [BufferData<T>];
}

impl<T> SoaSlicePtrIntoInner<T> for *const SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    fn into_inner(self) -> *const [BufferData<T>] {
        self as *const [BufferData<T>]
    }
}

trait SoaSlicePtrIntoInnerMut<T>: Copy
where
    T: AllocSoaTrusted + ?Sized,
{
    fn into_inner_mut(self) -> *mut [BufferData<T>];
}

impl<T> SoaSlicePtrIntoInnerMut<T> for *mut SoaSlice<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    fn into_inner_mut(self) -> *mut [BufferData<T>] {
        self as *mut [BufferData<T>]
    }
}

mod private {
    use crate::{
        buffer::BufferData,
        slice::SoaSlice,
        traits::{AllocSoa, AllocSoaTrusted},
    };

    pub trait Sealed {}

    impl<T> Sealed for *const SoaSlice<T> where T: AllocSoaTrusted + ?Sized {}
    impl<T> Sealed for *mut SoaSlice<T> where T: AllocSoaTrusted + ?Sized {}

    impl<T> Sealed for *const BufferData<T> where T: AllocSoa + ?Sized {}
    impl<T> Sealed for *mut BufferData<T> where T: AllocSoa + ?Sized {}
}
