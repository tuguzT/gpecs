use core::alloc::{Layout, LayoutError};

pub use self::{
    buffer_offsets::{BufferOffset, BufferOffsets},
    copied_descriptors::{CopiedFieldDescriptors, IntoCopiedFieldDescriptors},
    descriptors::{
        FieldDescriptor, FieldDescriptors, FieldDescriptorsIter, FieldDescriptorsOutput,
        FieldDescriptorsOwned,
    },
};

mod buffer_offsets;
mod copied_descriptors;
mod descriptors;

/// Calculates offsets for each provided region in a single buffer.
#[inline]
pub fn buffer_offsets<I>(fields: I, capacity: usize) -> BufferOffsets<I::IntoIter>
where
    I: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    let layout = Layout::new::<()>();
    let fields = fields.into_iter();
    unsafe { BufferOffsets::from_parts(layout, capacity, fields) }
}

/// Calculates layout needed to store provided regions in a single buffer.
#[inline]
pub fn buffer_layout<I>(fields: I, capacity: usize) -> Result<Layout, LayoutError>
where
    I: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    let mut offsets = buffer_offsets(fields, capacity);
    offsets.by_ref().try_for_each(|offset| offset.map(drop))?;
    Ok(offsets.into_layout())
}

/// Copy of [`Layout::repeat()`] functionality which could be used on stable channel.
#[inline]
pub const fn repeat_layout(layout: Layout, n: usize) -> Result<Layout, LayoutError> {
    const ERR: LayoutError = match Layout::from_size_align(usize::MAX, 1) {
        Ok(_) => unreachable!(),
        Err(err) => err,
    };

    let layout = layout.pad_to_align();
    let Some(size) = layout.size().checked_mul(n) else {
        return Err(ERR);
    };
    Layout::from_size_align(size, layout.align())
}
