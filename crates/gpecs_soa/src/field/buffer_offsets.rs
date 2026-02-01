use core::{
    alloc::{Layout, LayoutError},
    iter::FusedIterator,
};

use crate::field::{CopiedFieldDescriptors, FieldDescriptor, repeat_layout};

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
        let item = try_create_buffer_offset(desc, layout, capacity);
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
fn try_create_buffer_offset(
    desc: FieldDescriptor,
    total_layout: &mut Layout,
    capacity: usize,
) -> Result<BufferOffset, LayoutError> {
    let layout = repeat_layout(desc.layout(), capacity)?;

    let offset;
    (*total_layout, offset) = total_layout.extend(layout)?;

    Ok(BufferOffset {
        desc,
        layout,
        offset,
    })
}
