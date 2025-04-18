use std::ptr;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

use crate::{
    archetype::registry::ArchetypeInfo, component::registry::ComponentRegistry, entity::Entity,
};

#[derive(Debug)]
pub struct GpuArchetypeStorage {
    buffer: Buffer,
    // TODO: store byte offsets to each component slice of an archetype
}

impl GpuArchetypeStorage {
    #[inline]
    #[allow(unsafe_code)]
    pub fn new(components: &ComponentRegistry, gpu_device: &Device, info: &ArchetypeInfo) -> Self {
        let archetype_id = info.id();
        let label = format!("`gpecs` {archetype_id:?} buffer");

        let (entities, _erased_components) = info.storage().erased_components(components);

        let mut contents = vec![0; entities.len() * size_of::<Entity>()];
        unsafe {
            let src = entities.as_ptr().cast();
            let dst = contents.as_mut_ptr();
            ptr::copy_nonoverlapping(src, dst, entities.len() * size_of::<Entity>());

            // TODO: copy component data from CPU storage to GPU buffer
        }

        let buffer_desc = BufferInitDescriptor {
            label: Some(&label),
            contents: &contents,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        };
        let buffer = gpu_device.create_buffer_init(&buffer_desc);

        Self { buffer }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn buffer(&self) -> &Buffer {
        let Self { buffer, .. } = self;
        buffer
    }
}
