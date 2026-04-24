use std::{
    error::Error,
    fmt::{self, Display},
};

use wgpu::{CommandEncoder, Device};

use crate::{
    context::Context,
    executor::gpu::{
        archetype::registry::{GpuArchetypeId, GpuArchetypeRegistry},
        cache::{schedule::ScheduleCache, transfer::TransferCache},
    },
};

#[derive(Debug)]
pub struct MappedContext<'a> {
    context: &'a mut Context,
    transfer_cache: &'a mut TransferCache,
}

impl<'a> MappedContext<'a> {
    #[inline]
    pub(super) fn new(
        context: &'a mut Context,
        device: &Device,
        transfer_cache: &'a mut TransferCache,
        schedule_cache: &mut ScheduleCache,
        command_encoder: &mut CommandEncoder,
        archetypes: &mut GpuArchetypeRegistry,
    ) -> Self {
        transfer_cache.download_from(device, command_encoder, schedule_cache, archetypes);
        Self {
            context,
            transfer_cache,
        }
    }

    #[inline]
    pub fn context(&mut self) -> Result<&Context, MappedContextNotReadyError> {
        let Self {
            context,
            transfer_cache,
        } = self;

        let (_, _, _, archetypes) = unsafe { context.as_parts_mut() };
        transfer_cache
            .move_into(archetypes)
            .map_err(|_| MappedContextNotReadyError)?;

        Ok(context)
    }

    // TODO: methods to copy data from CPU to GPU
    //       do not grant mutable access to the context (yet)
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct MappedContextNotReadyError;

impl Display for MappedContextNotReadyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mapped context is not ready yet")
    }
}

impl Error for MappedContextNotReadyError {}

#[derive(Debug, Clone)]
pub struct MappedArchetypeNotReadyError {
    archetype_id: GpuArchetypeId,
}

impl MappedArchetypeNotReadyError {
    #[inline]
    pub(super) fn new(archetype_id: GpuArchetypeId) -> Self {
        Self { archetype_id }
    }

    #[inline]
    pub fn archetype_id(&self) -> GpuArchetypeId {
        let Self { archetype_id } = *self;
        archetype_id
    }
}

impl Display for MappedArchetypeNotReadyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { archetype_id } = self;
        write!(f, "mapped {archetype_id} is not ready yet")
    }
}

impl Error for MappedArchetypeNotReadyError {}
