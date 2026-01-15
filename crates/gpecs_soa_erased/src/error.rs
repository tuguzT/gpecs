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
    pub fn new(expected: usize, actual: usize) -> Self {
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

#[inline]
pub fn check_len(len: usize, expected: usize) -> Result<(), LenMismatchError> {
    if len != expected {
        return Err(LenMismatchError::new(expected, len));
    }
    Ok(())
}

#[derive(Clone)]
pub struct LayoutMismatchError {
    expected: Layout,
    actual: Layout,
}

impl LayoutMismatchError {
    #[inline]
    #[track_caller]
    pub fn new(expected: Layout, actual: Layout) -> Self {
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

#[inline]
pub fn check_layout(layout: Layout, expected: Layout) -> Result<(), LayoutMismatchError> {
    if layout != expected {
        return Err(LayoutMismatchError::new(expected, layout));
    }
    Ok(())
}

#[derive(Clone)]
pub struct InsufficientLenError {
    expected: usize,
    actual: usize,
}

impl InsufficientLenError {
    #[inline]
    #[track_caller]
    pub fn new(expected: usize, actual: usize) -> Self {
        assert!(
            actual < expected,
            "actual length should be smaller than expected length",
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

impl Debug for InsufficientLenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }

        let Self { expected, actual } = self;
        f.debug_struct("InsufficientLenError")
            .field("expected", expected)
            .field("actual", actual)
            .finish()
    }
}

impl Display for InsufficientLenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expected, actual } = self;
        write!(
            f,
            "expected length to be greater than {expected}, but got {actual}"
        )
    }
}

impl Error for InsufficientLenError {}

#[inline]
pub fn check_sufficient_len(len: usize, expected: usize) -> Result<(), InsufficientLenError> {
    if len < expected {
        return Err(InsufficientLenError::new(expected, len));
    }
    Ok(())
}

#[derive(Clone)]
pub struct NotAlignedError {
    ptr: *const u8,
    target_align: NonZeroUsize,
}

const _: () = assert!(
    size_of::<NotAlignedError>() == size_of::<Option<NotAlignedError>>(),
    "non-zero usize should allow for non-zero field optimization",
);
const _: () = assert!(
    align_of::<NotAlignedError>() == align_of::<Option<NotAlignedError>>(),
    "non-zero usize should allow for non-zero field optimization",
);

impl NotAlignedError {
    #[inline]
    pub fn new(ptr: *const u8, target_layout: Layout) -> Self {
        let target_align: NonZeroUsize = target_layout
            .align()
            .try_into()
            .expect("alignment should not be zero because it is power of two");
        assert!(
            ptr.align_offset(target_align.get()) != 0,
            "the pointer {ptr:p} should not be aligned to {target_align}",
        );

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
        target_align.get()
    }
}

impl Debug for NotAlignedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }

        let Self { ptr, target_align } = self;
        f.debug_struct("NotAlignedError")
            .field("ptr", ptr)
            .field("target_align", target_align)
            .finish()
    }
}

impl Display for NotAlignedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ptr, target_align } = self;
        let align_offset = ptr.align_offset(target_align.get());
        write!(
            f,
            "pointer {ptr:p} is not aligned to {target_align} (its current align offset is {align_offset})"
        )
    }
}

impl Error for NotAlignedError {}

#[inline]
pub fn check_align(ptr: *const u8, target_layout: Layout) -> Result<(), NotAlignedError> {
    match ptr.align_offset(target_layout.align()) {
        0 => Ok(()),
        _ => Err(NotAlignedError::new(ptr, target_layout)),
    }
}
