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
pub struct ErasedFieldIntoValueError<T>
where
    T: ?Sized,
{
    pub reason: LayoutMismatchError,
    pub value: T,
}

impl<T> ErasedFieldIntoValueError<T> {
    #[inline]
    fn new(value: T, reason: LayoutMismatchError) -> Self {
        Self { reason, value }
    }

    #[inline]
    pub fn map_value<U, F>(self, f: F) -> ErasedFieldIntoValueError<U>
    where
        F: FnOnce(T) -> U,
    {
        let Self { reason, value } = self;
        ErasedFieldIntoValueError::new(f(value), reason)
    }
}

impl<T> Display for ErasedFieldIntoValueError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, value } = self;
        write!(f, "failed to convert {value}: {reason}")
    }
}

impl<T> Error for ErasedFieldIntoValueError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[inline]
pub(crate) fn check_into_layout<T, U>(
    layout: Layout,
    value: U,
) -> Result<U, ErasedFieldIntoValueError<U>> {
    let expected = Layout::new::<T>();
    match check_layout(layout, expected) {
        Ok(()) => Ok(value),
        Err(reason) => Err(ErasedFieldIntoValueError::new(value, reason)),
    }
}

#[derive(Clone)]
pub enum ErasedFieldPtrError {
    NotAligned(NotAlignedError),
    LenMismatch(LenMismatchError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotAlignedError> for ErasedFieldPtrError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<LenMismatchError> for ErasedFieldPtrError {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<InsufficientAlignError> for ErasedFieldPtrError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Debug for ErasedFieldPtrError {
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

impl Display for ErasedFieldPtrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedFieldPtrError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotAligned(error) => Some(error),
            Self::LenMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ErasedFieldFromDescDataError<T> {
    LenMismatch(LenMismatchError),
    InsufficientAlign(InsufficientAlignError),
    FromLayout(T),
}

impl<T> From<LenMismatchError> for ErasedFieldFromDescDataError<T> {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl<T> From<InsufficientAlignError> for ErasedFieldFromDescDataError<T> {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl<T> Display for ErasedFieldFromDescDataError<T>
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

impl<T> Error for ErasedFieldFromDescDataError<T>
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
pub struct ErasedFieldFromValueError<T, V>
where
    V: ?Sized,
{
    pub reason: ErasedFieldFromValueErrorKind<T>,
    pub value: V,
}

impl<T, V> ErasedFieldFromValueError<T, V> {
    #[inline]
    pub(crate) fn new(reason: ErasedFieldFromValueErrorKind<T>, value: V) -> Self {
        Self { reason, value }
    }
}

impl<T, V> Display for ErasedFieldFromValueError<T, V>
where
    T: Display,
    V: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, value } = self;
        write!(f, "failed to convert {value}: {reason}")
    }
}

impl<T, V> Error for ErasedFieldFromValueError<T, V>
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
pub enum ErasedFieldFromValueErrorKind<T> {
    InsufficientAlign(InsufficientAlignError),
    FromLayout(T),
}

impl<T> From<InsufficientAlignError> for ErasedFieldFromValueErrorKind<T> {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl<T> Display for ErasedFieldFromValueErrorKind<T>
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

impl<T> Error for ErasedFieldFromValueErrorKind<T>
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
pub struct ErasedFieldFromStorageError<T>
where
    T: ?Sized,
{
    pub reason: ErasedFieldFromStorageErrorKind,
    pub storage: T,
}

impl<T> ErasedFieldFromStorageError<T> {
    pub(crate) fn new(reason: ErasedFieldFromStorageErrorKind, storage: T) -> Self {
        Self { reason, storage }
    }
}

impl<T> Display for ErasedFieldFromStorageError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, storage } = self;
        write!(f, "failed to create erased field with {storage}: {reason}")
    }
}

impl<T> Error for ErasedFieldFromStorageError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Clone)]
pub enum ErasedFieldFromStorageErrorKind {
    NotAligned(NotAlignedError),
    LenMismatch(LenMismatchError),
    LayoutMismatch(LayoutMismatchError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotAlignedError> for ErasedFieldFromStorageErrorKind {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<LenMismatchError> for ErasedFieldFromStorageErrorKind {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<LayoutMismatchError> for ErasedFieldFromStorageErrorKind {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<InsufficientAlignError> for ErasedFieldFromStorageErrorKind {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl From<ErasedFieldPtrError> for ErasedFieldFromStorageErrorKind {
    #[inline]
    fn from(error: ErasedFieldPtrError) -> Self {
        match error {
            ErasedFieldPtrError::NotAligned(error) => Self::NotAligned(error),
            ErasedFieldPtrError::LenMismatch(error) => Self::LenMismatch(error),
            ErasedFieldPtrError::InsufficientAlign(error) => Self::InsufficientAlign(error),
        }
    }
}

impl Debug for ErasedFieldFromStorageErrorKind {
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

impl Display for ErasedFieldFromStorageErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedFieldFromStorageErrorKind {
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
pub enum ErasedFieldSlicePtrError {
    NotAligned(NotAlignedError),
    LenMismatch(SliceLenMismatchError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotAlignedError> for ErasedFieldSlicePtrError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<SliceLenMismatchError> for ErasedFieldSlicePtrError {
    #[inline]
    fn from(error: SliceLenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<InsufficientAlignError> for ErasedFieldSlicePtrError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Debug for ErasedFieldSlicePtrError {
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

impl Display for ErasedFieldSlicePtrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedFieldSlicePtrError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotAligned(error) => Some(error),
            Self::LenMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}
