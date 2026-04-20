use crate::archetype::erased::{ErasedArchetype, ErasedArchetypeView};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErasedArchetypeCow<'a, Meta> {
    Borrowed(ErasedArchetypeView<'a, Meta>),
    Owned(ErasedArchetype<Meta>),
}

impl<Meta> ErasedArchetypeCow<'_, Meta> {
    #[inline]
    pub fn as_view(&self) -> ErasedArchetypeView<'_, Meta> {
        match *self {
            Self::Borrowed(archetype) => archetype,
            Self::Owned(ref archetype) => archetype.as_view(),
        }
    }
}

impl<Meta> ErasedArchetypeCow<'_, Meta>
where
    Meta: Clone,
{
    #[inline]
    pub fn into_owned(self) -> ErasedArchetype<Meta> {
        match self {
            Self::Borrowed(archetype) => archetype.into(),
            Self::Owned(archetype) => archetype,
        }
    }
}

impl<'a, Meta> From<ErasedArchetypeView<'a, Meta>> for ErasedArchetypeCow<'a, Meta> {
    #[inline]
    fn from(archetype: ErasedArchetypeView<'a, Meta>) -> Self {
        Self::Borrowed(archetype)
    }
}

impl<'a, Meta> From<&'a ErasedArchetype<Meta>> for ErasedArchetypeCow<'a, Meta> {
    #[inline]
    fn from(archetype: &'a ErasedArchetype<Meta>) -> Self {
        let archetype = archetype.as_view();
        Self::Borrowed(archetype)
    }
}

impl<Meta> From<ErasedArchetype<Meta>> for ErasedArchetypeCow<'_, Meta> {
    #[inline]
    fn from(archetype: ErasedArchetype<Meta>) -> Self {
        Self::Owned(archetype)
    }
}
