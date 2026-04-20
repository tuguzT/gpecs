use core::{
    alloc::Layout,
    ops::{Deref, DerefMut},
};

use crate::{
    archetype::registry::ArchetypeId,
    component::erased::{ErasedDrop, WithErasedDrop},
    soa::layout::WithLayout,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArchetypeInfo<Meta>
where
    Meta: ?Sized,
{
    archetype_id: ArchetypeId,
    meta: Meta,
}

impl<Meta> ArchetypeInfo<Meta> {
    #[inline]
    pub const fn new(archetype_id: ArchetypeId, meta: Meta) -> Self {
        Self { archetype_id, meta }
    }

    #[inline]
    pub fn map_meta<F, N>(self, f: F) -> ArchetypeInfo<N>
    where
        F: FnOnce(Meta) -> N,
    {
        let Self { archetype_id, meta } = self;

        let meta = f(meta);
        ArchetypeInfo { archetype_id, meta }
    }

    #[inline]
    pub fn into_parts(self) -> (ArchetypeId, Meta) {
        let Self { archetype_id, meta } = self;
        (archetype_id, meta)
    }

    #[inline]
    pub fn into_meta(self) -> Meta {
        let (_, meta) = self.into_parts();
        meta
    }
}

impl<Meta> ArchetypeInfo<Meta>
where
    Meta: ?Sized,
{
    #[inline]
    pub const fn archetype_id(&self) -> ArchetypeId {
        let Self { archetype_id, .. } = *self;
        archetype_id
    }

    #[inline]
    pub const fn as_meta(&self) -> &Meta {
        let Self { meta, .. } = self;
        meta
    }

    #[inline]
    pub const fn as_mut_meta(&mut self) -> &mut Meta {
        let Self { meta, .. } = self;
        meta
    }
}

impl<Meta> From<ArchetypeInfo<Meta>> for (ArchetypeId, Meta) {
    #[inline]
    fn from(info: ArchetypeInfo<Meta>) -> Self {
        info.into_parts()
    }
}

impl<Meta> From<ArchetypeInfo<Meta>> for ArchetypeId {
    #[inline]
    fn from(info: ArchetypeInfo<Meta>) -> Self {
        info.archetype_id()
    }
}

impl<Meta> Deref for ArchetypeInfo<Meta>
where
    Meta: ?Sized,
{
    type Target = Meta;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_meta()
    }
}

impl<Meta> DerefMut for ArchetypeInfo<Meta>
where
    Meta: ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_meta()
    }
}

impl<Meta, T> AsRef<T> for ArchetypeInfo<Meta>
where
    T: ?Sized,
    <Self as Deref>::Target: AsRef<T>,
{
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl<Meta, T> AsMut<T> for ArchetypeInfo<Meta>
where
    T: ?Sized,
    <Self as Deref>::Target: AsMut<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut().as_mut()
    }
}

impl<Meta> WithLayout for ArchetypeInfo<Meta>
where
    Meta: WithLayout + ?Sized,
{
    #[inline]
    fn layout(&self) -> Layout {
        self.as_meta().layout()
    }
}

impl<Meta> WithErasedDrop for ArchetypeInfo<Meta>
where
    Meta: WithErasedDrop + ?Sized,
{
    #[inline]
    fn erased_drop(&self) -> Option<ErasedDrop> {
        self.as_meta().erased_drop()
    }
}
