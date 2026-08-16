use core::{
    alloc::{Layout, LayoutError},
    error::Error,
    fmt::{self, Display},
    marker::PhantomData,
    mem::{ManuallyDrop, offset_of},
    ptr::{self, NonNull},
};

use crate::{
    field::FieldLayouts,
    layout::WithLayout,
    traits::{AllocSoa, AllocSoaContext, AllocSoaTrusted, MutPtrs, Ptrs, RawSoa, RawSoaContext},
};

#[cfg(test)]
mod tests;

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

#[repr(transparent)]
pub struct BufferDropCheck<T>(PhantomData<(T::Fields, T::Context)>)
where
    T: AllocSoa + ?Sized;

impl<T> Default for BufferDropCheck<T>
where
    T: AllocSoa + ?Sized,
{
    #[inline]
    fn default() -> Self {
        Self(PhantomData)
    }
}

union BufferData<T>
where
    T: AllocSoa + ?Sized,
{
    _align: ManuallyDrop<BufferAlign<T>>,
    _fields: ManuallyDrop<T::Fields>,
    _context: ManuallyDrop<T::Context>,
    _marker: ManuallyDrop<BufferDropCheck<T>>,
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
    let layout = buffer_layout_inner::<T>(context, capacity)?;

    let item_size = size_of::<BufferData<T>>();
    if item_size == 0 {
        return Ok(layout);
    }

    let size = layout
        .size()
        .checked_next_multiple_of(item_size)
        .unwrap_or(usize::MAX);
    Layout::from_size_align(size, layout.align())
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
    let offset_to_data = prefix
        .align_to(align)
        .expect("buffer layout is valid, so the alignment of prefix to the data should be")
        .pad_to_align()
        .size();

    let size = buffer_layout.size() - offset_to_data;
    let buffer_layout = Layout::from_size_align(size, buffer_layout.align())
        .expect("layout with size smaller than the buffer one should be valid");
    context.capacity_from(buffer_layout)
}

#[inline]
pub fn buffer_align<T>(context: &T::Context) -> usize
where
    T: AllocSoa + ?Sized,
{
    let align = align_of_fields(context.field_layouts());
    usize::max(align, align_of::<BufferData<T>>())
}

#[inline]
#[cfg_attr(not(feature = "alloc"), expect(unused))]
pub fn buffer_dangling<T>(context: &T::Context) -> NonNull<u8>
where
    T: AllocSoa + ?Sized,
{
    let addr = buffer_align::<T>(context)
        .try_into()
        .expect("alignment cannot be zero");
    NonNull::without_provenance(addr)
}

#[inline]
pub unsafe fn ptrs_from_buffer<T>(
    context: &T::Context,
    ptr: *const u8,
    capacity: usize,
) -> Ptrs<'_, T>
where
    T: AllocSoa + ?Sized,
{
    if fields_are_dangling::<T>(context, capacity) {
        return context.ptrs_dangling();
    }

    let buffer = unsafe { ptr_to_data::<T>(context, ptr, capacity).unwrap_unchecked() };
    unsafe { context.ptrs_from_buffer(buffer, capacity) }
}

#[inline]
pub unsafe fn ptrs_from_buffer_mut<T>(
    context: &T::Context,
    ptr: *mut u8,
    capacity: usize,
) -> MutPtrs<'_, T>
where
    T: AllocSoa + ?Sized,
{
    if fields_are_dangling::<T>(context, capacity) {
        return context.ptrs_dangling_mut();
    }

    let buffer = unsafe { ptr_to_data_mut::<T>(context, ptr, capacity).unwrap_unchecked() };
    unsafe { context.ptrs_from_buffer_mut(buffer, capacity) }
}

#[inline]
pub unsafe fn ptr_to_data<T>(
    context: &T::Context,
    ptr: *const u8,
    capacity: usize,
) -> Result<*const u8, PtrToDataError>
where
    T: AllocSoa + ?Sized,
{
    let offset = offset_to_data::<T>(context, capacity)?;
    let data = unsafe { ptr.add(offset) };
    Ok(data)
}

#[inline]
pub unsafe fn ptr_to_data_mut<T>(
    context: &T::Context,
    ptr: *mut u8,
    capacity: usize,
) -> Result<*mut u8, PtrToDataError>
where
    T: AllocSoa + ?Sized,
{
    let offset = offset_to_data::<T>(context, capacity)?;
    let data = unsafe { ptr.add(offset) };
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

    let buffer = context.buffer_layout(capacity)?;
    let prefix = Layout::new::<BufferPrefix<T>>();
    let (_, offset) = prefix.extend(buffer)?;
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

#[repr(transparent)]
pub struct DstBuffer<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    inner: [BufferData<T>],
}

impl<T> DstBuffer<T>
where
    T: AllocSoaTrusted + ?Sized,
{
    #[inline]
    pub unsafe fn ptr_from_raw_parts(data: *const u8, len: usize, capacity: usize) -> *const Self {
        let context = unsafe { ptr_to_context::<T>(data).as_ref_unchecked() };
        let len = Self::len_of_inner(context, len, capacity);
        let inner = ptr::slice_from_raw_parts(data.cast(), len);
        Self::ptr_from_inner(inner)
    }

    #[inline]
    pub unsafe fn ptr_from_raw_parts_mut(data: *mut u8, len: usize, capacity: usize) -> *mut Self {
        let context = unsafe { ptr_to_context::<T>(data).as_ref_unchecked() };
        let len = Self::len_of_inner(context, len, capacity);
        let inner = ptr::slice_from_raw_parts_mut(data.cast(), len);
        Self::ptr_from_inner_mut(inner)
    }

    fn ptr_from_inner(inner: *const [BufferData<T>]) -> *const Self {
        // Self is transparent over a slice of `BufferData<T>`
        inner as *const Self
    }

    fn ptr_from_inner_mut(inner: *mut [BufferData<T>]) -> *mut Self {
        // Self is transparent over a slice of `BufferData<T>`
        inner as *mut Self
    }

    fn len_of_inner(context: &T::Context, len: usize, capacity: usize) -> usize {
        if buffer_is_dangling::<T>(context, capacity) {
            return len;
        }

        let capacity_in_bytes = buffer_layout::<T>(context, capacity)
            .expect("layout size should not exceed `isize::MAX`")
            .size();
        capacity_in_bytes / size_of::<BufferData<T>>()
    }

    #[inline]
    pub fn ptr_as_ptr(this: *const Self) -> *const u8 {
        Self::ptr_as_inner(this).cast()
    }

    #[inline]
    pub fn ptr_as_mut_ptr(this: *mut Self) -> *mut u8 {
        Self::ptr_as_inner_mut(this).cast()
    }

    fn ptr_as_inner(this: *const Self) -> *const [BufferData<T>] {
        // Self is transparent over a slice of `BufferData<T>`
        this as *const [BufferData<T>]
    }

    fn ptr_as_inner_mut(this: *mut Self) -> *mut [BufferData<T>] {
        // Self is transparent over a slice of `BufferData<T>`
        this as *mut [BufferData<T>]
    }

    #[inline]
    pub unsafe fn ptr_len(this: *const Self) -> usize {
        if Self::ptr_is_dangling(this) {
            return Self::ptr_as_inner(this).len();
        }
        unsafe { ptr_to_len::<T>(Self::ptr_as_ptr(this)).read() }
    }

    #[inline]
    pub unsafe fn ptr_capacity(this: *const Self) -> usize {
        let context = unsafe { ptr_to_context::<T>(Self::ptr_as_ptr(this)).as_ref_unchecked() };
        if fields_are_zst::<T>(context) {
            return usize::MAX;
        }
        if Self::ptr_is_dangling(this) {
            return 0;
        }
        unsafe { ptr_to_capacity::<T>(Self::ptr_as_ptr(this)).read() }
    }

    fn ptr_is_dangling(this: *const Self) -> bool {
        size_of::<BufferData<T>>() == 0 || Self::ptr_as_inner(this).len() == 0
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { inner } = self;
        inner.as_ptr().cast()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { inner } = self;
        inner.as_mut_ptr().cast()
    }

    #[inline]
    pub fn context(&self) -> &T::Context {
        let this = ptr::from_ref(self);
        unsafe { ptr_to_context::<T>(Self::ptr_as_ptr(this)).as_ref_unchecked() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let this = ptr::from_ref(self);
        unsafe { Self::ptr_len(this) }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let this = ptr::from_ref(self);
        unsafe { Self::ptr_capacity(this) }
    }
}

#[inline]
pub unsafe fn ptr_to_context<T>(buffer: *const u8) -> *const T::Context
where
    T: AllocSoa + ?Sized,
{
    let offset = offset_of!(BufferPrefix<T>, context);
    let ptr = unsafe { buffer.add(offset) };
    ptr.cast()
}

#[inline]
pub unsafe fn ptr_to_len<T>(buffer: *const u8) -> *const usize
where
    T: AllocSoa + ?Sized,
{
    let offset = offset_of!(BufferPrefix<T>, len);
    let ptr = unsafe { buffer.add(offset) };
    ptr.cast()
}

#[inline]
pub unsafe fn ptr_to_capacity<T>(buffer: *const u8) -> *const usize
where
    T: AllocSoa + ?Sized,
{
    let offset = offset_of!(BufferPrefix<T>, capacity);
    let ptr = unsafe { buffer.add(offset) };
    ptr.cast()
}

#[inline]
#[cfg_attr(not(feature = "alloc"), expect(unused))]
pub unsafe fn ptr_to_context_mut<T>(buffer: *mut u8) -> *mut T::Context
where
    T: AllocSoa + ?Sized,
{
    let offset = offset_of!(BufferPrefix<T>, context);
    let ptr = unsafe { buffer.add(offset) };
    ptr.cast()
}

#[inline]
#[cfg_attr(not(feature = "alloc"), expect(unused))]
pub unsafe fn ptr_to_len_mut<T>(buffer: *mut u8) -> *mut usize
where
    T: AllocSoa + ?Sized,
{
    let offset = offset_of!(BufferPrefix<T>, len);
    let ptr = unsafe { buffer.add(offset) };
    ptr.cast()
}

#[inline]
#[cfg_attr(not(feature = "alloc"), expect(unused))]
pub unsafe fn ptr_to_capacity_mut<T>(buffer: *mut u8) -> *mut usize
where
    T: AllocSoa + ?Sized,
{
    let offset = offset_of!(BufferPrefix<T>, capacity);
    let ptr = unsafe { buffer.add(offset) };
    ptr.cast()
}
