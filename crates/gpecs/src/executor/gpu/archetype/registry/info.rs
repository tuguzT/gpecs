use std::ops::{Deref, DerefMut};

use crate::executor::gpu::archetype::registry::GpuArchetypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuArchetypeInfo<Meta>
where
    Meta: ?Sized,
{
    archetype_id: GpuArchetypeId,
    meta: Meta,
}

impl<Meta> GpuArchetypeInfo<Meta> {
    #[inline]
    pub const fn new(archetype_id: GpuArchetypeId, meta: Meta) -> Self {
        Self { archetype_id, meta }
    }

    #[inline]
    pub fn map_meta<F, N>(self, f: F) -> GpuArchetypeInfo<N>
    where
        F: FnOnce(Meta) -> N,
    {
        let Self { archetype_id, meta } = self;

        let meta = f(meta);
        GpuArchetypeInfo { archetype_id, meta }
    }

    #[inline]
    pub fn into_parts(self) -> (GpuArchetypeId, Meta) {
        let Self { archetype_id, meta } = self;
        (archetype_id, meta)
    }

    #[inline]
    pub fn into_meta(self) -> Meta {
        let (_, meta) = self.into_parts();
        meta
    }
}

impl<Meta> GpuArchetypeInfo<Meta>
where
    Meta: ?Sized,
{
    #[inline]
    pub const fn archetype_id(&self) -> GpuArchetypeId {
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

impl<Meta> From<GpuArchetypeInfo<Meta>> for (GpuArchetypeId, Meta) {
    #[inline]
    fn from(info: GpuArchetypeInfo<Meta>) -> Self {
        info.into_parts()
    }
}

impl<Meta> From<GpuArchetypeInfo<Meta>> for GpuArchetypeId {
    #[inline]
    fn from(info: GpuArchetypeInfo<Meta>) -> Self {
        info.archetype_id()
    }
}

impl<Meta> Deref for GpuArchetypeInfo<Meta>
where
    Meta: ?Sized,
{
    type Target = Meta;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_meta()
    }
}

impl<Meta> DerefMut for GpuArchetypeInfo<Meta>
where
    Meta: ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_meta()
    }
}

impl<Meta, T> AsRef<T> for GpuArchetypeInfo<Meta>
where
    T: ?Sized,
    <Self as Deref>::Target: AsRef<T>,
{
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl<Meta, T> AsMut<T> for GpuArchetypeInfo<Meta>
where
    T: ?Sized,
    <Self as Deref>::Target: AsMut<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut().as_mut()
    }
}
