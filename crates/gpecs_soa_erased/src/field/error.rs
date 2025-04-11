use core::{
    alloc::Layout,
    error::Error,
    fmt::{self, Debug, Display},
    num::NonZeroUsize,
};

use crate::error::{LayoutMismatchError, LenMismatchError};

#[derive(Clone)]
pub struct PtrNotAlignedError {
    ptr: *const u8,
    target_align: NonZeroUsize,
}

const _: () = assert!(
    size_of::<PtrNotAlignedError>() == size_of::<Option<PtrNotAlignedError>>(),
    "non-zero usize should allow for non-zero field optimization",
);
const _: () = assert!(
    align_of::<PtrNotAlignedError>() == align_of::<Option<PtrNotAlignedError>>(),
    "non-zero usize should allow for non-zero field optimization",
);

impl PtrNotAlignedError {
    #[inline]
    pub(super) fn new(ptr: *const u8, target_layout: Layout) -> Self {
        let target_align = target_layout
            .align()
            .try_into()
            .expect("alignment should not be zero because it is power of two");
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
        let align_offset = ptr.align_offset(target_align.get());
        write!(f, "pointer {ptr:p} is not aligned to {target_align} (its current align offset is {align_offset})")
    }
}

impl Error for PtrNotAlignedError {}

#[derive(Clone)]
pub struct SliceLenMismatchError {
    item_size: usize,
    len: usize,
    actual: usize,
}

impl SliceLenMismatchError {
    #[inline]
    #[track_caller]
    pub(super) fn new(item_size: usize, len: usize, actual: usize) -> Self {
        assert_ne!(
            item_size * len,
            actual,
            "expected and actual lengths should differ from each other",
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

impl Debug for SliceLenMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }

        let Self {
            item_size,
            len,
            actual,
        } = self;
        f.debug_struct("SliceLenMismatchError")
            .field("item_size", item_size)
            .field("len", len)
            .field("actual", actual)
            .finish()
    }
}

impl Display for SliceLenMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            item_size,
            len,
            actual,
        } = self;

        write!(
            f,
            "expected length to be item size of {item_size} * {len} items, but got {actual}",
        )
    }
}

impl Error for SliceLenMismatchError {}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct IntoValueError<T>
where
    T: ?Sized,
{
    pub reason: LayoutMismatchError,
    pub value: T,
}

impl<T> IntoValueError<T> {
    #[inline]
    pub(super) fn new(value: T, reason: LayoutMismatchError) -> Self {
        Self { reason, value }
    }
}

impl<T> Display for IntoValueError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, value } = self;
        write!(f, "failed to convert {value}: {reason}")
    }
}

impl<T> Error for IntoValueError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Clone)]
pub enum ErasedFieldError {
    PtrNotAligned(PtrNotAlignedError),
    LenMismatch(LenMismatchError),
}

impl From<PtrNotAlignedError> for ErasedFieldError {
    #[inline]
    fn from(error: PtrNotAlignedError) -> Self {
        Self::PtrNotAligned(error)
    }
}

impl From<LenMismatchError> for ErasedFieldError {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl Debug for ErasedFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }
        match self {
            Self::PtrNotAligned(error) => f.debug_tuple("PtrNotAligned").field(error).finish(),
            Self::LenMismatch(error) => f.debug_tuple("BufferLen").field(error).finish(),
        }
    }
}

impl Display for ErasedFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PtrNotAligned(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedFieldError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::PtrNotAligned(error) => Some(error),
            Self::LenMismatch(error) => Some(error),
        }
    }
}

#[derive(Clone)]
pub enum ErasedFieldSliceError {
    PtrNotAligned(PtrNotAlignedError),
    LenMismatch(SliceLenMismatchError),
}

impl From<PtrNotAlignedError> for ErasedFieldSliceError {
    #[inline]
    fn from(error: PtrNotAlignedError) -> Self {
        Self::PtrNotAligned(error)
    }
}

impl From<SliceLenMismatchError> for ErasedFieldSliceError {
    #[inline]
    fn from(error: SliceLenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl Debug for ErasedFieldSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }
        match self {
            Self::PtrNotAligned(error) => f.debug_tuple("PtrNotAligned").field(error).finish(),
            Self::LenMismatch(error) => f.debug_tuple("BufferLen").field(error).finish(),
        }
    }
}

impl Display for ErasedFieldSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PtrNotAligned(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedFieldSliceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::PtrNotAligned(error) => Some(error),
            Self::LenMismatch(error) => Some(error),
        }
    }
}
