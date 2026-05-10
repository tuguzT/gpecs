use std::iter::chain;

use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, Device};

use crate::{
    context::Context,
    executor::gpu::{
        archetype::{
            registry::{GpuArchetypeId, GpuArchetypeRegistry},
            storage::{GpuArchetypeStorage, GpuArchetypeStorageSlice},
        },
        component::registry::GpuComponentId,
        system::{
            registry::{GpuSystemId, GpuSystemRegistry},
            schedule::GpuSystemSchedule,
            shader::{GpuSystemShader, GpuSystemShaderEntry},
        },
    },
    hash::IndexMap,
};

#[derive(Debug, Default)]
pub struct ScheduleCache<'a> {
    systems: IndexMap<GpuSystemId, SystemCache<'a>>,
}

impl<'a> ScheduleCache<'a> {
    #[inline]
    pub fn request_system_resync(&mut self, system_id: GpuSystemId) {
        let Self { systems } = self;

        if let Some(system_cache) = systems.get_mut(&system_id) {
            system_cache.request_resync();
        }
    }

    #[inline]
    pub fn request_archetype_resync(&mut self, archetype_id: GpuArchetypeId) {
        let Self { systems } = self;

        for system_cache in systems.values_mut() {
            system_cache.request_archetype_resync(archetype_id);
        }
    }

    pub fn resync(
        &mut self,
        context: &Context,
        device: &Device,
        archetypes: &GpuArchetypeRegistry,
        gpu_systems: &GpuSystemRegistry,
        schedule: &GpuSystemSchedule,
    ) -> bool {
        let Self { systems } = self;

        let mut update_count = 0_usize;
        for system_id in schedule {
            let system_cache = systems.entry(system_id).or_insert_with(|| {
                update_count += 1;
                SystemCache::new(context, device, archetypes, gpu_systems, system_id, &[])
            });

            let updated = system_cache.resync(device, archetypes, gpu_systems, system_id);
            update_count += usize::from(updated);
        }

        update_count > 0
    }

    pub fn set_additional_entries(
        &mut self,
        context: &Context,
        device: &Device,
        archetypes: &GpuArchetypeRegistry,
        systems: &GpuSystemRegistry,
        system_id: GpuSystemId,
        additional_entries: &'a [BindGroupEntry<'_>],
    ) {
        let system_cache = SystemCache::new(
            context,
            device,
            archetypes,
            systems,
            system_id,
            additional_entries,
        );

        let Self { systems } = self;
        systems.insert(system_id, system_cache);
    }

    #[inline]
    pub fn system(&self, system_id: GpuSystemId) -> Option<&SystemCache<'a>> {
        let Self { systems } = self;
        systems.get(&system_id)
    }

    #[inline]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (GpuSystemId, &SystemCache<'a>)> {
        let Self { systems } = self;
        systems.iter().map(|(&id, cache)| (id, cache))
    }
}

#[derive(Debug, Default)]
pub struct SystemCache<'a> {
    additional_entries: &'a [BindGroupEntry<'a>],
    archetypes: IndexMap<GpuArchetypeId, ArchetypeCache>,
}

impl<'a> SystemCache<'a> {
    fn new(
        context: &Context,
        device: &Device,
        archetypes: &GpuArchetypeRegistry,
        systems: &GpuSystemRegistry,
        system_id: GpuSystemId,
        additional_entries: &'a [BindGroupEntry<'_>],
    ) -> Self {
        let Some(system_shader) = systems.get_system_shader(system_id) else {
            unreachable!("{system_id} should exist");
        };

        let components = &context.components().as_view();
        let component_ids = system_shader
            .bind_group_layout_entries()
            .components
            .map(|(component_id, _)| component_id.into());
        let Ok(compatible_archetypes) = context
            .archetypes()
            .compatible_archetypes_from(components, component_ids)
        else {
            unreachable!("{system_id} should have compatible archetypes");
        };

        let into_archetype_cache = |(archetype_id, _)| {
            let archetype_id = archetypes.map_archetype_id(archetype_id)?;
            let Some(archetype_storage) = archetypes.get_archetype_storage(archetype_id) else {
                unreachable!("{archetype_id} should exist");
            };

            let archetype_cache = ArchetypeCache::new(
                device,
                system_id,
                system_shader,
                archetype_id,
                archetype_storage,
                additional_entries,
            )?;
            Some((archetype_id, archetype_cache))
        };

        let archetypes = compatible_archetypes
            .filter_map(into_archetype_cache)
            .collect();
        Self {
            additional_entries,
            archetypes,
        }
    }

    #[inline]
    fn request_archetype_resync(&mut self, archetype_id: GpuArchetypeId) {
        let Self { archetypes, .. } = self;

        if let Some(archetype_cache) = archetypes.get_mut(&archetype_id) {
            archetype_cache.request_resync();
        }
    }

    #[inline]
    fn request_resync(&mut self) {
        let Self { archetypes, .. } = self;

        for archetype_cache in archetypes.values_mut() {
            archetype_cache.request_resync();
        }
    }

    fn resync(
        &mut self,
        device: &Device,
        gpu_archetypes: &GpuArchetypeRegistry,
        systems: &GpuSystemRegistry,
        system_id: GpuSystemId,
    ) -> bool {
        let Self {
            ref mut archetypes,
            additional_entries,
        } = *self;

        let mut update_count = 0_usize;
        archetypes.retain(|&archetype_id, archetype_cache| {
            let Some(system_shader) = systems.get_system_shader(system_id) else {
                unreachable!("{system_id} should exist");
            };
            let Some(archetype_storage) = gpu_archetypes.get_archetype_storage(archetype_id) else {
                unreachable!("{archetype_id} should exist");
            };

            let resync_result = archetype_cache.resync(
                device,
                system_id,
                system_shader,
                archetype_id,
                archetype_storage,
                additional_entries,
            );
            let Ok(updated) = resync_result else {
                return false;
            };

            update_count += usize::from(updated);
            true
        });

        update_count > 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { archetypes, .. } = self;
        archetypes.len()
    }

    #[inline]
    pub fn archetype(&self, archetype_id: GpuArchetypeId) -> Option<&ArchetypeCache> {
        let Self { archetypes, .. } = self;
        archetypes.get(&archetype_id)
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (GpuArchetypeId, &ArchetypeCache)> {
        let Self { archetypes, .. } = self;
        archetypes.iter().map(|(&id, cache)| (id, cache))
    }
}

#[derive(Debug)]
pub struct ArchetypeCache {
    should_resync: bool,
    bind_group: BindGroup,
}

impl ArchetypeCache {
    fn new(
        device: &Device,
        system_id: GpuSystemId,
        system_shader: &GpuSystemShader,
        archetype_id: GpuArchetypeId,
        archetype_storage: &GpuArchetypeStorage,
        additional_entries: &[BindGroupEntry<'_>],
    ) -> Option<Self> {
        if archetype_storage.is_empty() {
            return None;
        }

        let slices = archetype_storage.slices();
        let shader_entries = system_shader.bind_group_layout_entries();

        let entity_binding = bind_group_entry(shader_entries.entities, slices.entities);
        let component_bindings =
            component_entries_slices(shader_entries.components, slices.components)
                .into_iter()
                .filter_map(|(entry, slice)| bind_group_entry(entry, slice));
        let additional_entries = additional_entries.iter().cloned();

        let bind_group_label = match system_shader.label() {
            Some(label) => format!("`gpecs` {system_id:#} [{label}] {archetype_id:#} bind group"),
            None => format!("`gpecs` {system_id:#} {archetype_id:#} bind group"),
        };
        let bind_group_entries = chain(entity_binding, component_bindings)
            .chain(additional_entries)
            .collect::<Box<_>>();
        let bind_group_desc = BindGroupDescriptor {
            label: Some(&bind_group_label),
            layout: system_shader.bind_group_layout(),
            entries: bind_group_entries.as_ref(),
        };
        let bind_group = device.create_bind_group(&bind_group_desc);

        let me = Self {
            bind_group,
            should_resync: false,
        };
        Some(me)
    }

    #[inline]
    fn request_resync(&mut self) {
        let Self { should_resync, .. } = self;
        *should_resync = true;
    }

    fn resync(
        &mut self,
        device: &Device,
        system_id: GpuSystemId,
        system_shader: &GpuSystemShader,
        archetype_id: GpuArchetypeId,
        archetype_storage: &GpuArchetypeStorage,
        additional_entries: &[BindGroupEntry<'_>],
    ) -> Result<bool, ArchetypeCacheResyncError> {
        let Self { should_resync, .. } = *self;

        if should_resync {
            let new = Self::new(
                device,
                system_id,
                system_shader,
                archetype_id,
                archetype_storage,
                additional_entries,
            );
            let Some(new) = new else {
                return Err(ArchetypeCacheResyncError);
            };
            *self = new;
            return Ok(true);
        }
        Ok(false)
    }

    #[inline]
    pub fn bind_group(&self) -> &BindGroup {
        let Self { bind_group, .. } = self;
        bind_group
    }
}

struct ArchetypeCacheResyncError;

#[inline]
fn bind_group_entry<'a>(
    entry: Option<&GpuSystemShaderEntry>,
    slice: GpuArchetypeStorageSlice<'a>,
) -> Option<BindGroupEntry<'a>> {
    let binding = entry?.binding_index;
    let resource = unsafe { slice.as_slice() }?.into();
    Some(BindGroupEntry { binding, resource })
}

type ComponentEntriesSlicesOutputItem<'a> = (
    Option<&'a GpuSystemShaderEntry>,
    GpuArchetypeStorageSlice<'a>,
);

#[inline]
fn component_entries_slices<'a, E, S>(
    entries: E,
    slices: S,
) -> impl IntoIterator<Item = ComponentEntriesSlicesOutputItem<'a>>
where
    E: IntoIterator<Item = (GpuComponentId, Option<&'a GpuSystemShaderEntry>)>,
    S: IntoIterator<Item = (GpuComponentId, GpuArchetypeStorageSlice<'a>)>,
{
    let mut slices: IndexMap<_, _> = slices.into_iter().collect();
    entries.into_iter().map(move |(component_id, entry)| {
        let Some(slice) = slices.swap_remove(&component_id) else {
            unreachable!("{component_id} should exist");
        };
        (entry, slice)
    })
}
