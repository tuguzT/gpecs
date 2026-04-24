use bytemuck::must_cast_slice_mut;
use wgpu::{
    Buffer, BufferAsyncError, BufferDescriptor, BufferSize, BufferSlice, BufferUsages,
    CommandEncoder, Device, MapMode, WasmNotSend,
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

            let source = unsafe { entities.as_slice() };
            let archetype_cache = archetypes
                .entry(archetype_id)
                .and_modify(|c| c.entities.copy_from_slice(device, command_encoder, source))
                .or_insert_with(|| {
                    let entities = DownloadBuffer::from_slice(device, command_encoder, source);
                    ArchetypeCache::new(entities)
                });

            for (component_id, components) in storage_slices.components {
                let Some(components) = components else {
                    continue;
                };

                let source = unsafe { components.as_slice() };
                archetype_cache
                    .components
                    .entry(component_id)
                    .and_modify(|b| b.copy_from_slice(device, command_encoder, source))
                    .or_insert_with(|| DownloadBuffer::from_slice(device, command_encoder, source));
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

            let mapped_entities = archetype_cache.entities.as_slice();
            must_cast_slice_mut(entities).copy_from_slice(&mapped_entities.get_mapped_range());
            mapped_entities.buffer().unmap();

            for (&component_id, components) in &archetype_cache.components {
                let mapped_components = components.as_slice();

                let Some(mut components) = bundles.get_mut(component_id.into()) else {
                    continue;
                };
                let components = unsafe { components.as_mut_buffer() };

                components.write_copy_of_slice(&mapped_components.get_mapped_range());
                mapped_components.buffer().unmap();
            }
        }
    }
}

#[derive(Debug)]
pub struct ArchetypeCache {
    entities: DownloadBuffer,
    components: IndexMap<GpuComponentId, DownloadBuffer>,
}

impl ArchetypeCache {
    pub fn new(entities: DownloadBuffer) -> Self {
        Self {
            entities,
            components: IndexMap::default(),
        }
    }
}

#[derive(Debug)]
pub struct DownloadBuffer {
    buffer: Buffer,
    init_size: BufferSize,
}

impl DownloadBuffer {
    #[inline]
    pub fn from_slice(
        device: &Device,
        command_encoder: &mut CommandEncoder,
        source: BufferSlice<'_>,
    ) -> Self {
        let init_size = source.size();

        let size = init_size.get();
        let desc = BufferDescriptor {
            label: Some("`gpecs` transfer cache download buffer"),
            size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let buffer = device.create_buffer(&desc);

        command_encoder.copy_buffer_to_buffer(source.buffer(), source.offset(), &buffer, 0, size);
        Self { buffer, init_size }
    }

    #[inline]
    pub fn copy_from_slice(
        &mut self,
        device: &Device,
        command_encoder: &mut CommandEncoder,
        source: BufferSlice<'_>,
    ) {
        let Self { buffer, init_size } = self;

        let size = source.size().get();
        if buffer.size() < size {
            *self = Self::from_slice(device, command_encoder, source);
            return;
        }

        *init_size = source.size();
        command_encoder.copy_buffer_to_buffer(source.buffer(), source.offset(), buffer, 0, size);
    }

    #[inline]
    pub fn map_async<F>(&self, callback: F)
    where
        F: FnOnce(Result<(), BufferAsyncError>) + WasmNotSend + 'static,
    {
        self.as_slice().map_async(MapMode::Read, callback);
    }

    #[inline]
    pub fn as_slice(&self) -> BufferSlice<'_> {
        let Self { buffer, init_size } = self;
        buffer.slice(..init_size.get())
    }
}
