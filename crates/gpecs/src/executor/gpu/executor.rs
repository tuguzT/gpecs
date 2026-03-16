use std::{any::TypeId, num::NonZeroU32};

use itertools::chain;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, Buffer, BufferAddress,
    BufferDescriptor, BufferUsages, CommandEncoder, ComputePass, ComputePassDescriptor, Device,
    Features, QUERY_SIZE, QuerySet, QuerySetDescriptor, QueryType,
};

use crate::{
    archetype::error::{ArchetypeError, DuplicateComponentError},
    component::registry::ComponentInfo,
    context::Context,
    hash::IndexMap,
};

use super::{
    archetype::{
        registry::{GpuArchetypeId, GpuArchetypeInfo, GpuArchetypeRegistry},
        storage::{GpuArchetypeStorage, GpuArchetypeStorageSlice},
    },
    bundle::GpuBundle,
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
        shader::GpuSystemShaderEntry,
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
    schedule_cache: Option<ScheduleCache>,
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
    pub fn into_context(self) -> &'ctx mut Context {
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
    pub fn get_archetype_info(&self, archetype_id: GpuArchetypeId) -> Option<&GpuArchetypeInfo> {
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

        let new_cache = ScheduleCache::with_additional_bindings(
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
            ref systems,
            ref schedule,
            ref mut schedule_cache,
            ref mut timestamp_query_resources,
            ..
        } = *self;

        let cache_schedule = || ScheduleCache::new(context, device, archetypes, systems, schedule);
        let schedule_cache = &*schedule_cache.get_or_insert_with(cache_schedule);

        if timestamp_query_resources.is_none() {
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
        for (&system_id, system_cache) in &schedule_cache.systems {
            let Some(system_info) = systems.get_system_info(system_id) else {
                unreachable!("{system_id} should exist");
            };
            let shader = system_info.shader();

            let compute_pass_label = match shader.label() {
                Some(label) => format!("`gpecs` {system_id:#} [{label}] compute pass"),
                None => format!("`gpecs` {system_id:#} compute pass"),
            };
            let compute_pass_desc = ComputePassDescriptor {
                label: Some(&compute_pass_label),
                timestamp_writes: None,
            };
            let mut compute_pass = command_encoder.begin_compute_pass(&compute_pass_desc);

            for (&archetype_id, archetype_cache) in &system_cache.archetypes {
                let Some(archetype_info) = archetypes.get_archetype_info(archetype_id) else {
                    unreachable!("{archetype_id} should exist");
                };

                compute_pass.set_pipeline(shader.compute_pipeline());
                compute_pass.set_bind_group(0, &archetype_cache.bind_group, &[]);

                write_timestamp(&mut compute_pass, query_index);
                query_index += 1;

                let archetype_storage = archetype_info.storage();
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

    // TODO: methods to copy data from CPU to GPU and vice versa
    //       do not grant mutable access to the context
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

#[derive(Debug)]
pub struct TimestampQueryResources {
    query_set: QuerySet,
    count: NonZeroU32,
    resolve_buffer: Buffer,
}

impl TimestampQueryResources {
    #[inline]
    fn new(gpu_device: &Device, schedule_cache: &ScheduleCache) -> Option<Self> {
        let can_write_timestamps = gpu_device
            .features()
            .contains(Features::TIMESTAMP_QUERY_INSIDE_PASSES);
        if !can_write_timestamps {
            return None;
        }

        let ScheduleCache { systems } = schedule_cache;
        let count = systems
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
        let query_set = gpu_device.create_query_set(&query_set_desc);

        let resolve_buffer_desc = BufferDescriptor {
            label: Some("`gpecs` executor query set resolve buffer"),
            size: resolve_buffer_size(count),
            usage: BufferUsages::QUERY_RESOLVE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        };
        let resolve_buffer = gpu_device.create_buffer(&resolve_buffer_desc);

        Some(TimestampQueryResources {
            query_set,
            count,
            resolve_buffer,
        })
    }

    #[inline]
    pub unsafe fn query_set(&self) -> &QuerySet {
        let Self { query_set, .. } = self;
        query_set
    }

    #[inline]
    pub fn count(&self) -> NonZeroU32 {
        let Self { count, .. } = *self;
        count
    }

    #[inline]
    pub unsafe fn resolve_buffer(&self) -> &Buffer {
        let Self { resolve_buffer, .. } = self;
        resolve_buffer
    }

    #[inline]
    fn resolve(&self, command_encoder: &mut CommandEncoder) {
        let Self {
            query_set,
            count,
            resolve_buffer,
        } = self;
        command_encoder.resolve_query_set(query_set, 0..count.get(), resolve_buffer, 0);
    }
}

#[inline]
fn timestamp_count_for_system_cache(system_cache: &SystemCache) -> usize {
    let count = system_cache.archetypes.len();
    if count == 0 {
        return 0;
    }
    count + 1
}

#[inline]
fn resolve_buffer_size(query_set_count: NonZeroU32) -> BufferAddress {
    // cast operands first to avoid potential `u32` overflow
    let query_set_count = BufferAddress::from(query_set_count.get());
    let query_size = BufferAddress::from(QUERY_SIZE);

    let Some(size) = query_set_count.checked_mul(query_size) else {
        unreachable!("{query_set_count} * `wgpu::QUERY_SIZE` (which is {query_size}) overflow")
    };
    size
}

#[derive(Debug, Default)]
struct ScheduleCache {
    systems: IndexMap<GpuSystemId, SystemCache>,
}

impl ScheduleCache {
    #[inline]
    fn new(
        context: &Context,
        device: &Device,
        archetypes: &GpuArchetypeRegistry,
        systems: &GpuSystemRegistry,
        schedule: &GpuSystemSchedule,
    ) -> ScheduleCache {
        let additional_bindings = [];
        Self::with_additional_bindings::<_, [_; 0]>(
            context,
            device,
            archetypes,
            systems,
            schedule,
            additional_bindings,
        )
    }

    fn with_additional_bindings<'a, I, B>(
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
        let mut additional_bindings_cache = IndexMap::<GpuSystemId, Vec<BindGroupEntry>>::default();
        for (system_id, additional_bindings) in additional_bindings {
            let cached_entries = additional_bindings_cache.entry(system_id).or_default();
            cached_entries.extend(additional_bindings);
        }

        let mut schedule_cache = ScheduleCache::default();
        for system_id in schedule {
            let Some(system_info) = systems.get_system_info(system_id) else {
                unreachable!("{system_id} should exist");
            };

            let shader = system_info.shader();
            let component_ids = shader
                .bind_group_layout_entries()
                .components
                .map(|(component_id, _)| component_id.into());
            let Ok(compatible_archetypes) = context
                .archetypes()
                .compatible_archetypes_from(context.components(), component_ids)
            else {
                unreachable!("{system_id} should have compatible archetypes");
            };
            for archetype_info in compatible_archetypes {
                let Some(archetype_id) = archetypes.map_archetype_id(archetype_info.id()) else {
                    continue;
                };
                let Some(archetype_info) = archetypes.get_archetype_info(archetype_id) else {
                    unreachable!("{archetype_id} should exist");
                };

                let additional_bindings = additional_bindings_cache
                    .get(&system_id)
                    .into_iter()
                    .flatten()
                    .cloned();
                let Some(archetype_cache) =
                    ArchetypeCache::new(device, system_info, archetype_info, additional_bindings)
                else {
                    continue;
                };

                let SystemCache { archetypes } =
                    schedule_cache.systems.entry(system_id).or_default();
                if archetypes.insert(archetype_id, archetype_cache).is_some() {
                    unreachable!("{archetype_id} cannot have multiple bind groups for {system_id}");
                }
            }
        }
        schedule_cache
    }
}

#[derive(Debug, Default)]
struct SystemCache {
    archetypes: IndexMap<GpuArchetypeId, ArchetypeCache>,
}

#[derive(Debug)]
struct ArchetypeCache {
    bind_group: BindGroup,
}

impl ArchetypeCache {
    #[inline]
    fn new<'a, I>(
        device: &Device,
        system_info: &GpuSystemInfo,
        archetype_info: &GpuArchetypeInfo,
        additional_bindings: I,
    ) -> Option<Self>
    where
        I: IntoIterator<Item = BindGroupEntry<'a>>,
    {
        let archetype_id = archetype_info.id();
        let archetype_storage = archetype_info.storage();
        if archetype_storage.is_empty() {
            return None;
        }

        let shader = system_info.shader();
        let system_id = system_info.id();

        let slices = archetype_storage.slices();
        let shader_entries = shader.bind_group_layout_entries();

        let entity_binding = bind_group_entry(shader_entries.entities, slices.entities);
        let component_bindings =
            component_entries_slices(shader_entries.components, slices.components)
                .into_iter()
                .filter_map(|(_, entry, slice)| bind_group_entry(entry, slice));

        let additional_bindings = additional_bindings.into_iter().map(upcast_bind_group_entry);

        let bind_group_label = match shader.label() {
            Some(label) => format!("`gpecs` {system_id:#} [{label}] {archetype_id:#} bind group"),
            None => format!("`gpecs` {system_id:#} {archetype_id:#} bind group"),
        };
        let bind_group_entries = chain(entity_binding, component_bindings)
            .chain(additional_bindings)
            .collect::<Box<_>>();
        let bind_group_desc = BindGroupDescriptor {
            label: Some(&bind_group_label),
            layout: shader.bind_group_layout(),
            entries: bind_group_entries.as_ref(),
        };
        let bind_group = device.create_bind_group(&bind_group_desc);

        Some(Self { bind_group })
    }
}

#[inline]
fn upcast_bind_group_entry<'short, 'long: 'short>(
    entry: BindGroupEntry<'long>,
) -> BindGroupEntry<'short> {
    entry
}

#[inline]
fn bind_group_entry<'a>(
    entry: Option<&GpuSystemShaderEntry>,
    slice: Option<GpuArchetypeStorageSlice<'a>>,
) -> Option<BindGroupEntry<'a>> {
    let binding = entry?.binding_index;
    let resource = unsafe { slice?.as_slice() }.into();
    Some(BindGroupEntry { binding, resource })
}

type ComponentEntriesSlicesOutputItem<'a> = (
    GpuComponentId,
    Option<&'a GpuSystemShaderEntry>,
    Option<GpuArchetypeStorageSlice<'a>>,
);

#[inline]
fn component_entries_slices<'a, E, S>(
    entries: E,
    slices: S,
) -> impl IntoIterator<Item = ComponentEntriesSlicesOutputItem<'a>>
where
    E: IntoIterator<Item = (GpuComponentId, Option<&'a GpuSystemShaderEntry>)>,
    S: IntoIterator<Item = (GpuComponentId, Option<GpuArchetypeStorageSlice<'a>>)>,
{
    let mut slices: IndexMap<_, _> = slices.into_iter().collect();
    entries.into_iter().map(move |(component_id, entry)| {
        let Some(slice) = slices.swap_remove(&component_id) else {
            unreachable!("{component_id} should exist");
        };
        (component_id, entry, slice)
    })
}
