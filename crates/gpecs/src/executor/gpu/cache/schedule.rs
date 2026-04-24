use std::iter::chain;

use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, Device};

use crate::{
    context::Context,
    executor::gpu::{
        archetype::{
            registry::{GpuArchetypeId, GpuArchetypeInfo, GpuArchetypeRegistry},
            storage::{GpuArchetypeStorage, GpuArchetypeStorageSlice},
        },
        component::registry::GpuComponentId,
        system::{
            registry::{GpuSystemId, GpuSystemInfo, GpuSystemRegistry},
            schedule::GpuSystemSchedule,
            shader::{GpuSystemShader, GpuSystemShaderEntry},
        },
    },
    hash::IndexMap,
};

#[derive(Debug)]
pub struct ScheduleCache {
    systems: IndexMap<GpuSystemId, SystemCache>,
}

impl ScheduleCache {
    #[inline]
    pub fn new(
        context: &Context,
        device: &Device,
        archetypes: &GpuArchetypeRegistry,
        systems: &GpuSystemRegistry,
        schedule: &GpuSystemSchedule,
    ) -> Self {
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

    pub fn with_additional_bindings<'a, I, B>(
        context: &Context,
        device: &Device,
        archetypes: &GpuArchetypeRegistry,
        systems: &GpuSystemRegistry,
        schedule: &GpuSystemSchedule,
        additional_bindings: I,
    ) -> Self
    where
        I: IntoIterator<Item = (GpuSystemId, B)>,
        B: IntoIterator<Item = BindGroupEntry<'a>>,
    {
        let mut additional_bindings_cache = IndexMap::<GpuSystemId, Vec<BindGroupEntry>>::default();
        for (system_id, additional_bindings) in additional_bindings {
            let cached_entries = additional_bindings_cache.entry(system_id).or_default();
            cached_entries.extend(additional_bindings);
        }

        let mut system_caches = IndexMap::default();
        for system_id in schedule {
            let Some(system_info) = systems.get_system_info(system_id) else {
                unreachable!("{system_id} should exist");
            };

            let shader = system_info.into_meta();
            let components = &context.components().as_view();
            let component_ids = shader
                .bind_group_layout_entries()
                .components
                .map(|(component_id, _)| component_id.into());
            let Ok(compatible_archetypes) = context
                .archetypes()
                .compatible_archetypes_from(components, component_ids)
            else {
                unreachable!("{system_id} should have compatible archetypes");
            };
            for archetype_info in compatible_archetypes {
                let archetype_id = archetype_info.archetype_id();
                let Some(archetype_id) = archetypes.map_archetype_id(archetype_id) else {
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

                let SystemCache { archetypes } = system_caches.entry(system_id).or_default();
                if archetypes.insert(archetype_id, archetype_cache).is_some() {
                    unreachable!("{archetype_id} cannot have multiple bind groups for {system_id}");
                }
            }
        }

        let systems = system_caches;
        Self { systems }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { systems } = self;
        systems.len()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = GpuSystemInfo<&SystemCache>> {
        let Self { systems } = self;
        systems
            .iter()
            .map(|(&id, cache)| GpuSystemInfo::new(id, cache))
    }
}

#[derive(Debug, Default)]
pub struct SystemCache {
    archetypes: IndexMap<GpuArchetypeId, ArchetypeCache>,
}

impl SystemCache {
    #[inline]
    pub fn len(&self) -> usize {
        let Self { archetypes } = self;
        archetypes.len()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = GpuArchetypeInfo<&ArchetypeCache>> {
        let Self { archetypes } = self;
        archetypes
            .iter()
            .map(|(&id, cache)| GpuArchetypeInfo::new(id, cache))
    }
}

#[derive(Debug)]
pub struct ArchetypeCache {
    bind_group: BindGroup,
}

impl ArchetypeCache {
    #[inline]
    pub fn new<'a, I>(
        device: &Device,
        system_info: GpuSystemInfo<&GpuSystemShader>,
        archetype_info: GpuArchetypeInfo<&GpuArchetypeStorage>,
        additional_bindings: I,
    ) -> Option<Self>
    where
        I: IntoIterator<Item = BindGroupEntry<'a>>,
    {
        let archetype_id = archetype_info.archetype_id();
        let archetype_storage = archetype_info.into_meta();
        if archetype_storage.is_empty() {
            return None;
        }

        let shader = system_info.into_meta();
        let system_id = system_info.system_id();

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

        let me = Self { bind_group };
        Some(me)
    }

    #[inline]
    pub fn bind_group(&self) -> &BindGroup {
        let Self { bind_group, .. } = self;
        bind_group
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
