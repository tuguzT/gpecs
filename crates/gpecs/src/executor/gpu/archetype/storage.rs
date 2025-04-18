use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

use crate::archetype::registry::ArchetypeInfo;

#[derive(Debug)]
pub struct GpuArchetypeStorage {
    buffer: Buffer,
    // TODO: store byte offsets to each component slice of an archetype
}

impl GpuArchetypeStorage {
    #[inline]
    pub fn new(gpu_device: &Device, info: &ArchetypeInfo) -> Self {
        let archetype_id = info.id();
        let label = format!("`gpecs` {archetype_id:?} buffer");

        let storage = info.storage();
        // TODO: copy all the data from CPU storage to GPU buffer
        let contents = vec![0; storage.capacity() + 1];

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
