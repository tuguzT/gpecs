use std::{
    alloc::{Layout, LayoutError},
    iter::{FusedIterator, zip},
    ops::Range,
};

use bytemuck::must_cast_slice;
use indexmap::IndexMap;
use wgpu::{
    Buffer, BufferAddress, BufferSize, BufferSlice, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    archetype::storage::ArchetypeStorage,
    component::registry::ComponentRegistry,
    entity::Entity,
    executor::gpu::component::registry::GpuComponentId,
    soa::field::{BufferOffset, CopiedFieldDescriptors, FieldDescriptor, repeat_layout},
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
        let entities_binding = BufferAddress::try_from(entities_bytes.len())
            .expect("entities' byte count should fit into `BufferAddress`")
            .try_into()
            .ok()
            .map(|size| BufferBindingDescriptor { offset: 0, size });

        let fields = erased_components
            .iter()
            .map(|(_, slice)| slice.descriptor());
        let offsets = storage_buffer_offsets(fields, len, gpu_device);

        let mut storage_buffer_contents = Vec::from(entities_bytes);
        let component_bindings = zip(&erased_components, offsets)
            .map(|((&component_id, slice), offset)| {
                let BufferOffset { offset, .. } = offset.unwrap();
                let component_bytes = slice.buffer();

                storage_buffer_contents.resize(offset, 0);
                storage_buffer_contents.extend_from_slice(component_bytes);

                let gpu_component_id = unsafe { GpuComponentId::from_id(component_id) };
                let offset = BufferAddress::try_from(offset)
                    .expect("components' offset should fit into `BufferAddress`");
                let components_binding = BufferAddress::try_from(component_bytes.len())
                    .expect("components' byte count should fit into `BufferAddress`")
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

#[derive(Debug, Clone)]
struct GpuArchetypeStorageBufferOffsets<I>
where
    I: ?Sized,
{
    layout: Layout,
    capacity: usize,
    min_offset_align: usize,
    fields: CopiedFieldDescriptors<I>,
}

impl<I> GpuArchetypeStorageBufferOffsets<I>
where
    I: ?Sized,
{
    #[inline]
    #[expect(dead_code)]
    pub const fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    #[expect(dead_code)]
    pub const fn capacity(&self) -> usize {
        let Self { capacity, .. } = *self;
        capacity
    }

    #[inline]
    #[expect(dead_code)]
    pub const fn min_offset_align(&self) -> usize {
        let Self {
            min_offset_align, ..
        } = *self;
        min_offset_align
    }
}

impl<I> GpuArchetypeStorageBufferOffsets<I> {
    #[inline]
    #[expect(dead_code)]
    pub fn into_fields(self) -> I {
        let Self { fields, .. } = self;
        fields.into_inner()
    }
}

impl<I> Iterator for GpuArchetypeStorageBufferOffsets<I>
where
    I: Iterator + ?Sized,
    I::Item: AsRef<FieldDescriptor>,
{
    type Item = Result<BufferOffset, LayoutError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut fields,
            ref mut layout,
            capacity,
            min_offset_align,
        } = *self;

        let desc = fields.next()?;
        let item = try_create_buffer_offset(desc, layout, capacity, min_offset_align);
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { fields, .. } = self;
        fields.size_hint()
    }
}

impl<I> ExactSizeIterator for GpuArchetypeStorageBufferOffsets<I>
where
    I: ExactSizeIterator + ?Sized,
    I::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { fields, .. } = self;
        fields.len()
    }
}

impl<I> FusedIterator for GpuArchetypeStorageBufferOffsets<I>
where
    I: FusedIterator + ?Sized,
    I::Item: AsRef<FieldDescriptor>,
{
}

#[inline]
fn try_create_buffer_offset(
    field_descriptor: FieldDescriptor,
    layout: &mut Layout,
    capacity: usize,
    min_offset_align: usize,
) -> Result<BufferOffset, LayoutError> {
    let fields_layout = repeat_layout(field_descriptor.layout(), capacity)?;

    let offset;
    let next = fields_layout.align_to(min_offset_align)?.pad_to_align();
    (*layout, offset) = layout.extend(next)?;

    let buffer_offset = BufferOffset {
        field_descriptor,
        fields_layout,
        offset,
    };
    Ok(buffer_offset)
}

#[inline]
fn storage_buffer_offsets<I>(
    fields: I,
    capacity: usize,
    device: &Device,
) -> GpuArchetypeStorageBufferOffsets<I::IntoIter>
where
    I: IntoIterator<Item: AsRef<FieldDescriptor>>,
{
    let layout = Layout::new::<Entity>();
    let layout = repeat_layout(layout, capacity).expect("entities' layout should be valid");
    let fields = fields.into_iter().into();
    let min_offset_align = device
        .limits()
        .min_storage_buffer_offset_alignment
        .try_into()
        .expect("min storage buffer offset alignment should fit into `usize`");

    GpuArchetypeStorageBufferOffsets {
        layout,
        capacity,
        min_offset_align,
        fields,
    }
}
