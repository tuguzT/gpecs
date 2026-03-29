use core::{alloc::Layout, num::NonZeroUsize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct FfiLayout {
    size: usize,
    align: NonZeroUsize,
}

impl FfiLayout {
    /// Creates a new field descriptor from the given [`Layout`].
    #[inline]
    pub const fn new(layout: Layout) -> Self {
        let size = layout.size();
        // SAFETY: Layout::align() is guaranteed to be a power of two, which is non-zero
        let align = unsafe { NonZeroUsize::new_unchecked(layout.align()) };
        Self { size, align }
    }

    /// Creates a new field descriptor from the given type.
    #[inline]
    pub const fn of<T>() -> Self {
        let layout = Layout::new::<T>();
        Self::new(layout)
    }

    /// Returns the [`Layout`] of this field descriptor.
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
