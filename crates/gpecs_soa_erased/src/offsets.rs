use core::{alloc::Layout, num::NonZeroUsize};

use crate::soa::{
    field::{BufferLayout, BufferOffset},
    identity::Identity,
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

unsafe impl<T, U> BufferOffsetsFrom<T> for Identity<U>
where
    U: BufferOffsetsFrom<T>,
{
    #[inline]
    unsafe fn next(&mut self, capacity: usize, desc: T) -> BufferOffset<T> {
        unsafe { self.as_inner_mut().next(capacity, desc) }
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

pub type BufferOffsetsOf<T> = <T as BufferOffsetsFromSelf>::BufferOffsets;

pub unsafe trait BufferOffsetsFromSelf: Sized {
    type BufferOffsets: BufferOffsetsFrom<Self> + Default + Clone;
}

unsafe impl<T> BufferOffsetsFromSelf for Identity<T>
where
    T: BufferOffsetsFromSelf,
    T::BufferOffsets: BufferOffsetsFrom<Self>,
{
    type BufferOffsets = T::BufferOffsets;
}

unsafe impl<T> BufferOffsetsFromSelf for &T
where
    T: BufferOffsetsFromSelf,
    T::BufferOffsets: BufferOffsetsFrom<Self>,
{
    type BufferOffsets = T::BufferOffsets;
}

unsafe impl<T> BufferOffsetsFromSelf for &mut T
where
    T: BufferOffsetsFromSelf,
    T::BufferOffsets: BufferOffsetsFrom<Self>,
{
    type BufferOffsets = T::BufferOffsets;
}

unsafe impl<K, V> BufferOffsetsFromSelf for (K, V)
where
    V: BufferOffsetsFromSelf,
    V::BufferOffsets: BufferOffsetsFrom<Self>,
{
    type BufferOffsets = V::BufferOffsets;
}

unsafe impl BufferOffsetsFromSelf for Layout {
    type BufferOffsets = BufferOffsetsFromLayout;
}
