use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_sparse::{
    iter::RawIterMut,
    soa::identity::{Identity, IdentitySlice},
};

use crate::entity::Entity;

type Inner<'a, Meta> = RawIterMut<'a, Entity, Identity<Meta>>;

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

        let inner = unsafe { inner.clone().deref_mut() };
        let (entities, metas) = inner.into_slices();
        let metas = metas.as_inner_mut();
        (entities, metas)
    }

    #[inline]
    pub fn as_slices(&self) -> (&[Entity], &[Meta]) {
        let Self { inner } = self;

        let inner = unsafe { inner.clone().deref() };
        let (entities, metas) = inner.into_slices();
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
        inner.next().map(inner_item_to_item_trusted)
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
        inner.last().map(inner_item_to_item_trusted)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(inner_item_to_item_trusted)
    }

    #[inline]
    fn for_each<F>(self, f: F)
    where
        F: FnMut(Self::Item),
    {
        let Self { inner } = self;
        inner.map(inner_item_to_item_trusted).for_each(f);
    }
}

impl<Meta> DoubleEndedIterator for IterMut<'_, Meta> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(inner_item_to_item_trusted)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(inner_item_to_item_trusted)
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

#[inline]
fn inner_item_to_item_trusted<'a, Meta>(
    (entity, meta): (*mut Entity, *mut Identity<Meta>),
) -> (Entity, &'a mut Meta) {
    let entity = unsafe { *entity };
    let meta = unsafe { meta.as_mut_unchecked() }.as_inner_mut();
    (entity, meta)
}
