use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use bytemuck::{must_cast_slice, must_cast_slice_mut};
use indexmap::map::Entry;
use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferSize, BufferSlice, BufferUsages, BufferView,
    CommandEncoder, Device, MapMode,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    archetype::{registry::ArchetypeRegistry, storage::ArchetypeStorage},
    executor::gpu::{
        archetype::{
            registry::{GpuArchetypeId, GpuArchetypeRegistry},
            storage::GpuArchetypeStorage,
        },
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
        for (archetype_id, _) in schedule_cache
            .iter()
            .flat_map(|(_, system_cache)| system_cache.iter())
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
            .any(|(_, system_cache)| system_cache.archetype(archetype_id).is_some());
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
        if let Entry::Occupied(ref entry) = archetype_cache_entry {
            match entry.get().state {
                ArchetypeCacheState::Invalidated => (),
                ArchetypeCacheState::CopiedFromGpu
                | ArchetypeCacheState::CopiedIntoCpu
                | ArchetypeCacheState::ShouldCopyIntoGpu => return,
            }
        }

        let Some(storage) = gpu_archetypes.get_archetype_storage(archetype_id) else {
            unreachable!("{archetype_id} should exist")
        };
        let storage_slices = storage.slices();

        let entities = unsafe { storage_slices.entities.as_slice() };
        let Some(source) = entities else {
            return;
        };

        let label = || format!("`gpecs` {archetype_id:#} entities download buffer");
        let archetype_cache = archetype_cache_entry
            .and_modify(|cache| {
                cache
                    .entities
                    .download
                    .copy_from_slice(device, command_encoder, source, label);
            })
            .or_insert_with(|| {
                let buffer = DownloadBuffer::from_slice(device, command_encoder, source, label());
                ArchetypeCache::new(buffer)
            });

        for (component_id, components) in storage_slices.components {
            let components = unsafe { components.as_slice() };
            let Some(source) = components else {
                continue;
            };

            let label = || format!("`gpecs` {archetype_id:#} {component_id:#} download buffer");
            archetype_cache
                .components
                .entry(component_id)
                .and_modify(|components| {
                    components
                        .download
                        .copy_from_slice(device, command_encoder, source, label);
                })
                .or_insert_with(|| {
                    let buffer =
                        DownloadBuffer::from_slice(device, command_encoder, source, label());
                    ArchetypeCacheEntry::new(buffer)
                });
        }

        archetype_cache.state = ArchetypeCacheState::CopiedFromGpu;
    }

    pub fn move_all_into<'a>(
        &mut self,
        cpu_archetypes: &'a mut ArchetypeRegistry,
    ) -> Result<&'a ArchetypeRegistry, MappedArchetypeNotReadyError> {
        let Self { archetypes } = self;

        for (&archetype_id, archetype_cache) in archetypes {
            let storage = unsafe { cpu_archetypes.get_archetype_storage_mut(archetype_id.into()) };
            let Some(storage) = storage else {
                unreachable!("{archetype_id} should exist")
            };

            Self::move_archetype_into_trusted(archetype_cache, storage)
                .map_err(|_| MappedArchetypeNotReadyError::new(archetype_id))?;
        }

        Ok(cpu_archetypes)
    }

    pub fn move_all_into_and_allow_mutation<'a>(
        &mut self,
        cpu_archetypes: &'a mut ArchetypeRegistry,
    ) -> Result<&'a mut ArchetypeRegistry, MappedArchetypeNotReadyError> {
        let Self { archetypes } = self;

        for (&archetype_id, archetype_cache) in archetypes {
            let storage = unsafe { cpu_archetypes.get_archetype_storage_mut(archetype_id.into()) };
            let Some(storage) = storage else {
                unreachable!("{archetype_id} should exist")
            };

            Self::move_archetype_into_trusted(archetype_cache, storage)
                .map_err(|_| MappedArchetypeNotReadyError::new(archetype_id))?;

            archetype_cache.state = match archetype_cache.state {
                ArchetypeCacheState::Invalidated | ArchetypeCacheState::CopiedFromGpu => {
                    unreachable!("wrong state detected: {:?}", archetype_cache.state)
                }
                ArchetypeCacheState::CopiedIntoCpu | ArchetypeCacheState::ShouldCopyIntoGpu => {
                    ArchetypeCacheState::ShouldCopyIntoGpu
                }
            };
        }

        Ok(cpu_archetypes)
    }

    pub fn move_archetype_into<'a>(
        &mut self,
        archetype_id: GpuArchetypeId,
        cpu_archetypes: &'a mut ArchetypeRegistry,
    ) -> Result<&'a ArchetypeStorage, MappedArchetypeNotReadyError> {
        let storage = unsafe { cpu_archetypes.get_archetype_storage_mut(archetype_id.into()) };
        let Some(storage) = storage else {
            unreachable!("{archetype_id} should exist")
        };

        let Self { archetypes } = self;
        let Some(archetype_cache) = archetypes.get_mut(&archetype_id) else {
            return Ok(storage);
        };

        Self::move_archetype_into_trusted(archetype_cache, storage)
            .map_err(|_| MappedArchetypeNotReadyError::new(archetype_id))?;
        Ok(storage)
    }

    pub fn move_archetype_into_and_allow_mutation<'a>(
        &mut self,
        archetype_id: GpuArchetypeId,
        cpu_archetypes: &'a mut ArchetypeRegistry,
    ) -> Result<&'a mut ArchetypeStorage, MappedArchetypeNotReadyError> {
        let storage = unsafe { cpu_archetypes.get_archetype_storage_mut(archetype_id.into()) };
        let Some(storage) = storage else {
            unreachable!("{archetype_id} should exist")
        };

        let Self { archetypes } = self;
        let Some(archetype_cache) = archetypes.get_mut(&archetype_id) else {
            return Ok(storage);
        };

        Self::move_archetype_into_trusted(archetype_cache, storage)
            .map_err(|_| MappedArchetypeNotReadyError::new(archetype_id))?;

        archetype_cache.state = match archetype_cache.state {
            ArchetypeCacheState::Invalidated | ArchetypeCacheState::CopiedFromGpu => {
                unreachable!("wrong state detected: {:?}", archetype_cache.state)
            }
            ArchetypeCacheState::CopiedIntoCpu | ArchetypeCacheState::ShouldCopyIntoGpu => {
                ArchetypeCacheState::ShouldCopyIntoGpu
            }
        };

        Ok(storage)
    }

    fn move_archetype_into_trusted(
        archetype_cache: &mut ArchetypeCache,
        storage: &mut ArchetypeStorage,
    ) -> Result<(), DownloadBufferNotReadyError> {
        match archetype_cache.state {
            ArchetypeCacheState::Invalidated => return Err(DownloadBufferNotReadyError),
            ArchetypeCacheState::CopiedFromGpu => (),
            ArchetypeCacheState::CopiedIntoCpu | ArchetypeCacheState::ShouldCopyIntoGpu => {
                return Ok(());
            }
        }

        let (entities, mut bundles, _) = unsafe { storage.as_mut_view().into_mut_slices() };

        let mapped_entities = archetype_cache.entities.download.as_slice()?;
        must_cast_slice_mut(entities).copy_from_slice(&mapped_entities);

        for (&component_id, components) in &archetype_cache.components {
            let mapped_components = components.download.as_slice()?;
            let Some(mut components) = bundles.get_mut(component_id.into()) else {
                continue;
            };

            let components = unsafe { components.as_mut_buffer() };
            components.write_copy_of_slice(&mapped_components);
        }

        archetype_cache.state = ArchetypeCacheState::CopiedIntoCpu;
        Ok(())
    }

    pub fn resync(
        &mut self,
        device: &Device,
        command_encoder: &mut CommandEncoder,
        schedule_cache: &mut ScheduleCache,
        cpu_archetypes: &ArchetypeRegistry,
        gpu_archetypes: &mut GpuArchetypeRegistry,
    ) {
        let Self { archetypes } = self;

        for (&archetype_id, archetype_cache) in archetypes {
            if let ArchetypeCacheState::ShouldCopyIntoGpu = archetype_cache.state {
                let cpu_storage = cpu_archetypes.get_archetype_storage(archetype_id.into());
                let Some(cpu_storage) = cpu_storage else {
                    unreachable!("{archetype_id} should exist")
                };

                let gpu_storage = unsafe { gpu_archetypes.get_archetype_storage_mut(archetype_id) };
                let Some(gpu_storage) = gpu_storage else {
                    unreachable!("{archetype_id} should exist")
                };

                Self::resync_archetype(
                    device,
                    command_encoder,
                    archetype_id,
                    archetype_cache,
                    cpu_storage,
                    gpu_storage,
                );
                schedule_cache.request_archetype_resync(archetype_id);
            }
            archetype_cache.state = ArchetypeCacheState::Invalidated;
        }
    }

    fn resync_archetype(
        device: &Device,
        command_encoder: &mut CommandEncoder,
        archetype_id: GpuArchetypeId,
        archetype_cache: &mut ArchetypeCache,
        cpu_storage: &ArchetypeStorage,
        gpu_storage: &mut GpuArchetypeStorage,
    ) {
        if gpu_storage.capacity() < cpu_storage.capacity() {
            *gpu_storage = GpuArchetypeStorage::new(device, archetype_id, cpu_storage);
            return;
        }

        unsafe { gpu_storage.set_len(cpu_storage.len()) }

        let (cpu_entities, cpu_bundles) = cpu_storage.as_slices();
        let gpu_slices = gpu_storage.slices();

        let gpu_entities = unsafe { gpu_slices.entities.as_slice() };
        let Some(gpu_entities) = gpu_entities else {
            return;
        };

        let staging_entities = &mut archetype_cache.entities.staging;
        let contents = must_cast_slice(cpu_entities);
        let label = || format!("`gpecs` {archetype_id:#} entities staging buffer");
        if let Some(staging_entities) = staging_entities {
            staging_entities.copy_from_slice(device, contents, label);
        }
        let staging_entities = staging_entities
            .get_or_insert_with(|| StagingBuffer::from_slice(device, contents, label()));

        staging_entities.copy_into_slice(command_encoder, gpu_entities);

        for (component_id, gpu_components) in gpu_slices.components {
            let Some(cpu_components) = cpu_bundles.get(component_id.into()) else {
                unreachable!("{component_id} should exist");
            };

            let gpu_components = unsafe { gpu_components.as_slice() };
            let Some(gpu_components) = gpu_components else {
                continue;
            };

            let Some(entry) = archetype_cache.components.get_mut(&component_id) else {
                unreachable!("{component_id} should exist");
            };
            let staging_components = &mut entry.staging;

            let contents = unsafe { cpu_components.as_buffer().assume_init_ref() };
            let label = || format!("`gpecs` {archetype_id:#} {component_id:#} staging buffer");
            if let Some(staging_components) = staging_components {
                staging_components.copy_from_slice(device, contents, label);
            }
            let staging_components = staging_components
                .get_or_insert_with(|| StagingBuffer::from_slice(device, contents, label()));

            staging_components.copy_into_slice(command_encoder, gpu_components);
        }
    }
}

#[derive(Debug)]
struct ArchetypeCache {
    state: ArchetypeCacheState,
    entities: ArchetypeCacheEntry,
    components: IndexMap<GpuComponentId, ArchetypeCacheEntry>,
}

impl ArchetypeCache {
    fn new(entities: DownloadBuffer) -> Self {
        Self {
            entities: ArchetypeCacheEntry::new(entities),
            components: IndexMap::default(),
            state: ArchetypeCacheState::Invalidated,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ArchetypeCacheState {
    Invalidated,
    CopiedFromGpu,
    CopiedIntoCpu,
    ShouldCopyIntoGpu,
}

#[derive(Debug)]
struct ArchetypeCacheEntry {
    download: DownloadBuffer,
    staging: Option<StagingBuffer>,
}

impl ArchetypeCacheEntry {
    fn new(download: DownloadBuffer) -> Self {
        Self {
            download,
            staging: None,
        }
    }
}

#[derive(Debug)]
struct DownloadBuffer {
    buffer: Buffer,
    init_size: BufferSize,
    is_mapped: Arc<AtomicBool>,
}

impl DownloadBuffer {
    #[inline]
    fn from_slice(
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
    fn copy_from_slice<L>(
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
    fn as_slice(&self) -> Result<BufferView, DownloadBufferNotReadyError> {
        let Self {
            buffer,
            init_size,
            is_mapped,
        } = self;

        if !is_mapped.load(Ordering::Acquire) {
            return Err(DownloadBufferNotReadyError);
        }

        let view = buffer.get_mapped_range(..init_size.get());
        Ok(view)
    }
}

struct DownloadBufferNotReadyError;

#[derive(Debug)]
struct StagingBuffer {
    buffer: Buffer,
    init_size: BufferAddress,
}

impl StagingBuffer {
    #[inline]
    fn from_slice(device: &Device, contents: &[u8], label: impl AsRef<str>) -> Self {
        let init_size = BufferAddress::try_from(contents.len())
            .expect("staging buffer size should fit into `BufferAddress`");

        let desc = BufferInitDescriptor {
            label: Some(label.as_ref()),
            usage: BufferUsages::MAP_WRITE | BufferUsages::COPY_SRC,
            contents,
        };
        let buffer = device.create_buffer_init(&desc);

        Self { buffer, init_size }
    }

    #[inline]
    fn copy_from_slice<L>(&mut self, device: &Device, contents: &[u8], label: impl FnOnce() -> L)
    where
        L: AsRef<str>,
    {
        let Self { buffer, init_size } = self;

        let new_size = BufferAddress::try_from(contents.len())
            .expect("staging buffer size should fit into `BufferAddress`");
        if buffer.size() < new_size {
            *self = Self::from_slice(device, contents, label());
            return;
        }

        *init_size = new_size;
        buffer
            .get_mapped_range_mut(0..*init_size)
            .copy_from_slice(contents);
        buffer.unmap();
    }

    #[inline]
    fn copy_into_slice(&self, command_encoder: &mut CommandEncoder, destination: BufferSlice<'_>) {
        let Self { buffer, .. } = self;

        command_encoder.copy_buffer_to_buffer(
            buffer,
            0,
            destination.buffer(),
            destination.offset(),
            destination.size().get(),
        );
        command_encoder.map_buffer_on_submit(buffer, MapMode::Write, .., |_| {});
    }
}
