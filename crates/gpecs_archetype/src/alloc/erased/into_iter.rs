use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr,
};

use gpecs_component::registry::ComponentId;
use gpecs_sparse::{
    iter::{IntoIter as SparseIntoIter, RawIter},
    soa::{field::FieldLayouts, identity::Identity, layout::WithLayout},
};

use crate::erased::Iter;

type Inner<Meta> = SparseIntoIter<u32, Identity<Meta>, Identity<Meta>>;

pub struct IntoIter<Meta> {
    inner: Inner<Meta>,
}

impl<Meta> IntoIter<Meta> {
    #[inline]
    pub(super) fn from_inner(inner: Inner<Meta>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, Meta> {
        let Self { inner } = self;

        let (context, components, metas) = inner.as_slices_with_context();
        let inner = RawIter::new(context, components, ptr::from_ref(metas));
        Iter::from_inner(inner)
    }
}

impl<Meta> Debug for IntoIter<Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter().fmt(f)
    }
}

impl<Meta> Clone for IntoIter<Meta>
where
    Meta: Clone,
{
    fn clone(&self) -> Self {
        let Self { inner } = self;
        let inner = inner.clone();
        Self { inner }
    }
}

impl<'a, Meta> IntoIterator for &'a IntoIter<Meta> {
    type Item = (ComponentId, &'a Meta);
    type IntoIter = Iter<'a, Meta>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Meta> Iterator for IntoIter<Meta> {
    type Item = (ComponentId, Meta);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(map_item)
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
        inner.nth(n).map(map_item)
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { mut inner } = self;
        inner.next_back().map(map_item)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.map(map_item).collect()
    }
}

impl<Meta> DoubleEndedIterator for IntoIter<Meta> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(map_item)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(map_item)
    }
}

impl<Meta> ExactSizeIterator for IntoIter<Meta> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for IntoIter<Meta> {}

impl<'a, Meta> FieldLayouts<'a> for IntoIter<Meta>
where
    Meta: WithLayout + 'a,
{
    type Output = Iter<'a, Meta>;
    type OutputIter = Iter<'a, Meta>;
    type OutputItem = (ComponentId, &'a Meta);

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        self.into_iter()
    }
}

#[inline]
fn map_item<Meta>(item: (u32, Identity<Meta>)) -> (ComponentId, Meta) {
    let (id, meta) = item;

    let component_id = unsafe { ComponentId::from_u32(id) };
    let meta = meta.into_inner();
    (component_id, meta)
}
