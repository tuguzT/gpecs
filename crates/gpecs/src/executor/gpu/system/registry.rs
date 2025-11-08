use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    num::NonZeroU32,
    ops::Range,
};

use wgpu::{BindGroupLayoutEntry, Device, ShaderModule};

use crate::{
    archetype::error::DuplicateComponentError, component::registry::ComponentRegistry,
    executor::gpu::component::registry::GpuComponentId,
};

use super::shader::GpuSystemShader;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct GpuSystemId(u32);

impl GpuSystemId {
    #[inline]
    pub const fn into_u32(&self) -> u32 {
        let Self(id) = *self;
        id
    }

    #[inline]
    pub const unsafe fn from_u32(id: u32) -> Self {
        Self(id)
    }
}

impl From<GpuSystemId> for u32 {
    #[inline]
    fn from(id: GpuSystemId) -> Self {
        id.into_u32()
    }
}

#[derive(Debug)]
pub struct GpuSystemInfo {
    id: GpuSystemId,
    shader: GpuSystemShader,
}

impl GpuSystemInfo {
    #[inline]
    pub fn id(&self) -> GpuSystemId {
        let Self { id, .. } = *self;
        id
    }

    #[inline]
    pub fn shader(&self) -> &GpuSystemShader {
        let Self { shader, .. } = self;
        shader
    }

    #[inline]
    pub fn shader_mut(&mut self) -> &mut GpuSystemShader {
        let Self { shader, .. } = self;
        shader
    }
}

pub const DEFAULT_WORKGROUP_SIZE: NonZeroU32 =
    NonZeroU32::new(64).expect("default workgroup size cannot be zero");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum GpuComponentAccess {
    ReadOnly,
    ReadWrite,
}

pub struct GpuSystemDescriptor<'entry_point, Components, Bindings> {
    pub shader_module: ShaderModule,
    pub workgroup_size: Option<NonZeroU32>,
    pub entry_point: Option<&'entry_point str>,
    pub bind_entities: bool,
    pub bind_components: Components,
    pub additional_bindings: Bindings,
}

#[derive(Debug, Default)]
pub struct GpuSystemRegistry {
    systems: Vec<GpuSystemInfo>,
}

impl GpuSystemRegistry {
    #[inline]
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    #[inline]
    pub fn register_system<C, B>(
        &mut self,
        components: &ComponentRegistry,
        gpu_device: &Device,
        descriptor: GpuSystemDescriptor<C, B>,
    ) -> Result<GpuSystemId, DuplicateComponentError>
    where
        C: IntoIterator<Item = (GpuComponentId, GpuComponentAccess)>,
        B: IntoIterator<Item = BindGroupLayoutEntry>,
    {
        let Self { systems } = self;

        let index = systems.len();
        let id = gpu_system_id_from_usize(index);

        let shader = GpuSystemShader::new(components, gpu_device, id, descriptor)?;
        let info = GpuSystemInfo { id, shader };
        systems.push(info);

        Ok(id)
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { systems } = self;
        systems.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { systems } = self;
        systems.is_empty()
    }

    #[inline]
    pub fn get_system_info(&self, id: GpuSystemId) -> Option<&GpuSystemInfo> {
        let Self { systems } = self;
        systems.get(gpu_system_id_into_usize(id))
    }

    #[inline]
    pub fn get_system_info_mut(&mut self, id: GpuSystemId) -> Option<&mut GpuSystemInfo> {
        let Self { systems } = self;
        systems.get_mut(gpu_system_id_into_usize(id))
    }

    #[inline]
    pub fn system_ids(&self) -> GpuSystemIds {
        let index = self.len();
        let len = gpu_system_id_from_usize(index).into_u32();
        GpuSystemIds { inner: 0..len }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GpuSystemIds {
    inner: Range<u32>,
}

impl GpuSystemIds {
    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { inner } = self;
        inner.is_empty()
    }
}

impl Debug for GpuSystemIds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;

        let Range { start, end } = *inner;
        let ids = gpu_system_id_trusted(start)..gpu_system_id_trusted(end);
        f.debug_struct("GpuSystemIds").field("ids", &ids).finish()
    }
}

impl Iterator for GpuSystemIds {
    type Item = GpuSystemId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(gpu_system_id_trusted)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        let Self { inner } = self;
        inner.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(gpu_system_id_trusted)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(gpu_system_id_trusted)
    }

    #[inline]
    fn min(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.min().map(gpu_system_id_trusted)
    }

    #[inline]
    fn max(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.max().map(gpu_system_id_trusted)
    }

    #[inline]
    fn is_sorted(self) -> bool {
        let Self { inner } = self;
        inner.is_sorted()
    }
}

impl DoubleEndedIterator for GpuSystemIds {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(gpu_system_id_trusted)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(gpu_system_id_trusted)
    }
}

impl ExactSizeIterator for GpuSystemIds {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for GpuSystemIds {}

#[inline]
fn gpu_system_id_from_usize(index: usize) -> GpuSystemId {
    let id = index.try_into().expect("`GpuSystemId` overflow");
    gpu_system_id_trusted(id)
}

#[inline]
fn gpu_system_id_into_usize(id: GpuSystemId) -> usize {
    let id = id.into_u32();
    id.try_into().expect("`GpuSystemId` overflow")
}

#[inline]
fn gpu_system_id_trusted(id: u32) -> GpuSystemId {
    unsafe { GpuSystemId::from_u32(id) }
}
