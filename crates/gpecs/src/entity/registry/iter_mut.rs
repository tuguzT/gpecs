use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_sparse::iter::IterMut as SparseIterMut;

use crate::{
    entity::Entity,
    soa::identity::{Identity, IdentitySlice},
};

type Inner<'a, Meta> = SparseIterMut<'a, 'a, Entity, Identity<Meta>>;

pub struct IterMut<'a, Meta>
where
    Meta: 'a,
{
    inner: Inner<'a, Meta>,
}

impl<'a, Meta> IterMut<'a, Meta> {
    #[inline]
    pub(super) fn from_inner(inner: Inner<'a, Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_entities(self) -> &'a [Entity] {
        let (entities, _) = self.into_slices();
        entities
    }

    #[inline]
    pub fn as_entities(&self) -> &[Entity] {
        let (entities, _) = self.as_slices();
        entities
    }

    #[inline]
    pub fn into_metas(self) -> &'a mut [Meta] {
        let (_, metas) = self.into_slices();
        metas
    }

    #[inline]
    pub fn as_metas(&self) -> &[Meta] {
        let (_, metas) = self.as_slices();
        metas
    }

    #[inline]
    pub fn into_slices(self) -> (&'a [Entity], &'a mut [Meta]) {
        let Self { inner } = self;

        let (entities, metas) = inner.into_slices();
        let metas = metas.as_inner_mut();
        (entities, metas)
    }

    #[inline]
    pub fn as_slices(&self) -> (&[Entity], &[Meta]) {
        let Self { inner } = self;

        let (entities, metas) = inner.as_slices();
        let metas = metas.as_inner();
        (entities, metas)
    }
}

impl<Meta> Debug for IterMut<'_, Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (entities, metas) = self.as_slices();
        f.debug_struct("Iter")
            .field("entities", &entities)
            .field("metas", &metas)
            .finish()
    }
}

impl<Meta> AsRef<[Meta]> for IterMut<'_, Meta> {
    #[inline]
    fn as_ref(&self) -> &[Meta] {
        self.as_metas()
    }
}

impl<'a, Meta> Iterator for IterMut<'a, Meta> {
    type Item = (Entity, &'a mut Meta);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(|(&entity, Identity(meta))| (entity, meta))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(|(&entity, Identity(meta))| (entity, meta))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(|(&entity, Identity(meta))| (entity, meta))
    }

    #[inline]
    fn for_each<F>(self, mut f: F)
    where
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.for_each(|(&entity, Identity(meta))| f((entity, meta)));
    }
}

impl<Meta> DoubleEndedIterator for IterMut<'_, Meta> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next_back()
            .map(|(&entity, Identity(meta))| (entity, meta))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .nth_back(n)
            .map(|(&entity, Identity(meta))| (entity, meta))
    }
}

impl<Meta> ExactSizeIterator for IterMut<'_, Meta> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for IterMut<'_, Meta> {}
