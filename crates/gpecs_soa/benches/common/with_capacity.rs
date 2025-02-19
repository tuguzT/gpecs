use std::{any::type_name, hint::black_box};

use criterion::{criterion_group, BenchmarkId, Criterion};
use gpecs_soa::prelude::*;

use super::*;

pub(super) trait WithCapacity: Soa {
    fn soa_with_capacity(capacity: usize) -> SoaVec<Self> {
        let capacity = black_box(capacity);
        let vec = SoaVec::<Self>::with_capacity(capacity);
        black_box(vec)
    }

    fn aos_with_capacity(capacity: usize) -> Vec<Self> {
        let capacity = black_box(capacity);
        let vec = Vec::<Self>::with_capacity(capacity);
        black_box(vec)
    }
}

impl WithCapacity for Zero {}

impl WithCapacity for Tiny {}

impl WithCapacity for Small {}

impl WithCapacity for Medium {}

impl WithCapacity for Big {}

impl WithCapacity for Large {}

fn with_capacity<T>(c: &mut Criterion)
where
    T: WithCapacity,
{
    const KB: usize = 1024;
    const CAPACITY_RANGE: [usize; 8] = [0, 1, 10, 100, KB, KB * 2, KB * 4, KB * 8];

    let mut group = c.benchmark_group(format!("With capacity for `{}`", type_name::<T>()));
    for capacity in CAPACITY_RANGE {
        group.bench_with_input(
            BenchmarkId::new(SOA_FUNCTION_NAME, capacity),
            &capacity,
            |b, &capacity| b.iter(|| T::soa_with_capacity(capacity)),
        );
        group.bench_with_input(
            BenchmarkId::new(AOS_FUNCTION_NAME, capacity),
            &capacity,
            |b, &capacity| b.iter(|| T::aos_with_capacity(capacity)),
        );
    }
}

criterion_group!(
    benches,
    with_capacity::<Zero>,
    with_capacity::<Tiny>,
    with_capacity::<Small>,
    with_capacity::<Medium>,
    with_capacity::<Big>,
    with_capacity::<Large>,
);
