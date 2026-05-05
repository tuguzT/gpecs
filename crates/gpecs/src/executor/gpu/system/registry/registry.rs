#![expect(clippy::module_inception)]

use std::num::NonZeroU32;

use wgpu::{BindGroupLayoutEntry, Device, Label, ShaderModule, util::DispatchIndirectArgs};

use crate::{
    archetype::erased::error::ArchetypeError,
    context::Components,
    executor::gpu::{
        component::registry::GpuComponentId,
        system::{
            registry::{
                GpuSystemId, GpuSystemIds,
                id::{gpu_system_id_from_usize, gpu_system_id_into_usize},
            },
            shader::GpuSystemShader,
        },
    },
};

pub const DEFAULT_WORKGROUP_SIZE: NonZeroU32 =
    NonZeroU32::new(64).expect("default workgroup size cannot be zero");

#[derive(Debug)]
#[non_exhaustive]
pub enum DispatchStrategy {
    Linear { workgroup_size: NonZeroU32 },
    // TODO: add custom strategy using boxed dyn trait object
}

impl DispatchStrategy {
    #[inline]
    pub fn workgroup_count(&self, len: u32) -> DispatchIndirectArgs {
        match self {
            Self::Linear { workgroup_size } => {
                let x = len.div_ceil(workgroup_size.get());
                DispatchIndirectArgs { x, y: 1, z: 1 }
            }
        }
    }
}

impl Default for DispatchStrategy {
    #[inline]
    fn default() -> Self {
        Self::Linear {
            workgroup_size: DEFAULT_WORKGROUP_SIZE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum GpuComponentAccess {
    ReadOnly,
    ReadWrite,
}

pub struct GpuSystemDescriptor<'a, Components, Bindings> {
    pub label: Label<'a>,
    pub shader_module: ShaderModule,
    pub entry_point: Option<&'a str>,
    pub dispatch_strategy: DispatchStrategy,
    pub bind_entities: bool,
    pub bind_components: Components,
    pub additional_bindings: Bindings,
}

#[derive(Debug, Default)]
pub struct GpuSystemRegistry {
    systems: Vec<GpuSystemShader>,
}

impl GpuSystemRegistry {
    #[inline]
    pub fn new() -> Self {
        let systems = Vec::new();
        Self { systems }
    }

    #[inline]
    pub fn register_system<C, B>(
        &mut self,
        components: &Components,
        gpu_device: &Device,
        descriptor: GpuSystemDescriptor<C, B>,
    ) -> Result<GpuSystemId, ArchetypeError>
    where
        C: IntoIterator<Item = (GpuComponentId, GpuComponentAccess)>,
        B: IntoIterator<Item = BindGroupLayoutEntry>,
    {
        let Self { systems } = self;

        let index = systems.len();
        let id = gpu_system_id_from_usize(index);

        let shader = GpuSystemShader::new(components, gpu_device, id, descriptor)?;
        systems.push(shader);

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
    pub fn get_system_shader(&self, system_id: GpuSystemId) -> Option<&GpuSystemShader> {
        let Self { systems } = self;
        systems.get(gpu_system_id_into_usize(system_id))
    }

    #[inline]
    pub fn system_ids(&self) -> GpuSystemIds {
        let index = self.len();
        let len = gpu_system_id_from_usize(index).into_u32();
        GpuSystemIds::new(0..len)
    }
}
