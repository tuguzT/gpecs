use core::{
    alloc::Layout,
    iter::FusedIterator,
    num::{NonZeroU32, TryFromIntError},
};

use bytemuck::{CheckedBitPattern, NoUninit};
use gpecs_soa_erased::{
    BufferOffsetsFrom, BufferOffsetsFromSelf, CovariantFieldLayouts,
    layout::WithLayout,
    soa::field::{BufferOffset, FieldLayouts, FieldLayoutsOutput},
};
use spirv_std::arch::IndexUnchecked;

/// Layout which consists of `u32`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, NoUninit, CheckedBitPattern)]
#[repr(C)]
pub struct GpuLayout {
    size: u32,
    align: NonZeroU32,
}

impl GpuLayout {
    /// Tries to create a new layout from the given [`Layout`].
    #[inline]
    pub fn new(layout: Layout) -> Result<Self, TryFromIntError> {
        let size = layout.size().try_into()?;
        let align = layout.align().try_into()?;

        // SAFETY: Layout::align() is guaranteed to be a power of two, which is non-zero
        let align = unsafe { NonZeroU32::new_unchecked(align) };

        let me = Self { size, align };
        Ok(me)
    }

    /// Tries to create a new layout from the given type.
    #[inline]
    pub fn of<T>() -> Result<Self, TryFromIntError> {
        let layout = Layout::new::<T>();
        Self::new(layout)
    }

    /// Returns the [`Layout`] of this layout.
    #[inline]
    pub const fn layout(self) -> Layout {
        let Self { size, align } = self;

        // SAFETY: self could only be created from a valid `Layout`
        unsafe { Layout::from_size_align_unchecked(size as usize, align.get() as usize) }
    }
}

impl TryFrom<Layout> for GpuLayout {
    type Error = TryFromIntError;

    #[inline]
    fn try_from(layout: Layout) -> Result<Self, Self::Error> {
        Self::new(layout)
    }
}

impl From<GpuLayout> for Layout {
    #[inline]
    fn from(layout: GpuLayout) -> Self {
        layout.layout()
    }
}

impl WithLayout for GpuLayout {
    #[inline]
    fn layout(&self) -> Layout {
        Self::layout(*self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, NoUninit, CheckedBitPattern)]
#[repr(C)]
pub struct GpuFieldLayout {
    layout: GpuLayout,
    offset: u32,
}

impl GpuFieldLayout {
    #[inline]
    pub unsafe fn new(layout: Layout, offset: usize) -> Result<Self, TryFromIntError> {
        let layout = GpuLayout::new(layout)?;
        let offset = offset.try_into()?;

        let me = Self { layout, offset };
        Ok(me)
    }

    #[inline]
    pub const fn layout(self) -> Layout {
        let Self { layout, .. } = self;
        layout.layout()
    }

    #[inline]
    pub const fn offset(self) -> usize {
        let Self { offset, .. } = self;
        offset as usize
    }

    #[inline]
    pub const fn into_parts(self) -> (Layout, usize) {
        let Self { layout, offset } = self;
        (layout.layout(), offset as usize)
    }
}

impl From<GpuFieldLayout> for (Layout, usize) {
    #[inline]
    fn from(layout: GpuFieldLayout) -> Self {
        layout.into_parts()
    }
}

impl WithLayout for GpuFieldLayout {
    #[inline]
    fn layout(&self) -> Layout {
        Self::layout(*self)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferOffsetsFromGpuFieldLayout;

unsafe impl BufferOffsetsFrom<GpuFieldLayout> for BufferOffsetsFromGpuFieldLayout {
    unsafe fn next(&mut self, _: usize, desc: GpuFieldLayout) -> BufferOffset<GpuFieldLayout> {
        BufferOffset::new(desc, desc.offset())
    }
}

unsafe impl BufferOffsetsFromSelf for GpuFieldLayout {
    type BufferOffsets = BufferOffsetsFromGpuFieldLayout;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuFieldLayouts<T>
where
    T: ?Sized,
{
    next: usize,
    layouts: T,
}

impl<T> From<T> for GpuFieldLayouts<T> {
    fn from(layouts: T) -> Self {
        Self { next: 0, layouts }
    }
}

impl<'a, T> FieldLayouts<'a> for GpuFieldLayouts<T>
where
    T: AsRef<[GpuFieldLayout]> + Clone,
{
    type Output = GpuFieldLayouts<&'a [GpuFieldLayout]>;
    type OutputIter = GpuFieldLayouts<&'a [GpuFieldLayout]>;
    type OutputItem = GpuFieldLayout;

    fn field_layouts(&'a self) -> Self::Output {
        let Self { ref layouts, next } = *self;

        let layouts = layouts.as_ref();
        GpuFieldLayouts { next, layouts }
    }

    unsafe fn index_field_layouts_unchecked(&self, n: usize) -> Self::OutputItem {
        let Self { next, ref layouts } = *self;

        let index = unsafe { next.unchecked_add(n) };
        *unsafe { layouts.as_ref().index_unchecked(index) }
    }
}

impl<T> CovariantFieldLayouts for GpuFieldLayouts<T>
where
    T: AsRef<[GpuFieldLayout]> + Clone,
{
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        from
    }
}

impl<T> Iterator for GpuFieldLayouts<T>
where
    T: AsRef<[GpuFieldLayout]> + ?Sized,
{
    type Item = GpuFieldLayout;

    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self {
            ref layouts,
            ref mut next,
        } = *self;

        let index = *next + n;
        let layouts = layouts.as_ref();
        if index >= layouts.len() {
            *next = layouts.len();
            return None;
        }
        *next = index + 1;

        let item = *unsafe { layouts.index_unchecked(index) };
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }
}

impl<T> ExactSizeIterator for GpuFieldLayouts<T>
where
    T: AsRef<[GpuFieldLayout]> + ?Sized,
{
    fn len(&self) -> usize {
        let Self { ref layouts, next } = *self;

        let layouts = layouts.as_ref();
        layouts.len() - next
    }
}

impl<T> FusedIterator for GpuFieldLayouts<T> where T: AsRef<[GpuFieldLayout]> + ?Sized {}
