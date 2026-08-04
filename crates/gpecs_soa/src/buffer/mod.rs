use core::{
    alloc::{Layout, LayoutError},
    error::Error,
    fmt::{self, Display},
    mem::{ManuallyDrop, offset_of},
};

use crate::{
    field::FieldLayouts,
    layout::WithLayout,
    traits::{AllocSoa, AllocSoaContext, MutPtrs, Ptrs, RawSoa, RawSoaContext},
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
pub fn packed_size_of_fields<I>(fields: I) -> usize
where
    I: IntoIterator<Item: WithLayout>,
{
    fields.into_iter().map(|item| item.layout().size()).sum()
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
    packed_size_of_fields(context.field_layouts()) == 0
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
