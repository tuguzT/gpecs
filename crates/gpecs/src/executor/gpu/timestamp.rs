use std::{
    error::Error,
    fmt::{self, Debug, Display},
    num::NonZeroU32,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, BufferView, CommandEncoder, Device,
    Features, MapMode, QUERY_SIZE, QuerySet, QuerySetDescriptor, QueryType,
};

use crate::executor::gpu::cache::{GpuCache, SystemCache};

#[derive(Debug)]
pub struct TimestampQueryResources {
    query_set: QuerySet,
    count: NonZeroU32,
    resolve_buffer: Buffer,
    download_buffer: Buffer,
    statistics_requested: Arc<AtomicBool>,
}

impl TimestampQueryResources {
    #[inline]
    pub(super) fn new(gpu_device: &Device, cache: &GpuCache) -> Option<Self> {
        let can_write_timestamps = gpu_device
            .features()
            .contains(Features::TIMESTAMP_QUERY_INSIDE_PASSES);
        if !can_write_timestamps {
            return None;
        }

        let count = cache
            .iter()
            .map(|info| timestamp_count_for_system_cache(&info))
            .sum::<usize>()
            .try_into()
            .expect("total timestamp count of schedule cache should fit into `u32`");
        let count = NonZeroU32::new(count)?;

        let query_set_desc = QuerySetDescriptor {
            label: Some("`gpecs` executor timestamp query set"),
            ty: QueryType::Timestamp,
            count: count.get(),
        };
        let query_set = gpu_device.create_query_set(&query_set_desc);

        let size = resolve_buffer_size(count);
        let resolve_buffer_desc = BufferDescriptor {
            label: Some("`gpecs` executor timestamp query resolve buffer"),
            size,
            usage: BufferUsages::QUERY_RESOLVE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        };
        let download_buffer_desc = BufferDescriptor {
            label: Some("`gpecs` executor timestamp query download buffer"),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            ..resolve_buffer_desc
        };

        let resolve_buffer = gpu_device.create_buffer(&resolve_buffer_desc);
        let download_buffer = gpu_device.create_buffer(&download_buffer_desc);

        let me = TimestampQueryResources {
            query_set,
            count,
            resolve_buffer,
            download_buffer,
            statistics_requested: AtomicBool::new(false).into(),
        };
        Some(me)
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
    pub unsafe fn download_buffer(&self) -> &Buffer {
        let Self {
            download_buffer, ..
        } = self;
        download_buffer
    }

    #[inline]
    pub fn request_statistics(&self) {
        let Self {
            download_buffer,
            statistics_requested,
            ..
        } = self;

        let statictics_requested = Arc::clone(statistics_requested);
        let callback = move |_| statictics_requested.store(true, Ordering::Release);
        download_buffer.map_async(MapMode::Read, .., callback);
    }

    #[inline]
    pub fn raw_statistics(&self) -> Result<TimestampQueryRawStatistics, TimestampQueryError> {
        let Self {
            download_buffer,
            statistics_requested,
            ..
        } = self;

        if !statistics_requested.load(Ordering::Acquire) {
            return Err(TimestampQueryError);
        }

        let download_buffer = download_buffer.get_mapped_range(..);
        let statistics = TimestampQueryRawStatistics { download_buffer };
        Ok(statistics)
    }

    #[inline]
    pub(super) fn resolve(&self, command_encoder: &mut CommandEncoder) {
        let Self {
            query_set,
            count,
            resolve_buffer,
            download_buffer,
            statistics_requested,
        } = self;

        command_encoder.resolve_query_set(query_set, 0..count.get(), resolve_buffer, 0);

        if statistics_requested.swap(false, Ordering::AcqRel) {
            download_buffer.unmap();
        }
        command_encoder.copy_buffer_to_buffer(
            resolve_buffer,
            0,
            download_buffer,
            0,
            resolve_buffer.size(),
        );
    }
}

pub struct TimestampQueryRawStatistics {
    download_buffer: BufferView,
}

impl TimestampQueryRawStatistics {
    #[inline]
    pub fn as_slice(&self) -> &[u64] {
        let Self { download_buffer } = self;
        bytemuck::cast_slice(download_buffer)
    }
}

impl Debug for TimestampQueryRawStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slice = self.as_slice();
        f.debug_tuple("TimestampQueryStatistics")
            .field(&slice)
            .finish()
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TimestampQueryError;

impl Display for TimestampQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "executor timestamp query statistics was not requested or is not ready yet"
        )
    }
}

impl Error for TimestampQueryError {}

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
