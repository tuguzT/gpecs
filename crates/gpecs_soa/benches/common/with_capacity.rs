use std::{any::type_name, hint::black_box};

use criterion::{criterion_group, BenchmarkId, Criterion};
use gpecs_soa::prelude::*;

use super::*;

fn with_capacity<T>(c: &mut Criterion)
where
    T: Soa,
{
    const KB: usize = 1024;
    const CAPACITY_RANGE: [usize; 8] = [0, 1, 10, 100, KB, KB * 2, KB * 4, KB * 8];

    fn soa<T>(capacity: usize)
    where
        T: Soa,
    {
        black_box(SoaVec::<T>::with_capacity(black_box(capacity)));
    }

    fn aos<T>(capacity: usize) {
        black_box(Vec::<T>::with_capacity(black_box(capacity)));
    }

    let mut group = c.benchmark_group(format!("With capacity for `{}`", type_name::<T>()));
    for capacity in CAPACITY_RANGE {
        group.bench_with_input(
            BenchmarkId::new(SOA_FUNCTION_NAME, capacity),
            &capacity,
            |b, &capacity| b.iter(|| soa::<T>(capacity)),
        );
        group.bench_with_input(
            BenchmarkId::new(AOS_FUNCTION_NAME, capacity),
            &capacity,
            |b, &capacity| b.iter(|| aos::<T>(capacity)),
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
