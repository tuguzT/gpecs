use core::{
    alloc::Layout,
    error::Error,
    fmt::{self, Debug, Display},
    num::NonZeroUsize,
};

#[derive(Debug, Clone)]
pub struct LenMismatchError {
    expected: usize,
    actual: usize,
}

impl LenMismatchError {
    #[inline]
    pub fn new(expected: usize, actual: usize) -> Option<Self> {
        if expected == actual {
            return None;
        }

        let me = unsafe { Self::new_unchecked(expected, actual) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(expected: usize, actual: usize) -> Self {
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

impl Display for LenMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expected, actual } = self;
        write!(f, "expected length to be {expected}, but got {actual}")
    }
}

impl Error for LenMismatchError {}

#[inline]
pub fn check_len(len: usize, expected: usize) -> Result<(), LenMismatchError> {
    LenMismatchError::new(expected, len).map_or(Ok(()), Err)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutMismatchError {
    expected: Layout,
    actual: Layout,
}

const _: () = assert_npo::<LayoutMismatchError>();

impl LayoutMismatchError {
    #[inline]
    pub fn new(expected: Layout, actual: Layout) -> Option<Self> {
        if expected == actual {
            return None;
        }

        let me = unsafe { Self::new_unchecked(expected, actual) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(expected: Layout, actual: Layout) -> Self {
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

impl Display for LayoutMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expected, actual } = self;
        write!(f, "{actual:?} does not match expected {expected:?}")
    }
}

impl Error for LayoutMismatchError {}

#[inline]
pub fn check_layout(layout: Layout, expected: Layout) -> Result<(), LayoutMismatchError> {
    LayoutMismatchError::new(expected, layout).map_or(Ok(()), Err)
}

#[derive(Debug, Clone)]
pub struct InsufficientLenError {
    expected: usize,
    actual: usize,
}

impl InsufficientLenError {
    #[inline]
    pub fn new(expected: usize, actual: usize) -> Option<Self> {
        if actual >= expected {
            return None;
        }

        let me = unsafe { Self::new_unchecked(expected, actual) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(expected: usize, actual: usize) -> Self {
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

impl Display for InsufficientLenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expected, actual } = self;
        write!(
            f,
            "expected length to be greater than or equal to {expected}, but got {actual}"
        )
    }
}

impl Error for InsufficientLenError {}

#[inline]
pub fn check_sufficient_len(len: usize, expected: usize) -> Result<(), InsufficientLenError> {
    InsufficientLenError::new(expected, len).map_or(Ok(()), Err)
}

#[derive(Debug, Clone)]
pub struct InsufficientAlignError {
    expected: NonZeroUsize,
    actual: NonZeroUsize,
}

const _: () = assert_npo::<InsufficientAlignError>();

impl InsufficientAlignError {
    #[inline]
    pub fn new(expected: Layout, actual: Layout) -> Option<Self> {
        let expected = nonzero_align(expected);
        let actual = nonzero_align(actual);
        if actual >= expected {
            return None;
        }

        let me = unsafe { Self::new_unchecked(expected, actual) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(expected: NonZeroUsize, actual: NonZeroUsize) -> Self {
        Self { expected, actual }
    }

    #[inline]
    pub fn expected(&self) -> NonZeroUsize {
        let Self { expected, .. } = *self;
        expected
    }

    #[inline]
    pub fn actual(&self) -> NonZeroUsize {
        let Self { actual, .. } = *self;
        actual
    }
}

impl Display for InsufficientAlignError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expected, actual } = self;
        write!(
            f,
            "expected alignment to be greater than or equal to {expected}, but got {actual}"
        )
    }
}

impl Error for InsufficientAlignError {}

#[inline]
pub fn check_sufficient_align(
    actual: Layout,
    expected: Layout,
) -> Result<(), InsufficientAlignError> {
    InsufficientAlignError::new(expected, actual).map_or(Ok(()), Err)
}

#[derive(Debug, Clone)]
pub struct NotAlignedError {
    ptr: *const u8,
    target_align: NonZeroUsize,
}

const _: () = assert_npo::<NotAlignedError>();

impl NotAlignedError {
    #[inline]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(ptr: *const u8, target_layout: Layout) -> Option<Self> {
        let target_align = nonzero_align(target_layout);
        if ptr.align_offset(target_align.get()) == 0 {
            return None;
        }

        let me = unsafe { Self::new_unchecked(ptr, target_align) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(ptr: *const u8, target_align: NonZeroUsize) -> Self {
        Self { ptr, target_align }
    }

    #[inline]
    pub fn ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn target_align(&self) -> NonZeroUsize {
        let Self { target_align, .. } = *self;
        target_align
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
pub fn check_ptr_align(ptr: *const u8, target_layout: Layout) -> Result<(), NotAlignedError> {
    NotAlignedError::new(ptr, target_layout).map_or(Ok(()), Err)
}

#[inline]
const fn assert_npo<T>() {
    assert!(
        size_of::<T>() == size_of::<Option<T>>(),
        "non-zero usize should allow for NPO",
    );
    assert!(
        align_of::<T>() == align_of::<Option<T>>(),
        "non-zero usize should allow for NPO",
    );
}

#[inline]
fn nonzero_align(layout: Layout) -> NonZeroUsize {
    layout
        .align()
        .try_into()
        .expect("alignment should not be zero because it is power of two")
}
