use core::{alloc::Layout, ops::Deref};

use gpecs_erased::layout::WithLayout;

use crate::{
    erased::{ErasedDrop, WithErasedDrop},
    registry::ComponentId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentInfo<Meta>
where
    Meta: ?Sized,
{
    component_id: ComponentId,
    meta: Meta,
}

impl<Meta> ComponentInfo<Meta> {
    #[inline]
    pub const fn new(component_id: ComponentId, meta: Meta) -> Self {
        Self { component_id, meta }
    }

    #[inline]
    pub fn map_meta<F, N>(self, f: F) -> ComponentInfo<N>
    where
        F: FnOnce(Meta) -> N,
    {
        let Self { component_id, meta } = self;

        let meta = f(meta);
        ComponentInfo { component_id, meta }
    }

    #[inline]
    pub fn into_parts(self) -> (ComponentId, Meta) {
        let Self { component_id, meta } = self;
        (component_id, meta)
    }
}

impl<Meta> ComponentInfo<Meta>
where
    Meta: ?Sized,
{
    #[inline]
    pub const fn component_id(&self) -> ComponentId {
        let Self { component_id, .. } = *self;
        component_id
    }

    #[inline]
    pub const fn as_meta(&self) -> &Meta {
        let Self { meta, .. } = self;
        meta
    }
}

impl<Meta> From<ComponentInfo<Meta>> for (ComponentId, Meta) {
    #[inline]
    fn from(info: ComponentInfo<Meta>) -> Self {
        info.into_parts()
    }
}

impl<Meta> From<ComponentInfo<Meta>> for ComponentId {
    #[inline]
    fn from(info: ComponentInfo<Meta>) -> Self {
        info.component_id()
    }
}

impl<Meta> Deref for ComponentInfo<Meta>
where
    Meta: ?Sized,
{
    type Target = Meta;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_meta()
    }
}

impl<Meta, T> AsRef<T> for ComponentInfo<Meta>
where
    T: ?Sized,
    <Self as Deref>::Target: AsRef<T>,
{
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl<Meta> WithLayout for ComponentInfo<Meta>
where
    Meta: WithLayout + ?Sized,
{
    #[inline]
    fn layout(&self) -> Layout {
        self.as_meta().layout()
    }
}

impl<Meta> WithErasedDrop for ComponentInfo<Meta>
where
    Meta: WithErasedDrop + ?Sized,
{
    #[inline]
    fn erased_drop(&self) -> Option<ErasedDrop> {
        self.as_meta().erased_drop()
    }
}
