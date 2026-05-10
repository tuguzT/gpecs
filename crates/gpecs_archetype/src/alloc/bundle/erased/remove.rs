use core::{
    error::Error,
    fmt::{self, Debug, Display},
};

use gpecs_soa_erased::{
    ErasedSoa,
    error::FromFieldsLayoutsError,
    ptr::slice::SliceItemPtrs,
    storage::{AlignedStorage, AlignedStorageFromLayout},
};
use itertools::zip_eq;

use crate::{
    bundle::erased::{
        ErasedBundle, ErasedBundleKind,
        traits::{ErasedArchetypeKind, ErasedBundleDrop},
    },
    erased::{ComponentIds, ErasedArchetype, ErasedArchetypeView, error::MissingComponentError},
};

pub struct RemovePair<ToRemove, D, S, P>
where
    ToRemove: ErasedArchetypeKind,
    D: ErasedBundleDrop<ToRemove::Meta>,
    S: AlignedStorageFromLayout,
    P: SliceItemPtrs<Item = S::Item>,
{
    pub retained: ErasedBundle<ToRemove::Meta, D, S, P>,
    pub removed: ErasedBundleKind<ToRemove, D, S, P>,
}

impl<ToRemove, D, S, P> Debug for RemovePair<ToRemove, D, S, P>
where
    ToRemove: ErasedArchetypeKind,
    D: ErasedBundleDrop<ToRemove::Meta>,
    S: AlignedStorageFromLayout<Item: Debug>,
    P: SliceItemPtrs<Item = S::Item>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { retained, removed } = self;
        f.debug_struct("RemovePair")
            .field("retained", retained)
            .field("removed", removed)
            .finish()
    }
}

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
    pub fn remove<ToRemove>(
        self,
        to_remove: ToRemove,
    ) -> Result<RemovePair<ToRemove, D, S, P>, RemoveError<Self, S::Error>>
    where
        ToRemove: ErasedArchetypeKind<Meta = T::Meta>,
    {
        let archetype_to_remove = to_remove.field_layouts();
        let bundle = self.check_remove(archetype_to_remove.component_ids())?;

        let retained_refs = bundle
            .as_refs()
            .into_iter()
            .filter(|component| !archetype_to_remove.contains(component.component_id()));
        let retained_iter = bundle
            .archetype()
            .into_iter()
            .filter(|&(component_id, _)| !archetype_to_remove.contains(component_id))
            .map(|(component_id, desc)| (component_id, desc.clone()));
        let retained_archetype = unsafe { ErasedArchetype::from_iter_unchecked(retained_iter) };
        let result = ErasedSoa::try_from_fields_layouts(retained_refs, retained_archetype);
        let retained_inner = match result.map_err(into_remove_error_kind) {
            Ok(inner) => inner,
            Err(source) => {
                let error = RemoveError { source, bundle };
                return Err(error);
            }
        };

        let removed_refs = bundle
            .as_refs()
            .into_iter()
            .filter(|component| archetype_to_remove.contains(component.component_id()));
        let result =
            ErasedSoa::<_, _, P>::try_from_fields_layouts(removed_refs, archetype_to_remove);
        let removed_inner = match result.map_err(into_remove_error_kind) {
            Ok(inner) => inner,
            Err(source) => {
                let error = RemoveError { source, bundle };
                return Err(error);
            }
        };
        let (removed_storage, _) = removed_inner.into_parts();
        let removed_inner = unsafe { ErasedSoa::from_parts(removed_storage, to_remove) };

        let _ = bundle.into_inner();
        let pair = RemovePair {
            retained: unsafe { ErasedBundleKind::from_inner(retained_inner) },
            removed: unsafe { ErasedBundleKind::from_inner(removed_inner) },
        };
        Ok(pair)
    }
}

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
    pub fn destroy(
        self,
        to_destroy: ErasedArchetypeView<impl Sized>,
    ) -> Result<ErasedBundle<T::Meta, D, S, P>, RemoveError<Self, S::Error>> {
        let mut bundle = self.check_remove(to_destroy.component_ids())?;

        let (refs, archetype) = bundle.as_mut_refs_with_archetype();
        let fields = zip_eq(refs, archetype).filter_map(|(mut field, (component_id, meta))| {
            if to_destroy.contains(component_id) {
                let to_drop = field.as_mut_component_ptr();
                unsafe { D::drop_in_place_with(to_drop, meta) };
                return None;
            }
            Some(field)
        });
        let iter = archetype.iter().filter_map(|(component_id, desc)| {
            if to_destroy.contains(component_id) {
                return None;
            }
            Some((component_id, desc.clone()))
        });
        let archetype = unsafe { ErasedArchetype::from_iter_unchecked(iter) };
        let result = ErasedSoa::try_from_fields_layouts(fields, archetype);
        let inner = match result.map_err(into_remove_error_kind) {
            Ok(inner) => inner,
            Err(source) => {
                let error = RemoveError { source, bundle };
                return Err(error);
            }
        };

        let _ = bundle.into_inner();
        let bundle = unsafe { ErasedBundle::from_inner(inner) };
        Ok(bundle)
    }
}

impl<T, D, S, P> ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorage,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    fn check_remove<E>(
        self,
        mut to_remove: ComponentIds<'_>,
    ) -> Result<Self, RemoveError<Self, E>> {
        if let Some(missing_component_id) = to_remove.find(|&id| !self.archetype().contains(id)) {
            let error = RemoveError {
                source: MissingComponentError::new(missing_component_id).into(),
                bundle: self,
            };
            return Err(error);
        }
        Ok(self)
    }
}

#[inline]
fn into_remove_error_kind<E>(error: FromFieldsLayoutsError<E>) -> RemoveErrorKind<E> {
    match error {
        FromFieldsLayoutsError::FromLayout(error) => RemoveErrorKind::FromLayout(error),
        FromFieldsLayoutsError::LenMismatch(error) => {
            unreachable!("failed to remove some components of bundle: {error}")
        }
        FromFieldsLayoutsError::InsufficientAlign(error) => {
            unreachable!("failed to remove some components of bundle: {error}")
        }
        FromFieldsLayoutsError::InvalidLayout(error) => {
            unreachable!("failed to remove some components of bundle: {error}")
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RemoveError<T, E> {
    pub source: RemoveErrorKind<E>,
    pub bundle: T,
}

impl<T, E> From<RemoveError<T, E>> for RemoveErrorKind<E> {
    #[inline]
    fn from(error: RemoveError<T, E>) -> Self {
        let RemoveError { source, .. } = error;
        source
    }
}

impl<T, E> Display for RemoveError<T, E>
where
    T: Display,
    E: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source, bundle } = self;
        write!(f, "failed to remove components from {bundle}: {source}")
    }
}

impl<T, E> Error for RemoveError<T, E>
where
    T: Debug + Display,
    E: Error,
{
    fn cause(&self) -> Option<&dyn Error> {
        let Self { source, .. } = self;
        Some(source)
    }
}

#[derive(Debug, Clone)]
pub enum RemoveErrorKind<T> {
    MissingComponent(MissingComponentError),
    FromLayout(T),
}

impl<T> From<MissingComponentError> for RemoveErrorKind<T> {
    #[inline]
    fn from(error: MissingComponentError) -> Self {
        Self::MissingComponent(error)
    }
}

impl<T> Display for RemoveErrorKind<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingComponent(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for RemoveErrorKind<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingComponent(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::MissingComponent(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}
