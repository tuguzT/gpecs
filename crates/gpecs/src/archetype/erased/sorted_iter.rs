use std::{
    fmt::{self, Debug},
    iter::{Enumerate, FusedIterator},
    slice::Iter,
};

use gpecs_soa_erased::CovariantFieldDescriptors;
use gpecs_sparse::item::SparseItem;

use crate::{
    archetype::erased::ErasedArchetype,
    component::registry::{ComponentId, ComponentInfo},
    soa::{
        field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsOutput},
        identity::IdentitySlice,
    },
};

pub struct ErasedArchetypeSortedIter<'a, Meta> {
    dense: &'a [Meta],
    sparse: Enumerate<Iter<'a, SparseItem<u32>>>,
}

impl<'a, Meta> ErasedArchetypeSortedIter<'a, Meta> {
    #[inline]
    pub fn new(archetype: &'a ErasedArchetype<Meta>) -> Self {
        let dense = archetype.components.as_value_slices().as_inner();
        let sparse = archetype.components.as_sparse_slice().iter().enumerate();
        Self { dense, sparse }
    }

    #[inline]
    fn component_from(
        dense: &'a [Meta],
        sparse_index: usize,
        dense_index: u32,
    ) -> ComponentInfo<&'a Meta> {
        let id = sparse_index.try_into().expect("`ComponentId` overflow");
        let component_id = unsafe { ComponentId::from_u32(id) };

        let dense_index: usize = dense_index.try_into().expect("`ComponentId` overflow");
        let meta = &dense[dense_index];

        ComponentInfo::new(component_id, meta)
    }
}

impl<Meta> Debug for ErasedArchetypeSortedIter<'_, Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone().map(From::from);
        f.debug_map().entries(entries).finish()
    }
}

impl<Meta> Clone for ErasedArchetypeSortedIter<'_, Meta> {
    fn clone(&self) -> Self {
        let Self { dense, sparse } = self;
        let sparse = sparse.clone();
        Self { dense, sparse }
    }
}

impl<'a, Meta> Iterator for ErasedArchetypeSortedIter<'a, Meta> {
    type Item = ComponentInfo<&'a Meta>;

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

impl<Meta> DoubleEndedIterator for ErasedArchetypeSortedIter<'_, Meta> {
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

impl<Meta> FusedIterator for ErasedArchetypeSortedIter<'_, Meta> {}

impl<'a, Meta> FieldDescriptors<'a> for ErasedArchetypeSortedIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = Self;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.clone()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedArchetypeSortedIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        from
    }
}
