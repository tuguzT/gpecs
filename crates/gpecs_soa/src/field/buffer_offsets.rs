use core::{
    alloc::{Layout, LayoutError},
    iter::FusedIterator,
};

use crate::{
    field::{CopiedFieldDescriptors, FieldDescriptor},
    layout::repeat_packed,
};

#[derive(Debug, Clone, Copy)]
pub struct BufferOffset {
    /// Descriptor of the processed field.
    pub desc: FieldDescriptor,
    /// Offset from the start of the buffer, in bytes.
    pub offset: usize,
}

impl BufferOffset {
    #[inline]
    pub const fn new(desc: FieldDescriptor, offset: usize) -> Self {
        Self { desc, offset }
    }
}

/// Iterator of offsets for each provided field in a single buffer of provided capacity.
///
/// Resulting layout could be retrieved using [`layout()`](BufferOffsets::layout()) method.
#[derive(Debug, Clone)]
pub struct BufferOffsets<I>
where
    I: ?Sized,
{
    state: RawBufferOffsets,
    inner: CopiedFieldDescriptors<I>,
}

impl<I> BufferOffsets<I>
where
    I: ?Sized,
{
    #[inline]
    pub const fn state(&self) -> RawBufferOffsets {
        let Self { state, .. } = *self;
        state
    }

    /// Retrieves layout of a buffer needed to store all fields processed by self.
    #[inline]
    pub const fn layout(&self) -> Layout {
        let Self { state, .. } = self;
        state.layout()
    }

    /// Capacity of a buffer needed to store all fields processed by self.
    #[inline]
    pub const fn capacity(&self) -> usize {
        let Self { state, .. } = self;
        state.capacity()
    }

    /// Returns a reference to the iterator over all fields to be processed by self.
    #[inline]
    pub const fn as_inner(&self) -> &I {
        let Self { inner, .. } = self;
        inner.as_inner()
    }
}

impl<I> BufferOffsets<I> {
    /// Creates a new buffer offsets iterator from its parts.
    #[inline]
    pub unsafe fn from_parts(state: RawBufferOffsets, fields: I) -> Self {
        let inner = fields.into();
        Self { state, inner }
    }

    /// Returns an iterator over all fields to be processed by self.
    #[inline]
    pub fn into_inner(self) -> I {
        let (_, inner) = self.into_parts();
        inner
    }

    /// Turns self into its parts.
    #[inline]
    pub fn into_parts(self) -> (RawBufferOffsets, I) {
        let Self { state, inner } = self;
        (state, inner.into_inner())
    }

    /// Turns self into layout of a buffer needed to store all fields processed by self.
    #[inline]
    pub fn into_layout(self) -> Layout {
        self.layout()
    }
}

impl<I> BufferOffsets<I>
where
    I: Iterator + ?Sized,
    I::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    pub unsafe fn next_unchecked(&mut self) -> BufferOffset {
        let Self { inner, state } = self;

        let desc = unsafe { inner.next().unwrap_unchecked() };
        let offset = unsafe { state.next_unchecked(desc) };
        BufferOffset::new(desc, offset)
    }
}

impl<I> Iterator for BufferOffsets<I>
where
    I: Iterator + ?Sized,
    I::Item: AsRef<FieldDescriptor>,
{
    type Item = Result<BufferOffset, LayoutError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, state } = self;

        let desc = inner.next()?;
        let item = state
            .next(desc)
            .map(|offset| BufferOffset::new(desc, offset));
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }
}

impl<I> ExactSizeIterator for BufferOffsets<I>
where
    I: ExactSizeIterator + ?Sized,
    I::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }
}

impl<I> FusedIterator for BufferOffsets<I>
where
    I: FusedIterator + ?Sized,
    I::Item: AsRef<FieldDescriptor>,
{
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RawBufferOffsets {
    layout: Layout,
    capacity: usize,
}

impl RawBufferOffsets {
    #[inline]
    pub const fn new(capacity: usize) -> Self {
        let layout = Layout::new::<()>();
        Self::from_parts(layout, capacity)
    }

    #[inline]
    pub const fn from_parts(layout: Layout, capacity: usize) -> Self {
        Self { layout, capacity }
    }

    #[inline]
    pub const fn layout(self) -> Layout {
        let Self { layout, .. } = self;
        layout
    }

    #[inline]
    pub const fn capacity(self) -> usize {
        let Self { capacity, .. } = self;
        capacity
    }

    #[inline]
    pub const fn into_parts(self) -> (Layout, usize) {
        let Self { layout, capacity } = self;
        (layout, capacity)
    }

    #[inline]
    pub const fn next(&mut self, desc: FieldDescriptor) -> Result<usize, LayoutError> {
        let Self {
            ref mut layout,
            capacity,
        } = *self;

        let padded_layout = desc.layout().pad_to_align();
        let next = match repeat_packed(padded_layout, capacity) {
            Ok(layout) => layout,
            Err(error) => return Err(error),
        };

        let offset;
        (*layout, offset) = match layout.extend(next) {
            Ok(ok) => ok,
            Err(error) => return Err(error),
        };

        Ok(offset)
    }

    #[inline]
    pub const unsafe fn next_unchecked(&mut self, desc: FieldDescriptor) -> usize {
        let Self {
            ref mut layout,
            capacity,
        } = *self;

        let padded_layout = desc.layout().pad_to_align();
        let next = unsafe { repeat_packed_unchecked(padded_layout, capacity) };

        let offset;
        (*layout, offset) = unsafe { extend_unchecked(*layout, next) };

        offset
    }
}

/// Unchecked copy of [`Layout::repeat_packed()`] on stable channel.
#[inline]
const unsafe fn repeat_packed_unchecked(layout: Layout, n: usize) -> Layout {
    // FIXME: use `unchecked_mul` instead
    let size = layout.size().wrapping_mul(n);
    unsafe { Layout::from_size_align_unchecked(size, layout.align()) }
}

/// Copy of [`Layout::extend()`] which Rust-GPU struggles to inline by itself.
#[inline]
const unsafe fn extend_unchecked(layout: Layout, next: Layout) -> (Layout, usize) {
    let new_align = usize_max(layout.align(), next.align());
    let offset = unsafe {
        let align_m1 = next.align().unchecked_sub(1);
        layout.size().unchecked_add(align_m1) & !align_m1
    };
    let new_size = unsafe { offset.unchecked_add(next.size()) };

    let layout = unsafe { Layout::from_size_align_unchecked(new_size, new_align) };
    (layout, offset)
}

/// The same as [`usize::max()`], but usable in const context.
#[inline]
const fn usize_max(lhs: usize, rhs: usize) -> usize {
    if rhs < lhs { lhs } else { rhs }
}
