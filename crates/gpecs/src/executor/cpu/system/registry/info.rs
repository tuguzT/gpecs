use std::ops::{Deref, DerefMut};

use crate::executor::cpu::system::registry::SystemId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemInfo<Meta>
where
    Meta: ?Sized,
{
    system_id: SystemId,
    meta: Meta,
}

impl<Meta> SystemInfo<Meta> {
    #[inline]
    pub const fn new(system_id: SystemId, meta: Meta) -> Self {
        Self { system_id, meta }
    }

    #[inline]
    pub fn map_meta<F, N>(self, f: F) -> SystemInfo<N>
    where
        F: FnOnce(Meta) -> N,
    {
        let Self { system_id, meta } = self;

        let meta = f(meta);
        SystemInfo { system_id, meta }
    }

    #[inline]
    pub fn into_parts(self) -> (SystemId, Meta) {
        let Self { system_id, meta } = self;
        (system_id, meta)
    }

    #[inline]
    pub fn into_meta(self) -> Meta {
        let (_, meta) = self.into_parts();
        meta
    }
}

impl<Meta> SystemInfo<Meta>
where
    Meta: ?Sized,
{
    #[inline]
    pub const fn system_id(&self) -> SystemId {
        let Self { system_id, .. } = *self;
        system_id
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

impl<Meta> From<SystemInfo<Meta>> for (SystemId, Meta) {
    #[inline]
    fn from(info: SystemInfo<Meta>) -> Self {
        info.into_parts()
    }
}

impl<Meta> From<SystemInfo<Meta>> for SystemId {
    #[inline]
    fn from(info: SystemInfo<Meta>) -> Self {
        info.system_id()
    }
}

impl<Meta> Deref for SystemInfo<Meta>
where
    Meta: ?Sized,
{
    type Target = Meta;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_meta()
    }
}

impl<Meta> DerefMut for SystemInfo<Meta>
where
    Meta: ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_meta()
    }
}

impl<Meta, T> AsRef<T> for SystemInfo<Meta>
where
    T: ?Sized,
    <Self as Deref>::Target: AsRef<T>,
{
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl<Meta, T> AsMut<T> for SystemInfo<Meta>
where
    T: ?Sized,
    <Self as Deref>::Target: AsMut<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut().as_mut()
    }
}
