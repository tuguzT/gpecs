use std::any::TypeId;

use wgpu::{
    BindGroupEntry, BindGroupLayoutEntry, CommandEncoder, CommandEncoderDescriptor, ComputePass,
    ComputePassDescriptor, Device, PollType, Queue, util::DispatchIndirectArgs,
};

use crate::{
    archetype::erased::error::{ArchetypeError, DuplicateComponentError},
    context::{ComponentDescriptor, Context},
    executor::gpu::{
        archetype::{
            registry::{GpuArchetypeId, GpuArchetypeRegistry},
            storage::GpuArchetypeStorage,
        },
        bundle::GpuBundle,
        cache::{schedule::ScheduleCache, transfer::TransferCache},
        component::{
            GpuComponent,
            registry::{GpuComponentId, GpuComponentRegistry},
        },
        context::ContextMapper,
        system::{
            registry::{GpuComponentAccess, GpuSystemDescriptor, GpuSystemId, GpuSystemRegistry},
            schedule::GpuSystemSchedule,
            shader::GpuSystemShader,
        },
        timestamp::{TimestampQueryError, TimestampQueryResources, TimestampQueryStatistics},
    },
};

#[derive(Debug)]
pub struct GpuExecutor<'entries, T>
where
    T: ?Sized,
{
    device: Device,
    components: GpuComponentRegistry,
    archetypes: GpuArchetypeRegistry,
    systems: GpuSystemRegistry,
    schedule: GpuSystemSchedule,
    schedule_cache: ScheduleCache<'entries>,
    transfer_cache: TransferCache,
    timestamp_query_resources: Option<TimestampQueryResources>,
    context: T,
}

impl<T> GpuExecutor<'_, T> {
    #[inline]
    pub fn new(context: T, device: Device) -> Self {
        Self {
            context,
            device,
            components: GpuComponentRegistry::new(),
            archetypes: GpuArchetypeRegistry::new(),
            systems: GpuSystemRegistry::new(),
            schedule: GpuSystemSchedule::new(),
            schedule_cache: ScheduleCache::default(),
            transfer_cache: TransferCache::default(),
            timestamp_query_resources: None,
        }
    }
}

impl<T> GpuExecutor<'_, T>
where
    T: ?Sized,
{
    #[inline]
    pub fn device(&self) -> &Device {
        let Self { device, .. } = self;
        device
    }

    #[inline]
    pub fn components(&self) -> &GpuComponentRegistry {
        let Self { components, .. } = self;
        components
    }

    #[inline]
    pub fn archetypes(&self) -> &GpuArchetypeRegistry {
        let Self { archetypes, .. } = self;
        archetypes
    }

    #[inline]
    pub fn systems(&self) -> &GpuSystemRegistry {
        let Self { systems, .. } = self;
        systems
    }

    #[inline]
    pub fn timestamp_query_resources(&self) -> Option<&TimestampQueryResources> {
        let Self {
            timestamp_query_resources,
            ..
        } = self;
        timestamp_query_resources.as_ref()
    }

    #[inline]
    pub fn components_mut(&mut self) -> &mut GpuComponentRegistry {
        let Self { components, .. } = self;
        components
    }

    #[inline]
    pub fn get_archetype_storage(
        &self,
        archetype_id: GpuArchetypeId,
    ) -> Option<&GpuArchetypeStorage> {
        let Self { archetypes, .. } = self;
        archetypes.get_archetype_storage(archetype_id)
    }

    #[inline]
    pub fn get_system_shader(&self, system_id: GpuSystemId) -> Option<&GpuSystemShader> {
        let Self { systems, .. } = self;
        systems.get_system_shader(system_id)
    }

    #[inline]
    pub fn add_system(&mut self, system_id: GpuSystemId) -> bool {
        let Self {
            schedule,
            schedule_cache,
            ..
        } = self;

        let added = schedule.add_system(system_id);
        if added {
            schedule_cache.request_system_resync(system_id);
        }

        added
    }

    #[inline]
    pub fn remove_system(&mut self, system_id: GpuSystemId) -> bool {
        let Self { schedule, .. } = self;
        schedule.remove_system(system_id)
    }

    pub fn timestamp_query_statistics(
        &self,
        queue: &Queue,
    ) -> Option<Result<TimestampQueryStatistics, TimestampQueryError>> {
        let Self {
            schedule,
            schedule_cache,
            timestamp_query_resources,
            ..
        } = self;

        let timestamp_query_resources = timestamp_query_resources.as_ref()?;
        let raw = match timestamp_query_resources.raw_statistics() {
            Ok(raw) => raw,
            Err(error) => return Some(Err(error)),
        };

        let statistics = TimestampQueryStatistics::new(&raw, schedule, schedule_cache, queue);
        Some(Ok(statistics))
    }
}

impl<'entries, T> GpuExecutor<'entries, T>
where
    T: AsRef<Context> + ?Sized,
{
    #[inline]
    pub fn get_component_descriptor(
        &self,
        component_id: GpuComponentId,
    ) -> Option<&ComponentDescriptor> {
        let Self { context, .. } = self;

        let context = context.as_ref();
        context.get_component_descriptor(component_id.into())
    }

    #[inline]
    pub fn component_id_from(&self, type_id: TypeId) -> Option<GpuComponentId> {
        let Self {
            context,
            components,
            ..
        } = self;

        let context = context.as_ref();
        let component_id = context.component_id_from(type_id)?;
        components.map_component_id(component_id)
    }

    #[inline]
    pub fn component_id<C>(&self) -> Option<GpuComponentId>
    where
        C: GpuComponent,
    {
        let Self {
            context,
            components,
            ..
        } = self;

        let context = context.as_ref();
        let component_id = context.component_id::<C>()?;
        components.map_component_id(component_id)
    }

    #[inline]
    pub fn archetype_id_of<B>(&self) -> Result<Option<GpuArchetypeId>, ArchetypeError>
    where
        B: GpuBundle,
    {
        let Self {
            context,
            archetypes,
            ..
        } = self;

        let context = context.as_ref();
        let Some(archetype_id) = context.archetype_id_of::<B>()? else {
            return Ok(None);
        };
        let archetype_id = archetypes.map_archetype_id(archetype_id);
        Ok(archetype_id)
    }

    #[inline]
    pub fn register_system<C, B>(
        &mut self,
        descriptor: GpuSystemDescriptor<C, B>,
    ) -> Result<GpuSystemId, ArchetypeError>
    where
        C: IntoIterator<Item = (GpuComponentId, GpuComponentAccess)>,
        B: IntoIterator<Item = BindGroupLayoutEntry>,
    {
        let Self {
            ref context,
            ref device,
            ref mut systems,
            ..
        } = *self;

        let context = context.as_ref();
        let components = context.components();
        systems.register_system(components, device, descriptor)
    }

    #[inline]
    pub fn set_additional_entries(
        &mut self,
        system_id: GpuSystemId,
        additional_entries: &'entries [BindGroupEntry<'_>],
    ) {
        let Self {
            ref context,
            ref device,
            ref archetypes,
            ref systems,
            ref mut schedule_cache,
            ..
        } = *self;

        schedule_cache.set_additional_entries(
            context.as_ref(),
            device,
            archetypes,
            systems,
            system_id,
            additional_entries,
        );
    }

    pub fn execute(&mut self, command_encoder: &mut CommandEncoder) {
        let Self {
            ref context,
            ref device,
            ref systems,
            ref schedule,
            ref mut archetypes,
            ref mut schedule_cache,
            ref mut transfer_cache,
            ref mut timestamp_query_resources,
            ..
        } = *self;

        let context = context.as_ref();
        let cpu_archetypes = context.archetypes();
        transfer_cache.resync(
            device,
            command_encoder,
            schedule_cache,
            cpu_archetypes,
            archetypes,
        );

        let updated = schedule_cache.resync(context, device, archetypes, systems, schedule);
        if updated || timestamp_query_resources.is_none() {
            *timestamp_query_resources = TimestampQueryResources::new(device, schedule_cache);
        }
        let timestamp_query_resources = timestamp_query_resources.as_ref();

        let write_timestamp = |compute_pass: &mut ComputePass, query_index| {
            if let Some(timestamp_query_resources) = timestamp_query_resources {
                let query_set = unsafe { timestamp_query_resources.query_set() };
                compute_pass.write_timestamp(query_set, query_index);
            }
        };

        let mut query_index = 0;
        for system_id in schedule {
            let Some(system_cache) = schedule_cache.system(system_id) else {
                unreachable!("{system_id} should exist")
            };
            let Some(system_shader) = systems.get_system_shader(system_id) else {
                unreachable!("{system_id} should exist");
            };

            let compute_pass_label = match system_shader.label() {
                Some(label) => format!("`gpecs` {system_id:#} [{label}] compute pass"),
                None => format!("`gpecs` {system_id:#} compute pass"),
            };
            let compute_pass_desc = ComputePassDescriptor {
                label: Some(&compute_pass_label),
                timestamp_writes: None,
            };
            let mut compute_pass = command_encoder.begin_compute_pass(&compute_pass_desc);
            compute_pass.set_pipeline(system_shader.compute_pipeline());

            for (archetype_id, archetype_cache) in system_cache.iter() {
                let Some(archetype_storage) = archetypes.get_archetype_storage(archetype_id) else {
                    unreachable!("{archetype_id} should exist");
                };

                compute_pass.set_bind_group(0, archetype_cache.bind_group(), &[]);

                write_timestamp(&mut compute_pass, query_index);
                query_index += 1;

                let len = archetype_storage
                    .len()
                    .try_into()
                    .expect("archetype storage len should fit into `u32`");
                let DispatchIndirectArgs { x, y, z } =
                    system_shader.dispatch_strategy().workgroup_count(len);
                compute_pass.dispatch_workgroups(x, y, z);

                write_timestamp(&mut compute_pass, query_index);
            }
            query_index += 1;
        }

        if let Some(timestamp_query_resources) = timestamp_query_resources {
            timestamp_query_resources.resolve(command_encoder);
        }
    }
}

impl<T> GpuExecutor<'_, T>
where
    T: AsMut<Context> + ?Sized,
{
    #[inline]
    pub fn register_component<C>(&mut self) -> GpuComponentId
    where
        C: GpuComponent,
    {
        let Self {
            context,
            components,
            ..
        } = self;

        let context = context.as_mut();
        components.register_component::<C>(context.components_mut())
    }

    #[inline]
    pub fn register_archetype_of<B>(&mut self) -> Result<GpuArchetypeId, DuplicateComponentError>
    where
        B: GpuBundle,
    {
        let Self {
            ref device,
            ref mut context,
            ref mut schedule_cache,
            components: ref mut gpu_components,
            archetypes: ref mut gpu_archetypes,
            ..
        } = *self;

        let context = context.as_mut();
        let (_, _, components, archetypes) = unsafe { context.as_parts_mut() };
        let archetype_id = gpu_archetypes.register_archetype_of::<B>(
            components,
            archetypes,
            gpu_components,
            device,
        )?;

        schedule_cache.request_archetype_resync(archetype_id);
        Ok(archetype_id)
    }

    #[inline]
    pub fn context_mapper(&mut self) -> ContextMapper<'_> {
        let Self {
            context,
            device,
            archetypes,
            schedule_cache,
            transfer_cache,
            ..
        } = self;

        let context = context.as_mut();
        ContextMapper::new(context, device, transfer_cache, schedule_cache, archetypes)
    }
}

impl<T> GpuExecutor<'_, T>
where
    T: AsMut<Context>,
{
    #[inline]
    pub fn into_context(mut self, queue: &Queue) -> T {
        let Self { ref device, .. } = self;
        let device = device.clone();

        let command_encoder_desc = CommandEncoderDescriptor {
            label: Some("`gpecs` context download command encoder"),
        };
        let mut command_encoder = device.create_command_encoder(&command_encoder_desc);

        let mut context_mapper = self.context_mapper();
        context_mapper.map_all(&mut command_encoder);

        let command_buffer = command_encoder.finish();
        let submission_index = queue.submit([command_buffer]);

        let poll_type = PollType::Wait {
            submission_index: Some(submission_index),
            timeout: None,
        };
        device
            .poll(poll_type)
            .expect("device should be polled successfully");

        context_mapper
            .get_all()
            .expect("all the data should be mapped successfully");

        let Self { context, .. } = self;
        context
    }
}
