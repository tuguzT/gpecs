use core::{alloc::Layout, iter::FusedIterator, num::NonZeroUsize};

use gpecs_soa_erased::{
    CovariantFieldLayouts,
    layout::WithLayout,
    soa::field::{FieldLayouts, FieldLayoutsOutput},
};
use spirv_std::arch::IndexUnchecked;

/// FFI-compatible layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct FfiLayout {
    size: usize,
    align: NonZeroUsize,
}

impl FfiLayout {
    /// Creates a new FFI-compatible layout from the given [`Layout`].
    #[inline]
    pub const fn new(layout: Layout) -> Self {
        let size = layout.size();
        // SAFETY: Layout::align() is guaranteed to be a power of two, which is non-zero
        let align = unsafe { NonZeroUsize::new_unchecked(layout.align()) };
        Self { size, align }
    }

    /// Creates a new FFI-compatible layout from the given type.
    #[inline]
    pub const fn of<T>() -> Self {
        let layout = Layout::new::<T>();
        Self::new(layout)
    }

    /// Returns the [`Layout`] of this FFI-compatible layout.
    #[inline]
    pub const fn layout(self) -> Layout {
        let Self { size, align } = self;
        // SAFETY: self could only be created from a valid `Layout`
        unsafe { Layout::from_size_align_unchecked(size, align.get()) }
    }
}

impl From<Layout> for FfiLayout {
    #[inline]
    fn from(layout: Layout) -> Self {
        Self::new(layout)
    }
}

impl From<FfiLayout> for Layout {
    #[inline]
    fn from(layout: FfiLayout) -> Self {
        layout.layout()
    }
}

impl WithLayout for FfiLayout {
    #[inline]
    fn layout(&self) -> Layout {
        Self::layout(*self)
    }
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
    T: AsRef<[FfiLayout]> + Clone,
{
    type Output = GpuFieldLayouts<&'a [FfiLayout]>;

    fn field_layouts(&'a self) -> Self::Output {
        let Self { ref layouts, next } = *self;

        let layouts = layouts.as_ref();
        GpuFieldLayouts { next, layouts }
    }
}

impl<T> CovariantFieldLayouts for GpuFieldLayouts<T>
where
    T: AsRef<[FfiLayout]> + Clone,
{
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        from
    }
}

impl<T> Iterator for GpuFieldLayouts<T>
where
    T: AsRef<[FfiLayout]> + ?Sized,
{
    type Item = FfiLayout;

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
    T: AsRef<[FfiLayout]> + ?Sized,
{
    fn len(&self) -> usize {
        let Self { ref layouts, next } = *self;

        let layouts = layouts.as_ref();
        layouts.len() - next
    }
}

impl<T> FusedIterator for GpuFieldLayouts<T> where T: AsRef<[FfiLayout]> + ?Sized {}
