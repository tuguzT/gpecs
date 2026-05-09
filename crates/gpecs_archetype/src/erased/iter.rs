use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_component::registry::{ComponentId, ComponentInfo};
use gpecs_soa_erased::CovariantFieldLayouts;
use gpecs_sparse::{
    iter::RawIter,
    soa::{
        field::{FieldLayouts, FieldLayoutsOutput},
        identity::Identity,
        layout::WithLayout,
    },
};

type Inner<'a, Meta> = RawIter<'a, u32, Identity<Meta>>;

#[repr(transparent)]
pub struct Iter<'a, Meta>
where
    Meta: 'a,
{
    inner: Inner<'a, Meta>,
}

impl<'a, Meta> Iter<'a, Meta> {
    #[inline]
    pub(crate) fn from_inner(inner: Inner<'a, Meta>) -> Self {
        Self { inner }
    }
}

impl<Meta> Debug for Iter<'_, Meta>
where
    Meta: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone().map(From::from);
        f.debug_map().entries(entries).finish()
    }
}

impl<Meta> Clone for Iter<'_, Meta> {
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<'a, Meta> Iterator for Iter<'a, Meta> {
    type Item = ComponentInfo<&'a Meta>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(inner_item_to_info_trusted)
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
        inner.nth(n).map(inner_item_to_info_trusted)
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.last().map(inner_item_to_info_trusted)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        let Self { inner } = self;
        inner.map(inner_item_to_info_trusted).collect()
    }
}

impl<Meta> DoubleEndedIterator for Iter<'_, Meta> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(inner_item_to_info_trusted)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(inner_item_to_info_trusted)
    }
}

impl<Meta> ExactSizeIterator for Iter<'_, Meta> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<Meta> FusedIterator for Iter<'_, Meta> {}

unsafe impl<Meta> Send for Iter<'_, Meta> where Meta: Sync {}
unsafe impl<Meta> Sync for Iter<'_, Meta> where Meta: Sync {}

impl<'a, Meta> FieldLayouts<'a> for Iter<'_, Meta>
where
    Meta: WithLayout + 'a,
{
    type Output = Iter<'a, Meta>;
    type OutputIter = Iter<'a, Meta>;
    type OutputItem = ComponentInfo<&'a Meta>;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        self.clone()
    }
}

impl<Meta> CovariantFieldLayouts for Iter<'_, Meta>
where
    Meta: WithLayout + 'static,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        from
    }
}

#[inline]
fn inner_item_to_info_trusted<'a, Meta>(
    (component_id, meta): (*const u32, *const Identity<Meta>),
) -> ComponentInfo<&'a Meta> {
    let component_id = unsafe { ComponentId::from_u32(*component_id) };
    let meta = unsafe { meta.as_ref_unchecked() }.as_inner();
    ComponentInfo::new(component_id, meta)
}
