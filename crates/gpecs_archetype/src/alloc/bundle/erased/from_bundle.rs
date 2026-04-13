use core::{
    alloc::LayoutError,
    error::Error,
    fmt::{self, Debug, Display},
    mem::MaybeUninit,
};

use gpecs_component::registry::{
    ComponentRegistry,
    traits::{ComponentIdFromOrInsertWith, FromComponentType, PushBackArray},
};
use gpecs_soa_erased::{
    ErasedSoa,
    error::{FromDescriptorsValueError, FromDescriptorsValueErrorKind, InsufficientAlignError},
    ptr::slice::SliceItemPtrs,
    soa::field::FieldDescriptor,
    storage::AlignedStorageFromLayout,
};

use crate::{
    bundle::{
        Bundle,
        erased::{ErasedBundle, traits::ErasedBundleDrop},
    },
    erased::{ErasedArchetype, FromComponentInfo, error::DuplicateComponentError},
};

impl<Meta, D, S, P> ErasedBundle<Meta, D, S, P>
where
    Meta: AsRef<FieldDescriptor> + 'static,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorageFromLayout,
    P: SliceItemPtrs<Item = MaybeUninit<S::Item>>,
{
    #[inline]
    pub fn from_bundle<'a, B, M, T>(
        components: &'a mut ComponentRegistry<M, T>,
        bundle: B,
    ) -> Result<Self, FromBundleError<B, S::Error>>
    where
        B: Bundle,
        Meta: FromComponentInfo<'a, M::Item>,
        M: PushBackArray<Item: FromComponentType>,
        T: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let archetype = match ErasedArchetype::register::<B, M, T>(components) {
            Ok(archetype) => archetype,
            Err(source) => return Err(FromBundleError::new(bundle, source.into())),
        };
        let inner = ErasedSoa::try_from_descriptors_value::<B, B>(archetype, B::CONTEXT, bundle)
            .map_err(into_from_bundle_error)?;

        let me = unsafe { Self::from_inner(inner) };
        Ok(me)
    }
}

#[inline]
fn into_from_bundle_error<B, T>(error: FromDescriptorsValueError<B, T>) -> FromBundleError<B, T> {
    let FromDescriptorsValueError { value, source, .. } = error;
    let source = match source {
        FromDescriptorsValueErrorKind::FromLayout(error) => FromBundleErrorKind::FromLayout(error),
        FromDescriptorsValueErrorKind::InsufficientAlign(error) => error.into(),
        FromDescriptorsValueErrorKind::InvalidLayout(error) => error.into(),
        FromDescriptorsValueErrorKind::LenMismatch(error) => {
            unreachable!("failed to erase some bundle: {error}")
        }
        FromDescriptorsValueErrorKind::LayoutMismatch(error) => {
            unreachable!("failed to erase some bundle: {error}")
        }
    };
    FromBundleError::new(value, source)
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FromBundleError<B, T> {
    pub source: FromBundleErrorKind<T>,
    pub bundle: B,
}

impl<B, T> FromBundleError<B, T> {
    #[inline]
    fn new(bundle: B, source: FromBundleErrorKind<T>) -> Self {
        Self { source, bundle }
    }
}

impl<B, T> Display for FromBundleError<B, T>
where
    B: Display,
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, bundle } = self;
        write!(f, "failed to create erased bundle from {bundle}: {source}")
    }
}

impl<B, T> Error for FromBundleError<B, T>
where
    B: Debug + Display,
    T: Error,
{
    fn cause(&self) -> Option<&dyn Error> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[derive(Debug, Clone)]
pub enum FromBundleErrorKind<T> {
    DuplicateComponent(DuplicateComponentError),
    InsufficientAlign(InsufficientAlignError),
    InvalidLayout(LayoutError),
    FromLayout(T),
}

impl<T> From<InsufficientAlignError> for FromBundleErrorKind<T> {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl<T> From<LayoutError> for FromBundleErrorKind<T> {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl<T> From<DuplicateComponentError> for FromBundleErrorKind<T> {
    #[inline]
    fn from(error: DuplicateComponentError) -> Self {
        Self::DuplicateComponent(error)
    }
}

impl<T> Display for FromBundleErrorKind<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateComponent(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for FromBundleErrorKind<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::DuplicateComponent(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}
