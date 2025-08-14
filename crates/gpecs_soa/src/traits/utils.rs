use core::{
    alloc::{Layout, LayoutError},
    iter::FusedIterator,
};

use super::FieldDescriptor;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CopiedFieldDescriptors<T>(pub T)
where
    T: ?Sized;

impl<T> CopiedFieldDescriptors<T> {
    #[inline]
    pub fn into_inner(self) -> T {
        let Self(inner) = self;
        inner
    }
}

impl<T> From<T> for CopiedFieldDescriptors<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Iterator for CopiedFieldDescriptors<T>
where
    T: Iterator + ?Sized,
    T::Item: AsRef<FieldDescriptor>,
{
    type Item = FieldDescriptor;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self(inner) = self;
        inner.next().map(|desc| *desc.as_ref())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self(inner) = self;
        inner.size_hint()
    }
}

impl<T> DoubleEndedIterator for CopiedFieldDescriptors<T>
where
    T: DoubleEndedIterator + ?Sized,
    T::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self(inner) = self;
        inner.next_back().map(|desc| *desc.as_ref())
    }
}

impl<T> ExactSizeIterator for CopiedFieldDescriptors<T>
where
    T: ExactSizeIterator + ?Sized,
    T::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self(inner) = self;
        inner.len()
    }
}

impl<T> FusedIterator for CopiedFieldDescriptors<T>
where
    T: FusedIterator + ?Sized,
    T::Item: AsRef<FieldDescriptor>,
{
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
    fields: CopiedFieldDescriptors<I>,
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
        fields.into_inner()
    }
}

impl<I> Iterator for BufferOffsets<I>
where
    I: Iterator + ?Sized,
    I::Item: AsRef<FieldDescriptor>,
{
    type Item = Result<(FieldDescriptor, usize), LayoutError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut fields,
            ref mut layout,
            capacity,
        } = *self;

        let desc = fields.next()?;
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
    I: ExactSizeIterator + ?Sized,
    I::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { fields, .. } = self;
        fields.len()
    }
}

impl<I> FusedIterator for BufferOffsets<I>
where
    I: FusedIterator + ?Sized,
    I::Item: AsRef<FieldDescriptor>,
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
        fields: fields.into_iter().into(),
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
