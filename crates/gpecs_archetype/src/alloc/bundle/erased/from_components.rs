use core::{
    alloc::LayoutError,
    error::Error,
    fmt::{self, Debug, Display},
};

use gpecs_component::{erased::ErasedComponent, registry::ComponentRegistryView};
use gpecs_soa_erased::{
    ErasedSoa,
    error::FromFieldsLayoutsError,
    ptr::slice::SliceItemPtrs,
    storage::{AlignedStorage, AlignedStorageFromLayout},
};

use crate::{
    bundle::erased::{
        ErasedBundle,
        traits::{ErasedArchetypeMeta, ErasedBundleDrop},
    },
    erased::{ErasedArchetype, error::ArchetypeError},
};

pub trait FromErasedComponent<S, P>: Sized
where
    S: AlignedStorage,
    P: SliceItemPtrs<Item = S::Item>,
{
    fn from_erased_component(component: &ErasedComponent<S, P>) -> Self;
}

impl<Meta, D, S, P> ErasedBundle<Meta, D, S, P>
where
    Meta: ErasedArchetypeMeta + FromErasedComponent<S, P>,
    D: ErasedBundleDrop<Meta>,
    S: AlignedStorageFromLayout<Item: Clone>,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    pub fn from_components<I>(
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        iter: I,
    ) -> Result<Self, FromComponentsError<S::Error>>
    where
        I: IntoIterator<Item = ErasedComponent<S, P>>,
    {
        let iter = iter
            .into_iter()
            .map(|component| (component.component_id(), component));
        let components = ErasedArchetype::<_>::from_iter(components, iter)?;

        let iter = components.iter().map(|(component_id, component)| {
            let meta = Meta::from_erased_component(component);
            (component_id, meta)
        });
        let archetype = unsafe { ErasedArchetype::from_iter_unchecked(iter) };

        let fields = components
            .into_iter()
            .map(|(_, component)| component.into_field());

        let inner = ErasedSoa::try_from_fields_layouts(fields, archetype)
            .map_err(into_from_components_error)?;

        let me = unsafe { Self::from_inner(inner) };
        Ok(me)
    }
}

#[inline]
fn into_from_components_error<T>(error: FromFieldsLayoutsError<T>) -> FromComponentsError<T> {
    match error {
        FromFieldsLayoutsError::FromLayout(error) => FromComponentsError::FromLayout(error),
        FromFieldsLayoutsError::InvalidLayout(error) => error.into(),
        FromFieldsLayoutsError::LenMismatch(error) => {
            unreachable!("failed to create erased bundle from components: {error}")
        }
        FromFieldsLayoutsError::InsufficientAlign(error) => {
            unreachable!("failed to create erased bundle from components: {error}")
        }
    }
}

#[derive(Debug, Clone)]
pub enum FromComponentsError<T> {
    Archetype(ArchetypeError),
    InvalidLayout(LayoutError),
    FromLayout(T),
}

impl<T> From<ArchetypeError> for FromComponentsError<T> {
    #[inline]
    fn from(error: ArchetypeError) -> Self {
        Self::Archetype(error)
    }
}

impl<T> From<LayoutError> for FromComponentsError<T> {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl<T> Display for FromComponentsError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Archetype(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for FromComponentsError<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Archetype(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::Archetype(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}
