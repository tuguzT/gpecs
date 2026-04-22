use std::num::NonZeroU32;

use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, CommandEncoder, Device, Features,
    QUERY_SIZE, QuerySet, QuerySetDescriptor, QueryType,
};

use crate::executor::gpu::cache::{ScheduleCache, SystemCache};

#[derive(Debug)]
pub struct TimestampQueryResources {
    query_set: QuerySet,
    count: NonZeroU32,
    resolve_buffer: Buffer,
}

impl TimestampQueryResources {
    #[inline]
    pub(super) fn new(gpu_device: &Device, schedule_cache: &ScheduleCache) -> Option<Self> {
        let can_write_timestamps = gpu_device
            .features()
            .contains(Features::TIMESTAMP_QUERY_INSIDE_PASSES);
        if !can_write_timestamps {
            return None;
        }

        let count = schedule_cache
            .iter()
            .map(|(_, cache)| timestamp_count_for_system_cache(cache))
            .sum::<usize>()
            .try_into()
            .expect("total timestamp count of schedule cache should fit into `u32`");
        let count = NonZeroU32::new(count)?;

        let query_set_desc = QuerySetDescriptor {
            label: Some("`gpecs` executor query set"),
            ty: QueryType::Timestamp,
            count: count.get(),
        };
        let query_set = gpu_device.create_query_set(&query_set_desc);

        let resolve_buffer_desc = BufferDescriptor {
            label: Some("`gpecs` executor query set resolve buffer"),
            size: resolve_buffer_size(count),
            usage: BufferUsages::QUERY_RESOLVE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        };
        let resolve_buffer = gpu_device.create_buffer(&resolve_buffer_desc);

        Some(TimestampQueryResources {
            query_set,
            count,
            resolve_buffer,
        })
    }

    #[inline]
    pub unsafe fn query_set(&self) -> &QuerySet {
        let Self { query_set, .. } = self;
        query_set
    }

    #[inline]
    pub fn count(&self) -> NonZeroU32 {
        let Self { count, .. } = *self;
        count
    }

    #[inline]
    pub unsafe fn resolve_buffer(&self) -> &Buffer {
        let Self { resolve_buffer, .. } = self;
        resolve_buffer
    }

    #[inline]
    pub(super) fn resolve(&self, command_encoder: &mut CommandEncoder) {
        let Self {
            query_set,
            count,
            resolve_buffer,
        } = self;
        command_encoder.resolve_query_set(query_set, 0..count.get(), resolve_buffer, 0);
    }
}

#[inline]
fn timestamp_count_for_system_cache(system_cache: &SystemCache) -> usize {
    let count = system_cache.len();
    if count == 0 {
        return 0;
    }
    count + 1
}

#[inline]
fn resolve_buffer_size(query_set_count: NonZeroU32) -> BufferAddress {
    // cast operands first to avoid potential `u32` overflow
    let query_set_count = BufferAddress::from(query_set_count.get());
    let query_size = BufferAddress::from(QUERY_SIZE);

    let Some(size) = query_set_count.checked_mul(query_size) else {
        unreachable!("{query_set_count} * `wgpu::QUERY_SIZE` (which is {query_size}) overflow")
    };
    size
}
