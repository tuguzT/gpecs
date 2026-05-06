use core::{alloc::LayoutError, iter::FusedIterator};

use crate::{field::BufferLayout, layout::WithLayout};

#[derive(Debug, Clone, Copy)]
pub struct BufferOffset<T> {
    /// Offset from the start of the buffer, in bytes.
    pub offset: usize,
    /// Descriptor of the processed field.
    pub desc: T,
}

impl<T> BufferOffset<T> {
    #[inline]
    pub const fn new(desc: T, offset: usize) -> Self {
        Self { offset, desc }
    }
}

/// Iterator of offsets for each provided field in a single buffer of provided capacity.
///
/// Resulting layout could be retrieved using [`buffer()`](BufferOffsets::buffer()) method.
#[derive(Debug, Clone)]
pub struct BufferOffsets<I>
where
    I: ?Sized,
{
    buffer: BufferLayout,
    inner: I,
}

impl<I> BufferOffsets<I>
where
    I: ?Sized,
{
    #[inline]
    pub const fn buffer(&self) -> BufferLayout {
        let Self { buffer, .. } = *self;
        buffer
    }

    /// Capacity of a buffer needed to store all the fields processed by self.
    #[inline]
    pub const fn capacity(&self) -> usize {
        let Self { buffer, .. } = self;
        buffer.capacity()
    }

    /// Returns a reference to the iterator over all the fields to be processed by self.
    #[inline]
    pub const fn as_inner(&self) -> &I {
        let Self { inner, .. } = self;
        inner
    }
}

impl<I> BufferOffsets<I> {
    /// Creates a new buffer offsets iterator from its parts.
    #[inline]
    pub unsafe fn from_parts(buffer: BufferLayout, inner: I) -> Self {
        Self { buffer, inner }
    }

    /// Returns an iterator over all the fields to be processed by self.
    #[inline]
    pub fn into_inner(self) -> I {
        let (_, inner) = self.into_parts();
        inner
    }

    /// Turns self into its parts.
    #[inline]
    pub fn into_parts(self) -> (BufferLayout, I) {
        let Self { buffer, inner } = self;
        (buffer, inner)
    }

    /// Turns self into layout of a buffer needed to store all the fields processed by self.
    #[inline]
    pub fn into_buffer(self) -> BufferLayout {
        self.buffer()
    }
}

impl<I> BufferOffsets<I>
where
    I: Iterator<Item: WithLayout> + ?Sized,
{
    #[inline]
    pub unsafe fn next_unchecked(&mut self) -> BufferOffset<I::Item> {
        let Self { inner, buffer } = self;

        let desc = unsafe { inner.next().unwrap_unchecked() };
        let offset = unsafe { buffer.extend_unchecked(desc.layout()) };
        BufferOffset::new(desc, offset)
    }
}

impl<I> Iterator for BufferOffsets<I>
where
    I: Iterator<Item: WithLayout> + ?Sized,
{
    type Item = Result<BufferOffset<I::Item>, LayoutError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, buffer } = self;

        let desc = inner.next()?;
        let item = buffer
            .extend(desc.layout())
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
    I: ExactSizeIterator<Item: WithLayout> + ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }
}

impl<I> FusedIterator for BufferOffsets<I> where I: FusedIterator<Item: WithLayout> + ?Sized {}
