use core::{
    alloc::LayoutError,
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::{
    aligned_bytes::AlignedBytesFromLayout,
    error::{LayoutMismatchError, LenMismatchError},
};

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
pub enum ErasedSoaFromBytesFieldsDescriptorsError {
    LenMismatch(IterOrFieldLenMismatchError),
    LayoutMismatch(LayoutMismatchError),
    InvalidLayout(LayoutError),
}

impl From<IterOrFieldLenMismatchError> for ErasedSoaFromBytesFieldsDescriptorsError {
    #[inline]
    fn from(value: IterOrFieldLenMismatchError) -> Self {
        Self::LenMismatch(value)
    }
}

impl From<LayoutMismatchError> for ErasedSoaFromBytesFieldsDescriptorsError {
    #[inline]
    fn from(value: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(value)
    }
}

impl From<LayoutError> for ErasedSoaFromBytesFieldsDescriptorsError {
    #[inline]
    fn from(value: LayoutError) -> Self {
        Self::InvalidLayout(value)
    }
}

impl Display for ErasedSoaFromBytesFieldsDescriptorsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedSoaFromBytesFieldsDescriptorsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
        }
    }
}

pub enum ErasedSoaFromFieldsDescriptorsError<B>
where
    B: AlignedBytesFromLayout,
{
    LenMismatch(IterOrFieldLenMismatchError),
    InvalidLayout(LayoutError),
    FromLayout(B::Error),
}

impl<B> From<IterOrFieldLenMismatchError> for ErasedSoaFromFieldsDescriptorsError<B>
where
    B: AlignedBytesFromLayout,
{
    #[inline]
    fn from(value: IterOrFieldLenMismatchError) -> Self {
        Self::LenMismatch(value)
    }
}

impl<B> From<LayoutError> for ErasedSoaFromFieldsDescriptorsError<B>
where
    B: AlignedBytesFromLayout,
{
    #[inline]
    fn from(value: LayoutError) -> Self {
        Self::InvalidLayout(value)
    }
}

impl<B> Debug for ErasedSoaFromFieldsDescriptorsError<B>
where
    B: AlignedBytesFromLayout,
    B::Error: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => f.debug_tuple("LenMismatch").field(error).finish(),
            Self::InvalidLayout(error) => f.debug_tuple("InvalidLayout").field(error).finish(),
            Self::FromLayout(error) => f.debug_tuple("FromLayout").field(error).finish(),
        }
    }
}

impl<B> Display for ErasedSoaFromFieldsDescriptorsError<B>
where
    B: AlignedBytesFromLayout,
    B::Error: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<B> Clone for ErasedSoaFromFieldsDescriptorsError<B>
where
    B: AlignedBytesFromLayout,
    B::Error: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::LenMismatch(error) => Self::LenMismatch(error.clone()),
            Self::InvalidLayout(error) => Self::InvalidLayout(error.clone()),
            Self::FromLayout(error) => Self::FromLayout(error.clone()),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Self::FromLayout(me), Self::FromLayout(source)) => me.clone_from(source),
            (me, source) => *me = source.clone(),
        }
    }
}

impl<B> Error for ErasedSoaFromFieldsDescriptorsError<B>
where
    B: AlignedBytesFromLayout,
    B::Error: Debug + Display,
{
}

#[derive(Clone)]
pub enum ErasedSoaFromBytesValueError {
    LayoutMismatch(LayoutMismatchError),
    InvalidLayout(LayoutError),
}

impl From<LayoutMismatchError> for ErasedSoaFromBytesValueError {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<LayoutError> for ErasedSoaFromBytesValueError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl Debug for ErasedSoaFromBytesValueError {
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

impl Display for ErasedSoaFromBytesValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedSoaFromBytesValueError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LayoutMismatch(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
        }
    }
}

pub enum ErasedSoaFromValueError<B>
where
    B: AlignedBytesFromLayout,
{
    InvalidLayout(LayoutError),
    FromLayout(B::Error),
}

impl<B> From<LayoutError> for ErasedSoaFromValueError<B>
where
    B: AlignedBytesFromLayout,
{
    #[inline]
    fn from(value: LayoutError) -> Self {
        Self::InvalidLayout(value)
    }
}

impl<B> Clone for ErasedSoaFromValueError<B>
where
    B: AlignedBytesFromLayout,
    B::Error: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::InvalidLayout(arg0) => Self::InvalidLayout(arg0.clone()),
            Self::FromLayout(arg0) => Self::FromLayout(arg0.clone()),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Self::FromLayout(me), Self::FromLayout(source)) => me.clone_from(source),
            (me, source) => *me = source.clone(),
        }
    }
}

impl<B> Debug for ErasedSoaFromValueError<B>
where
    B: AlignedBytesFromLayout,
    B::Error: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayout(error) => f.debug_tuple("InvalidLayout").field(error).finish(),
            Self::FromLayout(error) => f.debug_tuple("FromLayout").field(error).finish(),
        }
    }
}

impl<B> Display for ErasedSoaFromValueError<B>
where
    B: AlignedBytesFromLayout,
    B::Error: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<B> Error for ErasedSoaFromValueError<B>
where
    B: AlignedBytesFromLayout,
    B::Error: Debug + Display,
{
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ErasedSoaIntoValueError<T>
where
    T: ?Sized,
{
    pub reason: ErasedSoaIntoValueErrorKind,
    pub value: T,
}

impl<T> ErasedSoaIntoValueError<T> {
    #[inline]
    pub(crate) fn new(value: T, reason: ErasedSoaIntoValueErrorKind) -> Self {
        Self { reason, value }
    }
}

impl<T> Display for ErasedSoaIntoValueError<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { reason, value } = self;
        write!(f, "failed to convert {value}: {reason}")
    }
}

impl<T> Error for ErasedSoaIntoValueError<T>
where
    T: Debug + Display + ?Sized,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { reason, .. } = self;
        Some(reason)
    }
}

#[derive(Clone)]
pub enum ErasedSoaIntoValueErrorKind {
    LenMismatch(LenMismatchError),
    LayoutMismatch(LayoutMismatchError),
    InvalidLayout(LayoutError),
}

impl From<LenMismatchError> for ErasedSoaIntoValueErrorKind {
    #[inline]
    fn from(error: LenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<LayoutMismatchError> for ErasedSoaIntoValueErrorKind {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<LayoutError> for ErasedSoaIntoValueErrorKind {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl Debug for ErasedSoaIntoValueErrorKind {
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

impl Display for ErasedSoaIntoValueErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedSoaIntoValueErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
        }
    }
}
