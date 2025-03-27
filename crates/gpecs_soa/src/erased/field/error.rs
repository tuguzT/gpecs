use core::{
    alloc::Layout,
    error::Error,
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

impl Error for PtrNotAlignedError {}

#[derive(Clone)]
pub struct BufferLenError {
    expected: usize,
    actual: usize,
}

impl BufferLenError {
    #[inline]
    #[track_caller]
    pub(super) fn new(expected: usize, actual: usize) -> Self {
        assert_ne!(
            expected, actual,
            "expected and actual buffer lengths should differ from each other",
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

impl Debug for BufferLenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }

        let Self { expected, actual } = self;
        f.debug_struct("BufferLenError")
            .field("expected", expected)
            .field("actual", actual)
            .finish()
    }
}

impl Display for BufferLenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expected, actual } = self;
        write!(f, "expected buffer len to be {expected}, but got {actual}")
    }
}

impl Error for BufferLenError {}

#[derive(Clone)]
pub struct BufferSliceLenError {
    item_size: usize,
    len: usize,
    actual: usize,
}

impl BufferSliceLenError {
    #[inline]
    #[track_caller]
    pub(super) fn new(item_size: usize, len: usize, actual: usize) -> Self {
        assert_ne!(
            item_size * len,
            actual,
            "expected and actual buffer lengths should differ from each other",
        );
        Self {
            item_size,
            len,
            actual,
        }
    }

    #[inline]
    pub fn item_size(&self) -> usize {
        let Self { item_size, .. } = *self;
        item_size
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn expected(&self) -> usize {
        let Self { item_size, len, .. } = *self;
        item_size * len
    }

    #[inline]
    pub fn actual(&self) -> usize {
        let Self { actual, .. } = *self;
        actual
    }
}

impl Debug for BufferSliceLenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }

        let Self {
            item_size,
            len,
            actual,
        } = self;
        f.debug_struct("BufferSliceLenError")
            .field("item_size", item_size)
            .field("len", len)
            .field("actual", actual)
            .finish()
    }
}

impl Display for BufferSliceLenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            item_size,
            len,
            actual,
        } = self;

        write!(
            f,
            "expected buffer len to be item size of {item_size} * {len} items, but got {actual}",
        )
    }
}

impl Error for BufferSliceLenError {}

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
        write!(f, "{actual:?} does not match expected {expected:?}")
    }
}

impl<T> Error for LayoutMismatchError<T> where T: ?Sized {}

#[derive(Clone)]
pub enum ErasedFieldError {
    PtrNotAligned(PtrNotAlignedError),
    BufferLen(BufferLenError),
}

impl From<PtrNotAlignedError> for ErasedFieldError {
    #[inline]
    fn from(error: PtrNotAlignedError) -> Self {
        Self::PtrNotAligned(error)
    }
}

impl From<BufferLenError> for ErasedFieldError {
    #[inline]
    fn from(error: BufferLenError) -> Self {
        Self::BufferLen(error)
    }
}

impl Debug for ErasedFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }
        match self {
            Self::PtrNotAligned(error) => f.debug_tuple("PtrNotAligned").field(error).finish(),
            Self::BufferLen(error) => f.debug_tuple("BufferLen").field(error).finish(),
        }
    }
}

impl Display for ErasedFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PtrNotAligned(error) => Display::fmt(error, f),
            Self::BufferLen(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedFieldError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::PtrNotAligned(error) => Some(error),
            Self::BufferLen(error) => Some(error),
        }
    }
}

#[derive(Clone)]
pub enum ErasedFieldSliceError {
    PtrNotAligned(PtrNotAlignedError),
    BufferLen(BufferSliceLenError),
}

impl From<PtrNotAlignedError> for ErasedFieldSliceError {
    #[inline]
    fn from(error: PtrNotAlignedError) -> Self {
        Self::PtrNotAligned(error)
    }
}

impl From<BufferSliceLenError> for ErasedFieldSliceError {
    #[inline]
    fn from(error: BufferSliceLenError) -> Self {
        Self::BufferLen(error)
    }
}

impl Debug for ErasedFieldSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }
        match self {
            Self::PtrNotAligned(error) => f.debug_tuple("PtrNotAligned").field(error).finish(),
            Self::BufferLen(error) => f.debug_tuple("BufferLen").field(error).finish(),
        }
    }
}

impl Display for ErasedFieldSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PtrNotAligned(error) => Display::fmt(error, f),
            Self::BufferLen(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedFieldSliceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::PtrNotAligned(error) => Some(error),
            Self::BufferLen(error) => Some(error),
        }
    }
}
