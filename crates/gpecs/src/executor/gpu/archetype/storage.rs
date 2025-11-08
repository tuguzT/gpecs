use std::{
    alloc::{Layout, LayoutError},
    error::Error,
    fmt::{self, Debug, Display},
    iter::{FusedIterator, zip},
    num::TryFromIntError,
    ops::Range,
};

use bytemuck::must_cast_slice;
use indexmap::map::Iter as IndexMapIter;
use itertools::{Itertools, chain};
use wgpu::{
    Buffer, BufferAddress, BufferSize, BufferSlice, BufferUsages, COPY_BUFFER_ALIGNMENT, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    archetype::storage::ArchetypeStorage,
    component::registry::ComponentRegistry,
    entity::Entity,
    executor::gpu::component::registry::GpuComponentId,
    hash::IndexMap,
    soa::field::{BufferOffset, CopiedFieldDescriptors, FieldDescriptor, repeat_layout},
};

use super::registry::GpuArchetypeId;

#[derive(Debug)]
pub struct GpuArchetypeStorage {
    len: usize,
    storage_buffer: Buffer,
    entities_region: Option<StorageBufferRegion>,
    component_regions: IndexMap<GpuComponentId, Option<StorageBufferRegion>>,
}

impl GpuArchetypeStorage {
    #[inline]
    pub(super) fn new(
        components: &ComponentRegistry,
        gpu_device: &Device,
        archetype_id: GpuArchetypeId,
        archetype_storage: &ArchetypeStorage,
    ) -> Result<Self, GpuArchetypeStorageError> {
        use GpuArchetypeStorageError::IntoBufferAddress;

        let (entities, erased_components) = archetype_storage.erased_components(components);
        let len = archetype_storage.len();

        let entities_bytes = must_cast_slice(entities);
        let entities_region = BufferAddress::try_from(entities_bytes.len())
            .map_err(IntoBufferAddress)?
            .try_into()
            .ok()
            .map(|size| StorageBufferRegion { size, offset: 0 });

        let fields = erased_components
            .iter()
            .map(|(_, slice)| slice.descriptor());
        let offsets = storage_buffer_offsets(fields, len, gpu_device)?;

        let mut storage_buffer_contents = Vec::from(entities_bytes);
        let component_regions: IndexMap<_, _> = zip(&erased_components, offsets)
            .map(|((&component_id, slice), offset)| {
                let BufferOffset { offset, .. } = offset?;
                let component_bytes = slice.buffer();

                storage_buffer_contents.resize(offset, 0);
                storage_buffer_contents.extend_from_slice(component_bytes);

                let gpu_component_id = unsafe { GpuComponentId::from_id(component_id) };
                let offset = BufferAddress::try_from(offset).map_err(IntoBufferAddress)?;
                let components_binding = BufferAddress::try_from(component_bytes.len())
                    .map_err(IntoBufferAddress)?
                    .try_into()
                    .ok()
                    .map(|size| StorageBufferRegion { size, offset });
                Ok((gpu_component_id, components_binding))
            })
            .collect::<Result<_, GpuArchetypeStorageError>>()?;
        assert_regions_do_not_overlap(entities_region, component_regions.values().copied());

        let storage_buffer_label = format!("`gpecs` {archetype_id:?} storage buffer");
        let storage_buffer_desc = BufferInitDescriptor {
            label: Some(&storage_buffer_label),
            contents: storage_buffer_contents.as_slice(),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        };
        let storage_buffer = gpu_device.create_buffer_init(&storage_buffer_desc);

        Ok(Self {
            len,
            storage_buffer,
            entities_region,
            component_regions,
        })
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
    pub fn slices(&self) -> GpuArchetypeStorageSlices<'_> {
        let Self {
            storage_buffer,
            entities_region,
            component_regions,
            ..
        } = self;

        let to_slice = slice_from_region(storage_buffer);
        GpuArchetypeStorageSlices {
            entities: entities_region.map(to_slice),
            components: GpuArchetypeStorageComponentSlices {
                storage_buffer,
                inner: component_regions.iter(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum GpuArchetypeStorageError {
    Layout(LayoutError),
    FromBufferAddress(TryFromIntError),
    IntoBufferAddress(TryFromIntError),
}

impl From<LayoutError> for GpuArchetypeStorageError {
    #[inline]
    fn from(err: LayoutError) -> Self {
        Self::Layout(err)
    }
}

impl Display for GpuArchetypeStorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Layout(err) => Display::fmt(err, f),
            Self::FromBufferAddress(_) => write!(f, "couldn't convert buffer address to `usize`"),
            Self::IntoBufferAddress(_) => write!(f, "couldn't convert `usize` to buffer address"),
        }
    }
}

impl Error for GpuArchetypeStorageError {}

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
    storage_buffer: &'a Buffer,
    inner: IndexMapIter<'a, GpuComponentId, Option<StorageBufferRegion>>,
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
        let Self {
            ref mut inner,
            storage_buffer,
        } = *self;

        let to_slice = slice_from_region(storage_buffer);
        inner.next().map(|(&id, region)| (id, region.map(to_slice)))
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
        let Self {
            ref mut inner,
            storage_buffer,
        } = *self;

        let to_slice = slice_from_region(storage_buffer);
        inner
            .nth(n)
            .map(|(&id, &region)| (id, region.map(to_slice)))
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let Self {
            inner,
            storage_buffer,
        } = self;

        let to_slice = slice_from_region(storage_buffer);
        inner
            .last()
            .map(|(&id, &region)| (id, region.map(to_slice)))
    }

    #[inline]
    fn collect<B>(self) -> B
    where
        B: FromIterator<Self::Item>,
    {
        let Self {
            inner,
            storage_buffer,
        } = self;

        let to_slice = slice_from_region(storage_buffer);
        inner
            .map(|(&id, &region)| (id, region.map(to_slice)))
            .collect()
    }
}

impl DoubleEndedIterator for GpuArchetypeStorageComponentSlices<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut inner,
            storage_buffer,
        } = *self;

        let to_slice = slice_from_region(storage_buffer);
        inner
            .next_back()
            .map(|(&id, &region)| (id, region.map(to_slice)))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let Self {
            ref mut inner,
            storage_buffer,
        } = *self;

        let to_slice = slice_from_region(storage_buffer);
        inner
            .nth_back(n)
            .map(|(&id, &region)| (id, region.map(to_slice)))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StorageBufferRegion {
    size: BufferSize,
    offset: BufferAddress,
}

impl From<StorageBufferRegion> for Range<BufferAddress> {
    #[inline]
    fn from(region: StorageBufferRegion) -> Self {
        let StorageBufferRegion { size, offset } = region;
        let end = offset
            .checked_add(size.into())
            .expect("storage buffer region should be valid");
        offset..end
    }
}

#[inline]
fn slice_from_region<'a>(
    storage_buffer: &'a Buffer,
) -> impl FnOnce(StorageBufferRegion) -> GpuArchetypeStorageSlice<'a> + Copy {
    |region| GpuArchetypeStorageSlice {
        slice: storage_buffer.slice(Range::from(region)),
    }
}

#[inline]
fn assert_regions_do_not_overlap<I>(
    entities_region: Option<StorageBufferRegion>,
    component_regions: I,
) where
    I: IntoIterator<Item = Option<StorageBufferRegion>>,
    I::IntoIter: Clone,
{
    let entities_region = entities_region.map(Range::from);
    let component_regions = component_regions
        .into_iter()
        .filter_map(|region| region.map(Range::from));

    chain(entities_region, component_regions)
        .tuple_combinations()
        .for_each(|(lhs, rhs)| assert_ranges_do_not_overlap(lhs, rhs));
}

#[inline]
fn assert_ranges_do_not_overlap(lhs: Range<BufferAddress>, rhs: Range<BufferAddress>) {
    assert!(
        lhs.end <= rhs.start || rhs.end <= lhs.start,
        "storage buffer regions should not overlap, but {lhs:?} and {rhs:?} do overlap",
    );
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
        let item = storage_buffer_offset(desc, layout, capacity, min_offset_align);
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
fn storage_buffer_offset(
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
) -> Result<GpuArchetypeStorageBufferOffsets<I::IntoIter>, GpuArchetypeStorageError>
where
    I: IntoIterator,
    I::Item: AsRef<FieldDescriptor>,
{
    const ENTITY_LAYOUT: Layout = Layout::new::<Entity>();

    let layout = repeat_layout(ENTITY_LAYOUT, capacity)?;
    let fields = fields.into_iter().into();

    let storage_offset_align = device.limits().min_storage_buffer_offset_alignment.into();
    let min_offset_align = BufferAddress::max(storage_offset_align, COPY_BUFFER_ALIGNMENT)
        .try_into()
        .map_err(GpuArchetypeStorageError::FromBufferAddress)?;

    Ok(GpuArchetypeStorageBufferOffsets {
        layout,
        capacity,
        min_offset_align,
        fields,
    })
}
