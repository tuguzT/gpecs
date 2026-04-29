use core::ops::{Deref, DerefMut};

use crate::executor::gpu::component::registry::GpuComponentId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuComponentInfo<Meta>
where
    Meta: ?Sized,
{
    component_id: GpuComponentId,
    meta: Meta,
}

impl<Meta> GpuComponentInfo<Meta> {
    #[inline]
    pub const fn new(component_id: GpuComponentId, meta: Meta) -> Self {
        Self { component_id, meta }
    }

    #[inline]
    pub fn map_meta<F, N>(self, f: F) -> GpuComponentInfo<N>
    where
        F: FnOnce(Meta) -> N,
    {
        let Self { component_id, meta } = self;

        let meta = f(meta);
        GpuComponentInfo { component_id, meta }
    }

    #[inline]
    pub fn into_parts(self) -> (GpuComponentId, Meta) {
        let Self { component_id, meta } = self;
        (component_id, meta)
    }

    #[inline]
    pub fn into_meta(self) -> Meta {
        let (_, meta) = self.into_parts();
        meta
    }
}

impl<Meta> GpuComponentInfo<Meta>
where
    Meta: ?Sized,
{
    #[inline]
    pub const fn component_id(&self) -> GpuComponentId {
        let Self { component_id, .. } = *self;
        component_id
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

impl<Meta> From<GpuComponentInfo<Meta>> for (GpuComponentId, Meta) {
    #[inline]
    fn from(info: GpuComponentInfo<Meta>) -> Self {
        info.into_parts()
    }
}

impl<Meta> From<GpuComponentInfo<Meta>> for GpuComponentId {
    #[inline]
    fn from(info: GpuComponentInfo<Meta>) -> Self {
        info.component_id()
    }
}

impl<Meta> Deref for GpuComponentInfo<Meta>
where
    Meta: ?Sized,
{
    type Target = Meta;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_meta()
    }
}

impl<Meta> DerefMut for GpuComponentInfo<Meta>
where
    Meta: ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_meta()
    }
}

impl<Meta, T> AsRef<T> for GpuComponentInfo<Meta>
where
    T: ?Sized,
    <Self as Deref>::Target: AsRef<T>,
{
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl<Meta, T> AsMut<T> for GpuComponentInfo<Meta>
where
    T: ?Sized,
    <Self as Deref>::Target: AsMut<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut().as_mut()
    }
}
