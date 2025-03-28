use core::{
    alloc::Layout,
    error::Error,
    fmt::{self, Debug, Display},
    num::NonZeroUsize,
};

#[derive(Clone)]
pub struct LenMismatchError {
    expected: usize,
    actual: usize,
}

impl LenMismatchError {
    #[inline]
    #[track_caller]
    pub(crate) fn new(expected: usize, actual: usize) -> Self {
        assert_ne!(
            expected, actual,
            "expected and actual lengths should differ from each other",
        );
        Self { expected, actual }
    }

    #[inline]
    pub fn expected(&self) -> usize {
        let Self { expected, .. } = *self;
        expected
    }

    #[inline]
    pub fn actual(&self) -> usize {
        let Self { actual, .. } = *self;
        actual
    }
}

impl Debug for LenMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }

        let Self { expected, actual } = self;
        f.debug_struct("LenMismatchError")
            .field("expected", expected)
            .field("actual", actual)
            .finish()
    }
}

impl Display for LenMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expected, actual } = self;
        write!(f, "expected length to be {expected}, but got {actual}")
    }
}

impl Error for LenMismatchError {}

#[derive(Clone)]
pub struct LayoutMismatchError {
    expected: Layout,
    actual: Layout,
}

impl LayoutMismatchError {
    #[inline]
    #[track_caller]
    pub(crate) fn new(expected: Layout, actual: Layout) -> Self {
        assert_ne!(
            expected, actual,
            "expected and actual layouts should differ from each other",
        );
        Self { expected, actual }
    }

    #[inline]
    pub fn expected(&self) -> Layout {
        let Self { expected, .. } = *self;
        expected
    }

    #[inline]
    pub fn actual(&self) -> Layout {
        let Self { actual, .. } = *self;
        actual
    }
}

impl Debug for LayoutMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }

        let Self { expected, actual } = self;
        f.debug_struct("LayoutMismatchError")
            .field("expected", expected)
            .field("actual", actual)
            .finish()
    }
}

impl Display for LayoutMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expected, actual } = self;
        write!(f, "{actual:?} does not match expected {expected:?}")
    }
}

impl Error for LayoutMismatchError {}

#[derive(Clone)]
pub struct InvalidLayoutError {
    layout: Layout,
    max_align: NonZeroUsize,
}

impl InvalidLayoutError {
    #[inline]
    #[track_caller]
    pub(crate) fn new(layout: Layout, max_align: Layout) -> Self {
        assert!(
            layout.align() > max_align.align(),
            "input align should be greater than max align to be an error",
        );
        let max_align = max_align
            .align()
            .try_into()
            .expect("alignment should not be zero because it is power of two");
        Self { layout, max_align }
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn max_align(&self) -> usize {
        let Self { max_align, .. } = *self;
        max_align.get()
    }
}

impl Debug for InvalidLayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }

        let Self { layout, max_align } = self;
        f.debug_struct("InvalidLayoutError")
            .field("layout", layout)
            .field("max_align", max_align)
            .finish()
    }
}

impl Display for InvalidLayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { layout, max_align } = self;
        write!(
            f,
            "alignment of input {layout:?} must be less than or equal to {max_align}",
        )
    }
}

impl Error for InvalidLayoutError {}
