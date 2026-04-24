use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use bytemuck::must_cast_slice;
use indexmap::map::Iter as IndexMapIter;
use wgpu::{
    Buffer, BufferAddress, BufferSize, BufferSlice, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    archetype::storage::ArchetypeStorage, entity::Entity,
    executor::gpu::component::registry::GpuComponentId, hash::IndexMap,
};

use super::registry::GpuArchetypeId;

#[derive(Debug)]
pub struct GpuArchetypeStorage {
    len: usize,
    capacity: usize,
    entities_buffer: Option<StorageBuffer>,
    component_buffers: IndexMap<GpuComponentId, Option<StorageBuffer>>,
}

impl GpuArchetypeStorage {
    #[inline]
    pub(super) fn new(
        gpu_device: &Device,
        archetype_id: GpuArchetypeId,
        archetype_storage: &ArchetypeStorage,
    ) -> Self {
        let (entities, bundles) = archetype_storage.as_slices();
        let len = archetype_storage.len();
        let capacity = archetype_storage.capacity();

        let entities_contents = must_cast_slice(entities);
        let entities_label = || format!("`gpecs` {archetype_id:#} entities storage buffer");
        let entities_capacity_in_bytes = size_of::<Entity>().strict_mul(capacity);
        let entities_buffer = StorageBuffer::new(
            gpu_device,
            entities_contents,
            entities_capacity_in_bytes,
            entities_label,
        );

        let component_buffers = bundles
            .into_iter()
            .map(|components| {
                let component_id = unsafe { GpuComponentId::from_id(components.component_id()) };
                let components_label =
                    || format!("`gpecs` {archetype_id:#} {component_id:#} storage buffer");

                // SAFETY: GPU components implement `NoUninit`, and so all the bytes of its slice should be initialized, too
                let components_contents = unsafe { components.as_buffer().assume_init_ref() };
                let components_capacity_in_bytes =
                    components.fields().layout().size().strict_mul(capacity);
                let buffer = StorageBuffer::new(
                    gpu_device,
                    components_contents,
                    components_capacity_in_bytes,
                    components_label,
                );
                (component_id, buffer)
            })
            .collect();

        Self {
            len,
            capacity,
            entities_buffer,
            component_buffers,
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
    pub fn capacity(&self) -> usize {
        let Self { capacity, .. } = *self;
        capacity
    }

    #[inline]
    pub fn slices(&self) -> GpuArchetypeStorageSlices<'_> {
        let Self {
            entities_buffer,
            component_buffers,
            ..
        } = self;

        GpuArchetypeStorageSlices {
            entities: entities_buffer.as_ref().map(StorageBuffer::as_slice),
            components: GpuArchetypeStorageComponentSlices {
                inner: component_buffers.iter(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpuArchetypeStorageSlice<'a> {
    slice: BufferSlice<'a>,
}

impl<'a> GpuArchetypeStorageSlice<'a> {
    #[inline]
    pub fn size(&self) -> BufferSize {
        let Self { slice } = self;
        slice.size()
    }

    #[inline]
    pub fn offset(&self) -> BufferAddress {
        let Self { slice } = self;
        slice.offset()
    }

    #[inline]
    pub unsafe fn as_slice(&self) -> BufferSlice<'a> {
        let Self { slice } = *self;
        slice
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GpuArchetypeStorageSlices<'a> {
    pub entities: Option<GpuArchetypeStorageSlice<'a>>,
    pub components: GpuArchetypeStorageComponentSlices<'a>,
}

#[derive(Clone)]
pub struct GpuArchetypeStorageComponentSlices<'a> {
    inner: IndexMapIter<'a, GpuComponentId, Option<StorageBuffer>>,
}

impl GpuArchetypeStorageComponentSlices<'_> {
    #[inline]
    fn map_inner_item<'a>(
        item: (&GpuComponentId, &'a Option<StorageBuffer>),
    ) -> (GpuComponentId, Option<GpuArchetypeStorageSlice<'a>>) {
        let (&component_id, storage_buffer) = item;
        let slice = storage_buffer.as_ref().map(StorageBuffer::as_slice);
        (component_id, slice)
    }
}

impl Debug for GpuArchetypeStorageComponentSlices<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.clone()).finish()
    }
}

impl<'a> Iterator for GpuArchetypeStorageComponentSlices<'a> {
    type Item = (GpuComponentId, Option<GpuArchetypeStorageSlice<'a>>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next().map(Self::map_inner_item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        let Self { inner, .. } = self;
        inner.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth(n).map(Self::map_inner_item)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.last().map(Self::map_inner_item)
    }

    #[inline]
    fn collect<B>(self) -> B
    where
        B: FromIterator<Self::Item>,
    {
        let Self { inner } = self;
        inner.map(Self::map_inner_item).collect()
    }
}

impl DoubleEndedIterator for GpuArchetypeStorageComponentSlices<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.next_back().map(Self::map_inner_item)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self { inner } = self;
        inner.nth_back(n).map(Self::map_inner_item)
    }
}

impl ExactSizeIterator for GpuArchetypeStorageComponentSlices<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }
}

impl FusedIterator for GpuArchetypeStorageComponentSlices<'_> {}

#[derive(Debug, Clone)]
struct StorageBuffer {
    buffer: Buffer,
    init_size: BufferSize,
}

impl StorageBuffer {
    #[inline]
    fn new<L>(
        device: &Device,
        contents: &[u8],
        capacity_in_bytes: usize,
        label: impl FnOnce() -> L,
    ) -> Option<Self>
    where
        L: AsRef<str>,
    {
        let init_size = BufferAddress::try_from(contents.len())
            .expect("contents size should fit into `BufferAddress`")
            .try_into()
            .ok()?;

        let mut contents = contents.to_vec();
        contents.resize(capacity_in_bytes, 0);

        let label = label();
        let desc = BufferInitDescriptor {
            label: Some(label.as_ref()),
            contents: contents.as_slice(),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
        };
        let buffer = device.create_buffer_init(&desc);

        let me = Self { buffer, init_size };
        Some(me)
    }

    #[inline]
    fn as_slice(&self) -> GpuArchetypeStorageSlice<'_> {
        let Self {
            ref buffer,
            init_size,
        } = *self;

        let size = init_size.into();
        let slice = buffer.slice(0..size);
        GpuArchetypeStorageSlice { slice }
    }
}
