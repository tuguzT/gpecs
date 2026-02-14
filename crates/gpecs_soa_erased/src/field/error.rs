use core::{
    alloc::Layout,
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::error::{
    InsufficientAlignError, LayoutMismatchError, LenMismatchError, NotAlignedError, check_layout,
};

#[derive(Clone)]
pub struct SliceLenMismatchError {
    item_size: usize,
    len: usize,
    actual: usize,
}

impl SliceLenMismatchError {
    #[inline]
    pub fn new(item_size: usize, len: usize, actual: usize) -> Option<Self> {
        if (item_size == 0 && actual == 0) || (item_size * len == actual) {
            return None;
        }

        let me = unsafe { Self::new_unchecked(item_size, len, actual) };
        Some(me)
    }

    #[inline]
    pub unsafe fn new_unchecked(item_size: usize, len: usize, actual: usize) -> Self {
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
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

#[inline]
pub fn check_slice_len(
    len: usize,
    item_size: usize,
    expected_len: usize,
) -> Result<(), SliceLenMismatchError> {
    SliceLenMismatchError::new(item_size, expected_len, len).map_or(Ok(()), Err)
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DowncastError<T>
where
    T: ?Sized,
{
    pub reason: LayoutMismatchError,
    pub value: T,
}

impl<T> DowncastError<T> {
    #[inline]
    fn new(value: T, reason: LayoutMismatchError) -> Self {
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

#[inline]
pub(super) fn check_downcast<T, U>(layout: Layout, value: U) -> Result<U, DowncastError<U>> {
    let expected = Layout::new::<T>();
    match check_layout(layout, expected) {
        Ok(()) => Ok(value),
        Err(reason) => Err(DowncastError::new(value, reason)),
    }
}

#[derive(Clone)]
pub enum PtrError {
    NotAligned(NotAlignedError),
    LenMismatch(LenMismatchError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotAlignedError> for PtrError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<LenMismatchError> for PtrError {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<InsufficientAlignError> for PtrError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Debug for PtrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }
        match self {
            Self::NotAligned(error) => f.debug_tuple("NotAligned").field(error).finish(),
            Self::LenMismatch(error) => f.debug_tuple("LenMismatch").field(error).finish(),
            Self::InsufficientAlign(error) => {
                f.debug_tuple("InsufficientAlign").field(error).finish()
            }
        }
    }
}

impl Display for PtrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for PtrError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotAligned(error) => Some(error),
            Self::LenMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FromDescDataError<T> {
    LenMismatch(LenMismatchError),
    InsufficientAlign(InsufficientAlignError),
    FromLayout(T),
}

impl<T> From<LenMismatchError> for FromDescDataError<T> {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl<T> From<InsufficientAlignError> for FromDescDataError<T> {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl<T> Display for FromDescDataError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for FromDescDataError<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromValueError<T, V>
where
    V: ?Sized,
{
    pub reason: FromValueErrorKind<T>,
    pub value: V,
}

impl<T, V> FromValueError<T, V> {
    #[inline]
    pub(crate) fn new(reason: FromValueErrorKind<T>, value: V) -> Self {
        Self { reason, value }
    }
}

impl<T, V> Display for FromValueError<T, V>
where
    T: Display,
    V: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, value } = self;
        write!(f, "failed to convert {value}: {reason}")
    }
}

impl<T, V> Error for FromValueError<T, V>
where
    T: Error,
    V: Debug + Display + ?Sized,
{
    fn cause(&self) -> Option<&dyn Error> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Debug, Clone)]
pub enum FromValueErrorKind<T> {
    InsufficientAlign(InsufficientAlignError),
    FromLayout(T),
}

impl<T> From<InsufficientAlignError> for FromValueErrorKind<T> {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl<T> Display for FromValueErrorKind<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientAlign(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for FromValueErrorKind<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromStorageError<T>
where
    T: ?Sized,
{
    pub reason: FromStorageErrorKind,
    pub storage: T,
}

impl<T> FromStorageError<T> {
    pub(crate) fn new(reason: FromStorageErrorKind, storage: T) -> Self {
        Self { reason, storage }
    }
}

impl<T> Display for FromStorageError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, storage } = self;
        write!(f, "failed to create erased field with {storage}: {reason}")
    }
}

impl<T> Error for FromStorageError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Clone)]
pub enum FromStorageErrorKind {
    NotAligned(NotAlignedError),
    LenMismatch(LenMismatchError),
    LayoutMismatch(LayoutMismatchError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotAlignedError> for FromStorageErrorKind {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<LenMismatchError> for FromStorageErrorKind {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<LayoutMismatchError> for FromStorageErrorKind {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<InsufficientAlignError> for FromStorageErrorKind {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl From<PtrError> for FromStorageErrorKind {
    #[inline]
    fn from(error: PtrError) -> Self {
        match error {
            PtrError::NotAligned(error) => Self::NotAligned(error),
            PtrError::LenMismatch(error) => Self::LenMismatch(error),
            PtrError::InsufficientAlign(error) => Self::InsufficientAlign(error),
        }
    }
}

impl Debug for FromStorageErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }
        match self {
            Self::NotAligned(error) => f.debug_tuple("NotAligned").field(error).finish(),
            Self::LenMismatch(error) => f.debug_tuple("LenMismatch").field(error).finish(),
            Self::LayoutMismatch(error) => f.debug_tuple("LayoutMismatch").field(error).finish(),
            Self::InsufficientAlign(error) => {
                f.debug_tuple("InsufficientAlign").field(error).finish()
            }
        }
    }
}

impl Display for FromStorageErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for FromStorageErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotAligned(error) => Some(error),
            Self::LenMismatch(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}

#[derive(Clone)]
pub enum SlicePtrError {
    NotAligned(NotAlignedError),
    LenMismatch(SliceLenMismatchError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotAlignedError> for SlicePtrError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<SliceLenMismatchError> for SlicePtrError {
    #[inline]
    fn from(error: SliceLenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<InsufficientAlignError> for SlicePtrError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Debug for SlicePtrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return Display::fmt(self, f);
        }
        match self {
            Self::NotAligned(error) => f.debug_tuple("NotAligned").field(error).finish(),
            Self::LenMismatch(error) => f.debug_tuple("LenMismatch").field(error).finish(),
            Self::InsufficientAlign(error) => {
                f.debug_tuple("InsufficientAlign").field(error).finish()
            }
        }
    }
}

impl Display for SlicePtrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for SlicePtrError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotAligned(error) => Some(error),
            Self::LenMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}
