use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use bytemuck::must_cast_slice_mut;
use indexmap::map::Entry;
use wgpu::{
    Buffer, BufferDescriptor, BufferSize, BufferSlice, BufferUsages, BufferView, CommandEncoder,
    Device, MapMode,
};

use crate::{
    archetype::{registry::ArchetypeRegistry, storage::ArchetypeStorage},
    executor::gpu::{
        archetype::registry::{GpuArchetypeId, GpuArchetypeRegistry},
        cache::schedule::ScheduleCache,
        component::registry::GpuComponentId,
        context::MappedArchetypeNotReadyError,
    },
    hash::IndexMap,
};

#[derive(Debug, Default)]
pub struct TransferCache {
    archetypes: IndexMap<GpuArchetypeId, ArchetypeCache>,
}

impl TransferCache {
    pub fn download_all_from(
        &mut self,
        device: &Device,
        command_encoder: &mut CommandEncoder,
        schedule_cache: &ScheduleCache,
        gpu_archetypes: &GpuArchetypeRegistry,
    ) {
        for archetype_id in schedule_cache
            .iter()
            .flat_map(|system_cache| system_cache.iter())
            .map(GpuArchetypeId::from)
        {
            self.download_archetype_from_trusted(
                device,
                command_encoder,
                archetype_id,
                gpu_archetypes,
            );
        }
    }

    pub fn download_archetype_from(
        &mut self,
        device: &Device,
        command_encoder: &mut CommandEncoder,
        archetype_id: GpuArchetypeId,
        schedule_cache: &ScheduleCache,
        gpu_archetypes: &GpuArchetypeRegistry,
    ) {
        let should_download = schedule_cache
            .iter()
            .any(|system_cache| system_cache.archetype(archetype_id).is_some());
        if !should_download {
            return;
        }

        self.download_archetype_from_trusted(device, command_encoder, archetype_id, gpu_archetypes);
    }

    fn download_archetype_from_trusted(
        &mut self,
        device: &Device,
        command_encoder: &mut CommandEncoder,
        archetype_id: GpuArchetypeId,
        gpu_archetypes: &GpuArchetypeRegistry,
    ) {
        let Self { archetypes } = self;

        let archetype_cache_entry = archetypes.entry(archetype_id);
        if let Entry::Occupied(ref entry) = archetype_cache_entry
            && entry.get().is_downloaded
        {
            return;
        }

        let Some(storage) = gpu_archetypes.get_archetype_info(archetype_id) else {
            unreachable!("{archetype_id} should exist")
        };
        let storage_slices = storage.slices();

        let Some(entities) = storage_slices.entities else {
            return;
        };

        let source = unsafe { entities.as_slice() };
        let label = || format!("`gpecs` {archetype_id:#} entities download buffer");
        let archetype_cache = archetype_cache_entry
            .and_modify(|cache| {
                cache
                    .entities
                    .copy_from_slice(device, command_encoder, source, label);
            })
            .or_insert_with(|| {
                let buffer = DownloadBuffer::from_slice(device, command_encoder, source, label());
                ArchetypeCache::new(buffer)
            });

        for (component_id, components) in storage_slices.components {
            let Some(components) = components else {
                continue;
            };

            let source = unsafe { components.as_slice() };
            let label = || format!("`gpecs` {archetype_id:#} {component_id:#} download buffer");
            archetype_cache
                .components
                .entry(component_id)
                .and_modify(|components| {
                    components.copy_from_slice(device, command_encoder, source, label);
                })
                .or_insert_with(|| {
                    DownloadBuffer::from_slice(device, command_encoder, source, label())
                });
        }

        archetype_cache.is_downloaded = true;
    }

    pub fn move_all_into<'a>(
        &mut self,
        cpu_archetypes: &'a mut ArchetypeRegistry,
    ) -> Result<&'a ArchetypeRegistry, MappedArchetypeNotReadyError> {
        let Self { archetypes } = self;

        for (&archetype_id, archetype_cache) in archetypes {
            let storage = unsafe { cpu_archetypes.get_archetype_info_mut(archetype_id.into()) };
            let Some(storage) = storage else {
                unreachable!("{archetype_id} should exist")
            };
            let storage = storage.into_meta();

            Self::move_archetype_into_trusted(archetype_cache, storage)
                .ok_or_else(|| MappedArchetypeNotReadyError::new(archetype_id))?;
        }

        Ok(cpu_archetypes)
    }

    pub fn move_archetype_into<'a>(
        &mut self,
        archetype_id: GpuArchetypeId,
        cpu_archetypes: &'a mut ArchetypeRegistry,
    ) -> Result<&'a ArchetypeStorage, MappedArchetypeNotReadyError> {
        let storage = unsafe { cpu_archetypes.get_archetype_info_mut(archetype_id.into()) };
        let Some(storage) = storage else {
            unreachable!("{archetype_id} should exist")
        };
        let storage = storage.into_meta();

        let Self { archetypes } = self;
        let Some(archetype_cache) = archetypes.get_mut(&archetype_id) else {
            return Ok(storage);
        };

        Self::move_archetype_into_trusted(archetype_cache, storage)
            .ok_or_else(|| MappedArchetypeNotReadyError::new(archetype_id))
    }

    fn move_archetype_into_trusted<'a>(
        archetype_cache: &mut ArchetypeCache,
        storage: &'a mut ArchetypeStorage,
    ) -> Option<&'a ArchetypeStorage> {
        if !archetype_cache.is_downloaded {
            return None;
        }
        if archetype_cache.is_moved {
            return Some(storage);
        }

        let (entities, mut bundles, _) = unsafe { storage.as_mut_view().into_mut_slices() };

        let mapped_entities = archetype_cache.entities.as_slice()?;
        must_cast_slice_mut(entities).copy_from_slice(&mapped_entities);

        for (&component_id, components) in &archetype_cache.components {
            let mapped_components = components.as_slice()?;
            let Some(mut components) = bundles.get_mut(component_id.into()) else {
                continue;
            };

            let components = unsafe { components.as_mut_buffer() };
            components.write_copy_of_slice(&mapped_components);
        }

        archetype_cache.is_moved = true;
        Some(storage)
    }

    pub fn invalidate(&mut self) {
        let Self { archetypes } = self;

        for archetype_cache in archetypes.values_mut() {
            archetype_cache.is_downloaded = false;
            archetype_cache.is_moved = false;
        }
    }
}

#[derive(Debug)]
pub struct ArchetypeCache {
    is_downloaded: bool,
    is_moved: bool,
    entities: DownloadBuffer,
    components: IndexMap<GpuComponentId, DownloadBuffer>,
}

impl ArchetypeCache {
    pub fn new(entities: DownloadBuffer) -> Self {
        Self {
            is_downloaded: false,
            is_moved: false,
            entities,
            components: IndexMap::default(),
        }
    }
}

#[derive(Debug)]
pub struct DownloadBuffer {
    buffer: Buffer,
    init_size: BufferSize,
    is_mapped: Arc<AtomicBool>,
}

impl DownloadBuffer {
    #[inline]
    pub fn from_slice(
        device: &Device,
        command_encoder: &mut CommandEncoder,
        source: BufferSlice<'_>,
        label: impl AsRef<str>,
    ) -> Self {
        let init_size = source.size();

        let size = init_size.get();
        let desc = BufferDescriptor {
            label: Some(label.as_ref()),
            size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let buffer = device.create_buffer(&desc);

        command_encoder.copy_buffer_to_buffer(source.buffer(), source.offset(), &buffer, 0, size);

        let is_mapped = AtomicBool::new(false).into();
        {
            let is_mapped = Arc::clone(&is_mapped);
            let callback = move |_| is_mapped.store(true, Ordering::Release);
            command_encoder.map_buffer_on_submit(&buffer, MapMode::Read, ..size, callback);
        }

        Self {
            buffer,
            init_size,
            is_mapped,
        }
    }

    #[inline]
    pub fn copy_from_slice<L>(
        &mut self,
        device: &Device,
        command_encoder: &mut CommandEncoder,
        source: BufferSlice<'_>,
        label: impl FnOnce() -> L,
    ) where
        L: AsRef<str>,
    {
        let Self {
            buffer,
            init_size,
            is_mapped,
        } = self;

        if is_mapped.swap(false, Ordering::AcqRel) {
            buffer.unmap();
        }

        let size = source.size().get();
        if buffer.size() < size {
            *self = Self::from_slice(device, command_encoder, source, label());
            return;
        }

        *init_size = source.size();
        command_encoder.copy_buffer_to_buffer(source.buffer(), source.offset(), buffer, 0, size);

        let is_mapped = Arc::clone(is_mapped);
        let callback = move |_| is_mapped.store(true, Ordering::Release);
        command_encoder.map_buffer_on_submit(buffer, MapMode::Read, ..size, callback);
    }

    #[inline]
    pub fn as_slice(&self) -> Option<BufferView> {
        let Self {
            buffer,
            init_size,
            is_mapped,
        } = self;

        if !is_mapped.load(Ordering::Acquire) {
            return None;
        }

        let view = buffer.get_mapped_range(..init_size.get());
        Some(view)
    }
}
