use core::{
    alloc::LayoutError,
    error::Error,
    fmt::{self, Debug, Display},
};

use crate::{
    error::{
        InsufficientAlignError, InsufficientLenError, LayoutMismatchError, LenMismatchError,
        NotAlignedError,
    },
    storage::{AddressableUnit, AlignedStorageFromLayout},
};

#[derive(Clone)]
pub struct InvalidOffsetError {
    offset: usize,
    capacity: usize,
}

impl InvalidOffsetError {
    #[inline]
    #[track_caller]
    pub fn new(offset: usize, capacity: usize) -> Self {
        assert!(offset > capacity, "offset should be greater than capacity");
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
    if offset > capacity {
        return Err(InvalidOffsetError::new(offset, capacity));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub enum ErasedSoaPtrsError {
    NotAligned(NotAlignedError),
    InvalidLayout(LayoutError),
    InvalidOffset(InvalidOffsetError),
    InsufficientLen(InsufficientLenError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotAlignedError> for ErasedSoaPtrsError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<LayoutError> for ErasedSoaPtrsError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<InvalidOffsetError> for ErasedSoaPtrsError {
    #[inline]
    fn from(error: InvalidOffsetError) -> Self {
        Self::InvalidOffset(error)
    }
}

impl From<InsufficientLenError> for ErasedSoaPtrsError {
    #[inline]
    fn from(error: InsufficientLenError) -> Self {
        Self::InsufficientLen(error)
    }
}

impl From<InsufficientAlignError> for ErasedSoaPtrsError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Display for ErasedSoaPtrsError {
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

impl Error for ErasedSoaPtrsError {
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
    pub fn new(offset: usize, len: usize, capacity: usize) -> Self {
        assert!(
            offset + len > capacity,
            "offset + len should be greater than capacity",
        );
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
    if offset + len > capacity {
        return Err(InvalidOffsetLenError::new(offset, len, capacity));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub enum ErasedSoaSlicePtrsError {
    NotAligned(NotAlignedError),
    InvalidLayout(LayoutError),
    InvalidOffset(InvalidOffsetError),
    InvalidOffsetLen(InvalidOffsetLenError),
    InsufficientLen(InsufficientLenError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<NotAlignedError> for ErasedSoaSlicePtrsError {
    #[inline]
    fn from(error: NotAlignedError) -> Self {
        Self::NotAligned(error)
    }
}

impl From<LayoutError> for ErasedSoaSlicePtrsError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<InvalidOffsetError> for ErasedSoaSlicePtrsError {
    #[inline]
    fn from(error: InvalidOffsetError) -> Self {
        Self::InvalidOffset(error)
    }
}

impl From<InvalidOffsetLenError> for ErasedSoaSlicePtrsError {
    #[inline]
    fn from(error: InvalidOffsetLenError) -> Self {
        Self::InvalidOffsetLen(error)
    }
}

impl From<InsufficientLenError> for ErasedSoaSlicePtrsError {
    #[inline]
    fn from(error: InsufficientLenError) -> Self {
        Self::InsufficientLen(error)
    }
}

impl From<InsufficientAlignError> for ErasedSoaSlicePtrsError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Display for ErasedSoaSlicePtrsError {
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

impl Error for ErasedSoaSlicePtrsError {
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
pub enum ErasedSoaFromStorageFieldsDescriptorsError {
    LenMismatch(IterOrFieldLenMismatchError),
    LayoutMismatch(LayoutMismatchError),
    InsufficientAlign(InsufficientAlignError),
    InvalidLayout(LayoutError),
}

impl From<IterOrFieldLenMismatchError> for ErasedSoaFromStorageFieldsDescriptorsError {
    #[inline]
    fn from(error: IterOrFieldLenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<LayoutMismatchError> for ErasedSoaFromStorageFieldsDescriptorsError {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<InsufficientAlignError> for ErasedSoaFromStorageFieldsDescriptorsError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl From<LayoutError> for ErasedSoaFromStorageFieldsDescriptorsError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl Display for ErasedSoaFromStorageFieldsDescriptorsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedSoaFromStorageFieldsDescriptorsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::LayoutMismatch(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
        }
    }
}

pub enum ErasedSoaFromFieldsDescriptorsError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
{
    LenMismatch(IterOrFieldLenMismatchError),
    InsufficientAlign(InsufficientAlignError),
    InvalidLayout(LayoutError),
    FromLayout(T::Error),
}

impl<T, A> From<IterOrFieldLenMismatchError> for ErasedSoaFromFieldsDescriptorsError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
{
    #[inline]
    fn from(value: IterOrFieldLenMismatchError) -> Self {
        Self::LenMismatch(value)
    }
}

impl<T, A> From<InsufficientAlignError> for ErasedSoaFromFieldsDescriptorsError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
{
    #[inline]
    fn from(value: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(value)
    }
}

impl<T, A> From<LayoutError> for ErasedSoaFromFieldsDescriptorsError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
{
    #[inline]
    fn from(value: LayoutError) -> Self {
        Self::InvalidLayout(value)
    }
}

impl<T, A> Debug for ErasedSoaFromFieldsDescriptorsError<T, A>
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
            Self::InvalidLayout(error) => f.debug_tuple("InvalidLayout").field(error).finish(),
            Self::FromLayout(error) => f.debug_tuple("FromLayout").field(error).finish(),
        }
    }
}

impl<T, A> Display for ErasedSoaFromFieldsDescriptorsError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Display,
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

impl<T, A> Clone for ErasedSoaFromFieldsDescriptorsError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::LenMismatch(error) => Self::LenMismatch(error.clone()),
            Self::InsufficientAlign(error) => Self::InsufficientAlign(error.clone()),
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

impl<T, A> Error for ErasedSoaFromFieldsDescriptorsError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Debug + Display,
{
}

#[derive(Clone)]
pub enum ErasedSoaFromStorageValueError {
    LayoutMismatch(LayoutMismatchError),
    InvalidLayout(LayoutError),
}

impl From<LayoutMismatchError> for ErasedSoaFromStorageValueError {
    #[inline]
    fn from(error: LayoutMismatchError) -> Self {
        Self::LayoutMismatch(error)
    }
}

impl From<LayoutError> for ErasedSoaFromStorageValueError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl Debug for ErasedSoaFromStorageValueError {
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

impl Display for ErasedSoaFromStorageValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LayoutMismatch(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ErasedSoaFromStorageValueError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LayoutMismatch(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
        }
    }
}

pub enum ErasedSoaFromValueError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
{
    InvalidLayout(LayoutError),
    FromLayout(T::Error),
}

impl<T, A> From<LayoutError> for ErasedSoaFromValueError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
{
    #[inline]
    fn from(value: LayoutError) -> Self {
        Self::InvalidLayout(value)
    }
}

impl<T, A> Clone for ErasedSoaFromValueError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Clone,
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

impl<T, A> Debug for ErasedSoaFromValueError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayout(error) => f.debug_tuple("InvalidLayout").field(error).finish(),
            Self::FromLayout(error) => f.debug_tuple("FromLayout").field(error).finish(),
        }
    }
}

impl<T, A> Display for ErasedSoaFromValueError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T, A> Error for ErasedSoaFromValueError<T, A>
where
    A: AddressableUnit,
    T: AlignedStorageFromLayout<A>,
    T::Error: Debug + Display,
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

    #[inline]
    pub fn map_value<U, F>(self, f: F) -> ErasedSoaIntoValueError<U>
    where
        F: FnOnce(T) -> U,
    {
        let Self { reason, value } = self;
        ErasedSoaIntoValueError::new(f(value), reason)
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
