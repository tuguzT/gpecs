use core::{
    alloc::{Layout, LayoutError},
    iter::FusedIterator,
};

use crate::field::{CopiedFieldDescriptors, FieldDescriptor};

#[derive(Debug, Clone, Copy)]
pub struct BufferOffset {
    /// Descriptor of the processed field.
    pub desc: FieldDescriptor,
    /// Layout of fields in the buffer of provided capacity.
    pub layout: Layout,
    /// Offset from the start of the buffer, in bytes.
    pub offset: usize,
}

/// Iterator of offsets for each provided field in a single buffer of provided capacity.
///
/// Resulting layout could be retrieved using [`layout()`](BufferOffsets::layout()) method.
#[derive(Debug, Clone)]
pub struct BufferOffsets<I>
where
    I: ?Sized,
{
    layout: Layout,
    capacity: usize,
    inner: CopiedFieldDescriptors<I>,
}

impl<I> BufferOffsets<I>
where
    I: ?Sized,
{
    /// Retrieves layout of a buffer needed to store all fields processed by self.
    #[inline]
    pub const fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    /// Capacity of a buffer needed to store all fields processed by self.
    #[inline]
    pub const fn capacity(&self) -> usize {
        let Self { capacity, .. } = *self;
        capacity
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
    pub unsafe fn from_parts(layout: Layout, capacity: usize, fields: I) -> Self {
        Self {
            layout,
            capacity,
            inner: fields.into(),
        }
    }

    /// Returns an iterator over all fields to be processed by self.
    #[inline]
    pub fn into_inner(self) -> I {
        let Self { inner, .. } = self;
        inner.into_inner()
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
        let Self {
            ref mut inner,
            ref mut layout,
            capacity,
        } = *self;

        let desc = unsafe { inner.next().unwrap_unchecked() };
        unsafe { buffer_offset_from_desc_unchecked(desc, layout, capacity) }
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
        let Self {
            ref mut inner,
            ref mut layout,
            capacity,
        } = *self;

        let desc = inner.next()?;
        let item = try_buffer_offset_from_desc(desc, layout, capacity);
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

#[inline]
fn try_buffer_offset_from_desc(
    desc: FieldDescriptor,
    buffer_layout: &mut Layout,
    capacity: usize,
) -> Result<BufferOffset, LayoutError> {
    let layout = desc.layout().pad_to_align();
    let layout = repeat_packed(layout, capacity)?;

    let offset;
    (*buffer_layout, offset) = buffer_layout.extend(layout)?;

    Ok(BufferOffset {
        desc,
        layout,
        offset,
    })
}

/// Copy of [`Layout::repeat_packed()`] on stable channel.
#[inline]
fn repeat_packed(layout: Layout, n: usize) -> Result<Layout, LayoutError> {
    let size = layout.size().saturating_mul(n);
    Layout::from_size_align(size, layout.align())
}

#[inline]
unsafe fn buffer_offset_from_desc_unchecked(
    desc: FieldDescriptor,
    buffer_layout: &mut Layout,
    capacity: usize,
) -> BufferOffset {
    let layout = desc.layout().pad_to_align();
    let layout = unsafe { repeat_packed_unchecked(layout, capacity) };

    let offset;
    (*buffer_layout, offset) = unsafe { extend_unchecked(*buffer_layout, layout) };

    BufferOffset {
        desc,
        layout,
        offset,
    }
}

/// Unchecked copy of [`Layout::repeat_packed()`] on stable channel.
#[inline]
unsafe fn repeat_packed_unchecked(layout: Layout, n: usize) -> Layout {
    let size = layout.size().wrapping_mul(n);
    unsafe { Layout::from_size_align_unchecked(size, layout.align()) }
}

/// Copy of [`Layout::extend()`] which Rust-GPU struggles to inline by itself.
#[inline]
unsafe fn extend_unchecked(layout: Layout, next: Layout) -> (Layout, usize) {
    let new_align = usize::max(layout.align(), next.align());
    let offset = unsafe {
        let align_m1 = next.align().unchecked_sub(1);
        layout.size().unchecked_add(align_m1) & !align_m1
    };
    let new_size = unsafe { offset.unchecked_add(next.size()) };

    let layout = unsafe { Layout::from_size_align_unchecked(new_size, new_align) };
    (layout, offset)
}
