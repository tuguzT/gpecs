use bytemuck::must_cast_slice_mut;
use wgpu::{
    Buffer, BufferAsyncError, BufferDescriptor, BufferSlice, BufferUsages, CommandEncoder, Device,
    MapMode, WasmNotSend,
};

use crate::{
    archetype::registry::ArchetypeRegistry,
    executor::gpu::{
        archetype::registry::{GpuArchetypeId, GpuArchetypeRegistry},
        cache::schedule::ScheduleCache,
        component::registry::GpuComponentId,
    },
    hash::{IndexMap, IndexSet},
};

#[derive(Debug, Default)]
pub struct TransferCache {
    archetypes: IndexMap<GpuArchetypeId, ArchetypeCache>,
}

impl TransferCache {
    pub fn download_from(
        &mut self,
        device: &Device,
        command_encoder: &mut CommandEncoder,
        schedule_cache: &ScheduleCache,
        gpu_archetypes: &GpuArchetypeRegistry,
    ) {
        let Self { archetypes } = self;

        let archetypes_to_download: IndexSet<_> = schedule_cache
            .iter()
            .flat_map(|system_cache| system_cache.iter())
            .map(GpuArchetypeId::from)
            .collect();

        for archetype_id in archetypes_to_download {
            let Some(storage) = gpu_archetypes.get_archetype_info(archetype_id) else {
                unreachable!("{archetype_id} should exist")
            };
            let storage_slices = storage.slices();

            let Some(entities) = storage_slices.entities else {
                continue;
            };

            let archetype_cache = archetypes.entry(archetype_id).or_default();
            archetype_cache
                .entities
                .copy_from_buffer(device, command_encoder, unsafe { entities.as_slice() });

            for (component_id, components) in storage_slices.components {
                let Some(components) = components else {
                    continue;
                };

                let download_components =
                    archetype_cache.components.entry(component_id).or_default();
                download_components
                    .copy_from_buffer(device, command_encoder, unsafe { components.as_slice() });
            }
        }
    }

    pub fn map_async_all<F>(&self, callback: F)
    where
        F: FnOnce(Result<(), BufferAsyncError>) + WasmNotSend + Clone + 'static,
    {
        let Self { archetypes } = self;

        if archetypes.is_empty() {
            callback(Ok(()));
            return;
        }

        for archetype_cache in archetypes.values() {
            archetype_cache.entities.map_async(callback.clone());
            for component_buffer in archetype_cache.components.values() {
                component_buffer.map_async(callback.clone());
            }
        }
    }

    pub fn move_into(&self, cpu_archetypes: &mut ArchetypeRegistry) {
        let Self { archetypes } = self;

        for (&archetype_id, archetype_cache) in archetypes {
            let storage = unsafe { cpu_archetypes.get_archetype_info_mut(archetype_id.into()) };
            let Some(storage) = storage else {
                unreachable!("{archetype_id} should exist")
            };

            let storage = storage.into_meta();
            let (entities, mut bundles, _) = unsafe { storage.as_mut_view().into_mut_slices() };

            let mapped_entities = archetype_cache.entities.get_buffer();
            let Some(mapped_entities) = mapped_entities else {
                unreachable!("entities of {archetype_id} should be mapped");
            };
            must_cast_slice_mut(entities).copy_from_slice(&mapped_entities.get_mapped_range(..));
            mapped_entities.unmap();

            for (&component_id, components) in &archetype_cache.components {
                let Some(mapped_components) = components.get_buffer() else {
                    unreachable!("{component_id} of {archetype_id} should be mapped");
                };

                let Some(mut components) = bundles.get_mut(component_id.into()) else {
                    continue;
                };
                let components = unsafe { components.as_mut_buffer() };

                components.write_copy_of_slice(&mapped_components.get_mapped_range(..));
                mapped_components.unmap();
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ArchetypeCache {
    entities: DownloadBuffer,
    components: IndexMap<GpuComponentId, DownloadBuffer>,
}

#[derive(Debug, Default)]
pub struct DownloadBuffer {
    buffer: Option<Buffer>,
}

impl DownloadBuffer {
    #[inline]
    pub fn copy_from_buffer(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        source: BufferSlice<'_>,
    ) {
        let Self { buffer } = self;

        let size = source.size().get();
        let new_buffer = || {
            let desc = BufferDescriptor {
                label: Some("`gpecs` cache download buffer"),
                size,
                usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            };
            device.create_buffer(&desc)
        };
        let buffer = buffer.get_or_insert_with(new_buffer);

        if buffer.size() < size {
            *buffer = new_buffer();
        }

        encoder.copy_buffer_to_buffer(source.buffer(), source.offset(), buffer, 0, size);
    }

    #[inline]
    pub fn map_async<F>(&self, callback: F)
    where
        F: FnOnce(Result<(), BufferAsyncError>) + WasmNotSend + 'static,
    {
        let Self { buffer } = self;
        if let Some(buffer) = buffer {
            buffer.map_async(MapMode::Read, .., callback);
        }
    }

    #[inline]
    pub fn get_buffer(&self) -> Option<&Buffer> {
        let Self { buffer } = self;
        buffer.as_ref()
    }
}
