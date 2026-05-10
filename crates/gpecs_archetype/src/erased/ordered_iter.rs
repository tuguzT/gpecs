use core::{
    fmt::{self, Debug},
    iter::{Enumerate, FusedIterator},
    slice::Iter,
};

use gpecs_sparse::item::SparseItem;

use gpecs_component::registry::ComponentId;

pub struct ComponentIdOrderedIter<'a, Meta> {
    dense: &'a [Meta],
    sparse: Enumerate<Iter<'a, SparseItem<u32>>>,
}

impl<'a, Meta> ComponentIdOrderedIter<'a, Meta> {
    #[inline]
    pub(super) fn from_inner(dense: &'a [Meta], sparse: &'a [SparseItem<u32>]) -> Self {
        let sparse = sparse.iter().enumerate();
        Self { dense, sparse }
    }

    #[inline]
    fn component_from(
        dense: &'a [Meta],
        sparse_index: usize,
        dense_index: u32,
    ) -> (ComponentId, &'a Meta) {
        let id = sparse_index.try_into().expect("`ComponentId` overflow");
        let component_id = unsafe { ComponentId::from_u32(id) };

        let dense_index: usize = dense_index.try_into().expect("`ComponentId` overflow");
        let meta = &dense[dense_index];
        (component_id, meta)
    }
}

impl<Meta> Debug for ComponentIdOrderedIter<'_, Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_map().entries(entries).finish()
    }
}

impl<Meta> Clone for ComponentIdOrderedIter<'_, Meta> {
    fn clone(&self) -> Self {
        let Self { dense, sparse } = self;
        let sparse = sparse.clone();
        Self { dense, sparse }
    }
}

impl<'a, Meta> Iterator for ComponentIdOrderedIter<'a, Meta> {
    type Item = (ComponentId, &'a Meta);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut sparse,
            dense,
        } = *self;

        let (sparse_index, dense_index) = sparse.find_map(|(index, item)| {
            let dense_index = item.into_dense_index()?;
            Some((index, dense_index))
        })?;

        let item = Self::component_from(dense, sparse_index, dense_index);
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { dense, sparse } = self;

        let upper = usize::min(dense.len(), sparse.len());
        (0, Some(upper))
    }
}

impl<Meta> DoubleEndedIterator for ComponentIdOrderedIter<'_, Meta> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut sparse,
            dense,
        } = *self;

        let (sparse_index, dense_index) = sparse.rev().find_map(|(index, item)| {
            let dense_index = item.into_dense_index()?;
            Some((index, dense_index))
        })?;

        let item = Self::component_from(dense, sparse_index, dense_index);
        Some(item)
    }
}

impl<Meta> FusedIterator for ComponentIdOrderedIter<'_, Meta> {}
