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
    InsufficientAlign(InsufficientAlignError),
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

impl<T, A> From<InsufficientAlignError> for ErasedFieldFromDescDataError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
{
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
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
            Self::InsufficientAlign(error) => Self::InsufficientAlign(error.clone()),
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
            Self::InsufficientAlign(error) => {
                f.debug_tuple("InsufficientAlign").field(error).finish()
            }
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
            Self::InsufficientAlign(error) => Display::fmt(error, f),
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
            Self::InsufficientAlign(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }
}

#[non_exhaustive]
pub struct ErasedFieldFromValueError<T, V, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    V: ?Sized,
{
    pub reason: ErasedFieldFromValueErrorKind<T, A>,
    pub value: V,
}

impl<T, V, A> ErasedFieldFromValueError<T, V, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
{
    #[inline]
    pub(crate) fn new(reason: ErasedFieldFromValueErrorKind<T, A>, value: V) -> Self {
        Self { reason, value }
    }
}

impl<T, V, A> Debug for ErasedFieldFromValueError<T, V, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Debug,
    V: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, value } = self;
        f.debug_struct("ErasedFieldFromValueError")
            .field("reason", reason)
            .field("value", &value)
            .finish()
    }
}

impl<T, V, A> Clone for ErasedFieldFromValueError<T, V, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Clone,
    V: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { reason, value } = self;
        Self {
            reason: reason.clone(),
            value: value.clone(),
        }
    }
}

impl<T, V, A> Display for ErasedFieldFromValueError<T, V, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Display,
    V: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, value } = self;
        write!(f, "failed to convert {value}: {reason}")
    }
}

impl<T, V, A> Error for ErasedFieldFromValueError<T, V, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Debug + Display,
    V: Debug + Display + ?Sized,
{
}

pub enum ErasedFieldFromValueErrorKind<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
{
    InsufficientAlign(InsufficientAlignError),
    FromLayout(T::Error),
}

impl<T, A> From<InsufficientAlignError> for ErasedFieldFromValueErrorKind<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
{
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl<T, A> Debug for ErasedFieldFromValueErrorKind<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientAlign(error) => {
                f.debug_tuple("InsufficientAlign").field(error).finish()
            }
            Self::FromLayout(error) => f.debug_tuple("FromLayout").field(error).finish(),
        }
    }
}

impl<T, A> Clone for ErasedFieldFromValueErrorKind<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Self::InsufficientAlign(error) => Self::InsufficientAlign(error.clone()),
            Self::FromLayout(error) => Self::FromLayout(error.clone()),
        }
    }
}

impl<T, A> Display for ErasedFieldFromValueErrorKind<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientAlign(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T, A> Error for ErasedFieldFromValueErrorKind<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Debug + Display + Error,
{
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
