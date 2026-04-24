use std::{
    error::Error,
    fmt::{self, Debug, Display},
    iter::FusedIterator,
    num::NonZeroU32,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use indexmap::map;
use itertools::Itertools;
use num_traits::ToPrimitive;
use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, BufferView, CommandEncoder, Device,
    Features, MapMode, QUERY_SIZE, QuerySet, QuerySetDescriptor, QueryType, Queue,
};

use crate::{
    executor::gpu::{
        archetype::registry::{GpuArchetypeId, GpuArchetypeInfo},
        cache::schedule::{ScheduleCache, SystemCache},
        system::registry::{GpuSystemId, GpuSystemInfo},
    },
    hash::{BuildHasher, IndexMap},
};

#[derive(Debug)]
pub struct TimestampQueryResources {
    query_set: QuerySet,
    count: NonZeroU32,
    resolve_buffer: Buffer,
    download_buffer: Buffer,
    is_ready: Arc<AtomicBool>,
}

impl TimestampQueryResources {
    #[inline]
    pub(super) fn new(gpu_device: &Device, cache: &ScheduleCache) -> Option<Self> {
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
            is_ready: AtomicBool::new(false).into(),
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
    pub fn raw_statistics(&self) -> Result<TimestampQueryRawStatistics, TimestampQueryError> {
        let Self {
            download_buffer,
            is_ready,
            ..
        } = self;

        if !is_ready.load(Ordering::Acquire) {
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
            is_ready,
        } = self;

        command_encoder.resolve_query_set(query_set, 0..count.get(), resolve_buffer, 0);

        if is_ready.swap(false, Ordering::AcqRel) {
            download_buffer.unmap();
        }
        command_encoder.copy_buffer_to_buffer(
            resolve_buffer,
            0,
            download_buffer,
            0,
            resolve_buffer.size(),
        );

        let is_ready = Arc::clone(is_ready);
        let callback = move |_| is_ready.store(true, Ordering::Release);
        command_encoder.map_buffer_on_submit(download_buffer, MapMode::Read, .., callback);
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

pub struct TimestampQueryStatistics {
    stats: IndexMap<GpuSystemId, TimestampQuerySystemStatistics>,
}

impl TimestampQueryStatistics {
    pub(super) fn new(
        raw: &TimestampQueryRawStatistics,
        cache: &ScheduleCache,
        queue: &Queue,
    ) -> Self {
        let mut pairs = raw.as_slice().iter().copied().tuple_windows();

        let hash_builder = BuildHasher::default();
        let mut stats = IndexMap::with_capacity_and_hasher(cache.len(), hash_builder);

        for system_cache in cache.iter() {
            let system_id = system_cache.system_id();
            let system_stats =
                TimestampQuerySystemStatistics::new(system_cache, queue, pairs.by_ref());

            if stats.insert(system_id, system_stats).is_some() {
                unreachable!("{system_id} should be unique in cache")
            }
            pairs.next();
        }
        assert!(pairs.count() == 0);

        Self { stats }
    }

    #[inline]
    pub fn get_system_statistics(
        &self,
        system_id: GpuSystemId,
    ) -> Option<&TimestampQuerySystemStatistics> {
        let Self { stats } = self;
        stats.get(&system_id)
    }

    #[inline]
    pub fn iter(&self) -> TimestampQueryStatisticsIter<'_> {
        let Self { stats } = self;

        let inner = stats.iter();
        TimestampQueryStatisticsIter { inner }
    }
}

impl Debug for TimestampQueryStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TimestampQueryStatistics")
            .field(&self.iter())
            .finish()
    }
}

impl<'a> IntoIterator for &'a TimestampQueryStatistics {
    type Item = GpuSystemInfo<&'a TimestampQuerySystemStatistics>;
    type IntoIter = TimestampQueryStatisticsIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone)]
pub struct TimestampQueryStatisticsIter<'a> {
    inner: map::Iter<'a, GpuSystemId, TimestampQuerySystemStatistics>,
}

impl Debug for TimestampQueryStatisticsIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl<'a> Iterator for TimestampQueryStatisticsIter<'a> {
    type Item = GpuSystemInfo<&'a TimestampQuerySystemStatistics>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next()
            .map(|(&id, stats)| GpuSystemInfo::new(id, stats))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl DoubleEndedIterator for TimestampQueryStatisticsIter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next_back()
            .map(|(&id, stats)| GpuSystemInfo::new(id, stats))
    }
}

impl ExactSizeIterator for TimestampQueryStatisticsIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for TimestampQueryStatisticsIter<'_> {}

pub struct TimestampQuerySystemStatistics {
    stats: IndexMap<GpuArchetypeId, TimestampQueryArchetypeStatistics>,
}

impl TimestampQuerySystemStatistics {
    fn new(
        system_cache: GpuSystemInfo<&SystemCache>,
        queue: &Queue,
        pairs: impl IntoIterator<Item = (u64, u64)>,
    ) -> Self {
        let mut pairs = pairs.into_iter();

        let hash_builder = BuildHasher::default();
        let mut stats = IndexMap::with_capacity_and_hasher(system_cache.len(), hash_builder);

        let timestamp_period_nanos = queue.get_timestamp_period();
        let system_id = system_cache.system_id();
        for archetype_cache in system_cache.iter() {
            let archetype_id = archetype_cache.archetype_id();
            let Some((first, second)) = pairs.next() else {
                unreachable!("item for {system_id} and {archetype_id} should exist")
            };

            let diff = second.saturating_sub(first);
            let nanos = diff.to_f32().unwrap_or_default() * timestamp_period_nanos;
            let duration = Duration::from_nanos(nanos.to_u64().unwrap_or_default());
            let archetype_stats = TimestampQueryArchetypeStatistics {
                first,
                second,
                duration,
            };
            if stats.insert(archetype_id, archetype_stats).is_some() {
                unreachable!("{archetype_id} cannot have multiple items for {system_id}")
            }
        }

        Self { stats }
    }

    #[inline]
    pub fn get_archetype_statistics(
        &self,
        archetype_id: GpuArchetypeId,
    ) -> Option<&TimestampQueryArchetypeStatistics> {
        let Self { stats } = self;
        stats.get(&archetype_id)
    }

    #[inline]
    pub fn iter(&self) -> TimestampQuerySystemStatisticsIter<'_> {
        let Self { stats } = self;

        let inner = stats.iter();
        TimestampQuerySystemStatisticsIter { inner }
    }
}

impl Debug for TimestampQuerySystemStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TimestampQuerySystemStatistics")
            .field(&self.iter())
            .finish()
    }
}

impl<'a> IntoIterator for &'a TimestampQuerySystemStatistics {
    type Item = GpuArchetypeInfo<TimestampQueryArchetypeStatistics>;
    type IntoIter = TimestampQuerySystemStatisticsIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone)]
pub struct TimestampQuerySystemStatisticsIter<'a> {
    inner: map::Iter<'a, GpuArchetypeId, TimestampQueryArchetypeStatistics>,
}

impl Debug for TimestampQuerySystemStatisticsIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.clone();
        f.debug_set().entries(entries).finish()
    }
}

impl Iterator for TimestampQuerySystemStatisticsIter<'_> {
    type Item = GpuArchetypeInfo<TimestampQueryArchetypeStatistics>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next()
            .map(|(&id, &stats)| GpuArchetypeInfo::new(id, stats))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl DoubleEndedIterator for TimestampQuerySystemStatisticsIter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;
        inner
            .next_back()
            .map(|(&id, &stats)| GpuArchetypeInfo::new(id, stats))
    }
}

impl ExactSizeIterator for TimestampQuerySystemStatisticsIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl FusedIterator for TimestampQuerySystemStatisticsIter<'_> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimestampQueryArchetypeStatistics {
    /// First timestamp.
    pub first: u64,
    /// Second timestamp.
    pub second: u64,
    /// Duration between two timestamps above.
    pub duration: Duration,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TimestampQueryError;

impl Display for TimestampQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "executor timestamp query statistics is not ready yet")
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
