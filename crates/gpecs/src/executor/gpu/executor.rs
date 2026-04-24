use std::{any::TypeId, num::NonZeroU32};

use wgpu::{
    BindGroupEntry, BindGroupLayoutEntry, CommandEncoder, CommandEncoderDescriptor, ComputePass,
    ComputePassDescriptor, Device, PollType, Queue,
};

use crate::{
    archetype::erased::error::{ArchetypeError, DuplicateComponentError},
    context::{ComponentInfo, Context},
    executor::gpu::{
        archetype::{
            registry::{GpuArchetypeId, GpuArchetypeInfo, GpuArchetypeRegistry},
            storage::GpuArchetypeStorage,
        },
        bundle::GpuBundle,
        cache::GpuCache,
        component::{
            GpuComponent,
            registry::{GpuComponentId, GpuComponentRegistry},
        },
        system::{
            registry::{
                DEFAULT_WORKGROUP_SIZE, GpuComponentAccess, GpuSystemDescriptor, GpuSystemId,
                GpuSystemInfo, GpuSystemRegistry,
            },
            schedule::GpuSystemSchedule,
            shader::GpuSystemShader,
        },
        timestamp::{TimestampQueryError, TimestampQueryResources, TimestampQueryStatistics},
    },
};

#[derive(Debug)]
pub struct GpuExecutor<'ctx> {
    context: &'ctx mut Context,
    device: Device,
    components: GpuComponentRegistry,
    archetypes: GpuArchetypeRegistry,
    systems: GpuSystemRegistry,
    schedule: GpuSystemSchedule,
    cache: Option<GpuCache>,
    timestamp_query_resources: Option<TimestampQueryResources>,
}

impl<'ctx> GpuExecutor<'ctx> {
    #[inline]
    pub fn new(context: &'ctx mut Context, device: Device) -> Self {
        Self {
            context,
            device,
            components: GpuComponentRegistry::new(),
            archetypes: GpuArchetypeRegistry::new(),
            systems: GpuSystemRegistry::new(),
            schedule: GpuSystemSchedule::new(),
            cache: None,
            timestamp_query_resources: None,
        }
    }

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
    pub fn register_component<C>(&mut self) -> GpuComponentId
    where
        C: GpuComponent,
    {
        let Self {
            context,
            components,
            ..
        } = self;

        components.register_component::<C>(context.components_mut())
    }

    #[inline]
    pub fn get_component_info(&self, component_id: GpuComponentId) -> Option<ComponentInfo<'_>> {
        let Self { context, .. } = self;
        context.get_component_info(component_id.into())
    }

    #[inline]
    pub fn component_id_from(&self, type_id: TypeId) -> Option<GpuComponentId> {
        let Self {
            context,
            components,
            ..
        } = self;

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

        let component_id = context.component_id::<C>()?;
        components.map_component_id(component_id)
    }

    #[inline]
    pub fn register_archetype_of<B>(&mut self) -> Result<GpuArchetypeId, DuplicateComponentError>
    where
        B: GpuBundle,
    {
        let Self {
            ref mut context,
            ref device,
            components: ref mut gpu_components,
            archetypes: ref mut gpu_archetypes,
            ..
        } = *self;

        let (_, _, components, archetypes) = unsafe { context.as_parts_mut() };
        gpu_archetypes.register_archetype_of::<B>(components, archetypes, gpu_components, device)
    }

    #[inline]
    pub fn get_archetype_info(
        &self,
        archetype_id: GpuArchetypeId,
    ) -> Option<GpuArchetypeInfo<&GpuArchetypeStorage>> {
        let Self { archetypes, .. } = self;
        archetypes.get_archetype_info(archetype_id)
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

        let components = context.components();
        systems.register_system(components, device, descriptor)
    }

    #[inline]
    pub fn get_system_info(
        &self,
        system_id: GpuSystemId,
    ) -> Option<GpuSystemInfo<&GpuSystemShader>> {
        let Self { systems, .. } = self;
        systems.get_system_info(system_id)
    }

    #[inline]
    pub fn add_system(&mut self, system_id: GpuSystemId) -> bool {
        let Self { schedule, .. } = self;
        schedule.add_system(system_id)
    }

    #[inline]
    pub fn remove_system(&mut self, system_id: GpuSystemId) -> bool {
        let Self { schedule, .. } = self;
        schedule.remove_system(system_id)
    }

    #[inline]
    pub fn set_additional_bindings<'a, I, B>(&mut self, additional_bindings: I)
    where
        I: IntoIterator<Item = (GpuSystemId, B)>,
        B: IntoIterator<Item = BindGroupEntry<'a>>,
    {
        let Self {
            ref context,
            ref device,
            ref archetypes,
            ref systems,
            ref schedule,
            ref mut cache,
            ..
        } = *self;

        let new_cache = GpuCache::with_additional_bindings(
            context,
            device,
            archetypes,
            systems,
            schedule,
            additional_bindings,
        );
        cache.replace(new_cache);
    }

    pub fn execute(&mut self, command_encoder: &mut CommandEncoder) {
        let Self {
            ref context,
            ref device,
            ref archetypes,
            ref systems,
            ref schedule,
            ref mut cache,
            ref mut timestamp_query_resources,
            ..
        } = *self;

        let new_cache = || GpuCache::new(context, device, archetypes, systems, schedule);
        let cache = &*cache.get_or_insert_with(new_cache);

        if timestamp_query_resources.is_none() {
            *timestamp_query_resources = TimestampQueryResources::new(device, cache);
        }
        let timestamp_query_resources = timestamp_query_resources.as_ref();

        let write_timestamp = |compute_pass: &mut ComputePass, query_index| {
            if let Some(timestamp_query_resources) = timestamp_query_resources {
                let query_set = unsafe { timestamp_query_resources.query_set() };
                compute_pass.write_timestamp(query_set, query_index);
            }
        };

        let mut query_index = 0;
        for system_cache in cache.iter() {
            let system_id = system_cache.system_id();
            let Some(system_info) = systems.get_system_info(system_id) else {
                unreachable!("{system_id} should exist");
            };
            let shader = system_info.into_meta();

            let compute_pass_label = match shader.label() {
                Some(label) => format!("`gpecs` {system_id:#} [{label}] compute pass"),
                None => format!("`gpecs` {system_id:#} compute pass"),
            };
            let compute_pass_desc = ComputePassDescriptor {
                label: Some(&compute_pass_label),
                timestamp_writes: None,
            };
            let mut compute_pass = command_encoder.begin_compute_pass(&compute_pass_desc);
            compute_pass.set_pipeline(shader.compute_pipeline());

            for archetype_cache in system_cache.iter() {
                let archetype_id = archetype_cache.archetype_id();
                let Some(archetype_info) = archetypes.get_archetype_info(archetype_id) else {
                    unreachable!("{archetype_id} should exist");
                };

                compute_pass.set_bind_group(0, archetype_cache.bind_group(), &[]);

                write_timestamp(&mut compute_pass, query_index);
                query_index += 1;

                let archetype_storage = archetype_info.into_meta();
                let workgroup_size = shader.workgroup_size().unwrap_or(DEFAULT_WORKGROUP_SIZE);
                let workgroup_count = workgroup_count(archetype_storage, workgroup_size);
                compute_pass.dispatch_workgroups(workgroup_count, 1, 1);

                write_timestamp(&mut compute_pass, query_index);
            }
            query_index += 1;
        }

        if let Some(timestamp_query_resources) = timestamp_query_resources {
            timestamp_query_resources.resolve(command_encoder);
        }
    }

    pub fn timestamp_query_statistics(
        &self,
        queue: &Queue,
    ) -> Option<Result<TimestampQueryStatistics, TimestampQueryError>> {
        let Self {
            cache,
            timestamp_query_resources,
            ..
        } = self;

        let (cache, timestamp_query_resources) =
            cache.as_ref().zip(timestamp_query_resources.as_ref())?;

        let raw_statistics = match timestamp_query_resources.raw_statistics() {
            Ok(raw_statistics) => raw_statistics,
            Err(error) => return Some(Err(error)),
        };

        let statistics = TimestampQueryStatistics::new(&raw_statistics, cache, queue);
        Some(Ok(statistics))
    }

    pub fn into_context(mut self, queue: &Queue) -> &'ctx mut Context {
        let Self {
            context,
            ref device,
            ref mut archetypes,
            ref mut cache,
            ..
        } = self;

        if let Some(cache) = cache {
            let command_encoder_desc = CommandEncoderDescriptor {
                label: Some("`gpecs` context download command encoder"),
            };
            let mut command_encoder = device.create_command_encoder(&command_encoder_desc);

            cache.download_from(device, &mut command_encoder, archetypes);
            let command_buffer = command_encoder.finish();

            let submission_index = queue.submit([command_buffer]);
            cache.map_async_all(|_| {});

            let poll_type = PollType::Wait {
                submission_index: Some(submission_index),
                timeout: None,
            };
            device
                .poll(poll_type)
                .expect("context download should be successful");

            let (_, _, _, archetypes) = unsafe { context.as_parts_mut() };
            cache.move_into(archetypes);
        }

        context
    }

    // TODO: methods to copy data from CPU to GPU
    //       do not grant mutable access to the context (yet)
}

#[inline]
fn workgroup_count(archetype_storage: &GpuArchetypeStorage, workgroup_size: NonZeroU32) -> u32 {
    let storage_len = archetype_storage.len();
    let workgroup_size = workgroup_size
        .get()
        .try_into()
        .expect("workgroup size should fit into `usize`");
    storage_len
        .div_ceil(workgroup_size)
        .try_into()
        .expect("workgroup count should fit into `u32`")
}
