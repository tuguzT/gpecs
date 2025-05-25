use core::{
    alloc::{Layout, LayoutError},
    iter::FusedIterator,
};

/// Iterator of offsets for each provided region in a single buffer.
///
/// Resulting layout could be retrieved using [`layout()`](BufferOffsets::layout()) method.
pub struct BufferOffsets<I> {
    layout: Layout,
    regions: I,
}

impl<I> BufferOffsets<I> {
    /// Layout of a buffer needed to store all regions processed by iterator.
    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }
}

impl<I> Iterator for BufferOffsets<I>
where
    I: Iterator<Item = Result<Layout, LayoutError>>,
{
    type Item = Result<usize, LayoutError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { layout, regions } = self;

        let region = regions.next()?;
        let offset = region.and_then(|region| {
            let offset;
            (*layout, offset) = layout.extend(region)?;
            Ok(offset)
        });
        Some(offset)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { regions, .. } = self;
        regions.size_hint()
    }
}

impl<I> ExactSizeIterator for BufferOffsets<I>
where
    I: Iterator<Item = Result<Layout, LayoutError>> + ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { regions, .. } = self;
        regions.len()
    }
}

impl<I> FusedIterator for BufferOffsets<I> where
    I: Iterator<Item = Result<Layout, LayoutError>> + FusedIterator
{
}

/// Calculates offsets for each provided region in a single buffer.
#[inline]
pub fn buffer_offsets<I>(regions: I) -> BufferOffsets<I::IntoIter>
where
    I: IntoIterator<Item = Result<Layout, LayoutError>>,
{
    BufferOffsets {
        layout: Layout::new::<()>(),
        regions: regions.into_iter(),
    }
}

/// Calculates layout needed to store provided regions in a single buffer.
#[inline]
pub fn buffer_layout<I>(regions: I) -> Result<Layout, LayoutError>
where
    I: IntoIterator<Item = Result<Layout, LayoutError>>,
{
    let mut offsets = buffer_offsets(regions);
    offsets.by_ref().try_for_each(|offset| offset.map(drop))?;
    Ok(offsets.layout())
}

/// Copy of [`Layout::repeat()`] functionality which could be used on stable channel.
#[inline]
pub fn repeat_layout(layout: Layout, n: usize) -> Result<Layout, LayoutError> {
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
