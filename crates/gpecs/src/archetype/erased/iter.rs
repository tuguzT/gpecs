use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_soa_erased::CovariantFieldDescriptors;
use gpecs_sparse::iter::Iter;

use crate::{
    component::registry::{ComponentId, ComponentInfo},
    soa::{
        field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsOutput},
        identity::Identity,
    },
};

type Inner<'a, Meta> = Iter<'a, 'a, u32, Identity<Meta>>;

#[repr(transparent)]
pub struct ErasedArchetypeIter<'a, Meta>
where
    Meta: 'a,
{
    inner: Inner<'a, Meta>,
}

impl<'a, Meta> ErasedArchetypeIter<'a, Meta> {
    #[inline]
    pub(super) fn from_inner(inner: Inner<'a, Meta>) -> Self {
        Self { inner }
    }
}

impl<Meta> Debug for ErasedArchetypeIter<'_, Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone().map(From::from);
        f.debug_map().entries(entries).finish()
    }
}

impl<Meta> Clone for ErasedArchetypeIter<'_, Meta> {
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<'a, Meta> Iterator for ErasedArchetypeIter<'a, Meta> {
    type Item = ComponentInfo<&'a Meta>;

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

impl<Meta> DoubleEndedIterator for ErasedArchetypeIter<'_, Meta> {
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

impl<Meta> ExactSizeIterator for ErasedArchetypeIter<'_, Meta> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for ErasedArchetypeIter<'_, Meta> {}

impl<'a, Meta> FieldDescriptors<'a> for ErasedArchetypeIter<'_, Meta>
where
    Meta: AsRef<FieldDescriptor>,
{
    type Output = Self;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.clone()
    }
}

impl<Meta> CovariantFieldDescriptors for ErasedArchetypeIter<'_, Meta>
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

#[inline]
fn inner_item_to_info<'a, Meta>(item: (&'a u32, &'a Identity<Meta>)) -> ComponentInfo<&'a Meta> {
    let (&id, meta) = item;

    let component_id = unsafe { ComponentId::from_u32(id) };
    let meta = meta.as_inner();
    ComponentInfo::new(component_id, meta)
}
