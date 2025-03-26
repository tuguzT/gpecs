use core::{
    alloc::Layout,
    fmt::{self, Debug, Display},
};

#[derive(Clone)]
pub struct PtrNotAlignedError {
    ptr: *const u8,
    target_align: usize,
}

impl PtrNotAlignedError {
    #[inline]
    pub(super) fn new(ptr: *const u8, target_layout: Layout) -> Self {
        let target_align = target_layout.align();
        Self { ptr, target_align }
    }

    #[inline]
    pub fn ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn target_align(&self) -> usize {
        let Self { target_align, .. } = *self;
        target_align
    }
}

impl Debug for PtrNotAlignedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }

        let Self { ptr, target_align } = self;
        f.debug_struct("PtrNotAlignedError")
            .field("ptr", ptr)
            .field("target_align", target_align)
            .finish()
    }
}

impl Display for PtrNotAlignedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptr, target_align } = self;
        let align_offset = ptr.align_offset(*target_align);
        write!(f, "pointer {ptr:p} is not aligned to {target_align} (its current align offset is {align_offset})")
    }
}

#[derive(Clone)]
pub struct LayoutMismatchError<T>
where
    T: ?Sized,
{
    expected: Layout,
    actual: Layout,
    pub value: T,
}

impl<T> LayoutMismatchError<T> {
    #[inline]
    #[track_caller]
    pub(super) fn new(value: T, expected: Layout, actual: Layout) -> Self {
        assert_ne!(
            expected, actual,
            "expected and actual layouts should differ from each other",
        );
        Self {
            value,
            expected,
            actual,
        }
    }

    #[inline]
    pub fn map<U, F>(self, f: F) -> LayoutMismatchError<U>
    where
        F: FnOnce(T) -> U,
    {
        let Self {
            expected,
            actual,
            value,
        } = self;

        LayoutMismatchError {
            expected,
            actual,
            value: f(value),
        }
    }
}

impl<T> LayoutMismatchError<T>
where
    T: ?Sized,
{
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

impl<T> Debug for LayoutMismatchError<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }

        let Self {
            expected, actual, ..
        } = self;
        f.debug_struct("LayoutMismatchError")
            .field("expected", expected)
            .field("actual", actual)
            .finish()
    }
}

impl<T> Display for LayoutMismatchError<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            expected, actual, ..
        } = self;
        write!(f, "{expected:?} does not match expected {actual:?}")
    }
}
