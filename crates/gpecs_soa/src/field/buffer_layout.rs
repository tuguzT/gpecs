use core::{
    alloc::{Layout, LayoutError},
    num::NonZeroUsize,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferLayout {
    total_layout: Layout,
    capacity: usize,
    slice_align: NonZeroUsize,
}

impl BufferLayout {
    #[inline]
    pub const fn new(capacity: usize, slice_align: NonZeroUsize) -> Self {
        let total_layout = Layout::new::<()>();
        Self::from_parts(total_layout, capacity, slice_align)
    }

    #[inline]
    pub const fn from_parts(
        total_layout: Layout,
        capacity: usize,
        slice_align: NonZeroUsize,
    ) -> Self {
        Self {
            total_layout,
            capacity,
            slice_align,
        }
    }

    #[inline]
    pub const fn layout(self) -> Layout {
        let Self { total_layout, .. } = self;
        total_layout
    }

    #[inline]
    pub const fn capacity(self) -> usize {
        let Self { capacity, .. } = self;
        capacity
    }

    #[inline]
    pub const fn slice_align(self) -> NonZeroUsize {
        let Self { slice_align, .. } = self;
        slice_align
    }

    #[inline]
    pub const fn into_parts(self) -> (Layout, usize, NonZeroUsize) {
        let Self {
            total_layout,
            capacity,
            slice_align,
        } = self;
        (total_layout, capacity, slice_align)
    }

    #[inline]
    pub const fn extend(&mut self, layout: Layout) -> Result<usize, LayoutError> {
        let Self {
            ref mut total_layout,
            capacity,
            slice_align,
        } = *self;

        let layout = layout.pad_to_align();
        let layout = match layout.repeat_packed(capacity) {
            Ok(layout) => layout,
            Err(error) => return Err(error),
        };

        let next = match layout.align_to(slice_align.get()) {
            Ok(layout) => layout,
            Err(error) => return Err(error),
        };

        let offset;
        (*total_layout, offset) = match total_layout.extend(next) {
            Ok(ok) => ok,
            Err(error) => return Err(error),
        };

        Ok(offset)
    }

    #[inline]
    pub const unsafe fn extend_unchecked(&mut self, layout: Layout) -> usize {
        let Self {
            ref mut total_layout,
            capacity,
            slice_align,
        } = *self;

        let layout = layout.pad_to_align();
        let layout = unsafe { repeat_packed_unchecked(layout, capacity) };

        let next = unsafe { align_to_unchecked(layout, slice_align.get()) };

        let offset;
        (*total_layout, offset) = unsafe { extend_unchecked(*total_layout, next) };

        offset
    }
}

/// Unchecked copy of [`Layout::repeat_packed()`] on stable channel.
#[inline]
const unsafe fn repeat_packed_unchecked(layout: Layout, n: usize) -> Layout {
    // FIXME: use `unchecked_mul` instead
    let size = layout.size().wrapping_mul(n);
    unsafe { Layout::from_size_align_unchecked(size, layout.align()) }
}

/// Unchecked copy of [`Layout::align_to()`] which Rust-GPU struggles to inline by itself.
#[inline]
const unsafe fn align_to_unchecked(layout: Layout, align: usize) -> Layout {
    let align = usize_max(layout.align(), align);
    unsafe { Layout::from_size_align_unchecked(layout.size(), align) }
}

/// Unchecked copy of [`Layout::extend()`] which Rust-GPU struggles to inline by itself.
#[inline]
const unsafe fn extend_unchecked(layout: Layout, next: Layout) -> (Layout, usize) {
    let new_align = usize_max(layout.align(), next.align());
    let offset = unsafe {
        let align_m1 = next.align().unchecked_sub(1);
        layout.size().unchecked_add(align_m1) & !align_m1
    };
    let new_size = unsafe { offset.unchecked_add(next.size()) };

    let layout = unsafe { Layout::from_size_align_unchecked(new_size, new_align) };
    (layout, offset)
}

/// The same as [`usize::max()`], but usable in const context.
#[inline]
const fn usize_max(lhs: usize, rhs: usize) -> usize {
    if rhs < lhs { lhs } else { rhs }
}
