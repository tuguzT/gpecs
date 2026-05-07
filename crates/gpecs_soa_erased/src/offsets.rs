use core::{alloc::Layout, num::NonZeroUsize};

use crate::soa::{
    field::{BufferLayout, BufferOffset},
    layout::WithLayout,
};

pub unsafe trait BufferOffsetsFrom<T> {
    unsafe fn next(&mut self, capacity: usize, desc: T) -> BufferOffset<T>;
}

unsafe impl<T, U> BufferOffsetsFrom<T> for &mut U
where
    U: BufferOffsetsFrom<T> + ?Sized,
{
    #[inline]
    unsafe fn next(&mut self, capacity: usize, desc: T) -> BufferOffset<T> {
        unsafe { (**self).next(capacity, desc) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferOffsetsFromLayout {
    total_layout: Layout,
}

impl Default for BufferOffsetsFromLayout {
    #[inline]
    fn default() -> Self {
        let total_layout = Layout::new::<()>();
        Self { total_layout }
    }
}

unsafe impl<T> BufferOffsetsFrom<T> for BufferOffsetsFromLayout
where
    T: WithLayout,
{
    #[inline]
    unsafe fn next(&mut self, capacity: usize, desc: T) -> BufferOffset<T> {
        let Self { total_layout } = self;

        let mut buffer = BufferLayout::from_parts(*total_layout, capacity, NonZeroUsize::MIN);
        let offset = unsafe { buffer.extend_unchecked(desc.layout()) };
        *total_layout = buffer.layout();

        BufferOffset::new(desc, offset)
    }
}
