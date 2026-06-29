use std::{
    error::Error,
    fmt::{self, Display},
};

use wgpu::{CommandEncoder, Device};

use crate::{
    archetype::storage::ArchetypeStorage,
    context::{Components, Context},
    executor::gpu::{
        archetype::registry::{GpuArchetypeId, GpuArchetypeRegistry},
        cache::{schedule::ScheduleCache, transfer::TransferCache},
    },
};

#[derive(Debug)]
pub struct ContextMapper<'a, E> {
    context: &'a mut Context,
    device: &'a Device,
    schedule_cache: &'a ScheduleCache<E>,
    transfer_cache: &'a mut TransferCache,
    archetypes: &'a mut GpuArchetypeRegistry,
}

impl<'a, E> ContextMapper<'a, E> {
    #[inline]
    pub(super) fn new(
        context: &'a mut Context,
        device: &'a Device,
        transfer_cache: &'a mut TransferCache,
        schedule_cache: &'a ScheduleCache<E>,
        archetypes: &'a mut GpuArchetypeRegistry,
    ) -> Self {
        Self {
            context,
            device,
            schedule_cache,
            transfer_cache,
            archetypes,
        }
    }

    #[inline]
    pub fn components(&self) -> &Components {
        let Self { context, .. } = self;
        context.components()
    }

    #[inline]
    pub fn map_archetype(
        &mut self,
        archetype_id: GpuArchetypeId,
        command_encoder: &mut CommandEncoder,
    ) {
        let Self {
            device,
            schedule_cache,
            transfer_cache,
            archetypes,
            ..
        } = self;

        transfer_cache.download_archetype_from(
            device,
            command_encoder,
            archetype_id,
            schedule_cache,
            archetypes,
        );
    }

    #[inline]
    pub fn map_all(&mut self, command_encoder: &mut CommandEncoder) {
        let Self {
            device,
            schedule_cache,
            transfer_cache,
            archetypes,
            ..
        } = self;
        transfer_cache.download_all_from(device, command_encoder, schedule_cache, archetypes);
    }

    #[inline]
    pub fn get_archetype_with_components(
        &mut self,
        archetype_id: GpuArchetypeId,
    ) -> Result<(&ArchetypeStorage, &Components), MappedArchetypeNotReadyError> {
        let Self {
            context,
            transfer_cache,
            ..
        } = self;

        let (_, _, components, archetypes) = unsafe { context.as_parts_mut() };
        let storage = transfer_cache.move_archetype_into(archetype_id, archetypes)?;
        Ok((storage, components))
    }

    #[inline]
    pub fn get_archetype(
        &mut self,
        archetype_id: GpuArchetypeId,
    ) -> Result<&ArchetypeStorage, MappedArchetypeNotReadyError> {
        let (storage, _) = self.get_archetype_with_components(archetype_id)?;
        Ok(storage)
    }

    #[inline]
    pub fn get_mut_archetype_with_components(
        &mut self,
        archetype_id: GpuArchetypeId,
    ) -> Result<(&mut ArchetypeStorage, &Components), MappedArchetypeNotReadyError> {
        let Self {
            context,
            transfer_cache,
            ..
        } = self;

        let (_, _, components, archetypes) = unsafe { context.as_parts_mut() };
        let storage =
            transfer_cache.move_archetype_into_and_allow_mutation(archetype_id, archetypes)?;
        Ok((storage, components))
    }

    #[inline]
    pub fn get_mut_archetype(
        &mut self,
        archetype_id: GpuArchetypeId,
    ) -> Result<&mut ArchetypeStorage, MappedArchetypeNotReadyError> {
        let (storage, _) = self.get_mut_archetype_with_components(archetype_id)?;
        Ok(storage)
    }

    #[inline]
    pub fn get_all(&mut self) -> Result<&Context, MappedContextNotReadyError> {
        let Self {
            context,
            transfer_cache,
            ..
        } = self;

        let (_, _, _, archetypes) = unsafe { context.as_parts_mut() };
        transfer_cache
            .move_all_into(archetypes)
            .map_err(|_| MappedContextNotReadyError)?;

        Ok(context)
    }

    #[inline]
    pub fn get_mut_all(&mut self) -> Result<&mut Context, MappedContextNotReadyError> {
        let Self {
            context,
            transfer_cache,
            ..
        } = self;

        let (_, _, _, archetypes) = unsafe { context.as_parts_mut() };
        transfer_cache
            .move_all_into_and_allow_mutation(archetypes)
            .map_err(|_| MappedContextNotReadyError)?;

        Ok(context)
    }
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
