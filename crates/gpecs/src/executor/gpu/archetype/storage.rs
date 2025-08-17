use std::ptr;

use bytemuck::must_cast_slice;
use indexmap::IndexMap;
use wgpu::{
    Buffer, BufferAddress, BufferBinding, BufferSize, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    archetype::registry::ArchetypeInfo,
    component::registry::{ComponentId, ComponentRegistry},
};

use super::registry::GpuArchetypeId;

#[derive(Debug, Clone, Copy)]
struct BufferBindingDescriptor {
    offset: BufferAddress,
    size: BufferSize,
}

#[derive(Debug)]
pub struct BufferBindings<'a> {
    pub entities: Option<BufferBinding<'a>>,
    pub components: IndexMap<ComponentId, Option<BufferBinding<'a>>>,
}

#[derive(Debug)]
pub struct GpuArchetypeStorage {
    len: usize,
    storage_buffer: Buffer,
    #[expect(dead_code)]
    download_buffer: Option<Buffer>,
    entities_binding: Option<BufferBindingDescriptor>,
    component_bindings: IndexMap<ComponentId, Option<BufferBindingDescriptor>>,
}

impl GpuArchetypeStorage {
    #[inline]
    pub(super) fn new(
        components: &ComponentRegistry,
        gpu_device: &Device,
        info: &ArchetypeInfo,
    ) -> Self {
        let storage = info.storage();
        let len = storage.len();

        let (entities, erased_components) = storage.erased_components(components);
        let mut component_bindings = IndexMap::with_capacity(erased_components.len());

        let entities_bytes = must_cast_slice(entities);
        let entities_byte_count = entities_bytes.len();
        let entities_binding = u64::try_from(entities_byte_count)
            .expect("entities byte count should fit into `u64`")
            .try_into()
            .ok()
            .map(|size| BufferBindingDescriptor { offset: 0, size });

        let mut contents = Vec::from(entities_bytes);

        let min_offset_align = gpu_device.limits().min_storage_buffer_offset_alignment;
        let min_offset_align = min_offset_align
            .try_into()
            .expect("min storage buffer offset alignment should fit into `usize`");

        let mut components_offset = entities_byte_count;
        for (component_id, slice) in erased_components {
            components_offset = components_offset.next_multiple_of(min_offset_align);
            let offset = BufferAddress::try_from(components_offset)
                .expect("components offset should fit into `BufferAddress`");

            let components_bytes = slice.buffer();
            let components_byte_count = components_bytes.len();
            contents.resize(components_offset + components_byte_count, 0);

            let src = components_bytes.as_ptr().cast();
            let dst = unsafe { contents.as_mut_ptr().add(components_offset) };
            unsafe {
                ptr::copy_nonoverlapping(src, dst, components_byte_count);
            }

            let components_binding = u64::try_from(components_byte_count)
                .expect("components byte count should fit into `u64`")
                .try_into()
                .ok()
                .map(|size| BufferBindingDescriptor { offset, size });
            component_bindings.insert(component_id, components_binding);

            components_offset += components_byte_count;
        }

        let archetype_id = unsafe { GpuArchetypeId::from_id(info.id()) };
        let storage_buffer_label = format!("`gpecs` {archetype_id:?} storage buffer");
        let storage_buffer_desc = BufferInitDescriptor {
            label: Some(&storage_buffer_label),
            contents: contents.as_slice(),
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
    pub unsafe fn storage_buffer_bindings(&self) -> BufferBindings<'_> {
        let Self {
            storage_buffer,
            entities_binding,
            component_bindings,
            ..
        } = self;

        let map_binding = |binding: BufferBindingDescriptor| BufferBinding {
            buffer: storage_buffer,
            offset: binding.offset,
            size: Some(binding.size),
        };
        BufferBindings {
            entities: entities_binding.map(map_binding),
            components: component_bindings
                .iter()
                .map(|(&component_id, binding)| (component_id, binding.map(map_binding)))
                .collect(),
        }
    }
}
