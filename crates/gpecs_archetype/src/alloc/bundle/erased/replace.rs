use core::{
    alloc::LayoutError,
    error::Error,
    fmt::{self, Debug, Display},
    iter::chain,
};

use gpecs_component::erased::WithErasedDrop;
use gpecs_soa_erased::{
    ErasedSoa, error::FromFieldsLayoutsError, ptr::slice::SliceItemPtrs,
    storage::AlignedStorageFromLayout,
};
use itertools::zip_eq;

use crate::{
    bundle::erased::{
        ErasedBundle, ErasedBundleKind,
        traits::{ErasedArchetypeKind, ErasedBundleDrop},
    },
    erased::ErasedArchetype,
};

impl<T, D, S, P> ErasedBundleKind<T, D, S, P>
where
    T: ErasedArchetypeKind<Meta: Clone + WithErasedDrop>,
    D: ErasedBundleDrop<T::Meta>,
    S: AlignedStorageFromLayout<Item: Clone>,
    P: SliceItemPtrs<Item = S::Item>,
{
    #[inline]
    #[expect(clippy::type_complexity)]
    // FIXME: can we optimize this?
    pub fn replace<ToReplace>(
        mut self,
        to_replace: ErasedBundleKind<ToReplace, D, S, P>,
    ) -> Result<
        ErasedBundle<T::Meta, D, S, P>,
        ReplaceError<Self, ErasedBundleKind<ToReplace, D, S, P>, S::Error>,
    >
    where
        ToReplace: ErasedArchetypeKind<Meta = T::Meta>,
    {
        let (ptrs, archetype) = self.as_mut_ptrs_with_archetype();

        let ptrs = zip_eq(ptrs, archetype).map(|(dst, component_info)| {
            if to_replace.archetype().contains(dst.component_id()) {
                if let Some(erased_drop) = component_info.erased_drop() {
                    unsafe { erased_drop.drop_in_place(dst) }
                }
                let src = to_replace
                    .as_ptrs()
                    .get(dst.component_id())
                    .expect("to replace archetype should contain component");
                unsafe { dst.copy_from_nonoverlapping(src, 1) }
            }
            dst.cast_const()
        });
        let ptrs_to_append = to_replace
            .as_ptrs()
            .into_iter()
            .filter(|ptr| !archetype.contains(ptr.component_id()));
        let refs = chain(ptrs, ptrs_to_append).map(|ptr| unsafe { ptr.as_ref_unchecked() });

        let metas_to_append = to_replace
            .archetype()
            .into_iter()
            .filter(|component_info| !archetype.contains(component_info.component_id()));
        let iter = chain(archetype, metas_to_append)
            .map(|component_info| component_info.map_meta(Clone::clone).into_parts());
        let archetype = unsafe { ErasedArchetype::from_iter_unchecked(iter) };

        let result = ErasedSoa::try_from_fields_layouts(refs, archetype);
        let inner = match result.map_err(into_replace_error_kind) {
            Ok(inner) => inner,
            Err(source) => {
                let error = ReplaceError {
                    source,
                    bundle: self,
                    to_replace,
                };
                return Err(error);
            }
        };

        let _ = (self.into_inner(), to_replace.into_inner());
        let bundle = unsafe { ErasedBundle::from_inner(inner) };
        Ok(bundle)
    }
}

#[inline]
fn into_replace_error_kind<E>(error: FromFieldsLayoutsError<E>) -> ReplaceErrorKind<E> {
    match error {
        FromFieldsLayoutsError::FromLayout(error) => ReplaceErrorKind::FromLayout(error),
        FromFieldsLayoutsError::InvalidLayout(error) => error.into(),
        FromFieldsLayoutsError::LenMismatch(error) => {
            unreachable!("failed to replace some components in bundle: {error}")
        }
        FromFieldsLayoutsError::InsufficientAlign(error) => {
            unreachable!("failed to replace some components in bundle: {error}")
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReplaceError<T, R, E> {
    pub source: ReplaceErrorKind<E>,
    pub bundle: T,
    pub to_replace: R,
}

impl<T, R, E> From<ReplaceError<T, R, E>> for ReplaceErrorKind<E> {
    #[inline]
    fn from(error: ReplaceError<T, R, E>) -> Self {
        let ReplaceError { source, .. } = error;
        source
    }
}

impl<T, R, E> Display for ReplaceError<T, R, E>
where
    T: Display,
    R: Display,
    E: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            source,
            bundle,
            to_replace,
        } = self;

        write!(f, "failed to replace {to_replace} in {bundle}: {source}")
    }
}

impl<T, R, E> Error for ReplaceError<T, R, E>
where
    T: Debug + Display,
    R: Debug + Display,
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
pub enum ReplaceErrorKind<T> {
    InvalidLayout(LayoutError),
    FromLayout(T),
}

impl<T> From<LayoutError> for ReplaceErrorKind<T> {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl<T> Display for ReplaceErrorKind<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::FromLayout(error) => Display::fmt(error, f),
        }
    }
}

impl<T> Error for ReplaceErrorKind<T>
where
    T: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(_) => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::InvalidLayout(error) => Some(error),
            Self::FromLayout(error) => Some(error),
        }
    }
}
