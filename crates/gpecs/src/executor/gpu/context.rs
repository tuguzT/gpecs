use wgpu::{CommandEncoderDescriptor, Device, PollType, Queue, SubmissionIndex};

use crate::{
    context::Context,
    executor::gpu::{archetype::registry::GpuArchetypeRegistry, cache::GpuCache},
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
        cache: Option<&'a mut GpuCache>,
        queue: &Queue,
        archetypes: &mut GpuArchetypeRegistry,
    ) -> Self {
        let state = cache.map(|cache| MappedContextState::new(device, cache, queue, archetypes));
        Self { context, state }
    }

    #[inline]
    pub fn context(&mut self) -> &Context {
        let Self { context, state } = self;

        if let Some(state) = state {
            state.make_ready(context);
        }
        context
    }

    // TODO: methods to copy data from CPU to GPU
    //       do not grant mutable access to the context (yet)
}

#[derive(Debug)]
struct MappedContextState<'a> {
    device: &'a Device,
    cache: &'a mut GpuCache,
    submission_index: SubmissionIndex,
}

impl<'a> MappedContextState<'a> {
    fn new(
        device: &'a Device,
        cache: &'a mut GpuCache,
        queue: &Queue,
        archetypes: &mut GpuArchetypeRegistry,
    ) -> Self {
        let command_encoder_desc = CommandEncoderDescriptor {
            label: Some("`gpecs` context download command encoder"),
        };
        let mut command_encoder = device.create_command_encoder(&command_encoder_desc);

        cache.download_from(device, &mut command_encoder, archetypes);
        let command_buffer = command_encoder.finish();

        let submission_index = queue.submit([command_buffer]);
        cache.map_async_all(|_| {
            // TODO: set atomic flag to true
        });

        Self {
            device,
            cache,
            submission_index,
        }
    }

    fn make_ready(&mut self, context: &mut Context) {
        let Self {
            device,
            cache,
            submission_index,
        } = self;

        // TODO: check for atomic flag

        let poll_type = PollType::Wait {
            submission_index: Some(submission_index.clone()),
            timeout: None,
        };
        device
            .poll(poll_type)
            .expect("context download should be successful");

        let (_, _, _, archetypes) = unsafe { context.as_parts_mut() };
        cache.move_into(archetypes);
    }
}
