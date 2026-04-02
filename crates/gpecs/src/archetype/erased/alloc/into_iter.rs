use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_sparse::iter::{IntoIter, Iter};

use crate::{
    archetype::erased::ErasedArchetypeIter,
    component::registry::{ComponentId, ComponentInfo},
    soa::{
        field::{FieldDescriptor, FieldDescriptors},
        identity::Identity,
    },
};

type Inner<Meta> = IntoIter<u32, Identity<Meta>, Identity<Meta>>;

pub struct ErasedArchetypeIntoIter<Meta> {
    inner: Inner<Meta>,
}

impl<Meta> ErasedArchetypeIntoIter<Meta> {
    #[inline]
    pub(super) fn from_inner(inner: Inner<Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn iter(&self) -> ErasedArchetypeIter<'_, Meta> {
        let Self { inner } = self;

        let (context, components, metas) = inner.as_slices_with_context();
        let inner = Iter::new(context, components, metas);
        ErasedArchetypeIter::from_inner(inner)
    }
}

impl<Meta> Debug for ErasedArchetypeIntoIter<Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter().fmt(f)
    }
}

impl<Meta> Clone for ErasedArchetypeIntoIter<Meta>
where
    Meta: Clone,
{
    fn clone(&self) -> Self {
        let Self { inner } = self;
        let inner = inner.clone();
        Self { inner }
    }
}

impl<'a, Meta> IntoIterator for &'a ErasedArchetypeIntoIter<Meta> {
    type Item = ComponentInfo<&'a Meta>;
    type IntoIter = ErasedArchetypeIter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Meta> Iterator for ErasedArchetypeIntoIter<Meta> {
    type Item = ComponentInfo<Meta>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(inner_item_to_info)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(inner_item_to_info)
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last().map(inner_item_to_info)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.map(inner_item_to_info).collect()
    }
}

impl<Meta> DoubleEndedIterator for ErasedArchetypeIntoIter<Meta> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(inner_item_to_info)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(inner_item_to_info)
    }
}

impl<Meta> ExactSizeIterator for ErasedArchetypeIntoIter<Meta> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for ErasedArchetypeIntoIter<Meta> {}

impl<'a, Meta> FieldDescriptors<'a> for ErasedArchetypeIntoIter<Meta>
where
    Meta: AsRef<FieldDescriptor> + 'a,
{
    type Output = ErasedArchetypeIter<'a, Meta>;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.into_iter()
    }
}

#[inline]
fn inner_item_to_info<Meta>(item: (u32, Identity<Meta>)) -> ComponentInfo<Meta> {
    let (id, meta) = item;

    let component_id = unsafe { ComponentId::from_u32(id) };
    let meta = meta.into_inner();
    ComponentInfo::new(component_id, meta)
}
