use std::ops::Range;

use bytemuck::must_cast_slice;
use indexmap::IndexMap;
use wgpu::{
    Buffer, BufferAddress, BufferSize, BufferSlice, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    archetype::storage::ArchetypeStorage, component::registry::ComponentRegistry,
    executor::gpu::component::registry::GpuComponentId,
};

use super::registry::GpuArchetypeId;

#[derive(Debug)]
pub struct GpuArchetypeStorage {
    len: usize,
    storage_buffer: Buffer,
    #[expect(dead_code)]
    download_buffer: Option<Buffer>,
    entities_binding: Option<BufferBindingDescriptor>,
    component_bindings: IndexMap<GpuComponentId, Option<BufferBindingDescriptor>>,
}

impl GpuArchetypeStorage {
    #[inline]
    pub(super) fn new(
        components: &ComponentRegistry,
        gpu_device: &Device,
        archetype_id: GpuArchetypeId,
        archetype_storage: &ArchetypeStorage,
    ) -> Self {
        let (entities, erased_components) = archetype_storage.erased_components(components);
        let len = archetype_storage.len();

        let entities_bytes = must_cast_slice(entities);
        let entities_byte_count = entities_bytes.len();
        let entities_binding = u64::try_from(entities_byte_count)
            .expect("entities byte count should fit into `u64`")
            .try_into()
            .ok()
            .map(|size| BufferBindingDescriptor { offset: 0, size });

        let min_offset_align = gpu_device.limits().min_storage_buffer_offset_alignment;
        let min_offset_align = min_offset_align
            .try_into()
            .expect("min storage buffer offset alignment should fit into `usize`");

        let mut components_offset = entities_byte_count;
        let mut storage_buffer_contents = Vec::from(entities_bytes);
        let component_bindings = erased_components
            .into_iter()
            .map(|(component_id, slice)| {
                components_offset = components_offset.next_multiple_of(min_offset_align);
                storage_buffer_contents.resize(components_offset, 0);

                let components_bytes = slice.buffer();
                storage_buffer_contents.extend_from_slice(components_bytes);

                let components_byte_count = components_bytes.len();
                let offset = BufferAddress::try_from(components_offset)
                    .expect("components offset should fit into `BufferAddress`");
                components_offset += components_byte_count;

                let gpu_component_id = unsafe { GpuComponentId::from_id(component_id) };
                let components_binding = u64::try_from(components_byte_count)
                    .expect("components byte count should fit into `u64`")
                    .try_into()
                    .ok()
                    .map(|size| BufferBindingDescriptor { offset, size });
                (gpu_component_id, components_binding)
            })
            .collect();

        let storage_buffer_label = format!("`gpecs` {archetype_id:?} storage buffer");
        let storage_buffer_desc = BufferInitDescriptor {
            label: Some(&storage_buffer_label),
            contents: storage_buffer_contents.as_slice(),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        };
        let storage_buffer = gpu_device.create_buffer_init(&storage_buffer_desc);

        Self {
            len,
            storage_buffer,
            download_buffer: None,
            entities_binding,
            component_bindings,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub unsafe fn storage_buffer(&self) -> &Buffer {
        let Self { storage_buffer, .. } = self;
        storage_buffer
    }

    #[inline]
    pub unsafe fn storage_buffer_slices(&self) -> GpuArchetypeStorageBufferSlices<'_> {
        let Self {
            storage_buffer,
            entities_binding,
            component_bindings,
            ..
        } = self;

        let to_slice = |binding| storage_buffer.slice(Range::from(binding));
        GpuArchetypeStorageBufferSlices {
            entities: entities_binding.map(to_slice),
            components: component_bindings
                .iter()
                .map(|(&component_id, binding)| (component_id, binding.map(to_slice)))
                .collect(),
        }
    }
}
#[derive(Debug)]
pub struct GpuArchetypeStorageBufferSlices<'a> {
    pub entities: Option<BufferSlice<'a>>,
    pub components: IndexMap<GpuComponentId, Option<BufferSlice<'a>>>,
}

#[derive(Debug, Clone, Copy)]
struct BufferBindingDescriptor {
    offset: BufferAddress,
    size: BufferSize,
}

impl From<BufferBindingDescriptor> for Range<BufferAddress> {
    #[inline]
    fn from(binding: BufferBindingDescriptor) -> Self {
        let BufferBindingDescriptor { offset, size } = binding;
        offset..(offset + size.get())
    }
}
