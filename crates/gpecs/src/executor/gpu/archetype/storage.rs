use std::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
};

use bytemuck::must_cast_slice;
use indexmap::map::Iter as IndexMapIter;
use wgpu::{
    Buffer, BufferAddress, BufferSlice, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    archetype::storage::ArchetypeStorage,
    component::erased::ErasedComponentSlice,
    entity::Entity,
    executor::gpu::component::registry::{GpuComponentId, GpuComponentInfo},
    hash::IndexMap,
};

use super::registry::GpuArchetypeId;

#[derive(Debug)]
pub struct GpuArchetypeStorage {
    len: usize,
    capacity: usize,
    entities_buffer: StorageBuffer,
    component_buffers: IndexMap<GpuComponentId, StorageBuffer>,
}

impl GpuArchetypeStorage {
    pub(in crate::executor::gpu) fn new(
        gpu_device: &Device,
        archetype_id: GpuArchetypeId,
        archetype_storage: &ArchetypeStorage,
    ) -> Self {
        let (entities, bundles) = archetype_storage.as_slices();
        let len = archetype_storage.len();
        let capacity = archetype_storage.capacity();

        let entities_label = || format!("`gpecs` {archetype_id:#} entities storage buffer");
        let entities_buffer =
            StorageBuffer::from_entities(gpu_device, entities, capacity, entities_label);

        let component_buffers = bundles
            .into_iter()
            .map(|components| {
                let component_id = unsafe { GpuComponentId::from_id(components.component_id()) };
                let components_label =
                    || format!("`gpecs` {archetype_id:#} {component_id:#} storage buffer");

                let buffer = StorageBuffer::from_components(
                    gpu_device,
                    components,
                    capacity,
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
    #[expect(unused)]
    pub(in crate::executor::gpu) unsafe fn set_len(&mut self, new_len: usize) {
        let Self {
            capacity,
            ref mut len,
            ref mut entities_buffer,
            ref mut component_buffers,
        } = *self;

        debug_assert!(new_len <= capacity);
        *len = new_len;

        unsafe { entities_buffer.set_len(new_len) }
        for components in component_buffers.values_mut() {
            unsafe { components.set_len(new_len) }
        }
    }

    #[inline]
    pub fn slices(&self) -> GpuArchetypeStorageSlices<'_> {
        let Self {
            entities_buffer,
            component_buffers,
            ..
        } = self;

        GpuArchetypeStorageSlices {
            entities: entities_buffer.as_slice(),
            components: GpuArchetypeStorageComponentSlices {
                inner: component_buffers.iter(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpuArchetypeStorageSlice<'a> {
    slice: Option<BufferSlice<'a>>,
    item_layout: Layout,
}

impl<'a> GpuArchetypeStorageSlice<'a> {
    #[inline]
    pub unsafe fn as_slice(&self) -> Option<BufferSlice<'a>> {
        let Self { slice, .. } = *self;
        slice
    }

    #[inline]
    pub fn item_layout(&self) -> Layout {
        let Self { item_layout, .. } = *self;
        item_layout
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GpuArchetypeStorageSlices<'a> {
    pub entities: GpuArchetypeStorageSlice<'a>,
    pub components: GpuArchetypeStorageComponentSlices<'a>,
}

#[derive(Clone)]
pub struct GpuArchetypeStorageComponentSlices<'a> {
    inner: IndexMapIter<'a, GpuComponentId, StorageBuffer>,
}

impl GpuArchetypeStorageComponentSlices<'_> {
    #[inline]
    fn map_inner_item<'a>(
        item: (&GpuComponentId, &'a StorageBuffer),
    ) -> GpuComponentInfo<GpuArchetypeStorageSlice<'a>> {
        let (&component_id, storage_buffer) = item;
        let slice = storage_buffer.as_slice();
        GpuComponentInfo::new(component_id, slice)
    }
}

impl Debug for GpuArchetypeStorageComponentSlices<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.clone()).finish()
    }
}

impl<'a> Iterator for GpuArchetypeStorageComponentSlices<'a> {
    type Item = GpuComponentInfo<GpuArchetypeStorageSlice<'a>>;

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
    init_size: BufferAddress,
    item_layout: Layout,
}

impl StorageBuffer {
    #[inline]
    fn from_entities<L>(
        device: &Device,
        entities: &[Entity],
        capacity: usize,
        label: impl FnOnce() -> L,
    ) -> Self
    where
        L: AsRef<str>,
    {
        let contents = must_cast_slice(entities);
        let item_layout = Layout::new::<Entity>();
        let capacity_in_bytes = item_layout.size().strict_mul(capacity);
        unsafe { Self::from_contents(device, item_layout, contents, capacity_in_bytes, label) }
    }

    #[inline]
    fn from_components<L>(
        device: &Device,
        components: ErasedComponentSlice<'_, *const MaybeUninit<u8>>,
        capacity: usize,
        label: impl FnOnce() -> L,
    ) -> Self
    where
        L: AsRef<str>,
    {
        // SAFETY: GPU components implement `NoUninit`, and so all the bytes of its slice should be initialized, too
        let contents = unsafe { components.as_buffer().assume_init_ref() };
        let item_layout = components.fields().layout();
        let capacity_in_bytes = item_layout.size().strict_mul(capacity);
        unsafe { Self::from_contents(device, item_layout, contents, capacity_in_bytes, label) }
    }

    unsafe fn from_contents<L>(
        device: &Device,
        item_layout: Layout,
        contents: &[u8],
        capacity_in_bytes: usize,
        label: impl FnOnce() -> L,
    ) -> Self
    where
        L: AsRef<str>,
    {
        let init_size = BufferAddress::try_from(contents.len())
            .expect("storage buffer size should fit into `BufferAddress`");

        let new_contents_capacity = usize::max(contents.len(), capacity_in_bytes);
        let mut new_contents = Vec::with_capacity(new_contents_capacity);
        new_contents.extend_from_slice(contents);
        new_contents.resize(capacity_in_bytes, 0);

        let contents = new_contents.as_slice();
        assert!(contents.len().is_multiple_of(item_layout.size()));

        let label = label();
        let desc = BufferInitDescriptor {
            label: Some(label.as_ref()),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            contents,
        };
        let buffer = device.create_buffer_init(&desc);

        Self {
            buffer,
            init_size,
            item_layout,
        }
    }

    #[inline]
    fn as_slice(&self) -> GpuArchetypeStorageSlice<'_> {
        let Self {
            ref buffer,
            init_size,
            item_layout,
        } = *self;

        let slice = (init_size != 0).then(|| buffer.slice(0..init_size));
        GpuArchetypeStorageSlice { slice, item_layout }
    }

    #[inline]
    unsafe fn set_len(&mut self, new_len: usize) {
        let Self {
            ref buffer,
            ref mut init_size,
            item_layout,
        } = *self;

        let new_init_size = BufferAddress::try_from(item_layout.size().strict_mul(new_len))
            .expect("storage buffer size should fit into `BufferAddress`");
        debug_assert!(new_init_size <= buffer.size());

        *init_size = new_init_size;
    }
}
