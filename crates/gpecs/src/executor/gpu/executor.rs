use std::{any::TypeId, num::NonZeroU32};

use indexmap::IndexMap;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, Buffer, BufferDescriptor,
    BufferUsages, CommandEncoder, ComputePassDescriptor, Device, Features, QuerySet,
    QuerySetDescriptor, QueryType,
};

use crate::{
    archetype::error::{DuplicateComponentError, GetComponentsError},
    component::registry::ComponentInfo,
    context::Context,
};

use super::{
    archetype::{
        registry::{GpuArchetypeId, GpuArchetypeInfo, GpuArchetypeRegistry},
        storage::GpuArchetypeStorageBufferSlices,
    },
    bundle::GpuBundle,
    component::{
        GpuComponent,
        registry::{GpuComponentId, GpuComponentRegistry},
    },
    system::{
        registry::{GpuSystemDescriptor, GpuSystemId, GpuSystemInfo, GpuSystemRegistry},
        schedule::GpuSystemSchedule,
    },
};

type ScheduleCache = IndexMap<GpuSystemId, SystemCache>;
type SystemCache = IndexMap<GpuArchetypeId, BindGroup>;

#[derive(Debug)]
pub struct TimestampQueryResources {
    query_set: QuerySet,
    count: NonZeroU32,
    resolve_buffer: Buffer,
}

impl TimestampQueryResources {
    #[inline]
    pub unsafe fn query_set(&self) -> &QuerySet {
        let Self { query_set, .. } = self;
        query_set
    }

    #[inline]
    pub fn count(&self) -> NonZeroU32 {
        let Self { count, .. } = self;
        *count
    }

    #[inline]
    pub unsafe fn resolve_buffer(&self) -> &Buffer {
        let Self { resolve_buffer, .. } = self;
        resolve_buffer
    }
}

#[derive(Debug)]
pub struct GpuExecutor<'context> {
    context: &'context mut Context,
    device: Device,
    components: GpuComponentRegistry,
    archetypes: GpuArchetypeRegistry,
    systems: GpuSystemRegistry,
    schedule: GpuSystemSchedule,
    schedule_cache: Option<ScheduleCache>,
    timestamp_query_resources: Option<TimestampQueryResources>,
}

impl<'context> GpuExecutor<'context> {
    #[inline]
    pub fn new(context: &'context mut Context, device: Device) -> Self {
        Self {
            context,
            device,
            components: GpuComponentRegistry::new(),
            archetypes: GpuArchetypeRegistry::new(),
            systems: GpuSystemRegistry::new(),
            schedule: GpuSystemSchedule::new(),
            schedule_cache: None,
            timestamp_query_resources: None,
        }
    }

    #[inline]
    pub fn context(&self) -> &Context {
        let Self { context, .. } = self;
        context
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
    pub fn into_context(self) -> &'context mut Context {
        let Self { context, .. } = self;
        context
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
    pub fn get_component_info(&self, component_id: GpuComponentId) -> Option<&ComponentInfo> {
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
    pub fn register_archetype<B>(&mut self) -> Result<GpuArchetypeId, DuplicateComponentError>
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

        gpu_archetypes.register_archetype::<B>(components, archetypes, gpu_components, device)
    }

    #[inline]
    pub fn get_archetype_info(&self, archetype_id: GpuArchetypeId) -> Option<&GpuArchetypeInfo> {
        let Self { archetypes, .. } = self;
        archetypes.get_archetype_info(archetype_id)
    }

    #[inline]
    pub fn archetype_id<B>(&self) -> Result<Option<GpuArchetypeId>, GetComponentsError>
    where
        B: GpuBundle,
    {
        let Self {
            context,
            archetypes,
            ..
        } = self;

        let Some(archetype_id) = context.archetype_id::<B>()? else {
            return Ok(None);
        };
        let archetype_id = archetypes.map_archetype_id(archetype_id);
        Ok(archetype_id)
    }

    #[inline]
    pub fn register_system<C, B>(
        &mut self,
        descriptor: GpuSystemDescriptor<C, B>,
    ) -> Result<GpuSystemId, DuplicateComponentError>
    where
        C: IntoIterator<Item = GpuComponentId>,
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
    pub fn get_system_info(&self, system_id: GpuSystemId) -> Option<&GpuSystemInfo> {
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
            ref mut schedule_cache,
            ..
        } = *self;

        let new_cache = cache_schedule(
            context,
            device,
            archetypes,
            systems,
            schedule,
            additional_bindings,
        );
        schedule_cache.replace(new_cache);
    }

    #[inline]
    pub fn execute(&mut self, command_encoder: &mut CommandEncoder) {
        let Self {
            ref context,
            ref device,
            ref archetypes,
            ref mut systems,
            ref mut schedule,
            ref mut schedule_cache,
            ref mut timestamp_query_resources,
            ..
        } = *self;

        let cache_schedule = || cache_schedule_pure(context, device, archetypes, systems, schedule);
        let schedule_cache = &*schedule_cache.get_or_insert_with(cache_schedule);

        let can_write_timestamps = device
            .features()
            .contains(Features::TIMESTAMP_QUERY_INSIDE_PASSES);
        if can_write_timestamps && timestamp_query_resources.is_none() {
            *timestamp_query_resources = create_timestamp_query_resources(device, schedule_cache);
        }

        let compute_pass_desc = ComputePassDescriptor {
            label: Some("`gpecs` executor compute pass"),
            timestamp_writes: None,
        };
        let mut compute_pass = command_encoder.begin_compute_pass(&compute_pass_desc);

        let mut query_index = 0;
        for (&system_id, archetypes_bind_groups) in schedule_cache {
            let Some(system_info) = systems.get_system_info(system_id) else {
                unreachable!("system {system_id:?} should exist");
            };
            let shader = system_info.shader();
            for (&archetype_id, bind_group) in archetypes_bind_groups {
                let Some(archetype_info) = archetypes.get_archetype_info(archetype_id) else {
                    unreachable!("archetype {archetype_id:?} should exist");
                };
                compute_pass.set_pipeline(shader.compute_pipeline());
                compute_pass.set_bind_group(0, bind_group, &[]);

                let storage_len = u32::try_from(archetype_info.storage().len())
                    .expect("storage length should fit into `u32`");
                let workgroup_count = storage_len.div_ceil(shader.workgroup_count().unwrap_or(64));

                if let Some(timestamp_query_resources) = timestamp_query_resources.as_ref() {
                    let TimestampQueryResources { query_set, .. } = timestamp_query_resources;
                    compute_pass.write_timestamp(query_set, query_index);
                    query_index += 1;
                }
                compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
                if let Some(timestamp_query_resources) = timestamp_query_resources.as_ref() {
                    let TimestampQueryResources { query_set, .. } = timestamp_query_resources;
                    compute_pass.write_timestamp(query_set, query_index);
                }
            }
            query_index += 1;
        }
        drop(compute_pass);

        if let Some(timestamp_query_resources) = timestamp_query_resources.as_ref() {
            let TimestampQueryResources {
                query_set,
                count,
                resolve_buffer,
            } = timestamp_query_resources;
            command_encoder.resolve_query_set(query_set, 0..count.get(), resolve_buffer, 0);
        }
    }

    // TODO: methods to copy data from CPU to GPU and vice versa
    //       do not grant mutable access to the context
}

const SIZE_OF_U64_AS_U64: u64 = size_of::<u64>() as u64;

#[inline]
fn timestamp_count_for_system_cache(system_cache: &SystemCache) -> usize {
    match system_cache.len() {
        0 => 0,
        count => count + 1,
    }
}

#[inline]
fn create_timestamp_query_resources(
    device: &Device,
    schedule_cache: &ScheduleCache,
) -> Option<TimestampQueryResources> {
    let count = schedule_cache
        .values()
        .map(timestamp_count_for_system_cache)
        .sum::<usize>()
        .try_into()
        .expect("total timestamp count of schedule cache should fit into `u32`");
    let count = NonZeroU32::new(count)?;

    let query_set_desc = QuerySetDescriptor {
        label: Some("`gpecs` executor query set"),
        ty: QueryType::Timestamp,
        count: count.get(),
    };
    let query_set = device.create_query_set(&query_set_desc);

    let resolve_buffer_desc = BufferDescriptor {
        label: Some("`gpecs` executor query set resolve buffer"),
        size: u64::from(count.get()) * SIZE_OF_U64_AS_U64,
        usage: BufferUsages::QUERY_RESOLVE | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    };
    let resolve_buffer = device.create_buffer(&resolve_buffer_desc);

    Some(TimestampQueryResources {
        query_set,
        count,
        resolve_buffer,
    })
}

#[inline]
fn cache_schedule_pure(
    context: &Context,
    device: &Device,
    archetypes: &GpuArchetypeRegistry,
    systems: &GpuSystemRegistry,
    schedule: &GpuSystemSchedule,
) -> ScheduleCache {
    cache_schedule::<_, [_; 0]>(context, device, archetypes, systems, schedule, [])
}

fn cache_schedule<'a, I, B>(
    context: &Context,
    device: &Device,
    archetypes: &GpuArchetypeRegistry,
    systems: &GpuSystemRegistry,
    schedule: &GpuSystemSchedule,
    additional_bindings: I,
) -> ScheduleCache
where
    I: IntoIterator<Item = (GpuSystemId, B)>,
    B: IntoIterator<Item = BindGroupEntry<'a>>,
{
    let mut additional_bindings_cache = IndexMap::<GpuSystemId, Vec<BindGroupEntry>>::new();
    for (system_id, additional_bindings) in additional_bindings {
        let cached_entries = additional_bindings_cache.entry(system_id).or_default();
        cached_entries.extend(additional_bindings);
    }

    let mut schedule_cache = ScheduleCache::default();
    for system_id in schedule {
        let Some(system_info) = systems.get_system_info(system_id) else {
            unreachable!("system {system_id:?} should exist");
        };

        let shader = system_info.shader();
        let component_ids = shader
            .component_entries()
            .map(|(component_id, _)| component_id.into());
        let Ok(compatible_archetypes) = context.archetypes().compatible_archetypes(component_ids)
        else {
            unreachable!("system {system_id:?} should have compatible archetypes");
        };
        for archetype_info in compatible_archetypes {
            let Some(archetype_id) = archetypes.map_archetype_id(archetype_info.id()) else {
                continue;
            };
            let Some(archetype_info) = archetypes.get_archetype_info(archetype_id) else {
                unreachable!("archetype {archetype_id:?} should exist");
            };

            let GpuArchetypeStorageBufferSlices {
                entities: entity_buffer_binding,
                components: mut components_component_buffer_bindings,
            } = unsafe { archetype_info.storage().storage_buffer_slices() };
            let mut bind_group_entries = Vec::new();

            if let Some(entity_entry) = shader.entity_entry() {
                let Some(entity_buffer_binding) = entity_buffer_binding else {
                    continue;
                };

                let entity_entry = BindGroupEntry {
                    binding: entity_entry.binding,
                    resource: entity_buffer_binding.into(),
                };
                bind_group_entries.extend(Some(entity_entry));
            }

            for (component_id, component_entry) in shader.component_entries() {
                let Some(component_entry) = component_entry else {
                    continue;
                };
                let Some(component_buffer_binding) =
                    components_component_buffer_bindings.swap_remove(&component_id)
                else {
                    unreachable!("archetype {archetype_id:?} should have {component_id:?}");
                };
                let Some(component_buffer_binding) = component_buffer_binding else {
                    continue;
                };

                let component_entry = BindGroupEntry {
                    binding: component_entry.binding,
                    resource: component_buffer_binding.into(),
                };
                bind_group_entries.extend(Some(component_entry));
            }

            if bind_group_entries.is_empty() {
                continue;
            }

            let additional_bindings = additional_bindings_cache
                .get(&system_id)
                .into_iter()
                .flatten()
                .cloned();
            bind_group_entries.extend(additional_bindings);

            let bind_group_label = format!("`gpecs` {system_id:?} bind group for {archetype_id:?}");
            let bind_group_desc = BindGroupDescriptor {
                label: Some(&bind_group_label),
                layout: shader.bind_group_layout(),
                entries: bind_group_entries.as_ref(),
            };
            let bind_group = device.create_bind_group(&bind_group_desc);

            let system_archetypes = schedule_cache.entry(system_id).or_default();
            if system_archetypes.insert(archetype_id, bind_group).is_some() {
                unreachable!(
                    "archetype {archetype_id:?} cannot have multiple bind groups for system {system_id:?}"
                );
            }
        }
    }
    schedule_cache
}
