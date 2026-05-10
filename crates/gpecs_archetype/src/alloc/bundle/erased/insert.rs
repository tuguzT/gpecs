use core::{
    alloc::LayoutError,
    error::Error,
    fmt::{self, Debug, Display},
    iter::chain,
};

use gpecs_soa_erased::{
    ErasedSoa, error::FromFieldsLayoutsError, ptr::slice::SliceItemPtrs,
    storage::AlignedStorageFromLayout,
};

use crate::{
    bundle::erased::{
        ErasedBundle, ErasedBundleKind,
        traits::{ErasedArchetypeKind, ErasedBundleDrop},
    },
    erased::{ErasedArchetype, error::AlreadyHasComponentError},
};

impl<T, D, S, P> ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind<Meta: Clone>,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorageFromLayout<Item: Clone>,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    #[expect(clippy::type_complexity)]
    // FIXME: can we optimize this?
    pub fn insert<ToInsert>(
        self,
        to_insert: ErasedBundleKind<ToInsert, D, S, P>,
    ) -> Result<
        ErasedBundle<T::Meta, D, S, P>,
        InsertError<Self, ErasedBundleKind<ToInsert, D, S, P>, S::Error>,
    >
    where
        ToInsert: ErasedArchetypeKind<Meta = T::Meta>,
    {
        if let Err(error) = self.archetype().has_no_components(to_insert.archetype()) {
            let error = InsertError {
                source: error.into(),
                bundle: self,
                to_insert,
            };
            return Err(error);
        }

        let refs = chain(self.as_refs(), to_insert.as_refs());
        let iter = chain(self.archetype(), to_insert.archetype())
            .map(|(component_id, desc)| (component_id, desc.clone()));
        let archetype = unsafe { ErasedArchetype::from_iter_unchecked(iter) };

        let result = ErasedSoa::try_from_fields_layouts(refs, archetype);
        let inner = match result.map_err(into_insert_error_kind) {
            Ok(inner) => inner,
            Err(source) => {
                let error = InsertError {
                    source,
                    bundle: self,
                    to_insert,
                };
                return Err(error);
            }
        };

        let _ = (self.into_inner(), to_insert.into_inner());
        let bundle = unsafe { ErasedBundle::from_inner(inner) };
        Ok(bundle)
    }
}

#[inline]
fn into_insert_error_kind<E>(error: FromFieldsLayoutsError<E>) -> InsertErrorKind<E> {
    match error {
        FromFieldsLayoutsError::FromLayout(error) => InsertErrorKind::FromLayout(error),
        FromFieldsLayoutsError::InvalidLayout(error) => error.into(),
        FromFieldsLayoutsError::LenMismatch(error) => {
            unreachable!("failed to insert some components into bundle: {error}")
        }
        FromFieldsLayoutsError::InsufficientAlign(error) => {
            unreachable!("failed to insert some components into bundle: {error}")
        }
    }
}

#[derive(Debug, Clone)]
pub struct InsertError<T, I, E> {
    pub source: InsertErrorKind<E>,
    pub bundle: T,
    pub to_insert: I,
}

impl<T, I, E> From<InsertError<T, I, E>> for InsertErrorKind<E> {
    #[inline]
    fn from(error: InsertError<T, I, E>) -> Self {
        let InsertError { source, .. } = error;
        source
    }
}

impl<T, I, E> Display for InsertError<T, I, E>
where
    T: Display,
    I: Display,
    E: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            source,
            bundle,
            to_insert,
        } = self;

        write!(f, "failed to insert {to_insert} into {bundle}: {source}")
    }
}

impl<T, I, E> Error for InsertError<T, I, E>
where
    T: Debug + Display,
    I: Debug + Display,
    E: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let Self { source, .. } = self;
        source.source()
    }

    #[expect(deprecated)]
    fn cause(&self) -> Option<&dyn Error> {
        let Self { source, .. } = self;
        source.cause()
    }
}

#[derive(Debug, Clone)]
pub enum InsertErrorKind<T> {
    AlreadyHasComponent(AlreadyHasComponentError),
    InvalidLayout(LayoutError),
    FromLayout(T),
}

impl<T> From<AlreadyHasComponentError> for InsertErrorKind<T> {
    #[inline]
    fn from(error: AlreadyHasComponentError) -> Self {
        Self::AlreadyHasComponent(error)
    }
}

impl<T> From<LayoutError> for InsertErrorKind<T> {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl<T> Display for InsertErrorKind<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyHasComponent(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for InsertErrorKind<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AlreadyHasComponent(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::AlreadyHasComponent(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}
