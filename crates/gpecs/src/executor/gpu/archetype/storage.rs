use std::ptr;

use indexmap::IndexMap;
use wgpu::{
    Buffer, BufferAddress, BufferBinding, BufferSize, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    archetype::registry::ArchetypeInfo, component::registry::ComponentRegistry,
    prelude::ComponentId,
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
    _download_buffer: Option<Buffer>,
    entities_binding: Option<BufferBindingDescriptor>,
    component_bindings: IndexMap<ComponentId, Option<BufferBindingDescriptor>>,
}

impl GpuArchetypeStorage {
    #[inline]
    #[allow(unsafe_code)]
    pub(super) fn new(
        components: &ComponentRegistry,
        gpu_device: &Device,
        info: &ArchetypeInfo,
    ) -> Self {
        let (entities, erased_components) = info.storage().erased_components(components);
        let mut component_bindings = IndexMap::with_capacity(erased_components.len());

        let entities_byte_count = size_of_val(entities);
        let entities_binding = u64::try_from(entities_byte_count)
            .unwrap()
            .try_into()
            .ok()
            .map(|size| BufferBindingDescriptor { offset: 0, size });

        let min_offset_alignment = gpu_device
            .limits()
            .min_storage_buffer_offset_alignment
            .into();
        let mut contents = vec![0; entities_byte_count];
        unsafe {
            let src = entities.as_ptr().cast();
            let dst = contents.as_mut_ptr();
            ptr::copy_nonoverlapping(src, dst, entities_byte_count);

            let mut offset = BufferAddress::try_from(entities_byte_count)
                .expect("entities byte count should fit into `BufferAddress`");
            for (&component_id, slice) in erased_components.iter() {
                offset = offset.div_ceil(min_offset_alignment) * min_offset_alignment;

                let components_bytes = slice.buffer();
                let components_byte_count = components_bytes.len();
                let contents_offset = usize::try_from(offset).unwrap();
                contents.resize(contents_offset + components_byte_count, 0);

                let src = components_bytes.as_ptr().cast();
                let dst = contents.as_mut_ptr().add(contents_offset);
                ptr::copy_nonoverlapping(src, dst, components_byte_count);

                let components_binding = u64::try_from(components_byte_count)
                    .unwrap()
                    .try_into()
                    .ok()
                    .map(|size| BufferBindingDescriptor { offset, size });
                component_bindings.insert(component_id, components_binding);

                offset += BufferAddress::try_from(components_byte_count)
                    .expect("components byte count should fit into `BufferAddress`");
            }
        }

        let archetype_id = unsafe { GpuArchetypeId::from_id(info.id()) };
        let storage_buffer_label = format!("`gpecs` {archetype_id:?} storage buffer");
        let storage_buffer_desc = BufferInitDescriptor {
            label: Some(&storage_buffer_label),
            contents: &contents,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        };
        let storage_buffer = gpu_device.create_buffer_init(&storage_buffer_desc);

        Self {
            len: entities.len(),
            storage_buffer,
            _download_buffer: None,
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
    #[allow(unsafe_code)]
    pub unsafe fn storage_buffer(&self) -> &Buffer {
        let Self { storage_buffer, .. } = self;
        storage_buffer
    }

    #[inline]
    #[allow(unsafe_code)]
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
