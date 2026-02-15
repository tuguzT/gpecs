use core::{
    alloc::LayoutError,
    error::Error,
    fmt::{self, Debug, Display},
};

pub use gpecs_erased::error::*;

#[derive(Clone)]
pub struct InvalidOffsetError {
    offset: usize,
    capacity: usize,
}

impl InvalidOffsetError {
    #[inline]
    pub fn new(offset: usize, capacity: usize) -> Option<Self> {
        if offset <= capacity {
            return None;
        }

        let me = unsafe { Self::new_unchecked(offset, capacity) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(offset: usize, capacity: usize) -> Self {
        Self { offset, capacity }
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { offset, .. } = *self;
        offset
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { capacity, .. } = *self;
        capacity
    }
}

impl Debug for InvalidOffsetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }

        let Self { offset, capacity } = self;
        f.debug_struct("InvalidOffsetError")
            .field("offset", offset)
            .field("capacity", capacity)
            .finish()
    }
}

impl Display for InvalidOffsetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { offset, capacity } = self;
        write!(
            f,
            "expected offset to be smaller than or equal to capacity {capacity}, but got {offset}"
        )
    }
}

impl Error for InvalidOffsetError {}

#[inline]
pub fn check_offset(offset: usize, capacity: usize) -> Result<(), InvalidOffsetError> {
    InvalidOffsetError::new(offset, capacity).map_or(Ok(()), Err)
}

#[derive(Debug, Clone)]
pub enum PtrsError {
    NotAligned(NotAlignedError),
    InvalidLayout(LayoutError),
    InvalidOffset(InvalidOffsetError),
    InsufficientLen(InsufficientLenError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotAlignedError> for PtrsError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<LayoutError> for PtrsError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<InvalidOffsetError> for PtrsError {
    #[inline]
    fn from(error: InvalidOffsetError) -> Self {
        Self::InvalidOffset(error)
    }
}

impl From<InsufficientLenError> for PtrsError {
    #[inline]
    fn from(error: InsufficientLenError) -> Self {
        Self::InsufficientLen(error)
    }
}

impl From<InsufficientAlignError> for PtrsError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Display for PtrsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::InvalidOffset(error) => Display::fmt(error, f),
            Self::InsufficientLen(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for PtrsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotAligned(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::InvalidOffset(error) => Some(error),
            Self::InsufficientLen(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}

#[derive(Clone)]
pub struct InvalidOffsetLenError {
    offset: usize,
    len: usize,
    capacity: usize,
}

impl InvalidOffsetLenError {
    #[inline]
    pub fn new(offset: usize, len: usize, capacity: usize) -> Option<Self> {
        if offset + len <= capacity {
            return None;
        }

        let me = unsafe { Self::new_unchecked(offset, len, capacity) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(offset: usize, len: usize, capacity: usize) -> Self {
        Self {
            offset,
            len,
            capacity,
        }
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { offset, .. } = *self;
        offset
    }

    #[inline]
    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { capacity, .. } = *self;
        capacity
    }
}

impl Debug for InvalidOffsetLenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }

        let Self {
            offset,
            len,
            capacity,
        } = self;
        f.debug_struct("InvalidOffsetLenError")
            .field("offset", offset)
            .field("len", len)
            .field("capacity", capacity)
            .finish()
    }
}

impl Display for InvalidOffsetLenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            offset,
            len,
            capacity,
        } = self;
        write!(
            f,
            "expected offset + len to be smaller than or equal to capacity {capacity}, but got {offset} + {len}",
        )
    }
}

impl Error for InvalidOffsetLenError {}

#[inline]
pub fn check_offset_len(
    offset: usize,
    len: usize,
    capacity: usize,
) -> Result<(), InvalidOffsetLenError> {
    InvalidOffsetLenError::new(offset, len, capacity).map_or(Ok(()), Err)
}

#[derive(Debug, Clone)]
pub enum SlicePtrsError {
    NotAligned(NotAlignedError),
    InvalidLayout(LayoutError),
    InvalidOffset(InvalidOffsetError),
    InvalidOffsetLen(InvalidOffsetLenError),
    InsufficientLen(InsufficientLenError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotAlignedError> for SlicePtrsError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<LayoutError> for SlicePtrsError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<InvalidOffsetError> for SlicePtrsError {
    #[inline]
    fn from(error: InvalidOffsetError) -> Self {
        Self::InvalidOffset(error)
    }
}

impl From<InvalidOffsetLenError> for SlicePtrsError {
    #[inline]
    fn from(error: InvalidOffsetLenError) -> Self {
        Self::InvalidOffsetLen(error)
    }
}

impl From<InsufficientLenError> for SlicePtrsError {
    #[inline]
    fn from(error: InsufficientLenError) -> Self {
        Self::InsufficientLen(error)
    }
}

impl From<InsufficientAlignError> for SlicePtrsError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Display for SlicePtrsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::InvalidOffset(error) => Display::fmt(error, f),
            Self::InvalidOffsetLen(error) => Display::fmt(error, f),
            Self::InsufficientLen(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for SlicePtrsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotAligned(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::InvalidOffset(error) => Some(error),
            Self::InvalidOffsetLen(error) => Some(error),
            Self::InsufficientLen(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IterOrFieldLenMismatchError {
    IterLenMismatch(LenMismatchError),
    FieldLenMismatch {
        error: LenMismatchError,
        field_index: usize,
    },
}

impl Display for IterOrFieldLenMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IterLenMismatch(error) => write!(f, "iterator length mismatch: {error}"),
            Self::FieldLenMismatch { error, field_index } => {
                write!(f, "field {field_index} length mismatch: {error}")
            }
        }
    }
}

impl Error for IterOrFieldLenMismatchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IterLenMismatch(error) | Self::FieldLenMismatch { error, .. } => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FromStorageFieldsDescriptorsError {
    LenMismatch(IterOrFieldLenMismatchError),
    LayoutMismatch(LayoutMismatchError),
    InsufficientAlign(InsufficientAlignError),
    InvalidLayout(LayoutError),
}

impl From<IterOrFieldLenMismatchError> for FromStorageFieldsDescriptorsError {
    #[inline]
    fn from(error: IterOrFieldLenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<LayoutMismatchError> for FromStorageFieldsDescriptorsError {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<InsufficientAlignError> for FromStorageFieldsDescriptorsError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl From<LayoutError> for FromStorageFieldsDescriptorsError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl Display for FromStorageFieldsDescriptorsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
        }
    }
}

impl Error for FromStorageFieldsDescriptorsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FromFieldsDescriptorsError<T> {
    LenMismatch(IterOrFieldLenMismatchError),
    InsufficientAlign(InsufficientAlignError),
    InvalidLayout(LayoutError),
    FromLayout(T),
}

impl<T> From<IterOrFieldLenMismatchError> for FromFieldsDescriptorsError<T> {
    #[inline]
    fn from(value: IterOrFieldLenMismatchError) -> Self {
        Self::LenMismatch(value)
    }
}

impl<T> From<InsufficientAlignError> for FromFieldsDescriptorsError<T> {
    #[inline]
    fn from(value: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(value)
    }
}

impl<T> From<LayoutError> for FromFieldsDescriptorsError<T> {
    #[inline]
    fn from(value: LayoutError) -> Self {
        Self::InvalidLayout(value)
    }
}

impl<T> Display for FromFieldsDescriptorsError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for FromFieldsDescriptorsError<T>
where
    T: Error,
{
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}

#[derive(Clone)]
pub enum FromStorageValueError {
    LayoutMismatch(LayoutMismatchError),
    InvalidLayout(LayoutError),
}

impl From<LayoutMismatchError> for FromStorageValueError {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<LayoutError> for FromStorageValueError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl Debug for FromStorageValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }
        match self {
            Self::LayoutMismatch(arg0) => f.debug_tuple("LayoutMismatch").field(arg0).finish(),
            Self::InvalidLayout(arg0) => f.debug_tuple("InvalidLayout").field(arg0).finish(),
        }
    }
}

impl Display for FromStorageValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
        }
    }
}

impl Error for FromStorageValueError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LayoutMismatch(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FromValueError<T> {
    InvalidLayout(LayoutError),
    FromLayout(T),
}

impl<T> From<LayoutError> for FromValueError<T> {
    #[inline]
    fn from(value: LayoutError) -> Self {
        Self::InvalidLayout(value)
    }
}

impl<T> Display for FromValueError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for FromValueError<T>
where
    T: Error,
{
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DowncastError<T>
where
    T: ?Sized,
{
    pub reason: DowncastErrorKind,
    pub value: T,
}

impl<T> DowncastError<T> {
    #[inline]
    pub(crate) fn new(value: T, reason: DowncastErrorKind) -> Self {
        Self { reason, value }
    }

    #[inline]
    pub fn map_value<U, F>(self, f: F) -> DowncastError<U>
    where
        F: FnOnce(T) -> U,
    {
        let Self { reason, value } = self;
        DowncastError::new(f(value), reason)
    }
}

impl<T> Display for DowncastError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, value } = self;
        write!(f, "failed to downcast {value}: {reason}")
    }
}

impl<T> Error for DowncastError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Clone)]
pub enum DowncastErrorKind {
    LenMismatch(LenMismatchError),
    LayoutMismatch(LayoutMismatchError),
    InvalidLayout(LayoutError),
}

impl From<LenMismatchError> for DowncastErrorKind {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<LayoutMismatchError> for DowncastErrorKind {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<LayoutError> for DowncastErrorKind {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl Debug for DowncastErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }
        match self {
            Self::LenMismatch(error) => f.debug_tuple("LenMismatch").field(error).finish(),
            Self::LayoutMismatch(error) => f.debug_tuple("LayoutMismatch").field(error).finish(),
            Self::InvalidLayout(error) => f.debug_tuple("InvalidLayout").field(error).finish(),
        }
    }
}

impl Display for DowncastErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
        }
    }
}

impl Error for DowncastErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
        }
    }
}
