use std::ptr;

use indexmap::IndexMap;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferAddress, BufferBinding, BufferSize, BufferUsages, Device,
};

use crate::{
    archetype::registry::ArchetypeInfo, component::registry::ComponentRegistry, entity::Entity,
    prelude::ComponentId,
};

#[derive(Debug, Clone, Copy)]
struct BufferBindingDescriptor {
    offset: BufferAddress,
    size: BufferSize,
}

#[derive(Debug, Clone)]
pub struct BufferBindings<'a> {
    pub entities: Option<BufferBinding<'a>>,
    pub components: IndexMap<ComponentId, Option<BufferBinding<'a>>>,
}

#[derive(Debug)]
pub struct GpuArchetypeStorage {
    buffer: Buffer,
    entities_binding: Option<BufferBindingDescriptor>,
    component_bindings: IndexMap<ComponentId, Option<BufferBindingDescriptor>>,
}

impl GpuArchetypeStorage {
    #[inline]
    #[allow(unsafe_code)]
    pub fn new(components: &ComponentRegistry, gpu_device: &Device, info: &ArchetypeInfo) -> Self {
        let (entities, erased_components) = info.storage().erased_components(components);
        let mut component_bindings = IndexMap::with_capacity(erased_components.len());

        let entities_byte_count = entities.len() * size_of::<Entity>();
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

            let mut offset = BufferAddress::try_from(entities_byte_count).unwrap();
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
            }
        }

        let buffer_label = format!("`gpecs` {:?} storage buffer", info.id());
        let buffer_desc = BufferInitDescriptor {
            label: Some(&buffer_label),
            contents: &contents,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        };
        let buffer = gpu_device.create_buffer_init(&buffer_desc);

        Self {
            buffer,
            entities_binding,
            component_bindings,
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn buffer(&self) -> &Buffer {
        let Self { buffer, .. } = self;
        buffer
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn buffer_bindings(&self) -> BufferBindings {
        let Self {
            buffer,
            entities_binding,
            component_bindings,
        } = self;

        let map_binding = |binding: BufferBindingDescriptor| BufferBinding {
            buffer,
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
