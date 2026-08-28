use core::{
    alloc::{Layout, LayoutError},
    error::Error,
    fmt::{self, Display},
    marker::PhantomData,
    mem::{ManuallyDrop, offset_of},
    ptr,
};

use crate::traits::{AllocSoa, AllocSoaContext, AllocSoaTrusted, MutPtrs, Ptrs, RawSoaContext};

#[cfg(test)]
mod tests;

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

#[repr(C)]
pub struct BufferPrefix<T>
where
    T: AllocSoa + ?Sized,
{
    pub context: T::Context,
    pub len: usize,
    pub capacity: usize,
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

#[inline]
pub fn layout_is_dangling(layout: Layout) -> bool {
    layout.size() == 0
}

#[inline]
pub fn fields_are_zst<T>(context: &T::Context) -> bool
where
    T: AllocSoa + ?Sized,
{
    context.packed_size_of_fields() == Some(0)
}

#[inline]
pub fn buffer_align<T>(context: &T::Context) -> usize
where
    T: AllocSoa + ?Sized,
{
    context.buffer_align().max(align_of::<BufferAlign<T>>())
}

#[inline]
pub fn buffer_layout<T>(context: &T::Context, capacity: usize) -> Result<Layout, LayoutError>
where
    T: AllocSoa + ?Sized,
{
    let layout = buffer_layout_inner::<T>(context, capacity)?;

    let item_layout = Layout::new::<BufferData<T>>();
    let layout = fit_layout_in_array(layout, item_layout)?;

    let align = buffer_align::<T>(context);
    layout.align_to(align)
}

fn fit_layout_in_array(layout: Layout, item_layout: Layout) -> Result<Layout, LayoutError> {
    if layout_is_dangling(item_layout) {
        return layout.align_to(item_layout.align());
    }

    let item_layout = item_layout.pad_to_align();
    let size = layout
        .size()
        .checked_next_multiple_of(item_layout.size())
        .unwrap_or(usize::MAX);
    let align = usize::max(layout.align(), item_layout.align());
    Layout::from_size_align(size, align)
}

fn buffer_layout_inner<T>(context: &T::Context, capacity: usize) -> Result<Layout, LayoutError>
where
    T: AllocSoa + ?Sized,
{
    let buffer_layout = context.buffer_layout(capacity)?;
    let prefix = Layout::new::<BufferPrefix<T>>();
    let Some(buffer_layout) = buffer_layout_with_prefix(buffer_layout, prefix) else {
        let n = (size_of_val(context) != 0).into();
        return prefix.repeat_packed(n);
    };

    let BufferLayoutWithPrefix { layout, .. } = buffer_layout?;
    Ok(layout)
}

struct BufferLayoutWithPrefix {
    layout: Layout,
    buffer_offset: usize,
}

impl BufferLayoutWithPrefix {
    fn new(layout: Layout, buffer_offset: usize) -> Self {
        Self {
            layout,
            buffer_offset,
        }
    }
}

fn buffer_layout_with_prefix(
    buffer_layout: Layout,
    prefix: Layout,
) -> Option<Result<BufferLayoutWithPrefix, LayoutError>> {
    if layout_is_dangling(buffer_layout) {
        return None;
    }

    let buffer_layout = prefix
        .extend(buffer_layout)
        .map(|(layout, buffer_offset)| BufferLayoutWithPrefix::new(layout, buffer_offset));
    Some(buffer_layout)
}

#[inline]
pub fn capacity_from<T>(context: &T::Context, buffer_layout: Layout) -> usize
where
    T: AllocSoa + ?Sized,
{
    let prefix = Layout::new::<BufferPrefix<T>>();
    let align = context.buffer_align();
    let Some(buffer_layout) = buffer_layout_without_prefix(buffer_layout, prefix, align) else {
        return 0;
    };

    context.capacity_from(buffer_layout)
}

fn buffer_layout_without_prefix(
    buffer_layout: Layout,
    prefix: Layout,
    align: usize,
) -> Option<Layout> {
    let offset_to_data = prefix.align_to(align).ok()?.pad_to_align().size();
    let size = buffer_layout.size().checked_sub(offset_to_data)?;

    let buffer_layout = Layout::from_size_align(size, buffer_layout.align())
        .expect("layout with size smaller than the buffer one should be valid");
    Some(buffer_layout)
}

#[inline]
#[cfg_attr(not(feature = "alloc"), expect(unused))]
pub fn buffer_layout_capacity<T>(
    context: &T::Context,
    capacity: usize,
) -> Result<(Layout, usize), LayoutError>
where
    T: AllocSoa + ?Sized,
{
    let buffer_layout = buffer_layout::<T>(context, capacity)?;
    let capacity = capacity_from::<T>(context, buffer_layout);
    Ok((buffer_layout, capacity))
}

#[inline]
pub const unsafe fn ptr_to_buffer_context<T>(buffer: *const u8) -> *const T::Context
where
    T: AllocSoa + ?Sized,
{
    const { assert_buffer_context::<T>() }
    buffer.cast()
}

#[inline]
#[cfg_attr(not(feature = "alloc"), expect(unused))]
pub const unsafe fn ptr_to_buffer_context_mut<T>(buffer: *mut u8) -> *mut T::Context
where
    T: AllocSoa + ?Sized,
{
    const { assert_buffer_context::<T>() }
    buffer.cast()
}

const fn assert_buffer_context<T>()
where
    T: AllocSoa + ?Sized,
{
    assert!(
        offset_of!(BufferPrefix<T>, context) == 0,
        "buffer prefix should always start with SoA context",
    );
}

#[inline]
#[expect(unused)]
pub unsafe fn ptr_to_buffer_prefix<T>(
    context: &T::Context,
    capacity: usize,
    buffer: *const u8,
) -> Result<Option<*const BufferPrefix<T>>, LayoutError>
where
    T: AllocSoa + ?Sized,
{
    let buffer_layout = buffer_layout::<T>(context, capacity)?;
    if layout_is_dangling(buffer_layout) {
        return Ok(None);
    }

    let prefix = unsafe { ptr_to_buffer_prefix_unchecked::<T>(buffer) };
    Ok(Some(prefix))
}

#[inline]
#[cfg_attr(not(feature = "alloc"), expect(unused))]
pub unsafe fn ptr_to_buffer_prefix_mut<T>(
    context: &T::Context,
    capacity: usize,
    buffer: *mut u8,
) -> Result<Option<*mut BufferPrefix<T>>, LayoutError>
where
    T: AllocSoa + ?Sized,
{
    let buffer_layout = buffer_layout::<T>(context, capacity)?;
    if layout_is_dangling(buffer_layout) {
        return Ok(None);
    }

    let prefix = unsafe { ptr_to_buffer_prefix_unchecked_mut::<T>(buffer) };
    Ok(Some(prefix))
}

#[inline]
pub const unsafe fn ptr_to_buffer_prefix_unchecked<T>(buffer: *const u8) -> *const BufferPrefix<T>
where
    T: AllocSoa + ?Sized,
{
    buffer.cast()
}

#[inline]
pub const unsafe fn ptr_to_buffer_prefix_unchecked_mut<T>(buffer: *mut u8) -> *mut BufferPrefix<T>
where
    T: AllocSoa + ?Sized,
{
    buffer.cast()
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
    let buffer = unsafe { ptr_to_buffer_data::<T>(context, ptr, capacity) };
    let Ok(buffer) = buffer else {
        return context.ptrs_dangling();
    };

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
    let buffer = unsafe { ptr_to_buffer_data_mut::<T>(context, ptr, capacity) };
    let Ok(buffer) = buffer else {
        return context.ptrs_dangling_mut();
    };

    unsafe { context.ptrs_from_buffer_mut(buffer, capacity) }
}

#[inline]
pub unsafe fn ptr_to_buffer_data<T>(
    context: &T::Context,
    ptr: *const u8,
    capacity: usize,
) -> Result<*const u8, PtrToDataError>
where
    T: AllocSoa + ?Sized,
{
    let offset = offset_to_buffer_data::<T>(context, capacity)?;
    let data = unsafe { ptr.add(offset) };
    Ok(data)
}

#[inline]
pub unsafe fn ptr_to_buffer_data_mut<T>(
    context: &T::Context,
    ptr: *mut u8,
    capacity: usize,
) -> Result<*mut u8, PtrToDataError>
where
    T: AllocSoa + ?Sized,
{
    let offset = offset_to_buffer_data::<T>(context, capacity)?;
    let data = unsafe { ptr.add(offset) };
    Ok(data)
}

fn offset_to_buffer_data<T>(context: &T::Context, capacity: usize) -> Result<usize, PtrToDataError>
where
    T: AllocSoa + ?Sized,
{
    let buffer_layout = context.buffer_layout(capacity)?;
    let prefix = Layout::new::<BufferPrefix<T>>();
    let Some(buffer_layout) = buffer_layout_with_prefix(buffer_layout, prefix) else {
        let error = DanglingError(());
        return Err(error.into());
    };

    let BufferLayoutWithPrefix { buffer_offset, .. } = buffer_layout?;
    Ok(buffer_offset)
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
        let context = unsafe { ptr_to_buffer_context::<T>(data).as_ref_unchecked() };
        let len = Self::len_of_inner(context, len, capacity);
        let inner = ptr::slice_from_raw_parts(data.cast(), len);
        Self::ptr_from_inner(inner)
    }

    #[inline]
    pub unsafe fn ptr_from_raw_parts_mut(data: *mut u8, len: usize, capacity: usize) -> *mut Self {
        let context = unsafe { ptr_to_buffer_context::<T>(data).as_ref_unchecked() };
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
        let buffer_layout = buffer_layout::<T>(context, capacity)
            .expect("layout size should not exceed `isize::MAX`");

        if layout_is_dangling(buffer_layout) {
            return len;
        }
        buffer_layout.size() / size_of::<BufferData<T>>()
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
    pub unsafe fn ptr_to_context(this: *const Self) -> *const T::Context {
        let buffer = Self::ptr_as_ptr(this);
        unsafe { ptr_to_buffer_context::<T>(buffer) }
    }

    #[inline]
    pub unsafe fn ptr_to_len(this: *const Self) -> Option<*const usize> {
        let prefix = unsafe { Self::ptr_to_prefix(this) }?;
        let len = unsafe { &raw const (*prefix).len };
        Some(len)
    }

    #[inline]
    pub unsafe fn ptr_to_capacity(this: *const Self) -> Option<*const usize> {
        let prefix = unsafe { Self::ptr_to_prefix(this) }?;
        let capacity = unsafe { &raw const (*prefix).capacity };
        Some(capacity)
    }

    unsafe fn ptr_to_prefix(this: *const Self) -> Option<*const BufferPrefix<T>> {
        if Self::ptr_is_dangling(this) {
            return None;
        }

        let buffer = Self::ptr_as_ptr(this);
        let prefix = unsafe { ptr_to_buffer_prefix_unchecked::<T>(buffer) };
        Some(prefix)
    }

    fn ptr_is_dangling(this: *const Self) -> bool {
        size_of::<BufferData<T>>() == 0 || Self::ptr_as_inner(this).len() == 0
    }

    #[inline]
    pub unsafe fn ptr_len(this: *const Self) -> usize {
        let len = unsafe { Self::ptr_to_len(this) };
        let Some(len) = len else {
            return Self::ptr_as_inner(this).len();
        };
        unsafe { len.read() }
    }

    #[inline]
    pub unsafe fn ptr_capacity(this: *const Self) -> usize {
        let context = unsafe { Self::ptr_to_context(this).as_ref_unchecked() };
        if fields_are_zst::<T>(context) {
            return usize::MAX;
        }

        let capacity = unsafe { Self::ptr_to_capacity(this) };
        let Some(capacity) = capacity else {
            return 0;
        };
        unsafe { capacity.read() }
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
        unsafe { Self::ptr_to_context(this).as_ref_unchecked() }
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
