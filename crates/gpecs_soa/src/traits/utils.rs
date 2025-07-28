use core::{
    alloc::{Layout, LayoutError},
    iter::FusedIterator,
};

use super::FieldDescriptor;

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
    fields: I,
}

impl<I> BufferOffsets<I>
where
    I: ?Sized,
{
    /// Layout of a buffer needed to store all fields processed by self.
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
}

impl<I> BufferOffsets<I> {
    /// Returns an iterator over all fields to be processed by self.
    #[inline]
    pub fn into_fields(self) -> I {
        let Self { fields, .. } = self;
        fields
    }
}

impl<I> Iterator for BufferOffsets<I>
where
    I: ?Sized + Iterator<Item: AsRef<FieldDescriptor>>,
{
    type Item = Result<(FieldDescriptor, usize), LayoutError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut fields,
            ref mut layout,
            capacity,
        } = *self;

        let desc = *fields.next()?.as_ref();
        let region = repeat_layout(desc.layout(), capacity);
        let offset = region.and_then(|region| {
            let offset;
            (*layout, offset) = layout.extend(region)?;
            Ok(offset)
        });
        let item = offset.map(|offset| (desc, offset));
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { fields, .. } = self;
        fields.size_hint()
    }
}

impl<I> ExactSizeIterator for BufferOffsets<I>
where
    I: ?Sized + Iterator<Item: AsRef<FieldDescriptor>> + ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { fields, .. } = self;
        fields.len()
    }
}

impl<I> FusedIterator for BufferOffsets<I> where
    I: ?Sized + Iterator<Item: AsRef<FieldDescriptor>> + FusedIterator
{
}

/// Calculates offsets for each provided region in a single buffer.
#[inline]
pub fn buffer_offsets<I>(fields: I, capacity: usize) -> BufferOffsets<I::IntoIter>
where
    I: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    BufferOffsets {
        layout: Layout::new::<()>(),
        fields: fields.into_iter(),
        capacity,
    }
}

/// Calculates layout needed to store provided regions in a single buffer.
#[inline]
pub fn buffer_layout<I>(fields: I, capacity: usize) -> Result<Layout, LayoutError>
where
    I: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    let mut offsets = buffer_offsets(fields, capacity);
    offsets.by_ref().try_for_each(|offset| offset.map(drop))?;
    Ok(offsets.layout())
}

/// Copy of [`Layout::repeat()`] functionality which could be used on stable channel.
#[inline]
pub const fn repeat_layout(layout: Layout, n: usize) -> Result<Layout, LayoutError> {
    const ERR: LayoutError = match Layout::from_size_align(usize::MAX, 1) {
        Ok(_) => unreachable!(),
        Err(err) => err,
    };

    let layout = layout.pad_to_align();
    let size = match layout.size().checked_mul(n) {
        Some(v) => v,
        None => return Err(ERR),
    };
    Layout::from_size_align(size, layout.align())
}
