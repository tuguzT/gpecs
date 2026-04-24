use std::{
    error::Error,
    fmt::{self, Display},
    time::Duration,
};

use wgpu::{CommandEncoderDescriptor, Device, Queue, SubmissionIndex};

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
    state: Option<MappedContextState<'a>>,
}

impl<'a> MappedContext<'a> {
    #[inline]
    pub(super) fn new(
        context: &'a mut Context,
        device: &'a Device,
        transfer_cache: &'a mut TransferCache,
        schedule_cache: Option<&'a mut ScheduleCache>,
        queue: &Queue,
        archetypes: &mut GpuArchetypeRegistry,
    ) -> Self {
        let state = schedule_cache.map(|schedule_cache| {
            MappedContextState::new(device, transfer_cache, schedule_cache, queue, archetypes)
        });
        Self { context, state }
    }

    #[inline]
    pub fn context(&mut self, poll_type: PollType) -> Result<&Context, MappedContextNotReadyError> {
        let Self { context, state } = self;

        if let Some(state) = state {
            state.make_ready(context, poll_type)?;
        }
        Ok(context)
    }

    // TODO: methods to copy data from CPU to GPU
    //       do not grant mutable access to the context (yet)
}

#[derive(Debug)]
struct MappedContextState<'a> {
    device: &'a Device,
    transfer_cache: &'a mut TransferCache,
    submission_index: SubmissionIndex,
    ready: bool,
}

impl<'a> MappedContextState<'a> {
    fn new(
        device: &'a Device,
        transfer_cache: &'a mut TransferCache,
        schedule_cache: &ScheduleCache,
        queue: &Queue,
        archetypes: &mut GpuArchetypeRegistry,
    ) -> Self {
        let command_encoder_desc = CommandEncoderDescriptor {
            label: Some("`gpecs` context download command encoder"),
        };
        let mut command_encoder = device.create_command_encoder(&command_encoder_desc);

        transfer_cache.download_from(device, &mut command_encoder, schedule_cache, archetypes);
        let command_buffer = command_encoder.finish();
        let submission_index = queue.submit([command_buffer]);

        Self {
            device,
            transfer_cache,
            submission_index,
            ready: false,
        }
    }

    fn make_ready(
        &mut self,
        context: &mut Context,
        poll_type: PollType,
    ) -> Result<(), MappedContextNotReadyError> {
        let Self {
            device,
            transfer_cache,
            submission_index,
            ready,
        } = self;

        if *ready {
            return Ok(());
        }

        let submission_index = Some(submission_index.clone());
        let poll_type = poll_type.with_index(submission_index);
        device
            .poll(poll_type)
            .expect("context download should be successful");

        let (_, _, _, archetypes) = unsafe { context.as_parts_mut() };
        transfer_cache
            .move_into(archetypes)
            .map_err(|_| MappedContextNotReadyError)?;

        *ready = true;
        Ok(())
    }
}

/// The same as [`wgpu::PollType`], but without [submission index](SubmissionIndex).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PollType {
    Wait { timeout: Option<Duration> },
    Poll,
}

impl PollType {
    #[inline]
    pub const fn wait_indefinitely() -> Self {
        Self::Wait { timeout: None }
    }

    #[inline]
    pub const fn is_wait(&self) -> bool {
        matches!(self, Self::Wait { .. })
    }

    #[inline]
    pub fn with_index(self, submission_index: Option<SubmissionIndex>) -> wgpu::PollType {
        match self {
            Self::Wait { timeout } => wgpu::PollType::Wait {
                submission_index,
                timeout,
            },
            Self::Poll => wgpu::PollType::Poll,
        }
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
