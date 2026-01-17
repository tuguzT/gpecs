use core::{
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::{
    error::{InsufficientAlignError, LayoutMismatchError, LenMismatchError, NotAlignedError},
    storage::{AddressableUnit, AlignedStorageFromLayout},
};

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
    pub(super) fn new(value: T, reason: LayoutMismatchError) -> Self {
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

pub enum ErasedFieldFromDescDataError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
{
    LenMismatch(LenMismatchError),
    FromLayout(T::Error),
}

impl<T, A> From<LenMismatchError> for ErasedFieldFromDescDataError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
{
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl<T, A> Clone for ErasedFieldFromDescDataError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::LenMismatch(error) => Self::LenMismatch(error.clone()),
            Self::FromLayout(error) => Self::FromLayout(error.clone()),
        }
    }
}

impl<T, A> Debug for ErasedFieldFromDescDataError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => f.debug_tuple("LenMismatch").field(error).finish(),
            Self::FromLayout(error) => f.debug_tuple("FromLayout").field(error).finish(),
        }
    }
}

impl<T, A> Display for ErasedFieldFromDescDataError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T, A> Error for ErasedFieldFromDescDataError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }
}

#[derive(Clone)]
#[non_exhaustive]
pub struct ErasedFieldFromValueError<B, T, A>
where
    A: AddressableUnit,
    B: AlignedStorageFromLayout<A>,
    T: ?Sized,
{
    pub reason: B::Error,
    pub value: T,
}

impl<B, T, A> Debug for ErasedFieldFromValueError<B, T, A>
where
    A: AddressableUnit,
    B: AlignedStorageFromLayout<A>,
    B::Error: Debug,
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, value } = self;
        f.debug_struct("ErasedFieldFromValueError")
            .field("reason", reason)
            .field("value", &value)
            .finish()
    }
}

impl<B, T, A> ErasedFieldFromValueError<B, T, A>
where
    A: AddressableUnit,
    B: AlignedStorageFromLayout<A>,
{
    #[inline]
    pub(crate) fn new(reason: B::Error, value: T) -> Self {
        Self { reason, value }
    }
}

impl<B, T, A> Display for ErasedFieldFromValueError<B, T, A>
where
    A: AddressableUnit,
    B: AlignedStorageFromLayout<A>,
    B::Error: Display,
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, value } = self;
        write!(f, "failed to convert {value}: {reason}")
    }
}

impl<B, T, A> Error for ErasedFieldFromValueError<B, T, A>
where
    A: AddressableUnit,
    B: AlignedStorageFromLayout<A>,
    B::Error: Debug + Display,
    T: Debug + Display + ?Sized,
{
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ErasedFieldFromBytesError<T>
where
    T: ?Sized,
{
    pub reason: ErasedFieldFromBytesErrorKind,
    pub bytes: T,
}

impl<T> ErasedFieldFromBytesError<T> {
    pub(crate) fn new(reason: ErasedFieldFromBytesErrorKind, bytes: T) -> Self {
        Self { reason, bytes }
    }
}

impl<T> Display for ErasedFieldFromBytesError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, bytes } = self;
        write!(f, "failed to create erased field with {bytes}: {reason}")
    }
}

impl<T> Error for ErasedFieldFromBytesError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Clone)]
pub enum ErasedFieldFromBytesErrorKind {
    NotAligned(NotAlignedError),
    LenMismatch(LenMismatchError),
    LayoutMismatch(LayoutMismatchError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotAlignedError> for ErasedFieldFromBytesErrorKind {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<LenMismatchError> for ErasedFieldFromBytesErrorKind {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<LayoutMismatchError> for ErasedFieldFromBytesErrorKind {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<InsufficientAlignError> for ErasedFieldFromBytesErrorKind {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl From<ErasedFieldPtrError> for ErasedFieldFromBytesErrorKind {
    #[inline]
    fn from(error: ErasedFieldPtrError) -> Self {
        match error {
            ErasedFieldPtrError::NotAligned(error) => Self::NotAligned(error),
            ErasedFieldPtrError::LenMismatch(error) => Self::LenMismatch(error),
            ErasedFieldPtrError::InsufficientAlign(error) => Self::InsufficientAlign(error),
        }
    }
}

impl Debug for ErasedFieldFromBytesErrorKind {
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

impl Display for ErasedFieldFromBytesErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAligned(error) => Display::fmt(error, f),
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedFieldFromBytesErrorKind {
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
