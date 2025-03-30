use core::{
    alloc::Layout,
    error::Error,
    fmt::{self, Debug, Display},
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
