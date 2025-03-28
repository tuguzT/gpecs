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
    pub(super) fn new(expected: usize, actual: usize) -> Self {
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
    pub(super) fn new(expected: Layout, actual: Layout) -> Self {
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
    pub(super) fn new(layout: Layout, max_align: Layout) -> Self {
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

#[derive(Clone)]
#[non_exhaustive]
pub struct FromValueError<T>
where
    T: ?Sized,
{
    pub reason: InvalidLayoutError,
    pub value: T,
}

impl<T> FromValueError<T> {
    #[inline]
    pub(super) fn new(value: T, reason: InvalidLayoutError) -> Self {
        Self { reason, value }
    }
}

impl<T> Debug for FromValueError<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, .. } = self;
        Debug::fmt(reason, f)
    }
}

impl<T> Display for FromValueError<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, .. } = self;
        Display::fmt(reason, f)
    }
}

impl<T> Error for FromValueError<T>
where
    T: ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Clone)]
#[non_exhaustive]
pub struct IntoValueError<T>
where
    T: ?Sized,
{
    pub reason: IntoValueErrorKind,
    pub value: T,
}

impl<T> IntoValueError<T> {
    #[inline]
    pub(super) fn new(value: T, reason: IntoValueErrorKind) -> Self {
        Self { reason, value }
    }
}

impl<T> Debug for IntoValueError<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, .. } = self;
        Debug::fmt(reason, f)
    }
}

impl<T> Display for IntoValueError<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, .. } = self;
        Display::fmt(reason, f)
    }
}

impl<T> Error for IntoValueError<T>
where
    T: ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Clone)]
pub enum IntoValueErrorKind {
    InvalidLayout(InvalidLayoutError),
    LayoutMismatch(LayoutMismatchError),
    LenMismatch(LenMismatchError),
}

impl From<InvalidLayoutError> for IntoValueErrorKind {
    fn from(error: InvalidLayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<LayoutMismatchError> for IntoValueErrorKind {
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<LenMismatchError> for IntoValueErrorKind {
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl Debug for IntoValueErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }
        match self {
            Self::InvalidLayout(error) => f.debug_tuple("InvalidLayout").field(error).finish(),
            Self::LayoutMismatch(error) => f.debug_tuple("LayoutMismatch").field(error).finish(),
            Self::LenMismatch(error) => f.debug_tuple("LenMismatch").field(error).finish(),
        }
    }
}

impl Display for IntoValueErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
        }
    }
}

impl Error for IntoValueErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidLayout(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
            Self::LenMismatch(error) => Some(error),
        }
    }
}

#[derive(Clone)]
pub enum ErasedSoaError {
    LenMismatch(LenMismatchError),
    InvalidLayout(InvalidLayoutError),
}

impl From<LenMismatchError> for ErasedSoaError {
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<InvalidLayoutError> for ErasedSoaError {
    fn from(error: InvalidLayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl Debug for ErasedSoaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }
        match self {
            Self::LenMismatch(error) => f.debug_tuple("LenMismatch").field(error).finish(),
            Self::InvalidLayout(error) => f.debug_tuple("InvalidLayout").field(error).finish(),
        }
    }
}

impl Display for ErasedSoaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedSoaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
        }
    }
}
