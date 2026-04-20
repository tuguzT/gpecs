use core::{
    alloc::LayoutError,
    error::Error,
    fmt::{self, Debug, Display},
};

pub use gpecs_erased::error::*;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
pub enum FromStorageFieldsLayoutsError {
    LenMismatch(IterOrFieldLenMismatchError),
    LayoutMismatch(LayoutMismatchError),
    InsufficientAlign(InsufficientAlignError),
    InvalidLayout(LayoutError),
}

impl From<IterOrFieldLenMismatchError> for FromStorageFieldsLayoutsError {
    #[inline]
    fn from(error: IterOrFieldLenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<LayoutMismatchError> for FromStorageFieldsLayoutsError {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<InsufficientAlignError> for FromStorageFieldsLayoutsError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl From<LayoutError> for FromStorageFieldsLayoutsError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl Display for FromStorageFieldsLayoutsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
        }
    }
}

impl Error for FromStorageFieldsLayoutsError {
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
pub enum FromFieldsLayoutsError<T> {
    LenMismatch(IterOrFieldLenMismatchError),
    InsufficientAlign(InsufficientAlignError),
    InvalidLayout(LayoutError),
    FromLayout(T),
}

impl<T> From<IterOrFieldLenMismatchError> for FromFieldsLayoutsError<T> {
    #[inline]
    fn from(value: IterOrFieldLenMismatchError) -> Self {
        Self::LenMismatch(value)
    }
}

impl<T> From<InsufficientAlignError> for FromFieldsLayoutsError<T> {
    #[inline]
    fn from(value: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(value)
    }
}

impl<T> From<LayoutError> for FromFieldsLayoutsError<T> {
    #[inline]
    fn from(value: LayoutError) -> Self {
        Self::InvalidLayout(value)
    }
}

impl<T> Display for FromFieldsLayoutsError<T>
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

impl<T> Error for FromFieldsLayoutsError<T>
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

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromStorageValueError<T>
where
    T: ?Sized,
{
    pub source: FromStorageValueErrorKind,
    pub value: T,
}

impl<T> FromStorageValueError<T> {
    #[inline]
    pub(crate) fn new(value: T, source: FromStorageValueErrorKind) -> Self {
        Self { source, value }
    }

    #[inline]
    pub fn map_value<U, F>(self, f: F) -> FromStorageValueError<U>
    where
        F: FnOnce(T) -> U,
    {
        let Self { source, value } = self;
        FromStorageValueError::new(f(value), source)
    }
}

impl<T> Display for FromStorageValueError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, value } = self;
        write!(f, "failed to create erased SoA from {value}: {source}")
    }
}

impl<T> Error for FromStorageValueError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[inline]
pub(super) fn check_from_storage_value<F, R, T>(
    f: F,
    value: T,
) -> Result<(T, R), FromStorageValueError<T>>
where
    F: FnOnce() -> Result<R, FromStorageValueErrorKind>,
{
    match f() {
        Ok(ok) => Ok((value, ok)),
        Err(source) => Err(FromStorageValueError::new(value, source)),
    }
}

#[derive(Debug, Clone)]
pub enum FromStorageValueErrorKind {
    LayoutMismatch(LayoutMismatchError),
    InvalidLayout(LayoutError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<LayoutMismatchError> for FromStorageValueErrorKind {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<LayoutError> for FromStorageValueErrorKind {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<InsufficientAlignError> for FromStorageValueErrorKind {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Display for FromStorageValueErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for FromStorageValueErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LayoutMismatch(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromLayoutsValueError<T, E>
where
    T: ?Sized,
{
    pub source: FromLayoutsValueErrorKind<E>,
    pub value: T,
}

impl<T, E> FromLayoutsValueError<T, E> {
    #[inline]
    pub(crate) fn new(value: T, source: FromLayoutsValueErrorKind<E>) -> Self {
        Self { source, value }
    }

    #[inline]
    pub fn map_value<U, F>(self, f: F) -> FromLayoutsValueError<U, E>
    where
        F: FnOnce(T) -> U,
    {
        let Self { source, value } = self;
        FromLayoutsValueError::new(f(value), source)
    }
}

impl<T, E> Display for FromLayoutsValueError<T, E>
where
    T: Display + ?Sized,
    E: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, value } = self;
        write!(f, "failed to create erased SoA from {value}: {source}")
    }
}

impl<T, E> Error for FromLayoutsValueError<T, E>
where
    T: Debug + Display + ?Sized,
    E: Error,
{
    fn cause(&self) -> Option<&dyn Error> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[inline]
pub(super) fn check_from_layouts_value<F, R, T, E>(
    f: F,
    value: T,
) -> Result<(T, R), FromLayoutsValueError<T, E>>
where
    F: FnOnce() -> Result<R, FromLayoutsValueErrorKind<E>>,
{
    match f() {
        Ok(ok) => Ok((value, ok)),
        Err(source) => Err(FromLayoutsValueError::new(value, source)),
    }
}

#[derive(Debug, Clone)]
pub enum FromLayoutsValueErrorKind<T> {
    LenMismatch(LenMismatchError),
    LayoutMismatch(LayoutMismatchError),
    InvalidLayout(LayoutError),
    InsufficientAlign(InsufficientAlignError),
    FromLayout(T),
}

impl<T> From<LenMismatchError> for FromLayoutsValueErrorKind<T> {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl<T> From<LayoutMismatchError> for FromLayoutsValueErrorKind<T> {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl<T> From<LayoutError> for FromLayoutsValueErrorKind<T> {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl<T> From<InsufficientAlignError> for FromLayoutsValueErrorKind<T> {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl<T> Display for FromLayoutsValueErrorKind<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for FromLayoutsValueErrorKind<T>
where
    T: Error,
{
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromValueError<T, E>
where
    T: ?Sized,
{
    pub source: FromValueErrorKind<E>,
    pub value: T,
}

impl<T, E> FromValueError<T, E> {
    #[inline]
    pub(crate) fn new(value: T, source: FromValueErrorKind<E>) -> Self {
        Self { source, value }
    }

    #[inline]
    pub fn map_value<U, F>(self, f: F) -> FromValueError<U, E>
    where
        F: FnOnce(T) -> U,
    {
        let Self { source, value } = self;
        FromValueError::new(f(value), source)
    }

    #[inline]
    pub fn into_source(self) -> FromValueErrorKind<E> {
        let Self { source, .. } = self;
        source
    }
}

impl<T, E> Display for FromValueError<T, E>
where
    T: Display + ?Sized,
    E: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, value } = self;
        write!(f, "failed to create erased SoA from {value}: {source}")
    }
}

impl<T, E> Error for FromValueError<T, E>
where
    T: Debug + Display + ?Sized,
    E: Error,
{
    fn cause(&self) -> Option<&dyn Error> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[inline]
pub(super) fn check_from_value<F, R, T, E>(f: F, value: T) -> Result<(T, R), FromValueError<T, E>>
where
    F: FnOnce() -> Result<R, FromValueErrorKind<E>>,
{
    match f() {
        Ok(ok) => Ok((value, ok)),
        Err(source) => Err(FromValueError::new(value, source)),
    }
}

#[derive(Debug, Clone)]
pub enum FromValueErrorKind<T> {
    InvalidLayout(LayoutError),
    InsufficientAlign(InsufficientAlignError),
    FromLayout(T),
}

impl<T> From<InsufficientAlignError> for FromValueErrorKind<T> {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl<T> From<LayoutError> for FromValueErrorKind<T> {
    #[inline]
    fn from(value: LayoutError) -> Self {
        Self::InvalidLayout(value)
    }
}

impl<T> Display for FromValueErrorKind<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for FromValueErrorKind<T>
where
    T: Error,
{
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::InvalidLayout(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidLayout(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DowncastError<T>
where
    T: ?Sized,
{
    pub source: DowncastErrorKind,
    pub value: T,
}

impl<T> DowncastError<T> {
    #[inline]
    pub(crate) fn new(value: T, source: DowncastErrorKind) -> Self {
        Self { source, value }
    }

    #[inline]
    pub fn map_value<U, F>(self, f: F) -> DowncastError<U>
    where
        F: FnOnce(T) -> U,
    {
        let Self { source, value } = self;
        DowncastError::new(f(value), source)
    }
}

impl<T> Display for DowncastError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, value } = self;
        write!(f, "failed to downcast {value}: {source}")
    }
}

impl<T> Error for DowncastError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[derive(Debug, Clone)]
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
